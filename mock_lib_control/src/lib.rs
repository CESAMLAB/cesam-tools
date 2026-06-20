//! # mock_lib_control
//!
//! Briques de **régulation** et de **simulation de procédé** réutilisables par les
//! différents instruments simulés du workspace `Mock`.
//!
//! Le module est volontairement *pur* (aucune dépendance async / IO) afin de
//! pouvoir être testé unitairement et réutilisé dans n'importe quel contexte :
//!
//! - [`Pid`] : régulateur PID continu avec anti-emballement (anti-windup).
//! - [`OnOff`] : régulateur tout-ou-rien (TOR) avec hystérésis symétrique et
//!   anti-court-cycle.
//! - [`Pwm`] : modulateur de largeur d'impulsion (relais à cycle) pour piloter un
//!   organe TOR avec une commande continue.
//! - [`FirstOrderProcess`] : modèle de procédé du premier ordre avec retard pur
//!   (fonction de transfert `K·e^(-Ls) / (1 + T·s)`), typique d'un procédé thermique.
//!
//! Les conventions de signe sont décrites sur chaque type. Toutes les grandeurs
//! temporelles (`dt`, `tau`, `dead_time`) sont en **secondes**.

mod onoff;
mod pid;
mod process;
mod pwm;

pub use onoff::OnOff;
pub use pid::{Pid, PidConfig};
pub use process::FirstOrderProcess;
pub use pwm::Pwm;

/// Type d'algorithme de régulation appliqué à un sens donné (chaud ou froid).
///
/// Correspond à la valeur transportée par la table d'adresses Modbus pour les
/// registres « Mode de régulation sens 1 / sens 2 ».
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ControllerKind {
    /// Sens désactivé : aucune action de régulation dans ce sens.
    #[default]
    Off,
    /// Régulation PID continue (sortie 0..100 %).
    Pid,
    /// Régulation tout-ou-rien (TOR) avec hystérésis.
    OnOff,
    /// Régulation par train d'impulsions (relais à cycle / PWM) : un PID calcule
    /// le rapport cyclique, modulé sur un organe tout-ou-rien.
    Pwm,
}

impl ControllerKind {
    /// Code numérique utilisé dans la table Modbus (`0 = Off`, `1 = PID`, `2 = TOR`, `3 = PWM`).
    #[must_use]
    pub const fn to_code(self) -> u16 {
        match self {
            ControllerKind::Off => 0,
            ControllerKind::Pid => 1,
            ControllerKind::OnOff => 2,
            ControllerKind::Pwm => 3,
        }
    }

    /// Décode une valeur de registre Modbus. Toute valeur inconnue retombe sur [`ControllerKind::Off`].
    #[must_use]
    pub const fn from_code(code: u16) -> Self {
        match code {
            1 => ControllerKind::Pid,
            2 => ControllerKind::OnOff,
            3 => ControllerKind::Pwm,
            _ => ControllerKind::Off,
        }
    }
}
