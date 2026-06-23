//! Serveur **OPC UA** : expose le régulateur simulé via un espace d'adressage
//! minimal et route les écritures clients vers l'acteur de simulation.
//!
//! - Endpoint `opc.tcp://<bind_ip>:<port>/` ; sécurité **None** anonyme (défaut) ou
//!   **`Basic256Sha256`/SignAndEncrypt** + auth selon [`SecurityConfig`].
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
use opcua::server::{Server, ServerBuilder, ServerHandle, ServerUserToken};
use opcua::types::{
    DataTypeId, DataValue, MessageSecurityMode, NodeId, ObjectId, StatusCode, Variant,
};
use ractor::ActorRef;

use crate::actors::{SharedSnapshot, SimulationMsg};
use crate::config::{NetworkConfig, SecurityConfig};
use crate::regulator::{Command, Snapshot};

/// Référence vers l'acteur de simulation, capturée par les callbacks d'écriture.
type Sim = ActorRef<SimulationMsg>;

/// URI du namespace applicatif (les nœuds métier y vivent).
const NS_URI: &str = "urn:cesam-lab:ru-opcua";
/// URI de l'application serveur. **Distinct de [`NS_URI`]** : l'`application_uri`
/// définit le namespace « local » (index 1) ; le confondre avec `NS_URI` ferait
/// pointer `get_namespace_index(NS_URI)` sur l'index 1 au lieu de celui du node
/// manager → nœuds créés dans le mauvais namespace (`BadNodeIdUnknown`).
const APP_URI: &str = "urn:cesam-lab:ru-opcua-server";
/// Identifiant du jeton utilisateur anonyme.
const ANONYMOUS: &str = "ANONYMOUS";
/// Clé du jeton utilisateur/mot de passe (mode chiffré).
const USER_PASS_ID: &str = "user_pass";

/// Construit le serveur OPC UA selon `network` et `security`.
///
/// - `security.encryption = false` : un endpoint **`None`** anonyme (Phase 1b,
///   démarrage instantané, aucun certificat).
/// - `security.encryption = true` : un endpoint **`Basic256Sha256` / SignAndEncrypt**.
///   Un certificat d'instance auto-signé est généré au premier lancement (`pki/`).
///   Jetons acceptés : anonyme (si `allow_anonymous`) et/ou utilisateur/mot de passe.
pub fn build(network: &NetworkConfig, security: &SecurityConfig) -> Result<(Server, ServerHandle)> {
    let mut builder = ServerBuilder::new()
        .application_name("CESAM-Lab RU OPC UA")
        .application_uri(APP_URI)
        .product_uri(APP_URI)
        .host(network.bind_ip.clone())
        .port(network.port)
        .discovery_urls(vec![network.endpoint_url()])
        .with_node_manager(simple_node_manager(
            NamespaceMetadata {
                namespace_uri: NS_URI.to_owned(),
                ..Default::default()
            },
            "ru-opcua",
        ));

    if security.encryption {
        // Jetons utilisateur acceptés sur l'endpoint chiffré.
        let mut tokens: Vec<&str> = Vec::new();
        if security.allow_anonymous || !security.has_user() {
            tokens.push(ANONYMOUS);
        }
        if security.has_user() {
            builder = builder.add_user_token(
                USER_PASS_ID,
                ServerUserToken::user_pass(security.username.clone(), security.password.clone()),
            );
            tokens.push(USER_PASS_ID);
        }
        builder = builder
            // Certificat d'instance auto-signé (généré au 1er lancement dans `pki/`).
            .create_sample_keypair(true)
            .pki_dir("pki")
            // Simulateur : on fait confiance aux certificats clients (pas de PKI à gérer).
            .trust_client_certs(true)
            .add_endpoint(
                "secure",
                (
                    "/",
                    SecurityPolicy::Basic256Sha256,
                    MessageSecurityMode::SignAndEncrypt,
                    tokens.as_slice(),
                ),
            );
    } else {
        // Phase 1b : endpoint None anonyme, sans certificat (l'ERROR bénin du
        // magasin de certificats est filtré dans `main`).
        builder = builder.create_sample_keypair(false).add_endpoint(
            "none",
            (
                "/",
                SecurityPolicy::None,
                MessageSecurityMode::None,
                &[ANONYMOUS] as &[&str],
            ),
        );
    }

    builder
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

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use ractor::Actor;

    use opcua::client::{ClientBuilder, IdentityToken};
    use opcua::types::{
        AttributeId, DataValue, EndpointDescription, MessageSecurityMode, NodeId, ReadValueId,
        TimestampsToReturn, UserTokenPolicy, Variant, WriteValue,
    };

    use super::{build, install, NS_URI};
    use crate::actors::{SimulationActor, SimulationArgs};
    use crate::config::{NetworkConfig, SecurityConfig};
    use crate::regulator::{Regulator, RegulatorConfig};

    /// Attribue un port TCP libre sur la boucle locale.
    async fn free_port() -> u16 {
        let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        l.local_addr().unwrap().port()
    }

    /// Écriture de la valeur d'un nœud (attribut `Value`).
    fn write_value(ns: u16, name: &str, value: impl Into<Variant>) -> WriteValue {
        WriteValue {
            node_id: NodeId::new(ns, name),
            attribute_id: AttributeId::Value as u32,
            value: DataValue::new_now(value.into()),
            ..Default::default()
        }
    }

    /// Round-trip complet client↔serveur sur l'endpoint **None** (anonyme) :
    /// connexion, écriture de `Run`/`Setpoint`, relecture après application.
    #[tokio::test]
    async fn client_read_write_round_trip_none() {
        let port = free_port().await;
        let cfg = RegulatorConfig::default();
        let snapshot = Arc::new(Mutex::new(Regulator::new(cfg).snapshot()));
        let (sim, _sj) = Actor::spawn(
            None,
            SimulationActor,
            SimulationArgs { config: cfg, snapshot: snapshot.clone() },
        )
        .await
        .unwrap();

        let network = NetworkConfig { bind_ip: "127.0.0.1".to_string(), port };
        let (server, server_handle) = build(&network, &SecurityConfig::default()).unwrap();
        install(&server_handle, snapshot.clone(), sim.clone()).unwrap();
        let server_task = tokio::spawn(async move {
            let _ = server.run().await;
        });
        // Laisse le serveur lier le socket.
        tokio::time::sleep(Duration::from_millis(300)).await;

        let ns = server_handle.get_namespace_index(NS_URI).unwrap();
        let url = network.endpoint_url();

        let mut client = ClientBuilder::new()
            .application_name("ru_opcua-test-client")
            .application_uri("urn:cesam-lab:ru-opcua-test")
            // Endpoint None : aucun certificat client (pas de génération RSA).
            .create_sample_keypair(false)
            .trust_server_certs(true)
            .session_retry_limit(3)
            .client()
            .unwrap();
        let endpoint: EndpointDescription = (
            url.as_str(),
            "None",
            MessageSecurityMode::None,
            UserTokenPolicy::anonymous(),
        )
            .into();
        let (session, event_loop) = client
            .connect_to_matching_endpoint(endpoint, IdentityToken::Anonymous)
            .await
            .unwrap();
        let loop_handle = event_loop.spawn();
        session.wait_for_connection().await;

        // Lecture initiale : la mesure démarre à l'ambiant.
        let pv0 = session
            .read(&[ReadValueId::from(NodeId::new(ns, "ProcessValue"))], TimestampsToReturn::Both, 0.0)
            .await
            .unwrap();
        assert!(matches!(pv0[0].value, Some(Variant::Double(_))), "PV lue : {pv0:?}");

        // Écritures.
        let writes = vec![
            write_value(ns, "Run", true),
            write_value(ns, "Setpoint", 80.0_f64),
        ];
        let results = session.write(&writes).await.unwrap();
        assert!(results.iter().all(|s| s.is_good()), "écritures OK : {results:?}");

        // Laisse l'acteur de simulation appliquer les commandes.
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Relecture.
        let reads = vec![
            ReadValueId::from(NodeId::new(ns, "Setpoint")),
            ReadValueId::from(NodeId::new(ns, "Run")),
        ];
        let values = session.read(&reads, TimestampsToReturn::Both, 0.0).await.unwrap();
        let sp = match &values[0].value {
            Some(Variant::Double(d)) => *d,
            v => panic!("consigne inattendue : {v:?}"),
        };
        let run = match &values[1].value {
            Some(Variant::Boolean(b)) => *b,
            v => panic!("run inattendu : {v:?}"),
        };
        assert!((sp - 80.0).abs() < 1e-6, "consigne relue = {sp}");
        assert!(run, "Run relu = {run}");

        let _ = session.disconnect().await;
        loop_handle.abort();
        server_task.abort();
        sim.stop(None);
    }

    /// Round-trip sur l'endpoint **chiffré** (`Basic256Sha256` / SignAndEncrypt),
    /// jeton anonyme : prouve la génération du certificat serveur et le canal
    /// sécurisé de bout en bout avec un vrai client. **Ignoré par défaut** : la
    /// génération RSA (serveur + client) est lente en *debug*. À lancer
    /// explicitement : `cargo test -p mock_bin_ru_opcua -- --ignored`.
    #[tokio::test]
    #[ignore = "génération RSA lente en debug (cert serveur + keypair client)"]
    async fn client_round_trip_encrypted() {
        let port = free_port().await;
        let cfg = RegulatorConfig::default();
        let snapshot = Arc::new(Mutex::new(Regulator::new(cfg).snapshot()));
        let (sim, _sj) = Actor::spawn(
            None,
            SimulationActor,
            SimulationArgs { config: cfg, snapshot: snapshot.clone() },
        )
        .await
        .unwrap();

        let network = NetworkConfig { bind_ip: "127.0.0.1".to_string(), port };
        let security = SecurityConfig {
            encryption: true,
            ..SecurityConfig::default() // anonyme autorisé, pas d'utilisateur
        };
        let (server, server_handle) = build(&network, &security).unwrap();
        install(&server_handle, snapshot.clone(), sim.clone()).unwrap();
        let server_task = tokio::spawn(async move {
            let _ = server.run().await;
        });
        tokio::time::sleep(Duration::from_millis(500)).await;

        let ns = server_handle.get_namespace_index(NS_URI).unwrap();
        let url = network.endpoint_url();

        let mut client = ClientBuilder::new()
            .application_name("ru_opcua-test-client")
            .application_uri("urn:cesam-lab:ru-opcua-test")
            // Endpoint chiffré : le client a besoin d'un certificat (généré ici).
            .create_sample_keypair(true)
            .trust_server_certs(true)
            .session_retry_limit(3)
            .client()
            .unwrap();
        let endpoint: EndpointDescription = (
            url.as_str(),
            "Basic256Sha256",
            MessageSecurityMode::SignAndEncrypt,
            UserTokenPolicy::anonymous(),
        )
            .into();
        let (session, event_loop) = client
            .connect_to_matching_endpoint(endpoint, IdentityToken::Anonymous)
            .await
            .unwrap();
        let loop_handle = event_loop.spawn();
        session.wait_for_connection().await;

        let results = session
            .write(&[write_value(ns, "Setpoint", 55.0_f64)])
            .await
            .unwrap();
        assert!(results.iter().all(|s| s.is_good()), "écriture OK : {results:?}");

        let values = session
            .read(&[ReadValueId::from(NodeId::new(ns, "Setpoint"))], TimestampsToReturn::Both, 0.0)
            .await
            .unwrap();
        let sp = match &values[0].value {
            Some(Variant::Double(d)) => *d,
            v => panic!("consigne inattendue : {v:?}"),
        };
        assert!((sp - 55.0).abs() < 1e-6, "consigne relue = {sp}");

        let _ = session.disconnect().await;
        loop_handle.abort();
        server_task.abort();
        sim.stop(None);
    }
}
