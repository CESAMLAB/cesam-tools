//! Protocole **NAMUR** des appareils de laboratoire (façon IKA) : jeu de commandes
//! **ASCII** sur liaison série (ou TCP), une commande par ligne.
//!
//! # Trame
//!
//! Chaque requête est une ligne ASCII terminée par `CR LF`. Les **lectures**
//! (`IN_*`) renvoient une valeur ; les **écritures/actions** (`OUT_*`, `START_*`,
//! `STOP_*`, `RESET`) sont **silencieuses** (pas de réponse), conformément à
//! l'usage NAMUR.
//!
//! # Canaux (agitateur)
//!
//! - canal **4** = vitesse (tr/min) ;
//! - canal **5** = couple (N·cm).
//!
//! # Commandes gérées
//!
//! | Commande        | Effet                                            |
//! |-----------------|--------------------------------------------------|
//! | `IN_NAME`       | nom de l'appareil                                |
//! | `IN_TYPE`       | type d'appareil                                  |
//! | `IN_SW_VERSION` | version du firmware simulé                       |
//! | `IN_PV_4`       | vitesse **mesurée** (tr/min)                     |
//! | `IN_PV_5`       | couple **mesuré** (N·cm)                          |
//! | `IN_SP_4`       | consigne de vitesse (tr/min)                     |
//! | `OUT_SP_4 <v>`  | **régler** la consigne de vitesse                |
//! | `START_4`       | démarrer le moteur                               |
//! | `STOP_4`        | arrêter le moteur                                |
//! | `RESET`         | arrêt + retour en commande locale                |
//! | `OUT_WD1@<m>`   | chien de garde : arrêt sûr si silence > `m` s    |
//! | `OUT_WD2@<m>`   | chien de garde (idem v1 : arrêt sûr)             |

use crate::stirrer::{Command, StirrerSnapshot};

/// Nom d'appareil renvoyé par `IN_NAME`.
pub const DEVICE_NAME: &str = "CESAM-STIRRER";
/// Type d'appareil renvoyé par `IN_TYPE`.
pub const DEVICE_TYPE: &str = "OSNE";
/// Numéro de canal NAMUR de la vitesse.
pub const CHANNEL_SPEED: u8 = 4;
/// Numéro de canal NAMUR du couple.
pub const CHANNEL_TORQUE: u8 = 5;

/// Résultat de l'interprétation d'une ligne NAMUR.
#[derive(Debug, Clone, PartialEq)]
pub enum NamurResponse {
    /// Réponse à renvoyer au maître (sans `CR LF`).
    Reply(String),
    /// Commande métier à appliquer (écriture/action) — silencieuse.
    Apply(Command),
    /// (Re)configure le chien de garde : délai en secondes (`0` = désactivé).
    SetWatchdog(f32),
    /// Ligne reconnue mais sans effet ni réponse (ex. ligne vide).
    Ignore,
    /// Commande inconnue.
    Unknown,
}

/// Formate une réponse de lecture « valeur canal » (ex. `500.0 4`).
fn reply_value(value: f32, channel: u8) -> NamurResponse {
    NamurResponse::Reply(format!("{value:.1} {channel}"))
}

/// Interprète une ligne NAMUR (déjà dépouillée de son `CR LF`).
#[must_use]
pub fn handle_line(line: &str, snap: &StirrerSnapshot) -> NamurResponse {
    let line = line.trim();
    if line.is_empty() {
        return NamurResponse::Ignore;
    }

    // Chien de garde : jeton « OUT_WD1@<m> » / « OUT_WD2@<m> » (le « @ » est collé).
    if let Some(rest) = line
        .strip_prefix("OUT_WD1@")
        .or_else(|| line.strip_prefix("OUT_WD2@"))
    {
        return match rest.trim().parse::<f32>() {
            Ok(secs) if secs.is_finite() => NamurResponse::SetWatchdog(secs.max(0.0)),
            _ => NamurResponse::Unknown,
        };
    }

    // Découpe « commande [argument] ».
    let mut parts = line.split_whitespace();
    let cmd = parts.next().unwrap_or("");
    let arg = parts.next();

    match cmd {
        "IN_NAME" => NamurResponse::Reply(DEVICE_NAME.to_string()),
        "IN_TYPE" => NamurResponse::Reply(DEVICE_TYPE.to_string()),
        "IN_SW_VERSION" | "IN_VERSION" => {
            NamurResponse::Reply(env!("CARGO_PKG_VERSION").to_string())
        }
        "IN_PV_4" => reply_value(snap.speed, CHANNEL_SPEED),
        "IN_PV_5" => reply_value(snap.torque, CHANNEL_TORQUE),
        "IN_SP_4" => reply_value(snap.speed_sp, CHANNEL_SPEED),
        "OUT_SP_4" => match arg.and_then(|a| a.parse::<f32>().ok()) {
            Some(v) if v.is_finite() => NamurResponse::Apply(Command::SetSpeed(v)),
            _ => NamurResponse::Unknown,
        },
        "START_4" => NamurResponse::Apply(Command::SetOnOff(true)),
        "STOP_4" => NamurResponse::Apply(Command::SetOnOff(false)),
        "RESET" => NamurResponse::Apply(Command::SetOnOff(false)),
        _ => NamurResponse::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> StirrerSnapshot {
        StirrerSnapshot {
            on: true,
            speed_sp: 800.0,
            speed: 795.4,
            torque: 42.0,
            torque_max: 100.0,
            viscosity: 1.0,
            overload: false,
            pid: mock_lib_control::PidConfig::default(),
            speed_min: 0.0,
            speed_max: 2000.0,
            viscosity_min: 0.1,
            viscosity_max: 20.0,
            inertia: 0.02,
            load_coeff: 0.05,
            friction: 2.0,
        }
    }

    #[test]
    fn reads_speed_and_torque() {
        let s = sample();
        assert_eq!(handle_line("IN_PV_4", &s), NamurResponse::Reply("795.4 4".into()));
        assert_eq!(handle_line("IN_PV_5", &s), NamurResponse::Reply("42.0 5".into()));
        assert_eq!(handle_line("IN_SP_4", &s), NamurResponse::Reply("800.0 4".into()));
    }

    #[test]
    fn identity_commands() {
        let s = sample();
        assert_eq!(handle_line("IN_NAME", &s), NamurResponse::Reply(DEVICE_NAME.into()));
        assert_eq!(handle_line("IN_TYPE", &s), NamurResponse::Reply(DEVICE_TYPE.into()));
    }

    #[test]
    fn set_speed_and_start_stop() {
        let s = sample();
        assert_eq!(
            handle_line("OUT_SP_4 250", &s),
            NamurResponse::Apply(Command::SetSpeed(250.0))
        );
        assert_eq!(handle_line("START_4", &s), NamurResponse::Apply(Command::SetOnOff(true)));
        assert_eq!(handle_line("STOP_4", &s), NamurResponse::Apply(Command::SetOnOff(false)));
        assert_eq!(handle_line("RESET", &s), NamurResponse::Apply(Command::SetOnOff(false)));
    }

    #[test]
    fn watchdog_and_unknown() {
        let s = sample();
        assert_eq!(handle_line("OUT_WD1@30", &s), NamurResponse::SetWatchdog(30.0));
        assert_eq!(handle_line("OUT_WD2@5.5", &s), NamurResponse::SetWatchdog(5.5));
        assert_eq!(handle_line("OUT_SP_4 abc", &s), NamurResponse::Unknown);
        assert_eq!(handle_line("FOO_BAR", &s), NamurResponse::Unknown);
        assert_eq!(handle_line("   ", &s), NamurResponse::Ignore);
    }
}
