//! Régulateur industriel simulé.
//!
//! Assemble les sous-systèmes :
//! - l'**acteur de simulation** ([`actors::simulation`]) : boucle de régulation
//!   sur une fonction de transfert ;
//! - l'**acteur réseau** ([`actors::network`]) : serveur Modbus TCP (re)configurable ;
//! - l'**interface graphique** ([`gui`]) : pilotage, visualisation et paramétrage.
//!
//! Le runtime Tokio (acteurs + Modbus) tourne sur des threads de fond ; l'IHM
//! `eframe` occupe le thread principal.

// En mode headless (sans IHM), certaines commandes/réglages/ré-exports ne servent
// qu'à l'interface graphique : on tolère le code et les imports inutilisés pour
// cette configuration uniquement (le build par défaut reste strict).
#![cfg_attr(not(feature = "gui"), allow(unused))]

mod actors;
#[cfg(feature = "gui")]
mod branding;
mod config;
#[cfg(feature = "gui")]
mod gui;
mod i18n;
mod map;
mod modbus_server;
mod regulator;

use std::sync::{Arc, Mutex};

use anyhow::Context;
use ractor::Actor;

use actors::{ModbusServerActor, ModbusServerArgs, SimulationActor, SimulationArgs};
use config::{AppConfig, IpFilter, ServerStatus};
use map::MemoryMap;
use regulator::Regulator;

#[cfg(feature = "gui")]
use gui::RegulatorGui;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config_path = AppConfig::path();
    let config = AppConfig::load(&config_path);
    let regulator_config = config.to_regulator_config();

    // Structures partagées entre les acteurs et l'IHM.
    let initial = Regulator::new(regulator_config.clone()).snapshot();
    let snapshot = Arc::new(Mutex::new(initial));
    let mut map0 = MemoryMap::default();
    map0.refresh_from(&initial);
    let map = Arc::new(Mutex::new(map0));
    let allowlist = Arc::new(Mutex::new(IpFilter::new(config.network.allowlist.clone())));
    let status = Arc::new(Mutex::new(ServerStatus::default()));

    // Runtime Tokio multi-thread, maintenu vivant pendant toute la durée de l'IHM.
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
                map: map.clone(),
            },
        )
        .await
        .context("starting the simulation actor")?;

        let (net_actor, _net_join) = Actor::spawn(
            Some("modbus".to_string()),
            ModbusServerActor,
            ModbusServerArgs {
                network: config.network.clone(),
                sim: sim_actor.clone(),
                map: map.clone(),
                snapshot: snapshot.clone(),
                allowlist: allowlist.clone(),
                status: status.clone(),
            },
        )
        .await
        .context("starting the Modbus network actor")?;

        anyhow::Ok((sim_actor, net_actor))
    })?;

    // --- Mode graphique (feature `gui`) ---
    // L'IHM bloque le thread principal ; le runtime reste vivant grâce à `runtime`.
    #[cfg(feature = "gui")]
    {
        let title = format!(
            "ORME — {} (TCP/RTU)",
            i18n::tr(config.language, i18n::Msg::AppSubtitle)
        );
        let mut viewport = eframe::egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 760.0])
            .with_min_inner_size([900.0, 600.0])
            // `app_id` = identité de la fenêtre côté Wayland (et WM_CLASS sous X11).
            // Sous Wayland, l'icône de la barre des tâches n'est PAS prise depuis
            // `with_icon` (ignoré par le compositeur) mais résolue via le fichier
            // `orme.desktop` portant ce même nom (voir `packaging/`).
            .with_app_id("orme")
            .with_title(title);
        if let Some(icon) = branding::window_icon() {
            viewport = viewport.with_icon(icon);
        }
        let options = eframe::NativeOptions {
            viewport,
            ..Default::default()
        };

        let result = eframe::run_native(
            "mock_bin_ru_modbustcp",
            options,
            Box::new(move |_cc| {
                Ok(Box::new(RegulatorGui::new(
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

    // --- Mode headless (sans IHM) : serveur Modbus + simulation en tâche de fond ---
    #[cfg(not(feature = "gui"))]
    {
        log::info!("Headless mode — Modbus server running. Stop with Ctrl-C / SIGTERM.");
        // Garde les acteurs (et donc le serveur) en vie indéfiniment.
        let _actors = (sim_actor, net_actor);
        let _keep = (&snapshot, &map, &allowlist, &status, &config, &config_path);
        runtime.block_on(std::future::pending::<()>());
    }

    drop(runtime);
    Ok(())
}
