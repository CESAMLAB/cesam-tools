//! Configuration de l'application : réseau OPC UA, procédé et régulation, avec
//! persistance TOML. Toute valeur issue du fichier est **assainie** au chargement
//! ([`AppConfig::sanitized`]) pour éviter tout `panic!` (`f32::clamp`) ou valeur
//! aberrante.

use std::path::{Path, PathBuf};

use mock_lib_control::PidConfig;
use serde::{Deserialize, Serialize};

use crate::i18n::Lang;
use crate::regulator::{RegulatorConfig, DEFAULT_DT};

const DEFAULT_CONFIG_FILE: &str = "mock_ru_opcua.toml";

/// Paramètres réseau du serveur OPC UA.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct NetworkConfig {
    /// IP d'écoute (utilisée pour l'URL d'endpoint et le bind).
    pub bind_ip: String,
    /// Port TCP OPC UA (par défaut 4840).
    pub port: u16,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            bind_ip: "0.0.0.0".to_string(),
            port: 4840,
        }
    }
}

impl NetworkConfig {
    /// URL d'endpoint OPC UA (`opc.tcp://<ip>:<port>/`).
    #[must_use]
    pub fn endpoint_url(&self) -> String {
        format!("opc.tcp://{}:{}/", self.bind_ip, self.port)
    }
}

/// Paramètres du procédé simulé (fonction de transfert du premier ordre + retard).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProcessConfig {
    /// Gain statique `K`.
    pub k: f32,
    /// Constante de temps `tau` (s).
    pub tau: f32,
    /// Retard pur (s).
    pub dead_time: f32,
    /// Valeur ambiante (sortie au repos).
    pub ambient: f32,
}

impl Default for ProcessConfig {
    fn default() -> Self {
        let r = RegulatorConfig::default();
        Self {
            k: r.k,
            tau: r.tau,
            dead_time: r.dead_time,
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
    pub pid: PidConfig,
}

impl Default for RegulationConfig {
    fn default() -> Self {
        let r = RegulatorConfig::default();
        Self {
            sp_min: r.sp_min,
            sp_max: r.sp_max,
            pid: r.pid,
        }
    }
}

/// Paramètres de **sécurité OPC UA** (Phase 2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SecurityConfig {
    /// Active un endpoint **chiffré** `Basic256Sha256` (SignAndEncrypt). Génère un
    /// certificat d'instance auto-signé au premier lancement (dans `pki/`).
    /// Désactivé : un seul endpoint `None` anonyme (réseau de confiance).
    pub encryption: bool,
    /// Autorise le jeton **anonyme** (en plus d'un éventuel utilisateur/mot de passe).
    pub allow_anonymous: bool,
    /// Identifiant utilisateur (**vide** = pas d'authentification par mot de passe).
    pub username: String,
    /// Mot de passe en clair — **simulateur uniquement** (sur réseau de confiance).
    pub password: String,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        // Défaut = Phase 1b : pas de chiffrement, endpoint None anonyme (démarrage
        // instantané, aucun certificat généré).
        Self {
            encryption: false,
            allow_anonymous: true,
            username: String::new(),
            password: String::new(),
        }
    }
}

impl SecurityConfig {
    /// `true` si une authentification par utilisateur/mot de passe est configurée.
    #[must_use]
    pub fn has_user(&self) -> bool {
        !self.username.trim().is_empty()
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
    pub security: SecurityConfig,
    /// Vérifier au démarrage si une version plus récente est publiée (feature
    /// `gui`). Activé par défaut ; désactivable depuis le modal *Paramètres*.
    pub check_updates: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            language: Lang::default(),
            network: NetworkConfig::default(),
            process: ProcessConfig::default(),
            regulation: RegulationConfig::default(),
            security: SecurityConfig::default(),
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
        let dp = ProcessConfig::default();
        let dr = RegulationConfig::default();

        // Procédé.
        self.process.k = finite_or(self.process.k, dp.k);
        self.process.tau = finite_at_least(self.process.tau, 1e-3, dp.tau);
        self.process.dead_time = finite_at_least(self.process.dead_time, 0.0, dp.dead_time);
        self.process.ambient = finite_or(self.process.ambient, dp.ambient);

        // Bornes de consigne (finies puis ordonnées).
        let mut s_min = finite_or(self.regulation.sp_min, dr.sp_min);
        let mut s_max = finite_or(self.regulation.sp_max, dr.sp_max);
        if s_min > s_max {
            std::mem::swap(&mut s_min, &mut s_max);
        }
        self.regulation.sp_min = s_min;
        self.regulation.sp_max = s_max;

        // Gains et bornes PID.
        self.regulation.pid = sanitize_pid(self.regulation.pid, dr.pid);

        // Sécurité : garde-fou anti-verrouillage — en chiffré sans utilisateur ni
        // anonyme, plus aucun jeton ne permettrait de se connecter → on réautorise
        // l'anonyme (sur transport chiffré).
        if self.security.encryption && !self.security.allow_anonymous && !self.security.has_user() {
            self.security.allow_anonymous = true;
        }

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

/// État courant du serveur OPC UA, partagé avec l'IHM pour affichage.
#[derive(Debug, Clone, Default)]
pub struct ServerStatus {
    pub listening: bool,
    pub addr: String,
    pub error: Option<String>,
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
        // Ne panique pas en construisant le régulateur.
        let _ = crate::regulator::Regulator::new(cfg.to_regulator_config());
    }

    #[test]
    fn endpoint_url_format() {
        let net = NetworkConfig { bind_ip: "127.0.0.1".to_string(), port: 4840 };
        assert_eq!(net.endpoint_url(), "opc.tcp://127.0.0.1:4840/");
    }

    #[test]
    fn security_default_is_phase1b() {
        let s = SecurityConfig::default();
        assert!(!s.encryption && s.allow_anonymous && !s.has_user());
    }

    #[test]
    fn sanitize_reenables_anonymous_when_no_token_left() {
        let cfg = AppConfig {
            security: SecurityConfig {
                encryption: true,
                allow_anonymous: false, // et aucun utilisateur
                ..SecurityConfig::default()
            },
            ..AppConfig::default()
        }
        .sanitized();
        assert!(cfg.security.allow_anonymous, "garde-fou : au moins un jeton");
    }

    #[test]
    fn security_with_user_round_trips_through_toml() {
        let cfg = AppConfig {
            security: SecurityConfig {
                encryption: true,
                allow_anonymous: false,
                username: "scada".to_string(),
                password: "secret".to_string(),
            },
            ..AppConfig::default()
        };
        let s = toml::to_string_pretty(&cfg).unwrap();
        let back: AppConfig = toml::from_str(&s).unwrap();
        assert_eq!(cfg, back);
        assert!(back.security.has_user());
    }
}
