//! Modèle métier minimal du régulateur, **synchrone et testable**, réutilisant
//! [`mock_lib_control`] (PID + procédé du premier ordre avec retard).
//!
//! Identique d'esprit au régulateur ORME : un PID asservit la **mesure** (PV) d'un
//! procédé thermique vers une **consigne** (SP) en pilotant une **sortie** 0-100 %.
//! Aucune nouveauté métier : la Phase 1 concentre l'effort sur le protocole OPC UA.

use mock_lib_control::{FirstOrderProcess, Pid, PidConfig};

/// Pas de simulation (s). Procédé thermique lent → pas large (cf. ORME).
pub const DEFAULT_DT: f32 = 0.5;

/// Commande appliquée au régulateur (provenant d'un client OPC UA).
///
/// Le préfixe `Set` commun aux variantes est volontaire (pattern « commande »).
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum Command {
    /// Marche / arrêt de la régulation.
    SetRun(bool),
    /// Mode automatique (PID) vs manuel (sortie imposée).
    SetAuto(bool),
    /// Consigne de mesure (unité procédé, p. ex. °C).
    SetSetpoint(f32),
    /// Sortie manuelle imposée (%), utilisée hors mode automatique.
    SetManualOutput(f32),
}

/// Instantané de l'état, partagé **en lecture** avec le serveur OPC UA.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Snapshot {
    pub run: bool,
    pub auto: bool,
    pub setpoint: f32,
    pub pv: f32,
    pub output: f32,
    /// Sortie manuelle imposée (%), utilisée hors mode automatique.
    pub manual_output: f32,
}

/// Régulateur simulé complet (PID + procédé).
pub struct Regulator {
    pid: Pid,
    process: FirstOrderProcess,
    run: bool,
    auto: bool,
    setpoint: f32,
    manual_output: f32,
    output: f32,
    sp_min: f32,
    sp_max: f32,
}

impl Default for Regulator {
    fn default() -> Self {
        Self::new()
    }
}

impl Regulator {
    /// Construit un régulateur avec des réglages par défaut raisonnables.
    #[must_use]
    pub fn new() -> Self {
        let pid = Pid::new(PidConfig {
            kp: 4.0,
            ki: 0.3,
            kd: 0.0,
            out_min: 0.0,
            out_max: 100.0,
        });
        // K = 0,8 °C/%, tau = 40 s, retard = 4 s, ambiant = 20 °C.
        let process = FirstOrderProcess::new(0.8, 40.0, 4.0, 20.0);
        Self {
            pid,
            process,
            run: false,
            auto: true,
            setpoint: 60.0,
            manual_output: 0.0,
            output: 0.0,
            sp_min: 0.0,
            sp_max: 150.0,
        }
    }

    /// Période d'échantillonnage (s).
    #[must_use]
    pub fn dt(&self) -> f32 {
        DEFAULT_DT
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
                    self.setpoint = sp.clamp(self.sp_min, self.sp_max);
                }
            }
            Command::SetManualOutput(out) => {
                self.manual_output = if out.is_finite() { out.clamp(0.0, 100.0) } else { 0.0 };
            }
        }
    }

    /// Avance la simulation d'un pas.
    pub fn step(&mut self) {
        let dt = DEFAULT_DT;
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
        }
    }

    /// Avance la simulation de `seconds` secondes (utilitaire de test).
    #[cfg(test)]
    fn run_for(&mut self, seconds: f32) {
        let steps = (seconds / DEFAULT_DT).round() as usize;
        for _ in 0..steps {
            self.step();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pv_converges_to_setpoint_in_auto() {
        let mut reg = Regulator::new();
        reg.apply(Command::SetSetpoint(80.0));
        reg.apply(Command::SetRun(true));
        reg.run_for(600.0); // procédé lent : laisser le temps de converger
        let snap = reg.snapshot();
        assert!(
            (snap.pv - 80.0).abs() < 3.0,
            "PV={} doit tendre vers la consigne 80",
            snap.pv
        );
    }

    #[test]
    fn setpoint_is_clamped_and_nan_ignored() {
        let mut reg = Regulator::new();
        reg.apply(Command::SetSetpoint(9999.0));
        assert!((reg.snapshot().setpoint - 150.0).abs() < 1e-3);
        reg.apply(Command::SetSetpoint(f32::NAN)); // ignoré, pas de panic
        assert!((reg.snapshot().setpoint - 150.0).abs() < 1e-3);
    }

    #[test]
    fn stopped_process_relaxes_toward_ambient() {
        let mut reg = Regulator::new();
        reg.apply(Command::SetSetpoint(100.0));
        reg.apply(Command::SetRun(true));
        reg.run_for(300.0);
        assert!(reg.snapshot().pv > 50.0);
        reg.apply(Command::SetRun(false));
        reg.run_for(600.0);
        let snap = reg.snapshot();
        assert_eq!(snap.output, 0.0);
        assert!(snap.pv < 30.0, "à l'arrêt la PV doit retomber vers l'ambiant (={})", snap.pv);
    }
}
