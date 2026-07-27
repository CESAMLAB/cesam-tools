//! Configuration de l'application : réseau **serveur S7** (ISO-on-TCP), procédé et
//! régulation, avec persistance TOML. Toute valeur issue du fichier est **assainie**
//! au chargement ([`AppConfig::sanitized`]) pour éviter tout `panic!` (`f32::clamp`)
//! ou valeur aberrante.

use std::net::IpAddr;
use std::path::{Path, PathBuf};

use mock_lib_regulator::{ProcessConfig, RegulationConfig, RegulatorConfig, DEFAULT_DT};
use serde::{Deserialize, Serialize};

use crate::i18n::Lang;

const DEFAULT_CONFIG_FILE: &str = "mock_ru_s7.toml";

/// Port S7 standard (ISO-on-TCP / RFC1006). < 1024 → nécessite les droits root.
pub const DEFAULT_PORT: u16 = 102;

/// Paramètres réseau du serveur S7 (ISO-on-TCP).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct NetworkConfig {
    /// IP d'écoute.
    pub bind_ip: String,
    /// Port TCP (102 standard S7).
    pub port: u16,
    /// Liste blanche d'IP (motifs avec jokers `*` par octet). Vide = tout autorisé.
    pub allowlist: Vec<String>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            bind_ip: "0.0.0.0".to_string(),
            port: DEFAULT_PORT,
            allowlist: Vec::new(),
        }
    }
}

impl NetworkConfig {
    /// Adresse d'écoute `ip:port` pour le bind et l'affichage.
    #[must_use]
    pub fn listen_addr(&self) -> String {
        format!("{}:{}", self.bind_ip, self.port)
    }

    /// `true` si la posture est **exposée** : écoute sur toutes les interfaces
    /// (`0.0.0.0`/`::`) **et** liste blanche vide.
    #[must_use]
    pub fn is_exposed(&self) -> bool {
        let all_ifaces = self.bind_ip.trim() == "0.0.0.0" || self.bind_ip.trim() == "::";
        all_ifaces && Allowlist::new(self.allowlist.clone()).is_empty()
    }
}

/// Filtre d'adresses IP basé sur des motifs avec jokers `*` par octet (IPv4).
///
/// Une liste vide autorise toutes les connexions. Les adresses **IPv4-mapped IPv6**
/// (`::ffff:a.b.c.d`) sont ramenées à leur IPv4 avant comparaison. Pour une IPv6
/// « pure », seule l'égalité exacte de la représentation textuelle est gérée.
#[derive(Debug, Clone, Default)]
pub struct Allowlist {
    patterns: Vec<String>,
}

impl Allowlist {
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

    /// `true` si aucune restriction (liste vide).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
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
    #[must_use]
    pub fn to_regulator_config(&self) -> RegulatorConfig {
        let mut pid = self.regulation.pid;
        pid.out_min = 0.0;
        pid.out_max = 100.0;
        RegulatorConfig {
            dt: DEFAULT_DT,
            sp_min: self.regulation.sp_min,
            sp_max: self.regulation.sp_max,
            pid,
            k: self.process.k,
            tau: self.process.tau,
            dead_time: self.process.dead_time,
            ambient: self.process.ambient,
        }
    }

    /// Assainit les valeurs numériques issues du TOML (anti-panic / anti-aberration).
    #[must_use]
    pub fn sanitized(mut self) -> Self {
        let before = self.clone();

        self.process = self.process.sanitized();
        self.regulation = self.regulation.sanitized();

        if self != before {
            log::warn!("Configuration sanitized: out-of-range or non-finite values were corrected");
        }
        self
    }

    #[must_use]
    pub fn path() -> PathBuf {
        std::env::var_os("MOCK_CONFIG")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_FILE))
    }

    #[must_use]
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

/// État courant du serveur S7, partagé avec l'IHM pour affichage.
#[derive(Debug, Clone, Default)]
pub struct ServerStatus {
    /// `true` si le serveur écoute effectivement.
    pub listening: bool,
    /// Adresse d'écoute courante.
    pub addr: String,
    /// Dernière erreur réseau, le cas échéant.
    pub error: Option<String>,
    /// Adresse du dernier client S7 connecté (affichage).
    pub peer: Option<String>,
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
    fn sanitized_orders_inverted_bounds_without_panic() {
        let mut cfg = AppConfig::default();
        cfg.regulation.sp_min = 200.0;
        cfg.regulation.sp_max = 0.0;
        cfg.process.tau = f32::NAN;
        cfg.process.dead_time = -5.0;
        let cfg = cfg.sanitized();
        assert!(cfg.regulation.sp_min <= cfg.regulation.sp_max);
        assert!(cfg.process.tau.is_finite() && cfg.process.tau >= 1e-3);
        assert!(cfg.process.dead_time >= 0.0);
        let _ = mock_lib_regulator::Regulator::new(cfg.to_regulator_config());
    }

    #[test]
    fn listen_addr_format() {
        let net = NetworkConfig { bind_ip: "127.0.0.1".to_string(), port: 102, allowlist: vec![] };
        assert_eq!(net.listen_addr(), "127.0.0.1:102");
    }

    #[test]
    fn empty_allowlist_allows_all() {
        let f = Allowlist::new(vec![]);
        assert!(f.allows("8.8.8.8".parse().unwrap()));
    }

    #[test]
    fn wildcard_matches_subnet() {
        let f = Allowlist::new(vec!["192.168.1.*".to_string()]);
        assert!(f.allows("192.168.1.42".parse().unwrap()));
        assert!(!f.allows("192.168.2.42".parse().unwrap()));
    }

    #[test]
    fn ipv4_mapped_v6_is_reduced() {
        let f = Allowlist::new(vec!["192.168.1.*".to_string()]);
        assert!(f.allows("::ffff:192.168.1.42".parse().unwrap()));
        assert!(!f.allows("::ffff:192.168.2.42".parse().unwrap()));
    }

    #[test]
    fn exposed_when_all_ifaces_and_empty_allowlist() {
        assert!(NetworkConfig::default().is_exposed());
        let restricted = NetworkConfig { allowlist: vec!["10.0.0.*".to_string()], ..NetworkConfig::default() };
        assert!(!restricted.is_exposed());
    }
}
