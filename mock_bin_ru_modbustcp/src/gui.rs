//! Interface graphique (egui/eframe) du régulateur — page unique.
//!
//! Trois zones : panneau de commandes (gauche), supervision + courbe (centre),
//! table d'adresses Modbus live (droite). Un modal `⚙ Paramètres` permet de
//! configurer le réseau et la fonction de transfert. Les réglages sont persistés
//! dans un fichier TOML.
//!
//! # Organisation du fichier
//!
//! - [`RegulatorGui`] : état de l'IHM + méthodes d'envoi/sauvegarde/application.
//! - `impl eframe::App` : point d'entrée `update`, appelé à chaque frame.
//! - `impl RegulatorGui` (panneaux) : `top_panel`, `left_panel`, `right_panel`,
//!   `central_panel`, `settings_window`.
//! - Fonctions libres d'aide : `value_card`, `setpoint_row`, `mode_combo`,
//!   `pid_editor`, et la construction de la table Modbus (`modbus_rows`).
//!
//! # Principe d'interaction
//!
//! L'IHM **lit** des copies partagées ([`SharedSnapshot`], [`SharedStatus`]) et
//! **écrit** uniquement via des messages `cast` non bloquants vers les acteurs ;
//! elle ne possède aucun état métier (seul un brouillon de configuration pour le
//! modal et l'historique de la courbe sont locaux).

use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use eframe::egui;
use egui_plot::{Corner, Legend, Line, Plot, PlotPoints};
use ractor::ActorRef;

use mock_lib_control::{ControllerKind, PidConfig};

use crate::actors::{ModbusServerMsg, SharedSnapshot, SharedStatus, SimulationMsg};
use crate::config::{AppConfig, Parity, Transport};
use crate::i18n::{self, Lang, Msg};
use crate::map;
use crate::regulator::{AutoManual, Command, RegulatorSnapshot};

/// Nombre maximal de points conservés pour la courbe de tendance.
const HISTORY_LEN: usize = 3000;

/// Délai au-delà duquel le lien est considéré inactif (voyant de connexion gris)
/// faute de requête Modbus reçue. Un maître interroge typiquement bien plus vite.
const LINK_ACTIVE_TIMEOUT: Duration = Duration::from_secs(3);

// Couleurs (fixes) des courbes — partagées avec leur pastille de légende.
const COLOR_SP: egui::Color32 = egui::Color32::from_rgb(90, 140, 255); // bleu
const COLOR_PV: egui::Color32 = egui::Color32::from_rgb(230, 80, 80); // rouge
const COLOR_OUT: egui::Color32 = egui::Color32::from_rgb(170, 200, 60); // vert

struct Sample {
    t: f64,
    pv: f64,
    sp: f64,
    output: f64,
}

/// Application graphique du régulateur.
pub struct RegulatorGui {
    sim: ActorRef<SimulationMsg>,
    net: ActorRef<ModbusServerMsg>,
    snapshot: SharedSnapshot,
    status: SharedStatus,
    config: AppConfig,
    config_path: PathBuf,
    started: Instant,
    history: VecDeque<Sample>,
    // État du modal de paramètres.
    show_settings: bool,
    settings_draft: AppConfig,
    allowlist_text: String,
    feedback: Option<(String, bool)>,
    // Logos chargés paresseusement à la première frame (cf. `ensure_logos`).
    orme_logo: Option<egui::TextureHandle>,
    cesam_logo: Option<egui::TextureHandle>,
}

impl RegulatorGui {
    #[must_use]
    pub fn new(
        sim: ActorRef<SimulationMsg>,
        net: ActorRef<ModbusServerMsg>,
        snapshot: SharedSnapshot,
        status: SharedStatus,
        config: AppConfig,
        config_path: PathBuf,
    ) -> Self {
        Self {
            sim,
            net,
            snapshot,
            status,
            settings_draft: config.clone(),
            allowlist_text: config.network.allowlist.join("\n"),
            config,
            config_path,
            started: Instant::now(),
            history: VecDeque::with_capacity(HISTORY_LEN),
            show_settings: false,
            feedback: None,
            orme_logo: None,
            cesam_logo: None,
        }
    }

    /// Charge les textures de logo à la première frame (le contexte egui n'est
    /// disponible qu'à partir de `update`).
    fn ensure_logos(&mut self, ctx: &egui::Context) {
        if self.orme_logo.is_none() {
            self.orme_logo = crate::branding::load_texture(ctx, "orme-icon", crate::branding::ORME_ICON_PNG);
        }
        if self.cesam_logo.is_none() {
            self.cesam_logo = crate::branding::load_texture(ctx, "cesam-logo", crate::branding::CESAM_LOGO_PNG);
        }
    }

    fn send(&self, cmd: Command) {
        let _ = self.sim.cast(SimulationMsg::Command(cmd));
    }

    /// Persiste la configuration courante sur disque et mémorise un retour utilisateur.
    fn save_config(&mut self) {
        let lang = self.config.language;
        match self.config.save(&self.config_path) {
            Ok(()) => {
                self.feedback = Some((
                    format!(
                        "{} ({})",
                        i18n::tr(lang, Msg::SettingsSaved),
                        self.config_path.display()
                    ),
                    true,
                ))
            }
            Err(e) => {
                self.feedback = Some((format!("{} : {e}", i18n::tr(lang, Msg::SaveFailed)), false))
            }
        }
    }

    /// Applique une configuration complète : commandes simulation + réseau + sauvegarde.
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
        let _ = self.net.cast(ModbusServerMsg::Reconfigure(cfg.network.clone()));
        self.save_config();
    }
}

/// Indique si le serveur Modbus est « exposé » : transport TCP, écoute sur toutes
/// les interfaces (`0.0.0.0` / `::`) **et** liste blanche d'IP vide (toutes IP
/// autorisées). Sert à afficher un avertissement de sécurité dans l'IHM.
fn network_is_exposed(config: &AppConfig) -> bool {
    let net = &config.network;
    net.transport == Transport::Tcp
        && (net.bind_ip.trim() == "0.0.0.0" || net.bind_ip.trim() == "::")
        && net.allowlist.iter().all(|p| p.trim().is_empty())
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

/// Combo de sélection d'un mode de régulation. Renvoie `Some(valeur)` si modifié.
fn mode_combo(
    ui: &mut egui::Ui,
    id: &str,
    current: ControllerKind,
    lang: Lang,
) -> Option<ControllerKind> {
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

/// Éditeur des trois gains d'un PID. Renvoie la nouvelle config si modifiée.
fn pid_editor(ui: &mut egui::Ui, id: &str, cfg: PidConfig) -> Option<PidConfig> {
    let mut edited = cfg;
    let mut changed = false;
    ui.push_id(id, |ui| {
        egui::Grid::new("grid").num_columns(2).show(ui, |ui| {
            for (label, field) in [
                ("Kp", &mut edited.kp),
                ("Ki", &mut edited.ki),
                ("Kd", &mut edited.kd),
            ] {
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

        // Historique pour la courbe de tendance.
        let t = self.started.elapsed().as_secs_f64();
        self.history.push_back(Sample {
            t,
            pv: snap.pv as f64,
            sp: if snap.mode.is_auto() {
                snap.sp_auto as f64
            } else {
                f64::NAN
            },
            output: snap.output as f64,
        });
        while self.history.len() > HISTORY_LEN {
            self.history.pop_front();
        }

        self.top_panel(ctx, &snap);
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
                if let Some(logo) = &self.orme_logo {
                    logo_image(ui, logo, 44.0)
                        .on_hover_text("ORME — Open Regulator Modbus Emulator");
                }
                ui.add(egui::Label::new(
                    egui::RichText::new(format!("ORME — {}", t(Msg::AppSubtitle)))
                        .size(26.0)
                        .strong(),
                ))
                .on_hover_text("Open Regulator Modbus Emulator — CESAM-Lab");

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
                    // Signature éditeur : logo CESAM-Lab (le plus à droite).
                    if let Some(logo) = &self.cesam_logo {
                        logo_image(ui, logo, 48.0).on_hover_text("CESAM-Lab");
                        ui.separator();
                    }
                    // État appareil
                    let (txt, color) = if snap.on {
                        (
                            format!("● {}", t(Msg::DeviceRunning)),
                            egui::Color32::from_rgb(0, 180, 0),
                        )
                    } else {
                        (format!("● {}", t(Msg::DeviceStopped)), egui::Color32::GRAY)
                    };
                    ui.colored_label(color, txt);
                    ui.separator();
                    // État serveur Modbus + voyant de connexion/activité du lien.
                    if let Ok(st) = self.status.lock() {
                        if st.listening {
                            ui.colored_label(
                                egui::Color32::from_rgb(0, 150, 0),
                                format!("Modbus ● {}", st.addr),
                            );
                        } else if let Some(err) = &st.error {
                            ui.colored_label(egui::Color32::from_rgb(200, 60, 60), format!("Modbus ✖ {err}"));
                        } else {
                            ui.colored_label(egui::Color32::GRAY, "Modbus …");
                        }

                        // Voyant de connexion : vert si une requête a été reçue
                        // récemment (le maître interroge), gris sinon. En TCP on
                        // affiche en plus l'IP du maître ; en RTU (bus série sans
                        // connexion) un simple voyant d'activité suffit.
                        if st.listening {
                            ui.separator();
                            let active = st
                                .last_request
                                .is_some_and(|ts| ts.elapsed() < LINK_ACTIVE_TIMEOUT);
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
                                Transport::Rtu => {
                                    ui.colored_label(color, "●").on_hover_text(hover);
                                }
                            }
                        }
                    }
                });
            });
            // Retour de sauvegarde éventuel.
            if let Some((msg, ok)) = &self.feedback {
                let color = if *ok {
                    egui::Color32::from_rgb(0, 150, 0)
                } else {
                    egui::Color32::from_rgb(200, 60, 60)
                };
                ui.colored_label(color, msg);
            }
            // Avertissement de sécurité : serveur TCP exposé sur toutes les interfaces
            // sans liste blanche d'IP (Modbus n'a ni authentification ni chiffrement).
            if network_is_exposed(&self.config) {
                ui.colored_label(egui::Color32::from_rgb(200, 140, 0), t(Msg::SecurityExposed));
            }
            ui.add_space(2.0);
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

                    // Marche / Arrêt
                    let mut on = snap.on;
                    if ui.toggle_value(&mut on, t(Msg::OnOff)).changed() {
                        self.send(Command::SetOnOff(on));
                    }

                    // Auto / Manuel
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
                        ui.label(t(Msg::Sens1Hot))
                            .on_hover_text(format!("HR {} — 0=Off,1=PID,2=TOR,3=PWM", map::HR_MODE_SENS1));
                        if let Some(k) = mode_combo(ui, "mode_sens1", snap.mode_sens1, lang) {
                            self.send(Command::SetModeSens1(k));
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label(t(Msg::Sens2Cold))
                            .on_hover_text(format!("HR {} — 0=Off,1=PID,2=TOR,3=PWM", map::HR_MODE_SENS2));
                        if let Some(k) = mode_combo(ui, "mode_sens2", snap.mode_sens2, lang) {
                            self.send(Command::SetModeSens2(k));
                        }
                    });

                    ui.separator();
                    ui.label(egui::RichText::new(t(Msg::Setpoints)).strong());

                    // Consigne auto — toujours éditable, mise en avant si active.
                    if let Some(v) = setpoint_row(
                        ui,
                        t(Msg::SpAuto),
                        &format!("HR {}–{} (f32)", map::HR_SP_AUTO, map::HR_SP_AUTO + 1),
                        snap.mode.is_auto(),
                        snap.sp_auto,
                        snap.sp_min,
                        snap.sp_max,
                        " u",
                    ) {
                        self.send(Command::SetSpAuto(v));
                    }

                    // Consigne manuelle (% sortie signé).
                    if let Some(v) = setpoint_row(
                        ui,
                        t(Msg::SpManual),
                        &format!("HR {}–{} (f32)", map::HR_SP_MANUAL, map::HR_SP_MANUAL + 1),
                        !snap.mode.is_auto(),
                        snap.sp_manual,
                        -100.0,
                        100.0,
                        " %",
                    ) {
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
                    if ui
                        .add(egui::Slider::new(&mut hyst, 0.0..=20.0).text(t(Msg::HystSlider)))
                        .on_hover_text(format!("HR {}–{} (f32)", map::HR_HYSTERESIS, map::HR_HYSTERESIS + 1))
                        .changed()
                    {
                        self.config.regulation.hysteresis = hyst;
                        self.send(Command::SetHysteresis(hyst));
                    }
                    let mut min_cycle = snap.tor_min_cycle;
                    if ui
                        .add(egui::Slider::new(&mut min_cycle, 0.0..=120.0).text(t(Msg::TorMinCycleSlider)))
                        .on_hover_text(format!(
                            "{} — HR {}–{} (f32)",
                            t(Msg::HintAntiShortCycle),
                            map::HR_TOR_MIN_CYCLE,
                            map::HR_TOR_MIN_CYCLE + 1
                        ))
                        .changed()
                    {
                        self.config.regulation.tor_min_cycle = min_cycle;
                        self.send(Command::SetTorMinCycle(min_cycle));
                    }
                    let mut pwm_period = snap.pwm_period;
                    if ui
                        .add(egui::Slider::new(&mut pwm_period, 0.5..=120.0).text(t(Msg::PwmPeriodSlider)))
                        .on_hover_text(format!(
                            "{} — HR {}–{} (f32)",
                            t(Msg::HintCyclicRelay),
                            map::HR_PWM_PERIOD,
                            map::HR_PWM_PERIOD + 1
                        ))
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
        egui::SidePanel::right("table_modbus")
            .resizable(true)
            .default_width(360.0)
            .show(ctx, |ui| {
                ui.add_space(6.0);
                ui.label(egui::RichText::new(t(Msg::ModbusTable)).strong());
                ui.label(egui::RichText::new(t(Msg::ModbusTableNote)).small().weak());
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("grille_modbus")
                        .num_columns(5)
                        .striped(true)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            for h in [
                                Msg::ColName,
                                Msg::ColTable,
                                Msg::ColAddr,
                                Msg::ColValue,
                                Msg::ColAccess,
                            ] {
                                ui.label(egui::RichText::new(t(h)).strong().small());
                            }
                            ui.end_row();
                            for row in modbus_rows(snap, lang) {
                                ui.label(row.name);
                                ui.label(row.table);
                                ui.label(row.addr);
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

            // Dernières valeurs affichées directement dans la légende.
            let sp_txt = if snap.mode.is_auto() {
                format!("{:.2} u", snap.sp_auto)
            } else {
                t(Msg::ManualDash).to_string()
            };
            Plot::new("tendance")
                // Légende en haut à GAUCHE.
                .legend(Legend::default().position(Corner::LeftTop))
                .height(ui.available_height() - 10.0)
                .x_axis_label(t(Msg::AxisTime))
                .show(ui, |plot_ui| {
                    let sp: PlotPoints = self.history.iter().map(|s| [s.t, s.sp]).collect();
                    let pv: PlotPoints = self.history.iter().map(|s| [s.t, s.pv]).collect();
                    let out: PlotPoints = self.history.iter().map(|s| [s.t, s.output]).collect();
                    // Libellé = nom + dernière valeur ; la pastille colorée est ajoutée par la légende.
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
        // Langue de prévisualisation : celle du brouillon (mise à jour vivante au
        // changement du sélecteur, frame suivante).
        let lang = self.settings_draft.language;
        let t = |k: Msg| i18n::tr(lang, k);
        {
            let draft = &mut self.settings_draft;
            let allow_text = &mut self.allowlist_text;
            egui::Window::new(format!("⚙ {}", t(Msg::SettingsTitle)))
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .default_width(380.0)
                .show(ctx, |ui| {
                    // Sélecteur de langue de l'interface.
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

                    ui.label(egui::RichText::new(t(Msg::ModbusTransport)).strong());
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut draft.network.transport, Transport::Tcp, "TCP (Ethernet)");
                        ui.selectable_value(&mut draft.network.transport, Transport::Rtu, "RTU (RS485)");
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
                                    .desired_rows(3)
                                    .hint_text("192.168.1.*\n127.0.0.1"),
                            );
                        }
                        Transport::Rtu => {
                            let s = &mut draft.network.serial;
                            egui::Grid::new("serial").num_columns(2).show(ui, |ui| {
                                ui.label(t(Msg::SerialPort));
                                ui.text_edit_singleline(&mut s.port)
                                    .on_hover_text("/dev/ttyUSB0, /dev/ttyAMA0, COM3…");
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
                                ui.label(t(Msg::SlaveId));
                                ui.add(egui::DragValue::new(&mut s.slave_id).range(1..=247));
                                ui.end_row();
                            });
                            ui.label(egui::RichText::new(t(Msg::RtuPointToPoint)).small().weak());
                            #[cfg(not(feature = "rtu"))]
                            ui.colored_label(
                                egui::Color32::from_rgb(200, 60, 60),
                                t(Msg::RtuNoFeature),
                            );
                        }
                    }

                    ui.add_space(6.0);
                    ui.label(egui::RichText::new(t(Msg::ProcessTf)).strong());
                    ui.label(
                        egui::RichText::new("G(s) = K·e^(-L·s) / (1 + T·s)")
                            .small()
                            .weak(),
                    );
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

/// Affiche un logo à une hauteur donnée en conservant son rapport d'aspect.
fn logo_image(ui: &mut egui::Ui, tex: &egui::TextureHandle, height: f32) -> egui::Response {
    let size = tex.size_vec2();
    let width = if size.y > 0.0 { height * size.x / size.y } else { height };
    ui.add(egui::Image::new(tex).fit_to_exact_size(egui::vec2(width, height)))
}

/// Petite carte d'affichage d'une valeur instantanée.
fn value_card(ui: &mut egui::Ui, title: &str, value: &str) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new(title).small().weak());
            ui.label(egui::RichText::new(value).heading());
        });
    });
}

/// Ligne de consigne : libellé + champ numérique + slider, toujours éditable.
/// `active` met la ligne en avant. Renvoie `Some(valeur)` si modifiée.
#[allow(clippy::too_many_arguments)]
fn setpoint_row(
    ui: &mut egui::Ui,
    label: &str,
    addr_hint: &str,
    active: bool,
    current: f32,
    min: f32,
    max: f32,
    suffix: &str,
) -> Option<f32> {
    let mut value = current;
    let mut changed = false;
    ui.push_id(label, |ui| {
        ui.horizontal(|ui| {
            let title = egui::RichText::new(label);
            let title = if active { title.strong() } else { title.weak() };
            ui.label(title).on_hover_text(addr_hint);
            changed |= ui
                .add(egui::DragValue::new(&mut value).range(min..=max).suffix(suffix))
                .changed();
        });
        changed |= ui
            .add(egui::Slider::new(&mut value, min..=max).suffix(suffix).show_value(false))
            .changed();
    });
    changed.then_some(value)
}

/// Une ligne de la table d'adresses Modbus affichée dans l'IHM.
struct ModbusRow {
    name: String,
    table: &'static str,
    addr: String,
    value: String,
    access: &'static str,
}

fn f32_addr(a: u16) -> String {
    format!("{}–{}", a, a + 1)
}

/// Construit la liste des lignes de la table Modbus à partir de l'état courant.
fn modbus_rows(s: &RegulatorSnapshot, lang: Lang) -> Vec<ModbusRow> {
    let onoff = |b: bool| if b { "1" } else { "0" }.to_string();
    let kind = |k: ControllerKind| format!("{} ({})", k.to_code(), mode_label(lang, k));
    let t = |k: Msg| i18n::tr(lang, k);
    // Gains PID : « Kp/Ki/Kd » (universel) + sens traduit.
    let gain = |sym: &str, dir: Msg| format!("{sym} {}", t(dir));
    // Plage de registres occupée par la chaîne d'identification ASCII.
    let label_end = map::HR_LABEL + (map::LABEL_TEXT.len() as u16).div_ceil(2) - 1;
    vec![
        ModbusRow { name: "On/Off".to_string(), table: "Coil", addr: map::COIL_ON_OFF.to_string(), value: onoff(s.on), access: "R/W" },
        ModbusRow { name: "Auto/Manual".to_string(), table: "Coil", addr: map::COIL_AUTO_MANUAL.to_string(), value: onoff(s.mode.is_auto()), access: "R/W" },
        ModbusRow { name: t(Msg::RowRunning).to_string(), table: "DI", addr: map::DI_RUNNING.to_string(), value: onoff(s.on), access: "R" },
        ModbusRow { name: t(Msg::RowHeatingActive).to_string(), table: "DI", addr: map::DI_HEATING.to_string(), value: onoff(s.on && s.output > 0.0), access: "R" },
        ModbusRow { name: t(Msg::RowCoolingActive).to_string(), table: "DI", addr: map::DI_COOLING.to_string(), value: onoff(s.on && s.output < 0.0), access: "R" },
        ModbusRow { name: t(Msg::RowModeSens1).to_string(), table: "HR", addr: map::HR_MODE_SENS1.to_string(), value: kind(s.mode_sens1), access: "R/W" },
        ModbusRow { name: t(Msg::RowModeSens2).to_string(), table: "HR", addr: map::HR_MODE_SENS2.to_string(), value: kind(s.mode_sens2), access: "R/W" },
        ModbusRow { name: t(Msg::SpAuto).to_string(), table: "HR", addr: f32_addr(map::HR_SP_AUTO), value: format!("{:.2}", s.sp_auto), access: "R/W" },
        ModbusRow { name: format!("{} (%)", t(Msg::SpManual)), table: "HR", addr: f32_addr(map::HR_SP_MANUAL), value: format!("{:.2}", s.sp_manual), access: "R/W" },
        ModbusRow { name: gain("Kp", Msg::Dir1), table: "HR", addr: f32_addr(map::HR_KP_SENS1), value: format!("{:.3}", s.pid_heat.kp), access: "R/W" },
        ModbusRow { name: gain("Ki", Msg::Dir1), table: "HR", addr: f32_addr(map::HR_KI_SENS1), value: format!("{:.3}", s.pid_heat.ki), access: "R/W" },
        ModbusRow { name: gain("Kd", Msg::Dir1), table: "HR", addr: f32_addr(map::HR_KD_SENS1), value: format!("{:.3}", s.pid_heat.kd), access: "R/W" },
        ModbusRow { name: gain("Kp", Msg::Dir2), table: "HR", addr: f32_addr(map::HR_KP_SENS2), value: format!("{:.3}", s.pid_cool.kp), access: "R/W" },
        ModbusRow { name: gain("Ki", Msg::Dir2), table: "HR", addr: f32_addr(map::HR_KI_SENS2), value: format!("{:.3}", s.pid_cool.ki), access: "R/W" },
        ModbusRow { name: gain("Kd", Msg::Dir2), table: "HR", addr: f32_addr(map::HR_KD_SENS2), value: format!("{:.3}", s.pid_cool.kd), access: "R/W" },
        ModbusRow { name: t(Msg::RowHysteresis).to_string(), table: "HR", addr: f32_addr(map::HR_HYSTERESIS), value: format!("{:.2}", s.hysteresis), access: "R/W" },
        ModbusRow { name: t(Msg::TorMinCycleSlider).to_string(), table: "HR", addr: f32_addr(map::HR_TOR_MIN_CYCLE), value: format!("{:.2}", s.tor_min_cycle), access: "R/W" },
        ModbusRow { name: t(Msg::PwmPeriodSlider).to_string(), table: "HR", addr: f32_addr(map::HR_PWM_PERIOD), value: format!("{:.2}", s.pwm_period), access: "R/W" },
        ModbusRow { name: t(Msg::RowIdent).to_string(), table: "HR", addr: format!("{}–{}", map::HR_LABEL, label_end), value: format!("\"{}\"", map::LABEL_TEXT), access: "R" },
        ModbusRow { name: t(Msg::Measure).to_string(), table: "IR", addr: f32_addr(map::IR_PV), value: format!("{:.2}", s.pv), access: "R" },
        ModbusRow { name: t(Msg::OutputPct).to_string(), table: "IR", addr: f32_addr(map::IR_OUTPUT), value: format!("{:+.2}", s.output), access: "R" },
        ModbusRow { name: format!("{} ({})", t(Msg::SpAuto), t(Msg::Readback)), table: "IR", addr: f32_addr(map::IR_SP_AUTO), value: format!("{:.2}", s.sp_auto), access: "R" },
        ModbusRow { name: format!("{} ({})", t(Msg::SpManual), t(Msg::Readback)), table: "IR", addr: f32_addr(map::IR_SP_MANUAL), value: format!("{:.2}", s.sp_manual), access: "R" },
    ]
}
