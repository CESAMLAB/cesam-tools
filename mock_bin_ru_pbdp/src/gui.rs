//! Interface graphique (egui/eframe) du régulateur — page unique.
//!
//! Trois zones : panneau de commandes (gauche), supervision + courbe (centre),
//! blocs I/O PROFIBUS en direct (droite). Un modal `⚙ Paramètres` permet de
//! configurer la liaison série et la fonction de transfert. Les réglages sont
//! persistés dans un fichier TOML.
//!
//! ⚠️ Un bandeau permanent rappelle que ce simulateur ne respecte pas le timing du
//! bus PROFIBUS DP réel (voir `docs/fr/reference_profibus.md`).

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use eframe::egui;
use egui_plot::{Corner, Legend, Line, Plot, PlotPoints};
use ractor::ActorRef;

use mock_lib_control::{ControllerKind, PidConfig};
use mock_lib_update::UpdateStatus;

use crate::actors::{ProfibusServerMsg, SharedSnapshot, SharedStatus, SimulationMsg};
use crate::config::AppConfig;
use crate::i18n::{self, Lang, Msg};
use crate::regulator::{AutoManual, Command, RegulatorSnapshot};
use crate::trace::{Direction, SharedTrace};

const HISTORY_LEN: usize = 3000;
const LINK_ACTIVE_TIMEOUT: Duration = Duration::from_secs(3);

const UPDATE_REPO: &str = "CESAMLAB/cesam-tools";
const UPDATE_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Default)]
enum UpdateCheck {
    #[default]
    Idle,
    Checking,
    Done(Result<UpdateStatus, String>),
}

type SharedUpdate = Arc<Mutex<UpdateCheck>>;

const COLOR_SP: egui::Color32 = egui::Color32::from_rgb(90, 140, 255);
const COLOR_PV: egui::Color32 = egui::Color32::from_rgb(230, 80, 80);
const COLOR_OUT: egui::Color32 = egui::Color32::from_rgb(170, 200, 60);

struct Sample {
    t: f64,
    pv: f64,
    sp: f64,
    output: f64,
}

/// Application graphique du régulateur PROFIBUS DP.
pub struct RegulatorGui {
    sim: ActorRef<SimulationMsg>,
    net: ActorRef<ProfibusServerMsg>,
    snapshot: SharedSnapshot,
    status: SharedStatus,
    trace: SharedTrace,
    config: AppConfig,
    config_path: PathBuf,
    started: Instant,
    history: VecDeque<Sample>,
    show_settings: bool,
    settings_draft: AppConfig,
    feedback: Option<(String, bool)>,
    orpd_logo: Option<egui::TextureHandle>,
    cesam_logo: Option<egui::TextureHandle>,
    update: SharedUpdate,
    update_thread: Option<JoinHandle<()>>,
}

impl RegulatorGui {
    #[must_use]
    pub fn new(
        sim: ActorRef<SimulationMsg>,
        net: ActorRef<ProfibusServerMsg>,
        snapshot: SharedSnapshot,
        status: SharedStatus,
        trace: SharedTrace,
        config: AppConfig,
        config_path: PathBuf,
    ) -> Self {
        let check_at_startup = config.check_updates;
        let mut gui = Self {
            sim,
            net,
            snapshot,
            status,
            trace,
            settings_draft: config.clone(),
            config,
            config_path,
            started: Instant::now(),
            history: VecDeque::with_capacity(HISTORY_LEN),
            show_settings: false,
            feedback: None,
            orpd_logo: None,
            cesam_logo: None,
            update: Arc::new(Mutex::new(UpdateCheck::Idle)),
            update_thread: None,
        };
        if check_at_startup {
            gui.spawn_update_check();
        }
        gui
    }

    fn spawn_update_check(&mut self) {
        {
            let mut g = match self.update.lock() {
                Ok(g) => g,
                Err(_) => return,
            };
            if matches!(*g, UpdateCheck::Checking) {
                return;
            }
            *g = UpdateCheck::Checking;
        }
        let shared = self.update.clone();
        self.update_thread = Some(std::thread::spawn(move || {
            let res =
                mock_lib_update::check_blocking(UPDATE_REPO, env!("CARGO_PKG_VERSION"), UPDATE_TIMEOUT)
                    .map_err(|e| e.to_string());
            if let Ok(mut g) = shared.lock() {
                *g = UpdateCheck::Done(res);
            }
        }));
    }

    fn ensure_logos(&mut self, ctx: &egui::Context) {
        if self.orpd_logo.is_none() {
            self.orpd_logo = crate::branding::load_texture(ctx, "orpd-icon", crate::branding::ORPD_ICON_PNG);
        }
        if self.cesam_logo.is_none() {
            self.cesam_logo =
                crate::branding::load_texture(ctx, "cesam-logo", crate::branding::CESAM_LOGO_PNG);
        }
    }

    fn send(&self, cmd: Command) {
        let _ = self.sim.cast(SimulationMsg::Command(cmd));
    }

    fn save_config(&mut self) {
        let lang = self.config.language;
        match self.config.save(&self.config_path) {
            Ok(()) => {
                self.feedback = Some((
                    format!("{} ({})", i18n::tr(lang, Msg::SettingsSaved), self.config_path.display()),
                    true,
                ))
            }
            Err(e) => self.feedback = Some((format!("{} : {e}", i18n::tr(lang, Msg::SaveFailed)), false)),
        }
    }

    /// Applique une configuration complète : commandes simulation + liaison + sauvegarde.
    fn apply_settings(&mut self, cfg: AppConfig) {
        self.config = cfg.clone();
        self.send(Command::SetProcess {
            gain: cfg.process.gain,
            tau: cfg.process.tau,
            dead_time: cfg.process.dead_time,
            ambient: cfg.process.ambient,
        });
        self.send(Command::SetSpLimits {
            min: cfg.regulation.sp_min,
            max: cfg.regulation.sp_max,
        });
        self.send(Command::SetPidHeat(cfg.regulation.pid_heat));
        self.send(Command::SetPidCool(cfg.regulation.pid_cool));
        self.send(Command::SetHysteresis(cfg.regulation.hysteresis));
        let _ = self.net.cast(ProfibusServerMsg::Reconfigure(cfg.network.clone()));
        self.save_config();
    }
}

fn mode_label(lang: Lang, kind: ControllerKind) -> &'static str {
    i18n::tr(
        lang,
        match kind {
            ControllerKind::Off => Msg::ModeOff,
            ControllerKind::Pid => Msg::ModePid,
            ControllerKind::OnOff => Msg::ModeOnOff,
            ControllerKind::Pwm => Msg::ModePwm,
        },
    )
}

fn mode_combo(ui: &mut egui::Ui, id: &str, current: ControllerKind, lang: Lang) -> Option<ControllerKind> {
    let mut selected = current;
    egui::ComboBox::from_id_salt(id)
        .selected_text(mode_label(lang, selected))
        .show_ui(ui, |ui| {
            for kind in [
                ControllerKind::Off,
                ControllerKind::Pid,
                ControllerKind::OnOff,
                ControllerKind::Pwm,
            ] {
                ui.selectable_value(&mut selected, kind, mode_label(lang, kind));
            }
        });
    (selected != current).then_some(selected)
}

fn pid_editor(ui: &mut egui::Ui, id: &str, cfg: PidConfig) -> Option<PidConfig> {
    let mut edited = cfg;
    let mut changed = false;
    ui.push_id(id, |ui| {
        egui::Grid::new("grid").num_columns(2).show(ui, |ui| {
            for (label, field) in [("Kp", &mut edited.kp), ("Ki", &mut edited.ki), ("Kd", &mut edited.kd)] {
                ui.label(label);
                changed |= ui
                    .add(egui::DragValue::new(field).speed(0.02).range(0.0..=10_000.0))
                    .changed();
                ui.end_row();
            }
        });
    });
    changed.then_some(edited)
}

impl eframe::App for RegulatorGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ensure_logos(ctx);

        let snap = match self.snapshot.lock() {
            Ok(g) => *g,
            Err(_) => return,
        };

        let t = self.started.elapsed().as_secs_f64();
        self.history.push_back(Sample {
            t,
            pv: snap.pv as f64,
            sp: if snap.mode.is_auto() { snap.sp_auto as f64 } else { f64::NAN },
            output: snap.output as f64,
        });
        while self.history.len() > HISTORY_LEN {
            self.history.pop_front();
        }

        self.top_panel(ctx, &snap);
        self.frames_panel(ctx);
        self.left_panel(ctx, &snap);
        self.right_panel(ctx, &snap);
        self.central_panel(ctx, &snap);
        self.settings_window(ctx);

        ctx.request_repaint_after(Duration::from_millis(50));
    }
}

impl RegulatorGui {
    fn top_panel(&mut self, ctx: &egui::Context, snap: &RegulatorSnapshot) {
        let lang = self.config.language;
        let t = |k: Msg| i18n::tr(lang, k);
        egui::TopBottomPanel::top("entete").show(ctx, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                if let Some(logo) = &self.orpd_logo {
                    logo_image(ui, logo, 44.0).on_hover_text("ORPD — Open Regulator Profibus DP");
                }
                ui.add(egui::Label::new(
                    egui::RichText::new(format!("ORPD — {}", t(Msg::AppSubtitle))).size(22.0).strong(),
                ))
                .on_hover_text("Open Regulator Profibus DP — CESAM-Lab");

                ui.separator();
                if ui.button(format!("⚙ {}", t(Msg::SettingsBtn))).clicked() {
                    self.settings_draft = self.config.clone();
                    self.show_settings = true;
                }
                if ui.button(format!("💾 {}", t(Msg::SaveSettingsBtn))).clicked() {
                    self.save_config();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(logo) = &self.cesam_logo {
                        logo_image(ui, logo, 48.0).on_hover_text("CESAM-Lab");
                        ui.separator();
                    }
                    let (txt, color) = if snap.on {
                        (format!("● {}", t(Msg::DeviceRunning)), egui::Color32::from_rgb(0, 180, 0))
                    } else {
                        (format!("● {}", t(Msg::DeviceStopped)), egui::Color32::GRAY)
                    };
                    ui.colored_label(color, txt);
                    ui.separator();
                    if let Ok(st) = self.status.lock() {
                        if st.listening {
                            let state = st.state.as_deref().unwrap_or("?");
                            ui.colored_label(
                                egui::Color32::from_rgb(0, 150, 0),
                                format!("PROFIBUS ● {} [{state}]", st.addr),
                            );
                        } else if let Some(err) = &st.error {
                            ui.colored_label(egui::Color32::from_rgb(200, 60, 60), format!("PROFIBUS ✖ {err}"));
                        } else {
                            ui.colored_label(egui::Color32::GRAY, "PROFIBUS …");
                        }
                        if st.listening {
                            ui.separator();
                            let active = st.last_request.is_some_and(|ts| ts.elapsed() < LINK_ACTIVE_TIMEOUT);
                            let color = if active {
                                egui::Color32::from_rgb(0, 180, 0)
                            } else {
                                egui::Color32::GRAY
                            };
                            let hover = if active { t(Msg::LinkActive) } else { t(Msg::LinkIdle) };
                            ui.colored_label(color, "●").on_hover_text(hover);
                        }
                    }
                });
            });
            if let Some((msg, ok)) = &self.feedback {
                let color = if *ok {
                    egui::Color32::from_rgb(0, 150, 0)
                } else {
                    egui::Color32::from_rgb(200, 60, 60)
                };
                ui.colored_label(color, msg);
            }
            // Bandeau permanent : non-interopérabilité avec du matériel réel.
            ui.colored_label(egui::Color32::from_rgb(200, 140, 0), t(Msg::NonInterop));
            if let Ok(guard) = self.update.lock() {
                if let UpdateCheck::Done(Ok(UpdateStatus::Available(rel))) = &*guard {
                    ui.horizontal(|ui| {
                        ui.colored_label(
                            egui::Color32::from_rgb(0, 140, 200),
                            format!("{} v{}", t(Msg::UpdateAvailable), rel.version),
                        );
                        ui.hyperlink_to(t(Msg::UpdateDownload), &rel.url);
                    });
                }
            }
            ui.add_space(2.0);
        });
    }

    /// Mini-terminal en lecture seule : trames PROFIBUS reçues (RX) et émises (TX),
    /// journalisées en hexadécimal.
    fn frames_panel(&mut self, ctx: &egui::Context) {
        let lang = self.config.language;
        let t = |k: Msg| i18n::tr(lang, k);
        egui::TopBottomPanel::bottom("trames")
            .resizable(true)
            .default_height(160.0)
            .show(ctx, |ui| {
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("⇄ {}", t(Msg::FramesTitle))).strong());
                    if ui.button(format!("🗑 {}", t(Msg::ClearBtn))).clicked() {
                        if let Ok(mut tr) = self.trace.lock() {
                            tr.clear();
                        }
                    }
                });
                ui.separator();
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        if let Ok(trace) = self.trace.lock() {
                            for e in trace.iter() {
                                let secs = e.at.saturating_duration_since(self.started).as_secs_f32();
                                let (arrow, color) = match e.dir {
                                    Direction::Rx => ("← RX", egui::Color32::from_rgb(90, 170, 255)),
                                    Direction::Tx => ("→ TX", egui::Color32::from_rgb(120, 200, 120)),
                                };
                                ui.horizontal(|ui| {
                                    ui.monospace(egui::RichText::new(format!("{secs:8.1}s")).weak());
                                    ui.colored_label(color, egui::RichText::new(arrow).monospace());
                                    ui.monospace(e.text.as_str());
                                });
                            }
                        }
                    });
            });
    }

    fn left_panel(&mut self, ctx: &egui::Context, snap: &RegulatorSnapshot) {
        let lang = self.config.language;
        let t = |k: Msg| i18n::tr(lang, k);
        egui::SidePanel::left("commandes")
            .resizable(false)
            .default_width(290.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new(t(Msg::Commands)).strong());
                    ui.separator();

                    let mut on = snap.on;
                    if ui.toggle_value(&mut on, t(Msg::OnOff)).changed() {
                        self.send(Command::SetOnOff(on));
                    }

                    ui.horizontal(|ui| {
                        ui.label(t(Msg::ModeLabel));
                        if ui.selectable_label(!snap.mode.is_auto(), t(Msg::Manual)).clicked() {
                            self.send(Command::SetAutoManual(AutoManual::Manual));
                        }
                        if ui.selectable_label(snap.mode.is_auto(), t(Msg::Auto)).clicked() {
                            self.send(Command::SetAutoManual(AutoManual::Auto));
                        }
                    });

                    ui.separator();
                    ui.label(egui::RichText::new(t(Msg::RegModes)).strong());
                    ui.horizontal(|ui| {
                        ui.label(t(Msg::Sens1Hot));
                        if let Some(k) = mode_combo(ui, "mode_sens1", snap.mode_sens1, lang) {
                            self.send(Command::SetModeSens1(k));
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label(t(Msg::Sens2Cold));
                        if let Some(k) = mode_combo(ui, "mode_sens2", snap.mode_sens2, lang) {
                            self.send(Command::SetModeSens2(k));
                        }
                    });

                    ui.separator();
                    ui.label(egui::RichText::new(t(Msg::Setpoints)).strong());
                    if let Some(v) = setpoint_row(ui, t(Msg::SpAuto), snap.mode.is_auto(), snap.sp_auto, snap.sp_min, snap.sp_max, " u")
                    {
                        self.send(Command::SetSpAuto(v));
                    }
                    if let Some(v) = setpoint_row(ui, t(Msg::SpManual), !snap.mode.is_auto(), snap.sp_manual, -100.0, 100.0, " %")
                    {
                        self.send(Command::SetSpManual(v));
                    }

                    ui.separator();
                    ui.label(egui::RichText::new(t(Msg::PidSens1)).strong());
                    if let Some(cfg) = pid_editor(ui, "pid1", snap.pid_heat) {
                        self.config.regulation.pid_heat = cfg;
                        self.send(Command::SetPidHeat(cfg));
                    }
                    ui.label(egui::RichText::new(t(Msg::PidSens2)).strong());
                    if let Some(cfg) = pid_editor(ui, "pid2", snap.pid_cool) {
                        self.config.regulation.pid_cool = cfg;
                        self.send(Command::SetPidCool(cfg));
                    }

                    ui.separator();
                    ui.label(egui::RichText::new(t(Msg::TorPwmSettings)).strong());
                    let mut hyst = snap.hysteresis;
                    if ui.add(egui::Slider::new(&mut hyst, 0.0..=20.0).text(t(Msg::HystSlider))).changed() {
                        self.config.regulation.hysteresis = hyst;
                        self.send(Command::SetHysteresis(hyst));
                    }
                    let mut min_cycle = snap.tor_min_cycle;
                    if ui
                        .add(egui::Slider::new(&mut min_cycle, 0.0..=120.0).text(t(Msg::TorMinCycleSlider)))
                        .on_hover_text(t(Msg::HintAntiShortCycle))
                        .changed()
                    {
                        self.config.regulation.tor_min_cycle = min_cycle;
                        self.send(Command::SetTorMinCycle(min_cycle));
                    }
                    let mut pwm_period = snap.pwm_period;
                    if ui
                        .add(egui::Slider::new(&mut pwm_period, 0.5..=120.0).text(t(Msg::PwmPeriodSlider)))
                        .on_hover_text(t(Msg::HintCyclicRelay))
                        .changed()
                    {
                        self.config.regulation.pwm_period = pwm_period;
                        self.send(Command::SetPwmPeriod(pwm_period));
                    }
                });
            });
    }

    fn right_panel(&mut self, ctx: &egui::Context, snap: &RegulatorSnapshot) {
        let lang = self.config.language;
        let t = |k: Msg| i18n::tr(lang, k);
        egui::SidePanel::right("table_io")
            .resizable(true)
            .default_width(340.0)
            .show(ctx, |ui| {
                ui.add_space(6.0);
                ui.label(egui::RichText::new(t(Msg::IoTable)).strong());
                ui.label(egui::RichText::new(t(Msg::IoTableNote)).small().weak());
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("grille_io")
                        .num_columns(5)
                        .striped(true)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            for h in [Msg::ColName, Msg::ColBlock, Msg::ColOffset, Msg::ColValue, Msg::ColAccess] {
                                ui.label(egui::RichText::new(t(h)).strong().small());
                            }
                            ui.end_row();
                            for row in io_rows(snap, lang) {
                                ui.label(row.name);
                                ui.label(row.block);
                                ui.label(row.offset);
                                ui.label(row.value);
                                ui.label(row.access);
                                ui.end_row();
                            }
                        });
                });
            });
    }

    fn central_panel(&mut self, ctx: &egui::Context, snap: &RegulatorSnapshot) {
        let lang = self.config.language;
        let t = |k: Msg| i18n::tr(lang, k);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                value_card(ui, t(Msg::Measure), &format!("{:.2} u", snap.pv));
                let sp_txt = if snap.mode.is_auto() {
                    format!("{:.2} u", snap.sp_auto)
                } else {
                    format!("{:+.1} %", snap.sp_manual)
                };
                value_card(ui, t(Msg::ActiveSetpoint), &sp_txt);
                value_card(ui, t(Msg::Output), &format!("{:+.1} %", snap.output));
            });
            ui.add_space(8.0);

            let sp_txt = if snap.mode.is_auto() {
                format!("{:.2} u", snap.sp_auto)
            } else {
                t(Msg::ManualDash).to_string()
            };
            Plot::new("tendance")
                .legend(Legend::default().position(Corner::LeftTop))
                .height(ui.available_height() - 10.0)
                .x_axis_label(t(Msg::AxisTime))
                .show(ui, |plot_ui| {
                    let sp: PlotPoints = self.history.iter().map(|s| [s.t, s.sp]).collect();
                    let pv: PlotPoints = self.history.iter().map(|s| [s.t, s.pv]).collect();
                    let out: PlotPoints = self.history.iter().map(|s| [s.t, s.output]).collect();
                    plot_ui.line(Line::new(format!("{}   {sp_txt}", t(Msg::LegSetpoint)), sp).color(COLOR_SP));
                    plot_ui.line(
                        Line::new(format!("{}   {:.2} u", t(Msg::LegMeasure), snap.pv), pv).color(COLOR_PV),
                    );
                    plot_ui.line(
                        Line::new(format!("{}   {:+.1} %", t(Msg::LegOutput), snap.output), out).color(COLOR_OUT),
                    );
                });
        });
    }

    fn settings_window(&mut self, ctx: &egui::Context) {
        if !self.show_settings {
            return;
        }
        let mut open = true;
        let mut do_apply = false;
        let mut do_reset = false;
        let mut do_close = false;
        let mut do_check_now = false;
        let lang = self.settings_draft.language;
        let t = |k: Msg| i18n::tr(lang, k);
        let update_label: Option<(String, egui::Color32)> = match self.update.lock() {
            Ok(g) => match &*g {
                UpdateCheck::Idle => None,
                UpdateCheck::Checking => Some(("⏳".to_string(), egui::Color32::GRAY)),
                UpdateCheck::Done(Ok(UpdateStatus::UpToDate)) => {
                    Some((t(Msg::UpToDate).to_string(), egui::Color32::from_rgb(0, 150, 0)))
                }
                UpdateCheck::Done(Ok(UpdateStatus::Available(rel))) => Some((
                    format!("{} v{}", t(Msg::UpdateAvailable), rel.version),
                    egui::Color32::from_rgb(0, 140, 200),
                )),
                UpdateCheck::Done(Err(_)) => {
                    Some((t(Msg::UpdateCheckFailed).to_string(), egui::Color32::from_rgb(200, 140, 0)))
                }
            },
            Err(_) => None,
        };
        {
            let draft = &mut self.settings_draft;
            egui::Window::new(format!("⚙ {}", t(Msg::SettingsTitle)))
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .default_width(380.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(t(Msg::Language)).strong());
                        egui::ComboBox::from_id_salt("langue")
                            .selected_text(draft.language.native_name())
                            .show_ui(ui, |ui| {
                                for l in Lang::ALL {
                                    ui.selectable_value(&mut draft.language, l, l.native_name());
                                }
                            });
                    });
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.checkbox(&mut draft.check_updates, t(Msg::CheckUpdates));
                    });
                    ui.horizontal(|ui| {
                        if ui.button(t(Msg::CheckNow)).clicked() {
                            do_check_now = true;
                        }
                        if let Some((txt, color)) = &update_label {
                            ui.colored_label(*color, txt);
                        }
                    });
                    ui.add_space(6.0);

                    ui.label(egui::RichText::new(t(Msg::NonInterop)).small().weak());
                    ui.add_space(6.0);

                    let s = &mut draft.network.serial;
                    egui::Grid::new("serial").num_columns(2).show(ui, |ui| {
                        ui.label(t(Msg::SerialPort));
                        ui.text_edit_singleline(&mut s.port).on_hover_text("/dev/ttyUSB0, COM3…");
                        ui.end_row();
                        ui.label(t(Msg::Baud));
                        ui.add(egui::DragValue::new(&mut s.baud).range(9600..=12_000_000).speed(1000));
                        ui.end_row();
                        ui.label(t(Msg::StationAddress));
                        ui.add(egui::DragValue::new(&mut s.station_address).range(0..=125));
                        ui.end_row();
                    });
                    ui.checkbox(&mut s.watchdog_enabled, t(Msg::WatchdogEnabled));

                    ui.add_space(6.0);
                    ui.label(egui::RichText::new(t(Msg::ProcessTf)).strong());
                    ui.label(egui::RichText::new("G(s) = K·e^(-L·s) / (1 + T·s)").small().weak());
                    egui::Grid::new("proc").num_columns(2).show(ui, |ui| {
                        ui.label(t(Msg::GainK));
                        ui.add(egui::DragValue::new(&mut draft.process.gain).speed(0.01));
                        ui.end_row();
                        ui.label(t(Msg::ConstT));
                        ui.add(egui::DragValue::new(&mut draft.process.tau).speed(0.1).range(0.001..=100_000.0));
                        ui.end_row();
                        ui.label(t(Msg::DelayL));
                        ui.add(egui::DragValue::new(&mut draft.process.dead_time).speed(0.1).range(0.0..=100_000.0));
                        ui.end_row();
                        ui.label(t(Msg::Ambient));
                        ui.add(egui::DragValue::new(&mut draft.process.ambient).speed(0.1));
                        ui.end_row();
                    });

                    ui.add_space(6.0);
                    ui.label(egui::RichText::new(t(Msg::SpBounds)).strong());
                    egui::Grid::new("sp").num_columns(2).show(ui, |ui| {
                        ui.label(t(Msg::SpMin));
                        ui.add(egui::DragValue::new(&mut draft.regulation.sp_min).speed(0.5));
                        ui.end_row();
                        ui.label(t(Msg::SpMax));
                        ui.add(egui::DragValue::new(&mut draft.regulation.sp_max).speed(0.5));
                        ui.end_row();
                    });

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button(t(Msg::ApplyBtn)).clicked() {
                            do_apply = true;
                        }
                        if ui.button(t(Msg::ResetBtn)).clicked() {
                            do_reset = true;
                        }
                        if ui.button(t(Msg::CloseBtn)).clicked() {
                            do_close = true;
                        }
                    });
                });
        }

        if do_check_now {
            self.spawn_update_check();
        }
        if do_close {
            open = false;
        }
        if do_reset {
            self.settings_draft = AppConfig::default();
            let cfg = self.settings_draft.clone();
            self.apply_settings(cfg);
        } else if do_apply {
            let cfg = self.settings_draft.clone();
            self.apply_settings(cfg);
            open = false;
        }
        self.show_settings = open;
    }
}

fn logo_image(ui: &mut egui::Ui, tex: &egui::TextureHandle, height: f32) -> egui::Response {
    let size = tex.size_vec2();
    let width = if size.y > 0.0 { height * size.x / size.y } else { height };
    ui.add(egui::Image::new(tex).fit_to_exact_size(egui::vec2(width, height)))
}

fn value_card(ui: &mut egui::Ui, title: &str, value: &str) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new(title).small().weak());
            ui.label(egui::RichText::new(value).heading());
        });
    });
}

#[allow(clippy::too_many_arguments)]
fn setpoint_row(ui: &mut egui::Ui, label: &str, active: bool, current: f32, min: f32, max: f32, suffix: &str) -> Option<f32> {
    let mut value = current;
    let mut changed = false;
    ui.push_id(label, |ui| {
        ui.horizontal(|ui| {
            let title = egui::RichText::new(label);
            let title = if active { title.strong() } else { title.weak() };
            ui.label(title);
            changed |= ui.add(egui::DragValue::new(&mut value).range(min..=max).suffix(suffix)).changed();
        });
        changed |= ui
            .add(egui::Slider::new(&mut value, min..=max).suffix(suffix).show_value(false))
            .changed();
    });
    changed.then_some(value)
}

/// Une ligne de la table des blocs I/O PROFIBUS affichée dans l'IHM.
struct IoRow {
    name: String,
    block: &'static str,
    offset: String,
    value: String,
    access: &'static str,
}

fn io_rows(s: &RegulatorSnapshot, lang: Lang) -> Vec<IoRow> {
    let onoff = |b: bool| if b { "1" } else { "0" }.to_string();
    let kind = |k: ControllerKind| format!("{} ({})", k.to_code(), mode_label(lang, k));
    let t = |k: Msg| i18n::tr(lang, k);
    let gain = |sym: &str, dir: Msg| format!("{sym} {}", t(dir));
    vec![
        IoRow { name: t(Msg::RowRunning).to_string(), block: "Out/In", offset: "0".to_string(), value: onoff(s.on), access: "R/W" },
        IoRow { name: t(Msg::RowHeatingActive).to_string(), block: "In", offset: "0".to_string(), value: onoff(s.on && s.output > 0.0), access: "R" },
        IoRow { name: t(Msg::RowCoolingActive).to_string(), block: "In", offset: "0".to_string(), value: onoff(s.on && s.output < 0.0), access: "R" },
        IoRow { name: t(Msg::RowModeSens1).to_string(), block: "Out", offset: "0".to_string(), value: kind(s.mode_sens1), access: "R/W" },
        IoRow { name: t(Msg::RowModeSens2).to_string(), block: "Out", offset: "0".to_string(), value: kind(s.mode_sens2), access: "R/W" },
        IoRow { name: t(Msg::SpAuto).to_string(), block: "Out", offset: format!("{}-{}", 1, 4), value: format!("{:.2}", s.sp_auto), access: "R/W" },
        IoRow { name: format!("{} (%)", t(Msg::SpManual)), block: "Out", offset: format!("{}-{}", 5, 8), value: format!("{:.2}", s.sp_manual), access: "R/W" },
        IoRow { name: gain("Kp", Msg::Dir1), block: "Out", offset: format!("{}-{}", 9, 12), value: format!("{:.3}", s.pid_heat.kp), access: "R/W" },
        IoRow { name: gain("Ki", Msg::Dir1), block: "Out", offset: format!("{}-{}", 13, 16), value: format!("{:.3}", s.pid_heat.ki), access: "R/W" },
        IoRow { name: gain("Kd", Msg::Dir1), block: "Out", offset: format!("{}-{}", 17, 20), value: format!("{:.3}", s.pid_heat.kd), access: "R/W" },
        IoRow { name: gain("Kp", Msg::Dir2), block: "Out", offset: format!("{}-{}", 21, 24), value: format!("{:.3}", s.pid_cool.kp), access: "R/W" },
        IoRow { name: gain("Ki", Msg::Dir2), block: "Out", offset: format!("{}-{}", 25, 28), value: format!("{:.3}", s.pid_cool.ki), access: "R/W" },
        IoRow { name: gain("Kd", Msg::Dir2), block: "Out", offset: format!("{}-{}", 29, 32), value: format!("{:.3}", s.pid_cool.kd), access: "R/W" },
        IoRow { name: t(Msg::RowHysteresis).to_string(), block: "Out", offset: format!("{}-{}", 33, 36), value: format!("{:.2}", s.hysteresis), access: "R/W" },
        IoRow { name: t(Msg::TorMinCycleSlider).to_string(), block: "Out", offset: format!("{}-{}", 37, 40), value: format!("{:.2}", s.tor_min_cycle), access: "R/W" },
        IoRow { name: t(Msg::PwmPeriodSlider).to_string(), block: "Out", offset: format!("{}-{}", 41, 44), value: format!("{:.2}", s.pwm_period), access: "R/W" },
        IoRow { name: t(Msg::Measure).to_string(), block: "In", offset: format!("{}-{}", 1, 4), value: format!("{:.2}", s.pv), access: "R" },
        IoRow { name: t(Msg::OutputPct).to_string(), block: "In", offset: format!("{}-{}", 5, 8), value: format!("{:+.2}", s.output), access: "R" },
        IoRow { name: t(Msg::SpAuto).to_string(), block: "In", offset: format!("{}-{}", 9, 12), value: format!("{:.2}", s.sp_auto), access: "R" },
        IoRow { name: t(Msg::SpManual).to_string(), block: "In", offset: format!("{}-{}", 13, 16), value: format!("{:.2}", s.sp_manual), access: "R" },
    ]
}
