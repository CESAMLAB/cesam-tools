//! Configuration de l'application : paramètres réseau, procédé et régulation,
//! avec persistance au format TOML.

use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};

use mock_lib_control::PidConfig;
use serde::{Deserialize, Serialize};

use crate::i18n::Lang;
use crate::regulator::RegulatorConfig;

/// Nom de fichier de configuration par défaut (dans le répertoire courant),
/// surchargeable via la variable d'environnement `MOCK_CONFIG`.
const DEFAULT_CONFIG_FILE: &str = "mock_ru_modbustcp.toml";

/// Transport Modbus utilisé pour le maître distant.
///
/// L'appareil fonctionne avec **un seul transport à la fois** (comme un instrument
/// de terrain où l'on choisit le bus) : Modbus **TCP** (Ethernet) ou Modbus **RTU**
/// série (RS485).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Transport {
    /// Modbus TCP (Ethernet) — politique mono-maître avec éviction.
    #[default]
    Tcp,
    /// Modbus RTU sur liaison série RS485 (feature `rtu`).
    Rtu,
}

/// Parité d'une liaison série.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Parity {
    None,
    /// Parité paire — réglage Modbus RTU le plus courant.
    #[default]
    Even,
    Odd,
}

impl Parity {
    /// Code court pour l'affichage (`N`/`E`/`O`), façon « 8E1 ».
    #[must_use]
    pub fn code(self) -> char {
        match self {
            Parity::None => 'N',
            Parity::Even => 'E',
            Parity::Odd => 'O',
        }
    }
}

/// Paramètres d'une liaison série Modbus RTU (RS485).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SerialConfig {
    /// Chemin du port (`/dev/ttyUSB0`, `/dev/ttyAMA0` sous Linux, `COM3` sous Windows).
    pub port: String,
    /// Débit en bauds (ex. 9600, 19200, 115200).
    pub baud: u32,
    /// Parité.
    pub parity: Parity,
    /// Bits de données (7 ou 8).
    pub data_bits: u8,
    /// Bits de stop (1 ou 2).
    pub stop_bits: u8,
    /// Adresse esclave Modbus (1..247) annoncée par l'appareil.
    pub slave_id: u8,
}

impl Default for SerialConfig {
    fn default() -> Self {
        Self {
            // Défaut multi-plateforme raisonnable ; à adapter selon le matériel.
            port: default_serial_port(),
            baud: 19200,
            parity: Parity::Even,
            data_bits: 8,
            stop_bits: 1,
            slave_id: 1,
        }
    }
}

impl SerialConfig {
    /// Description courte pour l'IHM / le statut (« /dev/ttyUSB0 @19200 8E1 »).
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

#[cfg(feature = "rtu")]
impl SerialConfig {
    /// Ouvre la liaison série RS485 en mode asynchrone (Modbus RTU).
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

/// Port série par défaut selon la plateforme.
fn default_serial_port() -> String {
    if cfg!(windows) {
        "COM3".to_string()
    } else {
        "/dev/ttyUSB0".to_string()
    }
}

/// Paramètres réseau / liaison du serveur Modbus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct NetworkConfig {
    /// Transport actif (TCP ou RTU série).
    pub transport: Transport,
    /// Adresse IP d'écoute (`0.0.0.0` = toutes les interfaces).
    pub bind_ip: String,
    /// Port d'écoute Modbus TCP.
    pub port: u16,
    /// Motifs d'IP autorisées (jokers `*` par octet, ex. `192.168.1.*`).
    /// Liste vide = toutes les IP sont autorisées.
    pub allowlist: Vec<String>,
    /// Paramètres de la liaison série RS485 (utilisés si `transport = rtu`).
    pub serial: SerialConfig,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            transport: Transport::default(),
            bind_ip: "0.0.0.0".to_string(),
            port: 5502,
            allowlist: Vec::new(),
            serial: SerialConfig::default(),
        }
    }
}

impl NetworkConfig {
    /// Construit l'adresse socket d'écoute.
    pub fn socket_addr(&self) -> Result<SocketAddr, String> {
        format!("{}:{}", self.bind_ip, self.port)
            .parse()
            .map_err(|e| format!("invalid listen address ({}:{}): {e}", self.bind_ip, self.port))
    }
}

/// Paramètres de la fonction de transfert du procédé simulé.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProcessConfig {
    /// Gain statique `K` (unité de mesure par %).
    pub gain: f32,
    /// Constante de temps `T` (s).
    pub tau: f32,
    /// Retard pur `L` (s).
    pub dead_time: f32,
    /// Valeur ambiante / de repos.
    pub ambient: f32,
}

impl Default for ProcessConfig {
    // Les valeurs par défaut proviennent de la source unique `RegulatorConfig::default`
    // (le domaine métier), afin d'éviter toute duplication de constantes.
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

/// Paramètres de régulation persistés (bornes de consigne, gains, hystérésis).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RegulationConfig {
    pub sp_min: f32,
    pub sp_max: f32,
    pub pid_heat: PidConfig,
    pub pid_cool: PidConfig,
    pub hysteresis: f32,
    /// Temps de cycle minimal des régulateurs TOR (s) : anti-court-cycle.
    pub tor_min_cycle: f32,
    /// Période du cycle de modulation PWM / relais à cycle (s).
    pub pwm_period: f32,
}

impl Default for RegulationConfig {
    // Idem : dérivé de `RegulatorConfig::default` pour une source unique de vérité.
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AppConfig {
    /// Langue de l'interface graphique (par défaut : français).
    pub language: Lang,
    pub network: NetworkConfig,
    pub process: ProcessConfig,
    pub regulation: RegulationConfig,
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

    /// Chemin du fichier de configuration (`MOCK_CONFIG` ou défaut).
    pub fn path() -> PathBuf {
        std::env::var_os("MOCK_CONFIG")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_FILE))
    }

    /// Charge la configuration depuis le fichier ; retombe sur les valeurs par
    /// défaut si le fichier est absent ou illisible (avec un avertissement).
    pub fn load(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(cfg) => {
                    log::info!("Configuration loaded from {}", path.display());
                    cfg
                }
                Err(e) => {
                    log::warn!("Configuration unreadable ({e}) — using default values");
                    Self::default()
                }
            },
            Err(_) => {
                log::info!(
                    "No configuration file ({}) — using default values",
                    path.display()
                );
                Self::default()
            }
        }
    }

    /// Sauvegarde la configuration au format TOML.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let content = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, content).map_err(|e| e.to_string())?;
        log::info!("Configuration saved to {}", path.display());
        Ok(())
    }
}

/// Filtre d'adresses IP basé sur des motifs avec jokers `*` par octet (IPv4).
///
/// Une liste vide autorise toutes les connexions. Pour IPv6, seule l'égalité
/// exacte de la représentation textuelle est gérée.
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

    /// Indique si l'IP est autorisée.
    #[must_use]
    pub fn allows(&self, ip: IpAddr) -> bool {
        if self.patterns.is_empty() {
            return true;
        }
        self.patterns.iter().any(|pat| pattern_matches(pat, ip))
    }
}

/// Teste un motif (`192.168.1.*`, `127.0.0.1`, …) contre une adresse IP.
fn pattern_matches(pattern: &str, ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let parts: Vec<&str> = pattern.split('.').collect();
            if parts.len() != 4 {
                return pattern == ip.to_string();
            }
            let octets = v4.octets();
            parts.iter().zip(octets.iter()).all(|(p, o)| {
                *p == "*" || p.parse::<u8>().map(|n| n == *o).unwrap_or(false)
            })
        }
        IpAddr::V6(_) => pattern == ip.to_string(),
    }
}

/// État courant du serveur Modbus, partagé avec l'IHM pour affichage.
#[derive(Debug, Clone, Default)]
pub struct ServerStatus {
    /// `true` si le serveur écoute effectivement.
    pub listening: bool,
    /// Adresse d'écoute courante (pour affichage).
    pub addr: String,
    /// Dernière erreur réseau, le cas échéant.
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_allowlist_allows_all() {
        let f = IpFilter::new(vec![]);
        assert!(f.allows("8.8.8.8".parse().unwrap()));
    }

    #[test]
    fn wildcard_matches_subnet() {
        let f = IpFilter::new(vec!["192.168.1.*".to_string()]);
        assert!(f.allows("192.168.1.42".parse().unwrap()));
        assert!(!f.allows("192.168.2.42".parse().unwrap()));
    }

    #[test]
    fn exact_ip_matches() {
        let f = IpFilter::new(vec!["127.0.0.1".to_string()]);
        assert!(f.allows("127.0.0.1".parse().unwrap()));
        assert!(!f.allows("127.0.0.2".parse().unwrap()));
    }

    #[test]
    fn config_round_trips_through_toml() {
        let cfg = AppConfig::default();
        let s = toml::to_string_pretty(&cfg).unwrap();
        let back: AppConfig = toml::from_str(&s).unwrap();
        assert_eq!(cfg, back);
    }
}
