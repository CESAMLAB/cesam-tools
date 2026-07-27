//! Fragments de configuration persistée (procédé + régulation), communs aux
//! instruments RU « simple PID ». Le réseau (`NetworkConfig`) reste propre à
//! chaque instrument et n'est pas couvert ici.

use mock_lib_control::PidConfig;
use serde::{Deserialize, Serialize};

use crate::regulator::RegulatorConfig;

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

impl ProcessConfig {
    /// Assainit les valeurs issues du TOML (finies, `tau`/`dead_time` bornés).
    #[must_use]
    pub fn sanitized(mut self) -> Self {
        let d = Self::default();
        self.k = finite_or(self.k, d.k);
        self.tau = finite_at_least(self.tau, 1e-3, d.tau);
        self.dead_time = finite_at_least(self.dead_time, 0.0, d.dead_time);
        self.ambient = finite_or(self.ambient, d.ambient);
        self
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

impl RegulationConfig {
    /// Assainit les valeurs issues du TOML (bornes de consigne ordonnées, gains
    /// PID clampés).
    #[must_use]
    pub fn sanitized(mut self) -> Self {
        let d = Self::default();
        let mut s_min = finite_or(self.sp_min, d.sp_min);
        let mut s_max = finite_or(self.sp_max, d.sp_max);
        if s_min > s_max {
            std::mem::swap(&mut s_min, &mut s_max);
        }
        self.sp_min = s_min;
        self.sp_max = s_max;
        self.pid = sanitize_pid(self.pid, d.pid);
        self
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
