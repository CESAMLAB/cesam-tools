//! Modèle métier du régulateur, **synchrone et testable**, réutilisant
//! [`mock_lib_control`] (PID + procédé du premier ordre avec retard).
//!
//! Un PID asservit la **mesure** (PV) d'un procédé thermique vers une **consigne**
//! (SP) en pilotant une **sortie** 0-100 %. Aucune nouveauté métier : l'instrument
//! se distingue par son **transport OPC UA**, pas par sa physique.

use mock_lib_control::{FirstOrderProcess, Pid, PidConfig};

/// Pas de simulation (s). Procédé thermique lent → pas large.
pub const DEFAULT_DT: f32 = 0.5;

/// Configuration statique du régulateur (procédé + asservissement + bornes).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RegulatorConfig {
    /// Période de simulation (s).
    pub dt: f32,
    /// Bornes de consigne (unité procédé).
    pub sp_min: f32,
    pub sp_max: f32,
    /// Réglage du PID (sortie = commande 0-100 %).
    pub pid: PidConfig,
    /// Gain statique du procédé `K` (unité procédé par % de commande).
    pub k: f32,
    /// Constante de temps `tau` (s).
    pub tau: f32,
    /// Retard pur (s).
    pub dead_time: f32,
    /// Valeur ambiante (sortie au repos).
    pub ambient: f32,
}

impl Default for RegulatorConfig {
    fn default() -> Self {
        Self {
            dt: DEFAULT_DT,
            sp_min: 0.0,
            sp_max: 150.0,
            pid: PidConfig {
                kp: 4.0,
                ki: 0.3,
                kd: 0.0,
                out_min: 0.0,
                out_max: 100.0,
            },
            k: 0.8,
            tau: 40.0,
            dead_time: 4.0,
            ambient: 20.0,
        }
    }
}

/// Commande appliquée au régulateur (depuis l'IHM ou un client OPC UA).
///
/// Le préfixe `Set` commun aux variantes est volontaire (pattern « commande »).
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum Command {
    /// Marche / arrêt de la régulation.
    SetRun(bool),
    /// Mode automatique (PID) vs manuel (sortie imposée).
    SetAuto(bool),
    /// Consigne de mesure (unité procédé).
    SetSetpoint(f32),
    /// Sortie manuelle imposée (%), utilisée hors mode automatique.
    SetManualOutput(f32),
    /// Bornes de consigne `(min, max)`.
    SetSpBounds { min: f32, max: f32 },
    /// Réglage du PID.
    SetPid(PidConfig),
    /// Paramètres du procédé (fonction de transfert).
    SetProcess {
        k: f32,
        tau: f32,
        dead_time: f32,
        ambient: f32,
    },
}

/// Instantané de l'état, partagé **en lecture** avec l'IHM et le serveur OPC UA.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Snapshot {
    pub run: bool,
    pub auto: bool,
    pub setpoint: f32,
    pub pv: f32,
    pub output: f32,
    /// Sortie manuelle imposée (%), utilisée hors mode automatique.
    pub manual_output: f32,
    pub sp_min: f32,
    pub sp_max: f32,
    pub pid: PidConfig,
}

/// Régulateur simulé complet (PID + procédé).
pub struct Regulator {
    cfg: RegulatorConfig,
    pid: Pid,
    process: FirstOrderProcess,
    run: bool,
    auto: bool,
    setpoint: f32,
    manual_output: f32,
    output: f32,
}

impl Default for Regulator {
    fn default() -> Self {
        Self::new(RegulatorConfig::default())
    }
}

impl Regulator {
    /// Construit un régulateur à partir de sa configuration.
    #[must_use]
    pub fn new(cfg: RegulatorConfig) -> Self {
        let pid = Pid::new(cfg.pid);
        let process = FirstOrderProcess::new(cfg.k, cfg.tau, cfg.dead_time, cfg.ambient);
        let setpoint = cfg.sp_min.min(cfg.sp_max);
        Self {
            cfg,
            pid,
            process,
            run: false,
            auto: true,
            setpoint,
            manual_output: 0.0,
            output: 0.0,
        }
    }

    /// Période d'échantillonnage (s).
    #[must_use]
    pub fn dt(&self) -> f32 {
        self.cfg.dt
    }

    /// Applique une commande externe (valeurs assainies / bornées : la surface
    /// réseau ne peut produire ni `NaN`/`Inf` ni valeur aberrante).
    pub fn apply(&mut self, cmd: Command) {
        match cmd {
            Command::SetRun(on) => {
                self.run = on;
                if !on {
                    self.pid.reset();
                }
            }
            Command::SetAuto(auto) => {
                self.auto = auto;
                self.pid.reset_integral();
            }
            Command::SetSetpoint(sp) => {
                if sp.is_finite() {
                    self.setpoint = sp.clamp(self.cfg.sp_min, self.cfg.sp_max);
                }
            }
            Command::SetManualOutput(out) => {
                self.manual_output = if out.is_finite() { out.clamp(0.0, 100.0) } else { 0.0 };
            }
            Command::SetSpBounds { min, max } => {
                let (min, max) = if min <= max { (min, max) } else { (max, min) };
                self.cfg.sp_min = if min.is_finite() { min } else { 0.0 };
                self.cfg.sp_max = if max.is_finite() { max } else { self.cfg.sp_min };
                self.setpoint = self.setpoint.clamp(self.cfg.sp_min, self.cfg.sp_max);
            }
            Command::SetPid(pid) => {
                let pid = clamp_pid(pid);
                self.cfg.pid = pid;
                self.pid.set_config(pid);
            }
            Command::SetProcess {
                k,
                tau,
                dead_time,
                ambient,
            } => {
                self.cfg.k = if k.is_finite() { k } else { self.cfg.k };
                self.cfg.tau = if tau.is_finite() { tau.max(1e-3) } else { self.cfg.tau };
                self.cfg.dead_time = if dead_time.is_finite() { dead_time.max(0.0) } else { self.cfg.dead_time };
                self.cfg.ambient = if ambient.is_finite() { ambient } else { self.cfg.ambient };
                // Recrée le procédé en préservant la mesure courante (pas de saut).
                let pv = self.process.value();
                self.process =
                    FirstOrderProcess::new(self.cfg.k, self.cfg.tau, self.cfg.dead_time, self.cfg.ambient);
                self.process.reset(pv);
            }
        }
    }

    /// Avance la simulation d'un pas.
    pub fn step(&mut self) {
        let dt = self.cfg.dt;
        if !self.run {
            // À l'arrêt : plus de commande, le procédé relaxe vers l'ambiant.
            self.pid.reset();
            self.output = 0.0;
            self.process.step(0.0, dt);
            return;
        }
        let command = if self.auto {
            self.pid.step(self.setpoint, self.process.value(), dt)
        } else {
            self.manual_output
        };
        self.output = command.clamp(0.0, 100.0);
        self.process.step(self.output, dt);
    }

    /// Image instantanée de l'état courant.
    #[must_use]
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            run: self.run,
            auto: self.auto,
            setpoint: self.setpoint,
            pv: self.process.value(),
            output: self.output,
            manual_output: self.manual_output,
            sp_min: self.cfg.sp_min,
            sp_max: self.cfg.sp_max,
            pid: self.cfg.pid,
        }
    }

    /// Avance la simulation de `seconds` secondes (utilitaire de test).
    #[cfg(test)]
    fn run_for(&mut self, seconds: f32) {
        let steps = (seconds / self.cfg.dt).round() as usize;
        for _ in 0..steps {
            self.step();
        }
    }
}

/// Borne les gains PID à des valeurs finies ≥ 0 et garde les bornes de sortie
/// ordonnées (robustesse face aux écritures externes).
#[must_use]
fn clamp_pid(mut cfg: PidConfig) -> PidConfig {
    let sane = |v: f32, d: f32| if v.is_finite() { v.max(0.0) } else { d };
    cfg.kp = sane(cfg.kp, 0.0);
    cfg.ki = sane(cfg.ki, 0.0);
    cfg.kd = sane(cfg.kd, 0.0);
    if !cfg.out_min.is_finite() {
        cfg.out_min = 0.0;
    }
    if !cfg.out_max.is_finite() {
        cfg.out_max = 100.0;
    }
    if cfg.out_min > cfg.out_max {
        std::mem::swap(&mut cfg.out_min, &mut cfg.out_max);
    }
    cfg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pv_converges_to_setpoint_in_auto() {
        let mut reg = Regulator::default();
        reg.apply(Command::SetSetpoint(80.0));
        reg.apply(Command::SetRun(true));
        reg.run_for(600.0);
        let snap = reg.snapshot();
        assert!((snap.pv - 80.0).abs() < 3.0, "PV={} doit tendre vers 80", snap.pv);
    }

    #[test]
    fn setpoint_is_clamped_and_nan_ignored() {
        let mut reg = Regulator::default();
        reg.apply(Command::SetSetpoint(9999.0));
        assert!((reg.snapshot().setpoint - 150.0).abs() < 1e-3);
        reg.apply(Command::SetSetpoint(f32::NAN));
        assert!((reg.snapshot().setpoint - 150.0).abs() < 1e-3);
    }

    #[test]
    fn stopped_process_relaxes_toward_ambient() {
        let mut reg = Regulator::default();
        reg.apply(Command::SetSetpoint(100.0));
        reg.apply(Command::SetRun(true));
        reg.run_for(300.0);
        assert!(reg.snapshot().pv > 50.0);
        reg.apply(Command::SetRun(false));
        reg.run_for(600.0);
        let snap = reg.snapshot();
        assert_eq!(snap.output, 0.0);
        assert!(snap.pv < 30.0, "à l'arrêt la PV doit retomber (={})", snap.pv);
    }

    #[test]
    fn process_change_preserves_pv_and_orders_bounds() {
        let mut reg = Regulator::default();
        reg.apply(Command::SetSetpoint(70.0));
        reg.apply(Command::SetRun(true));
        reg.run_for(200.0);
        let pv_before = reg.snapshot().pv;
        reg.apply(Command::SetProcess { k: 1.2, tau: 20.0, dead_time: 2.0, ambient: 20.0 });
        assert!((reg.snapshot().pv - pv_before).abs() < 1e-3, "pas de saut de PV");
        // Bornes inversées réordonnées sans panic.
        reg.apply(Command::SetSpBounds { min: 100.0, max: 0.0 });
        let s = reg.snapshot();
        assert!(s.sp_min <= s.sp_max);
    }
}
