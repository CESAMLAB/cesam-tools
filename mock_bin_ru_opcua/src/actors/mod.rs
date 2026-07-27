//! Acteurs `ractor` du régulateur OPC UA.
//!
//! Un acteur unique ([`mock_lib_regulator::SimulationActor`]) possède l'état
//! métier ([`mock_lib_regulator::Regulator`]) ; toutes les mutations passent par
//! messages. Un instantané partagé ([`SharedSnapshot`]) est rafraîchi à chaque
//! pas et lu par l'IHM **et** par les callbacks du serveur OPC UA.

use std::sync::{Arc, Mutex};

use crate::config::ServerStatus;

pub mod network;

pub use mock_lib_regulator::{SharedSnapshot, SimulationActor, SimulationArgs, SimulationMsg};
pub use network::{OpcuaServerActor, OpcuaServerArgs, OpcuaServerMsg};

/// État du serveur OPC UA, partagé avec l'IHM pour affichage.
pub type SharedStatus = Arc<Mutex<ServerStatus>>;
