//! Configuration de l'application : paramètres réseau, procédé et régulation,
//! avec persistance au format TOML.

use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::time::Instant;

use mock_lib_control::PidConfig;
use serde::{Deserialize, Serialize};

use crate::i18n::Lang;
use crate::regulator::RegulatorConfig;

/// Nom de fichier de configuration par défaut (dans le répertoire courant),
/// surchargeable via la variable d'environnement `MOCK_CONFIG`.
const DEFAULT_CONFIG_FILE: &str = "mock_ru_modbustcp.toml";

/// Borne supérieure du retard pur (s) tolérée en configuration : au-delà, la ligne
/// à retard du procédé deviendrait inutilement volumineuse. Aligné sur l'IHM.
const MAX_DEAD_TIME: f32 = 100_000.0;

/// Renvoie `v` s'il est fini, sinon `default`.
#[inline]
fn finite_or(v: f32, default: f32) -> f32 {
    if v.is_finite() {
        v
    } else {
        default
    }
}

/// Renvoie `max(v, min)` si `v` est fini, sinon `default` (supposé valide).
#[inline]
fn finite_at_least(v: f32, min: f32, default: f32) -> f32 {
    if v.is_finite() {
        v.max(min)
    } else {
        default
    }
}

/// Assainit un réglage PID : gains finis ≥ 0, bornes de sortie finies et ordonnées
/// (`out_min <= out_max`) pour ne jamais paniquer dans `f32::clamp`.
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

    /// Assainit les valeurs numériques issues d'une source non fiable (fichier TOML
    /// édité à la main) afin d'éviter tout `panic!` ultérieur — `f32::clamp` exige
    /// `min <= max` et des bornes finies — et toute valeur physiquement aberrante :
    ///
    /// - bornes de consigne **réordonnées** et finies ;
    /// - bornes de sortie PID réordonnées et finies, gains finis ≥ 0 ;
    /// - constante de temps `tau ≥ 1e-3`, retard `dead_time ∈ [0, MAX_DEAD_TIME]` ;
    /// - hystérésis / cycle TOR ≥ 0, période PWM ≥ 1e-3.
    ///
    /// Journalise un `warn!` si une correction a été nécessaire.
    #[must_use]
    pub fn sanitized(mut self) -> Self {
        let before = self.clone();
        let dp = ProcessConfig::default();
        let dr = RegulationConfig::default();

        // Procédé.
        self.process.gain = finite_or(self.process.gain, dp.gain);
        self.process.tau = finite_at_least(self.process.tau, 1e-3, dp.tau);
        self.process.dead_time = if self.process.dead_time.is_finite() {
            self.process.dead_time.clamp(0.0, MAX_DEAD_TIME)
        } else {
            dp.dead_time
        };
        self.process.ambient = finite_or(self.process.ambient, dp.ambient);

        // Bornes de consigne : finies puis ordonnées.
        let mut sp_min = finite_or(self.regulation.sp_min, dr.sp_min);
        let mut sp_max = finite_or(self.regulation.sp_max, dr.sp_max);
        if sp_min > sp_max {
            std::mem::swap(&mut sp_min, &mut sp_max);
        }
        self.regulation.sp_min = sp_min;
        self.regulation.sp_max = sp_max;

        // Gains et bornes PID.
        self.regulation.pid_heat = sanitize_pid(self.regulation.pid_heat, dr.pid_heat);
        self.regulation.pid_cool = sanitize_pid(self.regulation.pid_cool, dr.pid_cool);

        // Réglages TOR / PWM.
        self.regulation.hysteresis = finite_at_least(self.regulation.hysteresis, 0.0, dr.hysteresis);
        self.regulation.tor_min_cycle =
            finite_at_least(self.regulation.tor_min_cycle, 0.0, dr.tor_min_cycle);
        self.regulation.pwm_period = finite_at_least(self.regulation.pwm_period, 1e-3, dr.pwm_period);

        if self != before {
            log::warn!("Configuration sanitized: out-of-range or non-finite values were corrected");
        }
        self
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
            Ok(content) => match toml::from_str::<Self>(&content) {
                Ok(cfg) => {
                    log::info!("Configuration loaded from {}", path.display());
                    // La config provient d'une source non fiable (fichier éditable) :
                    // on l'assainit pour éviter tout panic ou valeur aberrante.
                    cfg.sanitized()
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
/// Une liste vide autorise toutes les connexions. Les adresses **IPv4-mapped
/// IPv6** (`::ffff:a.b.c.d`) sont ramenées à leur IPv4 avant comparaison (utile en
/// double pile). Pour une IPv6 « pure », seule l'égalité exacte de la
/// représentation textuelle est gérée.
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
///
/// Une adresse **IPv4-mapped IPv6** (`::ffff:a.b.c.d`, cas d'un client IPv4 reçu
/// sur une socket double pile `bind = "::"`) est d'abord ramenée à son IPv4 pour
/// que les motifs IPv4 (`192.168.1.*`) s'appliquent comme attendu.
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
    /// Adresse du maître TCP actuellement (ou dernièrement) connecté. `None` en
    /// RTU (bus série sans notion de connexion) ou tant qu'aucun maître ne s'est
    /// connecté.
    pub peer: Option<String>,
    /// Instant de la dernière requête Modbus traitée. Sert de témoin d'activité
    /// (« liveness ») du lien, indépendamment du transport : l'IHM allume le
    /// voyant de connexion si une requête a été reçue récemment.
    pub last_request: Option<Instant>,
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
    fn ipv4_mapped_ipv6_matches_ipv4_pattern() {
        // Un client IPv4 arrivant sur une socket double pile (`::ffff:a.b.c.d`)
        // doit être filtré par les motifs IPv4 (S3).
        let f = IpFilter::new(vec!["192.168.1.*".to_string()]);
        assert!(f.allows("::ffff:192.168.1.42".parse().unwrap()));
        assert!(!f.allows("::ffff:192.168.2.42".parse().unwrap()));
    }

    #[test]
    fn pure_ipv6_matches_exact_pattern() {
        let f = IpFilter::new(vec!["::1".to_string()]);
        assert!(f.allows("::1".parse().unwrap()));
        assert!(!f.allows("::2".parse().unwrap()));
    }

    #[cfg(feature = "rtu")]
    #[test]
    fn serial_open_on_missing_port_errors() {
        let cfg = SerialConfig {
            port: "/dev/cesam_inexistant_42".to_string(),
            ..SerialConfig::default()
        };
        assert!(cfg.open().is_err(), "ouvrir un port série inexistant doit échouer");
    }

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
        // Le régulateur se construit sans paniquer (clamp des bornes désormais sûr).
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
    fn sanitized_orders_pid_output_bounds() {
        let mut cfg = AppConfig::default();
        cfg.regulation.pid_heat.out_min = 100.0;
        cfg.regulation.pid_heat.out_max = 0.0;
        cfg.regulation.pid_cool.kp = f32::NAN;
        let cfg = cfg.sanitized();
        assert!(cfg.regulation.pid_heat.out_min <= cfg.regulation.pid_heat.out_max);
        assert!(cfg.regulation.pid_cool.kp.is_finite() && cfg.regulation.pid_cool.kp >= 0.0);
    }

    #[test]
    fn sanitized_is_noop_on_defaults() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.clone(), cfg.sanitized());
    }
}
