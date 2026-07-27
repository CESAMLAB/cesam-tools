//! Disposition des blocs d'entrées/sorties PROFIBUS DP et conversion vers/depuis
//! l'état métier.
//!
//! # Plan d'adressage
//!
//! Contrairement à Modbus (table de registres adressables individuellement),
//! PROFIBUS DP échange à chaque cycle deux **blocs d'octets de taille fixe**
//! négociés lors du service `Chk_Cfg` : un bloc de **sortie** (maître → esclave,
//! commandes) et un bloc d'**entrée** (esclave → maître, mesure/état). Ce profil
//! est **propre à ce simulateur** (pas un profil GSD enregistré au PNO) ; il
//! reprend le même contenu métier que la table Modbus d'ORME pour une cohérence
//! pédagogique entre les instruments.
//!
//! Les flottants (`f32`) occupent 4 octets consécutifs, **big-endian**.
//!
//! ## Bloc de sortie — *Output* (maître → esclave, [`OUTPUT_LEN`] = 45 octets)
//!
//! | Octet(s) | Symbole            | Type | Description                                 |
//! |----------|--------------------|------|-----------------------------------------------|
//! | `0`      | `OUT_MODE`         | bits | bit0=marche, bit1=auto, [3:2]=mode sens1, [5:4]=mode sens2 |
//! | `1-4`    | `OUT_SP_AUTO`      | f32  | Consigne automatique                          |
//! | `5-8`    | `OUT_SP_MANUAL`    | f32  | Consigne manuelle (% sortie, signée)          |
//! | `9-12`   | `OUT_KP1`          | f32  | Gain proportionnel Kp sens 1                  |
//! | `13-16`  | `OUT_KI1`          | f32  | Gain intégral Ki sens 1                       |
//! | `17-20`  | `OUT_KD1`          | f32  | Gain dérivé Kd sens 1                         |
//! | `21-24`  | `OUT_KP2`          | f32  | Gain proportionnel Kp sens 2                  |
//! | `25-28`  | `OUT_KI2`          | f32  | Gain intégral Ki sens 2                       |
//! | `29-32`  | `OUT_KD2`          | f32  | Gain dérivé Kd sens 2                         |
//! | `33-36`  | `OUT_HYSTERESIS`   | f32  | Hystérésis des régulateurs TOR                |
//! | `37-40`  | `OUT_TOR_MIN_CYCLE`| f32  | Temps de cycle minimal TOR (s)                |
//! | `41-44`  | `OUT_PWM_PERIOD`   | f32  | Période du cycle de modulation PWM (s)        |
//!
//! ## Bloc d'entrée — *Input* (esclave → maître, [`INPUT_LEN`] = 17 octets)
//!
//! | Octet(s) | Symbole       | Type | Description                              |
//! |----------|---------------|------|-------------------------------------------|
//! | `0`      | `IN_STATUS`   | bits | bit0=en marche, bit1=sens1 actif, bit2=sens2 actif |
//! | `1-4`    | `IN_PV`       | f32  | Mesure / *process value*                  |
//! | `5-8`    | `IN_OUTPUT`   | f32  | Sortie appliquée (% signé)                |
//! | `9-12`   | `IN_SP_AUTO`  | f32  | Recopie (lecture seule) de la consigne auto |
//! | `13-16`  | `IN_SP_MANUAL`| f32  | Recopie (lecture seule) de la consigne manuelle |

use mock_lib_control::PidConfig;

use crate::profibus::decode_mode_byte;
use crate::regulator::{Command, RegulatorSnapshot};

pub const OUTPUT_LEN: usize = 45;
pub const INPUT_LEN: usize = 17;

const IN_STATUS_RUNNING: u8 = 0x01;
const IN_STATUS_HEATING: u8 = 0x02;
const IN_STATUS_COOLING: u8 = 0x04;

fn f32_at(data: &[u8], offset: usize) -> f32 {
    f32::from_be_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]])
}

fn put_f32(out: &mut [u8], offset: usize, v: f32) {
    out[offset..offset + 4].copy_from_slice(&v.to_be_bytes());
}

/// Décode le bloc de sortie reçu du maître en une liste de [`Command`] à
/// appliquer au régulateur. Trop court ou malformé : aucune commande (pas de
/// panic), le régulateur conserve son dernier état valide.
#[must_use]
pub fn decode_output(data: &[u8]) -> Vec<Command> {
    if data.len() < OUTPUT_LEN {
        return Vec::new();
    }
    let (on, auto, mode1, mode2) = decode_mode_byte(data[0]);
    vec![
        Command::SetOnOff(on),
        Command::SetAutoManual(auto),
        Command::SetModeSens1(mode1),
        Command::SetModeSens2(mode2),
        Command::SetSpAuto(f32_at(data, 1)),
        Command::SetSpManual(f32_at(data, 5)),
        Command::SetPidHeat(PidConfig {
            kp: f32_at(data, 9),
            ki: f32_at(data, 13),
            kd: f32_at(data, 17),
            out_min: 0.0,
            out_max: 100.0,
        }),
        Command::SetPidCool(PidConfig {
            kp: f32_at(data, 21),
            ki: f32_at(data, 25),
            kd: f32_at(data, 29),
            out_min: 0.0,
            out_max: 100.0,
        }),
        Command::SetHysteresis(f32_at(data, 33)),
        Command::SetTorMinCycle(f32_at(data, 37)),
        Command::SetPwmPeriod(f32_at(data, 41)),
    ]
}

/// Encode l'état courant du régulateur en bloc d'entrée à renvoyer au maître.
#[must_use]
pub fn encode_input(snap: &RegulatorSnapshot) -> Vec<u8> {
    let mut out = vec![0u8; INPUT_LEN];
    let mut status = 0u8;
    if snap.on {
        status |= IN_STATUS_RUNNING;
    }
    if snap.output > 0.0 {
        status |= IN_STATUS_HEATING;
    }
    if snap.output < 0.0 {
        status |= IN_STATUS_COOLING;
    }
    out[0] = status;
    put_f32(&mut out, 1, snap.pv);
    put_f32(&mut out, 5, snap.output);
    put_f32(&mut out, 9, snap.sp_auto);
    put_f32(&mut out, 13, snap.sp_manual);
    out
}

/// Construit un bloc de sortie minimal pour les tests (marche/arrêt, mode, consigne).
#[cfg(test)]
#[must_use]
pub fn encode_output_for_test(on: bool, auto: crate::regulator::AutoManual, sp_auto: f32) -> Vec<u8> {
    use crate::profibus::encode_mode_byte;
    use mock_lib_control::ControllerKind;
    let mut out = vec![0u8; OUTPUT_LEN];
    out[0] = encode_mode_byte(on, auto, ControllerKind::Pid, ControllerKind::Off);
    put_f32(&mut out, 1, sp_auto);
    put_f32(&mut out, 5, 0.0);
    // Gains PID par défaut pour ne pas geler la régulation à 0.
    put_f32(&mut out, 9, 2.0);
    put_f32(&mut out, 13, 0.15);
    put_f32(&mut out, 17, 0.5);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profibus::encode_mode_byte;
    use crate::regulator::AutoManual;
    use mock_lib_control::ControllerKind;

    #[test]
    fn decode_output_too_short_yields_no_commands() {
        assert!(decode_output(&[1, 2, 3]).is_empty());
    }

    #[test]
    fn decode_output_round_trips_setpoint_and_mode() {
        let mut data = vec![0u8; OUTPUT_LEN];
        data[0] = encode_mode_byte(true, AutoManual::Auto, ControllerKind::Pid, ControllerKind::OnOff);
        put_f32(&mut data, 1, 123.5);
        let cmds = decode_output(&data);
        assert!(cmds.contains(&Command::SetOnOff(true)));
        assert!(cmds.contains(&Command::SetAutoManual(AutoManual::Auto)));
        assert!(cmds.contains(&Command::SetModeSens1(ControllerKind::Pid)));
        assert!(cmds.contains(&Command::SetModeSens2(ControllerKind::OnOff)));
        assert!(cmds.contains(&Command::SetSpAuto(123.5)));
    }

    #[test]
    fn encode_input_reflects_snapshot() {
        let snap = RegulatorSnapshot {
            on: true,
            mode: AutoManual::Auto,
            mode_sens1: ControllerKind::Pid,
            mode_sens2: ControllerKind::Off,
            sp_auto: 120.0,
            sp_manual: 0.0,
            pv: 118.4,
            output: 42.0,
            pid_heat: PidConfig::default(),
            pid_cool: PidConfig::default(),
            hysteresis: 2.0,
            tor_min_cycle: 5.0,
            pwm_period: 10.0,
            sp_min: 0.0,
            sp_max: 250.0,
            process_gain: 1.6,
            process_tau: 30.0,
            process_dead_time: 2.0,
            ambient: 20.0,
        };
        let bytes = encode_input(&snap);
        assert_eq!(bytes.len(), INPUT_LEN);
        assert_eq!(bytes[0] & IN_STATUS_RUNNING, IN_STATUS_RUNNING);
        assert_eq!(bytes[0] & IN_STATUS_HEATING, IN_STATUS_HEATING);
        assert_eq!(f32_at(&bytes, 1), 118.4);
    }
}
