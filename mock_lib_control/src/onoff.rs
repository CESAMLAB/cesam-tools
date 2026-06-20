//! Régulateur tout-ou-rien (TOR) avec hystérésis symétrique et anti-court-cycle.

/// Valeur initiale du compteur « temps passé dans l'état » : suffisamment grande
/// pour qu'une **première** commutation soit toujours autorisée (pas de délai au
/// démarrage), quelle que soit la durée de cycle minimale configurée.
const READY_TO_SWITCH: f32 = 1e9;

/// Régulateur tout-ou-rien à hystérésis symétrique avec temps de cycle minimal.
///
/// L'entrée est une **erreur orientée** (positive = il faut agir) :
/// - la sortie passe à `true` (actif) lorsque `erreur > hystérésis / 2` ;
/// - la sortie passe à `false` (inactif) lorsque `erreur < −hystérésis / 2` ;
/// - entre les deux seuils, l'état précédent est conservé (zone morte).
///
/// Le relais **garde son état** à la traversée de la consigne : c'est à
/// l'appelant de lui fournir l'erreur signée à chaque pas (sans le réinitialiser
/// au changement de signe), faute de quoi la bande d'hystérésis se retrouve
/// tronquée à `[consigne − h/2, consigne]` au lieu de `[consigne − h/2,
/// consigne + h/2]`.
///
/// Un **temps de cycle minimal** (`min_cycle`, en secondes) interdit toute
/// commutation tant que le relais n'est pas resté au moins cette durée dans son
/// état courant. Cela modélise la protection d'un actionneur réel (relais,
/// compresseur) contre les cycles trop rapides (« court-cycle ») et évite le
/// broutage lorsque le pas de simulation est petit. `0` désactive la temporisation.
///
/// Pour un sens « chaud », fournir `erreur = consigne − mesure`.
/// Pour un sens « froid », fournir `erreur = mesure − consigne`.
#[derive(Debug, Clone)]
pub struct OnOff {
    hysteresis: f32,
    min_cycle: f32,
    state: bool,
    /// Temps écoulé (s) dans l'état courant, pour l'anti-court-cycle.
    time_in_state: f32,
}

impl OnOff {
    /// Crée un régulateur TOR avec la largeur d'hystérésis donnée (en unités de mesure)
    /// et sans temps de cycle minimal.
    #[must_use]
    pub fn new(hysteresis: f32) -> Self {
        Self::with_min_cycle(hysteresis, 0.0)
    }

    /// Crée un régulateur TOR avec hystérésis (unité de mesure) et temps de cycle
    /// minimal (secondes).
    #[must_use]
    pub fn with_min_cycle(hysteresis: f32, min_cycle: f32) -> Self {
        Self {
            hysteresis: hysteresis.max(0.0),
            min_cycle: min_cycle.max(0.0),
            state: false,
            time_in_state: READY_TO_SWITCH,
        }
    }

    /// Met à jour la largeur d'hystérésis sans changer l'état courant.
    pub fn set_hysteresis(&mut self, hysteresis: f32) {
        self.hysteresis = hysteresis.max(0.0);
    }

    /// Met à jour le temps de cycle minimal (secondes) sans changer l'état courant.
    pub fn set_min_cycle(&mut self, min_cycle: f32) {
        self.min_cycle = min_cycle.max(0.0);
    }

    /// Force l'état inactif (par exemple lors d'un arrêt de l'appareil) et
    /// réarme la commutation immédiate.
    pub fn reset(&mut self) {
        self.state = false;
        self.time_in_state = READY_TO_SWITCH;
    }

    /// État courant (`true` = sortie active).
    #[must_use]
    pub fn is_on(&self) -> bool {
        self.state
    }

    /// Met à jour l'état à partir d'une erreur orientée et du pas de temps `dt`
    /// (secondes), puis renvoie la sortie booléenne.
    ///
    /// Une commutation n'est appliquée que si le relais est resté au moins
    /// `min_cycle` secondes dans son état courant (anti-court-cycle).
    pub fn update(&mut self, error: f32, dt: f32) -> bool {
        if dt > 0.0 {
            self.time_in_state += dt;
        }
        let half = self.hysteresis / 2.0;
        // État souhaité d'après l'hystérésis ; zone morte = on garde l'état.
        let want = if error > half {
            true
        } else if error < -half {
            false
        } else {
            self.state
        };
        if want != self.state && self.time_in_state >= self.min_cycle {
            self.state = want;
            self.time_in_state = 0.0;
        }
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hysteresis_creates_deadband() {
        let mut tor = OnOff::new(2.0); // seuils à ±1.0, sans temporisation
        assert!(tor.update(5.0, 1.0)); // bien au-dessus -> ON
        assert!(tor.update(0.5, 1.0)); // dans la zone morte -> reste ON
        assert!(!tor.update(-5.0, 1.0)); // bien en-dessous -> OFF
        assert!(!tor.update(-0.5, 1.0)); // zone morte -> reste OFF
    }

    #[test]
    fn min_cycle_blocks_fast_toggling() {
        // Hystérésis nulle (commutation au signe), cycle minimal de 5 s.
        let mut tor = OnOff::with_min_cycle(0.0, 5.0);
        // Première commutation autorisée immédiatement (réarmé au démarrage).
        assert!(tor.update(1.0, 1.0));
        // La demande s'inverse mais on est ON depuis 0 s -> reste ON (bloqué).
        assert!(tor.update(-1.0, 1.0)); // 1 s dans l'état
        assert!(tor.update(-1.0, 1.0)); // 2 s
        assert!(tor.update(-1.0, 1.0)); // 3 s
        assert!(tor.update(-1.0, 1.0)); // 4 s
        // 5 s écoulées dans l'état ON -> la commutation vers OFF est enfin permise.
        assert!(!tor.update(-1.0, 1.0));
    }
}
