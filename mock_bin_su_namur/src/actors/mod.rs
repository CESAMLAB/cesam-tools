//! Acteurs `ractor` de l'agitateur.
//!
//! Un acteur unique ([`simulation::SimulationActor`]) possède l'état métier
//! ([`crate::stirrer::Stirrer`]) ; toutes les mutations passent par messages. Un
//! instantané partagé ([`SharedSnapshot`]) est rafraîchi à chaque pas et lu par
//! l'IHM **et** par le serveur NAMUR (les lectures NAMUR puisent dedans).

use std::sync::{Arc, Mutex};

use crate::config::{IpFilter, ServerStatus};
use crate::stirrer::StirrerSnapshot;

pub mod network;
pub mod simulation;

pub use network::{NamurServerActor, NamurServerArgs, NamurServerMsg};
pub use simulation::{SimulationActor, SimulationArgs, SimulationMsg};

/// Instantané typé de l'état, partagé avec l'IHM et le serveur NAMUR.
pub type SharedSnapshot = Arc<Mutex<StirrerSnapshot>>;

/// Liste blanche d'IP, partagée et modifiable à chaud.
pub type SharedAllowlist = Arc<Mutex<IpFilter>>;

/// État du serveur NAMUR, partagé avec l'IHM pour affichage.
pub type SharedStatus = Arc<Mutex<ServerStatus>>;
