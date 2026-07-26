//! Acteurs `ractor` du régulateur.
//!
//! L'architecture asynchrone repose sur un acteur unique propriétaire de l'état
//! métier ([`simulation::SimulationActor`]). Toutes les mutations passent par des
//! messages, ce qui élimine les accès concurrents sur le [`crate::regulator::Regulator`].
//!
//! Deux structures partagées (en lecture) sont rafraîchies à chaque pas :
//! - un [`SharedSnapshot`] typé, lu par l'IHM ;
//! - une [`SharedMap`] (image Modbus), lue par le serveur Modbus.

use std::sync::{Arc, Mutex};

use crate::config::{IpFilter, ServerStatus};
use crate::map::MemoryMap;
use crate::regulator::RegulatorSnapshot;

pub mod network;
pub mod simulation;

pub use network::{ModbusServerActor, ModbusServerArgs, ModbusServerMsg};
pub use simulation::{SimulationActor, SimulationArgs, SimulationMsg};

/// Instantané typé de l'état, partagé avec l'IHM.
pub type SharedSnapshot = Arc<Mutex<RegulatorSnapshot>>;

/// Image mémoire Modbus, partagée avec le serveur Modbus.
pub type SharedMap = Arc<Mutex<MemoryMap>>;

/// Liste blanche d'IP, partagée et modifiable à chaud.
pub type SharedAllowlist = Arc<Mutex<IpFilter>>;

/// État du serveur Modbus, partagé avec l'IHM pour affichage.
pub type SharedStatus = Arc<Mutex<ServerStatus>>;
