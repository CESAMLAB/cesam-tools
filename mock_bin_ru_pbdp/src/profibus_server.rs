//! Serveur **PROFIBUS DP** simulé : lit des trames sur un flux série (ou tout flux
//! `AsyncRead + AsyncWrite`, pour les tests), les interprète via la machine à
//! états [`crate::profibus::SlaveFsm`], et répond.
//!
//! La liaison série *est* l'unique maître (bus point-à-point) : pas de politique
//! multi-maître à gérer, contrairement à Modbus TCP (ORME) ou même NAMUR TCP
//! (OSNE). Le chien de garde protocolaire est ici une **vraie** partie du
//! protocole DP (armé par `Set_Prm`), pas un ajout maison comme sur NAMUR.

use std::time::{Duration, Instant};

use ractor::ActorRef;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::actors::{SharedSnapshot, SharedStatus, SimulationMsg};
use crate::config::ServerStatus;
use crate::profibus::{self, Handled, SlaveFsm, SlaveProfile, SlaveState};
use crate::regulator::Command;
use crate::trace::{self, Direction, SharedTrace};

/// Identifiant PROFIBUS **fictif** de ce simulateur, non enregistré au PNO — voir
/// `docs/fr/reference_profibus.md`.
pub const IDENT_NUMBER: u16 = 0xEE01;

fn mark_activity(status: &SharedStatus) {
    if let Ok(mut s) = status.lock() {
        s.last_request = Some(Instant::now());
    }
}

fn set_state(status: &SharedStatus, state: SlaveState) {
    if let Ok(mut s) = status.lock() {
        s.state = Some(state_label(state).to_string());
    }
}

fn state_label(state: SlaveState) -> &'static str {
    match state {
        SlaveState::PowerOn => "Power_On",
        SlaveState::WaitPrm => "Wait_Prm",
        SlaveState::WaitCfg => "Wait_Cfg",
        SlaveState::DataExchange => "Data_Exchange",
    }
}

/// Lit une trame PROFIBUS complète (délimiteur + charge utile de longueur
/// dépendante) depuis un flux asynchrone quelconque.
async fn read_frame<R: AsyncRead + Unpin>(r: &mut R) -> std::io::Result<Vec<u8>> {
    let sd = r.read_u8().await?;
    match sd {
        profibus::SD1 => {
            let mut rest = [0u8; 5];
            r.read_exact(&mut rest).await?;
            let mut buf = vec![sd];
            buf.extend_from_slice(&rest);
            Ok(buf)
        }
        profibus::SD2 => {
            let mut hdr = [0u8; 2]; // LE, LEr
            r.read_exact(&mut hdr).await?;
            let l = hdr[0];
            let mut rest = vec![0u8; 3 + l as usize];
            r.read_exact(&mut rest).await?;
            let mut buf = vec![sd, hdr[0], hdr[1]];
            buf.extend_from_slice(&rest);
            Ok(buf)
        }
        profibus::SD3 => {
            let mut rest = [0u8; 13];
            r.read_exact(&mut rest).await?;
            let mut buf = vec![sd];
            buf.extend_from_slice(&rest);
            Ok(buf)
        }
        profibus::SD4 => {
            let mut rest = [0u8; 2];
            r.read_exact(&mut rest).await?;
            let mut buf = vec![sd];
            buf.extend_from_slice(&rest);
            Ok(buf)
        }
        // Délimiteur inconnu (bruit de ligne) : trame d'un octet, rejetée par
        // `decode_request` — permet de resynchroniser sans planter la session.
        other => Ok(vec![other]),
    }
}

fn hex(bytes: &[u8]) -> String {
    format!("{bytes:02X?}")
}

/// Déroule une session PROFIBUS DP sur un flux quelconque jusqu'à EOF/erreur.
pub async fn run_session<S>(
    stream: S,
    sim: ActorRef<SimulationMsg>,
    snapshot: SharedSnapshot,
    status: SharedStatus,
    trace: SharedTrace,
    station_address: u8,
    watchdog_allowed: bool,
) where
    S: AsyncRead + AsyncWrite + Unpin + Send,
{
    let (mut rd, mut wr) = tokio::io::split(stream);
    let profile = SlaveProfile {
        ident_number: IDENT_NUMBER,
        out_len: crate::map::OUTPUT_LEN as u8,
        in_len: crate::map::INPUT_LEN as u8,
    };
    let mut fsm = SlaveFsm::new(profile);
    set_state(&status, fsm.state());
    let mut watchdog: Option<Duration> = None;

    loop {
        let frame = match watchdog {
            Some(d) => tokio::select! {
                res = read_frame(&mut rd) => res,
                () = tokio::time::sleep(d) => {
                    log::warn!("PROFIBUS DP watchdog timeout — forcing safe state");
                    let _ = sim.cast(SimulationMsg::Command(fsm.watchdog_expired()));
                    watchdog = None;
                    continue;
                }
            },
            None => read_frame(&mut rd).await,
        };

        let frame = match frame {
            Ok(f) => f,
            Err(e) => {
                log::info!("PROFIBUS DP link closed: {e}");
                break;
            }
        };

        trace::record(&trace, Direction::Rx, hex(&frame));

        let (target, master, req) = match profibus::decode_request(&frame) {
            Ok(v) => v,
            Err(e) => {
                log::debug!("PROFIBUS DP frame rejected ({e:?}): {}", hex(&frame));
                continue;
            }
        };
        if target != station_address {
            continue; // trame adressée à une autre station du bus (pas d'activité marquée)
        }
        mark_activity(&status);

        let snap = match snapshot.lock() {
            Ok(g) => *g,
            Err(_) => continue,
        };
        let Handled {
            response,
            commands,
            watchdog_ms,
        } = fsm.handle(req, &snap);

        for cmd in commands {
            apply(&sim, cmd);
        }
        if let Some(ms) = watchdog_ms {
            watchdog = if watchdog_allowed {
                ms.map(|ms| Duration::from_millis(u64::from(ms)))
            } else {
                None
            };
        }
        set_state(&status, fsm.state());

        let bytes = profibus::encode_response(station_address, master, &response);
        trace::record(&trace, Direction::Tx, hex(&bytes));
        if wr.write_all(&bytes).await.is_err() {
            break;
        }
    }
}

fn apply(sim: &ActorRef<SimulationMsg>, cmd: Command) {
    let _ = sim.cast(SimulationMsg::Command(cmd));
}

/// Sert la session PROFIBUS DP sur une liaison série déjà ouverte.
pub async fn serve_serial(
    serial: tokio_serial::SerialStream,
    sim: ActorRef<SimulationMsg>,
    snapshot: SharedSnapshot,
    status: SharedStatus,
    trace: SharedTrace,
    station_address: u8,
    watchdog_allowed: bool,
) {
    run_session(serial, sim, snapshot, status, trace, station_address, watchdog_allowed).await;
}

/// Construit un [`ServerStatus`] initial pour l'affichage IHM avant tout démarrage.
#[must_use]
pub fn initial_status() -> ServerStatus {
    ServerStatus {
        state: Some(state_label(SlaveState::PowerOn).to_string()),
        ..ServerStatus::default()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use ractor::Actor;
    use tokio::io::duplex;

    use super::*;
    use crate::actors::{SimulationActor, SimulationArgs};
    use crate::profibus::Request;
    use crate::regulator::{AutoManual, Regulator, RegulatorConfig};

    async fn spawn(
        station: u8,
        watchdog_allowed: bool,
    ) -> (
        ActorRef<SimulationMsg>,
        tokio::io::DuplexStream,
        SharedSnapshot,
        SharedStatus,
        SharedTrace,
    ) {
        let cfg = RegulatorConfig::default();
        let snapshot = Arc::new(Mutex::new(Regulator::new(cfg.clone()).snapshot()));
        let status = Arc::new(Mutex::new(ServerStatus::default()));
        let trace: SharedTrace = Arc::new(Mutex::new(std::collections::VecDeque::new()));

        let (sim, _sj) = Actor::spawn(None, SimulationActor, SimulationArgs {
            config: cfg,
            snapshot: snapshot.clone(),
        })
        .await
        .unwrap();

        let (master_side, slave_side) = duplex(4096);
        tokio::spawn(run_session(
            slave_side,
            sim.clone(),
            snapshot.clone(),
            status.clone(),
            trace.clone(),
            station,
            watchdog_allowed,
        ));

        (sim, master_side, snapshot, status, trace)
    }

    #[tokio::test]
    async fn full_handshake_then_data_exchange() {
        let (sim, mut master, snapshot, status, trace) = spawn(5, true).await;

        // Slave_Diag.
        let bytes = profibus::encode_request(5, 3, &Request::SlaveDiag);
        master.write_all(&bytes).await.unwrap();
        let mut buf = [0u8; 64];
        let n = master.read(&mut buf).await.unwrap();
        assert!(matches!(profibus::decode_response(&buf[..n]).unwrap(), profibus::Response::Diag(_)));

        // Set_Prm avec l'identifiant attendu.
        let bytes = profibus::encode_request(
            5,
            3,
            &Request::SetPrm {
                ident_number: IDENT_NUMBER,
                watchdog_ms: Some(300),
            },
        );
        master.write_all(&bytes).await.unwrap();
        let n = master.read(&mut buf).await.unwrap();
        assert_eq!(profibus::decode_response(&buf[..n]).unwrap(), profibus::Response::ShortAck);

        // Chk_Cfg avec les longueurs attendues.
        let bytes = profibus::encode_request(
            5,
            3,
            &Request::ChkCfg {
                out_len: crate::map::OUTPUT_LEN as u8,
                in_len: crate::map::INPUT_LEN as u8,
            },
        );
        master.write_all(&bytes).await.unwrap();
        let n = master.read(&mut buf).await.unwrap();
        assert_eq!(profibus::decode_response(&buf[..n]).unwrap(), profibus::Response::ShortAck);

        assert_eq!(status.lock().unwrap().state.as_deref(), Some("Data_Exchange"));

        // Data_Exchange : démarre l'appareil avec une consigne.
        let output = crate::map::encode_output_for_test(true, AutoManual::Auto, 120.0);
        let bytes = profibus::encode_request(5, 3, &Request::DataExchange { output });
        master.write_all(&bytes).await.unwrap();
        let n = master.read(&mut buf).await.unwrap();
        assert!(matches!(
            profibus::decode_response(&buf[..n]).unwrap(),
            profibus::Response::DataExchange(_)
        ));

        tokio::time::sleep(Duration::from_millis(80)).await;
        assert!(snapshot.lock().unwrap().on, "l'appareil doit être démarré");
        assert!(status.lock().unwrap().last_request.is_some());
        let t = trace.lock().unwrap();
        assert!(t.iter().any(|e| e.dir == Direction::Rx));
        assert!(t.iter().any(|e| e.dir == Direction::Tx));
        drop(t);

        sim.stop(None);
    }

    #[tokio::test]
    async fn frame_for_other_station_is_ignored() {
        let (sim, mut master, _snap, status, _trace) = spawn(5, true).await;

        let bytes = profibus::encode_request(9, 3, &Request::SlaveDiag); // station 9, pas 5
        master.write_all(&bytes).await.unwrap();

        tokio::time::sleep(Duration::from_millis(80)).await;
        assert!(status.lock().unwrap().last_request.is_none(), "aucune activité pour une autre station");

        sim.stop(None);
    }

    #[tokio::test]
    async fn watchdog_timeout_forces_safe_state() {
        let (sim, mut master, snapshot, _status, _trace) = spawn(5, true).await;
        let mut buf = [0u8; 64];

        for req in [
            Request::SlaveDiag,
            Request::SetPrm {
                ident_number: IDENT_NUMBER,
                watchdog_ms: Some(100),
            },
            Request::ChkCfg {
                out_len: crate::map::OUTPUT_LEN as u8,
                in_len: crate::map::INPUT_LEN as u8,
            },
        ] {
            let bytes = profibus::encode_request(5, 3, &req);
            master.write_all(&bytes).await.unwrap();
            let n = master.read(&mut buf).await.unwrap();
            let _ = &buf[..n];
        }
        let output = crate::map::encode_output_for_test(true, AutoManual::Auto, 120.0);
        let bytes = profibus::encode_request(5, 3, &Request::DataExchange { output });
        master.write_all(&bytes).await.unwrap();
        let n = master.read(&mut buf).await.unwrap();
        let _ = &buf[..n];

        tokio::time::sleep(Duration::from_millis(60)).await;
        assert!(snapshot.lock().unwrap().on, "en marche avant l'échéance du chien de garde");

        tokio::time::sleep(Duration::from_millis(250)).await;
        assert!(!snapshot.lock().unwrap().on, "le chien de garde doit forcer l'arrêt");

        sim.stop(None);
    }
}
