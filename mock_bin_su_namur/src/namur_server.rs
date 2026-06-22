//! Serveur **NAMUR** : lit des lignes ASCII, les interprète ([`crate::namur`]) et
//! répond. Les lectures puisent dans l'instantané partagé ; les écritures/actions
//! sont relayées à l'acteur de simulation.
//!
//! - **TCP** : une seule session servie à la fois (point-à-point). Un nouveau
//!   maître n'est accepté qu'à la déconnexion du précédent (file d'attente TCP).
//! - **Série** : la liaison RS-232 *est* l'unique maître.
//!
//! **Chien de garde** (`OUT_WD1@m` / `OUT_WD2@m`) : si aucune ligne n'arrive
//! pendant `m` secondes, le moteur est **arrêté** (état sûr).

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use ractor::ActorRef;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::io::AsyncBufReadExt;
use tokio::net::TcpListener;

use crate::actors::{SharedAllowlist, SharedSnapshot, SharedStatus, SimulationMsg};
use crate::namur::{self, NamurResponse};
use crate::stirrer::Command;
use crate::trace::{self, Direction, SharedTrace};

/// Marque l'activité du lien (témoin pour l'IHM).
fn mark_activity(status: &SharedStatus) {
    if let Ok(mut s) = status.lock() {
        s.last_request = Some(Instant::now());
    }
}

/// Déroule une session NAMUR ligne-à-ligne sur un flux quelconque jusqu'à EOF/erreur.
async fn run_line_session<S>(
    stream: S,
    sim: ActorRef<SimulationMsg>,
    snapshot: SharedSnapshot,
    status: SharedStatus,
    log: SharedTrace,
) where
    S: AsyncRead + AsyncWrite + Unpin + Send,
{
    let (rd, mut wr) = tokio::io::split(stream);
    let mut lines = BufReader::new(rd).lines();
    // Délai du chien de garde courant (`None` = désarmé).
    let mut watchdog: Option<Duration> = None;

    loop {
        // Lecture de la ligne suivante, éventuellement bornée par le chien de garde.
        let line = match watchdog {
            Some(d) => tokio::select! {
                res = lines.next_line() => res,
                _ = tokio::time::sleep(d) => {
                    log::warn!("NAMUR watchdog timeout — stopping motor (safe state)");
                    let _ = sim.cast(SimulationMsg::Command(Command::SetOnOff(false)));
                    watchdog = None; // désarme après déclenchement
                    continue;
                }
            },
            None => lines.next_line().await,
        };

        let line = match line {
            Ok(Some(l)) => l,
            Ok(None) => break, // EOF : le maître s'est déconnecté
            Err(e) => {
                log::warn!("NAMUR read error: {e}");
                break;
            }
        };

        mark_activity(&status);
        // Consigne la trame reçue (hors lignes vides) pour le moniteur de l'IHM.
        if !line.trim().is_empty() {
            trace::record(&log, Direction::Rx, line.clone());
        }
        let snap = match snapshot.lock() {
            Ok(g) => *g,
            Err(_) => continue,
        };

        match namur::handle_line(&line, &snap) {
            NamurResponse::Reply(reply) => {
                trace::record(&log, Direction::Tx, reply.clone());
                if wr.write_all(reply.as_bytes()).await.is_err() || wr.write_all(b"\r\n").await.is_err()
                {
                    break;
                }
            }
            NamurResponse::Apply(cmd) => {
                let _ = sim.cast(SimulationMsg::Command(cmd));
            }
            NamurResponse::SetWatchdog(secs) => {
                watchdog = (secs > 0.0).then(|| Duration::from_secs_f32(secs));
            }
            NamurResponse::Ignore => {}
            NamurResponse::Unknown => {
                log::debug!("NAMUR unknown command: {line:?}");
            }
        }
    }
}

/// Boucle d'écoute TCP : sert **un maître à la fois** (point-à-point). Ne retourne
/// qu'en cas d'erreur fatale d'`accept`.
pub async fn serve_tcp(
    listener: TcpListener,
    sim: ActorRef<SimulationMsg>,
    snapshot: SharedSnapshot,
    status: SharedStatus,
    allowlist: SharedAllowlist,
    log: SharedTrace,
) -> std::io::Result<()> {
    loop {
        let (stream, peer): (_, SocketAddr) = listener.accept().await?;
        let allowed = allowlist
            .lock()
            .map(|f| f.allows(peer.ip()))
            .unwrap_or(true);
        if !allowed {
            log::warn!("Connection refused from {peer} (not in allowlist)");
            continue;
        }
        log::info!("NAMUR master connected: {peer}");
        if let Ok(mut s) = status.lock() {
            s.peer = Some(peer.to_string());
        }
        // Sert ce maître jusqu'à sa déconnexion (les suivants patientent dans la
        // file d'attente TCP) : pas de tâche détachée à abandonner.
        run_line_session(stream, sim.clone(), snapshot.clone(), status.clone(), log.clone()).await;
        log::info!("NAMUR master disconnected: {peer}");
        if let Ok(mut s) = status.lock() {
            s.peer = None;
        }
    }
}

/// Boucle de service NAMUR sur une liaison série déjà ouverte (feature `serial`).
#[cfg(feature = "serial")]
pub async fn serve_serial(
    serial: tokio_serial::SerialStream,
    sim: ActorRef<SimulationMsg>,
    snapshot: SharedSnapshot,
    status: SharedStatus,
    log: SharedTrace,
) -> std::io::Result<()> {
    run_line_session(serial, sim, snapshot, status, log).await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use ractor::Actor;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::TcpStream;

    use super::*;
    use crate::actors::{SimulationActor, SimulationArgs};
    use crate::config::{IpFilter, ServerStatus};
    use crate::stirrer::{Stirrer, StirrerConfig};

    #[tokio::test]
    async fn tcp_namur_read_and_command() {
        let cfg = StirrerConfig::default();
        let snapshot = Arc::new(Mutex::new(Stirrer::new(cfg.clone()).snapshot()));
        let allowlist = Arc::new(Mutex::new(IpFilter::default()));
        let status = Arc::new(Mutex::new(ServerStatus::default()));
        let trace: SharedTrace = Arc::new(Mutex::new(std::collections::VecDeque::new()));

        let (sim, _sj) = Actor::spawn(None, SimulationActor, SimulationArgs {
            config: cfg,
            snapshot: snapshot.clone(),
        })
        .await
        .unwrap();

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(serve_tcp(listener, sim.clone(), snapshot, status.clone(), allowlist, trace.clone()));

        let stream = TcpStream::connect(addr).await.unwrap();
        let (rd, mut wr) = stream.into_split();
        let mut lines = BufReader::new(rd).lines();

        // Lecture du nom de l'appareil.
        wr.write_all(b"IN_NAME\r\n").await.unwrap();
        let name = lines.next_line().await.unwrap().unwrap();
        assert_eq!(name, crate::namur::DEVICE_NAME);

        // Commande de démarrage + consigne (silencieuses), puis on relit la consigne.
        wr.write_all(b"START_4\r\n").await.unwrap();
        wr.write_all(b"OUT_SP_4 500\r\n").await.unwrap();
        tokio::time::sleep(Duration::from_millis(80)).await;
        wr.write_all(b"IN_SP_4\r\n").await.unwrap();
        let sp = lines.next_line().await.unwrap().unwrap();
        assert_eq!(sp, "500.0 4");

        // Le témoin d'activité a été marqué.
        assert!(status.lock().unwrap().last_request.is_some());
        // Le journal a consigné des trames RX (requêtes) et TX (réponses).
        let t = trace.lock().unwrap();
        assert!(t.iter().any(|e| e.dir == Direction::Rx));
        assert!(t.iter().any(|e| e.dir == Direction::Tx));
        drop(t);
        sim.stop(None);
    }
}
