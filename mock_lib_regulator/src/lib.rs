//! Régulateur PID générique — état métier, configuration TOML et acteur `ractor` —
//! partagé par les instruments RU « simple PID » du workspace : ORUE (OPC UA),
//! ORSE (Sparkplug B), ORSS (S7comm) et OREE (EtherNet/IP). Ces quatre instruments
//! n'ont aucune nouveauté métier entre eux : seul le transport réseau change
//! (implémenté par chaque binaire dans son propre `*_server.rs`/`network.rs`).
//!
//! ORME (`mock_bin_ru_modbus`), OSNE (`mock_bin_su_namur`) et ORPD
//! (`mock_bin_ru_pbdp`) ont un modèle métier différent (TOR/PWM double sens ou
//! moteur) et restent hors du périmètre de cette crate.

mod config;
mod regulator;
mod simulation;

pub use config::{ProcessConfig, RegulationConfig};
pub use regulator::{Command, Regulator, RegulatorConfig, Snapshot, DEFAULT_DT};
pub use simulation::{SharedSnapshot, SimulationActor, SimulationArgs, SimulationMsg};
