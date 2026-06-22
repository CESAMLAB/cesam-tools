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

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use ractor::Actor;

    use super::*;
    use crate::config::{IpFilter, NetworkConfig, ServerStatus};
    use crate::stirrer::{Stirrer, StirrerConfig};

    /// Démarre une paire simulation + serveur NAMUR sur `network` et renvoie les
    /// poignées utiles aux tests de reconfiguration.
    async fn spawn_pair(
        network: NetworkConfig,
    ) -> (
        ActorRef<SimulationMsg>,
        ActorRef<NamurServerMsg>,
        SharedStatus,
        SharedAllowlist,
    ) {
        let cfg = StirrerConfig::default();
        let snapshot = Arc::new(Mutex::new(Stirrer::new(cfg.clone()).snapshot()));
        let allowlist = Arc::new(Mutex::new(IpFilter::default()));
        let status = Arc::new(Mutex::new(ServerStatus::default()));
        let trace: SharedTrace = Arc::new(Mutex::new(VecDeque::new()));

        let (sim, _sj) = Actor::spawn(None, crate::actors::SimulationActor, crate::actors::SimulationArgs {
            config: cfg,
            snapshot: snapshot.clone(),
        })
        .await
        .unwrap();

        let (net, _nj) = Actor::spawn(None, NamurServerActor, NamurServerArgs {
            network,
            sim: sim.clone(),
            snapshot,
            allowlist: allowlist.clone(),
            status: status.clone(),
            trace,
        })
        .await
        .unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;
        (sim, net, status, allowlist)
    }

    #[tokio::test]
    async fn namur_actor_binds_and_listens() {
        let network = NetworkConfig {
            bind_ip: "127.0.0.1".to_string(),
            port: 0, // port éphémère attribué par l'OS
            ..NetworkConfig::default()
        };
        let (sim, net, status, _allow) = spawn_pair(network).await;
        let st = status.lock().unwrap().clone();
        assert!(st.listening, "le serveur doit écouter (erreur: {:?})", st.error);
        net.stop(None);
        sim.stop(None);
    }

    #[tokio::test]
    async fn reconfigure_allowlist_only_keeps_socket_and_applies_filter() {
        let network = NetworkConfig {
            bind_ip: "127.0.0.1".to_string(),
            port: 0,
            ..NetworkConfig::default()
        };
        let (sim, net, status, allowlist) = spawn_pair(network.clone()).await;

        let addr_before = status.lock().unwrap().addr.clone();
        assert!(allowlist.lock().unwrap().allows("8.8.8.8".parse().unwrap()));

        // Même transport/IP/port (0), seule la liste blanche change : aucun rebind.
        let cfg = NetworkConfig {
            allowlist: vec!["10.0.0.1".to_string()],
            ..network
        };
        net.cast(NamurServerMsg::Reconfigure(cfg)).unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        let st = status.lock().unwrap().clone();
        assert!(st.listening);
        assert_eq!(st.addr, addr_before, "pas de réouverture du socket attendue");
        let f = allowlist.lock().unwrap();
        assert!(f.allows("10.0.0.1".parse().unwrap()));
        assert!(!f.allows("8.8.8.8".parse().unwrap()));

        net.stop(None);
        sim.stop(None);
    }

    #[tokio::test]
    async fn reconfigure_rebinds_on_port_change() {
        // Découvre un port libre, puis demande une reconfiguration vers ce port.
        let tmp = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let free_port = tmp.local_addr().unwrap().port();
        drop(tmp);

        let network = NetworkConfig {
            bind_ip: "127.0.0.1".to_string(),
            port: 0,
            ..NetworkConfig::default()
        };
        let (sim, net, status, _allow) = spawn_pair(network).await;
        let addr_before = status.lock().unwrap().addr.clone();

        let cfg = NetworkConfig {
            bind_ip: "127.0.0.1".to_string(),
            port: free_port,
            ..NetworkConfig::default()
        };
        net.cast(NamurServerMsg::Reconfigure(cfg)).unwrap();
        tokio::time::sleep(Duration::from_millis(150)).await;

        let st = status.lock().unwrap().clone();
        assert!(st.listening, "le serveur doit réécouter (erreur: {:?})", st.error);
        assert_ne!(st.addr, addr_before, "le socket doit avoir été rouvert");
        assert!(
            st.addr.ends_with(&format!(":{free_port}")),
            "doit écouter sur le nouveau port {free_port} (addr={})",
            st.addr
        );

        net.stop(None);
        sim.stop(None);
    }
}
