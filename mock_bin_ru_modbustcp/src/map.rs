//! Table d'adresses Modbus du régulateur et conversion vers/depuis l'état métier.
//!
//! # Plan d'adressage
//!
//! Les flottants (`f32`) occupent **2 registres consécutifs**, encodés en
//! big-endian, **mot de poids fort en premier** (ordre ABCD, le plus répandu).
//!
//! ## Bobines — *coils* (lecture/écriture, FC 1 / 5 / 15)
//!
//! | Adresse | Symbole              | Description                          |
//! |---------|----------------------|--------------------------------------|
//! | `0`     | `COIL_ON_OFF`        | Marche (1) / Arrêt (0)               |
//! | `1`     | `COIL_AUTO_MANUAL`   | Auto (1) / Manuel (0)                |
//!
//! ## Entrées discrètes — *discrete inputs* (lecture seule, FC 2)
//!
//! | Adresse | Symbole          | Description                |
//! |---------|------------------|----------------------------|
//! | `0`     | `DI_RUNNING`     | Appareil en marche         |
//! | `1`     | `DI_HEATING`     | Sens 1 (chaud) actif       |
//! | `2`     | `DI_COOLING`     | Sens 2 (froid) actif       |
//!
//! ## Registres de maintien — *holding registers* (lecture/écriture, FC 3 / 6 / 16)
//!
//! | Adresse | Symbole            | Type | Description                                 |
//! |---------|--------------------|------|---------------------------------------------|
//! | `0`     | `HR_MODE_SENS1`    | u16  | Mode sens 1 : 0=Off, 1=PID, 2=TOR, 3=PWM    |
//! | `1`     | `HR_MODE_SENS2`    | u16  | Mode sens 2 : 0=Off, 1=PID, 2=TOR, 3=PWM    |
//! | `2-3`   | `HR_SP_AUTO`       | f32  | Consigne automatique (unité de mesure)      |
//! | `4-5`   | `HR_SP_MANUAL`     | f32  | Consigne manuelle (% sortie, signée)        |
//! | `6-7`   | `HR_KP_SENS1`      | f32  | Gain proportionnel Kp sens 1                |
//! | `8-9`   | `HR_KI_SENS1`      | f32  | Gain intégral Ki sens 1                     |
//! | `10-11` | `HR_KD_SENS1`      | f32  | Gain dérivé Kd sens 1                       |
//! | `12-13` | `HR_KP_SENS2`      | f32  | Gain proportionnel Kp sens 2                |
//! | `14-15` | `HR_KI_SENS2`      | f32  | Gain intégral Ki sens 2                     |
//! | `16-17` | `HR_KD_SENS2`      | f32  | Gain dérivé Kd sens 2                       |
//! | `18-19` | `HR_HYSTERESIS`    | f32  | Hystérésis des régulateurs TOR              |
//! | `20-21` | `HR_TOR_MIN_CYCLE` | f32  | Temps de cycle minimal TOR (s, anti-court-cycle) |
//! | `22-23` | `HR_PWM_PERIOD`    | f32  | Période du cycle de modulation PWM (s)      |
//! | `42-46` | `HR_LABEL`         | ASCII| Identifiant « CESAM-Lab » (lecture seule, 2 car./registre) |
//!
//! ## Registres d'entrée — *input registers* (lecture seule, FC 4)
//!
//! | Adresse | Symbole        | Type | Description                          |
//! |---------|----------------|------|--------------------------------------|
//! | `0-1`   | `IR_PV`        | f32  | Mesure / *process value*             |
//! | `2-3`   | `IR_OUTPUT`    | f32  | Sortie appliquée (% signé)           |

use mock_lib_control::{ControllerKind, PidConfig};

use crate::regulator::{AutoManual, Command, RegulatorSnapshot};

// --- Bobines ---
pub const COIL_ON_OFF: u16 = 0;
pub const COIL_AUTO_MANUAL: u16 = 1;
pub const COIL_COUNT: usize = 2;

// --- Entrées discrètes ---
pub const DI_RUNNING: u16 = 0;
pub const DI_HEATING: u16 = 1;
pub const DI_COOLING: u16 = 2;
pub const DISCRETE_COUNT: usize = 3;

// --- Registres de maintien ---
pub const HR_MODE_SENS1: u16 = 0;
pub const HR_MODE_SENS2: u16 = 1;
pub const HR_SP_AUTO: u16 = 2;
pub const HR_SP_MANUAL: u16 = 4;
pub const HR_KP_SENS1: u16 = 6;
pub const HR_KI_SENS1: u16 = 8;
pub const HR_KD_SENS1: u16 = 10;
pub const HR_KP_SENS2: u16 = 12;
pub const HR_KI_SENS2: u16 = 14;
pub const HR_KD_SENS2: u16 = 16;
pub const HR_HYSTERESIS: u16 = 18;
pub const HR_TOR_MIN_CYCLE: u16 = 20;
pub const HR_PWM_PERIOD: u16 = 22;
/// Chaîne d'identification ASCII (lecture seule), 2 caractères par registre,
/// caractère de poids fort en premier. Voir [`LABEL_TEXT`].
pub const HR_LABEL: u16 = 42;
/// Texte d'identification exposé à partir de [`HR_LABEL`] (« CESAM-Lab »).
pub const LABEL_TEXT: &str = "CESAM-Lab";
// La table de maintien s'étend jusqu'à couvrir la chaîne d'identification
// (« CESAM-Lab » = 9 octets -> 5 registres : 42..=46). Les registres 24..41
// sont réservés (lus à 0).
pub const HOLDING_COUNT: usize = 47;

// --- Registres d'entrée ---
pub const IR_PV: u16 = 0;
pub const IR_OUTPUT: u16 = 2;
pub const INPUT_COUNT: usize = 4;

/// Découpe un `f32` en 2 registres Modbus (mot de poids fort en premier).
#[must_use]
pub fn f32_to_regs(value: f32) -> [u16; 2] {
    let b = value.to_be_bytes();
    [
        u16::from_be_bytes([b[0], b[1]]),
        u16::from_be_bytes([b[2], b[3]]),
    ]
}

/// Reconstitue un `f32` à partir de 2 registres Modbus (mot de poids fort en premier).
#[must_use]
pub fn regs_to_f32(hi: u16, lo: u16) -> f32 {
    let hi = hi.to_be_bytes();
    let lo = lo.to_be_bytes();
    f32::from_be_bytes([hi[0], hi[1], lo[0], lo[1]])
}

/// Image mémoire Modbus de l'appareil : les quatre tables standard.
///
/// Elle est rafraîchie à chaque pas de simulation par l'acteur, et lue par le
/// serveur Modbus pour répondre aux requêtes.
#[derive(Debug, Clone)]
pub struct MemoryMap {
    pub coils: Vec<bool>,
    pub discretes: Vec<bool>,
    pub holdings: Vec<u16>,
    pub inputs: Vec<u16>,
}

impl Default for MemoryMap {
    fn default() -> Self {
        Self {
            coils: vec![false; COIL_COUNT],
            discretes: vec![false; DISCRETE_COUNT],
            holdings: vec![0; HOLDING_COUNT],
            inputs: vec![0; INPUT_COUNT],
        }
    }
}

impl MemoryMap {
    /// Écrit un `f32` dans les registres de maintien à l'adresse donnée.
    fn set_holding_f32(&mut self, addr: u16, value: f32) {
        let [hi, lo] = f32_to_regs(value);
        let a = addr as usize;
        self.holdings[a] = hi;
        self.holdings[a + 1] = lo;
    }

    /// Écrit un `f32` dans les registres d'entrée à l'adresse donnée.
    fn set_input_f32(&mut self, addr: u16, value: f32) {
        let [hi, lo] = f32_to_regs(value);
        let a = addr as usize;
        self.inputs[a] = hi;
        self.inputs[a + 1] = lo;
    }

    /// Écrit une chaîne ASCII dans les registres de maintien à partir de `addr` :
    /// 2 caractères par registre, caractère de poids fort en premier ; le dernier
    /// octet est complété par `0` si la longueur est impaire.
    fn set_holding_ascii(&mut self, addr: u16, text: &str) {
        let bytes = text.as_bytes();
        let mut reg = addr as usize;
        let mut i = 0;
        while i < bytes.len() && reg < self.holdings.len() {
            let hi = bytes[i];
            let lo = bytes.get(i + 1).copied().unwrap_or(0);
            self.holdings[reg] = u16::from_be_bytes([hi, lo]);
            reg += 1;
            i += 2;
        }
    }

    /// Recopie l'état métier dans l'image mémoire Modbus.
    pub fn refresh_from(&mut self, s: &RegulatorSnapshot) {
        // Bobines
        self.coils[COIL_ON_OFF as usize] = s.on;
        self.coils[COIL_AUTO_MANUAL as usize] = s.mode.is_auto();

        // Entrées discrètes (état)
        self.discretes[DI_RUNNING as usize] = s.on;
        self.discretes[DI_HEATING as usize] = s.on && s.output > 0.0;
        self.discretes[DI_COOLING as usize] = s.on && s.output < 0.0;

        // Registres de maintien
        self.holdings[HR_MODE_SENS1 as usize] = s.mode_sens1.to_code();
        self.holdings[HR_MODE_SENS2 as usize] = s.mode_sens2.to_code();
        self.set_holding_f32(HR_SP_AUTO, s.sp_auto);
        self.set_holding_f32(HR_SP_MANUAL, s.sp_manual);
        self.set_holding_f32(HR_KP_SENS1, s.pid_heat.kp);
        self.set_holding_f32(HR_KI_SENS1, s.pid_heat.ki);
        self.set_holding_f32(HR_KD_SENS1, s.pid_heat.kd);
        self.set_holding_f32(HR_KP_SENS2, s.pid_cool.kp);
        self.set_holding_f32(HR_KI_SENS2, s.pid_cool.ki);
        self.set_holding_f32(HR_KD_SENS2, s.pid_cool.kd);
        self.set_holding_f32(HR_HYSTERESIS, s.hysteresis);
        self.set_holding_f32(HR_TOR_MIN_CYCLE, s.tor_min_cycle);
        self.set_holding_f32(HR_PWM_PERIOD, s.pwm_period);
        // Chaîne d'identification ASCII (constante, lecture seule).
        self.set_holding_ascii(HR_LABEL, LABEL_TEXT);

        // Registres d'entrée (mesures)
        self.set_input_f32(IR_PV, s.pv);
        self.set_input_f32(IR_OUTPUT, s.output);
    }
}

/// Traduit l'écriture d'une bobine en commande métier.
#[must_use]
pub fn coil_to_command(addr: u16, value: bool) -> Option<Command> {
    match addr {
        COIL_ON_OFF => Some(Command::SetOnOff(value)),
        COIL_AUTO_MANUAL => Some(Command::SetAutoManual(AutoManual::from_bool(value))),
        _ => None,
    }
}

/// Traduit l'écriture d'un bloc de registres de maintien en commandes métier.
///
/// `start` est l'adresse du premier registre écrit, `values` le bloc écrit.
/// Les `f32` ne sont décodés que si **leurs deux registres** sont présents dans le bloc.
#[must_use]
pub fn holdings_to_commands(start: u16, values: &[u16], current: &RegulatorSnapshot) -> Vec<Command> {
    let mut cmds = Vec::new();
    let end = start + values.len() as u16;
    let reg = |addr: u16| -> Option<u16> {
        if addr >= start && addr < end {
            Some(values[(addr - start) as usize])
        } else {
            None
        }
    };
    let float = |addr: u16| -> Option<f32> {
        match (reg(addr), reg(addr + 1)) {
            (Some(hi), Some(lo)) => Some(regs_to_f32(hi, lo)),
            _ => None,
        }
    };

    if let Some(v) = reg(HR_MODE_SENS1) {
        cmds.push(Command::SetModeSens1(ControllerKind::from_code(v)));
    }
    if let Some(v) = reg(HR_MODE_SENS2) {
        cmds.push(Command::SetModeSens2(ControllerKind::from_code(v)));
    }
    if let Some(v) = float(HR_SP_AUTO) {
        cmds.push(Command::SetSpAuto(v));
    }
    if let Some(v) = float(HR_SP_MANUAL) {
        cmds.push(Command::SetSpManual(v));
    }

    // Réglages PID : on part de la config courante et on remplace les gains
    // présents dans le bloc. Renvoie `Some` uniquement si au moins un gain a changé.
    let pid_from = |kp: u16, ki: u16, kd: u16, mut cfg: PidConfig| -> Option<PidConfig> {
        let mut changed = false;
        if let Some(v) = float(kp) {
            cfg.kp = v;
            changed = true;
        }
        if let Some(v) = float(ki) {
            cfg.ki = v;
            changed = true;
        }
        if let Some(v) = float(kd) {
            cfg.kd = v;
            changed = true;
        }
        changed.then(|| clamp_pid(cfg))
    };
    if let Some(pid) = pid_from(HR_KP_SENS1, HR_KI_SENS1, HR_KD_SENS1, current.pid_heat) {
        cmds.push(Command::SetPidHeat(pid));
    }
    if let Some(pid) = pid_from(HR_KP_SENS2, HR_KI_SENS2, HR_KD_SENS2, current.pid_cool) {
        cmds.push(Command::SetPidCool(pid));
    }

    if let Some(v) = float(HR_HYSTERESIS) {
        cmds.push(Command::SetHysteresis(v));
    }
    if let Some(v) = float(HR_TOR_MIN_CYCLE) {
        cmds.push(Command::SetTorMinCycle(v));
    }
    if let Some(v) = float(HR_PWM_PERIOD) {
        cmds.push(Command::SetPwmPeriod(v));
    }

    cmds
}

/// Borne les gains PID à des valeurs finies et positives (robustesse face aux écritures externes).
fn clamp_pid(mut cfg: PidConfig) -> PidConfig {
    let sane = |v: f32, default: f32| if v.is_finite() { v.max(0.0) } else { default };
    cfg.kp = sane(cfg.kp, 0.0);
    cfg.ki = sane(cfg.ki, 0.0);
    cfg.kd = sane(cfg.kd, 0.0);
    cfg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f32_round_trip() {
        let [hi, lo] = f32_to_regs(123.456);
        assert!((regs_to_f32(hi, lo) - 123.456).abs() < 1e-3);
    }

    #[test]
    fn decode_setpoint_write() {
        let snap = sample_snapshot();
        let regs = f32_to_regs(80.0);
        let cmds = holdings_to_commands(HR_SP_AUTO, &regs, &snap);
        assert_eq!(cmds, vec![Command::SetSpAuto(80.0)]);
    }

    #[test]
    fn label_ascii_is_written_at_register_42() {
        let mut map = MemoryMap::default();
        map.refresh_from(&sample_snapshot());
        // Décode les registres à partir de HR_LABEL en ASCII.
        let mut text = String::new();
        for i in 0..LABEL_TEXT.len().div_ceil(2) {
            let [hi, lo] = map.holdings[HR_LABEL as usize + i].to_be_bytes();
            text.push(hi as char);
            if lo != 0 {
                text.push(lo as char);
            }
        }
        assert_eq!(text, LABEL_TEXT);
        assert_eq!(HR_LABEL, 42);
    }

    #[test]
    fn decode_tor_and_pwm_writes() {
        let snap = sample_snapshot();
        // Écriture du temps de cycle minimal TOR.
        let cmds = holdings_to_commands(HR_TOR_MIN_CYCLE, &f32_to_regs(8.0), &snap);
        assert_eq!(cmds, vec![Command::SetTorMinCycle(8.0)]);
        // Écriture de la période PWM.
        let cmds = holdings_to_commands(HR_PWM_PERIOD, &f32_to_regs(12.5), &snap);
        assert_eq!(cmds, vec![Command::SetPwmPeriod(12.5)]);
    }

    #[test]
    fn new_holdings_round_trip_through_memory_map() {
        // refresh_from -> relecture : les nouveaux registres sont bien exposés.
        let mut snap = sample_snapshot();
        snap.tor_min_cycle = 7.0;
        snap.pwm_period = 15.0;
        let mut map = MemoryMap::default();
        map.refresh_from(&snap);
        let tor = regs_to_f32(
            map.holdings[HR_TOR_MIN_CYCLE as usize],
            map.holdings[HR_TOR_MIN_CYCLE as usize + 1],
        );
        let pwm = regs_to_f32(
            map.holdings[HR_PWM_PERIOD as usize],
            map.holdings[HR_PWM_PERIOD as usize + 1],
        );
        assert!((tor - 7.0).abs() < 1e-3);
        assert!((pwm - 15.0).abs() < 1e-3);
    }

    #[test]
    fn partial_float_block_is_ignored() {
        let snap = sample_snapshot();
        // Un seul registre de SP_AUTO écrit : pas assez pour un f32 -> aucune commande.
        let cmds = holdings_to_commands(HR_SP_AUTO, &[0x42a0], &snap);
        assert!(cmds.is_empty());
    }

    fn sample_snapshot() -> RegulatorSnapshot {
        RegulatorSnapshot {
            on: false,
            mode: AutoManual::Manual,
            mode_sens1: ControllerKind::Pid,
            mode_sens2: ControllerKind::Off,
            sp_auto: 50.0,
            sp_manual: 0.0,
            pv: 20.0,
            output: 0.0,
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
        }
    }
}
