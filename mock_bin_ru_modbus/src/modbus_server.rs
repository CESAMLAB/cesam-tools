//! Serveur Modbus TCP : expose l'image mémoire du régulateur et relaie les
//! écritures vers l'acteur de simulation.

use std::future::{self, Future, Ready};
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::Instant;

use ractor::ActorRef;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use tokio_modbus::prelude::{ExceptionCode, Request, Response};
use tokio_modbus::server::tcp::Server;
use tokio_modbus::server::Service;

use crate::actors::{SharedAllowlist, SharedMap, SharedSnapshot, SharedStatus, SimulationMsg};
use crate::map::{coil_to_command, holdings_to_commands};
use crate::regulator::Command;

/// Service Modbus partagé entre toutes les connexions.
///
/// Les lectures interrogent directement l'image mémoire ([`SharedMap`]) ; les
/// écritures sont décodées en [`Command`] puis transmises à l'acteur de simulation.
#[derive(Clone)]
pub struct RegulatorService {
    actor: ActorRef<SimulationMsg>,
    map: SharedMap,
    snapshot: SharedSnapshot,
    status: SharedStatus,
}

impl RegulatorService {
    #[must_use]
    pub fn new(
        actor: ActorRef<SimulationMsg>,
        map: SharedMap,
        snapshot: SharedSnapshot,
        status: SharedStatus,
    ) -> Self {
        Self {
            actor,
            map,
            snapshot,
            status,
        }
    }

    /// Envoie une commande à l'acteur (best-effort : un acteur arrêté est ignoré).
    fn dispatch(&self, cmd: Command) {
        let _ = self.actor.cast(SimulationMsg::Command(cmd));
    }

    /// Horodate la dernière requête reçue (témoin d'activité du lien pour l'IHM).
    /// Mise à jour partielle : ne touche ni au statut d'écoute ni au maître connecté.
    fn mark_activity(&self) {
        if let Ok(mut s) = self.status.lock() {
            s.last_request = Some(Instant::now());
        }
    }

    fn handle(&self, req: Request<'static>) -> Result<Response, ExceptionCode> {
        // Toute requête (lecture comprise) atteste que le maître interroge l'appareil.
        self.mark_activity();
        match req {
            Request::ReadCoils(addr, qty) => {
                let map = self.lock_map()?;
                read_bits(&map.coils, addr, qty).map(Response::ReadCoils)
            }
            Request::ReadDiscreteInputs(addr, qty) => {
                let map = self.lock_map()?;
                read_bits(&map.discretes, addr, qty).map(Response::ReadDiscreteInputs)
            }
            Request::ReadHoldingRegisters(addr, qty) => {
                let map = self.lock_map()?;
                read_words(&map.holdings, addr, qty).map(Response::ReadHoldingRegisters)
            }
            Request::ReadInputRegisters(addr, qty) => {
                let map = self.lock_map()?;
                read_words(&map.inputs, addr, qty).map(Response::ReadInputRegisters)
            }
            Request::WriteSingleCoil(addr, value) => {
                self.write_coils(addr, &[value])?;
                Ok(Response::WriteSingleCoil(addr, value))
            }
            Request::WriteMultipleCoils(addr, values) => {
                let count = values.len() as u16;
                self.write_coils(addr, &values)?;
                Ok(Response::WriteMultipleCoils(addr, count))
            }
            Request::WriteSingleRegister(addr, value) => {
                self.write_holdings(addr, &[value])?;
                Ok(Response::WriteSingleRegister(addr, value))
            }
            Request::WriteMultipleRegisters(addr, values) => {
                let count = values.len() as u16;
                self.write_holdings(addr, &values)?;
                Ok(Response::WriteMultipleRegisters(addr, count))
            }
            _ => Err(ExceptionCode::IllegalFunction),
        }
    }

    /// Applique l'écriture d'un bloc de bobines à partir de `addr`.
    ///
    /// Toute adresse ne correspondant à aucune bobine connue déclenche l'exception
    /// *Illegal Data Address*.
    fn write_coils(&self, addr: u16, values: &[bool]) -> Result<(), ExceptionCode> {
        for (i, &value) in values.iter().enumerate() {
            match coil_to_command(addr + i as u16, value) {
                Some(cmd) => self.dispatch(cmd),
                None => return Err(ExceptionCode::IllegalDataAddress),
            }
        }
        Ok(())
    }

    /// Applique l'écriture d'un bloc de registres de maintien à partir de `addr`.
    ///
    /// Un bloc ne ciblant aucun champ inscriptible déclenche l'exception
    /// *Illegal Data Address*.
    fn write_holdings(&self, addr: u16, values: &[u16]) -> Result<(), ExceptionCode> {
        let snap = self.current_snapshot()?;
        let cmds = holdings_to_commands(addr, values, &snap);
        if cmds.is_empty() {
            return Err(ExceptionCode::IllegalDataAddress);
        }
        for cmd in cmds {
            self.dispatch(cmd);
        }
        Ok(())
    }

    fn lock_map(&self) -> Result<std::sync::MutexGuard<'_, crate::map::MemoryMap>, ExceptionCode> {
        self.map.lock().map_err(|_| ExceptionCode::ServerDeviceFailure)
    }

    fn current_snapshot(&self) -> Result<crate::regulator::RegulatorSnapshot, ExceptionCode> {
        self.snapshot
            .lock()
            .map(|g| *g)
            .map_err(|_| ExceptionCode::ServerDeviceFailure)
    }
}

impl Service for RegulatorService {
    type Request = Request<'static>;
    type Response = Response;
    type Exception = ExceptionCode;
    type Future = Ready<Result<Self::Response, Self::Exception>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        future::ready(self.handle(req))
    }
}

/// Lit `qty` bits à partir de `addr`, en vérifiant les bornes.
fn read_bits(table: &[bool], addr: u16, qty: u16) -> Result<Vec<bool>, ExceptionCode> {
    let start = addr as usize;
    let end = start + qty as usize;
    if end > table.len() {
        return Err(ExceptionCode::IllegalDataAddress);
    }
    Ok(table[start..end].to_vec())
}

/// Lit `qty` registres à partir de `addr`, en vérifiant les bornes.
fn read_words(table: &[u16], addr: u16, qty: u16) -> Result<Vec<u16>, ExceptionCode> {
    let start = addr as usize;
    let end = start + qty as usize;
    if end > table.len() {
        return Err(ExceptionCode::IllegalDataAddress);
    }
    Ok(table[start..end].to_vec())
}

/// Flux TCP qui signale une **fin de flux** (EOF en lecture) dès qu'il reçoit un
/// ordre d'éviction. Sert à appliquer la politique **mono-maître** : à la
/// connexion d'un nouveau maître, la connexion précédente est fermée.
struct CancellableStream {
    inner: TcpStream,
    /// Reçoit l'ordre de fermeture (ou est résolu si l'émetteur est abandonné).
    kick: oneshot::Receiver<()>,
    closed: bool,
}

impl CancellableStream {
    fn new(inner: TcpStream, kick: oneshot::Receiver<()>) -> Self {
        Self {
            inner,
            kick,
            closed: false,
        }
    }
}

impl AsyncRead for CancellableStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        if self.closed {
            return Poll::Ready(Ok(())); // EOF : la boucle de traitement s'arrête.
        }
        // Évincé par un nouveau maître ? On simule une fin de flux pour fermer.
        if Pin::new(&mut self.kick).poll(cx).is_ready() {
            self.closed = true;
            return Poll::Ready(Ok(()));
        }
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl AsyncWrite for CancellableStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

/// Boucle de service d'un serveur Modbus TCP déjà lié à un `listener`.
///
/// Politique **mono-maître** : une seule connexion distante est servie à la fois.
/// À l'arrivée d'un nouveau maître (IP autorisée par la liste blanche), la
/// connexion précédente est **fermée**. Ne retourne qu'en cas d'erreur fatale.
pub async fn serve(
    listener: TcpListener,
    service: RegulatorService,
    allowlist: SharedAllowlist,
    status: SharedStatus,
) -> std::io::Result<()> {
    let server = Server::new(listener);
    // Émetteur d'éviction de la connexion actuellement servie (politique mono-maître).
    let current: Arc<Mutex<Option<oneshot::Sender<()>>>> = Arc::new(Mutex::new(None));

    let on_connected = move |stream: TcpStream, peer: SocketAddr| {
        let service = service.clone();
        let allowlist = allowlist.clone();
        let current = current.clone();
        let status = status.clone();
        async move {
            let allowed = allowlist
                .lock()
                .map(|f| f.allows(peer.ip()))
                .unwrap_or(true);
            if !allowed {
                log::warn!("Connection refused from {peer} (not in allowlist)");
                return Ok(None);
            }
            // Évince le maître précédent puis enregistre le nouveau.
            let (kick_tx, kick_rx) = oneshot::channel();
            if let Ok(mut guard) = current.lock() {
                if let Some(prev) = guard.take() {
                    let _ = prev.send(()); // ferme l'ancienne connexion
                    log::info!("New master {peer} — disconnecting previous master");
                } else {
                    log::info!("Master connected: {peer}");
                }
                *guard = Some(kick_tx);
            }
            // Publie le maître courant pour le voyant de connexion de l'IHM
            // (mise à jour partielle du statut).
            if let Ok(mut s) = status.lock() {
                s.peer = Some(peer.to_string());
            }
            Ok(Some((service, CancellableStream::new(stream, kick_rx))))
        }
    };
    let on_process_error = |err| {
        log::error!("Modbus processing error: {err}");
    };

    server.serve(&on_connected, on_process_error).await
}

/// Boucle de service d'un serveur Modbus **RTU** sur une liaison série déjà ouverte.
///
/// Le bus RS485 étant un média partagé, il n'y a pas de notion de connexion à
/// évincer : la liaison série *est* l'unique maître. ⚠️ Le serveur RTU de
/// `tokio-modbus` répond **quelle que soit l'adresse esclave** demandée (l'adresse
/// n'est pas transmise au service) : privilégier une liaison **point-à-point**.
#[cfg(feature = "rtu")]
pub async fn serve_rtu(
    serial: tokio_serial::SerialStream,
    service: RegulatorService,
) -> std::io::Result<()> {
    use tokio_modbus::server::rtu::Server;
    Server::new(serial).serve_forever(service).await
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use ractor::Actor;
    use tokio::io::AsyncReadExt;
    use tokio::net::TcpStream;

    use super::*;
    use crate::actors::{SimulationActor, SimulationArgs};
    use crate::config::{IpFilter, ServerStatus};
    use crate::map::MemoryMap;
    use crate::regulator::{Regulator, RegulatorConfig};

    #[tokio::test]
    async fn second_master_kicks_first() {
        let reg_cfg = RegulatorConfig::default();
        let snapshot = Arc::new(Mutex::new(Regulator::new(reg_cfg.clone()).snapshot()));
        let map = Arc::new(Mutex::new(MemoryMap::default()));
        let allowlist = Arc::new(Mutex::new(IpFilter::default()));
        let status = Arc::new(Mutex::new(ServerStatus::default()));

        let (sim, _sj) = Actor::spawn(
            None,
            SimulationActor,
            SimulationArgs {
                config: reg_cfg,
                snapshot: snapshot.clone(),
                map: map.clone(),
            },
        )
        .await
        .unwrap();

        let service = RegulatorService::new(sim.clone(), map, snapshot, status.clone());
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(serve(listener, service, allowlist, status));

        // Premier maître.
        let mut a = TcpStream::connect(addr).await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Deuxième maître : doit évincer le premier.
        let _b = TcpStream::connect(addr).await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        // La connexion du premier maître est fermée -> lecture = 0 octet (EOF).
        let mut buf = [0u8; 8];
        let n = a.read(&mut buf).await.unwrap();
        assert_eq!(n, 0, "le premier maître doit être déconnecté à l'arrivée du second");

        sim.stop(None);
    }

    #[tokio::test]
    async fn read_request_marks_activity() {
        // Toute requête traitée doit horodater le témoin d'activité du lien.
        let reg_cfg = RegulatorConfig::default();
        let snapshot = Arc::new(Mutex::new(Regulator::new(reg_cfg.clone()).snapshot()));
        let map = Arc::new(Mutex::new(MemoryMap::default()));
        let status = Arc::new(Mutex::new(ServerStatus::default()));

        let (sim, _sj) = Actor::spawn(
            None,
            SimulationActor,
            SimulationArgs {
                config: reg_cfg,
                snapshot: snapshot.clone(),
                map: map.clone(),
            },
        )
        .await
        .unwrap();

        let service = RegulatorService::new(sim.clone(), map, snapshot, status.clone());
        assert!(status.lock().unwrap().last_request.is_none());
        // Une simple lecture des registres d'entrée suffit à marquer l'activité.
        let _ = service.handle(Request::ReadInputRegisters(crate::map::IR_PV, 2));
        assert!(status.lock().unwrap().last_request.is_some());

        sim.stop(None);
    }
}
