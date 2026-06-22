//! Interface graphique (egui/eframe) de l'agitateur — page unique.
//!
//! L'IHM **lit** des copies partagées ([`SharedSnapshot`], [`SharedStatus`]) et
//! **écrit** uniquement via des `cast` non bloquants vers les acteurs.

use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use eframe::egui;
use egui_plot::{AxisHints, Corner, HPlacement, Legend, Line, Plot, PlotPoints};
use ractor::ActorRef;

use mock_lib_control::PidConfig;

use crate::actors::{NamurServerMsg, SharedSnapshot, SharedStatus, SimulationMsg};
use crate::config::{AppConfig, Parity, Transport};
use crate::i18n::{self, Lang, Msg};
use crate::namur::{self, NamurResponse};
use crate::stirrer::{Command, StirrerSnapshot};
use crate::trace::{self, Direction, SharedTrace};

const HISTORY_LEN: usize = 3000;
const LINK_ACTIVE_TIMEOUT: Duration = Duration::from_secs(3);

const COLOR_SP: egui::Color32 = egui::Color32::from_rgb(90, 140, 255); // bleu
const COLOR_SPEED: egui::Color32 = egui::Color32::from_rgb(230, 80, 80); // rouge
const COLOR_TORQUE: egui::Color32 = egui::Color32::from_rgb(170, 200, 60); // vert
const COLOR_RX: egui::Color32 = egui::Color32::from_rgb(90, 170, 255); // trame reçue
const COLOR_TX: egui::Color32 = egui::Color32::from_rgb(120, 200, 120); // trame émise

struct Sample {
    t: f64,
    speed: f64,
    sp: f64,
    torque: f64,
}

/// Application graphique de l'agitateur.
pub struct StirrerGui {
    sim: ActorRef<SimulationMsg>,
    net: ActorRef<NamurServerMsg>,
    snapshot: SharedSnapshot,
    status: SharedStatus,
    trace: SharedTrace,
    config: AppConfig,
    config_path: PathBuf,
    started: Instant,
    history: VecDeque<Sample>,
    show_settings: bool,
    settings_draft: AppConfig,
    allowlist_text: String,
    feedback: Option<(String, bool)>,
    /// Saisie de la ligne de commande NAMUR du mini-terminal.
    cmd_input: String,
    /// Historique des commandes envoyées (la plus ancienne en tête).
    cmd_history: Vec<String>,
    /// Position courante dans l'historique lors de la navigation (↑/↓) ; `None`
    /// = ligne en cours d'édition (hors navigation).
    history_pos: Option<usize>,
    cesam_logo: Option<egui::TextureHandle>,
}

impl StirrerGui {
    #[must_use]
    pub fn new(
        sim: ActorRef<SimulationMsg>,
        net: ActorRef<NamurServerMsg>,
        snapshot: SharedSnapshot,
        status: SharedStatus,
        trace: SharedTrace,
        config: AppConfig,
        config_path: PathBuf,
    ) -> Self {
        Self {
            sim,
            net,
            snapshot,
            status,
            trace,
            settings_draft: config.clone(),
            allowlist_text: config.network.allowlist.join("\n"),
            config,
            config_path,
            started: Instant::now(),
            history: VecDeque::with_capacity(HISTORY_LEN),
            show_settings: false,
            feedback: None,
            cmd_input: String::new(),
            cmd_history: Vec::new(),
            history_pos: None,
            cesam_logo: None,
        }
    }

    /// Rappelle la commande précédente de l'historique (flèche ↑).
    fn history_prev(&mut self) {
        if self.cmd_history.is_empty() {
            return;
        }
        let new = match self.history_pos {
            None => self.cmd_history.len() - 1,
            Some(0) => 0,
            Some(p) => p - 1,
        };
        self.history_pos = Some(new);
        self.cmd_input = self.cmd_history[new].clone();
    }

    /// Avance vers une commande plus récente (flèche ↓) ; au-delà de la plus
    /// récente, revient à une ligne vide.
    fn history_next(&mut self) {
        match self.history_pos {
            Some(p) if p + 1 < self.cmd_history.len() => {
                self.history_pos = Some(p + 1);
                self.cmd_input = self.cmd_history[p + 1].clone();
            }
            Some(_) => {
                self.history_pos = None;
                self.cmd_input.clear();
            }
            None => {}
        }
    }

    /// Injecte une ligne NAMUR tapée localement : décodée comme une trame maître,
    /// appliquée au moteur (ou répondue), et journalisée (RX puis TX éventuel).
    fn submit_command(&self, line: String) {
        let line = line.trim().to_string();
        if line.is_empty() {
            return;
        }
        trace::record(&self.trace, Direction::Rx, line.clone());
        let snap = match self.snapshot.lock() {
            Ok(g) => *g,
            Err(_) => return,
        };
        match namur::handle_line(&line, &snap) {
            NamurResponse::Reply(reply) => trace::record(&self.trace, Direction::Tx, reply),
            NamurResponse::Apply(cmd) => self.send(cmd),
            // Le chien de garde n'a pas de sens hors session réseau : sans effet ici.
            NamurResponse::SetWatchdog(_) | NamurResponse::Ignore => {}
            NamurResponse::Unknown => trace::record(&self.trace, Direction::Tx, format!("? {line}")),
        }
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
        self.send(Command::SetMotor {
            inertia: cfg.motor.inertia,
            load_coeff: cfg.motor.load_coeff,
            friction: cfg.motor.friction,
            torque_max: cfg.motor.torque_max,
        });
        self.send(Command::SetSpeedLimits {
            min: cfg.regulation.speed_min,
            max: cfg.regulation.speed_max,
        });
        self.send(Command::SetViscosityLimits {
            min: cfg.regulation.viscosity_min,
            max: cfg.regulation.viscosity_max,
        });
        self.send(Command::SetPid(cfg.regulation.pid));
        let _ = self.net.cast(NamurServerMsg::Reconfigure(cfg.network.clone()));
        self.save_config();
    }
}

fn parity_label(lang: Lang, parity: Parity) -> &'static str {
    i18n::tr(
        lang,
        match parity {
            Parity::None => Msg::ParityNone,
            Parity::Even => Msg::ParityEven,
            Parity::Odd => Msg::ParityOdd,
        },
    )
}

/// Indique si le serveur NAMUR/TCP est exposé (toutes interfaces + liste blanche vide).
fn network_is_exposed(config: &AppConfig) -> bool {
    let net = &config.network;
    net.transport == Transport::Tcp
        && (net.bind_ip.trim() == "0.0.0.0" || net.bind_ip.trim() == "::")
        && net.allowlist.iter().all(|p| p.trim().is_empty())
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

impl eframe::App for StirrerGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ensure_logos(ctx);

        let snap = match self.snapshot.lock() {
            Ok(g) => *g,
            Err(_) => return,
        };

        let t = self.started.elapsed().as_secs_f64();
        self.history.push_back(Sample {
            t,
            speed: snap.speed as f64,
            sp: if snap.on { snap.speed_sp as f64 } else { f64::NAN },
            torque: snap.torque as f64,
        });
        while self.history.len() > HISTORY_LEN {
            self.history.pop_front();
        }

        self.top_panel(ctx, &snap);
        self.frames_panel(ctx);
        self.left_panel(ctx, &snap);
        self.central_panel(ctx, &snap);
        self.settings_window(ctx);

        ctx.request_repaint_after(Duration::from_millis(50));
    }
}

impl StirrerGui {
    fn top_panel(&mut self, ctx: &egui::Context, snap: &StirrerSnapshot) {
        let lang = self.config.language;
        let t = |k: Msg| i18n::tr(lang, k);
        egui::TopBottomPanel::top("entete").show(ctx, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.add(egui::Label::new(
                    egui::RichText::new(format!("OSNE — {}", t(Msg::AppSubtitle))).size(24.0).strong(),
                ))
                .on_hover_text("Open Stirrer NAMUR Emulator — CESAM-Lab");

                ui.separator();
                if ui.button(format!("⚙ {}", t(Msg::SettingsBtn))).clicked() {
                    self.settings_draft = self.config.clone();
                    self.allowlist_text = self.config.network.allowlist.join("\n");
                    self.show_settings = true;
                }
                if ui.button(format!("💾 {}", t(Msg::SaveSettingsBtn))).clicked() {
                    self.save_config();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(logo) = &self.cesam_logo {
                        logo_image(ui, logo, 40.0).on_hover_text("CESAM-Lab");
                        ui.separator();
                    }
                    // État appareil.
                    let (txt, color) = if snap.on {
                        (format!("● {}", t(Msg::DeviceRunning)), egui::Color32::from_rgb(0, 180, 0))
                    } else {
                        (format!("● {}", t(Msg::DeviceStopped)), egui::Color32::GRAY)
                    };
                    ui.colored_label(color, txt);
                    ui.separator();
                    // État serveur + voyant de connexion/activité.
                    if let Ok(st) = self.status.lock() {
                        if st.listening {
                            ui.colored_label(egui::Color32::from_rgb(0, 150, 0), format!("NAMUR ● {}", st.addr));
                        } else if let Some(err) = &st.error {
                            ui.colored_label(egui::Color32::from_rgb(200, 60, 60), format!("NAMUR ✖ {err}"));
                        } else {
                            ui.colored_label(egui::Color32::GRAY, "NAMUR …");
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
                            match self.config.network.transport {
                                Transport::Tcp => {
                                    let txt = match &st.peer {
                                        Some(ip) => format!("● {} {}", t(Msg::Master), ip),
                                        None => format!("● {}", t(Msg::NoMaster)),
                                    };
                                    ui.colored_label(color, txt).on_hover_text(hover);
                                }
                                Transport::Serial => {
                                    ui.colored_label(color, "●").on_hover_text(hover);
                                }
                            }
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
            if network_is_exposed(&self.config) {
                ui.colored_label(egui::Color32::from_rgb(200, 140, 0), t(Msg::SecurityExposed));
            }
            ui.add_space(2.0);
        });
    }

    /// Mini-terminal : flux des trames NAMUR reçues (RX) et émises (TX).
    fn frames_panel(&mut self, ctx: &egui::Context) {
        let lang = self.config.language;
        let t = |k: Msg| i18n::tr(lang, k);
        egui::TopBottomPanel::bottom("trames")
            .resizable(true)
            .default_height(175.0)
            .show(ctx, |ui| {
                // Référence du protocole NAMUR, à droite du terminal (clic = insérer).
                egui::SidePanel::right("cmdref")
                    .resizable(true)
                    .default_width(250.0)
                    .show_inside(ui, |ui| {
                        ui.add_space(2.0);
                        ui.label(egui::RichText::new(t(Msg::CmdRefTitle)).strong());
                        ui.label(egui::RichText::new(t(Msg::CmdInsertHint)).small().weak());
                        ui.separator();
                        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                            let rows: [(&str, &str, Msg); 9] = [
                                ("IN_NAME", "IN_NAME", Msg::CmdIdentity),
                                ("IN_PV_4", "IN_PV_4", Msg::CmdReadSpeed),
                                ("IN_PV_5", "IN_PV_5", Msg::CmdReadTorque),
                                ("IN_SP_4", "IN_SP_4", Msg::CmdReadSetpoint),
                                ("OUT_SP_4 <v>", "OUT_SP_4 ", Msg::CmdSetSetpoint),
                                ("START_4", "START_4", Msg::CmdStart),
                                ("STOP_4", "STOP_4", Msg::CmdStop),
                                ("RESET", "RESET", Msg::CmdReset),
                                ("OUT_WD1@<m>", "OUT_WD1@", Msg::CmdWatchdog),
                            ];
                            egui::Grid::new("cmdref_grid")
                                .num_columns(2)
                                .striped(true)
                                .spacing([8.0, 4.0])
                                .show(ui, |ui| {
                                    for (disp, insert, desc) in rows {
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    egui::RichText::new(disp).monospace().small(),
                                                )
                                                .frame(false),
                                            )
                                            .clicked()
                                        {
                                            self.cmd_input = insert.to_string();
                                        }
                                        ui.label(egui::RichText::new(t(desc)).small().weak());
                                        ui.end_row();
                                    }
                                });
                        });
                    });
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("⇄ {}", t(Msg::FramesTitle))).strong());
                    if ui.button(format!("🗑 {}", t(Msg::ClearBtn))).clicked() {
                        if let Ok(mut tr) = self.trace.lock() {
                            tr.clear();
                        }
                    }
                });
                // Ligne de commande : pilotage NAMUR depuis l'IHM.
                let mut submit = false;
                let mut focus_id = None;
                ui.horizontal(|ui| {
                    let resp = ui
                        .add(
                            egui::TextEdit::singleline(&mut self.cmd_input)
                                .desired_width(280.0)
                                .font(egui::TextStyle::Monospace)
                                .hint_text("OUT_SP_4 500"),
                        )
                        .on_hover_text(
                            "IN_NAME · IN_PV_4 · IN_PV_5 · IN_SP_4 · OUT_SP_4 <v> · START_4 · STOP_4 · RESET",
                        );
                    // Édition manuelle = on sort de la navigation d'historique.
                    if resp.changed() {
                        self.history_pos = None;
                    }
                    // Flèches ↑/↓ : navigation dans les commandes précédentes.
                    if resp.has_focus() {
                        if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                            self.history_prev();
                        } else if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                            self.history_next();
                        }
                    }
                    if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        submit = true;
                        focus_id = Some(resp.id);
                    }
                    if ui.button(format!("▶ {}", t(Msg::SendBtn))).clicked() {
                        submit = true;
                        focus_id = Some(resp.id);
                    }
                });
                if submit {
                    let line = std::mem::take(&mut self.cmd_input);
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        // Empile dans l'historique (sans doublon consécutif, borné).
                        if self.cmd_history.last().map(String::as_str) != Some(trimmed) {
                            self.cmd_history.push(trimmed.to_string());
                            if self.cmd_history.len() > 100 {
                                self.cmd_history.remove(0);
                            }
                        }
                    }
                    self.history_pos = None;
                    self.submit_command(line);
                    // Garde le focus pour enchaîner les commandes.
                    if let Some(id) = focus_id {
                        ctx.memory_mut(|m| m.request_focus(id));
                    }
                }
                ui.separator();
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        if let Ok(trace) = self.trace.lock() {
                            for e in trace.iter() {
                                let secs = e.at.saturating_duration_since(self.started).as_secs_f32();
                                let (arrow, color) = match e.dir {
                                    Direction::Rx => ("← RX", COLOR_RX),
                                    Direction::Tx => ("→ TX", COLOR_TX),
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

    fn left_panel(&mut self, ctx: &egui::Context, snap: &StirrerSnapshot) {
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

                    let mut on = snap.on;
                    if ui.toggle_value(&mut on, t(Msg::OnOff)).changed() {
                        self.send(Command::SetOnOff(on));
                    }

                    ui.add_space(6.0);
                    ui.label(egui::RichText::new(t(Msg::SpeedSetpoint)).strong());
                    let mut sp = snap.speed_sp;
                    if ui
                        .add(egui::Slider::new(&mut sp, snap.speed_min..=snap.speed_max).suffix(" tr/min"))
                        .on_hover_text("NAMUR : OUT_SP_4 / IN_SP_4 (canal 4)")
                        .changed()
                    {
                        self.send(Command::SetSpeed(sp));
                    }

                    ui.add_space(6.0);
                    ui.label(egui::RichText::new(t(Msg::Viscosity)).strong());
                    let mut visc = snap.viscosity;
                    if ui
                        .add(
                            egui::Slider::new(&mut visc, snap.viscosity_min..=snap.viscosity_max)
                                .logarithmic(true),
                        )
                        .on_hover_text("Charge du milieu : ↑ viscosité ⇒ ↑ couple")
                        .changed()
                    {
                        self.config.regulation.viscosity = visc;
                        self.send(Command::SetViscosity(visc));
                    }

                    ui.separator();
                    ui.label(egui::RichText::new(t(Msg::PidSettings)).strong());
                    if let Some(cfg) = pid_editor(ui, "pid", snap.pid) {
                        self.config.regulation.pid = cfg;
                        self.send(Command::SetPid(cfg));
                    }
                });
            });
    }

    fn central_panel(&mut self, ctx: &egui::Context, snap: &StirrerSnapshot) {
        let lang = self.config.language;
        let t = |k: Msg| i18n::tr(lang, k);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                value_card(ui, t(Msg::Speed), &format!("{:.0} tr/min", snap.speed));
                value_card(ui, t(Msg::Torque), &format!("{:.1} N·cm", snap.torque));
                value_card(ui, t(Msg::Viscosity), &format!("{:.2}", snap.viscosity));
                if snap.overload {
                    value_card_colored(ui, t(Msg::Overload), "⚠", egui::Color32::from_rgb(200, 80, 0));
                }
            });
            ui.add_space(8.0);

            let sp_txt = if snap.on {
                format!("{:.0} tr/min", snap.speed_sp)
            } else {
                "—".to_string()
            };
            // Facteur d'échelle vitesse↔couple : le couple est tracé dans l'espace
            // « tr/min » (×k) et l'axe de droite ré-affiche les N·cm (÷k).
            let k = if snap.torque_max > 0.0 {
                f64::from(snap.speed_max) / f64::from(snap.torque_max)
            } else {
                1.0
            };
            // Axes nommés explicitement (grandeur + unité).
            let speed_axis = AxisHints::new_y().label(t(Msg::LegSpeed)); // « Vitesse (tr/min) »
            let torque_axis = AxisHints::new_y()
                .label(t(Msg::LegTorque)) // « Couple (N·cm) »
                .placement(HPlacement::Right)
                .formatter(move |mark, _range| format!("{:.0}", mark.value / k));
            Plot::new("tendance")
                .legend(Legend::default().position(Corner::LeftTop))
                .custom_y_axes(vec![speed_axis, torque_axis])
                .height(ui.available_height() - 10.0)
                .x_axis_label(t(Msg::AxisTime))
                .show(ui, |plot_ui| {
                    let speed: PlotPoints = self.history.iter().map(|s| [s.t, s.speed]).collect();
                    let sp: PlotPoints = self.history.iter().map(|s| [s.t, s.sp]).collect();
                    // Couple mis à l'échelle de l'axe vitesse (lu en N·cm à droite).
                    let torque: PlotPoints = self.history.iter().map(|s| [s.t, s.torque * k]).collect();
                    plot_ui.line(Line::new(format!("{}   {sp_txt}", t(Msg::LegSetpoint)), sp).color(COLOR_SP));
                    plot_ui.line(
                        Line::new(format!("{}   {:.0} tr/min", t(Msg::LegSpeed), snap.speed), speed)
                            .color(COLOR_SPEED),
                    );
                    plot_ui.line(
                        Line::new(format!("{}   {:.1} N·cm", t(Msg::LegTorque), snap.torque), torque)
                            .color(COLOR_TORQUE),
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
        let lang = self.settings_draft.language;
        let t = |k: Msg| i18n::tr(lang, k);
        {
            let draft = &mut self.settings_draft;
            let allow_text = &mut self.allowlist_text;
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

                        ui.label(egui::RichText::new(t(Msg::NamurTransport)).strong());
                        ui.horizontal(|ui| {
                            ui.selectable_value(&mut draft.network.transport, Transport::Tcp, "TCP (Ethernet)");
                            ui.selectable_value(&mut draft.network.transport, Transport::Serial, "Série (RS-232)");
                        });
                        ui.add_space(4.0);

                        match draft.network.transport {
                            Transport::Tcp => {
                                egui::Grid::new("net").num_columns(2).show(ui, |ui| {
                                    ui.label(t(Msg::BindIp));
                                    ui.text_edit_singleline(&mut draft.network.bind_ip);
                                    ui.end_row();
                                    ui.label(t(Msg::Port));
                                    ui.add(egui::DragValue::new(&mut draft.network.port).range(1..=65535));
                                    ui.end_row();
                                });
                                ui.label(t(Msg::AllowedIps));
                                ui.add(
                                    egui::TextEdit::multiline(allow_text)
                                        .desired_rows(2)
                                        .hint_text("192.168.1.*\n127.0.0.1"),
                                );
                            }
                            Transport::Serial => {
                                let s = &mut draft.network.serial;
                                egui::Grid::new("serial").num_columns(2).show(ui, |ui| {
                                    ui.label(t(Msg::SerialPort));
                                    ui.text_edit_singleline(&mut s.port)
                                        .on_hover_text("/dev/ttyUSB0, COM3…");
                                    ui.end_row();
                                    ui.label(t(Msg::Baud));
                                    ui.add(egui::DragValue::new(&mut s.baud).range(300..=1_000_000).speed(100));
                                    ui.end_row();
                                    ui.label(t(Msg::Parity));
                                    egui::ComboBox::from_id_salt("parity")
                                        .selected_text(parity_label(lang, s.parity))
                                        .show_ui(ui, |ui| {
                                            for p in [Parity::None, Parity::Even, Parity::Odd] {
                                                ui.selectable_value(&mut s.parity, p, parity_label(lang, p));
                                            }
                                        });
                                    ui.end_row();
                                    ui.label(t(Msg::DataBits));
                                    ui.add(egui::DragValue::new(&mut s.data_bits).range(7..=8));
                                    ui.end_row();
                                    ui.label(t(Msg::StopBits));
                                    ui.add(egui::DragValue::new(&mut s.stop_bits).range(1..=2));
                                    ui.end_row();
                                });
                                ui.label(egui::RichText::new(t(Msg::SerialPointToPoint)).small().weak());
                                #[cfg(not(feature = "serial"))]
                                ui.colored_label(egui::Color32::from_rgb(200, 60, 60), t(Msg::SerialNoFeature));
                            }
                        }

                        ui.add_space(6.0);
                        ui.label(egui::RichText::new(t(Msg::MotorParams)).strong());
                        egui::Grid::new("motor").num_columns(2).show(ui, |ui| {
                            ui.label(t(Msg::Inertia));
                            ui.add(egui::DragValue::new(&mut draft.motor.inertia).speed(0.001).range(1e-4..=10.0));
                            ui.end_row();
                            ui.label(t(Msg::LoadCoeff));
                            ui.add(egui::DragValue::new(&mut draft.motor.load_coeff).speed(0.001).range(0.0..=10.0));
                            ui.end_row();
                            ui.label(t(Msg::Friction));
                            ui.add(egui::DragValue::new(&mut draft.motor.friction).speed(0.1).range(0.0..=1000.0));
                            ui.end_row();
                            ui.label(t(Msg::TorqueMax));
                            ui.add(egui::DragValue::new(&mut draft.motor.torque_max).speed(1.0).range(0.1..=100_000.0));
                            ui.end_row();
                        });

                        ui.add_space(6.0);
                        ui.label(egui::RichText::new(t(Msg::SpeedBounds)).strong());
                        egui::Grid::new("speed").num_columns(2).show(ui, |ui| {
                            ui.label(t(Msg::SpeedMin));
                            ui.add(egui::DragValue::new(&mut draft.regulation.speed_min).speed(1.0));
                            ui.end_row();
                            ui.label(t(Msg::SpeedMax));
                            ui.add(egui::DragValue::new(&mut draft.regulation.speed_max).speed(1.0));
                            ui.end_row();
                        });

                        ui.add_space(6.0);
                        ui.label(egui::RichText::new(t(Msg::ViscosityBounds)).strong());
                        egui::Grid::new("visc").num_columns(2).show(ui, |ui| {
                            ui.label(t(Msg::ViscMin));
                            ui.add(egui::DragValue::new(&mut draft.regulation.viscosity_min).speed(0.05).range(0.001..=1000.0));
                            ui.end_row();
                            ui.label(t(Msg::ViscMax));
                            ui.add(egui::DragValue::new(&mut draft.regulation.viscosity_max).speed(0.05).range(0.001..=1000.0));
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

        if do_close {
            open = false;
        }
        if do_reset {
            self.settings_draft = AppConfig::default();
            self.allowlist_text = self.settings_draft.network.allowlist.join("\n");
            let cfg = self.settings_draft.clone();
            self.apply_settings(cfg);
        } else if do_apply {
            self.settings_draft.network.allowlist = self
                .allowlist_text
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect();
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

fn value_card_colored(ui: &mut egui::Ui, title: &str, value: &str, color: egui::Color32) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new(title).small().weak());
            ui.label(egui::RichText::new(value).heading().color(color));
        });
    });
}
