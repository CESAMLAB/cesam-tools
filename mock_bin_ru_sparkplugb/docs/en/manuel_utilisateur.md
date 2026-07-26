# User manual — Sparkplug B Regulator (ORSE)

*🌍 [FR](../fr/manuel_utilisateur.md) · **EN** · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

---

## 1. What the instrument is for

**ORSE** simulates a process **regulation unit** (PID + first-order thermal process)
and publishes its state over **MQTT Sparkplug B**, like an **edge node** that
connects to a **broker** and exposes metrics to a SCADA. It is used to test a
Sparkplug B acquisition chain (Ignition, Chariot, EMQX, Node-RED…) without real
hardware.

## 2. Prerequisite: an MQTT broker

As ORSE is a **client**, you need a reachable MQTT broker. Locally:

```bash
docker run -it --rm -p 1883:1883 eclipse-mosquitto
```

## 3. Getting started

```bash
cargo run -p mock_bin_ru_sparkplugb        # GUI + Sparkplug B edge node
```

At startup, the GUI tries to connect to the broker (`localhost:1883` by default).
The header shows the state: **Connected** (green) once the `NBIRTH` is published, or
**Disconnected** (red) with the reason. An orange **⚠ Plaintext MQTT** banner is a
reminder that there is no TLS.

## 4. Interface

- **Header**: title, *Settings* / *Save* buttons, run/stop state, Sparkplug B
  connection state, TLS/plaintext banner.
- **Left panel (Commands)**: *Run/Stop*, *Automatic mode (PID)*, *Setpoint*, *Manual
  output* (manual mode), **PID** tuning (Kp/Ki/Kd).
- **Center panel**: *Measurement / Setpoint / Output* cards + real-time **chart**.
- **Settings modal**: language, update check, **MQTT broker / Sparkplug B** (host,
  port, client_id, group_id, edge_node_id, keepalive, TLS, username/password,
  on-change/periodic publishing), **process** (K, τ, dead time, ambient), **setpoint
  bounds**. *Apply* restarts the connection and saves the TOML.

## 5. Controlling from a SCADA

The SCADA subscribes to `spBv1.0/<group_id>/#` and receives `NBIRTH` then `NDATA`.
To **command** the regulator, it publishes an `NCMD` on
`spBv1.0/<group_id>/NCMD/<edge_node_id>` with the controllable metrics (`Setpoint`,
`Run`, `Auto`, `ManualOutput`) or `Node Control/Rebirth = true` to force a rebirth.
Details: [`reference_sparkplugb.md`](reference_sparkplugb.md).

## 6. FAQ

- **Permanent "Disconnected"** → broker unreachable: check `broker_host`/
  `broker_port`, the firewall, and that the broker is running.
- **The SCADA sees nothing** → check the `group_id`/`edge_node_id` and the
  `spBv1.0/<group>/#` subscription; the payloads are **protobuf** (a Sparkplug
  decoder is required).
- **My NCMD writes are ignored** → non-controllable metric or wrong type (see the
  metrics table). Only `Setpoint`/`Run`/`Auto`/`ManualOutput` and `Rebirth` are
  accepted.
- **Where is the config file?** → `mock_ru_sparkplugb.toml` (current directory;
  overridable via `MOCK_CONFIG`).
