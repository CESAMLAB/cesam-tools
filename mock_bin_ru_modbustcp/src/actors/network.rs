//! Acteur réseau : possède le serveur Modbus TCP et le (re)démarre à chaud.
//!
//! Le changement de port ou d'adresse d'écoute impose de réouvrir le socket : on
//! arrête la tâche de service en cours et on en relance une. La liste blanche
//! d'IP, elle, est partagée et appliquée sans redémarrage.

use ractor::{Actor, ActorProcessingErr, ActorRef};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use crate::config::{IpFilter, NetworkConfig, ServerStatus, Transport};
use crate::modbus_server::{serve, RegulatorService};

use super::{SharedAllowlist, SharedMap, SharedSnapshot, SharedStatus, SimulationMsg};

/// Messages de l'acteur réseau.
#[derive(Debug)]
pub enum ModbusServerMsg {
    /// Applique une nouvelle configuration réseau (port, IP d'écoute, liste blanche).
    Reconfigure(NetworkConfig),
}

/// Arguments de démarrage de l'acteur réseau.
pub struct ModbusServerArgs {
    pub network: NetworkConfig,
    pub sim: ActorRef<SimulationMsg>,
    pub map: SharedMap,
    pub snapshot: SharedSnapshot,
    pub allowlist: SharedAllowlist,
    pub status: SharedStatus,
}

/// État interne de l'acteur réseau.
pub struct ModbusServerState {
    network: NetworkConfig,
    sim: ActorRef<SimulationMsg>,
    map: SharedMap,
    snapshot: SharedSnapshot,
    allowlist: SharedAllowlist,
    status: SharedStatus,
    handle: Option<JoinHandle<std::io::Result<()>>>,
}

impl ModbusServerState {
    /// Recopie la liste blanche courante dans la structure partagée.
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

    /// Publie un statut « à l'écoute » sur l'adresse `addr` (succès de démarrage).
    /// Remet à zéro le maître connecté et l'activité (nouveau cycle d'écoute).
    fn set_listening(&self, addr: String) {
        self.set_status(ServerStatus {
            listening: true,
            addr,
            error: None,
            ..ServerStatus::default()
        });
    }

    /// Journalise et publie une erreur de démarrage du transport sur `addr`.
    fn set_error(&self, addr: String, error: String) {
        log::error!("{error}");
        self.set_status(ServerStatus {
            listening: false,
            addr,
            error: Some(error),
            ..ServerStatus::default()
        });
    }

    /// Construit le service Modbus partagé (lectures sur la map, écritures vers l'acteur).
    fn make_service(&self) -> RegulatorService {
        RegulatorService::new(
            self.sim.clone(),
            self.map.clone(),
            self.snapshot.clone(),
            self.status.clone(),
        )
    }

    /// (Re)démarre le serveur Modbus selon le transport configuré.
    async fn restart(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
        self.apply_allowlist();

        match self.network.transport {
            Transport::Tcp => self.start_tcp().await,
            Transport::Rtu => self.start_rtu(),
        }
    }

    /// Démarre le serveur Modbus TCP (politique mono-maître dans [`serve`]).
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
                let service = self.make_service();
                let allowlist = self.allowlist.clone();
                let status = self.status.clone();
                let handle =
                    tokio::spawn(async move { serve(listener, service, allowlist, status).await });
                self.handle = Some(handle);
                log::info!("Modbus TCP server listening on {addr}");
                self.set_listening(addr.to_string());
            }
            Err(err) => {
                self.set_error(addr.to_string(), format!("failed to listen on {addr}: {err}"));
            }
        }
    }

    /// Démarre le serveur Modbus RTU sur la liaison série (feature `rtu`).
    fn start_rtu(&mut self) {
        let desc = self.network.serial.describe();
        #[cfg(feature = "rtu")]
        {
            match self.network.serial.open() {
                Ok(serial) => {
                    let service = self.make_service();
                    let handle = tokio::spawn(async move {
                        crate::modbus_server::serve_rtu(serial, service).await
                    });
                    self.handle = Some(handle);
                    log::info!("Modbus RTU server on {desc}");
                    log::warn!(
                        "RTU: the device responds regardless of the slave address \
                         (point-to-point link recommended)"
                    );
                    self.set_listening(format!("RTU {desc}"));
                }
                Err(err) => {
                    self.set_error(
                        format!("RTU {desc}"),
                        format!("failed to open serial port {}: {err}", self.network.serial.port),
                    );
                }
            }
        }
        #[cfg(not(feature = "rtu"))]
        {
            self.set_error(
                format!("RTU {desc}"),
                "RTU transport requested but binary built without the `rtu` feature".to_string(),
            );
        }
    }
}

/// Acteur supervisant le cycle de vie du serveur Modbus TCP.
pub struct ModbusServerActor;

impl Actor for ModbusServerActor {
    type Msg = ModbusServerMsg;
    type State = ModbusServerState;
    type Arguments = ModbusServerArgs;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let mut state = ModbusServerState {
            network: args.network,
            sim: args.sim,
            map: args.map,
            snapshot: args.snapshot,
            allowlist: args.allowlist,
            status: args.status,
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
            ModbusServerMsg::Reconfigure(cfg) => {
                // Tout changement autre que la liste blanche impose de réouvrir le
                // transport (changement de bus, de port TCP/série, de paramètres série).
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
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use ractor::Actor;

    use super::*;
    use crate::config::{IpFilter, NetworkConfig, ServerStatus};
    use crate::map::MemoryMap;
    use crate::regulator::{Regulator, RegulatorConfig};

    #[tokio::test]
    async fn modbus_server_actor_binds_and_listens() {
        let reg_cfg = RegulatorConfig::default();
        let snapshot = Arc::new(Mutex::new(Regulator::new(reg_cfg.clone()).snapshot()));
        let map = Arc::new(Mutex::new(MemoryMap::default()));
        let allowlist = Arc::new(Mutex::new(IpFilter::default()));
        let status = Arc::new(Mutex::new(ServerStatus::default()));

        let (sim, _sj) = Actor::spawn(None, crate::actors::SimulationActor, crate::actors::SimulationArgs {
            config: reg_cfg,
            snapshot: snapshot.clone(),
            map: map.clone(),
        })
        .await
        .unwrap();

        // Port 0 = port éphémère attribué par l'OS.
        let network = NetworkConfig {
            bind_ip: "127.0.0.1".to_string(),
            port: 0,
            ..NetworkConfig::default()
        };
        let (net, _nj) = Actor::spawn(None, ModbusServerActor, ModbusServerArgs {
            network,
            sim: sim.clone(),
            map,
            snapshot,
            allowlist,
            status: status.clone(),
        })
        .await
        .unwrap();

        // Laisse le temps au bind de s'effectuer.
        tokio::time::sleep(Duration::from_millis(100)).await;
        let st = status.lock().unwrap().clone();
        assert!(st.listening, "le serveur doit écouter (erreur: {:?})", st.error);

        net.stop(None);
        sim.stop(None);
    }

    /// Démarre une paire simulation + serveur réseau sur `network` et renvoie les
    /// poignées utiles aux tests de reconfiguration.
    async fn spawn_pair(
        network: NetworkConfig,
    ) -> (
        ActorRef<SimulationMsg>,
        ActorRef<ModbusServerMsg>,
        SharedStatus,
        SharedAllowlist,
    ) {
        let reg_cfg = RegulatorConfig::default();
        let snapshot = Arc::new(Mutex::new(Regulator::new(reg_cfg.clone()).snapshot()));
        let map = Arc::new(Mutex::new(MemoryMap::default()));
        let allowlist = Arc::new(Mutex::new(IpFilter::default()));
        let status = Arc::new(Mutex::new(ServerStatus::default()));

        let (sim, _sj) = Actor::spawn(None, crate::actors::SimulationActor, crate::actors::SimulationArgs {
            config: reg_cfg,
            snapshot: snapshot.clone(),
            map: map.clone(),
        })
        .await
        .unwrap();

        let (net, _nj) = Actor::spawn(None, ModbusServerActor, ModbusServerArgs {
            network,
            sim: sim.clone(),
            map,
            snapshot,
            allowlist: allowlist.clone(),
            status: status.clone(),
        })
        .await
        .unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;
        (sim, net, status, allowlist)
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
        net.cast(ModbusServerMsg::Reconfigure(cfg)).unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        let st = status.lock().unwrap().clone();
        assert!(st.listening);
        assert_eq!(st.addr, addr_before, "pas de réouverture du socket attendue");
        // Le filtre est appliqué à chaud, sans redémarrage.
        let f = allowlist.lock().unwrap();
        assert!(f.allows("10.0.0.1".parse().unwrap()));
        assert!(!f.allows("8.8.8.8".parse().unwrap()));

        net.stop(None);
        sim.stop(None);
    }

    #[tokio::test]
    async fn reconfigure_rebinds_on_port_change() {
        // Découvre un port libre, puis demande une reconfiguration vers ce port.
        let tmp = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
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
        net.cast(ModbusServerMsg::Reconfigure(cfg)).unwrap();
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
