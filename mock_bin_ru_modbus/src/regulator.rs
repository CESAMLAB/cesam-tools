//! Modèle métier du régulateur : état, configuration, commandes et pas de simulation.
//!
//! Ce module est **synchrone et sans IO**. Il est piloté par l'acteur de
//! simulation (voir [`crate::actors`]) qui appelle [`Regulator::step`] à intervalle
//! régulier et applique les [`Command`] provenant de l'IHM ou du serveur Modbus.

use mock_lib_control::{ControllerKind, FirstOrderProcess, OnOff, Pid, PidConfig, Pwm};

/// Pas d'échantillonnage par défaut de la boucle de simulation (secondes).
///
/// C'est aussi la période du « tick » de l'acteur de simulation. 50 ms offre un
/// bon compromis fluidité d'affichage / charge CPU pour des procédés thermiques
/// dont les constantes de temps se comptent en dizaines de secondes.
pub const DEFAULT_DT: f32 = 0.05;

/// Mode de fonctionnement automatique / manuel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AutoManual {
    /// Mode manuel : la sortie suit directement la consigne manuelle (en %).
    #[default]
    Manual,
    /// Mode automatique : la sortie est calculée par les régulateurs.
    Auto,
}

impl AutoManual {
    /// `false` = manuel, `true` = auto (codage de la bobine Modbus).
    #[must_use]
    pub const fn is_auto(self) -> bool {
        matches!(self, AutoManual::Auto)
    }

    /// Construit depuis le booléen de la bobine Modbus.
    #[must_use]
    pub const fn from_bool(auto: bool) -> Self {
        if auto {
            AutoManual::Auto
        } else {
            AutoManual::Manual
        }
    }
}

/// Configuration statique du régulateur (procédé + réglages).
#[derive(Debug, Clone)]
pub struct RegulatorConfig {
    /// Période de simulation en secondes (pas d'intégration).
    pub dt: f32,
    /// Bornes physiques de la consigne automatique (unité de mesure).
    pub sp_min: f32,
    pub sp_max: f32,
    /// Réglage PID du sens 1 (chaud).
    pub pid_heat: PidConfig,
    /// Réglage PID du sens 2 (froid).
    pub pid_cool: PidConfig,
    /// Hystérésis commune des régulateurs TOR (unité de mesure).
    pub hysteresis: f32,
    /// Temps de cycle minimal des régulateurs TOR (s) : anti-court-cycle.
    pub tor_min_cycle: f32,
    /// Période du cycle de modulation PWM / relais à cycle (s).
    pub pwm_period: f32,
    /// Gain statique du procédé (unité de mesure par %).
    pub process_gain: f32,
    /// Constante de temps du procédé (s).
    pub process_tau: f32,
    /// Retard pur du procédé (s).
    pub process_dead_time: f32,
    /// Valeur ambiante / de repos du procédé.
    pub ambient: f32,
}

impl Default for RegulatorConfig {
    fn default() -> Self {
        // Valeurs par défaut représentatives d'un four / bain thermostaté.
        let pid_heat = PidConfig {
            kp: 4.0,
            ki: 0.25,
            kd: 1.0,
            out_min: 0.0,
            out_max: 100.0,
        };
        let pid_cool = PidConfig {
            kp: 4.0,
            ki: 0.25,
            kd: 1.0,
            out_min: 0.0,
            out_max: 100.0,
        };
        Self {
            dt: DEFAULT_DT,
            sp_min: 0.0,
            sp_max: 250.0,
            pid_heat,
            pid_cool,
            hysteresis: 2.0,
            tor_min_cycle: 5.0,
            pwm_period: 10.0,
            process_gain: 1.6,
            process_tau: 30.0,
            process_dead_time: 2.0,
            ambient: 20.0,
        }
    }
}

/// Commande appliquée au régulateur (depuis l'IHM ou le serveur Modbus).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    /// Marche / arrêt de l'appareil.
    SetOnOff(bool),
    /// Bascule auto / manuel.
    SetAutoManual(AutoManual),
    /// Mode de régulation du sens 1 (chaud).
    SetModeSens1(ControllerKind),
    /// Mode de régulation du sens 2 (froid).
    SetModeSens2(ControllerKind),
    /// Consigne automatique (unité de mesure).
    SetSpAuto(f32),
    /// Consigne manuelle (% de sortie, signée : + chaud / − froid).
    SetSpManual(f32),
    /// Réglage PID du sens 1.
    SetPidHeat(PidConfig),
    /// Réglage PID du sens 2.
    SetPidCool(PidConfig),
    /// Hystérésis des régulateurs TOR.
    SetHysteresis(f32),
    /// Temps de cycle minimal des régulateurs TOR (s).
    SetTorMinCycle(f32),
    /// Période du cycle de modulation PWM (s).
    SetPwmPeriod(f32),
    /// Paramètres de la fonction de transfert du procédé `(gain, tau, dead_time, ambient)`.
    SetProcess {
        gain: f32,
        tau: f32,
        dead_time: f32,
        ambient: f32,
    },
    /// Bornes de la consigne automatique `(min, max)`.
    SetSpLimits { min: f32, max: f32 },
}

/// Image instantanée de l'état du régulateur, partagée avec l'IHM et le serveur Modbus.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RegulatorSnapshot {
    pub on: bool,
    pub mode: AutoManual,
    pub mode_sens1: ControllerKind,
    pub mode_sens2: ControllerKind,
    /// Consigne automatique (unité de mesure).
    pub sp_auto: f32,
    /// Consigne manuelle (% sortie signé).
    pub sp_manual: f32,
    /// Mesure (process value).
    pub pv: f32,
    /// Sortie appliquée (% signé : + chaud / − froid).
    pub output: f32,
    pub pid_heat: PidConfig,
    pub pid_cool: PidConfig,
    pub hysteresis: f32,
    /// Temps de cycle minimal des régulateurs TOR (s).
    pub tor_min_cycle: f32,
    /// Période du cycle de modulation PWM (s).
    pub pwm_period: f32,
    /// Bornes de la consigne automatique.
    pub sp_min: f32,
    pub sp_max: f32,
    /// Paramètres de la fonction de transfert du procédé.
    pub process_gain: f32,
    pub process_tau: f32,
    pub process_dead_time: f32,
    pub ambient: f32,
}

/// Sens d'action d'un régulateur, pour aiguiller vers le bon jeu d'organes
/// (chaud = sens 1, froid = sens 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Heat,
    Cool,
}

/// Magnitude de sortie d'un organe tout-ou-rien : 100 % si actif, 0 % sinon.
fn on_off_output(on: bool) -> f32 {
    if on {
        100.0
    } else {
        0.0
    }
}

/// Régulateur simulé complet : logique de régulation + procédé.
pub struct Regulator {
    cfg: RegulatorConfig,
    on: bool,
    mode: AutoManual,
    mode_sens1: ControllerKind,
    mode_sens2: ControllerKind,
    sp_auto: f32,
    sp_manual: f32,
    output: f32,
    pid_heat: Pid,
    pid_cool: Pid,
    onoff_heat: OnOff,
    onoff_cool: OnOff,
    pwm_heat: Pwm,
    pwm_cool: Pwm,
    process: FirstOrderProcess,
}

impl Regulator {
    /// Construit un régulateur à partir de sa configuration.
    #[must_use]
    pub fn new(cfg: RegulatorConfig) -> Self {
        let process = FirstOrderProcess::new(
            cfg.process_gain,
            cfg.process_tau,
            cfg.process_dead_time,
            cfg.ambient,
        );
        let sp_auto = ((cfg.sp_min + cfg.sp_max) / 2.0).clamp(cfg.sp_min, cfg.sp_max);
        Self {
            on: false,
            mode: AutoManual::default(),
            mode_sens1: ControllerKind::Pid,
            mode_sens2: ControllerKind::Off,
            sp_auto,
            sp_manual: 0.0,
            output: 0.0,
            pid_heat: Pid::new(cfg.pid_heat),
            pid_cool: Pid::new(cfg.pid_cool),
            onoff_heat: OnOff::with_min_cycle(cfg.hysteresis, cfg.tor_min_cycle),
            onoff_cool: OnOff::with_min_cycle(cfg.hysteresis, cfg.tor_min_cycle),
            pwm_heat: Pwm::new(cfg.pwm_period),
            pwm_cool: Pwm::new(cfg.pwm_period),
            process,
            cfg,
        }
    }

    /// Période d'échantillonnage configurée (s).
    #[must_use]
    pub fn dt(&self) -> f32 {
        self.cfg.dt
    }

    /// Applique une commande externe.
    pub fn apply(&mut self, cmd: Command) {
        match cmd {
            Command::SetOnOff(on) => {
                self.on = on;
                if !on {
                    self.reset_controllers();
                }
            }
            Command::SetAutoManual(mode) => {
                self.mode = mode;
                self.reset_controllers();
            }
            Command::SetModeSens1(kind) => self.mode_sens1 = kind,
            Command::SetModeSens2(kind) => self.mode_sens2 = kind,
            Command::SetSpAuto(sp) => {
                self.sp_auto = sp.clamp(self.cfg.sp_min, self.cfg.sp_max);
            }
            Command::SetSpManual(pct) => self.sp_manual = pct.clamp(-100.0, 100.0),
            Command::SetPidHeat(pid) => {
                self.cfg.pid_heat = pid;
                self.pid_heat.set_config(pid);
            }
            Command::SetPidCool(pid) => {
                self.cfg.pid_cool = pid;
                self.pid_cool.set_config(pid);
            }
            Command::SetHysteresis(h) => {
                self.cfg.hysteresis = h.max(0.0);
                self.onoff_heat.set_hysteresis(h);
                self.onoff_cool.set_hysteresis(h);
            }
            Command::SetTorMinCycle(c) => {
                self.cfg.tor_min_cycle = c.max(0.0);
                self.onoff_heat.set_min_cycle(c);
                self.onoff_cool.set_min_cycle(c);
            }
            Command::SetPwmPeriod(p) => {
                self.cfg.pwm_period = p.max(1e-3);
                self.pwm_heat.set_period(p);
                self.pwm_cool.set_period(p);
            }
            Command::SetProcess {
                gain,
                tau,
                dead_time,
                ambient,
            } => {
                self.cfg.process_gain = gain;
                self.cfg.process_tau = tau;
                self.cfg.process_dead_time = dead_time;
                self.cfg.ambient = ambient;
                self.process.reconfigure(gain, tau, dead_time, ambient);
            }
            Command::SetSpLimits { min, max } => {
                // On garantit min <= max.
                let (min, max) = if min <= max { (min, max) } else { (max, min) };
                self.cfg.sp_min = min;
                self.cfg.sp_max = max;
                self.sp_auto = self.sp_auto.clamp(min, max);
            }
        }
    }

    /// Calcule la sortie (magnitude ≥ 0, en %) d'un sens donné selon son mode.
    ///
    /// * `error` — erreur orientée pour ce sens (positive = il faut agir).
    /// * `dir` — aiguille vers les organes chaud ou froid.
    ///
    /// Les deux sens sont évalués à chaque pas sur leur erreur signée : le sens
    /// inactif reçoit une erreur négative, et son anti-emballement par bornage
    /// (`out_min = 0` pour le PID, bande disjointe pour le relais) garantit une
    /// sortie nulle sans qu'il soit nécessaire de le réinitialiser de force. La
    /// purge naturelle du terme intégral est ce qui évite l'erreur statique du
    /// PWM (l'effacer à chaque dépassement de consigne biaiserait la régulation).
    fn sens_output(&mut self, kind: ControllerKind, error: f32, dt: f32, dir: Direction) -> f32 {
        match kind {
            ControllerKind::Off => 0.0,
            // Sortie déjà bornée à [0, 100] (out_min = 0) : négative => 0.
            ControllerKind::Pid => self.pid_step(dir, error, dt),
            ControllerKind::OnOff => {
                let on = match dir {
                    Direction::Heat => self.onoff_heat.update(error, dt),
                    Direction::Cool => self.onoff_cool.update(error, dt),
                };
                on_off_output(on)
            }
            ControllerKind::Pwm => {
                // Le PID fournit le rapport cyclique (0..100 %), modulé par le relais à cycle.
                let duty = self.pid_step(dir, error, dt);
                let on = match dir {
                    Direction::Heat => self.pwm_heat.update(duty / 100.0, dt),
                    Direction::Cool => self.pwm_cool.update(duty / 100.0, dt),
                };
                on_off_output(on)
            }
        }
    }

    /// Pas du PID du sens `dir` sur l'erreur orientée (sortie bornée à `[0, 100]`).
    fn pid_step(&mut self, dir: Direction, error: f32, dt: f32) -> f32 {
        match dir {
            Direction::Heat => self.pid_heat.step_with_error(error, dt),
            Direction::Cool => self.pid_cool.step_with_error(error, dt),
        }
    }

    fn reset_controllers(&mut self) {
        self.pid_heat.reset();
        self.pid_cool.reset();
        self.onoff_heat.reset();
        self.onoff_cool.reset();
        self.pwm_heat.reset();
        self.pwm_cool.reset();
    }

    /// Avance la simulation d'un pas et met à jour la mesure.
    pub fn step(&mut self) {
        let dt = self.cfg.dt;
        let pv = self.process.value();

        self.output = if !self.on {
            self.reset_controllers();
            0.0
        } else if self.mode == AutoManual::Manual {
            // En manuel, la sortie suit directement la consigne manuelle.
            self.sp_manual
        } else {
            // En automatique, le sens chaud agit lorsque la mesure est sous la
            // consigne (erreur ≥ 0), le sens froid au-dessus. Les sorties des deux
            // sens sont calculées séparément puis combinées (chaud − froid).
            //
            // ⚠️ Les régulateurs TOR/PWM sont évalués sur l'**erreur signée** à
            // chaque pas, sans être réinitialisés au changement de signe : c'est
            // ce qui préserve leur hystérésis symétrique autour de la consigne.
            // Les deux relais restent malgré tout mutuellement exclusifs (leurs
            // bandes d'activation sont disjointes).
            let error = self.sp_auto - pv;
            let heat = self.sens_output(self.mode_sens1, error, dt, Direction::Heat);
            let cool = self.sens_output(self.mode_sens2, -error, dt, Direction::Cool);
            heat - cool
        };

        self.process.step(self.output, dt);
    }

    /// Avance la simulation de `seconds` secondes (utilitaire de test/scénario).
    #[cfg(test)]
    fn run_for(&mut self, seconds: f32) {
        let steps = (seconds / self.cfg.dt).round() as usize;
        for _ in 0..steps {
            self.step();
        }
    }

    /// Image instantanée de l'état courant.
    #[must_use]
    pub fn snapshot(&self) -> RegulatorSnapshot {
        RegulatorSnapshot {
            on: self.on,
            mode: self.mode,
            mode_sens1: self.mode_sens1,
            mode_sens2: self.mode_sens2,
            sp_auto: self.sp_auto,
            sp_manual: self.sp_manual,
            pv: self.process.value(),
            output: self.output,
            pid_heat: self.cfg.pid_heat,
            pid_cool: self.cfg.pid_cool,
            hysteresis: self.cfg.hysteresis,
            tor_min_cycle: self.cfg.tor_min_cycle,
            pwm_period: self.cfg.pwm_period,
            sp_min: self.cfg.sp_min,
            sp_max: self.cfg.sp_max,
            process_gain: self.cfg.process_gain,
            process_tau: self.cfg.process_tau,
            process_dead_time: self.cfg.process_dead_time,
            ambient: self.cfg.ambient,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pid_auto_converges_to_setpoint() {
        let mut reg = Regulator::new(RegulatorConfig::default());
        reg.apply(Command::SetModeSens1(ControllerKind::Pid));
        reg.apply(Command::SetAutoManual(AutoManual::Auto));
        reg.apply(Command::SetSpAuto(120.0));
        reg.apply(Command::SetOnOff(true));
        reg.run_for(600.0);
        let s = reg.snapshot();
        assert!(
            (s.pv - 120.0).abs() < 2.0,
            "PV={} doit converger vers la consigne 120",
            s.pv
        );
    }

    #[test]
    fn onoff_auto_oscillates_around_setpoint() {
        // Hystérésis pure (sans anti-court-cycle) pour observer la bande seule.
        let cfg = RegulatorConfig {
            hysteresis: 4.0,
            tor_min_cycle: 0.0,
            ..RegulatorConfig::default()
        };
        let mut reg = Regulator::new(cfg);
        reg.apply(Command::SetModeSens1(ControllerKind::OnOff));
        reg.apply(Command::SetAutoManual(AutoManual::Auto));
        reg.apply(Command::SetSpAuto(120.0));
        reg.apply(Command::SetOnOff(true));
        reg.run_for(400.0); // atteindre le cycle limite

        let mut min = f32::MAX;
        let mut max = f32::MIN;
        let steps = (200.0 / reg.dt()).round() as usize;
        for _ in 0..steps {
            reg.step();
            let pv = reg.snapshot().pv;
            min = min.min(pv);
            max = max.max(pv);
        }
        // La mesure oscille DE PART ET D'AUTRE de la consigne : preuve que le
        // relais ne coupe pas pile à la consigne (hystérésis symétrique restaurée).
        assert!(max > 120.0, "PV max={max} doit dépasser la consigne");
        assert!(min < 120.0, "PV min={min} doit passer sous la consigne");
        let mid = (min + max) / 2.0;
        assert!((mid - 120.0).abs() < 6.0, "centre du cycle {mid} doit cadrer la consigne 120");
    }

    #[test]
    fn pwm_auto_converges_near_setpoint() {
        let mut reg = Regulator::new(RegulatorConfig::default());
        reg.apply(Command::SetModeSens1(ControllerKind::Pwm));
        reg.apply(Command::SetAutoManual(AutoManual::Auto));
        reg.apply(Command::SetSpAuto(120.0));
        reg.apply(Command::SetOnOff(true));
        reg.run_for(800.0);

        // Sortie physique strictement tout-ou-rien, mais moyenne de PV ~ consigne.
        let mut sum = 0.0;
        let steps = (200.0 / reg.dt()).round() as usize;
        for _ in 0..steps {
            reg.step();
            let s = reg.snapshot();
            assert!(s.output == 0.0 || s.output == 100.0, "sortie PWM non binaire : {}", s.output);
            sum += s.pv;
        }
        let mean = sum / steps as f32;
        assert!((mean - 120.0).abs() < 5.0, "PV moyenne PWM = {mean} doit cadrer la consigne 120");
    }

    #[test]
    fn manual_output_drives_process_open_loop() {
        let mut reg = Regulator::new(RegulatorConfig::default());
        reg.apply(Command::SetAutoManual(AutoManual::Manual));
        reg.apply(Command::SetSpManual(50.0));
        reg.apply(Command::SetOnOff(true));
        reg.run_for(400.0);
        let s = reg.snapshot();
        // 50 % de sortie => régime établi ~ ambient + gain*50 = 20 + 1.6*50 = 100.
        assert!((s.pv - 100.0).abs() < 3.0, "PV={}", s.pv);
        assert!((s.output - 50.0).abs() < 1e-3);
    }

    #[test]
    fn off_device_relaxes_to_ambient() {
        let mut reg = Regulator::new(RegulatorConfig::default());
        reg.apply(Command::SetAutoManual(AutoManual::Manual));
        reg.apply(Command::SetSpManual(80.0));
        reg.apply(Command::SetOnOff(true));
        reg.run_for(200.0);
        reg.apply(Command::SetOnOff(false));
        reg.run_for(400.0);
        let s = reg.snapshot();
        assert_eq!(s.output, 0.0);
        assert!((s.pv - 20.0).abs() < 2.0, "PV={} doit revenir à l'ambiant", s.pv);
    }

    #[test]
    fn cooling_engages_only_when_pv_above_setpoint() {
        let mut reg = Regulator::new(RegulatorConfig::default());
        reg.apply(Command::SetModeSens2(ControllerKind::Pid));
        reg.apply(Command::SetAutoManual(AutoManual::Auto));
        reg.apply(Command::SetOnOff(true));

        // Consigne AU-DESSUS de la mesure (ambiant) : c'est le chaud qui agit,
        // le froid reste inactif (output >= 0) même si le sens 2 est en PID.
        reg.apply(Command::SetSpAuto(120.0));
        reg.step();
        assert!(reg.snapshot().output >= 0.0, "pas de refroidissement si PV < consigne");

        // Consigne SOUS la mesure : le froid s'active, sortie négative.
        reg.apply(Command::SetSpAuto(5.0));
        reg.step();
        assert!(
            reg.snapshot().output < 0.0,
            "le sens froid doit débiter (sortie < 0) quand PV > consigne"
        );
    }
}
