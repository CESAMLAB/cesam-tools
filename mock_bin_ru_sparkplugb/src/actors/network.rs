//! Acteur réseau : possède le **client MQTT** et exécute le cycle de vie
//! **Sparkplug B** de l'edge node (NBIRTH / NDATA / NCMD / NDEATH).
//!
//! Le client `rumqttc` se **connecte en sortie** au broker ; sa boucle d'événements
//! (`EventLoop`) tourne dans une tâche tokio dédiée dont l'acteur conserve le
//! `JoinHandle` (abandon à l'arrêt). Une reconfiguration (broker/identifiants/TLS…)
//! relance le client ; les autres réglages (procédé, PID) passent par l'acteur de
//! simulation.
//!
//! **NDEATH** est délivré par le **Last Will** MQTT : à la perte du lien (arrêt,
//! reconfiguration, panne réseau), le broker publie automatiquement le `NDEATH`
//! déposé à la connexion. On n'émet donc pas de `NDEATH` explicite.
//!
//! **bdSeq** est incrémenté à chaque (re)démarrage du client (`restart`) ; le Last
//! Will et le `NBIRTH` d'une même session portent **la même** valeur (invariant
//! Sparkplug). Les reconnexions automatiques internes à `rumqttc` conservent la
//! valeur de la session.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ractor::{Actor, ActorProcessingErr, ActorRef};
use rumqttc::{AsyncClient, Event, EventLoop, LastWill, MqttOptions, Packet, QoS, Transport};
use sparkplug_rs::NodeMessageType;
use tokio::task::JoinHandle;

use crate::config::{NetworkConfig, ServerStatus};
use crate::regulator::{Snapshot, DEFAULT_DT};
use crate::sparkplug_node as spb;

use super::{SharedSnapshot, SharedStatus, SimulationMsg};

/// Capacité du canal de requêtes du client MQTT.
const CLIENT_CAP: usize = 32;

/// Messages de l'acteur réseau.
#[derive(Debug)]
pub enum SparkplugMsg {
    /// Applique une nouvelle configuration réseau (relance le client si le broker,
    /// les identifiants, TLS ou les topics changent).
    Reconfigure { network: NetworkConfig },
}

/// Arguments de démarrage de l'acteur réseau.
pub struct SparkplugArgs {
    pub network: NetworkConfig,
    pub sim: ActorRef<SimulationMsg>,
    pub snapshot: SharedSnapshot,
    pub status: SharedStatus,
}

/// État interne de l'acteur réseau.
pub struct SparkplugState {
    network: NetworkConfig,
    sim: ActorRef<SimulationMsg>,
    snapshot: SharedSnapshot,
    status: SharedStatus,
    task: Option<JoinHandle<()>>,
    client: Option<AsyncClient>,
    bd_seq: u64,
}

impl SparkplugState {
    fn set_status(&self, status: ServerStatus) {
        if let Ok(mut s) = self.status.lock() {
            *s = status;
        }
    }

    /// Arrête le client courant (abandon de la tâche + fermeture du lien). La
    /// fermeture non propre fait publier le **NDEATH** (Last Will) par le broker.
    fn stop_current(&mut self) {
        if let Some(task) = self.task.take() {
            task.abort();
        }
        // Le drop du client ferme le canal de la boucle d'événements.
        self.client = None;
    }

    /// (Re)démarre le client MQTT / edge node Sparkplug B selon la configuration.
    fn restart(&mut self) {
        self.stop_current();
        self.bd_seq = self.bd_seq.wrapping_add(1);
        let label = self.network.endpoint_label();

        let options = build_options(&self.network, self.bd_seq);
        let (client, eventloop) = AsyncClient::new(options, CLIENT_CAP);
        self.client = Some(client.clone());

        let params = NodeParams {
            client,
            network: self.network.clone(),
            sim: self.sim.clone(),
            snapshot: self.snapshot.clone(),
            status: self.status.clone(),
            bd_seq: self.bd_seq,
        };
        self.task = Some(tokio::spawn(run_node(eventloop, params)));

        log::info!("Sparkplug B edge node connecting to {label} (bdSeq={})", self.bd_seq);
        self.set_status(ServerStatus {
            connected: false,
            addr: label,
            error: None,
            bd_seq: self.bd_seq,
        });
    }
}

/// Acteur supervisant le cycle de vie de l'edge node Sparkplug B.
pub struct SparkplugActor;

impl Actor for SparkplugActor {
    type Msg = SparkplugMsg;
    type State = SparkplugState;
    type Arguments = SparkplugArgs;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let mut state = SparkplugState {
            network: args.network,
            sim: args.sim,
            snapshot: args.snapshot,
            status: args.status,
            task: None,
            client: None,
            bd_seq: 0,
        };
        state.restart();
        Ok(state)
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            SparkplugMsg::Reconfigure { network } => {
                let changed = network != state.network;
                state.network = network;
                if changed {
                    state.restart();
                }
            }
        }
        Ok(())
    }

    async fn post_stop(
        &self,
        _myself: ActorRef<Self::Msg>,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        state.stop_current();
        Ok(())
    }
}

/// Construit les options MQTT (identifiants, TLS, keepalive, Last Will = NDEATH).
fn build_options(net: &NetworkConfig, bd_seq: u64) -> MqttOptions {
    let mut o = MqttOptions::new(net.client_id.clone(), net.broker_host.clone(), net.broker_port);
    o.set_keep_alive(Duration::from_secs(u64::from(net.keepalive_secs)));
    if net.has_user() {
        o.set_credentials(net.username.clone(), net.password.clone());
    }
    if net.tls {
        o.set_transport(Transport::tls_with_default_config());
    }
    let will_topic = spb::node_topic(&net.group_id, &NodeMessageType::NDEATH, &net.edge_node_id);
    let will_payload = spb::encode(&spb::build_ndeath(bd_seq));
    o.set_last_will(LastWill::new(will_topic, will_payload, QoS::AtLeastOnce, false));
    o
}

/// Paramètres figés transmis à la tâche cycle de vie.
struct NodeParams {
    client: AsyncClient,
    network: NetworkConfig,
    sim: ActorRef<SimulationMsg>,
    snapshot: SharedSnapshot,
    status: SharedStatus,
    bd_seq: u64,
}

/// Horodatage Unix en millisecondes (champ `timestamp` des payloads Sparkplug).
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or(0)
}

/// Lit l'instantané partagé (jamais de verrou tenu à travers un `.await`).
fn current_snapshot(snapshot: &SharedSnapshot) -> Snapshot {
    match snapshot.lock() {
        Ok(g) => *g,
        // Verrou empoisonné (ne devrait pas arriver) : instantané neutre.
        Err(_) => crate::regulator::Regulator::default().snapshot(),
    }
}

/// Tâche cycle de vie : publie NBIRTH/NDATA, traite NCMD, suit l'état de connexion.
async fn run_node(mut eventloop: EventLoop, p: NodeParams) {
    let nbirth_topic = spb::node_topic(&p.network.group_id, &NodeMessageType::NBIRTH, &p.network.edge_node_id);
    let ndata_topic = spb::node_topic(&p.network.group_id, &NodeMessageType::NDATA, &p.network.edge_node_id);
    let ncmd_topic = spb::ncmd_topic(&p.network.group_id, &p.network.edge_node_id);

    let period = if p.network.publish_on_change {
        Duration::from_secs_f32(DEFAULT_DT)
    } else {
        Duration::from_secs(u64::from(p.network.publish_period_secs))
    };
    let mut ticker = tokio::time::interval(period);

    let mut seq = spb::SeqCounter::default();
    let mut last = current_snapshot(&p.snapshot);
    let mut connected = false;

    loop {
        tokio::select! {
            event = eventloop.poll() => match event {
                Ok(Event::Incoming(Packet::ConnAck(_))) => {
                    // Naissance du nœud : NBIRTH (seq=0) puis abonnement aux NCMD.
                    seq.reset();
                    let snap = current_snapshot(&p.snapshot);
                    publish(&p.client, &nbirth_topic, spb::build_nbirth(&snap, p.bd_seq, seq.next(), now_ms())).await;
                    if let Err(e) = p.client.subscribe(ncmd_topic.clone(), QoS::AtLeastOnce).await {
                        log::warn!("NCMD subscribe failed: {e}");
                    }
                    last = snap;
                    connected = true;
                    if let Ok(mut s) = p.status.lock() {
                        s.connected = true;
                        s.error = None;
                        s.bd_seq = p.bd_seq;
                    }
                    log::info!("Sparkplug B NBIRTH published on {nbirth_topic}");
                }
                Ok(Event::Incoming(Packet::Publish(pkt))) => {
                    if pkt.topic == ncmd_topic {
                        match spb::decode(&pkt.payload) {
                            Ok(payload) => {
                                for action in spb::ncmd_to_actions(&payload) {
                                    match action {
                                        spb::NodeAction::Command(cmd) => {
                                            let _ = p.sim.cast(SimulationMsg::Command(cmd));
                                        }
                                        spb::NodeAction::Rebirth => {
                                            seq.reset();
                                            let snap = current_snapshot(&p.snapshot);
                                            publish(&p.client, &nbirth_topic, spb::build_nbirth(&snap, p.bd_seq, seq.next(), now_ms())).await;
                                            last = snap;
                                            log::info!("Sparkplug B rebirth requested → NBIRTH republished");
                                        }
                                    }
                                }
                            }
                            Err(e) => log::warn!("NCMD payload decode failed: {e}"),
                        }
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    if connected {
                        log::warn!("Sparkplug B MQTT link lost: {e}");
                    }
                    connected = false;
                    if let Ok(mut s) = p.status.lock() {
                        s.connected = false;
                        s.error = Some(e.to_string());
                    }
                    // Évite une boucle serrée tant que la connexion échoue.
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            },
            _ = ticker.tick() => {
                if connected {
                    let snap = current_snapshot(&p.snapshot);
                    let changed = spb::changed_metrics(&last, &snap);
                    if !changed.is_empty() {
                        publish(&p.client, &ndata_topic, spb::build_ndata(changed, seq.next(), now_ms())).await;
                        last = snap;
                    }
                }
            }
        }
    }
}

/// Publie un payload Sparkplug (QoS 1, non retenu) ; journalise un éventuel échec.
async fn publish(client: &AsyncClient, topic: &str, payload: sparkplug_rs::Payload) {
    let bytes = spb::encode(&payload);
    if let Err(e) = client.publish(topic, QoS::AtLeastOnce, false, bytes).await {
        log::warn!("Sparkplug B publish on {topic} failed: {e}");
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use ractor::Actor;

    use super::*;
    use crate::actors::{SimulationActor, SimulationArgs};
    use crate::config::{NetworkConfig, ServerStatus};
    use crate::regulator::{Regulator, RegulatorConfig};

    /// Démarrage de l'acteur **sans broker** : le statut s'initialise (déconnecté,
    /// bdSeq attribué) sans paniquer. Ne valide pas la connexion.
    #[tokio::test]
    async fn actor_starts_without_broker_and_sets_status() {
        let cfg = RegulatorConfig::default();
        let snapshot = Arc::new(Mutex::new(Regulator::new(cfg).snapshot()));
        let status = Arc::new(Mutex::new(ServerStatus::default()));

        let (sim, _sj) = Actor::spawn(None, SimulationActor, SimulationArgs {
            config: cfg,
            snapshot: snapshot.clone(),
        })
        .await
        .unwrap();

        // Broker volontairement injoignable : on ne teste que l'initialisation.
        let network = NetworkConfig {
            broker_host: "127.0.0.1".to_string(),
            broker_port: 1,
            ..NetworkConfig::default()
        };
        let (net, _nj) = Actor::spawn(None, SparkplugActor, SparkplugArgs {
            network,
            sim: sim.clone(),
            snapshot,
            status: status.clone(),
        })
        .await
        .unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;
        let st = status.lock().unwrap().clone();
        assert!(!st.connected, "pas de broker → non connecté");
        assert_eq!(st.bd_seq, 1, "bdSeq attribué à la 1re session");
        assert!(!st.addr.is_empty());

        net.stop(None);
        sim.stop(None);
    }

    /// Round-trip de bout en bout **avec un broker MQTT local** (mosquitto sur
    /// `localhost:1883`). Vérifie NBIRTH reçu par un client « SCADA », puis qu'un
    /// NCMD (`Setpoint`, `Run`) est appliqué et reflété par un NDATA ultérieur.
    ///
    /// **Ignoré par défaut** (CI sans broker). À lancer :
    /// `cargo test -p mock_bin_ru_sparkplugb -- --ignored`.
    #[tokio::test]
    #[ignore = "requiert un broker MQTT local (mosquitto sur :1883)"]
    async fn edge_node_round_trip_with_broker() {
        use rumqttc::Event;

        let cfg = RegulatorConfig::default();
        let snapshot = Arc::new(Mutex::new(Regulator::new(cfg).snapshot()));
        let status = Arc::new(Mutex::new(ServerStatus::default()));
        let (sim, _sj) = Actor::spawn(None, SimulationActor, SimulationArgs {
            config: cfg,
            snapshot: snapshot.clone(),
        })
        .await
        .unwrap();

        let network = NetworkConfig {
            broker_host: "localhost".to_string(),
            broker_port: 1883,
            group_id: "CESAMTEST".to_string(),
            edge_node_id: "RUTEST".to_string(),
            ..NetworkConfig::default()
        };
        let (net, _nj) = Actor::spawn(None, SparkplugActor, SparkplugArgs {
            network: network.clone(),
            sim: sim.clone(),
            snapshot,
            status,
        })
        .await
        .unwrap();

        // Client « SCADA » : s'abonne à tout le groupe et émet un NCMD.
        let mut opts = MqttOptions::new("scada-test", "localhost", 1883);
        opts.set_keep_alive(Duration::from_secs(5));
        let (scada, mut scada_loop) = AsyncClient::new(opts, CLIENT_CAP);
        scada
            .subscribe(format!("spBv1.0/{}/#", network.group_id), QoS::AtLeastOnce)
            .await
            .unwrap();

        let nbirth_topic = spb::node_topic(&network.group_id, &NodeMessageType::NBIRTH, &network.edge_node_id);
        let ndata_topic = spb::node_topic(&network.group_id, &NodeMessageType::NDATA, &network.edge_node_id);
        let ncmd_topic = spb::ncmd_topic(&network.group_id, &network.edge_node_id);

        let mut got_nbirth = false;
        let mut sent_cmd = false;
        let mut got_ndata_setpoint = false;
        let deadline = tokio::time::Instant::now() + Duration::from_secs(15);

        while tokio::time::Instant::now() < deadline && !got_ndata_setpoint {
            let ev = tokio::select! {
                e = scada_loop.poll() => e,
                () = tokio::time::sleep(Duration::from_secs(2)) => continue,
            };
            let Ok(Event::Incoming(Packet::Publish(pkt))) = ev else { continue };
            if pkt.topic == nbirth_topic {
                got_nbirth = true;
                // Émet un NCMD Setpoint=80 + Run=true.
                if !sent_cmd {
                    let mut cmd = sparkplug_rs::Payload::new();
                    cmd.metrics = vec![
                        spb::float_metric("Setpoint", 80.0),
                        spb::bool_metric("Run", true),
                    ];
                    scada.publish(ncmd_topic.clone(), QoS::AtLeastOnce, false, spb::encode(&cmd)).await.unwrap();
                    sent_cmd = true;
                }
            } else if pkt.topic == ndata_topic {
                if let Ok(payload) = spb::decode(&pkt.payload) {
                    if payload.metrics.iter().any(|m| m.name.as_deref() == Some("Setpoint")) {
                        got_ndata_setpoint = true;
                    }
                }
            }
        }

        assert!(got_nbirth, "NBIRTH attendu du nœud");
        assert!(got_ndata_setpoint, "NDATA reflétant le changement de consigne attendu");

        net.stop(None);
        sim.stop(None);
    }
}
