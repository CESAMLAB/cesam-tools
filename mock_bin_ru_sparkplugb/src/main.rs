//! Régulateur de procédé simulé exposé en **MQTT Sparkplug B** — marque **ORSE**
//! (*Open Regulator Sparkplug Emulator*) ; nom technique : RU/Sparkplug B.
//!
//! Assemble :
//! - l'**acteur de simulation** : régulateur PID + procédé du premier ordre ;
//! - l'**acteur réseau** : edge node **Sparkplug B** (client MQTT sortant vers un
//!   broker) (re)configurable à chaud ;
//! - l'**interface graphique** (feature `gui`) : pilotage et visualisation.

// En mode headless, certaines API ne servent qu'à l'IHM.
#![cfg_attr(not(feature = "gui"), allow(unused))]
// Sous Windows, le binaire IHM (release) utilise le sous-système « windows » : pas
// de console superflue. En headless ou en debug, on garde la console (logs).
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
mod regulator;
mod sparkplug_node;

use std::sync::{Arc, Mutex};

use anyhow::Context;
use ractor::Actor;

use actors::{SimulationActor, SimulationArgs, SparkplugActor, SparkplugArgs};
use config::{AppConfig, ServerStatus};
use regulator::Regulator;

#[cfg(feature = "gui")]
use gui::SparkplugGui;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config_path = AppConfig::path();
    let config = AppConfig::load(&config_path);
    let reg_config = config.to_regulator_config();

    let initial = Regulator::new(reg_config).snapshot();
    let snapshot = Arc::new(Mutex::new(initial));
    let status = Arc::new(Mutex::new(ServerStatus::default()));

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("creating the Tokio runtime")?;

    let (sim_actor, net_actor) = runtime.block_on(async {
        let (sim_actor, _sim_join) = Actor::spawn(
            Some("simulation".to_string()),
            SimulationActor,
            SimulationArgs {
                config: reg_config,
                snapshot: snapshot.clone(),
            },
        )
        .await
        .context("starting the simulation actor")?;

        let (net_actor, _net_join) = Actor::spawn(
            Some("sparkplug".to_string()),
            SparkplugActor,
            SparkplugArgs {
                network: config.network.clone(),
                sim: sim_actor.clone(),
                snapshot: snapshot.clone(),
                status: status.clone(),
            },
        )
        .await
        .context("starting the Sparkplug B network actor")?;

        anyhow::Ok((sim_actor, net_actor))
    })?;

    #[cfg(feature = "gui")]
    {
        let title = format!("ORSE — {}", i18n::tr(config.language, i18n::Msg::AppSubtitle));
        let mut viewport = eframe::egui::ViewportBuilder::default()
            .with_inner_size([1080.0, 700.0])
            .with_min_inner_size([860.0, 540.0])
            .with_app_id("ru_spb")
            .with_title(title);
        if let Some(icon) = branding::window_icon() {
            viewport = viewport.with_icon(icon);
        }
        let options = eframe::NativeOptions {
            viewport,
            ..Default::default()
        };
        let result = eframe::run_native(
            "mock_bin_ru_sparkplugb",
            options,
            Box::new(move |_cc| {
                Ok(Box::new(SparkplugGui::new(
                    sim_actor.clone(),
                    net_actor.clone(),
                    snapshot.clone(),
                    status.clone(),
                    config.clone(),
                    config_path.clone(),
                )))
            }),
        );
        result.map_err(|e| anyhow::anyhow!("GUI error: {e}"))?;
    }

    #[cfg(not(feature = "gui"))]
    {
        log::info!("Headless mode — Sparkplug B edge node running. Stop with Ctrl-C / SIGTERM.");
        let _actors = (sim_actor, net_actor);
        let _keep = (&snapshot, &status, &config, &config_path);
        runtime.block_on(std::future::pending::<()>());
    }

    drop(runtime);
    Ok(())
}
