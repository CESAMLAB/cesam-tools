//! Acteurs `ractor` du régulateur S7.
//!
//! Un acteur unique ([`mock_lib_regulator::SimulationActor`]) possède l'état
//! métier ([`mock_lib_regulator::Regulator`]) ; toutes les mutations passent par
//! messages. Un instantané partagé ([`SharedSnapshot`]) est rafraîchi à chaque
//! pas et lu par l'IHM **et** par les sessions du serveur S7.

use std::sync::{Arc, Mutex};

use crate::config::{Allowlist, ServerStatus};

pub mod network;

pub use mock_lib_regulator::{SharedSnapshot, SimulationActor, SimulationArgs, SimulationMsg};
pub use network::{S7ServerActor, S7ServerArgs, S7ServerMsg};

/// État du serveur S7, partagé avec l'IHM pour affichage.
pub type SharedStatus = Arc<Mutex<ServerStatus>>;

/// Liste blanche d'IP partagée avec les sessions S7 (lecture seule côté session).
pub type SharedAllowlist = Arc<Mutex<Allowlist>>;
