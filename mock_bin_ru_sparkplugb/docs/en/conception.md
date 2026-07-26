# Design — Sparkplug B Regulator (ORSE)

*🌍 [FR](../fr/conception.md) · **EN** · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

---

## 1. Overview

ORSE reuses the architecture of the other CESAM-Lab instruments: a **synchronous,
testable business model** (PID regulator + process), driven by **`ractor` actors**
on Tokio, and an **`egui` GUI** that reads a shared snapshot. Only the **transport
layer** changes: here, an **MQTT Sparkplug B edge node** (outbound client) instead
of a Modbus/OPC UA server.

```
        Command (cast)                      refresh every step
GUI   ───────────────────►  SimulationActor ───────────────────►  SharedSnapshot (GUI)
NCMD (broker) ───────────►  (Regulator)      ──────────────────►  SharedSnapshot (publication)
NBIRTH/NDATA (broker) ◄──────────────────────  SharedSnapshot
```

## 2. Actors

- **`SimulationActor`** — owns the single [`Regulator`]. Fixed-step loop (`Tick`
  every 0.5 s); applies `Command`s (GUI or NCMD); publishes the snapshot after each
  mutation. Identical to the other instruments.
- **`SparkplugActor`** — owns the **MQTT client** (`rumqttc`) and runs the
  **Sparkplug B lifecycle** in a dedicated tokio task (whose `JoinHandle` is aborted
  on shutdown). A `Reconfigure` message restarts the client if the broker/credentials/
  TLS change.

## 3. Protocol layer

[`sparkplug_node.rs`](../../src/sparkplug_node.rs) is **pure and synchronous** (no
tokio/rumqttc dependency): building the **topics**, the **metrics** table, crafting
the **payloads** (`NBIRTH`/`NDATA`/`NDEATH`), protobuf (de)serialization, mapping
**`NCMD` → commands**, and the `seq` counter. It is the equivalent of ORUE's
`opcua_server.rs`, isolated to be **testable without a broker**.

### Library choices

- **`rumqttc`** — async Tokio MQTT client (Last Will, automatic reconnection, TLS
  via rustls — already in the tree through OPC UA, **without OpenSSL**).
- **`sparkplug-rs`** — Eclipse Tahu protobuf structs (`Payload`/`Metric`/`Value`),
  generated in **100% Rust** (rust-protobuf, **no `protoc`** → clean cross). The
  crate re-exports `protobuf` (runtime), used for `write_to_bytes`/`parse_from_bytes`.
- **Discarded alternative: `srad`** — a high-level Sparkplug edge node framework that
  manages `bdSeq`/`seq`/rebirth itself. Discarded deliberately: we **own** the state
  machine in the network actor to make it explicit and testable (consistency with the
  other instruments).

## 4. Lifecycle & invariants

- **`bdSeq`** incremented at each client (re)start; the **same** value in the
  `NDEATH` Last Will and in a session's `NBIRTH`.
- **`seq`** rolling 0–255, reset to 0 at each `NBIRTH`.
- **`NDEATH`** carried by the **MQTT Last Will**: robust against any link loss.
- **`NDATA` publication** by snapshot **diff** (cadence = simulation step in
  *on-change* mode, or periodic). The snapshot lock is **never** held across an
  `.await`.

## 5. Security posture

- **No IP allowlist** (the instrument is a client, not a server): a deliberately
  **accepted** parity gap with ORME/OSNE.
- **Plaintext MQTT by default** (port 1883) — unencrypted, no network authentication.
  Warning banner in the GUI. Enable **TLS** + credentials to leave a trusted network.
- **Plaintext password** in the TOML — **simulator only**.
- **TOML sanitization** ([`AppConfig::sanitized`](../../src/config.rs)): process/
  PID/bounds finite and ordered, non-empty Sparkplug identifiers, bounded timeouts.
  Every NCMD write is **clamped/sanitized** by `Regulator::apply`.
