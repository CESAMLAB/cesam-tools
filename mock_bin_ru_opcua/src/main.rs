//! Prototype **Phase 1** : régulateur de procédé simulé exposé en **OPC UA**.
//!
//! Assemble une boucle de simulation (tâche tokio dédiée, propriétaire exclusif du
//! [`Regulator`]) et un serveur OPC UA (sécurité None). L'IHM `egui`, la
//! configuration TOML, l'i18n et l'alignement sur le modèle d'acteurs `ractor`
//! viendront en Phase 1b ; la sécurité OPC UA (certificats, chiffrement, auth) en
//! Phase 2.

mod opcua_server;
mod sim;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use log::{info, warn};

use sim::{Regulator, Snapshot};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // État partagé (lu par les callbacks OPC UA) + canal de commandes.
    let regulator = Regulator::new();
    let shared: Arc<Mutex<Snapshot>> = Arc::new(Mutex::new(regulator.snapshot()));
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    // Boucle de simulation : possède le régulateur, applique les commandes reçues
    // et publie l'instantané à chaque pas.
    let sim_shared = shared.clone();
    let sim_task = tokio::spawn(async move {
        let mut reg = regulator;
        let mut ticker = tokio::time::interval(Duration::from_secs_f32(reg.dt()));
        loop {
            ticker.tick().await;
            while let Ok(cmd) = rx.try_recv() {
                reg.apply(cmd);
            }
            reg.step();
            if let Ok(mut s) = sim_shared.lock() {
                *s = reg.snapshot();
            }
        }
    });

    // Serveur OPC UA : construction, déclaration des nœuds, exécution.
    let (server, handle) = opcua_server::build()?;
    opcua_server::install(&handle, shared.clone(), tx)?;
    info!("OPC UA server starting on opc.tcp://0.0.0.0:4840/ (SecurityPolicy::None)");

    // Arrêt propre sur Ctrl-C (ferme les sessions clientes proprement).
    let shutdown = handle.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            info!("Ctrl-C received — shutting down OPC UA server");
            shutdown.cancel();
        }
    });

    let result = server.run().await;
    sim_task.abort();
    if let Err(e) = result {
        warn!("OPC UA server stopped with error: {e}");
        return Err(anyhow::anyhow!("OPC UA server error: {e}"));
    }
    Ok(())
}
