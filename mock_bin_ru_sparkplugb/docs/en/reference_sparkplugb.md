# MQTT Sparkplug B reference — metrics & lifecycle (RU/Sparkplug B)

*🌍 [FR](../fr/reference_sparkplugb.md) · **EN** · [DE](../de/reference_sparkplugb.md) · [ES](../es/reference_sparkplugb.md) · [IT](../it/reference_sparkplugb.md) · [PT](../pt/reference_sparkplugb.md) · [NL](../nl/reference_sparkplugb.md) · [PL](../pl/reference_sparkplugb.md)*

> Source of truth: [`sparkplug_node.rs`](../../src/sparkplug_node.rs) (topics, metrics
> table, payloads, NCMD mapping). Any change is made **in that file** and reflected
> here.

---

## 1. Role & connection

The instrument is a **Sparkplug B edge node**: it **listens on no port**, it
**connects outbound** to an **external MQTT broker** (mosquitto, EMQX, HiveMQ…) and
publishes the regulator's state. Settings in the `[network]` section of the TOML /
the *Settings* modal:

| Key | Default | Role |
|---|---|---|
| `broker_host` | `localhost` | MQTT broker host |
| `broker_port` | `1883` | port (`8883` for TLS) |
| `client_id` | `ru_spb` | MQTT client identifier |
| `group_id` | `CESAM` | Sparkplug group (`spBv1.0/<group_id>/…`) |
| `edge_node_id` | `RU1` | edge node (`…/<edge_node_id>`) |
| `username` / `password` | *(empty)* | MQTT auth (password **in plaintext**, simulator only) |
| `tls` | `false` | TLS encryption (rustls) to the broker |
| `keepalive_secs` | `30` | MQTT keepalive |
| `publish_on_change` | `true` | `true`: `NDATA` as soon as a metric changes (cadence = simulation step, 0.5 s); `false`: periodic |
| `publish_period_secs` | `5` | periodic cadence when `publish_on_change = false` |

> ⚠️ **Plaintext MQTT by default**: without TLS, traffic is neither encrypted nor
> network-authenticated. Use only on a **trusted network**. The GUI shows a warning
> banner as long as `tls` is disabled.

---

## 2. Namespace (topics)

Namespace `spBv1.0`. Node topics:

```
spBv1.0/<group_id>/NBIRTH/<edge_node_id>
spBv1.0/<group_id>/NDATA/<edge_node_id>
spBv1.0/<group_id>/NDEATH/<edge_node_id>
spBv1.0/<group_id>/NCMD/<edge_node_id>
```

With the default values: `spBv1.0/CESAM/NBIRTH/RU1`, etc.

---

## 3. Metrics table

All data metrics live under the **edge node** (no *device* in this version).
Sparkplug type (Eclipse Tahu): `Float` (9), `Boolean` (11), `UInt64` (8).

| Metric | Type | Read/Write | Snapshot field (read) | NCMD → command (write) |
|---|---|:--:|---|---|
| `Setpoint` | Float | R/W | `setpoint` | `SetSetpoint` |
| `ProcessValue` | Float | R | `pv` | — |
| `Output` | Float | R | `output` | — |
| `ManualOutput` | Float | R/W | `manual_output` | `SetManualOutput` |
| `Run` | Boolean | R/W | `run` | `SetRun` |
| `Auto` | Boolean | R/W | `auto` | `SetAuto` |
| `SetpointMin` | Float | R | `sp_min` | *(set via GUI/TOML)* |
| `SetpointMax` | Float | R | `sp_max` | *(set via GUI/TOML)* |
| `PID/Kp` | Float | R | `pid.kp` | *(set via GUI/TOML)* |
| `PID/Ki` | Float | R | `pid.ki` | *(set via GUI/TOML)* |
| `PID/Kd` | Float | R | `pid.kd` | *(set via GUI/TOML)* |
| `bdSeq` | UInt64 | R | *(session counter)* | — |
| `Node Control/Rebirth` | Boolean | W | — | republishes an `NBIRTH` |

**`NCMD`-controllable surface**: `Setpoint`, `ManualOutput`, `Run`, `Auto`, plus
`Node Control/Rebirth` (parity with the OPC UA writes of the ORUE instrument). The
setpoint bounds and the PID gains are **published** (observable by a SCADA) but are
set via the GUI/TOML. An unknown metric or a **wrong type** in an `NCMD` is
**ignored** (never an error, never an aberrant value: the simulation sanitizes every
write).

---

## 4. Lifecycle

- **`NBIRTH`** — published at each connection (ConnAck). Contains **all** the metrics
  (with values), `bdSeq`, and `Node Control/Rebirth`. `seq = 0`.
- **`NDATA`** — **changed** metrics only, `seq` rolling **0–255**.
- **`NDEATH`** — contains `bdSeq` **alone**, **without** `seq`. Set as the **MQTT Last
  Will** at connection: the **broker** publishes it automatically on link loss
  (shutdown, reconfiguration, failure). No explicit `NDEATH` on the node side.
- **`NCMD`** — subscription `spBv1.0/<group>/NCMD/<node>` (QoS 1) subscribed right
  after the `NBIRTH`. Decoded → commands applied to the simulation.
- **`bdSeq`** — incremented at each client (re)start; the `NDEATH` (Last Will) and the
  `NBIRTH` of the **same session** carry the **same** value (Sparkplug invariant).
  Shown in the GUI (diagnostic).
- **`seq`** — reset to 0 at each `NBIRTH`, incremented (rolling) at each `NDATA`.
- **Rebirth** (`Node Control/Rebirth = true` via `NCMD`) → republication of an
  `NBIRTH` (SCADA resynchronization).

---

## 5. Client example (SCADA)

Subscribe to the whole group, then send a setpoint:

```bash
# Observe the node's messages
mosquitto_sub -h localhost -t 'spBv1.0/CESAM/#' -v

# (the payloads are Sparkplug B protobuf — use a Tahu decoder to read them)
```

An `NCMD` published on `spBv1.0/CESAM/NCMD/RU1` with the metrics `Run=true` and
`Setpoint=80.0` starts the regulation and sets the setpoint; a subsequent `NDATA`
reflects the change.
