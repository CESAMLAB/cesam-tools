//! Acteurs `ractor` du régulateur EtherNet/IP.
//!
//! Un acteur unique ([`mock_lib_regulator::SimulationActor`]) possède l'état
//! métier ([`mock_lib_regulator::Regulator`]) ; toutes les mutations passent par
//! messages. Un instantané partagé ([`SharedSnapshot`]) est rafraîchi à chaque
//! pas et lu par l'IHM **et** par les sessions du serveur EtherNet/IP.

use std::sync::{Arc, Mutex};

use crate::config::{Allowlist, ServerStatus};

pub mod network;

pub use mock_lib_regulator::{SharedSnapshot, SimulationActor, SimulationArgs, SimulationMsg};
pub use network::{EipServerActor, EipServerArgs, EipServerMsg};

/// État du serveur EtherNet/IP, partagé avec l'IHM pour affichage.
pub type SharedStatus = Arc<Mutex<ServerStatus>>;

/// Liste blanche d'IP partagée avec les sessions EtherNet/IP (lecture seule côté session).
pub type SharedAllowlist = Arc<Mutex<Allowlist>>;
