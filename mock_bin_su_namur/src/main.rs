//! Agitateur de laboratoire simulé (façon IKA) — OSNE.
//!
//! Assemble :
//! - l'**acteur de simulation** : moteur à fonction de transfert + asservissement
//!   de vitesse rapide, charge visqueuse réglable ;
//! - l'**acteur réseau** : serveur **NAMUR** (TCP, ou série RS-232 via la feature
//!   `serial`) (re)configurable à chaud ;
//! - l'**interface graphique** (feature `gui`) : pilotage et visualisation.

// En mode headless, certaines API ne servent qu'à l'IHM.
#![cfg_attr(not(feature = "gui"), allow(unused))]
// Sous Windows, le binaire IHM (release) utilise le sous-système « windows » : pas
// de console superflue à côté de la fenêtre. En **headless** (serveur, sans IHM)
// ou en build **debug**, on conserve la console pour afficher les logs.
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
mod motor;
mod namur;
mod namur_server;
mod stirrer;
mod trace;

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use anyhow::Context;
use ractor::Actor;

use actors::{NamurServerActor, NamurServerArgs, SimulationActor, SimulationArgs};
use config::{AppConfig, IpFilter, ServerStatus};
use stirrer::Stirrer;

#[cfg(feature = "gui")]
use gui::StirrerGui;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config_path = AppConfig::path();
    let config = AppConfig::load(&config_path);
    let stirrer_config = config.to_stirrer_config();

    let initial = Stirrer::new(stirrer_config.clone()).snapshot();
    let snapshot = Arc::new(Mutex::new(initial));
    let allowlist = Arc::new(Mutex::new(IpFilter::new(config.network.allowlist.clone())));
    let status = Arc::new(Mutex::new(ServerStatus::default()));
    // Journal des trames NAMUR (mini-terminal de l'IHM).
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
                config: stirrer_config,
                snapshot: snapshot.clone(),
            },
        )
        .await
        .context("starting the simulation actor")?;

        let (net_actor, _net_join) = Actor::spawn(
            Some("namur".to_string()),
            NamurServerActor,
            NamurServerArgs {
                network: config.network.clone(),
                sim: sim_actor.clone(),
                snapshot: snapshot.clone(),
                allowlist: allowlist.clone(),
                status: status.clone(),
                trace: trace.clone(),
            },
        )
        .await
        .context("starting the NAMUR network actor")?;

        anyhow::Ok((sim_actor, net_actor))
    })?;

    #[cfg(feature = "gui")]
    {
        let title = format!(
            "OSNE — {} (NAMUR)",
            i18n::tr(config.language, i18n::Msg::AppSubtitle)
        );
        let mut viewport = eframe::egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 720.0])
            .with_min_inner_size([880.0, 560.0])
            .with_app_id("osne")
            .with_title(title);
        if let Some(icon) = branding::window_icon() {
            viewport = viewport.with_icon(icon);
        }
        let options = eframe::NativeOptions {
            viewport,
            ..Default::default()
        };
        let result = eframe::run_native(
            "mock_bin_su_namur",
            options,
            Box::new(move |_cc| {
                Ok(Box::new(StirrerGui::new(
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
        log::info!("Headless mode — NAMUR server running. Stop with Ctrl-C / SIGTERM.");
        let _actors = (sim_actor, net_actor);
        let _keep = (&snapshot, &allowlist, &status, &trace, &config, &config_path);
        runtime.block_on(std::future::pending::<()>());
    }

    drop(runtime);
    Ok(())
}
