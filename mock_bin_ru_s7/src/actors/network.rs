//! Acteur réseau : possède le **serveur S7** (ISO-on-TCP) et le (re)démarre à chaud.
//!
//! Le serveur écoute en TCP et traite **plusieurs sessions clientes simultanées**
//! (comportement usuel d'un automate ; pas de mono-maître, à l'inverse d'ORME). La
//! boucle d'acceptation tourne dans une tâche tokio dédiée dont l'acteur conserve le
//! `JoinHandle` (abandon à l'arrêt) ; ses sessions sont portées par un `JoinSet`
//! **interne** à la tâche, donc toutes abattues quand la boucle est abandonnée
//! (aucune tâche détachée n'est laissée derrière).
//!
//! La **liste blanche d'IP** est partagée ([`SharedAllowlist`]) : une
//! reconfiguration met à jour le filtre appliqué aux **nouvelles** connexions, et
//! relance l'écoute si l'IP/port change.

use std::sync::{Arc, Mutex};

use ractor::{Actor, ActorProcessingErr, ActorRef};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::{JoinHandle, JoinSet};

use crate::config::{Allowlist, NetworkConfig, ServerStatus};
use mock_lib_regulator::Snapshot;
use crate::s7_server;

use super::{SharedAllowlist, SharedSnapshot, SharedStatus, SimulationMsg};

/// Taille maximale d'une trame TPKT acceptée (garde-fou réseau).
const MAX_TPKT: usize = 4096;

/// Messages de l'acteur réseau.
#[derive(Debug)]
pub enum S7ServerMsg {
    /// Applique une nouvelle configuration réseau (relance si l'IP ou le port
    /// change ; met à jour la liste blanche dans tous les cas).
    Reconfigure { network: NetworkConfig },
}

/// Arguments de démarrage de l'acteur réseau.
pub struct S7ServerArgs {
    pub network: NetworkConfig,
    pub sim: ActorRef<SimulationMsg>,
    pub snapshot: SharedSnapshot,
    pub status: SharedStatus,
}

/// État interne de l'acteur réseau.
pub struct S7ServerState {
    network: NetworkConfig,
    sim: ActorRef<SimulationMsg>,
    snapshot: SharedSnapshot,
    status: SharedStatus,
    allowlist: SharedAllowlist,
    task: Option<JoinHandle<()>>,
}

impl S7ServerState {
    fn stop_current(&mut self) {
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }

    /// (Re)démarre la boucle d'écoute S7 selon la configuration courante.
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

/// Acteur supervisant le cycle de vie du serveur S7.
pub struct S7ServerActor;

impl Actor for S7ServerActor {
    type Msg = S7ServerMsg;
    type State = S7ServerState;
    type Arguments = S7ServerArgs;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let allowlist = Arc::new(Mutex::new(Allowlist::new(args.network.allowlist.clone())));
        let mut state = S7ServerState {
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
            S7ServerMsg::Reconfigure { network } => {
                let rebind = network.bind_ip != state.network.bind_ip || network.port != state.network.port;
                state.network = network;
                if rebind {
                    state.restart();
                } else if let Ok(mut a) = state.allowlist.lock() {
                    // Même socket : on actualise seulement la liste blanche.
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

/// Paramètres figés de la boucle d'écoute.
struct ListenerParams {
    addr: String,
    sim: ActorRef<SimulationMsg>,
    snapshot: SharedSnapshot,
    status: SharedStatus,
    allowlist: SharedAllowlist,
}

/// Lit l'instantané partagé (jamais de verrou tenu à travers un `.await`).
fn current_snapshot(snapshot: &SharedSnapshot) -> Snapshot {
    match snapshot.lock() {
        Ok(g) => *g,
        Err(_) => mock_lib_regulator::Regulator::default().snapshot(),
    }
}

/// Boucle d'acceptation : lie le socket puis sert chaque client dans une tâche
/// fille (portée par un `JoinSet` local → abattue avec la boucle).
async fn run_listener(p: ListenerParams) {
    let listener = match TcpListener::bind(&p.addr).await {
        Ok(l) => l,
        Err(e) => {
            log::error!("S7 server bind on {} failed: {e}", p.addr);
            if let Ok(mut s) = p.status.lock() {
                *s = ServerStatus { listening: false, addr: p.addr.clone(), error: Some(e.to_string()), peer: None };
            }
            return;
        }
    };
    log::info!("S7 server listening on {}", p.addr);
    if let Ok(mut s) = p.status.lock() {
        *s = ServerStatus { listening: true, addr: p.addr.clone(), error: None, peer: None };
    }

    let mut sessions: JoinSet<()> = JoinSet::new();
    loop {
        tokio::select! {
            accepted = listener.accept() => {
                let Ok((socket, peer)) = accepted else { continue };
                let allowed = p.allowlist.lock().map(|a| a.allows(peer.ip())).unwrap_or(false);
                if !allowed {
                    log::warn!("S7 connection from {peer} refused (not in allowlist)");
                    continue;
                }
                if let Ok(mut s) = p.status.lock() {
                    s.peer = Some(peer.to_string());
                }
                log::info!("S7 client connected: {peer}");
                sessions.spawn(serve_client(socket, p.sim.clone(), p.snapshot.clone()));
            }
            // Récupère les sessions terminées pour borner le JoinSet.
            Some(_) = sessions.join_next(), if !sessions.is_empty() => {}
        }
    }
}

/// Sert une session cliente : lit des trames TPKT, répond, route les écritures.
async fn serve_client(mut socket: TcpStream, sim: ActorRef<SimulationMsg>, snapshot: SharedSnapshot) {
    let mut header = [0u8; 4];
    loop {
        if socket.read_exact(&mut header).await.is_err() {
            break; // déconnexion / EOF
        }
        if header[0] != 0x03 {
            break; // pas du TPKT
        }
        let len = u16::from_be_bytes([header[2], header[3]]) as usize;
        if !(4..=MAX_TPKT).contains(&len) {
            break;
        }
        let mut frame = Vec::with_capacity(len);
        frame.extend_from_slice(&header);
        frame.resize(len, 0);
        if socket.read_exact(&mut frame[4..]).await.is_err() {
            break;
        }

        let snap = current_snapshot(&snapshot);
        let (response, commands) = s7_server::handle_frame(&frame, &snap);
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

    async fn spawn_pair(network: NetworkConfig) -> (ActorRef<SimulationMsg>, ActorRef<S7ServerMsg>, SharedStatus) {
        let cfg = RegulatorConfig::default();
        let snapshot = Arc::new(Mutex::new(Regulator::new(cfg).snapshot()));
        let status = Arc::new(Mutex::new(ServerStatus::default()));
        let (sim, _sj) = Actor::spawn(None, SimulationActor, SimulationArgs { config: cfg, snapshot: snapshot.clone() })
            .await
            .unwrap();
        let (net, _nj) = Actor::spawn(None, S7ServerActor, S7ServerArgs {
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
    async fn s7_server_binds_and_listens() {
        let port = free_port().await;
        let network = NetworkConfig { bind_ip: "127.0.0.1".to_string(), port, allowlist: vec![] };
        let (sim, net, status) = spawn_pair(network).await;
        let st = status.lock().unwrap().clone();
        assert!(st.listening, "doit écouter (erreur: {:?})", st.error);
        net.stop(None);
        sim.stop(None);
    }

    /// Round-trip TCP réel : connexion COTP, Setup, écriture puis relecture de la
    /// consigne via des trames S7 brutes (sans dépendance client externe).
    #[tokio::test]
    async fn client_read_write_round_trip() {
        let port = free_port().await;
        let network = NetworkConfig { bind_ip: "127.0.0.1".to_string(), port, allowlist: vec![] };
        let (sim, net, _status) = spawn_pair(network).await;

        let mut sock = TcpStream::connect(("127.0.0.1", port)).await.unwrap();

        // 1) Connection Request → Confirm.
        let cr = [
            0x03, 0x00, 0x00, 0x16, 0x11, 0xE0, 0x00, 0x00, 0x00, 0x01, 0x00, 0xc0, 0x01, 0x0a,
            0xc1, 0x02, 0x01, 0x00, 0xc2, 0x02, 0x01, 0x02,
        ];
        sock.write_all(&cr).await.unwrap();
        let mut buf = [0u8; 256];
        let n = sock.read(&mut buf).await.unwrap();
        assert!(n > 5 && buf[5] == 0xD0, "CC attendu");

        // 2) Write Var REAL DBD0 = 80.0.
        let mut s7 = vec![0x32, 0x01, 0x00, 0x00, 0x00, 0x02];
        let params = {
            let mut p = vec![0x05, 0x01, 0x12, 0x0a, 0x10, 0x08];
            p.extend_from_slice(&1u16.to_be_bytes()); // count
            p.extend_from_slice(&1u16.to_be_bytes()); // db
            p.push(0x84); // area DB
            p.extend_from_slice(&0u32.to_be_bytes()[1..4]); // addr 0
            p
        };
        let mut data = vec![0x00, 0x04, 0x00, 0x20];
        data.extend_from_slice(&80.0f32.to_be_bytes());
        s7.extend_from_slice(&(params.len() as u16).to_be_bytes());
        s7.extend_from_slice(&(data.len() as u16).to_be_bytes());
        s7.extend_from_slice(&params);
        s7.extend_from_slice(&data);
        let mut cotp = vec![0x02, 0xF0, 0x80];
        cotp.extend_from_slice(&s7);
        let total = (cotp.len() + 4) as u16;
        let mut frame = vec![0x03, 0x00];
        frame.extend_from_slice(&total.to_be_bytes());
        frame.extend_from_slice(&cotp);
        sock.write_all(&frame).await.unwrap();
        let _ = sock.read(&mut buf).await.unwrap();

        // Laisse l'acteur appliquer la commande.
        tokio::time::sleep(Duration::from_millis(200)).await;

        // 3) Read Var REAL DBD0 → doit refléter 80.0.
        let mut s7r = vec![0x32, 0x01, 0x00, 0x00, 0x00, 0x03];
        let pr = {
            let mut p = vec![0x04, 0x01, 0x12, 0x0a, 0x10, 0x08];
            p.extend_from_slice(&1u16.to_be_bytes());
            p.extend_from_slice(&1u16.to_be_bytes());
            p.push(0x84);
            p.extend_from_slice(&0u32.to_be_bytes()[1..4]);
            p
        };
        s7r.extend_from_slice(&(pr.len() as u16).to_be_bytes());
        s7r.extend_from_slice(&0u16.to_be_bytes());
        s7r.extend_from_slice(&pr);
        let mut cotpr = vec![0x02, 0xF0, 0x80];
        cotpr.extend_from_slice(&s7r);
        let totalr = (cotpr.len() + 4) as u16;
        let mut framer = vec![0x03, 0x00];
        framer.extend_from_slice(&totalr.to_be_bytes());
        framer.extend_from_slice(&cotpr);
        sock.write_all(&framer).await.unwrap();
        let n = sock.read(&mut buf).await.unwrap();
        // data REAL aux 4 octets suivant rc(1)+transport(1)+len(2), à l'offset 21.
        assert!(n >= 25);
        let value = f32::from_be_bytes([buf[25], buf[26], buf[27], buf[28]]);
        assert!((value - 80.0).abs() < 1e-3, "consigne relue = {value}");

        net.stop(None);
        sim.stop(None);
    }
}
