//! Acteur réseau : possède le serveur **OPC UA** et le (re)démarre à chaud.
//!
//! Le serveur `async-opcua` tourne dans une tâche tokio dédiée (`server.run()`).
//! L'acteur en conserve le `JoinHandle` (abandon à l'arrêt) et le [`ServerHandle`]
//! (annulation propre des sessions). Une reconfiguration de l'IP/port relance le
//! serveur ; les autres réglages (procédé, PID) passent par l'acteur de simulation.

use ractor::{Actor, ActorProcessingErr, ActorRef};
use tokio::task::JoinHandle;

use opcua::server::ServerHandle;

use crate::config::{NetworkConfig, SecurityConfig, ServerStatus};
use crate::opcua_server;

use super::{SharedSnapshot, SharedStatus, SimulationMsg};

/// Messages de l'acteur réseau.
#[derive(Debug)]
pub enum OpcuaServerMsg {
    /// Applique une nouvelle configuration réseau / sécurité (relance si l'IP, le
    /// port ou les paramètres de sécurité changent).
    Reconfigure {
        network: NetworkConfig,
        security: SecurityConfig,
    },
}

/// Arguments de démarrage de l'acteur réseau.
pub struct OpcuaServerArgs {
    pub network: NetworkConfig,
    pub security: SecurityConfig,
    pub sim: ActorRef<SimulationMsg>,
    pub snapshot: SharedSnapshot,
    pub status: SharedStatus,
}

/// État interne de l'acteur réseau.
pub struct OpcuaServerState {
    network: NetworkConfig,
    security: SecurityConfig,
    sim: ActorRef<SimulationMsg>,
    snapshot: SharedSnapshot,
    status: SharedStatus,
    task: Option<JoinHandle<()>>,
    server_handle: Option<ServerHandle>,
}

impl OpcuaServerState {
    fn set_status(&self, status: ServerStatus) {
        if let Ok(mut s) = self.status.lock() {
            *s = status;
        }
    }

    /// Arrête le serveur courant (annulation propre + abandon de la tâche).
    fn stop_current(&mut self) {
        if let Some(handle) = self.server_handle.take() {
            handle.cancel();
        }
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }

    /// (Re)démarre le serveur OPC UA selon la configuration courante.
    fn restart(&mut self) {
        self.stop_current();
        let url = self.network.endpoint_url();
        match opcua_server::build(&self.network, &self.security) {
            Ok((server, handle)) => {
                if let Err(e) = opcua_server::install(&handle, self.snapshot.clone(), self.sim.clone()) {
                    self.set_status(ServerStatus {
                        listening: false,
                        addr: url,
                        error: Some(e.to_string()),
                    });
                    return;
                }
                let task = tokio::spawn(async move {
                    if let Err(e) = server.run().await {
                        log::error!("OPC UA server stopped: {e}");
                    }
                });
                self.task = Some(task);
                self.server_handle = Some(handle);
                let policy = if self.security.encryption {
                    "Basic256Sha256/SignAndEncrypt"
                } else {
                    "None/anonymous"
                };
                log::info!("OPC UA server listening on {url} ({policy})");
                self.set_status(ServerStatus {
                    listening: true,
                    addr: url,
                    error: None,
                });
            }
            Err(e) => {
                log::error!("{e}");
                self.set_status(ServerStatus {
                    listening: false,
                    addr: url,
                    error: Some(e.to_string()),
                });
            }
        }
    }
}

/// Acteur supervisant le cycle de vie du serveur OPC UA.
pub struct OpcuaServerActor;

impl Actor for OpcuaServerActor {
    type Msg = OpcuaServerMsg;
    type State = OpcuaServerState;
    type Arguments = OpcuaServerArgs;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let mut state = OpcuaServerState {
            network: args.network,
            security: args.security,
            sim: args.sim,
            snapshot: args.snapshot,
            status: args.status,
            task: None,
            server_handle: None,
        };
        state.restart();
        Ok(state)
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            OpcuaServerMsg::Reconfigure { network, security } => {
                let changed = network != state.network || security != state.security;
                state.network = network;
                state.security = security;
                if changed {
                    state.restart();
                }
            }
        }
        Ok(())
    }

    async fn post_stop(
        &self,
        _myself: ActorRef<Self::Msg>,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        state.stop_current();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use ractor::Actor;

    use super::*;
    use crate::actors::{SimulationActor, SimulationArgs};
    use crate::config::{NetworkConfig, SecurityConfig, ServerStatus};
    use crate::regulator::{Regulator, RegulatorConfig};

    /// Attribue un port TCP libre sur la boucle locale (puis le relâche).
    async fn free_port() -> u16 {
        let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        l.local_addr().unwrap().port()
    }

    /// Démarre une paire simulation + serveur OPC UA (endpoint None) sur `network`
    /// et renvoie les poignées utiles aux tests de reconfiguration.
    async fn spawn_pair(
        network: NetworkConfig,
    ) -> (ActorRef<SimulationMsg>, ActorRef<OpcuaServerMsg>, SharedStatus) {
        let reg_cfg = RegulatorConfig::default();
        let snapshot = Arc::new(Mutex::new(Regulator::new(reg_cfg).snapshot()));
        let status = Arc::new(Mutex::new(ServerStatus::default()));

        let (sim, _sj) = Actor::spawn(None, SimulationActor, SimulationArgs {
            config: reg_cfg,
            snapshot: snapshot.clone(),
        })
        .await
        .unwrap();

        let (net, _nj) = Actor::spawn(None, OpcuaServerActor, OpcuaServerArgs {
            network,
            security: SecurityConfig::default(),
            sim: sim.clone(),
            snapshot,
            status: status.clone(),
        })
        .await
        .unwrap();

        // Laisse le temps au bind de s'effectuer.
        tokio::time::sleep(Duration::from_millis(200)).await;
        (sim, net, status)
    }

    #[tokio::test]
    async fn opcua_server_actor_binds_and_listens() {
        let port = free_port().await;
        let network = NetworkConfig { bind_ip: "127.0.0.1".to_string(), port };
        let (sim, net, status) = spawn_pair(network).await;

        let st = status.lock().unwrap().clone();
        assert!(st.listening, "le serveur doit écouter (erreur: {:?})", st.error);

        net.stop(None);
        sim.stop(None);
    }

    #[tokio::test]
    async fn reconfigure_same_config_keeps_socket() {
        let port = free_port().await;
        let network = NetworkConfig { bind_ip: "127.0.0.1".to_string(), port };
        let (sim, net, status) = spawn_pair(network.clone()).await;
        let addr_before = status.lock().unwrap().addr.clone();

        // Configuration identique (réseau + sécurité) → aucun rebind attendu.
        net.cast(OpcuaServerMsg::Reconfigure {
            network,
            security: SecurityConfig::default(),
        })
        .unwrap();
        tokio::time::sleep(Duration::from_millis(150)).await;

        let st = status.lock().unwrap().clone();
        assert!(st.listening);
        assert_eq!(st.addr, addr_before, "pas de réouverture du socket attendue");

        net.stop(None);
        sim.stop(None);
    }

    #[tokio::test]
    async fn reconfigure_port_rebinds() {
        let port = free_port().await;
        let network = NetworkConfig { bind_ip: "127.0.0.1".to_string(), port };
        let (sim, net, status) = spawn_pair(network).await;
        let addr_before = status.lock().unwrap().addr.clone();

        // Changement de port → relance du serveur sur la nouvelle adresse.
        let new_port = free_port().await;
        net.cast(OpcuaServerMsg::Reconfigure {
            network: NetworkConfig { bind_ip: "127.0.0.1".to_string(), port: new_port },
            security: SecurityConfig::default(),
        })
        .unwrap();
        tokio::time::sleep(Duration::from_millis(300)).await;

        let st = status.lock().unwrap().clone();
        assert!(st.listening, "doit réécouter après rebind (erreur: {:?})", st.error);
        assert_ne!(st.addr, addr_before, "l'adresse doit refléter le nouveau port");

        net.stop(None);
        sim.stop(None);
    }
}
