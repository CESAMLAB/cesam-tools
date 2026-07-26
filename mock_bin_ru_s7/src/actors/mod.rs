//! Acteurs `ractor` du régulateur S7.
//!
//! Un acteur unique ([`simulation::SimulationActor`]) possède l'état métier
//! ([`crate::regulator::Regulator`]) ; toutes les mutations passent par messages.
//! Un instantané partagé ([`SharedSnapshot`]) est rafraîchi à chaque pas et lu par
//! l'IHM **et** par les sessions du serveur S7.

use std::sync::{Arc, Mutex};

use crate::config::{Allowlist, ServerStatus};
use crate::regulator::Snapshot;

pub mod network;
pub mod simulation;

pub use network::{S7ServerActor, S7ServerArgs, S7ServerMsg};
pub use simulation::{SimulationActor, SimulationArgs, SimulationMsg};

/// Instantané typé de l'état, partagé avec l'IHM et le serveur S7.
pub type SharedSnapshot = Arc<Mutex<Snapshot>>;

/// État du serveur S7, partagé avec l'IHM pour affichage.
pub type SharedStatus = Arc<Mutex<ServerStatus>>;

/// Liste blanche d'IP partagée avec les sessions S7 (lecture seule côté session).
pub type SharedAllowlist = Arc<Mutex<Allowlist>>;
