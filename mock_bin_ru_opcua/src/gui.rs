//! Interface graphique (egui/eframe) du régulateur OPC UA — page unique.
//!
//! L'IHM **lit** des copies partagées ([`SharedSnapshot`], [`SharedStatus`]) et
//! **écrit** uniquement via des `cast` non bloquants vers les acteurs.

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;
use std::time::Instant;

use eframe::egui;
use egui_plot::{AxisHints, Corner, HPlacement, Legend, Line, Plot, PlotPoints};
use ractor::ActorRef;

use mock_lib_control::PidConfig;
use mock_lib_update::UpdateStatus;

use crate::actors::{OpcuaServerMsg, SharedSnapshot, SharedStatus, SimulationMsg};
use crate::config::AppConfig;
use crate::i18n::{self, Lang, Msg};
use crate::regulator::{Command, Snapshot};

const HISTORY_LEN: usize = 3000;

/// Dépôt GitHub interrogé pour la dernière release (vérification de mise à jour).
const UPDATE_REPO: &str = "CESAMLAB/cesam-tools";
/// Timeout réseau de la vérification de mise à jour (commodité, jamais bloquante).
const UPDATE_TIMEOUT: Duration = Duration::from_secs(5);

const COLOR_SP: egui::Color32 = egui::Color32::from_rgb(90, 140, 255); // bleu
const COLOR_PV: egui::Color32 = egui::Color32::from_rgb(230, 80, 80); // rouge
const COLOR_OUT: egui::Color32 = egui::Color32::from_rgb(170, 200, 60); // vert

/// État de la vérification de mise à jour, partagé avec le thread de requête.
#[derive(Default)]
enum UpdateCheck {
    #[default]
    Idle,
    Checking,
    Done(Result<UpdateStatus, String>),
}

type SharedUpdate = Arc<Mutex<UpdateCheck>>;

struct Sample {
    t: f64,
    pv: f64,
    sp: f64,
    out: f64,
}

/// Application graphique du régulateur OPC UA.
pub struct OpcuaGui {
    sim: ActorRef<SimulationMsg>,
    net: ActorRef<OpcuaServerMsg>,
    snapshot: SharedSnapshot,
    status: SharedStatus,
    config: AppConfig,
    config_path: PathBuf,
    started: Instant,
    history: VecDeque<Sample>,
    show_settings: bool,
    settings_draft: AppConfig,
    feedback: Option<(String, bool)>,
    cesam_logo: Option<egui::TextureHandle>,
    update: SharedUpdate,
    update_thread: Option<JoinHandle<()>>,
}

impl OpcuaGui {
    #[must_use]
    pub fn new(
        sim: ActorRef<SimulationMsg>,
        net: ActorRef<OpcuaServerMsg>,
        snapshot: SharedSnapshot,
        status: SharedStatus,
        config: AppConfig,
        config_path: PathBuf,
    ) -> Self {
        let check_at_startup = config.check_updates;
        let mut gui = Self {
            sim,
            net,
            snapshot,
            status,
            settings_draft: config.clone(),
            config,
            config_path,
            started: Instant::now(),
            history: VecDeque::with_capacity(HISTORY_LEN),
            show_settings: false,
            feedback: None,
            cesam_logo: None,
            update: Arc::new(Mutex::new(UpdateCheck::Idle)),
            update_thread: None,
        };
        if check_at_startup {
            gui.spawn_update_check();
        }
        gui
    }

    /// Lance (si aucune n'est en cours) une vérification de mise à jour dans un
    /// thread dédié : la requête HTTPS est bornée par [`UPDATE_TIMEOUT`].
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

    /// Applique une configuration complète : commandes simulation + réseau + sauvegarde.
    fn apply_settings(&mut self, cfg: AppConfig) {
        self.config = cfg.clone();
        self.send(Command::SetProcess {
            k: cfg.process.k,
            tau: cfg.process.tau,
            dead_time: cfg.process.dead_time,
            ambient: cfg.process.ambient,
        });
        self.send(Command::SetSpBounds {
            min: cfg.regulation.sp_min,
            max: cfg.regulation.sp_max,
        });
        self.send(Command::SetPid(cfg.regulation.pid));
        let _ = self.net.cast(OpcuaServerMsg::Reconfigure(cfg.network.clone()));
        self.save_config();
    }
}

/// Indique si l'endpoint OPC UA est anonyme (sécurité None) : toujours vrai en
/// Phase 1b (la sécurité arrive en Phase 2). Le bandeau rappelle l'exposition.
fn endpoint_is_anonymous() -> bool {
    true
}

/// Éditeur des trois gains d'un PID. Renvoie la nouvelle config si modifiée.
fn pid_editor(ui: &mut egui::Ui, id: &str, cfg: PidConfig) -> Option<PidConfig> {
    let mut edited = cfg;
    let mut changed = false;
    ui.push_id(id, |ui| {
        egui::Grid::new("grid").num_columns(2).show(ui, |ui| {
            for (label, field) in [("Kp", &mut edited.kp), ("Ki", &mut edited.ki), ("Kd", &mut edited.kd)] {
                ui.label(label);
                changed |= ui
                    .add(egui::DragValue::new(field).speed(0.01).range(0.0..=10_000.0))
                    .changed();
                ui.end_row();
            }
        });
    });
    changed.then_some(edited)
}

impl eframe::App for OpcuaGui {
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
            sp: if snap.run { snap.setpoint as f64 } else { f64::NAN },
            out: snap.output as f64,
        });
        while self.history.len() > HISTORY_LEN {
            self.history.pop_front();
        }

        self.top_panel(ctx, &snap);
        self.left_panel(ctx, &snap);
        self.central_panel(ctx, &snap);
        self.settings_window(ctx);

        ctx.request_repaint_after(Duration::from_millis(50));
    }
}

impl OpcuaGui {
    fn top_panel(&mut self, ctx: &egui::Context, snap: &Snapshot) {
        let lang = self.config.language;
        let t = |k: Msg| i18n::tr(lang, k);
        egui::TopBottomPanel::top("entete").show(ctx, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.add(egui::Label::new(
                    egui::RichText::new(format!("RU/OPC UA — {}", t(Msg::AppSubtitle))).size(22.0).strong(),
                ))
                .on_hover_text("Regulation Unit over OPC UA — CESAM-Lab");

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
                        logo_image(ui, logo, 38.0).on_hover_text("CESAM-Lab");
                        ui.separator();
                    }
                    let (txt, color) = if snap.run {
                        (format!("● {}", t(Msg::DeviceRunning)), egui::Color32::from_rgb(0, 180, 0))
                    } else {
                        (format!("● {}", t(Msg::DeviceStopped)), egui::Color32::GRAY)
                    };
                    ui.colored_label(color, txt);
                    ui.separator();
                    if let Ok(st) = self.status.lock() {
                        if st.listening {
                            ui.colored_label(
                                egui::Color32::from_rgb(0, 150, 0),
                                format!("OPC UA ● {}", st.addr),
                            );
                        } else if let Some(err) = &st.error {
                            ui.colored_label(egui::Color32::from_rgb(200, 60, 60), format!("OPC UA ✖ {err}"));
                        } else {
                            ui.colored_label(egui::Color32::GRAY, "OPC UA …");
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
            if endpoint_is_anonymous() {
                ui.colored_label(egui::Color32::from_rgb(200, 140, 0), t(Msg::SecurityAnonymous));
            }
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

    fn left_panel(&mut self, ctx: &egui::Context, snap: &Snapshot) {
        let lang = self.config.language;
        let t = |k: Msg| i18n::tr(lang, k);
        egui::SidePanel::left("commandes")
            .resizable(false)
            .default_width(300.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new(t(Msg::Commands)).strong());
                    ui.separator();

                    let mut run = snap.run;
                    if ui.toggle_value(&mut run, t(Msg::RunStop)).changed() {
                        self.send(Command::SetRun(run));
                    }
                    let mut auto = snap.auto;
                    if ui.toggle_value(&mut auto, t(Msg::AutoMode)).changed() {
                        self.send(Command::SetAuto(auto));
                    }

                    ui.add_space(6.0);
                    ui.label(egui::RichText::new(t(Msg::Setpoint)).strong());
                    let mut sp = snap.setpoint;
                    if ui
                        .add(egui::Slider::new(&mut sp, snap.sp_min..=snap.sp_max))
                        .on_hover_text("OPC UA : Setpoint")
                        .changed()
                    {
                        self.send(Command::SetSetpoint(sp));
                    }

                    ui.add_space(6.0);
                    ui.add_enabled_ui(!snap.auto, |ui| {
                        ui.label(egui::RichText::new(t(Msg::ManualOutput)).strong());
                        let mut out = snap.manual_output;
                        if ui
                            .add(egui::Slider::new(&mut out, 0.0..=100.0).suffix(" %"))
                            .on_hover_text("OPC UA : ManualOutput (mode manuel)")
                            .changed()
                        {
                            self.send(Command::SetManualOutput(out));
                        }
                    });

                    ui.separator();
                    ui.label(egui::RichText::new(t(Msg::PidSettings)).strong());
                    if let Some(cfg) = pid_editor(ui, "pid", snap.pid) {
                        self.config.regulation.pid = cfg;
                        self.send(Command::SetPid(cfg));
                    }
                });
            });
    }

    fn central_panel(&mut self, ctx: &egui::Context, snap: &Snapshot) {
        let lang = self.config.language;
        let t = |k: Msg| i18n::tr(lang, k);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                value_card(ui, t(Msg::ProcessValue), &format!("{:.1}", snap.pv));
                value_card(ui, t(Msg::Setpoint), &if snap.run { format!("{:.1}", snap.setpoint) } else { "—".to_string() });
                value_card(ui, t(Msg::Output), &format!("{:.0} %", snap.output));
            });
            ui.add_space(8.0);

            // Sortie (%) tracée dans l'espace « unité procédé » (×k) ; l'axe de
            // droite ré-affiche les %.
            let k = if snap.sp_max > 0.0 { f64::from(snap.sp_max) / 100.0 } else { 1.0 };
            let pv_axis = AxisHints::new_y().label(t(Msg::LegPv));
            let out_axis = AxisHints::new_y()
                .label(t(Msg::LegOutput))
                .placement(HPlacement::Right)
                .formatter(move |mark, _range| format!("{:.0}", mark.value / k));
            Plot::new("tendance")
                .legend(Legend::default().position(Corner::LeftTop))
                .custom_y_axes(vec![pv_axis, out_axis])
                .height(ui.available_height() - 10.0)
                .x_axis_label(t(Msg::AxisTime))
                .show(ui, |plot_ui| {
                    let pv: PlotPoints = self.history.iter().map(|s| [s.t, s.pv]).collect();
                    let sp: PlotPoints = self.history.iter().map(|s| [s.t, s.sp]).collect();
                    let out: PlotPoints = self.history.iter().map(|s| [s.t, s.out * k]).collect();
                    plot_ui.line(Line::new(t(Msg::LegSetpoint), sp).color(COLOR_SP));
                    plot_ui.line(
                        Line::new(format!("{}   {:.1}", t(Msg::LegPv), snap.pv), pv).color(COLOR_PV),
                    );
                    plot_ui.line(
                        Line::new(format!("{}   {:.0} %", t(Msg::LegOutput), snap.output), out)
                            .color(COLOR_OUT),
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
                .default_width(390.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
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

                        ui.checkbox(&mut draft.check_updates, t(Msg::CheckUpdates));
                        ui.horizontal(|ui| {
                            if ui.button(t(Msg::CheckNow)).clicked() {
                                do_check_now = true;
                            }
                            if let Some((txt, color)) = &update_label {
                                ui.colored_label(*color, txt);
                            }
                        });
                        ui.add_space(6.0);

                        ui.label(egui::RichText::new(t(Msg::Endpoint)).strong());
                        egui::Grid::new("net").num_columns(2).show(ui, |ui| {
                            ui.label(t(Msg::BindIp));
                            ui.text_edit_singleline(&mut draft.network.bind_ip);
                            ui.end_row();
                            ui.label(t(Msg::Port));
                            ui.add(egui::DragValue::new(&mut draft.network.port).range(1..=65535));
                            ui.end_row();
                        });

                        ui.add_space(6.0);
                        ui.label(egui::RichText::new(t(Msg::ProcessParams)).strong());
                        egui::Grid::new("proc").num_columns(2).show(ui, |ui| {
                            ui.label(t(Msg::Gain));
                            ui.add(egui::DragValue::new(&mut draft.process.k).speed(0.01).range(-1000.0..=1000.0));
                            ui.end_row();
                            ui.label(t(Msg::Tau));
                            ui.add(egui::DragValue::new(&mut draft.process.tau).speed(0.5).range(1e-3..=100_000.0));
                            ui.end_row();
                            ui.label(t(Msg::DeadTime));
                            ui.add(egui::DragValue::new(&mut draft.process.dead_time).speed(0.1).range(0.0..=100_000.0));
                            ui.end_row();
                            ui.label(t(Msg::Ambient));
                            ui.add(egui::DragValue::new(&mut draft.process.ambient).speed(0.5));
                            ui.end_row();
                        });

                        ui.add_space(6.0);
                        ui.label(egui::RichText::new(t(Msg::SpBounds)).strong());
                        egui::Grid::new("sp").num_columns(2).show(ui, |ui| {
                            ui.label(t(Msg::SpMin));
                            ui.add(egui::DragValue::new(&mut draft.regulation.sp_min).speed(1.0));
                            ui.end_row();
                            ui.label(t(Msg::SpMax));
                            ui.add(egui::DragValue::new(&mut draft.regulation.sp_max).speed(1.0));
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
