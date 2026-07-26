//! Configuration de l'application : connexion **broker MQTT / edge node Sparkplug B**,
//! procédé et régulation, avec persistance TOML. Toute valeur issue du fichier est
//! **assainie** au chargement ([`AppConfig::sanitized`]) pour éviter tout `panic!`
//! (`f32::clamp`) ou valeur aberrante.

use std::path::{Path, PathBuf};

use mock_lib_control::PidConfig;
use serde::{Deserialize, Serialize};

use crate::i18n::Lang;
use crate::regulator::{RegulatorConfig, DEFAULT_DT};

const DEFAULT_CONFIG_FILE: &str = "mock_ru_sparkplugb.toml";

/// Paramètres de connexion de l'edge node Sparkplug B vers un **broker MQTT externe**.
///
/// Contrairement à ORME/ORUE, le simulateur n'écoute aucun port : il se **connecte
/// en sortie** au broker et publie sous `spBv1.0/<group_id>/.../<edge_node_id>`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct NetworkConfig {
    /// Hôte du broker MQTT.
    pub broker_host: String,
    /// Port du broker MQTT (1883 en clair, 8883 en TLS).
    pub broker_port: u16,
    /// Identifiant de client MQTT.
    pub client_id: String,
    /// Groupe Sparkplug (`spBv1.0/<group_id>/...`).
    pub group_id: String,
    /// Identifiant du nœud edge (`.../<edge_node_id>`).
    pub edge_node_id: String,
    /// Utilisateur MQTT (**vide** = pas d'authentification).
    pub username: String,
    /// Mot de passe MQTT en clair — **simulateur uniquement** (réseau de confiance).
    pub password: String,
    /// Active TLS (rustls) vers le broker.
    pub tls: bool,
    /// Keepalive MQTT (s).
    pub keepalive_secs: u16,
    /// `true` : publier un `NDATA` à chaque changement ; `false` : périodiquement.
    pub publish_on_change: bool,
    /// Cadence de publication périodique (s) quand `publish_on_change = false`.
    pub publish_period_secs: u16,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            broker_host: "localhost".to_string(),
            broker_port: 1883,
            client_id: "ru_spb".to_string(),
            group_id: "CESAM".to_string(),
            edge_node_id: "RU1".to_string(),
            username: String::new(),
            password: String::new(),
            tls: false,
            keepalive_secs: 30,
            publish_on_change: true,
            publish_period_secs: 5,
        }
    }
}

impl NetworkConfig {
    /// `true` si une authentification utilisateur/mot de passe est configurée.
    #[must_use]
    pub fn has_user(&self) -> bool {
        !self.username.trim().is_empty()
    }

    /// Libellé lisible pour l'IHM : `mqtt[s]://host:port  spBv1.0/<group>/.../<node>`.
    #[must_use]
    pub fn endpoint_label(&self) -> String {
        let scheme = if self.tls { "mqtts" } else { "mqtt" };
        format!(
            "{scheme}://{}:{}  spBv1.0/{}/…/{}",
            self.broker_host, self.broker_port, self.group_id, self.edge_node_id
        )
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

/// Configuration complète de l'application.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub language: Lang,
    pub network: NetworkConfig,
    pub process: ProcessConfig,
    pub regulation: RegulationConfig,
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

    /// Assainit les valeurs issues du TOML (anti-panic / anti-aberration).
    #[must_use]
    pub fn sanitized(mut self) -> Self {
        let before = self.clone();
        let dp = ProcessConfig::default();
        let dr = RegulationConfig::default();
        let dn = NetworkConfig::default();

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

        // Réseau : identifiants Sparkplug obligatoires (sinon topics invalides) et
        // temporisations bornées.
        if self.network.broker_host.trim().is_empty() {
            self.network.broker_host = dn.broker_host;
        }
        if self.network.client_id.trim().is_empty() {
            self.network.client_id = dn.client_id;
        }
        if self.network.group_id.trim().is_empty() {
            self.network.group_id = dn.group_id;
        }
        if self.network.edge_node_id.trim().is_empty() {
            self.network.edge_node_id = dn.edge_node_id;
        }
        self.network.keepalive_secs = self.network.keepalive_secs.max(5);
        self.network.publish_period_secs = self.network.publish_period_secs.max(1);

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

/// État de **connexion** de l'edge node Sparkplug B, partagé avec l'IHM.
///
/// (Le simulateur étant un client, `connected` reflète la session MQTT, pas une
/// socket d'écoute.)
#[derive(Debug, Clone, Default)]
pub struct ServerStatus {
    /// Session MQTT établie (et `NBIRTH` publié).
    pub connected: bool,
    /// Libellé broker + namespace Sparkplug.
    pub addr: String,
    /// Dernière erreur (déconnexion, échec de publication…).
    pub error: Option<String>,
    /// `bdSeq` de la session courante (affichage diagnostic).
    pub bd_seq: u64,
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
    fn network_with_user_round_trips_through_toml() {
        let cfg = AppConfig {
            network: NetworkConfig {
                broker_host: "broker.local".to_string(),
                broker_port: 8883,
                username: "scada".to_string(),
                password: "secret".to_string(),
                tls: true,
                ..NetworkConfig::default()
            },
            ..AppConfig::default()
        };
        let s = toml::to_string_pretty(&cfg).unwrap();
        let back: AppConfig = toml::from_str(&s).unwrap();
        assert_eq!(cfg, back);
        assert!(back.network.has_user());
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
    fn sanitize_defaults_empty_ids_and_clamps_timings() {
        let mut cfg = AppConfig::default();
        cfg.network.broker_host = "  ".to_string();
        cfg.network.client_id = String::new();
        cfg.network.group_id = String::new();
        cfg.network.edge_node_id = String::new();
        cfg.network.keepalive_secs = 0;
        cfg.network.publish_period_secs = 0;
        let cfg = cfg.sanitized();
        assert!(!cfg.network.broker_host.trim().is_empty());
        assert!(!cfg.network.client_id.is_empty());
        assert!(!cfg.network.group_id.is_empty());
        assert!(!cfg.network.edge_node_id.is_empty());
        assert!(cfg.network.keepalive_secs >= 5);
        assert!(cfg.network.publish_period_secs >= 1);
    }

    #[test]
    fn endpoint_label_format() {
        let net = NetworkConfig {
            broker_host: "127.0.0.1".to_string(),
            broker_port: 1883,
            group_id: "CESAM".to_string(),
            edge_node_id: "RU1".to_string(),
            ..NetworkConfig::default()
        };
        assert_eq!(net.endpoint_label(), "mqtt://127.0.0.1:1883  spBv1.0/CESAM/…/RU1");
    }
}
