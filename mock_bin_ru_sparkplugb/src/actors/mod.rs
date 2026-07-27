//! Acteurs `ractor` du régulateur Sparkplug B.
//!
//! Un acteur unique ([`mock_lib_regulator::SimulationActor`]) possède l'état
//! métier ([`mock_lib_regulator::Regulator`]) ; toutes les mutations passent par
//! messages. Un instantané partagé ([`SharedSnapshot`]) est rafraîchi à chaque
//! pas et lu par l'IHM **et** par la tâche de publication de l'edge node
//! Sparkplug B.

use std::sync::{Arc, Mutex};

use crate::config::ServerStatus;

pub mod network;

pub use mock_lib_regulator::{SharedSnapshot, SimulationActor, SimulationArgs, SimulationMsg};
pub use network::{SparkplugActor, SparkplugArgs, SparkplugMsg};

/// État de connexion de l'edge node Sparkplug B, partagé avec l'IHM pour affichage.
pub type SharedStatus = Arc<Mutex<ServerStatus>>;
