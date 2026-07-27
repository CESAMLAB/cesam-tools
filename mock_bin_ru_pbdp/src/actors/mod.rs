//! Acteurs `ractor` du régulateur PROFIBUS DP.
//!
//! Un acteur unique ([`simulation::SimulationActor`]) possède l'état métier
//! ([`crate::regulator::Regulator`]) ; toutes les mutations passent par messages.
//! Un instantané partagé ([`SharedSnapshot`]) est rafraîchi à chaque pas et lu par
//! l'IHM **et** par le serveur PROFIBUS (les réponses `Data_Exchange` y puisent).

use std::sync::{Arc, Mutex};

use crate::config::ServerStatus;
use crate::regulator::RegulatorSnapshot;

pub mod network;
pub mod simulation;

pub use network::{ProfibusServerActor, ProfibusServerArgs, ProfibusServerMsg};
pub use simulation::{SimulationActor, SimulationArgs, SimulationMsg};

/// Instantané typé de l'état, partagé avec l'IHM et le serveur PROFIBUS.
pub type SharedSnapshot = Arc<Mutex<RegulatorSnapshot>>;

/// État du serveur PROFIBUS, partagé avec l'IHM pour affichage.
pub type SharedStatus = Arc<Mutex<ServerStatus>>;
