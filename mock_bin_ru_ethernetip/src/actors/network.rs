//! Acteur réseau : possède le **serveur EtherNet/IP** et le (re)démarre à chaud.
//!
//! Le serveur écoute en TCP (port 44818) et traite **plusieurs sessions clientes
//! simultanées** (comportement usuel d'un adaptateur ; pas de mono-maître). La boucle
//! d'acceptation tourne dans une tâche tokio dédiée dont l'acteur conserve le
//! `JoinHandle` ; ses sessions sont portées par un `JoinSet` **interne** (toutes
//! abattues avec la boucle — aucune tâche détachée laissée derrière).
//!
//! La **liste blanche d'IP** est partagée ([`SharedAllowlist`]) : une reconfiguration
//! met à jour le filtre des **nouvelles** connexions et relance l'écoute si l'IP/port
//! change.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use ractor::{Actor, ActorProcessingErr, ActorRef};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::{JoinHandle, JoinSet};

use crate::config::{Allowlist, NetworkConfig, ServerStatus};
use crate::eip_server;
use mock_lib_regulator::Snapshot;

use super::{SharedAllowlist, SharedSnapshot, SharedStatus, SimulationMsg};

/// En-tête d'encapsulation EtherNet/IP (octets).
const ENCAP_HEADER_LEN: usize = 24;
/// Taille maximale de données acceptée après l'en-tête (garde-fou réseau).
const MAX_DATA: usize = 65535;

/// Messages de l'acteur réseau.
#[derive(Debug)]
pub enum EipServerMsg {
    /// Applique une nouvelle configuration réseau (relance si l'IP ou le port
    /// change ; met à jour la liste blanche dans tous les cas).
    Reconfigure { network: NetworkConfig },
}

/// Arguments de démarrage de l'acteur réseau.
pub struct EipServerArgs {
    pub network: NetworkConfig,
    pub sim: ActorRef<SimulationMsg>,
    pub snapshot: SharedSnapshot,
    pub status: SharedStatus,
}

/// État interne de l'acteur réseau.
pub struct EipServerState {
    network: NetworkConfig,
    sim: ActorRef<SimulationMsg>,
    snapshot: SharedSnapshot,
    status: SharedStatus,
    allowlist: SharedAllowlist,
    task: Option<JoinHandle<()>>,
}

impl EipServerState {
    fn stop_current(&mut self) {
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }

    fn restart(&mut self) {
        self.stop_current();
        if let Ok(mut a) = self.allowlist.lock() {
            *a = Allowlist::new(self.network.allowlist.clone());
        }
        let params = ListenerParams {
            addr: self.network.listen_addr(),
            sim: self.sim.clone(),
            snapshot: self.snapshot.clone(),
            status: self.status.clone(),
            allowlist: self.allowlist.clone(),
        };
        self.task = Some(tokio::spawn(run_listener(params)));
    }
}

/// Acteur supervisant le cycle de vie du serveur EtherNet/IP.
pub struct EipServerActor;

impl Actor for EipServerActor {
    type Msg = EipServerMsg;
    type State = EipServerState;
    type Arguments = EipServerArgs;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let allowlist = Arc::new(Mutex::new(Allowlist::new(args.network.allowlist.clone())));
        let mut state = EipServerState {
            network: args.network,
            sim: args.sim,
            snapshot: args.snapshot,
            status: args.status,
            allowlist,
            task: None,
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
            EipServerMsg::Reconfigure { network } => {
                let rebind = network.bind_ip != state.network.bind_ip || network.port != state.network.port;
                state.network = network;
                if rebind {
                    state.restart();
                } else if let Ok(mut a) = state.allowlist.lock() {
                    *a = Allowlist::new(state.network.allowlist.clone());
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

struct ListenerParams {
    addr: String,
    sim: ActorRef<SimulationMsg>,
    snapshot: SharedSnapshot,
    status: SharedStatus,
    allowlist: SharedAllowlist,
}

fn current_snapshot(snapshot: &SharedSnapshot) -> Snapshot {
    match snapshot.lock() {
        Ok(g) => *g,
        Err(_) => mock_lib_regulator::Regulator::default().snapshot(),
    }
}

async fn run_listener(p: ListenerParams) {
    let listener = match TcpListener::bind(&p.addr).await {
        Ok(l) => l,
        Err(e) => {
            log::error!("EtherNet/IP server bind on {} failed: {e}", p.addr);
            if let Ok(mut s) = p.status.lock() {
                *s = ServerStatus { listening: false, addr: p.addr.clone(), error: Some(e.to_string()), peer: None };
            }
            return;
        }
    };
    log::info!("EtherNet/IP server listening on {}", p.addr);
    if let Ok(mut s) = p.status.lock() {
        *s = ServerStatus { listening: true, addr: p.addr.clone(), error: None, peer: None };
    }

    // Générateur de handles de session (un par connexion, non nul).
    let handles = Arc::new(AtomicU32::new(1));
    let mut sessions: JoinSet<()> = JoinSet::new();
    loop {
        tokio::select! {
            accepted = listener.accept() => {
                let Ok((socket, peer)) = accepted else { continue };
                let allowed = p.allowlist.lock().map(|a| a.allows(peer.ip())).unwrap_or(false);
                if !allowed {
                    log::warn!("EtherNet/IP connection from {peer} refused (not in allowlist)");
                    continue;
                }
                if let Ok(mut s) = p.status.lock() {
                    s.peer = Some(peer.to_string());
                }
                let handle = handles.fetch_add(1, Ordering::Relaxed);
                log::info!("EtherNet/IP client connected: {peer} (session {handle:#x})");
                sessions.spawn(serve_client(socket, handle, p.sim.clone(), p.snapshot.clone()));
            }
            Some(_) = sessions.join_next(), if !sessions.is_empty() => {}
        }
    }
}

/// Sert une session cliente : lit des paquets d'encapsulation, répond, route les
/// écritures CIP vers l'acteur de simulation.
async fn serve_client(mut socket: TcpStream, handle: u32, sim: ActorRef<SimulationMsg>, snapshot: SharedSnapshot) {
    let mut header = [0u8; ENCAP_HEADER_LEN];
    loop {
        if socket.read_exact(&mut header).await.is_err() {
            break;
        }
        let length = u16::from_le_bytes([header[2], header[3]]) as usize;
        if length > MAX_DATA {
            break;
        }
        let mut packet = Vec::with_capacity(ENCAP_HEADER_LEN + length);
        packet.extend_from_slice(&header);
        packet.resize(ENCAP_HEADER_LEN + length, 0);
        if socket.read_exact(&mut packet[ENCAP_HEADER_LEN..]).await.is_err() {
            break;
        }

        let snap = current_snapshot(&snapshot);
        let (response, commands) = eip_server::handle_packet(&packet, handle, &snap);
        for cmd in commands {
            let _ = sim.cast(SimulationMsg::Command(cmd));
        }
        if let Some(resp) = response {
            if socket.write_all(&resp).await.is_err() {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use ractor::Actor;

    use super::*;
    use crate::actors::{SimulationActor, SimulationArgs};
    use crate::config::{NetworkConfig, ServerStatus};
    use mock_lib_regulator::{Regulator, RegulatorConfig};

    async fn free_port() -> u16 {
        let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        l.local_addr().unwrap().port()
    }

    async fn spawn_pair(network: NetworkConfig) -> (ActorRef<SimulationMsg>, ActorRef<EipServerMsg>, SharedStatus) {
        let cfg = RegulatorConfig::default();
        let snapshot = Arc::new(Mutex::new(Regulator::new(cfg).snapshot()));
        let status = Arc::new(Mutex::new(ServerStatus::default()));
        let (sim, _sj) = Actor::spawn(None, SimulationActor, SimulationArgs { config: cfg, snapshot: snapshot.clone() })
            .await
            .unwrap();
        let (net, _nj) = Actor::spawn(None, EipServerActor, EipServerArgs {
            network,
            sim: sim.clone(),
            snapshot,
            status: status.clone(),
        })
        .await
        .unwrap();
        tokio::time::sleep(Duration::from_millis(200)).await;
        (sim, net, status)
    }

    #[tokio::test]
    async fn eip_server_binds_and_listens() {
        let port = free_port().await;
        let network = NetworkConfig { bind_ip: "127.0.0.1".to_string(), port, allowlist: vec![] };
        let (sim, net, status) = spawn_pair(network).await;
        let st = status.lock().unwrap().clone();
        assert!(st.listening, "doit écouter (erreur: {:?})", st.error);
        net.stop(None);
        sim.stop(None);
    }

    /// Round-trip TCP réel : RegisterSession, Write Tag (Setpoint=80), Read Tag.
    #[tokio::test]
    async fn client_read_write_round_trip() {
        let port = free_port().await;
        let network = NetworkConfig { bind_ip: "127.0.0.1".to_string(), port, allowlist: vec![] };
        let (sim, net, _status) = spawn_pair(network).await;
        let mut sock = TcpStream::connect(("127.0.0.1", port)).await.unwrap();

        // Helpers (réutilisent l'encodage little-endian).
        let encap = |cmd: u16, session: u32, data: &[u8]| -> Vec<u8> {
            let mut v = Vec::new();
            v.extend_from_slice(&cmd.to_le_bytes());
            v.extend_from_slice(&(data.len() as u16).to_le_bytes());
            v.extend_from_slice(&session.to_le_bytes());
            v.extend_from_slice(&0u32.to_le_bytes()); // status
            v.extend_from_slice(&[0u8; 8]); // sender context
            v.extend_from_slice(&0u32.to_le_bytes()); // options
            v.extend_from_slice(data);
            v
        };
        let epath = |tag: &str| -> Vec<u8> {
            let mut p = vec![0x91u8, tag.len() as u8];
            p.extend_from_slice(tag.as_bytes());
            if p.len() % 2 == 1 { p.push(0); }
            p
        };
        let send_rr = |cip: &[u8], session: u32| -> Vec<u8> {
            let mut cpf = Vec::new();
            cpf.extend_from_slice(&0u32.to_le_bytes());
            cpf.extend_from_slice(&0u16.to_le_bytes());
            cpf.extend_from_slice(&2u16.to_le_bytes());
            cpf.extend_from_slice(&0u16.to_le_bytes());
            cpf.extend_from_slice(&0u16.to_le_bytes());
            cpf.extend_from_slice(&0x00B2u16.to_le_bytes());
            cpf.extend_from_slice(&(cip.len() as u16).to_le_bytes());
            cpf.extend_from_slice(cip);
            encap(0x006F, session, &cpf)
        };

        // 1) RegisterSession.
        sock.write_all(&encap(0x0065, 0, &[0x01, 0x00, 0x00, 0x00])).await.unwrap();
        let mut buf = [0u8; 512];
        let n = sock.read(&mut buf).await.unwrap();
        assert!(n >= 8);
        let session = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
        assert_ne!(session, 0, "handle de session attribué");

        // 2) Write Tag Setpoint = 80.0 (REAL).
        let mut wcip = vec![0x4D, (epath("Setpoint").len() / 2) as u8];
        wcip.extend_from_slice(&epath("Setpoint"));
        wcip.extend_from_slice(&0x00CAu16.to_le_bytes()); // type REAL
        wcip.extend_from_slice(&1u16.to_le_bytes());
        wcip.extend_from_slice(&80.0f32.to_le_bytes());
        sock.write_all(&send_rr(&wcip, session)).await.unwrap();
        let _ = sock.read(&mut buf).await.unwrap();

        tokio::time::sleep(Duration::from_millis(200)).await;

        // 3) Read Tag Setpoint.
        let mut rcip = vec![0x4C, (epath("Setpoint").len() / 2) as u8];
        rcip.extend_from_slice(&epath("Setpoint"));
        rcip.extend_from_slice(&1u16.to_le_bytes());
        sock.write_all(&send_rr(&rcip, session)).await.unwrap();
        let n = sock.read(&mut buf).await.unwrap();
        // CIP réponse à l'offset 40 (cf. eip_server) : service, _, status, _, type(2), data(4).
        assert!(n >= 50);
        let v = f32::from_le_bytes([buf[46], buf[47], buf[48], buf[49]]);
        assert!((v - 80.0).abs() < 1e-3, "consigne relue = {v}");

        net.stop(None);
        sim.stop(None);
    }
}
