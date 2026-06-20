# User manual — ORME (simulated Modbus controller)

*🌍 [FR](../fr/manuel_utilisateur.md) · **EN** · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

> **ORME** — *Open Regulator Modbus Emulator* · binary `mock_bin_ru_modbustcp` ·
> MIT License · Publisher: **CESAM-Lab** · Modbus device identifier: **CESAM-Lab**
>
> *"Open the bus."* A field controller that exists only on your Modbus bus
> (TCP/RTU) — to test SCADA, PLCs and HMIs without real hardware.

This manual is intended for the **user** of the simulated controller: how to
launch it, drive it from the interface, configure it, and connect it over Modbus
TCP. No programming knowledge is required.

---

## 1. What is this software for?

It simulates an **industrial controller** (oven or thermostatic bath type):

- a realistic **physical process** (the "measurement" rises/falls according to the
  command);
- automatic or manual **control**, in **heating** and/or **cooling**;
- a **Modbus TCP server** to drive/supervise it from another software (PLC, SCADA,
  gateway…);
- a **graphical interface** for operation and visualization.

It is a **test** tool: it allows you to develop and demonstrate a supervisor or a
PLC **without real hardware**.

---

## 2. Starting the software

Launch the executable corresponding to your system:

| System | File |
|---------|---------|
| Windows | `orme-windows-x86_64.exe` (double-click) |
| Linux PC | `./orme-linux-x86_64` |
| Raspberry Pi (screen) | `./orme-rpi-arm64` |

The window opens and the **Modbus server starts automatically** (port `5502` by
default). The header shows the state:

- **● RUNNING / ● STOPPED**: device state;
- **Modbus ● 0.0.0.0:5502** (green): server listening; **✖ …** (red) in case of a
  network problem.

> Without a screen (server only), see **§ 9 (Use without a screen)**.

---

## 3. The interface at a glance

The window has four areas:

```
┌───────────────────────────── Header: title, ⚙ Settings, 💾 Save, states ─────────────────────────────┐
├──────────────────┬─────────────────────────────────────────────────┬───────────────────────────────────────────┤
│  COMMANDS         │   SUPERVISION                                   │   MODBUS ADDRESS TABLE                    │
│  (left)           │   - instantaneous values (Measurement /         │   (right)                                 │
│  Run/Stop         │     Setpoint / Output)                          │   live list: label, table,                │
│  Auto/Manual      │   - real-time TREND CURVE                       │   address, value, access                  │
│  Modes, setpoints │                                                 │                                           │
│  PID settings…    │                                                 │                                           │
└──────────────────┴─────────────────────────────────────────────────┴───────────────────────────────────────────┘
```

---

## 4. Driving the controller (left panel)

### 4.1 Run / Stop
**Run / Stop** button. When stopped, the output is zero and the measurement slowly
returns toward the ambient value.

### 4.2 Auto / Manual
- **Manual**: *you* impose the output via the **manual setpoint** (in %).
- **Auto**: the controller computes the output to reach the **auto setpoint**.

### 4.3 The setpoints
Each setpoint has a **numeric field** (precise keyboard entry) and a **slider**.
Both are always editable; the **active** setpoint (according to the mode) is shown
in bold.

| Setpoint | Unit | Role |
|----------|-------|------|
| **SP auto** | measurement unit (e.g. °C) | target to reach in Auto mode |
| **SP manual** | output %, from −100 to +100 | imposed output in Manual mode (**+** heating / **−** cooling) |

### 4.4 Control modes — direction 1 (heating) and direction 2 (cooling)
Each direction is set independently:

- **Disabled** — the direction does not act;
- **PID** — continuous control (output 0…100 %), precise and smooth;
- **On/off (TOR)** — relay with hysteresis: output 0 % or 100 %, simple but
  oscillating around the setpoint;
- **Cycle relay (PWM)** — a PID computes a duty cycle, *chopped* over a fixed
  period: the physical output stays on/off (0/100 %), but its **average** follows
  the PID. It is the best trade-off to finely drive a device that can only open or
  close (relay, on/off valve).

> 👉 **Important — see **§ 6 (Understanding the control)****: selecting PID/TOR/PWM
> for cooling *arms* the cooling, but it only **delivers when the measurement
> exceeds the setpoint**.

### 4.5 PID settings (Kp, Ki, Kd)
For each direction, three gains adjustable live:

- **Kp** (proportional): the larger it is, the more responsive the reaction (risk of oscillation);
- **Ki** (integral): cancels the residual error over time (too strong → overshoot);
- **Kd** (derivative): damps/anticipates (too strong → sensitive to noise).

### 4.6 TOR / PWM settings
- **TOR hysteresis** — width of the **dead band** of the On/off mode, centered on
  the setpoint (`[SP − h/2, SP + h/2]`): prevents the output from chattering
  endlessly. The wider it is, the larger the ripple but the more spaced out the
  switchings.
- **Min. TOR cycle (s)** — minimum duration during which the relay stays in a
  state before being able to switch again (**anti-short-cycle**). Protects a real
  actuator (relay, compressor) and smooths the behavior. `0` = disabled.
- **PWM period (s)** — duration of one cycle of the **cycle relay**. Short → more
  faithful average but frequent switchings; long → less wear but more pronounced
  ripple. Choose it much smaller than the time constant of the process.

---

## 5. Reading the trend curve

The curve (center) plots three quantities in real time. The **legend, at the top
left**, recalls the color **and the last value** of each series:

| Color | Series | Meaning |
|---------|-------|---------------|
| 🔵 blue | **Setpoint (SP)** | target (in Auto) |
| 🔴 red | **Measurement (PV)** | process value |
| 🟢 green | **Output (%)** | applied command (**+** heating / **−** cooling) |

Above the curve, three cards display the instantaneous values (Measurement, active
Setpoint, Output). You can zoom/pan the curve with the mouse.

---

## 6. Understanding the control (heating / cooling)

The controller acts in **a single direction at a time**, chosen according to the
error `Setpoint − Measurement`:

| Situation | Acting direction | Output | Indicator |
|-----------|---------------|--------|--------|
| Measurement **< ** Setpoint (need to heat) | **Direction 1 (heating)** | **positive** (0…+100 %) | **Heating active = 1** |
| Measurement **> ** Setpoint (need to cool) | **Direction 2 (cooling)** | **negative** (−100…0 %) | **Cooling active = 1** |

Practical consequences:

- Selecting **PID/TOR for cooling** is not enough to light up "Cooling active":
  the **measurement must be above the setpoint**. As long as the measurement is
  below, it is the **heating** that works.
- To see cooling deliver: in **Auto**, direction 2 in PID/TOR, **lower the setpoint
  below the current measurement** (or wait for an overshoot). The output becomes
  negative and **Cooling active** turns to 1.
- In **TOR**, the relay switches on the **half-hysteresis** on either side of the
  setpoint (symmetric dead band) and respects the **minimum cycle** between two
  switchings. In **PWM**, the output chops at 0/100 % but its average follows the
  PID.

---

## 7. Settings (⚙ button)

The **⚙ Settings** button opens a window to configure:

### Modbus transport
Choice of communication bus — **only one active at a time**:

**TCP (Ethernet)**
- **Listening IP** (`0.0.0.0` = all interfaces) and **Port** (default 5502);
- **Allowed IPs**: one per line, `*` wildcards accepted (e.g. `192.168.1.*`).
  **Empty list = all IPs allowed.** The others are refused.

**RTU (RS485)** — requires a binary compiled with the `rtu` feature
- **Serial port**: `/dev/ttyUSB0`, `/dev/ttyAMA0` (Raspberry Pi), `COM3` (Windows)…;
- **Baud** (default 19200), **Parity** (default Even), **Data bits** (8), **Stop
  bits** (1) — to match the master;
- **Slave address** (1–247).

> ⚠️ **Only one remote master at a time.** In TCP, the connection of a new master
> **automatically disconnects** the previous one. The local GUI is **not** a
> master: it always stays active. In RTU, favor a **point-to-point** link (the
> device responds whatever address is requested).

### Transfer function (process)
Simulated physical behavior `G(s) = K·e^(−L·s) / (1 + T·s)`:
- **Gain K**: measurement variation per % of output;
- **Constant T** (s): inertia/responsiveness;
- **Dead time L** (s): dead time before reaction;
- **Ambient**: rest value.

### Setpoint bounds
Min/max limits of the auto setpoint.

Buttons: **Apply** (takes effect immediately **and** saves), **Reset to defaults**,
**Close**.

### Saving settings
Settings are **saved** to a `mock_ru_modbustcp.toml` file (next to the software)
and **reloaded at the next startup**. The **💾 Save settings** button in the header
also saves the PID gains, the hysteresis, the minimum TOR cycle and the PWM period
modified from the left panel.

---

## 8. Connecting a Modbus client

The software is a **Modbus slave** (TCP port 5502 by default, or serial RTU
depending on the transport chosen in § 7). A client (PLC, SCADA, `mbpoll`…) can
**read** the state and **write** the setpoints/modes. Reminder: **only one remote
master at a time** (in TCP, a newcomer disconnects the previous one).

Main landmarks (addresses **base 0**):

| Data | Table | Address | Type | Access |
|--------|-------|---------|------|-------|
| Run/Stop | Coil | 0 | bit | R/W |
| Auto/Manual | Coil | 1 | bit | R/W |
| Mode direction 1 / direction 2 | Holding | 0 / 1 | 0=Off,1=PID,2=TOR,3=PWM | R/W |
| Auto setpoint | Holding | 2–3 | float | R/W |
| Manual setpoint | Holding | 4–5 | float | R/W |
| Min. TOR cycle (s) | Holding | 20–21 | float | R/W |
| PWM period (s) | Holding | 22–23 | float | R/W |
| Measurement (PV) | Input | 0–1 | float | R |
| Output (%) | Input | 2–3 | float | R |
| Identifier "CESAM-Lab" | Holding | 42–46 | ASCII text | R |

> The **complete table** (PID gains, hysteresis, float encoding, function codes,
> `mbpoll` examples) is in **[table_modbus.md](table_modbus.md)**. The same table
> is also visible **live** in the right panel of the GUI.

---

## 9. Use without a screen ("headless" / Docker)

For a background deployment (Raspberry Pi without a screen, server), a version
**without an interface** exists: it runs the simulation and the Modbus server,
controllable **only via Modbus**.

```bash
# Docker image (deployable anywhere):
docker run --rm -p 5502:5502 -v "$PWD/conf:/data" orme:headless
```

The folder mounted on `/data` allows you to provide/keep `mock_ru_modbustcp.toml`.

---

## 10. Frequently asked questions

| Question / symptom | Answer |
|---------------------|---------|
| **"Cooling active" does not turn to 1 even though I set PID/TOR.** | Normal: cooling only delivers if **the measurement exceeds the setpoint**. Lower the setpoint below the measurement (Auto mode). See **§ 6 (Understanding the control)**. |
| The measurement does not move. | Check that the device is **Running**, and that the setpoint/output are non-zero. |
| In manual, changing the direction 1/2 modes does nothing. | Normal: modes only apply in **Auto**. |
| The header shows **Modbus ✖**. | Port already in use or < 1024 without privileges: change the **port** in ⚙ Settings. |
| My Modbus client is refused. | Its IP is not in the **allowlist**: empty the list or add a pattern (`192.168.1.*`). |
| The floats read are inconsistent. | **Word order** problem on the client side (high word first). See table_modbus.md. |
| A setpoint written over Modbus is ignored. | A float occupies **2 registers**: write them **together**. |
| My settings are not kept. | Click **Apply** / **💾 Save**. The `mock_ru_modbustcp.toml` file must be writable. |

---

*Associated technical documentation: [conception.md](conception.md) ·
[table_modbus.md](table_modbus.md) · [maintenance.md](maintenance.md).*
