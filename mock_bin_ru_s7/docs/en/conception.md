# Design — S7 Regulator (ORSS)

*🌍 [FR](../fr/conception.md) · **EN** · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

---

## 1. Overview

ORSS reuses the architecture of the other CESAM-Lab instruments: a **synchronous,
testable business model** (PID + process), **`ractor` actors** on Tokio, and an
**`egui` GUI** reading a shared snapshot. Only the **transport layer** changes: an
**S7comm server** (ISO-on-TCP / RFC1006) instead of Modbus/OPC UA.

```
        Command (cast)                      refresh each step
GUI   ───────────────────►  SimulationActor ───────────────────►  SharedSnapshot (GUI)
S7 Write Var ────────────►  (Regulator)      ──────────────────►  SharedSnapshot
S7 Read Var  ◄────────────────────────────────  SharedSnapshot (DB1 image)
```

## 2. Actors

- **`SimulationActor`** — owns the single [`Regulator`]. Fixed-step loop; applies
  `Command`s (GUI or S7 writes); publishes the snapshot after each mutation.
- **`S7ServerActor`** — owns the **TCP listen loop**. A dedicated tokio task binds
  the socket and accepts clients; each session is carried by an **internal** `JoinSet`
  (so it is torn down with the loop — no detached task). `Reconfigure` restarts
  listening if the IP/port changes and updates the shared **allowlist**.

## 3. Protocol layer

[`s7_server.rs`](../../src/s7_server.rs) is **pure and synchronous** (no network
dependency): TPKT framing, COTP (CR→CC, DT) and S7comm (Setup, Read Var, Write Var)
over a **DB1 byte image**. Parsing is **bounded** (access via checked `get`/slices):
a malformed frame coming from the network **never** causes a panic, only a missing
response. It is the S7 equivalent of `opcua_server.rs`, isolated to be **testable
without a socket**.

### Why a hand-written server

There is no S7 **server** library in Rust (the `s7`/`s7-comm` crates are
**client**-oriented). The required subset (COTP class 0 + S7 Read/Write Var over a
DB) is compact and well specified: implementing it by hand gives full control and a
testable surface, consistent with the other instruments.

## 4. Session policy

Several **simultaneous** S7 clients are accepted (PLC behavior), unlike the
single-master policy of ORME (eviction) and the point-to-point policy of OSNE
(squat). Each session reads the current DB1 image and routes its writes to the
simulation; "last writer wins", like a real PLC.

## 5. Security posture

- **No authentication or encryption** (S7 "classic"): only the **IP allowlist** and
  the network topology protect access. `0.0.0.0` + empty list = exposed → warning
  banner in the GUI ([`NetworkConfig::is_exposed`](../../src/config.rs)).
- **TOML sanitization** ([`AppConfig::sanitized`](../../src/config.rs)): process/
  PID/bounds finite and ordered. Every S7 write is **clamped/sanitized** by
  `Regulator::apply`: the network surface cannot produce `NaN`/`Inf` or an aberrant
  value.
- **Bounded network parsing**: no frame can cause a panic (see §3).
