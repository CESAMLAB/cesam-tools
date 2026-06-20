# Design document — Simulated Modbus TCP controller

*🌍 [FR](../fr/conception.md) · **EN** · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

> Product: **ORME** · Crate: `mock_bin_ru_modbustcp` · Workspace: `cesam-tools` · License: MIT

This document describes the architecture, technical choices and operating
principles of the simulated industrial controller. It is intended for developers
who maintain or extend the project.

---

## 1. Objective and scope

Provide a **virtual industrial instrument**: a process controller that behaves
realistically and communicates over **Modbus TCP** (slave), in order to develop
and test supervisors / PLCs / gateways **without hardware**.

The simulator covers:

- a **physical process** modelled by a transfer function;
- bidirectional **control** (heating / cooling): PID, on/off (TOR) or cycle relay
  (PWM);
- a **Modbus TCP interface** exposing the complete state;
- a **GUI** for control, visualization and configuration;
- the **persistence** of parameters.

Currently out of scope: Modbus RTU, redundancy, long-term historization, strong
authentication (only an IP allowlist is provided).

---

## 2. Overview

```
┌──────────────────────────────────────────────────────────────────────┐
│                       Process (main thread)                            │
│                                                                        │
│   ┌─────────────────────────┐         reads (Mutex)                    │
│   │   GUI  egui / eframe     │◄──────────────── SharedSnapshot         │
│   │   (gui.rs)               │◄──────────────── SharedStatus           │
│   └───────────┬─────────────┘                                          │
│               │ cast (non-blocking)                                    │
└───────────────┼────────────────────────────────────────────────────────┘
                │
   ┌────────────┼──────────── Tokio runtime (background threads) ────────┐
   │            ▼                                                         │
   │   ┌──────────────────┐  refresh  ┌──────────────┐                   │
   │   │ SimulationActor   ├──────────►│ SharedSnapshot│ (GUI)            │
   │   │  (ractor)         ├──────────►│ SharedMap     │ (Modbus)         │
   │   │  owns the          │           └──────┬───────┘                  │
   │   │  Regulator         │◄── Command ──┐    │ reads                   │
   │   └──────────────────┘              │    ▼                          │
   │          ▲ Command (cast)            │  ┌──────────────────────┐     │
   │          │                           └──┤ RegulatorService      │     │
   │   ┌──────┴───────────┐  manage/rebind   │ (trait Service)       │     │
   │   │ ModbusServerActor ├─────────────────►  Modbus TCP server    │◄──── clients
   │   │  (ractor)         │  IP filter ──────► (tokio-modbus)        │     │
   │   └──────────────────┘   (SharedAllowlist)└──────────────────────┘     │
   └────────────────────────────────────────────────────────────────────┘
```

Guiding principle: **a single owner of the business state**. The `Regulator` is
never shared; it lives inside `SimulationActor`. All writes (GUI or Modbus) are
`Command` **messages**. Reads operate on **copies** refreshed at each step
(`SharedSnapshot`, `SharedMap`), which eliminates locks on the logic and race
conditions.

---

## 3. Technical choices

| Need | Choice | Rationale |
|--------|-------|---------------|
| Concurrency | **`ractor`** (actors) on **Tokio** | Isolates the mutable state inside an actor; mutations serialized by messages, without application-level locks. Project preference. |
| Modbus TCP slave | **`tokio-modbus`** (`tcp-server`) | Mature async implementation; the `Service` trait maps request→response cleanly. |
| GUI | **`egui` / `eframe`** + `egui_plot` | Immediate mode, cross-platform, no complex UI state to synchronize. |
| Process | **FOPDT** (1st order + dead time) | Standard model, sufficient for a thermal process; few parameters, intuitive. |
| Persistence | **`serde` + `toml`** | Human-readable/editable format, ideal for device parameters. |

### Why separate synchronous and asynchronous logic

`mock_lib_control` and `regulator.rs` are **purely synchronous** (no IO, no
async). Benefits: deterministically unit-testable, reusable by other instruments,
and easy to review. The asynchronous part is confined to the **actors** and the
**network layer**.

---

## 4. Data model

### Business state (`regulator.rs`)

- `Regulator` — owning aggregate: modes, setpoints, controllers (`Pid`, `OnOff`)
  and process (`FirstOrderProcess`). Not `Clone`, not shared.
- `RegulatorConfig` — static configuration (process, gains, bounds, `dt`).
  **Single source** of default values (the TOML config derives from it).
- `RegulatorSnapshot` — **immutable copy** (`Copy`) of the observable state,
  published at each step. It is the read contract for the GUI and the Modbus
  table.
- `Command` — enumeration of possible mutations (run, mode, setpoints, settings,
  process, bounds).

### Shared structures (`actors/mod.rs`, `config.rs`)

| Type | Content | Written by | Read by |
|------|---------|-----------|--------|
| `SharedSnapshot` | typed `RegulatorSnapshot` | SimulationActor | GUI |
| `SharedMap` | `MemoryMap` (images of the 4 Modbus tables) | SimulationActor | RegulatorService |
| `SharedAllowlist` | `IpFilter` | ModbusServerActor | connection acceptance |
| `SharedStatus` | `ServerStatus` (listening / error) | ModbusServerActor | GUI |

All are `Arc<Mutex<…>>`: **short** critical sections (copy / refresh), never held
during a computation or IO.

---

## 5. Components

### 5.1 `mock_lib_control` (library)

- `Pid` — discrete-time PID, derivative on the error, **anti-windup** by clamping
  the integral term. API: `step(sp, pv, dt)` or `step_with_error(err, dt)`
  (reused for the cooling direction).
- `OnOff` — on/off with **symmetric hysteresis** (dead band) **and
  anti-short-cycle**: a minimum cycle time (`min_cycle`, s) forbids any switching
  as long as the relay has not remained long enough in its state, modelling the
  protection of a real actuator. The relay **latches** its state: it is the caller
  who must pass it the signed error without resetting it on a sign change (see
  § 5.2).
- `Pwm` — pulse-width modulator (**cycle relay** / *time-proportioning*): over a
  fixed period `T_c`, the on/off output is active for the `duty` fraction of the
  cycle (`duty` **sampled once per cycle** to avoid a steady-state bias). Allows
  fine control of an on/off device.
- `FirstOrderProcess` — transfer function `K·e^(-L·s)/(1+T·s)`, Euler integration
  + delay line. `reconfigure(...)` changes the parameters without a jump.
- `ControllerKind` — `Off` / `Pid` / `OnOff` / `Pwm`, with Modbus encoding
  (`to_code`/`from_code`).

### 5.2 `regulator.rs`

Orchestration of control at each step (`step`):

1. if **stopped** → output 0, controllers reset;
2. if **manual** → output = manual setpoint (signed %);
3. if **auto** → the heating contribution (direction 1, error `SP − PV`) and the
   cooling contribution (direction 2, error `PV − SP`) are computed **separately**,
   each ≥ 0, then `output = heating − cooling`:
   - **PID**: output clamped to `[0, 100]` (`out_min = 0`) — the inactive
     direction (negative error) outputs 0 and its integral **purges naturally**
     through clamping. We do **not** force it to zero: with the strong ripple of
     PWM, clearing it at each setpoint crossing would introduce a steady-state
     error;
   - **TOR**: the relay is evaluated on the signed error and keeps its state when
     crossing the setpoint, which restores a **symmetric** hysteresis band
     `[SP − h/2, SP + h/2]` (the heating/cooling bands remain disjoint, so the two
     relays are mutually exclusive);
   - **PWM**: a PID computes the duty cycle, modulated by the cycle relay; the
     physical output is strictly 0 % or 100 %, but its average follows the PID.
4. the output drives the process, which produces the new measurement (PV).

> **History**: before this revision, the heating/cooling switching was done by
> the sign of the error and **reset** the TOR relay when crossing the setpoint —
> which truncated the hysteresis to `[SP − h/2, SP]` (half the band, asymmetric)
> and made the TOR control poor. The separate-direction computation fixes this
> defect.

### 5.3 `actors/simulation.rs`

`SimulationActor` (ractor). `pre_start` arms a `send_interval(dt)` that emits
`Tick`. `handle` processes `Tick` (advances the simulation) and `Command`
(applies a mutation), then **publishes** the state into `SharedSnapshot` and
`SharedMap`.

### 5.4 `actors/network.rs`

`ModbusServerActor` owns the Modbus server. `Reconfigure(NetworkConfig)`:
- updates the shared **allowlist** (immediate effect, without restart);
- if the **transport** (TCP/RTU), the **port / IP** or the **serial parameters**
  change, **stops** the server task and **restarts** it (`start_tcp` or
  `start_rtu`); publishes the state into `SharedStatus` (success or error).

A **single transport** is active at a time (`Transport::Tcp` or `Rtu`). RTU is
behind the **`rtu` feature**; without it, selecting RTU publishes an explicit
status error.

### 5.5 `modbus_server.rs`

`RegulatorService` implements `tokio_modbus::server::Service` **synchronously**
(`future::Ready`): reads = slice of `SharedMap`; writes = decoding into a
`Command` (via `map.rs`) then `cast` to `SimulationActor`.

**Single-master policy.** `serve` (TCP) allows **only one remote master at a
time**: on each new connection (IP allowed by the allowlist), the previous one is
closed. Mechanism: the `TcpStream` is wrapped in a `CancellableStream` which, upon
receiving a `oneshot` signal, returns **EOF on read** — `tokio-modbus`'s
processing loop then terminates and closes the socket. `serve_rtu` (`rtu` feature)
serves the serial bus via `rtu::Server::serve_forever`: the RS485 bus *is* the
unique master (nothing to evict).

> ⚠️ The GUI does not take this path: it sends its `Command`s directly to the
> actor, so it is never counted as a master.
>
> ⚠️ The RTU server of `tokio-modbus` 0.17 does not pass the slave address to the
> service: the device therefore responds whatever address is requested. A
> **point-to-point** link is recommended. `slave_id` is persisted and displayed,
> but not used for filtering (upstream limitation).

### 5.6 `map.rs`

**Source of truth** for the Modbus addressing plan. Address constants, `MemoryMap`
(table images), `refresh_from(snapshot)` (state→registers) and
`*_to_command(s)` (writes→commands). Encoding of `f32` over 2 registers,
big-endian, high word first.

### 5.7 `config.rs`

`AppConfig` (network / process / regulation) ⇄ TOML. `IpFilter` (`*` wildcards per
IPv4 octet). `ServerStatus`. `to_regulator_config()` bridges to the domain.

### 5.8 `gui.rs`

**Single-page** GUI: header (states + buttons), commands panel (left), supervision
+ curve (center), live Modbus table (right), Settings modal. Reads the `Shared*`,
sends `Command`s via non-blocking `cast`.

---

## 6. Scenarios (sequences)

**Modbus read (PV)**: client → `RegulatorService::call(ReadInputRegisters)` →
`SharedMap` read → `Response`. No interaction with the actor (minimal latency).

**Modbus write (setpoint)**: client → `call(WriteMultipleRegisters)` →
`map::holdings_to_commands` → `cast(Command::SetSpAuto)` → the actor applies it at
the next step → republishes `SharedMap`/`SharedSnapshot`.

**GUI command**: interaction → `cast(Command)` → same.

**Network reconfiguration**: *Apply* modal → `cast(Reconfigure)` →
ModbusServerActor rebinds if necessary → `SharedStatus` updated → the GUI header
reflects the state.

**Tick**: timer → `Tick` → `Regulator::step` → publication.

---

## 7. Control theory

**Process (FOPDT)**: `v[k+1] = v[k] + (dt/T)·(target − v[k])`, with
`target = ambient + K·u` and `u` delayed by `L` seconds (delay line).

**PID**: `u = Kp·e + Ki·∫e + Kd·de/dt`, integral clamped to `[out_min, out_max]`
(anti-windup). Derivative on the error (simplicity/heating-cooling-symmetry
trade-off).

**TOR**: active if `e > +H/2`, inactive if `e < −H/2`, otherwise state preserved.

**Bidirectional**: a single direction acts at a time, selected by the sign of the
error; the global output is signed (+ heating / − cooling).

---

## 8. Decisions and trade-offs

- **Dual publication (`Snapshot` + `Map`)** rather than a single structure: the
  GUI manipulates business types, Modbus manipulates raw registers; both remain
  simple and decoupled, at the cost of a slight, negligible copy overhead.
- **Modbus reads without going through the actor**: `SharedMap` is read directly
  to minimize latency; the actor remains the sole **writer**, so no race.
- **Synchronous Modbus service** (`future::Ready`): all the work is non-blocking
  (short lock + cast), no need to box a future.
- **Rebind on port change**: a socket cannot change port; we accept a short
  service interruption on reconfiguration.
- **Derivative on the error** (and not on the measurement): a slight "kick" on a
  setpoint change, accepted to keep the algorithm symmetric and simple.

---

## 9. Possible evolutions

- Modbus RTU / serial (reuse `RegulatorService`, change the transport).
- Setpoint ramp, PID auto-tuning, simulated faults (sensor failure, saturation).
- Historization / CSV export of the trend.
- Switch the GUI to **tabs** if the single page becomes too dense.
- New instruments: create `mock_bin_<name>` and factor the common parts into
  `mock_lib_*` (see [maintenance.md](maintenance.md)).
