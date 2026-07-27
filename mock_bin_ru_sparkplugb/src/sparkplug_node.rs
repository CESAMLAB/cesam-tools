//! Couche **Sparkplug B** : construction des topics, des charges utiles (`Payload`
//! protobuf Eclipse Tahu) et décodage des commandes `NCMD`.
//!
//! Tout ici est **pur et synchrone** (aucune dépendance à tokio/rumqttc) afin
//! d'être testable **sans broker** : l'IO MQTT vit dans l'acteur réseau
//! ([`crate::actors::network`]). C'est l'équivalent Sparkplug du `opcua_server.rs`
//! des autres instruments : table de métriques + (dé)sérialisation.
//!
//! Cycle de vie côté edge node (rappel) :
//! - `NBIRTH` (seq=0) : **toutes** les métriques + `bdSeq` + `Node Control/Rebirth` ;
//! - `NDATA` (seq roulant) : seules les métriques **modifiées** ;
//! - `NDEATH` : `bdSeq` **seul**, **sans** `seq` (porté par le Last Will MQTT) ;
//! - `NCMD` : écritures clients → [`Command`] / [`NodeAction::Rebirth`].

use sparkplug_rs::payload::{metric::Value, Metric};
use sparkplug_rs::protobuf::Message;
use sparkplug_rs::{NodeMessageType, Payload, TopicNamespace};

use mock_lib_regulator::{Command, Snapshot};

/// Codes `DataType` Sparkplug B (Eclipse Tahu) utilisés ici. Le `.proto` stocke le
/// type en `uint32` brut ; ces constantes en fixent la sémantique.
pub mod datatype {
    /// `Float` (32 bits).
    pub const FLOAT: u32 = 9;
    /// `Boolean`.
    pub const BOOLEAN: u32 = 11;
    /// `UInt64` (utilisé pour `bdSeq`).
    pub const UINT64: u32 = 8;
}

/// Nom de la métrique de contrôle « renaissance » (resynchronisation SCADA).
pub const REBIRTH_METRIC: &str = "Node Control/Rebirth";
/// Nom de la métrique de séquence naissance/mort.
pub const BDSEQ_METRIC: &str = "bdSeq";

/// Action déduite d'un message `NCMD` reçu.
#[derive(Debug, Clone, PartialEq)]
pub enum NodeAction {
    /// Commande métier à transmettre à l'acteur de simulation.
    Command(Command),
    /// Demande de **renaissance** (republier un `NBIRTH`).
    Rebirth,
}

// --- Topics --------------------------------------------------------------------

/// Topic d'un message **nœud** : `spBv1.0/<group>/<type>/<node>`.
#[must_use]
pub fn node_topic(group: &str, msg_type: &NodeMessageType, node: &str) -> String {
    format!("{}/{}/{}/{}", TopicNamespace::SPBV1_0.to_string(), group, msg_type.to_string(), node)
}

/// Topic d'abonnement aux commandes `NCMD` adressées à ce nœud.
#[must_use]
pub fn ncmd_topic(group: &str, node: &str) -> String {
    node_topic(group, &NodeMessageType::NCMD, node)
}

// --- Construction de métriques -------------------------------------------------

pub(crate) fn float_metric(name: &str, value: f32) -> Metric {
    Metric {
        name: Some(name.to_string()),
        datatype: Some(datatype::FLOAT),
        value: Some(Value::FloatValue(value)),
        ..Default::default()
    }
}

pub(crate) fn bool_metric(name: &str, value: bool) -> Metric {
    Metric {
        name: Some(name.to_string()),
        datatype: Some(datatype::BOOLEAN),
        value: Some(Value::BooleanValue(value)),
        ..Default::default()
    }
}

fn long_metric(name: &str, value: u64) -> Metric {
    Metric {
        name: Some(name.to_string()),
        datatype: Some(datatype::UINT64),
        value: Some(Value::LongValue(value)),
        ..Default::default()
    }
}

/// Métriques **de données** (sans `bdSeq` ni contrôle) reflétant l'instantané.
///
/// `ProcessValue`/`Output` sont en lecture seule ; les autres sont pilotables via
/// `NCMD` (cf. [`ncmd_to_actions`]) ou simplement observables.
fn data_metrics(snap: &Snapshot) -> Vec<Metric> {
    vec![
        float_metric("Setpoint", snap.setpoint),
        float_metric("ProcessValue", snap.pv),
        float_metric("Output", snap.output),
        float_metric("ManualOutput", snap.manual_output),
        bool_metric("Run", snap.run),
        bool_metric("Auto", snap.auto),
        float_metric("SetpointMin", snap.sp_min),
        float_metric("SetpointMax", snap.sp_max),
        float_metric("PID/Kp", snap.pid.kp),
        float_metric("PID/Ki", snap.pid.ki),
        float_metric("PID/Kd", snap.pid.kd),
    ]
}

/// Métriques **modifiées** entre deux instantanés (corps d'un `NDATA`).
#[must_use]
pub fn changed_metrics(prev: &Snapshot, cur: &Snapshot) -> Vec<Metric> {
    let mut v = Vec::new();
    if cur.setpoint != prev.setpoint {
        v.push(float_metric("Setpoint", cur.setpoint));
    }
    if cur.pv != prev.pv {
        v.push(float_metric("ProcessValue", cur.pv));
    }
    if cur.output != prev.output {
        v.push(float_metric("Output", cur.output));
    }
    if cur.manual_output != prev.manual_output {
        v.push(float_metric("ManualOutput", cur.manual_output));
    }
    if cur.run != prev.run {
        v.push(bool_metric("Run", cur.run));
    }
    if cur.auto != prev.auto {
        v.push(bool_metric("Auto", cur.auto));
    }
    if cur.sp_min != prev.sp_min {
        v.push(float_metric("SetpointMin", cur.sp_min));
    }
    if cur.sp_max != prev.sp_max {
        v.push(float_metric("SetpointMax", cur.sp_max));
    }
    if cur.pid.kp != prev.pid.kp {
        v.push(float_metric("PID/Kp", cur.pid.kp));
    }
    if cur.pid.ki != prev.pid.ki {
        v.push(float_metric("PID/Ki", cur.pid.ki));
    }
    if cur.pid.kd != prev.pid.kd {
        v.push(float_metric("PID/Kd", cur.pid.kd));
    }
    v
}

// --- Construction de Payloads --------------------------------------------------

/// `NBIRTH` : toutes les métriques + `bdSeq` + `Node Control/Rebirth`, `seq` = `seq`.
#[must_use]
pub fn build_nbirth(snap: &Snapshot, bd_seq: u64, seq: u8, timestamp_ms: u64) -> Payload {
    let mut p = Payload::new();
    p.set_timestamp(timestamp_ms);
    p.set_seq(u64::from(seq));
    p.metrics = data_metrics(snap);
    p.metrics.push(long_metric(BDSEQ_METRIC, bd_seq));
    p.metrics.push(bool_metric(REBIRTH_METRIC, false));
    p
}

/// `NDATA` : métriques modifiées, `seq` roulant.
#[must_use]
pub fn build_ndata(changed: Vec<Metric>, seq: u8, timestamp_ms: u64) -> Payload {
    let mut p = Payload::new();
    p.set_timestamp(timestamp_ms);
    p.set_seq(u64::from(seq));
    p.metrics = changed;
    p
}

/// `NDEATH` : `bdSeq` **seul**, **sans** `seq` (déposé en Last Will MQTT).
#[must_use]
pub fn build_ndeath(bd_seq: u64) -> Payload {
    let mut p = Payload::new();
    p.metrics = vec![long_metric(BDSEQ_METRIC, bd_seq)];
    p
}

// --- (Dé)sérialisation ---------------------------------------------------------

/// Sérialise un `Payload` en octets protobuf (corps du message MQTT).
#[must_use]
pub fn encode(payload: &Payload) -> Vec<u8> {
    payload.write_to_bytes().unwrap_or_default()
}

/// Décode un `Payload` depuis des octets protobuf.
pub fn decode(bytes: &[u8]) -> Result<Payload, sparkplug_rs::protobuf::Error> {
    Payload::parse_from_bytes(bytes)
}

// --- NCMD → actions ------------------------------------------------------------

/// Extrait un `f32` d'une valeur de métrique numérique (tolère les types entiers).
fn as_f32(value: &Value) -> Option<f32> {
    match value {
        Value::FloatValue(f) => Some(*f),
        Value::DoubleValue(d) => Some(*d as f32),
        Value::IntValue(i) => Some(*i as f32),
        Value::LongValue(l) => Some(*l as f32),
        _ => None,
    }
}

/// Traduit un `Payload` `NCMD` en actions. Les métriques **non pilotables** ou de
/// **mauvais type** sont **ignorées** (jamais d'erreur, jamais de panique).
///
/// Surface pilotable (parité avec les écritures OPC UA de l'instrument ORUE) :
/// `Setpoint`, `ManualOutput`, `Run`, `Auto`, plus `Node Control/Rebirth`. Les
/// bornes de consigne et les gains PID sont **publiés** mais réglés via l'IHM/TOML.
#[must_use]
pub fn ncmd_to_actions(payload: &Payload) -> Vec<NodeAction> {
    let mut actions = Vec::new();
    for m in &payload.metrics {
        let (Some(name), Some(value)) = (m.name.as_deref(), m.value.as_ref()) else {
            continue;
        };
        let action = match name {
            "Setpoint" => as_f32(value).map(|v| NodeAction::Command(Command::SetSetpoint(v))),
            "ManualOutput" => as_f32(value).map(|v| NodeAction::Command(Command::SetManualOutput(v))),
            "Run" => match value {
                Value::BooleanValue(b) => Some(NodeAction::Command(Command::SetRun(*b))),
                _ => None,
            },
            "Auto" => match value {
                Value::BooleanValue(b) => Some(NodeAction::Command(Command::SetAuto(*b))),
                _ => None,
            },
            REBIRTH_METRIC => match value {
                Value::BooleanValue(true) => Some(NodeAction::Rebirth),
                _ => None,
            },
            _ => None,
        };
        if let Some(a) = action {
            actions.push(a);
        }
    }
    actions
}

// --- Compteur de séquence ------------------------------------------------------

/// Compteur `seq` Sparkplug B : roulant **0–255**. Remis à 0 à chaque `NBIRTH`,
/// incrémenté à chaque message suivant.
#[derive(Debug, Default)]
pub struct SeqCounter(u8);

impl SeqCounter {
    /// Renvoie la valeur courante puis incrémente (avec repli 255 → 0).
    pub fn next(&mut self) -> u8 {
        let v = self.0;
        self.0 = self.0.wrapping_add(1);
        v
    }

    /// Remet le compteur à zéro (à utiliser avant un `NBIRTH`).
    pub fn reset(&mut self) {
        self.0 = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mock_lib_regulator::{Regulator, RegulatorConfig};

    fn sample() -> Snapshot {
        Regulator::new(RegulatorConfig::default()).snapshot()
    }

    fn metric_named<'a>(p: &'a Payload, name: &str) -> Option<&'a Metric> {
        p.metrics.iter().find(|m| m.name.as_deref() == Some(name))
    }

    #[test]
    fn topics_follow_spec() {
        assert_eq!(node_topic("CESAM", &NodeMessageType::NBIRTH, "RU1"), "spBv1.0/CESAM/NBIRTH/RU1");
        assert_eq!(ncmd_topic("CESAM", "RU1"), "spBv1.0/CESAM/NCMD/RU1");
    }

    #[test]
    fn nbirth_contains_all_metrics_and_bdseq_with_seq_zero() {
        let p = build_nbirth(&sample(), 7, 0, 1234);
        assert!(p.has_seq() && p.seq() == 0, "NBIRTH doit porter seq=0");
        for name in [
            "Setpoint", "ProcessValue", "Output", "ManualOutput", "Run", "Auto",
            "SetpointMin", "SetpointMax", "PID/Kp", "PID/Ki", "PID/Kd", BDSEQ_METRIC, REBIRTH_METRIC,
        ] {
            assert!(metric_named(&p, name).is_some(), "métrique manquante: {name}");
        }
        let bd = metric_named(&p, BDSEQ_METRIC).unwrap();
        assert_eq!(bd.value, Some(Value::LongValue(7)));
    }

    #[test]
    fn ndeath_has_bdseq_only_and_no_seq() {
        let p = build_ndeath(42);
        assert!(!p.has_seq(), "NDEATH ne doit pas porter de seq");
        assert_eq!(p.metrics.len(), 1);
        assert_eq!(p.metrics[0].name.as_deref(), Some(BDSEQ_METRIC));
        assert_eq!(p.metrics[0].value, Some(Value::LongValue(42)));
    }

    #[test]
    fn payload_encode_decode_round_trip() {
        let p = build_nbirth(&sample(), 3, 0, 9999);
        let bytes = encode(&p);
        assert!(!bytes.is_empty());
        let back = decode(&bytes).expect("décodage");
        assert_eq!(back.seq(), p.seq());
        assert_eq!(back.metrics.len(), p.metrics.len());
        assert_eq!(metric_named(&back, "Setpoint").unwrap().value, metric_named(&p, "Setpoint").unwrap().value);
    }

    #[test]
    fn changed_metrics_reports_only_diffs() {
        let prev = sample();
        let mut cur = prev;
        cur.pv = prev.pv + 1.0;
        cur.run = !prev.run;
        let changed = changed_metrics(&prev, &cur);
        let names: Vec<_> = changed.iter().filter_map(|m| m.name.clone()).collect();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"ProcessValue".to_string()));
        assert!(names.contains(&"Run".to_string()));
    }

    #[test]
    fn ncmd_maps_writable_metrics_to_commands() {
        let mut p = Payload::new();
        p.metrics = vec![
            float_metric("Setpoint", 80.0),
            bool_metric("Run", true),
            bool_metric("Auto", false),
            float_metric("ManualOutput", 25.0),
        ];
        let actions = ncmd_to_actions(&p);
        assert_eq!(actions, vec![
            NodeAction::Command(Command::SetSetpoint(80.0)),
            NodeAction::Command(Command::SetRun(true)),
            NodeAction::Command(Command::SetAuto(false)),
            NodeAction::Command(Command::SetManualOutput(25.0)),
        ]);
    }

    #[test]
    fn ncmd_rebirth_true_triggers_rebirth_false_ignored() {
        let mut p = Payload::new();
        p.metrics = vec![bool_metric(REBIRTH_METRIC, true)];
        assert_eq!(ncmd_to_actions(&p), vec![NodeAction::Rebirth]);

        let mut p2 = Payload::new();
        p2.metrics = vec![bool_metric(REBIRTH_METRIC, false)];
        assert!(ncmd_to_actions(&p2).is_empty());
    }

    #[test]
    fn ncmd_rejects_wrong_datatype_and_unknown_metrics() {
        let mut p = Payload::new();
        // `Run` reçoit un flottant (mauvais type) → ignoré.
        p.metrics = vec![
            float_metric("Run", 1.0),
            bool_metric("ProcessValue", true), // lecture seule + mauvais type → ignoré
            float_metric("Inconnue", 3.0),     // métrique inconnue → ignorée
        ];
        assert!(ncmd_to_actions(&p).is_empty());
    }

    #[test]
    fn seq_counter_wraps_255_to_0() {
        let mut c = SeqCounter::default();
        assert_eq!(c.next(), 0);
        assert_eq!(c.next(), 1);
        for _ in 2..=255 {
            c.next();
        }
        // 256 appels effectués (0..=255) → le prochain repasse à 0.
        assert_eq!(c.next(), 0);
        c.reset();
        assert_eq!(c.next(), 0);
    }
}
