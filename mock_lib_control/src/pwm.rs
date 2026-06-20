//! Modulation de largeur d'impulsion (relais à cycle / *time-proportioning*).
//!
//! Permet de piloter un actionneur **tout-ou-rien** (relais, triac, vanne tor)
//! avec une commande quasi continue : sur une période de cycle fixe `T_c`, la
//! sortie reste active pendant la fraction `duty` de la période puis inactive le
//! reste du temps. La **valeur moyenne** de la sortie suit donc `duty`, ce que
//! font les régulateurs thermiques industriels pour réguler finement avec un
//! organe qui ne sait que s'ouvrir ou se fermer.
//!
//! Le rapport cyclique `duty` provient typiquement d'un [`crate::Pid`] : on
//! conserve ainsi la qualité de réglage d'un PID tout en commandant un organe TOR.

/// Modulateur de largeur d'impulsion à période fixe.
///
/// Le rapport cyclique est **échantillonné et figé au début de chaque période**
/// puis maintenu constant jusqu'au cycle suivant. C'est le comportement des
/// régulateurs à relais à cycle réels et cela découple le PID de l'ondulation
/// intra-cycle de la mesure : sans ce maintien, la corrélation entre le rapport
/// cyclique recalculé en continu et l'ondulation introduit un biais d'environ
/// quelques pour-cent en régime établi.
#[derive(Debug, Clone)]
pub struct Pwm {
    /// Période de cycle `T_c` en secondes (toujours strictement positive).
    period: f32,
    /// Temps écoulé (s) depuis le début de la période courante.
    elapsed: f32,
    /// Rapport cyclique figé pour la période en cours (fraction `0..1`).
    held_duty: f32,
    /// `false` tant qu'aucune période n'a encore échantillonné le rapport cyclique.
    started: bool,
}

impl Pwm {
    /// Crée un modulateur de période `period` (s, forcée à une valeur > 0).
    #[must_use]
    pub fn new(period: f32) -> Self {
        Self {
            period: period.max(1e-3),
            elapsed: 0.0,
            held_duty: 0.0,
            started: false,
        }
    }

    /// Met à jour la période de cycle (s) sans réinitialiser la phase courante.
    pub fn set_period(&mut self, period: f32) {
        self.period = period.max(1e-3);
    }

    /// Réinitialise la phase au début d'une période et oublie le rapport cyclique figé.
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.held_duty = 0.0;
        self.started = false;
    }

    /// Avance d'un pas `dt` (s) et renvoie l'état de la sortie tout-ou-rien pour
    /// le rapport cyclique `duty` (fraction `0..1`, bornée).
    ///
    /// `duty` n'est échantillonné qu'au **début de chaque période** (et au tout
    /// premier appel) ; il est ensuite maintenu pour le reste du cycle. La sortie
    /// est active pendant la première fraction `held_duty` de la période :
    /// `duty = 0` => toujours inactif, `duty = 1` => toujours actif.
    pub fn update(&mut self, duty: f32, dt: f32) -> bool {
        let duty = duty.clamp(0.0, 1.0);
        // Échantillonnage initial puis à chaque franchissement de période.
        if !self.started {
            self.held_duty = duty;
            self.started = true;
        }
        if dt > 0.0 {
            self.elapsed += dt;
        }
        if self.elapsed >= self.period {
            self.elapsed %= self.period;
            self.held_duty = duty;
        }
        self.elapsed < self.held_duty * self.period
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duty_zero_and_one_are_constant() {
        let mut pwm = Pwm::new(10.0);
        // Rapport cyclique nul -> jamais actif.
        for _ in 0..1000 {
            assert!(!pwm.update(0.0, 0.05));
        }
        // Rapport cyclique plein -> toujours actif.
        let mut pwm = Pwm::new(10.0);
        for _ in 0..1000 {
            assert!(pwm.update(1.0, 0.05));
        }
    }

    #[test]
    fn average_output_tracks_duty() {
        let mut pwm = Pwm::new(10.0);
        let dt = 0.05;
        let duty = 0.3;
        let mut on = 0u32;
        let total = 4000; // 20 périodes
        for _ in 0..total {
            if pwm.update(duty, dt) {
                on += 1;
            }
        }
        let ratio = on as f32 / total as f32;
        assert!((ratio - duty).abs() < 0.02, "rapport cyclique moyen = {ratio}");
    }
}
