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
