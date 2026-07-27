//! Acteur réseau : possède le serveur PROFIBUS DP (liaison série) et le
//! (re)démarre à chaud.

use ractor::{Actor, ActorProcessingErr, ActorRef};
use tokio::task::JoinHandle;

use crate::config::{NetworkConfig, ServerStatus};
use crate::trace::SharedTrace;

use super::{SharedSnapshot, SharedStatus, SimulationMsg};

/// Messages de l'acteur réseau.
#[derive(Debug)]
pub enum ProfibusServerMsg {
    /// Applique une nouvelle configuration réseau (port série, débit, adresse de
    /// station, chien de garde).
    Reconfigure(NetworkConfig),
}

/// Arguments de démarrage de l'acteur réseau.
pub struct ProfibusServerArgs {
    pub network: NetworkConfig,
    pub sim: ActorRef<SimulationMsg>,
    pub snapshot: SharedSnapshot,
    pub status: SharedStatus,
    pub trace: SharedTrace,
}

/// État interne de l'acteur réseau.
pub struct ProfibusServerState {
    network: NetworkConfig,
    sim: ActorRef<SimulationMsg>,
    snapshot: SharedSnapshot,
    status: SharedStatus,
    trace: SharedTrace,
    handle: Option<JoinHandle<()>>,
}

impl ProfibusServerState {
    fn set_status(&self, status: ServerStatus) {
        if let Ok(mut s) = self.status.lock() {
            *s = status;
        }
    }

    fn set_listening(&self, addr: String) {
        self.set_status(ServerStatus {
            listening: true,
            addr,
            ..crate::profibus_server::initial_status()
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

    /// (Re)démarre la liaison série PROFIBUS DP.
    fn restart(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
        let desc = self.network.serial.describe();
        match self.network.serial.open() {
            Ok(serial) => {
                let (sim, snapshot, status, trace, station, watchdog_allowed) = (
                    self.sim.clone(),
                    self.snapshot.clone(),
                    self.status.clone(),
                    self.trace.clone(),
                    self.network.serial.station_address,
                    self.network.serial.watchdog_enabled,
                );
                let handle = tokio::spawn(async move {
                    crate::profibus_server::serve_serial(
                        serial,
                        sim,
                        snapshot,
                        status,
                        trace,
                        station,
                        watchdog_allowed,
                    )
                    .await
                });
                self.handle = Some(handle);
                log::info!("PROFIBUS DP serial link on {desc}");
                self.set_listening(format!("Série {desc}"));
            }
            Err(err) => {
                self.set_error(
                    format!("Série {desc}"),
                    format!("failed to open serial port {}: {err}", self.network.serial.port),
                );
            }
        }
    }
}

/// Acteur supervisant le cycle de vie du serveur PROFIBUS DP.
pub struct ProfibusServerActor;

impl Actor for ProfibusServerActor {
    type Msg = ProfibusServerMsg;
    type State = ProfibusServerState;
    type Arguments = ProfibusServerArgs;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let mut state = ProfibusServerState {
            network: args.network,
            sim: args.sim,
            snapshot: args.snapshot,
            status: args.status,
            trace: args.trace,
            handle: None,
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
            ProfibusServerMsg::Reconfigure(cfg) => {
                let unchanged = cfg == state.network;
                state.network = cfg;
                if !unchanged {
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
    use crate::config::SerialConfig;
    use crate::regulator::{Regulator, RegulatorConfig};

    #[tokio::test]
    async fn missing_serial_port_reports_error() {
        let cfg = RegulatorConfig::default();
        let snapshot = Arc::new(Mutex::new(Regulator::new(cfg.clone()).snapshot()));
        let status = Arc::new(Mutex::new(ServerStatus::default()));
        let trace: SharedTrace = Arc::new(Mutex::new(VecDeque::new()));

        let (sim, _sj) = Actor::spawn(None, crate::actors::SimulationActor, crate::actors::SimulationArgs {
            config: cfg,
            snapshot: snapshot.clone(),
        })
        .await
        .unwrap();

        let network = NetworkConfig {
            serial: SerialConfig {
                port: "/dev/cesam_inexistant_42".to_string(),
                ..SerialConfig::default()
            },
        };
        let (net, _nj) = Actor::spawn(None, ProfibusServerActor, ProfibusServerArgs {
            network,
            sim: sim.clone(),
            snapshot,
            status: status.clone(),
            trace,
        })
        .await
        .unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;
        let st = status.lock().unwrap().clone();
        assert!(!st.listening);
        assert!(st.error.is_some());

        net.stop(None);
        sim.stop(None);
    }
}
