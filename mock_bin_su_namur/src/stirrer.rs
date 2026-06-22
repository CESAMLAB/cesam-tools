//! Modèle métier de l'agitateur : état, configuration, commandes, pas de simulation.
//!
//! Module **synchrone et sans IO**, piloté par l'acteur de simulation qui appelle
//! [`Stirrer::step`] à intervalle régulier et applique les [`Command`] provenant de
//! l'IHM ou du serveur NAMUR.
//!
//! Boucle : un **PID rapide** ([`mock_lib_control::Pid`]) asservit la **vitesse**
//! (tr/min) en pilotant le **couple moteur** ([`crate::motor::Motor`]) ; la
//! **viscosité** réglable agit comme charge (couple ∝ viscosité·vitesse).

use mock_lib_control::{Pid, PidConfig};

use crate::motor::Motor;

/// Pas d'échantillonnage par défaut (s). 20 ms (50 Hz) : la régulation de vitesse
/// d'un moteur est bien plus rapide qu'un procédé thermique, d'où un pas plus fin
/// que celui du régulateur ORME.
pub const DEFAULT_DT: f32 = 0.02;

/// Configuration statique de l'agitateur (moteur + asservissement + bornes).
#[derive(Debug, Clone)]
pub struct StirrerConfig {
    /// Période de simulation (s).
    pub dt: f32,
    /// Bornes de consigne de vitesse (tr/min).
    pub speed_min: f32,
    pub speed_max: f32,
    /// Couple moteur maximal (N·cm) : borne la sortie du PID (surcharge au-delà).
    pub torque_max: f32,
    /// Réglage du PID de vitesse (sortie = couple moteur, bornée à `[0, torque_max]`).
    pub pid: PidConfig,
    /// Inertie / réactivité du moteur (petit = rapide).
    pub inertia: f32,
    /// Coefficient de charge visqueuse.
    pub load_coeff: f32,
    /// Frottement sec résiduel (N·cm).
    pub friction: f32,
    /// Viscosité relative initiale (1.0 = référence « eau »).
    pub viscosity: f32,
    /// Bornes de la viscosité relative réglable.
    pub viscosity_min: f32,
    pub viscosity_max: f32,
}

impl Default for StirrerConfig {
    fn default() -> Self {
        // PID volontairement « raide » (Kp élevé, Ki franc) : la sortie sature au
        // couple max tant que l'erreur est grande, d'où une montée en vitesse rapide.
        let pid = PidConfig {
            kp: 0.2,
            ki: 3.0,
            kd: 0.0,
            out_min: 0.0,
            out_max: 100.0,
        };
        Self {
            dt: DEFAULT_DT,
            speed_min: 0.0,
            speed_max: 2000.0,
            torque_max: 100.0,
            pid,
            inertia: 0.02,
            load_coeff: 0.05,
            friction: 2.0,
            viscosity: 1.0,
            viscosity_min: 0.1,
            viscosity_max: 20.0,
        }
    }
}

/// Commande appliquée à l'agitateur (depuis l'IHM ou le serveur NAMUR).
///
/// Le préfixe `Set` commun aux variantes est volontaire (pattern « commande »).
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum Command {
    /// Marche / arrêt du moteur.
    SetOnOff(bool),
    /// Consigne de vitesse (tr/min).
    SetSpeed(f32),
    /// Viscosité relative du milieu (charge).
    SetViscosity(f32),
    /// Réglage du PID de vitesse.
    SetPid(PidConfig),
    /// Bornes de la consigne de vitesse `(min, max)`.
    SetSpeedLimits { min: f32, max: f32 },
    /// Bornes de la viscosité réglable `(min, max)`.
    SetViscosityLimits { min: f32, max: f32 },
    /// Paramètres moteur `(inertia, load_coeff, friction, torque_max)`.
    SetMotor {
        inertia: f32,
        load_coeff: f32,
        friction: f32,
        torque_max: f32,
    },
}

/// Image instantanée de l'état de l'agitateur, partagée avec l'IHM et le serveur.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StirrerSnapshot {
    pub on: bool,
    /// Consigne de vitesse (tr/min).
    pub speed_sp: f32,
    /// Vitesse mesurée (tr/min).
    pub speed: f32,
    /// Couple moteur appliqué / affiché (N·cm).
    pub torque: f32,
    /// Couple moteur maximal (N·cm).
    pub torque_max: f32,
    /// Viscosité relative courante.
    pub viscosity: f32,
    /// `true` si le moteur sature sans atteindre la consigne (surcharge).
    pub overload: bool,
    pub pid: PidConfig,
    pub speed_min: f32,
    pub speed_max: f32,
    pub viscosity_min: f32,
    pub viscosity_max: f32,
    pub inertia: f32,
    pub load_coeff: f32,
    pub friction: f32,
}

/// Agitateur simulé complet : asservissement de vitesse + moteur chargé.
pub struct Stirrer {
    cfg: StirrerConfig,
    on: bool,
    speed_sp: f32,
    viscosity: f32,
    torque: f32,
    pid: Pid,
    motor: Motor,
}

impl Stirrer {
    /// Construit un agitateur à partir de sa configuration.
    #[must_use]
    pub fn new(cfg: StirrerConfig) -> Self {
        let motor = Motor::new(cfg.inertia, cfg.load_coeff, cfg.friction);
        // Consigne initiale = borne basse (le moteur démarre à l'arrêt). `min` reste
        // dans `[speed_min, speed_max]` sans le no-op trompeur d'un `clamp` sur soi-même.
        let speed_sp = cfg.speed_min.min(cfg.speed_max);
        let viscosity = cfg.viscosity.clamp(cfg.viscosity_min, cfg.viscosity_max);
        Self {
            pid: Pid::new(cfg.pid),
            motor,
            on: false,
            speed_sp,
            viscosity,
            torque: 0.0,
            cfg,
        }
    }

    /// Période d'échantillonnage configurée (s).
    #[must_use]
    pub fn dt(&self) -> f32 {
        self.cfg.dt
    }

    /// Applique une commande externe (valeurs assainies/bornées).
    pub fn apply(&mut self, cmd: Command) {
        match cmd {
            Command::SetOnOff(on) => {
                self.on = on;
                if !on {
                    self.pid.reset();
                }
            }
            Command::SetSpeed(sp) => {
                self.speed_sp = sp.clamp(self.cfg.speed_min, self.cfg.speed_max);
            }
            Command::SetViscosity(v) => {
                self.viscosity = v.clamp(self.cfg.viscosity_min, self.cfg.viscosity_max);
            }
            Command::SetPid(pid) => {
                let pid = clamp_pid(pid);
                self.cfg.pid = pid;
                self.pid.set_config(pid);
            }
            Command::SetSpeedLimits { min, max } => {
                let (min, max) = if min <= max { (min, max) } else { (max, min) };
                self.cfg.speed_min = min;
                self.cfg.speed_max = max;
                self.speed_sp = self.speed_sp.clamp(min, max);
            }
            Command::SetViscosityLimits { min, max } => {
                let (min, max) = if min <= max { (min, max) } else { (max, min) };
                self.cfg.viscosity_min = min.max(1e-3);
                self.cfg.viscosity_max = max.max(self.cfg.viscosity_min);
                self.viscosity = self
                    .viscosity
                    .clamp(self.cfg.viscosity_min, self.cfg.viscosity_max);
            }
            Command::SetMotor {
                inertia,
                load_coeff,
                friction,
                torque_max,
            } => {
                self.cfg.inertia = inertia.max(1e-4);
                self.cfg.load_coeff = load_coeff.max(0.0);
                self.cfg.friction = friction.max(0.0);
                self.cfg.torque_max = torque_max.max(1e-3);
                self.cfg.pid.out_max = self.cfg.torque_max;
                self.pid.set_config(self.cfg.pid);
                self.motor.reconfigure(inertia, load_coeff, friction);
            }
        }
    }

    /// Avance la simulation d'un pas.
    pub fn step(&mut self) {
        let dt = self.cfg.dt;
        if !self.on {
            // Moteur coupé : plus de couple moteur, le milieu freine la rotation.
            self.pid.reset();
            self.torque = 0.0;
            self.motor.step(0.0, self.viscosity, dt);
            return;
        }
        // PID rapide : erreur de vitesse -> couple moteur (borné à [0, torque_max]).
        let error = self.speed_sp - self.motor.speed();
        let drive = self
            .pid
            .step_with_error(error, dt)
            .clamp(0.0, self.cfg.torque_max);
        self.torque = drive;
        self.motor.step(drive, self.viscosity, dt);
    }

    /// Indique si le moteur sature sans tenir la consigne (surcharge).
    #[must_use]
    fn overload(&self) -> bool {
        self.on
            && self.torque >= self.cfg.torque_max * 0.98
            && self.motor.speed() < self.speed_sp * 0.95
    }

    /// Image instantanée de l'état courant.
    #[must_use]
    pub fn snapshot(&self) -> StirrerSnapshot {
        StirrerSnapshot {
            on: self.on,
            speed_sp: self.speed_sp,
            speed: self.motor.speed(),
            torque: self.torque,
            torque_max: self.cfg.torque_max,
            viscosity: self.viscosity,
            overload: self.overload(),
            pid: self.cfg.pid,
            speed_min: self.cfg.speed_min,
            speed_max: self.cfg.speed_max,
            viscosity_min: self.cfg.viscosity_min,
            viscosity_max: self.cfg.viscosity_max,
            inertia: self.cfg.inertia,
            load_coeff: self.cfg.load_coeff,
            friction: self.cfg.friction,
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
    fn speed_converges_to_setpoint_fast() {
        let mut s = Stirrer::new(StirrerConfig::default());
        s.apply(Command::SetSpeed(800.0));
        s.apply(Command::SetOnOff(true));
        // Régulation rapide : convergence en quelques secondes.
        s.run_for(5.0);
        let snap = s.snapshot();
        assert!((snap.speed - 800.0).abs() < 20.0, "vitesse={} doit tenir 800", snap.speed);
        assert!(!snap.overload);
    }

    #[test]
    fn torque_rises_with_viscosity_at_same_speed() {
        let torque_at = |visc: f32| {
            let mut s = Stirrer::new(StirrerConfig::default());
            s.apply(Command::SetSpeed(600.0));
            s.apply(Command::SetViscosity(visc));
            s.apply(Command::SetOnOff(true));
            s.run_for(8.0);
            s.snapshot().torque
        };
        let low = torque_at(1.0);
        let high = torque_at(3.0);
        assert!(high > low, "couple à visc 3 ({high}) doit dépasser visc 1 ({low})");
    }

    #[test]
    fn high_viscosity_triggers_overload() {
        let mut s = Stirrer::new(StirrerConfig::default());
        s.apply(Command::SetSpeed(2000.0));
        s.apply(Command::SetViscosity(20.0)); // charge maximale
        s.apply(Command::SetOnOff(true));
        s.run_for(10.0);
        let snap = s.snapshot();
        // Le couple sature sans atteindre 2000 tr/min -> surcharge signalée.
        assert!(snap.overload, "forte viscosité + consigne max doit déclencher la surcharge");
        assert!(snap.speed < 2000.0);
    }

    #[test]
    fn stop_decelerates_motor() {
        let mut s = Stirrer::new(StirrerConfig::default());
        s.apply(Command::SetSpeed(1000.0));
        s.apply(Command::SetOnOff(true));
        s.run_for(5.0);
        assert!(s.snapshot().speed > 100.0);
        s.apply(Command::SetOnOff(false));
        s.run_for(10.0);
        let snap = s.snapshot();
        assert_eq!(snap.torque, 0.0);
        assert!(snap.speed < 5.0, "à l'arrêt la vitesse doit retomber (={})", snap.speed);
    }

    #[test]
    fn setpoint_is_clamped_to_limits() {
        let mut s = Stirrer::new(StirrerConfig::default());
        s.apply(Command::SetSpeed(99999.0));
        assert!((s.snapshot().speed_sp - 2000.0).abs() < 1e-3);
        s.apply(Command::SetSpeed(-50.0));
        assert!((s.snapshot().speed_sp - 0.0).abs() < 1e-3);
    }
}
