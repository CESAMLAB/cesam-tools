//! Configuration de l'application : liaison série PROFIBUS DP, procédé et
//! régulation, avec persistance au format TOML.

use std::path::{Path, PathBuf};
use std::time::Instant;

use mock_lib_control::PidConfig;
use serde::{Deserialize, Serialize};

use crate::i18n::Lang;
use crate::regulator::RegulatorConfig;

/// Nom de fichier de configuration par défaut (dans le répertoire courant),
/// surchargeable via la variable d'environnement `MOCK_CONFIG`.
const DEFAULT_CONFIG_FILE: &str = "mock_ru_pbdp.toml";

/// Borne supérieure du retard pur (s) tolérée en configuration.
const MAX_DEAD_TIME: f32 = 100_000.0;

/// Adresse de station PROFIBUS maximale (0-125 ; 126/127 réservées par la norme).
const MAX_STATION_ADDRESS: u8 = 125;

#[inline]
fn finite_or(v: f32, default: f32) -> f32 {
    if v.is_finite() {
        v
    } else {
        default
    }
}

#[inline]
fn finite_at_least(v: f32, min: f32, default: f32) -> f32 {
    if v.is_finite() {
        v.max(min)
    } else {
        default
    }
}

#[must_use]
fn sanitize_pid(mut cfg: PidConfig, default: PidConfig) -> PidConfig {
    cfg.kp = finite_at_least(cfg.kp, 0.0, default.kp);
    cfg.ki = finite_at_least(cfg.ki, 0.0, default.ki);
    cfg.kd = finite_at_least(cfg.kd, 0.0, default.kd);
    let mut out_min = finite_or(cfg.out_min, default.out_min);
    let mut out_max = finite_or(cfg.out_max, default.out_max);
    if out_min > out_max {
        std::mem::swap(&mut out_min, &mut out_max);
    }
    cfg.out_min = out_min;
    cfg.out_max = out_max;
    cfg
}

/// Paramètres de la liaison série PROFIBUS DP (RS-485).
///
/// La parité/format de trame (8E1) est **fixée par la norme PROFIBUS DP** et
/// n'est donc pas configurable ici (contrairement à Modbus RTU/NAMUR série) —
/// seuls le port, le débit (parmi les valeurs standard DP) et l'adresse de
/// station le sont.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SerialConfig {
    /// Chemin du port (`/dev/ttyUSB0`, `/dev/ttyAMA0` sous Linux, `COM3` sous Windows).
    pub port: String,
    /// Débit en bauds — valeur normalisée DP (9600..12_000_000). Non contrôlé à
    /// l'ouverture : un débit non standard est transmis tel quel au port série.
    pub baud: u32,
    /// Adresse de station PROFIBUS de cet esclave (0-125).
    pub station_address: u8,
    /// Chien de garde protocolaire activable par le maître (`Set_Prm`). Ce
    /// réglage local ne fait qu'**autoriser** le chien de garde annoncé par le
    /// maître dans sa trame `Set_Prm` ; il ne l'arme pas lui-même.
    pub watchdog_enabled: bool,
}

impl Default for SerialConfig {
    fn default() -> Self {
        Self {
            port: default_serial_port(),
            baud: 500_000,
            station_address: 3,
            watchdog_enabled: true,
        }
    }
}

impl SerialConfig {
    /// Description courte pour l'IHM / le statut (« /dev/ttyUSB0 @500000 8E1 »).
    #[must_use]
    pub fn describe(&self) -> String {
        format!("{} @{} 8E1 (station {})", self.port, self.baud, self.station_address)
    }
}

impl SerialConfig {
    /// Ouvre la liaison série RS-485 en mode asynchrone. Format de trame **8E1**
    /// fixé (norme PROFIBUS DP), non paramétrable.
    pub fn open(&self) -> std::io::Result<tokio_serial::SerialStream> {
        use tokio_serial::{DataBits, Parity, SerialPortBuilderExt, StopBits};

        let stream = tokio_serial::new(&self.port, self.baud)
            .parity(Parity::Even)
            .data_bits(DataBits::Eight)
            .stop_bits(StopBits::One)
            .open_native_async()?;
        Ok(stream)
    }
}

fn default_serial_port() -> String {
    if cfg!(windows) {
        "COM3".to_string()
    } else {
        "/dev/ttyUSB0".to_string()
    }
}

/// Paramètres réseau/liaison du simulateur PROFIBUS DP : uniquement la liaison
/// série (pas de transport TCP — PROFIBUS DP n'a pas d'équivalent standard, et en
/// inventer un serait trompeur ; pas de liste blanche d'IP non plus, la liaison
/// série étant intrinsèquement point-à-point).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct NetworkConfig {
    pub serial: SerialConfig,
}

/// Paramètres de la fonction de transfert du procédé simulé.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProcessConfig {
    pub gain: f32,
    pub tau: f32,
    pub dead_time: f32,
    pub ambient: f32,
}

impl Default for ProcessConfig {
    fn default() -> Self {
        let r = RegulatorConfig::default();
        Self {
            gain: r.process_gain,
            tau: r.process_tau,
            dead_time: r.process_dead_time,
            ambient: r.ambient,
        }
    }
}

/// Paramètres de régulation persistés.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RegulationConfig {
    pub sp_min: f32,
    pub sp_max: f32,
    pub pid_heat: PidConfig,
    pub pid_cool: PidConfig,
    pub hysteresis: f32,
    pub tor_min_cycle: f32,
    pub pwm_period: f32,
}

impl Default for RegulationConfig {
    fn default() -> Self {
        let r = RegulatorConfig::default();
        Self {
            sp_min: r.sp_min,
            sp_max: r.sp_max,
            pid_heat: r.pid_heat,
            pid_cool: r.pid_cool,
            hysteresis: r.hysteresis,
            tor_min_cycle: r.tor_min_cycle,
            pwm_period: r.pwm_period,
        }
    }
}

/// Configuration complète de l'application.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub language: Lang,
    pub network: NetworkConfig,
    pub process: ProcessConfig,
    pub regulation: RegulationConfig,
    pub check_updates: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            language: Lang::default(),
            network: NetworkConfig::default(),
            process: ProcessConfig::default(),
            regulation: RegulationConfig::default(),
            check_updates: true,
        }
    }
}

impl AppConfig {
    /// Traduit la configuration en [`RegulatorConfig`] pour l'acteur de simulation.
    pub fn to_regulator_config(&self) -> RegulatorConfig {
        RegulatorConfig {
            dt: crate::regulator::DEFAULT_DT,
            sp_min: self.regulation.sp_min,
            sp_max: self.regulation.sp_max,
            pid_heat: self.regulation.pid_heat,
            pid_cool: self.regulation.pid_cool,
            hysteresis: self.regulation.hysteresis,
            tor_min_cycle: self.regulation.tor_min_cycle,
            pwm_period: self.regulation.pwm_period,
            process_gain: self.process.gain,
            process_tau: self.process.tau,
            process_dead_time: self.process.dead_time,
            ambient: self.process.ambient,
        }
    }

    /// Assainit les valeurs issues d'une source non fiable (TOML édité à la
    /// main) : bornes réordonnées et finies, gains PID finis ≥ 0, adresse de
    /// station bornée à `[0, 125]`. Journalise un `warn!` si une correction a
    /// été nécessaire.
    #[must_use]
    pub fn sanitized(mut self) -> Self {
        let before = self.clone();
        let dp = ProcessConfig::default();
        let dr = RegulationConfig::default();

        self.process.gain = finite_or(self.process.gain, dp.gain);
        self.process.tau = finite_at_least(self.process.tau, 1e-3, dp.tau);
        self.process.dead_time = if self.process.dead_time.is_finite() {
            self.process.dead_time.clamp(0.0, MAX_DEAD_TIME)
        } else {
            dp.dead_time
        };
        self.process.ambient = finite_or(self.process.ambient, dp.ambient);

        let mut sp_min = finite_or(self.regulation.sp_min, dr.sp_min);
        let mut sp_max = finite_or(self.regulation.sp_max, dr.sp_max);
        if sp_min > sp_max {
            std::mem::swap(&mut sp_min, &mut sp_max);
        }
        self.regulation.sp_min = sp_min;
        self.regulation.sp_max = sp_max;

        self.regulation.pid_heat = sanitize_pid(self.regulation.pid_heat, dr.pid_heat);
        self.regulation.pid_cool = sanitize_pid(self.regulation.pid_cool, dr.pid_cool);

        self.regulation.hysteresis = finite_at_least(self.regulation.hysteresis, 0.0, dr.hysteresis);
        self.regulation.tor_min_cycle =
            finite_at_least(self.regulation.tor_min_cycle, 0.0, dr.tor_min_cycle);
        self.regulation.pwm_period = finite_at_least(self.regulation.pwm_period, 1e-3, dr.pwm_period);

        self.network.serial.station_address =
            self.network.serial.station_address.min(MAX_STATION_ADDRESS);

        if self != before {
            log::warn!("Configuration sanitized: out-of-range or non-finite values were corrected");
        }
        self
    }

    pub fn path() -> PathBuf {
        std::env::var_os("MOCK_CONFIG")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_FILE))
    }

    pub fn load(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(content) => match toml::from_str::<Self>(&content) {
                Ok(cfg) => {
                    log::info!("Configuration loaded from {}", path.display());
                    cfg.sanitized()
                }
                Err(e) => {
                    log::warn!("Configuration unreadable ({e}) — using default values");
                    Self::default()
                }
            },
            Err(_) => {
                log::info!("No configuration file ({}) — using default values", path.display());
                Self::default()
            }
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let content = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, content).map_err(|e| e.to_string())?;
        log::info!("Configuration saved to {}", path.display());
        Ok(())
    }
}

/// État courant du serveur PROFIBUS, partagé avec l'IHM pour affichage.
#[derive(Debug, Clone, Default)]
pub struct ServerStatus {
    pub listening: bool,
    pub addr: String,
    pub error: Option<String>,
    /// État courant de la machine à états DP-V0 (texte pour affichage IHM).
    pub state: Option<String>,
    /// Instant de la dernière requête PROFIBUS traitée (témoin d'activité du lien).
    pub last_request: Option<Instant>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_round_trips_through_toml() {
        let cfg = AppConfig::default();
        let s = toml::to_string_pretty(&cfg).unwrap();
        let back: AppConfig = toml::from_str(&s).unwrap();
        assert_eq!(cfg, back);
    }

    #[test]
    fn sanitized_orders_inverted_setpoint_bounds_without_panic() {
        let mut cfg = AppConfig::default();
        cfg.regulation.sp_min = 300.0;
        cfg.regulation.sp_max = 0.0;
        let cfg = cfg.sanitized();
        assert!(cfg.regulation.sp_min <= cfg.regulation.sp_max);
        let _ = crate::regulator::Regulator::new(cfg.to_regulator_config());
    }

    #[test]
    fn sanitized_replaces_non_finite_values() {
        let mut cfg = AppConfig::default();
        cfg.process.tau = f32::NAN;
        cfg.process.dead_time = f32::INFINITY;
        cfg.regulation.sp_min = f32::NAN;
        cfg.regulation.hysteresis = f32::NEG_INFINITY;
        let cfg = cfg.sanitized();
        assert!(cfg.process.tau.is_finite() && cfg.process.tau >= 1e-3);
        assert!(cfg.process.dead_time.is_finite() && cfg.process.dead_time <= MAX_DEAD_TIME);
        assert!(cfg.regulation.sp_min.is_finite());
        assert!(cfg.regulation.hysteresis.is_finite() && cfg.regulation.hysteresis >= 0.0);
    }

    #[test]
    fn sanitized_clamps_station_address() {
        let mut cfg = AppConfig::default();
        cfg.network.serial.station_address = 200;
        let cfg = cfg.sanitized();
        assert!(cfg.network.serial.station_address <= MAX_STATION_ADDRESS);
    }

    #[test]
    fn sanitized_is_noop_on_defaults() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.clone(), cfg.sanitized());
    }

    #[test]
    fn serial_open_on_missing_port_errors() {
        let cfg = SerialConfig {
            port: "/dev/cesam_inexistant_42".to_string(),
            ..SerialConfig::default()
        };
        assert!(cfg.open().is_err());
    }
}
