# Design document — Simulated laboratory stirrer (OSNE)

*🌍 [FR](../fr/conception.md) · **EN** · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

> Crate: `mock_bin_su_namur` · Executable: **OSNE** (*Open Stirrer NAMUR Emulator*)

Architecture and modelling document. Modelled on the **ORME** controller
(`mock_bin_ru_modbustcp`): same split into **synchronous business model / ractor
actors / protocol layer / egui GUI**, same invariants.

---

## 1. Objective

Simulate a **laboratory stirrer** (IKA-style) driven by the **NAMUR** serial
protocol. The motor has a **transfer function** (speed dynamics) controlled by a
**fast feedback loop**, and the **viscosity** of the medium is adjustable and
affects torque.

---

## 2. Physical model

### Motor ([`motor.rs`](../../src/motor.rs))

Torque balance, integrated by explicit Euler:

```text
J · dω/dt = T_moteur − k · η · ω − T_frottement
```

- `ω`: speed (tr/min);
- `T_moteur`: motor torque (command, N·cm, ≥ 0);
- `k · η · ω`: **viscous load torque** (∝ viscosity `η` and speed);
- `T_frottement`: residual dry friction;
- `J` (`inertia`): sets the **responsiveness** (small ⇒ fast).

At steady state, `T_moteur = k·η·ω + T_frottement`: the torque needed to hold a
speed **grows with viscosity**. If this torque exceeds the **maximum torque**,
the setpoint can no longer be reached → **overload**.

### Feedback control ([`stirrer.rs`](../../src/stirrer.rs))

A **PID** ([`mock_lib_control::Pid`], reused from ORME) takes the speed error
`setpoint − measurement` and produces the **motor torque**, bounded to
`[0, torque_max]`. The default gains are deliberately "stiff": the output
saturates at maximum torque as long as the error is large (fast ramp-up), then
the integral term stabilizes. The simulation step is **20 ms** (50 Hz), finer
than ORME's because a motor's dynamics are fast.

---

## 3. Architecture (actors)

```
GUI (egui) ──Command(cast)──►  SimulationActor ──refresh──► SharedSnapshot ──► GUI
NAMUR server ──Command(cast)─►   (Stirrer)     ──refresh──► SharedSnapshot ──► NAMUR reads
```

- **`SimulationActor`** ([`actors/simulation.rs`](../../src/actors/simulation.rs)):
  sole owner of the `Stirrer`; advances the simulation on a re-armed one-shot
  timer (no detached timer) and publishes a `SharedSnapshot`.
- **`NamurServerActor`** ([`actors/network.rs`](../../src/actors/network.rs)):
  owns the NAMUR server, hot-restartable (`Reconfigure`); shared IP allowlist;
  listening status published for the GUI.
- **NAMUR server** ([`namur_server.rs`](../../src/namur_server.rs)): reads ASCII
  lines, interprets them ([`namur.rs`](../../src/namur.rs)), answers reads and
  relays writes/actions to the actor. **One master at a time** (point-to-point).
  Per-session **watchdog**.

NAMUR reads draw from the `SharedSnapshot` (no separate memory table like ORME's
Modbus: the NAMUR protocol is "command"-oriented, not "register"-oriented).

---

## 4. Configuration & security

- `AppConfig` (language / serial-network / motor / control) serialized to **TOML**
  ([`config.rs`](../../src/config.rs)), **sanitized on load**
  (`AppConfig::sanitized`: ordered bounds, finite floats) — invariant shared with
  ORME (never `clamp` with unvalidated bounds).
- NAMUR has **neither authentication nor encryption**: trusted network + IP
  allowlist (TCP). Default `0.0.0.0` + empty list ⇒ exposed: the GUI shows a
  **warning banner**.

---

## 5. Possible evolutions

- Direction of rotation (CW/CCW) and acceleration ramp.
- Temperature sensor (`IN_PV_2/3`) if a thermal model is added.
- Non-linear load torque (turbulent regime ∝ ω²).
- Promotion of the motor model into `mock_lib_control` if it serves a second
  instrument.
