# User manual — OSNE (simulated NAMUR laboratory stirrer)

*🌍 [FR](../fr/manuel_utilisateur.md) · **EN** · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

> **OSNE** — *Open Stirrer NAMUR Emulator* · binary `mock_bin_su_namur`
> (executable `osne`) · MIT License · Publisher: **CESAM-Lab** · NAMUR identity:
> name `CESAM-STIRRER`, type `OSNE`.
>
> *A laboratory stirrer (IKA-style) that exists only on your NAMUR link — to test
> supervisors, scripts and gateways without real hardware.*

This manual is intended for the **user** of the simulated stirrer: how to launch
it, drive it from the interface, configure it, and connect it over **NAMUR** (TCP
or serial RS-232). No programming knowledge is required.

---

## 1. What is this software for?

It simulates a **laboratory stirrer** (benchtop overhead stirrer, IKA-style):

- a realistic **physical motor**: the speed rises/falls according to the applied
  torque, with **fast speed control**;
- an **adjustable viscous load**: the more viscous the medium, the higher the
  torque required — up to **overload** (unreachable setpoint);
- a **NAMUR server** (the ASCII serial protocol of lab devices) to drive/supervise
  it from another piece of software or a script;
- a **graphical interface** for operation, visualization and **protocol testing**
  (built-in mini NAMUR terminal).

It is a **test** tool: it lets you develop and demonstrate a supervisor, an
acquisition script or a gateway **without real hardware**.

---

## 2. Starting the software

Launch the executable matching your system:

| System | File |
|--------|------|
| Windows | `osne-windows-x86_64.exe` (double-click) |
| Linux PC | `./osne-linux-x86_64` |
| Raspberry Pi (screen) | `./osne-rpi-arm64` |

The window opens and the **NAMUR server starts automatically** (port `4001` by
default). The header shows the state:

- **● RUNNING / ● STOPPED**: motor state;
- **NAMUR ● 0.0.0.0:4001** (green): server listening; **✖ …** (red) in case of a
  problem (port busy, serial unavailable…);
- a **connection indicator**: over TCP it shows the connected master (or "no
  master"), over serial a simple dot. It turns **green** when a frame has been
  received recently (active link), grey otherwise.

> Without a screen (server only), see **§ 9 (Headless use)**.

---

## 3. The interface at a glance

```
┌──────────────── Header: OSNE title, ⚙ Settings, 💾 Save, states & indicators ────────────────┐
├──────────────────┬──────────────────────────────────────────────────────────────────────────────────┤
│  COMMANDS         │   MONITORING                                                                      │
│  (left)           │   - value cards (Speed / Torque / Viscosity / Overload)                           │
│  Start/Stop       │   - real-time trend CHART (Setpoint / Speed / Torque)                             │
│  Speed setpoint   │                                                                                   │
│  Viscosity        │                                                                                   │
│  PID settings     │                                                                                   │
├──────────────────┴──────────────────────────────────────────────────────────────────────────────────┤
│  ⇄ NAMUR FRAMES: mini-terminal (RX/TX) + command line + protocol reference (right)                    │
└───────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 4. Driving the stirrer (left panel)

### 4.1 Start / Stop
**Start / Stop** button. When stopped, the motor decelerates freely until it comes
to rest (friction + load), with zero motor torque.

### 4.2 Speed setpoint
**Speed setpoint** slider (in `tr/min`), bounded by the min/max limits set in
*Settings*. This is the same quantity as the NAMUR command `OUT_SP_4` (channel 4).
While running, the feedback loop drives the measured speed toward this setpoint.

### 4.3 Medium viscosity
**Viscosity** slider (logarithmic scale). It represents the **load** of the stirred
medium:

- **low** viscosity → low torque, the setpoint is reached quickly;
- **high** viscosity → large load torque; if the required torque exceeds the
  **maximum motor torque**, the setpoint speed is **no longer reached** → the
  **Overload ⚠** indicator lights up (the behaviour of a real stirrer facing a
  medium that is too thick).

### 4.4 PID settings (Kp, Ki, Kd)
The three gains of the speed feedback loop, adjustable live:

- **Kp** (proportional): the larger it is, the sharper the speed ramp-up (risk of
  overshoot/oscillation);
- **Ki** (integral): cancels the residual speed error over time;
- **Kd** (derivative): damps/anticipates (too strong → sensitive to noise).

> The default gains are deliberately "stiff": the output saturates at maximum
> torque as long as the error is large (fast ramp-up), then the integral term
> stabilizes. The PID output **is** the motor torque, bounded to `[0, torque_max]`.

---

## 5. Reading the trend chart

The chart (center) plots three quantities in real time. The **legend, top left**,
recalls the colour **and the latest value** of each series:

| Colour | Series | Meaning |
|--------|--------|---------|
| 🔵 blue | **Setpoint** | speed setpoint (while running) |
| 🔴 red | **Speed** | measured speed (`tr/min`, left axis) |
| 🟢 green | **Torque** | measured torque (`N·cm`, **right axis**) |

> The chart has **two vertical axes**: the **speed** (`tr/min`) on the left, the
> **torque** (`N·cm`) on the right. Torque is scaled to share the plot, but the
> right axis does display `N·cm`.

Above the chart, **cards** show the instantaneous values: **Speed**, **Torque**,
**Viscosity**, and **Overload ⚠** when the motor saturates. You can zoom/pan the
chart with the mouse.

---

## 6. The NAMUR mini-terminal (bottom of window)

The **⇄ NAMUR Frames** panel lets you **test the protocol** directly from the GUI,
without an external client:

- the **log** shows **received** frames (`← RX`, blue) and **sent** frames
  (`→ TX`, green), timestamped;
- the **command line** sends a NAMUR frame to the simulator (**Enter** key or
  **▶ Send** button). The **↑/↓** arrows recall previous commands (history);
- the **protocol reference** (right panel) lists the commands: a **click** inserts
  the command into the input line;
- the **🗑 Clear** button empties the log.

> Frames typed here are interpreted exactly like those of a network master:
> `OUT_SP_4 500` sets the setpoint, `START_4`/`STOP_4` start/stop, `IN_PV_4` reads
> the speed, etc. The **watchdog** (`OUT_WD1@…`) only takes effect within a real
> network session (see § 8).

---

## 7. Settings (⚙ button)

The **⚙ Settings** button opens a window to configure:

### Interface language
Selector at the top: **Français, English, Deutsch, Español, Italiano, Português,
Nederlands, Polski** (8 languages). The language is persisted.

### NAMUR transport
Choice of link — **only one active at a time**:

**TCP (Ethernet)**
- **Listening IP** (`0.0.0.0` = all interfaces) and **Port** (default 4001);
- **Allowed IPs**: one per line, `*` wildcards accepted (e.g. `192.168.1.*`).
  **Empty list = all IPs allowed.** Others are refused.

**Serial (RS-232)** — requires a binary compiled with the `serial` feature
- **Serial port**: `/dev/ttyUSB0` (Linux), `COM3` (Windows)…;
- **Baud** (default 9600), **Parity** (default Even), **Data bits** (7),
  **Stop bits** (1) — typical lab NAMUR setting: **9600 7E1**.

> ⚠️ **One master at a time.** Over TCP, a new master **waits** until the previous
> one disconnects (point-to-point link). The local GUI is **not** a master. Over
> serial, the bus *is* the unique master; prefer a **point-to-point link** (the
> server answers regardless of the requested address).

### Motor parameters
Simulated physical behaviour `J·dω/dt = T − k·η·ω − friction`:
- **Inertia** (`J`): motor responsiveness (small ⇒ fast);
- **Load coefficient** (`k`): weight of viscosity on torque;
- **Friction** (`N·cm`): residual dry friction;
- **Max torque** (`N·cm`): maximum motor torque (ceiling of the PID output).

### Speed bounds
Min/max limits of the speed setpoint (`tr/min`).

### Viscosity bounds
Min/max limits of the viscosity slider.

Buttons: **Apply** (takes effect immediately **and** saves), **Reset to
defaults**, **Close**.

### Saving settings
Settings are **saved** in a `mock_su_namur.toml` file (next to the software) and
**reloaded at the next start**. The **💾 Save** button in the header also stores the
PID gains and viscosity modified from the left panel.

---

## 8. Connecting a NAMUR client

The software is a **NAMUR slave** (TCP port 4001 by default, or serial depending
on the transport chosen in § 7). A client (script, terminal, gateway) **sends one
ASCII line per request**, terminated by `CR LF`. **Reads** (`IN_*`) return a value;
**writes/actions** (`OUT_*`, `START_*`, `STOP_*`, `RESET`) are **silent** (no
reply), as per NAMUR usage.

Main landmarks:

| Command | Effect |
|---------|--------|
| `IN_NAME` / `IN_TYPE` | identity (`CESAM-STIRRER` / `OSNE`) |
| `IN_PV_4` / `IN_PV_5` | read the speed (`tr/min`) / the torque (`N·cm`) |
| `IN_SP_4` | read the speed setpoint |
| `OUT_SP_4 <v>` | **set** the speed setpoint |
| `START_4` / `STOP_4` / `RESET` | start / stop / reset |
| `OUT_WD1@<m>` | **watchdog**: safe stop if silent for `<m>` s |

Example with `nc` (netcat):

```text
$ nc 127.0.0.1 4001
IN_NAME
CESAM-STIRRER
OUT_SP_4 1200          (silent)
START_4                (silent)
IN_PV_4
1200.0 4
STOP_4                 (silent)
```

> The **watchdog** `OUT_WD1@30` automatically stops the motor if **no line**
> arrives for 30 s (protection against loss of communication). `OUT_WD1@0` disarms
> it. The counter is rearmed on every command received.

> The **full protocol reference** (channels, encoding, examples) is in
> **[commandes_namur.md](commandes_namur.md)**. The same list is recalled **live**
> in the right panel of the mini-terminal.

---

## 9. Headless use (Docker)

For a background deployment (headless Raspberry Pi, server), a version **without an
interface** exists: it runs the simulation and the NAMUR server, drivable **only
through NAMUR**.

```bash
# Docker image (deployable anywhere):
docker run --rm -p 4001:4001 -v "$PWD/conf:/data" osne:headless
```

The directory mounted on `/data` lets you provide/keep `mock_su_namur.toml`.

---

## 10. Frequently asked questions

| Question / symptom | Answer |
|--------------------|--------|
| **Overload ⚠** lights up and the speed does not reach the setpoint. | Normal: the **viscosity** demands more torque than the motor provides. Lower the viscosity or the setpoint, or increase the **max torque** (Settings). |
| The speed does not move. | Check that the stirrer is **Running** and the setpoint is non-zero. |
| The header shows **NAMUR ✖**. | Port already in use or < 1024 without privileges (TCP), or serial port unavailable: change the setting in ⚙ Settings. |
| My NAMUR/TCP client is refused. | Its IP is not in the **allowlist**: empty the list or add a pattern (`192.168.1.*`). |
| `OUT_SP_4 …` returns nothing. | Normal: NAMUR writes/actions are **silent**. Read with `IN_SP_4` / `IN_PV_4`. |
| The motor stops on its own. | A **watchdog** is armed (`OUT_WD1@…`) and no command arrived in time. Disarm it (`OUT_WD1@0`) or send frames regularly. |
| The serial link does not open. | Binary compiled **without** the `serial` feature, or wrong port/permissions (`dialout` group on Linux). |
| My settings are not kept. | Click **Apply** / **💾 Save**. The `mock_su_namur.toml` file must be writable. |

---

*Related technical documentation: [conception.md](conception.md) ·
[commandes_namur.md](commandes_namur.md) · [maintenance.md](maintenance.md).*
