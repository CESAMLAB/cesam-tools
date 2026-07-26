//! Acteurs `ractor` du régulateur Sparkplug B.
//!
//! Un acteur unique ([`simulation::SimulationActor`]) possède l'état métier
//! ([`crate::regulator::Regulator`]) ; toutes les mutations passent par messages.
//! Un instantané partagé ([`SharedSnapshot`]) est rafraîchi à chaque pas et lu par
//! l'IHM **et** par la tâche de publication de l'edge node Sparkplug B.

use std::sync::{Arc, Mutex};

use crate::config::ServerStatus;
use crate::regulator::Snapshot;

pub mod network;
pub mod simulation;

pub use network::{SparkplugActor, SparkplugArgs, SparkplugMsg};
pub use simulation::{SimulationActor, SimulationArgs, SimulationMsg};

/// Instantané typé de l'état, partagé avec l'IHM et l'edge node Sparkplug B.
pub type SharedSnapshot = Arc<Mutex<Snapshot>>;

/// État de connexion de l'edge node Sparkplug B, partagé avec l'IHM pour affichage.
pub type SharedStatus = Arc<Mutex<ServerStatus>>;
