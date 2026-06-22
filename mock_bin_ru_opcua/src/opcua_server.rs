//! Serveur **OPC UA** : expose le régulateur simulé via un espace d'adressage
//! minimal et route les écritures clients vers l'acteur de simulation.
//!
//! - Endpoint unique `opc.tcp://<bind_ip>:<port>/`, sécurité **None** (anonyme) —
//!   la sécurité (certificats, `Basic256Sha256`, auth) viendra en Phase 2.
//! - Lectures : callbacks branchés sur l'instantané partagé (valeurs vivantes,
//!   échantillonnées pour les abonnements).
//! - Écritures : callbacks qui émettent une [`Command`] vers l'acteur de
//!   simulation par `cast` non bloquant.

use anyhow::{anyhow, Result};
use opcua::crypto::SecurityPolicy;
use opcua::nodes::VariableBuilder;
use opcua::server::address_space::AddressSpace;
use opcua::server::diagnostics::NamespaceMetadata;
use opcua::server::node_manager::memory::{simple_node_manager, SimpleNodeManager};
use opcua::server::{Server, ServerBuilder, ServerHandle};
use opcua::types::{
    DataTypeId, DataValue, MessageSecurityMode, NodeId, ObjectId, StatusCode, Variant,
};
use ractor::ActorRef;

use crate::actors::{SharedSnapshot, SimulationMsg};
use crate::config::NetworkConfig;
use crate::regulator::{Command, Snapshot};

/// Référence vers l'acteur de simulation, capturée par les callbacks d'écriture.
type Sim = ActorRef<SimulationMsg>;

/// URI du namespace applicatif (les nœuds métier y vivent).
const NS_URI: &str = "urn:cesam-lab:ru-opcua";
/// Identifiant du jeton utilisateur anonyme (sécurité None).
const ANONYMOUS: &str = "ANONYMOUS";

/// Construit le serveur OPC UA (un seul endpoint, sécurité None) selon `network`.
pub fn build(network: &NetworkConfig) -> Result<(Server, ServerHandle)> {
    ServerBuilder::new()
        .application_name("CESAM-Lab RU OPC UA")
        .application_uri(NS_URI)
        .product_uri(NS_URI)
        .host(network.bind_ip.clone())
        .port(network.port)
        .discovery_urls(vec![network.endpoint_url()])
        .with_node_manager(simple_node_manager(
            NamespaceMetadata {
                namespace_uri: NS_URI.to_owned(),
                ..Default::default()
            },
            "ru-opcua",
        ))
        .add_endpoint(
            "none",
            (
                "/",
                SecurityPolicy::None,
                MessageSecurityMode::None,
                &[ANONYMOUS] as &[&str],
            ),
        )
        .trust_client_certs(false)
        // Endpoint None uniquement (Phase 1b) : pas de certificat d'instance. On
        // n'en génère PAS (la génération RSA en Rust pur est très lente en debug et
        // écrirait dans `pki/`). La vraie sécurité (endpoints chiffrés, certs) =
        // Phase 2. L'ERROR bénin du magasin de certificats est filtré dans `main`.
        .create_sample_keypair(false)
        .build()
        .map_err(|e| anyhow!("OPC UA server build failed: {e}"))
}

/// Crée les nœuds Variable et branche les callbacks lecture/écriture.
pub fn install(handle: &ServerHandle, snapshot: SharedSnapshot, sim: Sim) -> Result<()> {
    let nm = handle
        .node_managers()
        .get_of_type::<SimpleNodeManager>()
        .ok_or_else(|| anyhow!("simple node manager not found"))?;
    let ns = handle
        .get_namespace_index(NS_URI)
        .ok_or_else(|| anyhow!("namespace {NS_URI} not registered"))?;

    // 1) Déclaration des nœuds dans l'espace d'adressage.
    {
        let mut guard = nm.address_space().write();
        let addr = &mut *guard;
        add_var(addr, ns, "Setpoint", "Consigne", true, false);
        add_var(addr, ns, "ProcessValue", "Mesure", false, false);
        add_var(addr, ns, "Output", "Sortie (%)", false, false);
        add_var(addr, ns, "ManualOutput", "Sortie manuelle (%)", true, false);
        add_var(addr, ns, "Run", "Marche", true, true);
        add_var(addr, ns, "Auto", "Mode automatique", true, true);
    }

    // 2) Lectures : valeurs vivantes issues de l'instantané.
    on_read_f64(&nm, ns, "Setpoint", snapshot.clone(), |s| s.setpoint as f64);
    on_read_f64(&nm, ns, "ProcessValue", snapshot.clone(), |s| s.pv as f64);
    on_read_f64(&nm, ns, "Output", snapshot.clone(), |s| s.output as f64);
    on_read_f64(&nm, ns, "ManualOutput", snapshot.clone(), |s| s.manual_output as f64);
    on_read_bool(&nm, ns, "Run", snapshot.clone(), |s| s.run);
    on_read_bool(&nm, ns, "Auto", snapshot.clone(), |s| s.auto);

    // 3) Écritures : routées vers l'acteur de simulation.
    on_write_f64(&nm, ns, "Setpoint", sim.clone(), Command::SetSetpoint);
    on_write_f64(&nm, ns, "ManualOutput", sim.clone(), Command::SetManualOutput);
    on_write_bool(&nm, ns, "Run", sim.clone(), Command::SetRun);
    on_write_bool(&nm, ns, "Auto", sim.clone(), Command::SetAuto);

    Ok(())
}

/// Ajoute un nœud Variable (`Double` ou `Boolean`) organisé sous `Objects`.
fn add_var(addr: &mut AddressSpace, ns: u16, name: &str, display: &str, writable: bool, is_bool: bool) {
    let id = NodeId::new(ns, name);
    let (data_type, init): (NodeId, Variant) = if is_bool {
        (DataTypeId::Boolean.into(), false.into())
    } else {
        (DataTypeId::Double.into(), 0.0_f64.into())
    };
    let mut builder = VariableBuilder::new(&id, name, display)
        .data_type(data_type)
        .value(init)
        .organized_by(ObjectId::ObjectsFolder);
    if writable {
        builder = builder.writable();
    }
    builder.insert(addr);
}

/// Branche un callback de lecture renvoyant un `f64` issu de l'instantané.
fn on_read_f64(nm: &SimpleNodeManager, ns: u16, name: &str, snapshot: SharedSnapshot, get: fn(&Snapshot) -> f64) {
    nm.inner().add_read_callback(NodeId::new(ns, name), move |_range, _tss, _max_age| {
        let v = snapshot.lock().map(|s| get(&s)).unwrap_or(f64::NAN);
        Ok(DataValue::new_now(v))
    });
}

/// Branche un callback de lecture renvoyant un `bool` issu de l'instantané.
fn on_read_bool(nm: &SimpleNodeManager, ns: u16, name: &str, snapshot: SharedSnapshot, get: fn(&Snapshot) -> bool) {
    nm.inner().add_read_callback(NodeId::new(ns, name), move |_range, _tss, _max_age| {
        let v = snapshot.lock().map(|s| get(&s)).unwrap_or(false);
        Ok(DataValue::new_now(v))
    });
}

/// Branche un callback d'écriture `Double` → [`Command`] castée vers la simulation.
fn on_write_f64(nm: &SimpleNodeManager, ns: u16, name: &str, sim: Sim, make: fn(f32) -> Command) {
    nm.inner().add_write_callback(NodeId::new(ns, name), move |dv, _range| {
        let value = match dv.value {
            Some(Variant::Double(d)) => d as f32,
            Some(Variant::Float(f)) => f,
            Some(_) => return StatusCode::BadTypeMismatch,
            None => return StatusCode::BadNothingToDo,
        };
        let _ = sim.cast(SimulationMsg::Command(make(value)));
        StatusCode::Good
    });
}

/// Branche un callback d'écriture `Boolean` → [`Command`] castée vers la simulation.
fn on_write_bool(nm: &SimpleNodeManager, ns: u16, name: &str, sim: Sim, make: fn(bool) -> Command) {
    nm.inner().add_write_callback(NodeId::new(ns, name), move |dv, _range| {
        match dv.value {
            Some(Variant::Boolean(b)) => {
                let _ = sim.cast(SimulationMsg::Command(make(b)));
                StatusCode::Good
            }
            Some(_) => StatusCode::BadTypeMismatch,
            None => StatusCode::BadNothingToDo,
        }
    });
}
