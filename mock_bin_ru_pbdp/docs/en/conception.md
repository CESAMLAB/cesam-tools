# Design — Simulated PROFIBUS DP Regulator (ORPD)

*🌍 [FR](../fr/conception.md) · **EN** · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

> Crate: `mock_bin_ru_pbdp` · Executable: **ru_pbdp** (*Regulation Unit over PROFIBUS DP*)

Architecture and modelling document. Modelled on the **ORME** controller
(`mock_bin_ru_modbus`) for the business model and actors, and on **OSNE**
(`mock_bin_su_namur`) for the serial link. Only the **protocol layer**
changes: a **software simulator of PROFIBUS DP-V0 frames**, built from
scratch (no published `profibus`/`profibus-dp` crate exists in the Rust
ecosystem to date).

---

## 1. Purpose

Simulate a **process regulator** (PID loop on a first-order thermal process,
model **identical** to ORME) and expose it through a **PROFIBUS DP-V0 frame
structure** over a serial link (RS-485/RS-232).

**This document assumes the reader has read the non-interoperability
warning** (see [`manuel_utilisateur.md`](manuel_utilisateur.md) and
[`reference_profibus.md`](reference_profibus.md) §6): real PROFIBUS DP
requires bit-level bus timing compliance (slot time, `Tsdr` min/max, a
watchdog in the tens of milliseconds) that only a dedicated ASIC (SPC3/VPC3)
can guarantee. This simulator makes no such claim — it is an educational and
software-testing tool, not a bus driver.

---

## 2. Physical model ([`regulator.rs`](../../src/regulator.rs))

Reused as-is from the ORME controller:
[`mock_lib_control::FirstOrderProcess`] (first-order transfer function with
pure dead time) and [`mock_lib_control::Pid`] (anti-windup PID), with the
same modes (Off/PID/On-off/PWM) on both directions (heat/cool). Simulation
step: **50 ms**. All writes are **sanitized** in `Regulator::apply` (bounds
reordered, non-finite floats ignored, PID gains clamped) — the same invariant
as everywhere else in the workspace: never call `f32::clamp` with unvalidated
bounds.

---

## 3. Architecture (actors)

```
GUI (egui) ──Command(cast)──►  SimulationActor  ──refresh──► SharedSnapshot ──► GUI
Simulated PROFIBUS master ──►  (Regulator)       ──refresh──► SharedSnapshot ──► Data_Exchange replies
```

- **`SimulationActor`** ([`actors/simulation.rs`](../../src/actors/simulation.rs)):
  identical in shape to ORME's/OSNE's — sole owner of the `Regulator`,
  re-armed one-shot timer, publishes the `SharedSnapshot` on every step.
- **`ProfibusServerActor`** ([`actors/network.rs`](../../src/actors/network.rs)):
  owns the serial link; `Reconfigure` closes/reopens the transport if the
  port/baud/station address changes; keeps the session's `JoinHandle`
  (aborted on shutdown); publishes the link state (`ServerStatus`, including
  the current DP-V0 state machine state) for the GUI.
- **[`profibus.rs`](../../src/profibus.rs)** — **source of truth** for the
  protocol: frame codec (`SD1`/`SD2`/`SD3`/`SD4`/`SC`, FCS), decoding of the
  services (`Slave_Diag`/`Set_Prm`/`Chk_Cfg`/`Data_Exchange`) and the slave's
  state machine `SlaveFsm` (`PowerOn → WaitPrm → WaitCfg → DataExchange`).
- **[`map.rs`](../../src/map.rs)** — conversion of the `Data_Exchange` I/O
  byte blocks to/from the regulator's `Command`s (see
  [`reference_profibus.md`](reference_profibus.md) §3).
- **[`profibus_server.rs`](../../src/profibus_server.rs)** — session loop
  over any `AsyncRead + AsyncWrite` stream (the serial port in production, a
  `tokio::io::duplex` in tests): reads a frame, decodes it, calls
  `SlaveFsm::handle`, applies the resulting `Command`s, encodes and sends the
  reply back. It also handles the **protocol watchdog**
  (`tokio::select!` between frame reading and a delay, like OSNE's NAMUR
  watchdog — but here it is a **genuine part of the DP protocol**, armed by
  `Set_Prm`, not a homemade addition).

Unlike Modbus (ORME, a separate memory table regenerated on every tick) and
like OPC UA/NAMUR, there is **no persistent memory table**: the
`Data_Exchange` input block is recomputed on the fly from the
`SharedSnapshot` at the moment of replying.

**No multi-master policy to manage**: the serial link *is* the sole master
(like Modbus RTU or the NAMUR serial port), unlike ORME's Modbus TCP
(eviction) or even OSNE's NAMUR TCP (point-to-point without eviction).

---

## 4. PROFIBUS DP-V0 codec — choices and accepted limitations

- **Frame delimiters** (`SD1=0x10`, `SD2=0x68`, `SD3=0xA2`, `SD4=0xDC`,
  `SC=0xE5`, `ED=0x16`) and **FCS** (modulo-256 sum): compliant with the
  standard, well documented publicly.
- **SAP numbers of the parametrization services** (`Slave_Diag=61`,
  `Set_Prm=62`, `Chk_Cfg=63`): compliant.
- **Exact encoding of the FC field bits**, **precise layout of the
  diagnostic bytes**, and **layout of the I/O blocks** (`map.rs`): these are
  **conventions specific to this simulator**, not a GSD profile registered
  with the PNO. The simulator systematically uses **SD2** frames (variable
  length) for `Data_Exchange`, even when `SD3` (fixed 8 bytes) would suffice
  in a real profile — a choice that simplifies the codec without losing any
  coverage of the protocol concepts.
- **PROFIBUS identifier** (`Ident_Number = 0xEE01`): **fictitious**, not
  registered with the PNO (PROFIBUS & PROFINET International) — does not
  represent any real catalogue device.
- **No bus timing whatsoever**: neither a response window (`Tsdr`), nor a
  token, nor multi-master arbitration are implemented — see §1.

Full detail in [`reference_profibus.md`](reference_profibus.md).

---

## 5. Configuration & persistence

`AppConfig` (language / serial link / process / control / update check)
serialized as **TOML** ([`config.rs`](../../src/config.rs)), **sanitized on
load** (`AppConfig::sanitized`: bounds ordered, `τ ≥ 1e-3`, `dead_time ≥ 0`,
finite floats, station address bounded to `[0, 125]`). File:
`mock_ru_pbdp.toml` (overridable via `MOCK_CONFIG`). Unlike ORME/OSNE, **no
IP allowlist** (the serial link is inherently point-to-point, no notion of a
network address).

---

## 6. Future directions

- A genuine **simulated PROFIBUS DP master** tool (separate binary), using
  the same encoding/decoding functions exposed for testing in `profibus.rs`,
  to drive this simulator or any other software slave without depending on
  an ad hoc script.
- Generation of an illustrative **GSD** file (non-functional on the simulator
  side) documenting the simulated I/O profile, for educational purposes.
- **DP-V1** support (acyclic access, alarms) should the educational need
  arise — out of scope initially (DP-V0 only).
- Promotion of the controller model into a shared `mock_lib_*` (today
  duplicated between ORME and this instrument, as with ORUE).
