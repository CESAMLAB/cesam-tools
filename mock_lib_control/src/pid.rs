//! Régulateur PID continu avec anti-emballement.

/// Paramètres de réglage (tuning) d'un [`Pid`].
///
/// La sortie est bornée à `[out_min, out_max]`. Le terme intégral est lui aussi
/// borné à ce même intervalle, ce qui fournit un anti-emballement simple mais
/// robuste (« integral clamping »).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PidConfig {
    /// Gain proportionnel `Kp`.
    pub kp: f32,
    /// Gain intégral `Ki` (par seconde).
    pub ki: f32,
    /// Gain dérivé `Kd` (secondes).
    pub kd: f32,
    /// Borne basse de la sortie.
    pub out_min: f32,
    /// Borne haute de la sortie.
    pub out_max: f32,
}

impl Default for PidConfig {
    fn default() -> Self {
        Self {
            kp: 2.0,
            ki: 0.15,
            kd: 0.5,
            out_min: 0.0,
            out_max: 100.0,
        }
    }
}

/// Régulateur PID à temps discret.
///
/// - Terme intégral accumulé à chaque pas et borné (anti-windup).
/// - Terme dérivé calculé sur l'erreur.
///
/// La convention de signe est : une **erreur positive produit une sortie qui
/// augmente**. Pour un sens « chaud », fournir `erreur = consigne − mesure` ;
/// pour un sens « froid », fournir `erreur = mesure − consigne` (voir
/// [`Pid::step_with_error`]).
#[derive(Debug, Clone)]
pub struct Pid {
    cfg: PidConfig,
    integral: f32,
    prev_error: f32,
    initialized: bool,
}

impl Pid {
    /// Crée un PID à partir de sa configuration.
    #[must_use]
    pub fn new(cfg: PidConfig) -> Self {
        Self {
            cfg,
            integral: 0.0,
            prev_error: 0.0,
            initialized: false,
        }
    }

    /// Remplace la configuration sans réinitialiser l'état interne.
    pub fn set_config(&mut self, cfg: PidConfig) {
        self.cfg = cfg;
        // Le terme intégral doit rester cohérent avec les nouvelles bornes.
        self.integral = self.integral.clamp(self.cfg.out_min, self.cfg.out_max);
    }

    /// Configuration courante.
    #[must_use]
    pub fn config(&self) -> PidConfig {
        self.cfg
    }

    /// Réinitialise l'état dynamique (intégrale et mémoire de dérivée).
    pub fn reset(&mut self) {
        self.integral = 0.0;
        self.prev_error = 0.0;
        self.initialized = false;
    }

    /// Vide uniquement le terme intégral (utile lorsqu'un sens devient inactif).
    pub fn reset_integral(&mut self) {
        self.integral = 0.0;
    }

    /// Calcule la sortie à partir d'une consigne `sp` et d'une mesure `pv`.
    ///
    /// `dt` est la période d'échantillonnage en secondes.
    pub fn step(&mut self, sp: f32, pv: f32, dt: f32) -> f32 {
        self.step_with_error(sp - pv, dt)
    }

    /// Calcule la sortie à partir d'une erreur déjà orientée.
    ///
    /// Permet de réutiliser le même algorithme pour un sens « froid » en passant
    /// `erreur = mesure − consigne`.
    pub fn step_with_error(&mut self, error: f32, dt: f32) -> f32 {
        if dt <= 0.0 {
            return (self.cfg.kp * error + self.integral).clamp(self.cfg.out_min, self.cfg.out_max);
        }

        // Terme intégral avec anti-emballement par bornage.
        self.integral += self.cfg.ki * error * dt;
        self.integral = self.integral.clamp(self.cfg.out_min, self.cfg.out_max);

        // Terme dérivé sur l'erreur. Au premier pas on évite le « coup de fouet ».
        let derivative = if self.initialized {
            (error - self.prev_error) / dt
        } else {
            0.0
        };
        self.prev_error = error;
        self.initialized = true;

        let output = self.cfg.kp * error + self.integral + self.cfg.kd * derivative;
        output.clamp(self.cfg.out_min, self.cfg.out_max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proportional_only_tracks_error() {
        let cfg = PidConfig {
            kp: 1.0,
            ki: 0.0,
            kd: 0.0,
            out_min: -100.0,
            out_max: 100.0,
        };
        let mut pid = Pid::new(cfg);
        assert_eq!(pid.step(50.0, 30.0, 0.1), 20.0);
    }

    #[test]
    fn output_is_clamped() {
        let mut pid = Pid::new(PidConfig::default());
        let out = pid.step(1000.0, 0.0, 0.1);
        assert!(out <= 100.0, "la sortie doit être bornée à out_max");
    }

    #[test]
    fn integral_anti_windup() {
        let mut pid = Pid::new(PidConfig::default());
        for _ in 0..1000 {
            pid.step(1000.0, 0.0, 0.1);
        }
        // Même après une longue saturation, la sortie redevient bornée immédiatement.
        let out = pid.step(0.0, 0.0, 0.1);
        assert!((0.0..=100.0).contains(&out));
    }
}
