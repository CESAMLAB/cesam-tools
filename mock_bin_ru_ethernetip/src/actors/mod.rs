//! Acteurs `ractor` du régulateur EtherNet/IP.
//!
//! Un acteur unique ([`simulation::SimulationActor`]) possède l'état métier
//! ([`crate::regulator::Regulator`]) ; toutes les mutations passent par messages.
//! Un instantané partagé ([`SharedSnapshot`]) est rafraîchi à chaque pas et lu par
//! l'IHM **et** par les sessions du serveur EtherNet/IP.

use std::sync::{Arc, Mutex};

use crate::config::{Allowlist, ServerStatus};
use crate::regulator::Snapshot;

pub mod network;
pub mod simulation;

pub use network::{EipServerActor, EipServerArgs, EipServerMsg};
pub use simulation::{SimulationActor, SimulationArgs, SimulationMsg};

/// Instantané typé de l'état, partagé avec l'IHM et le serveur EtherNet/IP.
pub type SharedSnapshot = Arc<Mutex<Snapshot>>;

/// État du serveur EtherNet/IP, partagé avec l'IHM pour affichage.
pub type SharedStatus = Arc<Mutex<ServerStatus>>;

/// Liste blanche d'IP partagée avec les sessions S7 (lecture seule côté session).
pub type SharedAllowlist = Arc<Mutex<Allowlist>>;
