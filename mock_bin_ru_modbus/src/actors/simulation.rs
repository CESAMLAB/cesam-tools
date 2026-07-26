//! Acteur de simulation : propriétaire exclusif du [`Regulator`].

use std::time::Duration;

use ractor::concurrency::JoinHandle;
use ractor::{Actor, ActorProcessingErr, ActorRef, MessagingErr};

use crate::regulator::{Command, Regulator, RegulatorConfig};

use super::{SharedMap, SharedSnapshot};

/// Handle du timer one-shot du prochain `Tick` (conservé pour pouvoir l'abandonner
/// proprement à l'arrêt de l'acteur — invariant « pas de timer détaché »).
type TickTimer = JoinHandle<Result<(), MessagingErr<SimulationMsg>>>;

/// Messages acceptés par l'acteur de simulation.
#[derive(Debug)]
pub enum SimulationMsg {
    /// Top d'horloge périodique : avance la simulation d'un pas.
    Tick,
    /// Commande métier (depuis l'IHM ou le serveur Modbus).
    Command(Command),
}

/// Arguments de démarrage de l'acteur.
pub struct SimulationArgs {
    pub config: RegulatorConfig,
    pub snapshot: SharedSnapshot,
    pub map: SharedMap,
}

/// État interne de l'acteur.
pub struct SimulationState {
    regulator: Regulator,
    snapshot: SharedSnapshot,
    map: SharedMap,
    /// Timer du prochain `Tick` (ré-armé à chaque pas, abandonné à `post_stop`).
    timer: Option<TickTimer>,
}

impl SimulationState {
    /// Publie l'état courant dans les structures partagées (IHM + Modbus).
    ///
    /// ⚠️ Les verrous (`std::sync::Mutex`) ne sont jamais tenus au-delà d'un `.await`
    /// (cette fonction est purement synchrone).
    fn publish(&self) {
        let snap = self.regulator.snapshot();
        if let Ok(mut guard) = self.snapshot.lock() {
            *guard = snap;
        }
        if let Ok(mut guard) = self.map.lock() {
            guard.refresh_from(&snap);
        }
    }

    /// (Ré)arme le prochain `Tick` via un timer one-shot dont on conserve le
    /// handle. On évite ainsi un timer détaché (cf. invariants du projet) et on
    /// obtient un cadencement « auto-régulé » (le tick suivant est planifié après
    /// le traitement du précédent).
    fn arm_timer(&mut self, myself: &ActorRef<SimulationMsg>) {
        let period = Duration::from_secs_f32(self.regulator.dt());
        self.timer = Some(myself.send_after(period, || SimulationMsg::Tick));
    }
}

/// Acteur pilotant la boucle de régulation à fréquence fixe.
pub struct SimulationActor;

impl Actor for SimulationActor {
    type Msg = SimulationMsg;
    type State = SimulationState;
    type Arguments = SimulationArgs;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let regulator = Regulator::new(args.config);
        let mut state = SimulationState {
            regulator,
            snapshot: args.snapshot,
            map: args.map,
            timer: None,
        };
        // Publie l'état initial puis arme le premier tick de simulation.
        state.publish();
        state.arm_timer(&myself);
        Ok(state)
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            SimulationMsg::Tick => {
                state.regulator.step();
                // Ré-arme le prochain tick (le timer précédent vient d'expirer).
                state.arm_timer(&myself);
            }
            SimulationMsg::Command(cmd) => state.regulator.apply(cmd),
        }
        state.publish();
        Ok(())
    }

    async fn post_stop(
        &self,
        _myself: ActorRef<Self::Msg>,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        // Abandonne le timer en attente pour ne pas laisser de tâche détachée.
        if let Some(timer) = state.timer.take() {
            timer.abort();
        }
        Ok(())
    }
}
