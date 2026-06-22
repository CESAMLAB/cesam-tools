//! Configuration de l'application : réseau/série, moteur et régulation, avec
//! persistance TOML. Toute valeur issue du fichier est **assainie** au chargement
//! ([`AppConfig::sanitized`]) pour éviter tout `panic!` (`f32::clamp`) ou valeur
//! aberrante.

use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::time::Instant;

use mock_lib_control::PidConfig;
use serde::{Deserialize, Serialize};

use crate::i18n::Lang;
use crate::stirrer::{StirrerConfig, DEFAULT_DT};

const DEFAULT_CONFIG_FILE: &str = "mock_su_namur.toml";

/// Transport de la liaison NAMUR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Transport {
    /// NAMUR ASCII sur TCP (pratique pour tester sans matériel).
    #[default]
    Tcp,
    /// NAMUR ASCII sur liaison série RS-232 (feature `serial`).
    Serial,
}

/// Parité d'une liaison série.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Parity {
    None,
    /// Parité paire — réglage NAMUR le plus courant (7E1).
    #[default]
    Even,
    Odd,
}

impl Parity {
    #[must_use]
    pub fn code(self) -> char {
        match self {
            Parity::None => 'N',
            Parity::Even => 'E',
            Parity::Odd => 'O',
        }
    }
}

/// Paramètres d'une liaison série RS-232.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SerialConfig {
    pub port: String,
    pub baud: u32,
    pub parity: Parity,
    pub data_bits: u8,
    pub stop_bits: u8,
}

impl Default for SerialConfig {
    fn default() -> Self {
        // Réglage NAMUR de labo typique : 9600 bauds, 7E1.
        Self {
            port: default_serial_port(),
            baud: 9600,
            parity: Parity::Even,
            data_bits: 7,
            stop_bits: 1,
        }
    }
}

impl SerialConfig {
    #[must_use]
    pub fn describe(&self) -> String {
        format!(
            "{} @{} {}{}{}",
            self.port,
            self.baud,
            self.data_bits,
            self.parity.code(),
            self.stop_bits
        )
    }
}

#[cfg(feature = "serial")]
impl SerialConfig {
    /// Ouvre la liaison série RS-232 en mode asynchrone.
    pub fn open(&self) -> std::io::Result<tokio_serial::SerialStream> {
        use tokio_serial::{DataBits, Parity as SParity, SerialPortBuilderExt, StopBits};

        let parity = match self.parity {
            Parity::None => SParity::None,
            Parity::Even => SParity::Even,
            Parity::Odd => SParity::Odd,
        };
        let data_bits = match self.data_bits {
            5 => DataBits::Five,
            6 => DataBits::Six,
            7 => DataBits::Seven,
            _ => DataBits::Eight,
        };
        let stop_bits = if self.stop_bits >= 2 {
            StopBits::Two
        } else {
            StopBits::One
        };
        let stream = tokio_serial::new(&self.port, self.baud)
            .parity(parity)
            .data_bits(data_bits)
            .stop_bits(stop_bits)
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

/// Paramètres réseau / liaison du serveur NAMUR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct NetworkConfig {
    pub transport: Transport,
    pub bind_ip: String,
    pub port: u16,
    /// Motifs d'IP autorisées (jokers `*`, TCP uniquement). Vide = toutes.
    pub allowlist: Vec<String>,
    pub serial: SerialConfig,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            transport: Transport::default(),
            bind_ip: "0.0.0.0".to_string(),
            port: 4001,
            allowlist: Vec::new(),
            serial: SerialConfig::default(),
        }
    }
}

impl NetworkConfig {
    pub fn socket_addr(&self) -> Result<SocketAddr, String> {
        format!("{}:{}", self.bind_ip, self.port)
            .parse()
            .map_err(|e| format!("invalid listen address ({}:{}): {e}", self.bind_ip, self.port))
    }
}

/// Paramètres du moteur simulé (fonction de transfert rotationnelle).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MotorConfig {
    /// Inertie / réactivité (petit = rapide).
    pub inertia: f32,
    /// Coefficient de charge visqueuse.
    pub load_coeff: f32,
    /// Frottement sec résiduel (N·cm).
    pub friction: f32,
    /// Couple moteur maximal (N·cm).
    pub torque_max: f32,
}

impl Default for MotorConfig {
    fn default() -> Self {
        let s = StirrerConfig::default();
        Self {
            inertia: s.inertia,
            load_coeff: s.load_coeff,
            friction: s.friction,
            torque_max: s.torque_max,
        }
    }
}

/// Paramètres de régulation / d'exploitation persistés.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RegulationConfig {
    pub speed_min: f32,
    pub speed_max: f32,
    pub pid: PidConfig,
    pub viscosity: f32,
    pub viscosity_min: f32,
    pub viscosity_max: f32,
}

impl Default for RegulationConfig {
    fn default() -> Self {
        let s = StirrerConfig::default();
        Self {
            speed_min: s.speed_min,
            speed_max: s.speed_max,
            pid: s.pid,
            viscosity: s.viscosity,
            viscosity_min: s.viscosity_min,
            viscosity_max: s.viscosity_max,
        }
    }
}

/// Configuration complète de l'application.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub language: Lang,
    pub network: NetworkConfig,
    pub motor: MotorConfig,
    pub regulation: RegulationConfig,
    /// Vérifier au démarrage si une version plus récente est publiée (feature
    /// `gui`). Activé par défaut ; désactivable depuis le modal *Paramètres*.
    /// `#[serde(default)]` du conteneur le ramène à `true` pour les anciens TOML.
    pub check_updates: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            language: Lang::default(),
            network: NetworkConfig::default(),
            motor: MotorConfig::default(),
            regulation: RegulationConfig::default(),
            check_updates: true,
        }
    }
}

impl AppConfig {
    /// Traduit la configuration en [`StirrerConfig`] pour l'acteur de simulation.
    pub fn to_stirrer_config(&self) -> StirrerConfig {
        let mut pid = self.regulation.pid;
        pid.out_min = 0.0;
        pid.out_max = self.motor.torque_max;
        StirrerConfig {
            dt: DEFAULT_DT,
            speed_min: self.regulation.speed_min,
            speed_max: self.regulation.speed_max,
            torque_max: self.motor.torque_max,
            pid,
            inertia: self.motor.inertia,
            load_coeff: self.motor.load_coeff,
            friction: self.motor.friction,
            viscosity: self.regulation.viscosity,
            viscosity_min: self.regulation.viscosity_min,
            viscosity_max: self.regulation.viscosity_max,
        }
    }

    /// Assainit les valeurs numériques issues du TOML (anti-panic / anti-aberration).
    #[must_use]
    pub fn sanitized(mut self) -> Self {
        let before = self.clone();
        let dm = MotorConfig::default();
        let dr = RegulationConfig::default();

        // Moteur.
        self.motor.inertia = finite_at_least(self.motor.inertia, 1e-4, dm.inertia);
        self.motor.load_coeff = finite_at_least(self.motor.load_coeff, 0.0, dm.load_coeff);
        self.motor.friction = finite_at_least(self.motor.friction, 0.0, dm.friction);
        self.motor.torque_max = finite_at_least(self.motor.torque_max, 1e-3, dm.torque_max);

        // Bornes de vitesse (finies puis ordonnées).
        let mut s_min = finite_or(self.regulation.speed_min, dr.speed_min);
        let mut s_max = finite_or(self.regulation.speed_max, dr.speed_max);
        if s_min > s_max {
            std::mem::swap(&mut s_min, &mut s_max);
        }
        self.regulation.speed_min = s_min.max(0.0);
        self.regulation.speed_max = s_max.max(0.0);

        // Bornes de viscosité (finies, strictement positives, ordonnées).
        let mut v_min = finite_at_least(self.regulation.viscosity_min, 1e-3, dr.viscosity_min);
        let mut v_max = finite_at_least(self.regulation.viscosity_max, 1e-3, dr.viscosity_max);
        if v_min > v_max {
            std::mem::swap(&mut v_min, &mut v_max);
        }
        self.regulation.viscosity_min = v_min;
        self.regulation.viscosity_max = v_max;
        self.regulation.viscosity =
            finite_or(self.regulation.viscosity, dr.viscosity).clamp(v_min, v_max);

        // Gains et bornes PID.
        self.regulation.pid = sanitize_pid(self.regulation.pid, dr.pid);

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

/// Filtre d'adresses IP (jokers `*` par octet, IPv4). Les adresses IPv4-mapped
/// IPv6 (`::ffff:a.b.c.d`) sont ramenées à leur IPv4 avant comparaison.
#[derive(Debug, Clone, Default)]
pub struct IpFilter {
    patterns: Vec<String>,
}

impl IpFilter {
    #[must_use]
    pub fn new(patterns: Vec<String>) -> Self {
        Self {
            patterns: patterns
                .into_iter()
                .map(|p| p.trim().to_string())
                .filter(|p| !p.is_empty())
                .collect(),
        }
    }

    #[must_use]
    pub fn allows(&self, ip: IpAddr) -> bool {
        if self.patterns.is_empty() {
            return true;
        }
        self.patterns.iter().any(|pat| pattern_matches(pat, ip))
    }
}

fn pattern_matches(pattern: &str, ip: IpAddr) -> bool {
    let ip = match ip {
        IpAddr::V6(v6) => v6.to_ipv4_mapped().map_or(IpAddr::V6(v6), IpAddr::V4),
        v4 => v4,
    };
    match ip {
        IpAddr::V4(v4) => {
            let parts: Vec<&str> = pattern.split('.').collect();
            if parts.len() != 4 {
                return pattern == ip.to_string();
            }
            let octets = v4.octets();
            parts
                .iter()
                .zip(octets.iter())
                .all(|(p, o)| *p == "*" || p.parse::<u8>().map(|n| n == *o).unwrap_or(false))
        }
        IpAddr::V6(_) => pattern == ip.to_string(),
    }
}

/// État courant du serveur NAMUR, partagé avec l'IHM pour affichage.
#[derive(Debug, Clone, Default)]
pub struct ServerStatus {
    pub listening: bool,
    pub addr: String,
    pub error: Option<String>,
    /// Adresse du maître TCP connecté (None en série ou si aucun).
    pub peer: Option<String>,
    /// Instant de la dernière requête NAMUR traitée (témoin d'activité du lien).
    pub last_request: Option<Instant>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_allowlist_allows_all() {
        assert!(IpFilter::new(vec![]).allows("8.8.8.8".parse().unwrap()));
    }

    #[test]
    fn wildcard_and_ipv4_mapped() {
        let f = IpFilter::new(vec!["192.168.1.*".to_string()]);
        assert!(f.allows("192.168.1.42".parse().unwrap()));
        assert!(!f.allows("192.168.2.42".parse().unwrap()));
        assert!(f.allows("::ffff:192.168.1.42".parse().unwrap()));
    }

    #[test]
    fn pure_ipv6_matches_exact_pattern_only() {
        // Une IPv6 non mappée se compare à l'identique (pas de jokers par octet).
        let f = IpFilter::new(vec!["::1".to_string()]);
        assert!(f.allows("::1".parse().unwrap()));
        assert!(!f.allows("::2".parse().unwrap()));
        // Un motif IPv4 ne matche pas une IPv6 pure (fail-closed).
        let f4 = IpFilter::new(vec!["10.0.0.*".to_string()]);
        assert!(!f4.allows("fe80::1".parse().unwrap()));
    }

    #[test]
    fn sanitize_pid_orders_bounds_and_drops_non_finite() {
        let dflt = PidConfig::default();
        let cfg = sanitize_pid(
            PidConfig {
                kp: f32::NAN,
                ki: -3.0,
                kd: f32::INFINITY,
                out_min: 100.0,
                out_max: 0.0,
            },
            dflt,
        );
        assert_eq!(cfg.kp, dflt.kp); // NaN -> défaut
        assert_eq!(cfg.ki, 0.0); // négatif -> 0
        assert_eq!(cfg.kd, dflt.kd); // Inf -> défaut
        assert!(cfg.out_min <= cfg.out_max); // bornes réordonnées
    }

    #[test]
    fn config_round_trips_through_toml() {
        let cfg = AppConfig::default();
        let s = toml::to_string_pretty(&cfg).unwrap();
        let back: AppConfig = toml::from_str(&s).unwrap();
        assert_eq!(cfg, back);
    }

    #[test]
    fn sanitized_orders_inverted_bounds_without_panic() {
        let mut cfg = AppConfig::default();
        cfg.regulation.speed_min = 2000.0;
        cfg.regulation.speed_max = 0.0;
        cfg.motor.inertia = f32::NAN;
        cfg.regulation.viscosity = f32::INFINITY;
        let cfg = cfg.sanitized();
        assert!(cfg.regulation.speed_min <= cfg.regulation.speed_max);
        assert!(cfg.motor.inertia.is_finite() && cfg.motor.inertia >= 1e-4);
        assert!(cfg.regulation.viscosity.is_finite());
        // Ne panique pas en construisant l'agitateur.
        let _ = crate::stirrer::Stirrer::new(cfg.to_stirrer_config());
    }

    #[cfg(feature = "serial")]
    #[test]
    fn serial_open_on_missing_port_errors() {
        let cfg = SerialConfig {
            port: "/dev/cesam_inexistant_42".to_string(),
            ..SerialConfig::default()
        };
        assert!(cfg.open().is_err());
    }
}
