//! Acteurs `ractor` du régulateur OPC UA.
//!
//! Un acteur unique ([`simulation::SimulationActor`]) possède l'état métier
//! ([`crate::regulator::Regulator`]) ; toutes les mutations passent par messages.
//! Un instantané partagé ([`SharedSnapshot`]) est rafraîchi à chaque pas et lu par
//! l'IHM **et** par les callbacks du serveur OPC UA.

use std::sync::{Arc, Mutex};

use crate::config::ServerStatus;
use crate::regulator::Snapshot;

pub mod network;
pub mod simulation;

pub use network::{OpcuaServerActor, OpcuaServerArgs, OpcuaServerMsg};
pub use simulation::{SimulationActor, SimulationArgs, SimulationMsg};

/// Instantané typé de l'état, partagé avec l'IHM et le serveur OPC UA.
pub type SharedSnapshot = Arc<Mutex<Snapshot>>;

/// État du serveur OPC UA, partagé avec l'IHM pour affichage.
pub type SharedStatus = Arc<Mutex<ServerStatus>>;
