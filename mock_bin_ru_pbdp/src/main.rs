//! Régulateur industriel simulé, exposé via un **simulateur logiciel de trames
//! PROFIBUS DP-V0** (voir `docs/fr/reference_profibus.md` pour les limites de
//! conformité — ⚠️ non interopérable avec du matériel PROFIBUS réel).
//!
//! Assemble les sous-systèmes :
//! - l'**acteur de simulation** ([`actors::simulation`]) : boucle de régulation
//!   sur une fonction de transfert (modèle identique à ORME) ;
//! - l'**acteur réseau** ([`actors::network`]) : liaison série PROFIBUS DP
//!   (re)configurable à chaud ;
//! - l'**interface graphique** ([`gui`]) : pilotage, visualisation et paramétrage.
//!
//! Le runtime Tokio (acteurs + liaison série) tourne sur des threads de fond ;
//! l'IHM `eframe` occupe le thread principal.

#![cfg_attr(not(feature = "gui"), allow(unused))]
#![cfg_attr(
    all(target_os = "windows", feature = "gui", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod actors;
#[cfg(feature = "gui")]
mod branding;
mod config;
#[cfg(feature = "gui")]
mod gui;
mod i18n;
mod map;
mod profibus;
mod profibus_server;
mod regulator;
mod trace;

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use anyhow::Context;
use ractor::Actor;

use actors::{ProfibusServerActor, ProfibusServerArgs, SimulationActor, SimulationArgs};
use config::{AppConfig, ServerStatus};
use regulator::Regulator;

#[cfg(feature = "gui")]
use gui::RegulatorGui;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config_path = AppConfig::path();
    let config = AppConfig::load(&config_path);
    let regulator_config = config.to_regulator_config();

    let initial = Regulator::new(regulator_config.clone()).snapshot();
    let snapshot = Arc::new(Mutex::new(initial));
    let status = Arc::new(Mutex::new(ServerStatus::default()));
    // Journal des trames PROFIBUS (mini-terminal de l'IHM).
    let trace: trace::SharedTrace = Arc::new(Mutex::new(VecDeque::new()));

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("creating the Tokio runtime")?;

    let (sim_actor, net_actor) = runtime.block_on(async {
        let (sim_actor, _sim_join) = Actor::spawn(
            Some("simulation".to_string()),
            SimulationActor,
            SimulationArgs {
                config: regulator_config,
                snapshot: snapshot.clone(),
            },
        )
        .await
        .context("starting the simulation actor")?;

        let (net_actor, _net_join) = Actor::spawn(
            Some("profibus".to_string()),
            ProfibusServerActor,
            ProfibusServerArgs {
                network: config.network.clone(),
                sim: sim_actor.clone(),
                snapshot: snapshot.clone(),
                status: status.clone(),
                trace: trace.clone(),
            },
        )
        .await
        .context("starting the PROFIBUS DP network actor")?;

        anyhow::Ok((sim_actor, net_actor))
    })?;

    #[cfg(feature = "gui")]
    {
        let title = format!(
            "ORPD — {} (PROFIBUS DP)",
            i18n::tr(config.language, i18n::Msg::AppSubtitle)
        );
        let mut viewport = eframe::egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 760.0])
            .with_min_inner_size([900.0, 600.0])
            .with_app_id("ru_pbdp")
            .with_title(title);
        if let Some(icon) = branding::window_icon() {
            viewport = viewport.with_icon(icon);
        }
        let options = eframe::NativeOptions {
            viewport,
            ..Default::default()
        };

        let result = eframe::run_native(
            "mock_bin_ru_pbdp",
            options,
            Box::new(move |_cc| {
                Ok(Box::new(RegulatorGui::new(
                    sim_actor.clone(),
                    net_actor.clone(),
                    snapshot.clone(),
                    status.clone(),
                    trace.clone(),
                    config.clone(),
                    config_path.clone(),
                )))
            }),
        );
        result.map_err(|e| anyhow::anyhow!("GUI error: {e}"))?;
    }

    #[cfg(not(feature = "gui"))]
    {
        log::info!("Headless mode — PROFIBUS DP link running. Stop with Ctrl-C / SIGTERM.");
        let _actors = (sim_actor, net_actor);
        let _keep = (&snapshot, &status, &trace, &config, &config_path);
        runtime.block_on(std::future::pending::<()>());
    }

    drop(runtime);
    Ok(())
}
