# Design — Simulated process regulator (RU/OPC UA)

*🌍 [FR](../fr/conception.md) · **EN** · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

> Crate: `mock_bin_ru_opcua` · Executable: **ru_opcua** (*Regulation Unit over OPC UA*)

Architecture and modeling document. Modeled on the **ORME** regulator
(`mock_bin_ru_modbustcp`): same split into **synchronous business model / ractor
actors / protocol layer / egui GUI**, same invariants. Only the **transport**
changes: **OPC UA** instead of Modbus.

---

## 1. Purpose

Simulate a **process regulator** (PID loop over a first-order thermal process)
and expose it via **OPC UA**, the industrial supervision standard (Industry 4.0).
Unlike ORME (Modbus) and OSNE (NAMUR) — **fieldbus protocols without security** —
OPC UA natively carries authentication, signing and encryption (planned for
Phase 2).

---

## 2. Physical model ([`regulator.rs`](../../src/regulator.rs))

The **process** reuses [`mock_lib_control::FirstOrderProcess`] (shared with
ORME): first-order transfer function with pure delay

```text
PV(s) / U(s) = K · e^(−L·s) / (1 + τ·s)
```

- `PV`: measurement (process unit, e.g. °C);
- `U`: command / output (0-100 %);
- `K`: static gain; `τ`: time constant; `L`: pure delay;
- `ambient`: at-rest value (zero output).

A **PID** ([`mock_lib_control::Pid`], also reused from ORME) drives the
measurement toward the **setpoint** by controlling the output, bounded to
`[0, 100]`. Two modes: **automatic** (the PID computes the output) and **manual**
(forced output). The simulation step is **0.5 s** (slow thermal process).

All writes (network or GUI) are **sanitized** in `Regulator::apply`: non-finite
floats ignored, setpoint bounded, bounds reordered (`min ≤ max`), PID gains
clamped. **Invariant: never call `f32::clamp` with unvalidated bounds** (panics
if `min > max` or `NaN`).

---

## 3. Architecture (actors)

```
GUI (egui) ───Command(cast)──►  SimulationActor ──refresh──► SharedSnapshot ──► GUI
OPC UA server ─Command(cast)─►   (Regulator)    ──refresh──► SharedSnapshot ──► OPC UA reads
```

- **`SimulationActor`** ([`actors/simulation.rs`](../../src/actors/simulation.rs)):
  **sole** owner of the `Regulator`; advances the simulation on a re-armed
  one-shot timer (no detached timer) and publishes a `SharedSnapshot` at each
  step.
- **`OpcuaServerActor`** ([`actors/network.rs`](../../src/actors/network.rs)):
  owns the OPC UA server (tokio task `server.run()`); restartable at runtime
  (`Reconfigure`: rebind if the IP/port changes); keeps the `JoinHandle` (aborted
  at shutdown) and the `ServerHandle` (clean session cancellation); publishes its
  listening status to the GUI.
- **OPC UA server** ([`opcua_server.rs`](../../src/opcua_server.rs)): builds the
  [`async-opcua`](https://crates.io/crates/async-opcua) server, declares the
  address space and wires up the callbacks. **Reads** draw from the
  `SharedSnapshot`; **writes** emit a `Command` toward the `SimulationActor` via a
  non-blocking `cast`.

Like NAMUR (OSNE) and unlike ORME's Modbus, there is **no separate memory table**:
the OPC UA nodes read the shared snapshot directly.

---

## 4. OPC UA stack — technical choices

- **`async-opcua`** (server, `server` feature): a **tokio-native**
  implementation (one task per connection) that fits into the ractor/tokio stack.
  **100 % Rust** crypto (RustCrypto: `rsa`, `aes`, `sha2`, `x509-cert`) — **no
  OpenSSL dependency**, which preserves cross-compilation (Linux/Windows/RPi).
- **Address space**: an in-memory `SimpleNodeManager`; `Variable` nodes organized
  under `Objects` (see [`reference_opcua.md`](reference_opcua.md)).
- **Callbacks**: `add_read_callback` (live value, sampled for subscriptions) and
  `add_write_callback` (routes to the simulation).
- **License**: `async-opcua` is under **MPL-2.0** (the whole OPC UA lineage in
  Rust is). Copyleft **per file**: unmodified use → the CESAM-Lab code stays MIT
  (see the `NOTICE` file at the root).

---

## 5. Security

- **Phase 1b (current state)**: a **single endpoint**, `SecurityPolicy::None`,
  **anonymous** token. No authentication nor encryption: **trusted network
  only**. The GUI shows a permanent **warning banner**. No certificate is
  generated (pure-Rust RSA generation is slow in debug).
- **Phase 2 (planned)**: encrypted endpoints (`Basic256Sha256`), instance
  certificate, user authentication. This is the **differentiator** of OPC UA
  against fieldbus protocols.

---

## 6. Configuration & persistence

`AppConfig` (language / network / process / regulation / update check) serialized
as **TOML** ([`config.rs`](../../src/config.rs)), **sanitized at load**
(`AppConfig::sanitized`: ordered bounds, `τ ≥ 1e-3`, `dead_time ≥ 0`, finite
floats). File: `mock_ru_opcua.toml` (overridable via `MOCK_CONFIG`).

---

## 7. Future directions

- **Phase 2**: OPC UA security (certificates, encryption, auth).
- OPC UA methods (`Reset`, `Autotune`) in addition to variables.
- Typed information model (regulator ObjectType) rather than flat variables.
- Historization / `HistoryRead` on the measurement.
- Promotion of ORME's regulator model into a shared `mock_lib_*` (it is today
  duplicated between ORME and this instrument).
