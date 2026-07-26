# Design — EtherNet/IP Regulator (OREE)

*🌍 [FR](../fr/conception.md) · **EN** · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

---

## 1. Overview

OREE reuses the architecture of the other CESAM-Lab instruments: a **synchronous,
testable business model** (PID + process), **`ractor` actors** on Tokio, and an
**`egui` GUI** reading a shared snapshot. Only the **transport layer** changes: an
**EtherNet/IP adapter** (encapsulation + CIP) instead of Modbus/OPC UA/S7.

```
        Command (cast)                      refresh each step
GUI   ───────────────────►  SimulationActor ───────────────────►  SharedSnapshot (GUI)
CIP Write Tag ───────────►  (Regulator)      ──────────────────►  SharedSnapshot
CIP Read Tag  ◄────────────────────────────────  SharedSnapshot
```

## 2. Actors

- **`SimulationActor`** — owns the single [`Regulator`]; applies the `Command`s
  (GUI or CIP writes); publishes the snapshot after each mutation.
- **`EipServerActor`** — owns the **TCP listen loop**. A tokio task binds the
  socket and accepts clients; each session (with its *session handle*) is carried
  by an **internal** `JoinSet` (torn down with the loop — no detached task).
  `Reconfigure` restarts the listener if the IP/port changes and updates the
  shared **allowlist**.

## 3. Protocol layer

[`eip_server.rs`](../../src/eip_server.rs) is **pure and synchronous**: EtherNet/IP
encapsulation (`RegisterSession`, `SendRRData`/CPF) and CIP (`Read Tag`/`Write Tag`
by symbolic segment). Everything is **little-endian**. Parsing is **bounded**
(checked slices): a malformed packet from the network **never** causes a panic, only
an absence of response. This is the equivalent of `opcua_server.rs`, isolated to be
**testable without a socket**.

### Why a hand-written adapter

There is no EtherNet/IP **server/adapter** library in Rust (the `rseip`,
`rust-ethernet-ip`, `cip` crates are **client/scanner** oriented). The required
subset (encapsulation + CIP Read/Write Tag on named tags) is compact: implementing
it by hand gives full control and a testable surface, consistent with the other
instruments.

## 4. Session policy

Multiple **simultaneous** clients are accepted (adapter behaviour), unlike ORME's
single-master model. Each session receives a *session handle* and reads the current
snapshot; "last writer wins".

## 5. Security posture

- **No authentication or encryption** (EtherNet/IP "classic"): only the **IP
  allowlist** and the network topology protect access. `0.0.0.0` + empty list =
  exposed → warning banner ([`NetworkConfig::is_exposed`](../../src/config.rs)).
- **TOML sanitization** ([`AppConfig::sanitized`](../../src/config.rs)): process/
  PID/bounds finite and ordered. Every CIP write is **clamped/sanitized** by
  `Regulator::apply`: the network surface cannot produce `NaN`/`Inf` nor an aberrant
  value.
- **Bounded network parsing**: no packet can cause a panic (cf. §3).
