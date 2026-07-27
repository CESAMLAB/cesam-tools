# User Manual — Simulated PROFIBUS DP Regulator (ORPD)

*🌍 [FR](../fr/manuel_utilisateur.md) · **EN** · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

> Crate: `mock_bin_ru_pbdp` · Executable: **ru_pbdp** · Brand: **ORPD**

---

## ⚠️ Before you start: what this simulator is NOT

`ru_pbdp` **is not** a hardware-compliant PROFIBUS DP slave. PROFIBUS DP is a
token bus whose timing windows (*slot time*, `Tsdr`, watchdog) demand a
dedicated circuit (SPC3/VPC3 ASIC, Hilscher/Softing/Siemens CP master card).
An ordinary Tokio program, even connected to a real RS-485 port, **cannot
meet these constraints**: a real PLC (a Siemens S7 with a master card, for
example) will **never** recognize this simulator as a valid slave on a real
bus.

What `ru_pbdp` actually does: it implements, **in software and without
real-time constraints**, the frame structure and state machine of a DP-V0
slave (parametrization, configuration, diagnostics, cyclic exchange). It is a
tool to **understand the protocol** and **test software development**
(codec, state machine, tooling) — not to drive field equipment. See
[reference_profibus.md](reference_profibus.md) §6 for the detailed
limitations.

---

## 1. What this simulator is for

`ru_pbdp` simulates a **process regulator** (a PID loop on a thermal
process, model identical to ORME/Modbus) and exposes it through a simulated
PROFIBUS DP-V0 frame set, over a serial link (RS-485/RS-232). The graphical
interface lets you **drive** the simulation and **visualize** its dynamics;
the frame log shows the traffic exchanged in hexadecimal.

---

## 2. Getting started

```bash
cargo run -p mock_bin_ru_pbdp          # GUI + PROFIBUS DP serial link
```

On startup, the simulator tries to open the configured serial port (by
default `/dev/ttyUSB0` or `COM3`, 500 kbit/s, station address 3). If the
port doesn't exist (common without serial hardware), the GUI displays the
opening error in the header — the controller simulation keeps running,
only the link is unavailable. Set the **serial port** in *Settings* to point
to an available pseudo-terminal or USB-serial adapter.

---

## 3. The interface

### Header

- **Title** and **⚙ Settings** / **💾 Save settings** buttons.
- On the right: **device state** (RUNNING / STOPPED), **link state**
  (`PROFIBUS ● <port> [<state>]` in green if open — the state shown is that
  of the DP-V0 state machine: `Power_On`/`Wait_Prm`/`Wait_Cfg`/
  `Data_Exchange`), and the **CESAM-Lab logo**.
- A **permanent orange banner** reminds you of the non-interoperability with
  real hardware (see the warning above).

### Mini-terminal (bottom of the window)

Read-only log of **received** (← RX) and **sent** (→ TX) frames, timestamped
and shown in hexadecimal. **Clear** button to empty the log.

### Command panel (left)

Identical to ORME: **Run/Stop**, **Auto/Manual**, control modes for
**direction 1 (heat)** / **direction 2 (cool)** (Off/PID/On-off/PWM),
**setpoints** (automatic and manual), **PID tuning** for both directions,
**hysteresis**, **minimum on-off cycle**, **PWM period**.

### Right panel: PROFIBUS I/O blocks

Live table of the *Output* (master→slave) and *Input* (slave→master) blocks,
with the byte layout used by this simulator — see
[reference_profibus.md](reference_profibus.md) §3.

### Central area

**Measurement**, **Active setpoint**, **Output** cards, and a trend curve.

---

## 4. Settings (⚙ modal)

- Interface **language** (8 languages), persisted.
- **Check for updates at startup** + **Check now** button.
- **Serial port**, **baud rate** (use a standard PROFIBUS DP value: 9600,
  19200, 45450, 93750, 187500, 500000, 1500000, 3000000, 6000000 or
  12000000), **station address** (0-125).
- **Protocol watchdog (allowed)**: checkbox — if unchecked, the watchdog
  requested by the master via `Set_Prm` is **ignored** (never armed).
- **Process transfer function**: gain `K`, time constant `τ`, pure dead
  time, ambient value.
- **Setpoint bounds**: min / max (automatically reordered if inverted).
- **Apply** / **Reset to defaults** / **Close**.

A port/baud/station change **closes and reopens** the serial link. Settings
are saved to `mock_ru_pbdp.toml` (current directory; overridable via the
`MOCK_CONFIG` environment variable).

**The frame format (8E1) is fixed by the PROFIBUS DP standard** and is not
adjustable here, unlike Modbus RTU or NAMUR serial.

---

## 5. The mini-terminal as an educational tool

Without real PROFIBUS hardware, the best way to observe the protocol is to
have **two instances** of this tool talk to each other — or write a small
script that replays a `Slave_Diag` → `Set_Prm` → `Chk_Cfg` → `Data_Exchange`
sequence over a pseudo-terminal (`socat -d -d pty,raw,echo=0 pty,raw,echo=0`)
— and read the mini-terminal to see the exchanged frames in hexadecimal,
with their decoding in [reference_profibus.md](reference_profibus.md).

---

## 6. FAQ

**Can I connect this simulator to a real PROFIBUS DP PLC?** No — see the
warning at the top of this document and §6 of
[reference_profibus.md](reference_profibus.md).

**The serial port won't open.** The indicated file/device doesn't exist or
permissions are insufficient (`dialout` group on Linux). The exact error is
shown in the GUI header.

**The link stays in `Wait_Prm`.** The master hasn't yet sent a `Set_Prm`
with the expected identifier (`0xEE01`, a **fictitious** identifier, not
registered with the PNO). See [reference_profibus.md](reference_profibus.md) §2.

**The link stays in `Wait_Cfg`.** The received `Chk_Cfg` doesn't announce
the expected I/O lengths (45 output bytes, 17 input bytes for this
simulator).

**The device stops on its own.** The protocol watchdog (armed by the master
via `Set_Prm`) has expired for lack of a cyclic exchange received in time —
this is the expected safe state, not a bug.

**Run without a graphical interface?** Build it *headless*:
`cargo run -p mock_bin_ru_pbdp --no-default-features` — the serial link and
the simulation run without a GUI.
