//! Acteur réseau : possède le serveur NAMUR (TCP ou série) et le (re)démarre à chaud.

use ractor::{Actor, ActorProcessingErr, ActorRef};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use crate::config::{IpFilter, NetworkConfig, ServerStatus, Transport};
use crate::namur_server::serve_tcp;
use crate::trace::SharedTrace;

use super::{SharedAllowlist, SharedSnapshot, SharedStatus, SimulationMsg};

/// Messages de l'acteur réseau.
#[derive(Debug)]
pub enum NamurServerMsg {
    /// Applique une nouvelle configuration réseau (transport, port, IP, liste blanche, série).
    Reconfigure(NetworkConfig),
}

/// Arguments de démarrage de l'acteur réseau.
pub struct NamurServerArgs {
    pub network: NetworkConfig,
    pub sim: ActorRef<SimulationMsg>,
    pub snapshot: SharedSnapshot,
    pub allowlist: SharedAllowlist,
    pub status: SharedStatus,
    pub trace: SharedTrace,
}

/// État interne de l'acteur réseau.
pub struct NamurServerState {
    network: NetworkConfig,
    sim: ActorRef<SimulationMsg>,
    snapshot: SharedSnapshot,
    allowlist: SharedAllowlist,
    status: SharedStatus,
    trace: SharedTrace,
    handle: Option<JoinHandle<std::io::Result<()>>>,
}

impl NamurServerState {
    fn apply_allowlist(&self) {
        if let Ok(mut f) = self.allowlist.lock() {
            *f = IpFilter::new(self.network.allowlist.clone());
        }
    }

    fn set_status(&self, status: ServerStatus) {
        if let Ok(mut s) = self.status.lock() {
            *s = status;
        }
    }

    fn set_listening(&self, addr: String) {
        self.set_status(ServerStatus {
            listening: true,
            addr,
            ..ServerStatus::default()
        });
    }

    fn set_error(&self, addr: String, error: String) {
        log::error!("{error}");
        self.set_status(ServerStatus {
            listening: false,
            addr,
            error: Some(error),
            ..ServerStatus::default()
        });
    }

    /// (Re)démarre le serveur selon le transport configuré.
    async fn restart(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
        self.apply_allowlist();
        match self.network.transport {
            Transport::Tcp => self.start_tcp().await,
            Transport::Serial => self.start_serial(),
        }
    }

    async fn start_tcp(&mut self) {
        let addr = match self.network.socket_addr() {
            Ok(addr) => addr,
            Err(err) => {
                self.set_error(format!("{}:{}", self.network.bind_ip, self.network.port), err);
                return;
            }
        };
        match TcpListener::bind(addr).await {
            Ok(listener) => {
                let (sim, snapshot, status, allowlist, trace) = (
                    self.sim.clone(),
                    self.snapshot.clone(),
                    self.status.clone(),
                    self.allowlist.clone(),
                    self.trace.clone(),
                );
                let handle = tokio::spawn(async move {
                    serve_tcp(listener, sim, snapshot, status, allowlist, trace).await
                });
                self.handle = Some(handle);
                log::info!("NAMUR TCP server listening on {addr}");
                self.set_listening(addr.to_string());
            }
            Err(err) => {
                self.set_error(addr.to_string(), format!("failed to listen on {addr}: {err}"));
            }
        }
    }

    fn start_serial(&mut self) {
        let desc = self.network.serial.describe();
        #[cfg(feature = "serial")]
        {
            match self.network.serial.open() {
                Ok(serial) => {
                    let (sim, snapshot, status, trace) = (
                        self.sim.clone(),
                        self.snapshot.clone(),
                        self.status.clone(),
                        self.trace.clone(),
                    );
                    let handle = tokio::spawn(async move {
                        crate::namur_server::serve_serial(serial, sim, snapshot, status, trace).await
                    });
                    self.handle = Some(handle);
                    log::info!("NAMUR serial server on {desc}");
                    self.set_listening(format!("Serial {desc}"));
                }
                Err(err) => {
                    self.set_error(
                        format!("Serial {desc}"),
                        format!("failed to open serial port {}: {err}", self.network.serial.port),
                    );
                }
            }
        }
        #[cfg(not(feature = "serial"))]
        {
            self.set_error(
                format!("Serial {desc}"),
                "serial transport requested but binary built without the `serial` feature".to_string(),
            );
        }
    }
}

/// Acteur supervisant le cycle de vie du serveur NAMUR.
pub struct NamurServerActor;

impl Actor for NamurServerActor {
    type Msg = NamurServerMsg;
    type State = NamurServerState;
    type Arguments = NamurServerArgs;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let mut state = NamurServerState {
            network: args.network,
            sim: args.sim,
            snapshot: args.snapshot,
            allowlist: args.allowlist,
            status: args.status,
            trace: args.trace,
            handle: None,
        };
        state.restart().await;
        Ok(state)
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            NamurServerMsg::Reconfigure(cfg) => {
                let allowlist_only = cfg.transport == state.network.transport
                    && cfg.bind_ip == state.network.bind_ip
                    && cfg.port == state.network.port
                    && cfg.serial == state.network.serial;
                state.network = cfg;
                if allowlist_only {
                    state.apply_allowlist();
                } else {
                    state.restart().await;
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
        if let Some(handle) = state.handle.take() {
            handle.abort();
        }
        Ok(())
    }
}
