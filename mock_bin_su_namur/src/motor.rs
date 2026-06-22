//! Modèle physique du moteur d'agitation : dynamique rotationnelle du premier
//! ordre avec **charge visqueuse** réglable.
//!
//! La vitesse `ω` (tr/min) évolue selon l'équilibre des couples (Euler explicite) :
//!
//! ```text
//!   J · dω/dt = T_moteur − k · η · ω − T_frottement
//! ```
//!
//! où :
//! - `T_moteur` est le couple moteur appliqué (commande, en N·cm, ≥ 0) ;
//! - `k · η · ω` est le **couple de charge visqueux** : proportionnel à la vitesse
//!   et à la **viscosité relative** `η` (1.0 = référence type « eau ») ;
//! - `T_frottement` est un frottement sec résiduel (N·cm) ;
//! - `J` (`inertia`) règle la **réactivité** : plus il est petit, plus la réponse
//!   est rapide.
//!
//! En régime établi, `T_moteur = k·η·ω + T_frottement` : le couple nécessaire pour
//! tenir une vitesse **croît avec la viscosité**, ce qui reproduit la « tendance de
//! couple » d'un agitateur réel et, à forte viscosité, la **surcharge** (le couple
//! maximal ne suffit plus à atteindre la consigne).

/// Moteur d'agitation simulé (dynamique de vitesse + charge visqueuse).
#[derive(Debug, Clone)]
pub struct Motor {
    /// Inertie / réactivité `J` (petit = réponse rapide).
    inertia: f32,
    /// Coefficient de charge visqueuse `k` (couple par unité de viscosité et de vitesse).
    load_coeff: f32,
    /// Frottement sec résiduel (N·cm) opposé au mouvement.
    friction: f32,
    /// Vitesse courante (tr/min, ≥ 0 : sens unique).
    speed: f32,
}

impl Motor {
    /// Crée un moteur. `inertia` est forcé > 0, les autres coefficients ≥ 0.
    #[must_use]
    pub fn new(inertia: f32, load_coeff: f32, friction: f32) -> Self {
        Self {
            inertia: inertia.max(1e-4),
            load_coeff: load_coeff.max(0.0),
            friction: friction.max(0.0),
            speed: 0.0,
        }
    }

    /// Vitesse courante (tr/min).
    #[must_use]
    pub fn speed(&self) -> f32 {
        self.speed
    }

    /// Couple de charge visqueux courant (N·cm) pour la viscosité `viscosity`.
    #[must_use]
    pub fn load_torque(&self, viscosity: f32) -> f32 {
        self.load_coeff * viscosity.max(0.0) * self.speed
    }

    /// Met à jour les paramètres sans toucher à la vitesse courante.
    pub fn reconfigure(&mut self, inertia: f32, load_coeff: f32, friction: f32) {
        self.inertia = inertia.max(1e-4);
        self.load_coeff = load_coeff.max(0.0);
        self.friction = friction.max(0.0);
    }

    /// Avance d'un pas `dt` (s) avec un couple moteur `drive` (N·cm) et une
    /// viscosité relative `viscosity`. Renvoie la nouvelle vitesse (tr/min, ≥ 0).
    pub fn step(&mut self, drive: f32, viscosity: f32, dt: f32) -> f32 {
        if dt <= 0.0 {
            return self.speed;
        }
        let drive = drive.max(0.0);
        let load = self.load_torque(viscosity);
        // Le frottement sec ne s'oppose qu'au mouvement réel (évite un couple
        // « négatif » qui ferait reculer un moteur à l'arrêt).
        let friction = if self.speed > 1e-3 { self.friction } else { 0.0 };
        let net = drive - load - friction;
        self.speed = (self.speed + net / self.inertia * dt).max(0.0);
        self.speed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn higher_viscosity_lowers_steady_speed_for_same_drive() {
        // À couple moteur constant, une viscosité plus forte = vitesse plus basse.
        let run = |visc: f32| {
            let mut m = Motor::new(0.02, 0.05, 2.0);
            for _ in 0..5000 {
                m.step(80.0, visc, 0.02);
            }
            m.speed()
        };
        let low = run(1.0);
        let high = run(4.0);
        assert!(high < low, "visc 4 ({high}) doit tourner plus lentement que visc 1 ({low})");
        assert!(low > 0.0);
    }

    #[test]
    fn load_torque_grows_with_speed_and_viscosity() {
        let mut m = Motor::new(0.02, 0.05, 1.0);
        for _ in 0..2000 {
            m.step(60.0, 2.0, 0.02);
        }
        let t = m.load_torque(2.0);
        assert!(t > 0.0);
        // Doublé la viscosité => couple de charge doublé à vitesse égale.
        assert!((m.load_torque(4.0) - 2.0 * t).abs() < 1e-3);
    }

    #[test]
    fn zero_drive_decelerates_to_zero() {
        let mut m = Motor::new(0.02, 0.05, 2.0);
        for _ in 0..1000 {
            m.step(80.0, 1.0, 0.02);
        }
        assert!(m.speed() > 0.0);
        for _ in 0..5000 {
            m.step(0.0, 1.0, 0.02);
        }
        assert!(m.speed() < 1.0, "sans couple moteur, la vitesse doit retomber à 0");
    }
}
