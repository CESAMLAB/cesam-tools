//! Acteur de simulation : propriétaire exclusif du [`Stirrer`].

use std::time::Duration;

use ractor::concurrency::JoinHandle;
use ractor::{Actor, ActorProcessingErr, ActorRef, MessagingErr};

use crate::stirrer::{Command, Stirrer, StirrerConfig};

use super::SharedSnapshot;

/// Handle du timer one-shot du prochain `Tick` (conservé pour l'abandonner à l'arrêt).
type TickTimer = JoinHandle<Result<(), MessagingErr<SimulationMsg>>>;

/// Messages acceptés par l'acteur de simulation.
#[derive(Debug)]
pub enum SimulationMsg {
    /// Top d'horloge périodique : avance la simulation d'un pas.
    Tick,
    /// Commande métier (depuis l'IHM ou le serveur NAMUR).
    Command(Command),
}

/// Arguments de démarrage de l'acteur.
pub struct SimulationArgs {
    pub config: StirrerConfig,
    pub snapshot: SharedSnapshot,
}

/// État interne de l'acteur.
pub struct SimulationState {
    stirrer: Stirrer,
    snapshot: SharedSnapshot,
    timer: Option<TickTimer>,
}

impl SimulationState {
    /// Publie l'état courant dans l'instantané partagé (IHM + NAMUR).
    ///
    /// ⚠️ Le verrou n'est jamais tenu au-delà d'un `.await` (fonction synchrone).
    fn publish(&self) {
        let snap = self.stirrer.snapshot();
        if let Ok(mut guard) = self.snapshot.lock() {
            *guard = snap;
        }
    }

    /// (Ré)arme le prochain `Tick` (timer one-shot dont on conserve le handle).
    fn arm_timer(&mut self, myself: &ActorRef<SimulationMsg>) {
        let period = Duration::from_secs_f32(self.stirrer.dt());
        self.timer = Some(myself.send_after(period, || SimulationMsg::Tick));
    }
}

/// Acteur pilotant la boucle de simulation à fréquence fixe.
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
        let mut state = SimulationState {
            stirrer: Stirrer::new(args.config),
            snapshot: args.snapshot,
            timer: None,
        };
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
                state.stirrer.step();
                state.arm_timer(&myself);
            }
            SimulationMsg::Command(cmd) => state.stirrer.apply(cmd),
        }
        state.publish();
        Ok(())
    }

    async fn post_stop(
        &self,
        _myself: ActorRef<Self::Msg>,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        if let Some(timer) = state.timer.take() {
            timer.abort();
        }
        Ok(())
    }
}
