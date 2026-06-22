//! Serveur **OPC UA** (prototype) : expose le régulateur simulé via un espace
//! d'adressage minimal et route les écritures clients vers la simulation.
//!
//! - Endpoint unique `opc.tcp://<host>:4840/`, sécurité **None** (anonyme) — la
//!   sécurité (certificats, `Basic256Sha256`, auth) viendra en Phase 2.
//! - Lectures : callbacks branchés sur l'instantané partagé (valeurs vivantes,
//!   échantillonnées pour les abonnements).
//! - Écritures : callbacks qui émettent une [`Command`] vers la boucle de
//!   simulation via un canal non bloquant.

use std::sync::{Arc, Mutex};

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
use tokio::sync::mpsc::UnboundedSender;

use crate::sim::{Command, Snapshot};

/// Instantané partagé en lecture avec les callbacks OPC UA.
type Shared = Arc<Mutex<Snapshot>>;
/// Émetteur de commandes vers la boucle de simulation.
type Tx = UnboundedSender<Command>;

/// URI du namespace applicatif (les nœuds métier y vivent).
const NS_URI: &str = "urn:cesam-lab:ru-opcua";
/// Identifiant du jeton utilisateur anonyme (sécurité None).
const ANONYMOUS: &str = "ANONYMOUS";

/// Construit le serveur OPC UA (un seul endpoint, sécurité None).
pub fn build() -> Result<(Server, ServerHandle)> {
    ServerBuilder::new()
        .application_name("CESAM-Lab RU OPC UA (prototype)")
        .application_uri(NS_URI)
        .product_uri(NS_URI)
        .host("0.0.0.0")
        .port(4840)
        .discovery_urls(vec!["opc.tcp://0.0.0.0:4840/".to_owned()])
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
        .create_sample_keypair(false)
        .build()
        .map_err(|e| anyhow!("OPC UA server build failed: {e}"))
}

/// Crée les nœuds Variable et branche les callbacks lecture/écriture.
pub fn install(handle: &ServerHandle, shared: Shared, tx: Tx) -> Result<()> {
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
        add_var(addr, ns, "Setpoint", "Consigne (°C)", true, false);
        add_var(addr, ns, "ProcessValue", "Mesure (°C)", false, false);
        add_var(addr, ns, "Output", "Sortie (%)", false, false);
        add_var(addr, ns, "ManualOutput", "Sortie manuelle (%)", true, false);
        add_var(addr, ns, "Run", "Marche", true, true);
        add_var(addr, ns, "Auto", "Mode automatique", true, true);
    }

    // 2) Lectures : valeurs vivantes issues de l'instantané.
    on_read_f64(&nm, ns, "Setpoint", shared.clone(), |s| s.setpoint as f64);
    on_read_f64(&nm, ns, "ProcessValue", shared.clone(), |s| s.pv as f64);
    on_read_f64(&nm, ns, "Output", shared.clone(), |s| s.output as f64);
    on_read_f64(&nm, ns, "ManualOutput", shared.clone(), |s| s.manual_output as f64);
    on_read_bool(&nm, ns, "Run", shared.clone(), |s| s.run);
    on_read_bool(&nm, ns, "Auto", shared.clone(), |s| s.auto);

    // 3) Écritures : routées vers la simulation.
    on_write_f64(&nm, ns, "Setpoint", tx.clone(), Command::SetSetpoint);
    on_write_f64(&nm, ns, "ManualOutput", tx.clone(), Command::SetManualOutput);
    on_write_bool(&nm, ns, "Run", tx.clone(), Command::SetRun);
    on_write_bool(&nm, ns, "Auto", tx.clone(), Command::SetAuto);

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
fn on_read_f64(nm: &SimpleNodeManager, ns: u16, name: &str, shared: Shared, get: fn(&Snapshot) -> f64) {
    nm.inner().add_read_callback(NodeId::new(ns, name), move |_range, _tss, _max_age| {
        let v = shared.lock().map(|s| get(&s)).unwrap_or(f64::NAN);
        Ok(DataValue::new_now(v))
    });
}

/// Branche un callback de lecture renvoyant un `bool` issu de l'instantané.
fn on_read_bool(nm: &SimpleNodeManager, ns: u16, name: &str, shared: Shared, get: fn(&Snapshot) -> bool) {
    nm.inner().add_read_callback(NodeId::new(ns, name), move |_range, _tss, _max_age| {
        let v = shared.lock().map(|s| get(&s)).unwrap_or(false);
        Ok(DataValue::new_now(v))
    });
}

/// Branche un callback d'écriture `Double` → [`Command`].
fn on_write_f64(nm: &SimpleNodeManager, ns: u16, name: &str, tx: Tx, make: fn(f32) -> Command) {
    nm.inner().add_write_callback(NodeId::new(ns, name), move |dv, _range| {
        let value = match dv.value {
            Some(Variant::Double(d)) => d as f32,
            Some(Variant::Float(f)) => f,
            Some(_) => return StatusCode::BadTypeMismatch,
            None => return StatusCode::BadNothingToDo,
        };
        let _ = tx.send(make(value));
        StatusCode::Good
    });
}

/// Branche un callback d'écriture `Boolean` → [`Command`].
fn on_write_bool(nm: &SimpleNodeManager, ns: u16, name: &str, tx: Tx, make: fn(bool) -> Command) {
    nm.inner().add_write_callback(NodeId::new(ns, name), move |dv, _range| {
        match dv.value {
            Some(Variant::Boolean(b)) => {
                let _ = tx.send(make(b));
                StatusCode::Good
            }
            Some(_) => StatusCode::BadTypeMismatch,
            None => StatusCode::BadNothingToDo,
        }
    });
}
