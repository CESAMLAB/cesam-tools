//! Modèle de procédé : fonction de transfert du premier ordre avec retard pur.

use std::collections::VecDeque;

/// Procédé du premier ordre avec retard pur (FOPDT — *First Order Plus Dead Time*).
///
/// Représente la fonction de transfert continue :
///
/// ```text
///            K · e^(-L·s)
///   G(s) = ----------------
///             1 + T · s
/// ```
///
/// où :
/// - `K` (`gain`) est le gain statique, exprimé en **unité de mesure par % de sortie**
///   (par ex. °C/%) ;
/// - `T` (`tau`) est la constante de temps en secondes ;
/// - `L` (`dead_time`) est le retard pur en secondes ;
/// - `ambient` est la valeur de repos atteinte lorsque la commande est nulle.
///
/// La valeur de sortie (mesure / *process value*) tend, en régime établi, vers
/// `ambient + K · u`, où `u` est la commande appliquée (en %, signée pour gérer
/// le chaud `u > 0` et le froid `u < 0`).
///
/// L'intégration est réalisée par la méthode d'Euler explicite, et le retard pur
/// est modélisé par une ligne à retard (`VecDeque`).
#[derive(Debug, Clone)]
pub struct FirstOrderProcess {
    gain: f32,
    tau: f32,
    dead_time: f32,
    ambient: f32,
    /// Mesure courante.
    value: f32,
    /// Ligne à retard contenant les commandes en attente d'application.
    delay_line: VecDeque<f32>,
}

impl FirstOrderProcess {
    /// Crée un procédé.
    ///
    /// * `gain` — gain statique `K` (unité de mesure par %).
    /// * `tau` — constante de temps `T` en secondes (forcée à une valeur > 0).
    /// * `dead_time` — retard pur `L` en secondes (≥ 0).
    /// * `ambient` — valeur de repos, qui sert aussi de valeur initiale.
    #[must_use]
    pub fn new(gain: f32, tau: f32, dead_time: f32, ambient: f32) -> Self {
        Self {
            gain,
            tau: tau.max(1e-3),
            dead_time: dead_time.max(0.0),
            ambient,
            value: ambient,
            delay_line: VecDeque::new(),
        }
    }

    /// Mesure (process value) courante.
    #[must_use]
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Réinitialise la mesure et vide la ligne à retard.
    pub fn reset(&mut self, value: f32) {
        self.value = value;
        self.delay_line.clear();
    }

    /// Met à jour les paramètres de la fonction de transfert **sans toucher à la
    /// mesure courante** (pas de saut de la sortie). La ligne à retard est vidée
    /// car sa profondeur dépend du retard pur.
    pub fn reconfigure(&mut self, gain: f32, tau: f32, dead_time: f32, ambient: f32) {
        self.gain = gain;
        self.tau = tau.max(1e-3);
        self.dead_time = dead_time.max(0.0);
        self.ambient = ambient;
        self.delay_line.clear();
    }

    /// Avance la simulation d'un pas `dt` (secondes) avec la commande `command` (%),
    /// puis renvoie la nouvelle mesure.
    ///
    /// `command` est signée : positive pour le chaud, négative pour le froid.
    pub fn step(&mut self, command: f32, dt: f32) -> f32 {
        if dt <= 0.0 {
            return self.value;
        }

        // Gestion du retard pur via une ligne à retard dimensionnée selon dt.
        // On stocke `depth` échantillons : la commande qui agit « maintenant » est
        // celle émise `depth` pas plus tôt.
        let delayed_command = if self.dead_time <= 0.0 {
            command
        } else {
            let depth = (self.dead_time / dt).round().max(1.0) as usize;
            self.delay_line.push_back(command);
            if self.delay_line.len() > depth {
                self.delay_line.pop_front().unwrap_or(command)
            } else {
                *self.delay_line.front().unwrap_or(&command)
            }
        };

        // Cible de régime établi pour cette commande.
        let target = self.ambient + self.gain * delayed_command;
        // Euler explicite : dv = dt/T · (cible − v).
        let alpha = (dt / self.tau).min(1.0);
        self.value += alpha * (target - self.value);
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settles_to_ambient_when_idle() {
        let mut p = FirstOrderProcess::new(1.5, 10.0, 0.0, 20.0);
        for _ in 0..2000 {
            p.step(0.0, 0.1);
        }
        assert!((p.value() - 20.0).abs() < 0.1);
    }

    #[test]
    fn settles_to_ambient_plus_gain_times_command() {
        let mut p = FirstOrderProcess::new(1.5, 10.0, 0.0, 20.0);
        for _ in 0..5000 {
            p.step(100.0, 0.1);
        }
        // Régime établi attendu : 20 + 1.5 * 100 = 170.
        assert!((p.value() - 170.0).abs() < 1.0);
    }

    #[test]
    fn cooling_command_goes_below_ambient() {
        let mut p = FirstOrderProcess::new(1.0, 5.0, 0.0, 20.0);
        for _ in 0..5000 {
            p.step(-50.0, 0.1);
        }
        assert!(p.value() < 20.0);
    }
}
