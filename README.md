<p align="center">
  <img src="pic/Logo-CESAM-Couleur-vect-card.png" alt="CESAM-Lab" height="84">
</p>

# cesam-tools — CESAM-Lab toolbox

*🌍 **English** · [Français](README.fr.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Italiano](README.it.md) · [Português](README.pt.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md)*

<p align="center">
  <a href="https://github.com/CESAMLAB/cesam-tools/releases/latest"><img src="https://img.shields.io/github/v/release/CESAMLAB/cesam-tools?label=release" alt="Latest release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue" alt="License: MIT"></a>
</p>

A Rust workspace gathering the **CESAM-Lab tools**, starting with **simulators of
industrial instruments**: virtual devices that reproduce realistic physical
behavior and communicate via field protocols. Useful for developing, testing and
demonstrating supervisors, PLCs or gateways **without real hardware**.

> Distributed free of charge under the [MIT](LICENSE) license.

## Available instruments

| Crate | Product | Description | Protocol | GUI |
|-------|---------|-------------|-----------|-----|
| [`mock_bin_ru_modbus`](mock_bin_ru_modbus) | **ORME** | Controller (PID / TOR / PWM) over a transfer function | Modbus TCP & RTU (slave) | egui |
| [`mock_bin_su_namur`](mock_bin_su_namur) | **OSNE** | Overhead lab stirrer: motor transfer function, fast speed control, adjustable viscous load | NAMUR over TCP & serial RS-232 (slave) | egui |
| [`mock_bin_ru_opcua`](mock_bin_ru_opcua) | **ORUE** | Process regulator (anti-windup PID) over a first-order process, with configurable OPC UA security | OPC UA (server) | egui |
| [`mock_bin_ru_sparkplugb`](mock_bin_ru_sparkplugb) | **ORSE** | Process regulator exposed as an MQTT Sparkplug B edge node (outbound) | Sparkplug B / MQTT (client) | egui |
| [`mock_bin_ru_s7`](mock_bin_ru_s7) | **ORSS** | Process regulator exposed as an S7comm server over ISO-on-TCP (RFC1006) | S7comm (server) | egui |
| [`mock_bin_ru_ethernetip`](mock_bin_ru_ethernetip) | **OREE** | Process regulator exposed as an EtherNet/IP adapter (explicit messaging CIP) | EtherNet/IP (adapter) | egui |
| [`mock_bin_ru_pbdp`](mock_bin_ru_pbdp) | **ORPD** | Process regulator exposed as a simulated PROFIBUS DP-V0 slave over a serial link | PROFIBUS DP (slave, serial) | egui |

Shared library:

| Crate | Description |
|-------|-------------|
| [`mock_lib_control`](mock_lib_control) | Reusable control building blocks: anti-windup PID, on/off with hysteresis, first-order process + pure dead time (FOPDT). |

## ORME — the simulated controller

<p align="center">
  <img src="pic/orme-logo.svg" alt="ORME — Open Regulator Modbus Emulator" height="120">
</p>

> **ORME** — *Open Regulator Modbus Emulator*. **"Open the bus."**
> A field controller that exists only on your Modbus bus.

A complete virtual industrial controller:

- **Process** modelled by a first-order transfer function with pure dead time
  `K·e^(-Ls) / (1 + T·s)` (typical of an oven or thermostatic bath).
- Bidirectional **control**: direction 1 (heating) and direction 2 (cooling), each
  configurable as **PID**, **on/off (TOR)** or **cycle relay (PWM)**.
- **Modes** run/stop and automatic/manual.
- **Modbus server** in **TCP** or **serial RTU / RS485** (`rtu` feature), at your
  choice. Address table (setpoint, measurement, output, modes…), **IP allowlist**
  (`*` wildcards) configurable at runtime, and **single-master policy** (only one
  remote master at a time; in TCP a newcomer disconnects the previous one).
- **Single-page graphical interface**: operation, real-time **trend curve**, **live
  Modbus address table**, and a **Settings modal** (TCP/RTU transport, port,
  allowed IPs, serial parameters, transfer function, setpoint bounds).
- **Configuration persisted** in TOML format (`mock_ru_modbus.toml`), reloaded
  at startup, with a reset-to-defaults button.

### Asynchronous architecture

```
        Command (non-blocking cast)            shared snapshot
  GUI (egui) ──────────────────────►  SimulationActor  ──────────►  GUI (read)
  Modbus write ─────────────────►   (ractor)         ──────────►  Modbus image
  Modbus read  ◄──────────────────────────────────────  Modbus image
```

- **`ractor`**: a single actor owns the controller state; all mutations go through
  messages (no lock on the business logic).
- **`tokio-modbus`**: Modbus TCP and serial RTU server (`Service` trait).
- **`eframe`/`egui`**: graphical interface on the main thread.

## OSNE — the simulated lab stirrer

<p align="center">
  <img src="pic/osne-logo.svg" alt="OSNE — Open Stirrer NAMUR Emulator" height="120">
</p>

> **OSNE** — *Open Stirrer NAMUR Emulator*.
> A laboratory overhead stirrer (IKA-style) that exists only on your NAMUR link.

A complete virtual lab stirrer:

- **Motor** modelled by a rotational transfer function `J·dω/dt = T − k·η·ω −
  friction` (explicit Euler), with a **fast PID** driving torque to track the
  speed setpoint.
- **Adjustable viscosity** `η`: raises the load torque; at high viscosity the
  motor saturates and the setpoint becomes unreachable (**overload**) — like a
  real stirrer.
- **NAMUR server** (ASCII command protocol) over **TCP** (test without hardware)
  or **serial RS-232** (`serial` feature), with a per-session **watchdog**
  (`OUT_WD1@<m>`), **single-master** policy and an **IP allowlist** (TCP).
- **Single-page graphical interface**: speed setpoint, viscosity, live
  speed/torque **trend curve**, an embedded **NAMUR mini-terminal** (send/inspect
  frames with command history), and a **Settings modal** (TCP/serial transport,
  motor parameters, bounds, 8-language i18n).
- **Configuration persisted** in TOML format (`mock_su_namur.toml`), reloaded at
  startup, with a reset-to-defaults button.

It shares ORME's architecture (synchronous business model, `ractor` actors, `egui`
GUI). Run it with `cargo run -p mock_bin_su_namur`; the NAMUR server listens on
`0.0.0.0:4001` by default.

## ORUE — the simulated OPC UA regulator

<p align="center">
  <img src="pic/ru_opcua-logo.svg" alt="ORUE — Open Regulator UA Emulator" height="120">
</p>

> **ORUE** — *Open Regulator UA Emulator*. **"Unify the process."**
> A process regulator that exists only on your OPC UA address space.

A complete virtual process regulator:

- **Process** modelled by a first-order transfer function driven by an
  **anti-windup PID**, stepped every 0.5 s.
- **OPC UA server** (`async-opcua`, Tokio-native, 100% Rust crypto — no OpenSSL,
  MPL-2.0 stack). **Configurable security** (`SecurityConfig`): `None`/anonymous by
  default (instant startup) **or** `Basic256Sha256` / SignAndEncrypt with a
  self-signed certificate (`pki/`, generated on first encrypted run), plus anonymous
  and/or **username/password** tokens.
- **A posture that differs from ORME/OSNE**: OPC UA security rests on **certificate +
  authentication**, not on an IP allowlist (there is **none**); the server accepts
  **several concurrent client sessions** (no single-master, last writer wins). The
  default `None`/anonymous on `0.0.0.0:4840` is the most open of the workspace — a GUI
  banner warns whenever encryption is off.
- **Single-page graphical interface**: operation, real-time **trend curve**, and a
  **Settings modal** (network, process transfer function, PID gains, setpoint bounds,
  security, 8-language i18n).
- **Configuration persisted** in TOML format (`mock_ru_opcua.toml`), reloaded at
  startup, with a reset-to-defaults button.

It shares ORME's architecture (synchronous business model, `ractor` actors, `egui`
GUI). Run it with `cargo run -p mock_bin_ru_opcua`; the OPC UA server listens on
`0.0.0.0:4840` by default. The address space is documented in
[`mock_bin_ru_opcua/docs/en/reference_opcua.md`](mock_bin_ru_opcua/docs/en/reference_opcua.md).

## ORSE — the simulated Sparkplug B edge node

<p align="center">
  <img src="pic/ru_spb-logo.svg" alt="ORSE — Open Regulator Sparkplug Emulator" height="120">
</p>

> **ORSE** — *Open Regulator Sparkplug Emulator*.
> A process regulator that only exists as an MQTT Sparkplug B edge node.

A complete virtual process regulator, same PID + first-order process model as ORME:

- **MQTT Sparkplug B edge node** (outbound client, `rumqttc` + `sparkplug-rs`,
  Eclipse Tahu protobuf, 100% Rust — no `protoc`). Publishes `NBIRTH`/`NDATA` and
  a `NDEATH` carried by the MQTT **Last Will** (robust against any link loss);
  reacts to `NCMD` writes from the broker. `bdSeq`/`seq` counters owned and tested
  in a pure protocol layer, not delegated to a framework.
- **A posture that differs from ORME/OSNE**: being a client rather than a server,
  there is **no IP allowlist**. **Plaintext MQTT by default** (port 1883,
  unencrypted, no authentication) — a GUI banner warns until TLS + credentials
  are enabled to leave a trusted network.
- **Single-page graphical interface**: operation, real-time **trend curve**, and a
  **Settings modal** (broker address/credentials/TLS, process transfer function,
  PID gains, setpoint bounds, 8-language i18n).
- **Configuration persisted** in TOML format (`mock_ru_sparkplugb.toml`), reloaded
  at startup, with a reset-to-defaults button.

Run it with `cargo run -p mock_bin_ru_sparkplugb`; it connects outbound to the
broker configured in *Settings* (default `localhost:1883`) — no listening port.

## ORSS — the simulated S7 regulator

<p align="center">
  <img src="pic/ru_s7-logo.svg" alt="ORSS — Open Regulator S7 Server" height="120">
</p>

> **ORSS** — *Open Regulator S7 Server*.
> A process regulator that exists only on your S7comm link.

A complete virtual process regulator, same PID + first-order process model as ORME:

- **Hand-written S7comm server** over ISO-on-TCP (RFC1006), port 102: TPKT framing,
  COTP (CR→CC, DT) and S7comm (Setup, Read/Write Var) over a **DB1 byte image**.
  No S7 **server** crate exists in Rust (only client-oriented ones), so the
  required subset is implemented directly — bounded parsing, no panic on a
  malformed frame.
- **Multiple simultaneous clients accepted** (real-PLC behaviour), unlike ORME's
  single-master eviction policy — "last writer wins".
- **No authentication or encryption** ("classic" S7): only the **IP allowlist**
  and network topology protect access; a GUI banner warns when exposed
  (`0.0.0.0` + empty allowlist).
- **Single-page graphical interface**: operation, real-time **trend curve**, and a
  **Settings modal** (network, allowlist, process transfer function, PID gains,
  setpoint bounds, 8-language i18n).
- **Configuration persisted** in TOML format (`mock_ru_s7.toml`), reloaded at
  startup, with a reset-to-defaults button.

Run it with `cargo run -p mock_bin_ru_s7`; the S7comm server listens on
`0.0.0.0:102` by default (port < 1024 needs root privileges).

## OREE — the simulated EtherNet/IP regulator

<p align="center">
  <img src="pic/ru_eip-logo.svg" alt="OREE — Open Regulator EtherNet/IP Emulator" height="120">
</p>

> **OREE** — *Open Regulator EtherNet/IP Emulator*.
> A process regulator that exists only on your EtherNet/IP link.

A complete virtual process regulator, same PID + first-order process model as ORME:

- **Hand-written EtherNet/IP adapter** (encapsulation `RegisterSession`,
  `SendRRData`/CPF, and CIP `Read Tag`/`Write Tag` by symbolic segment,
  **little-endian**), port 44818. No EtherNet/IP **adapter** crate exists in Rust
  (only client/scanner-oriented ones), so the required subset is implemented
  directly — bounded parsing, no panic on a malformed packet.
- **Multiple simultaneous clients accepted** (adapter behaviour), unlike ORME's
  single-master eviction policy — each session gets a *session handle*, "last
  writer wins".
- **No authentication or encryption** ("classic" EtherNet/IP): only the **IP
  allowlist** and network topology protect access; a GUI banner warns when
  exposed.
- **Single-page graphical interface**: operation, real-time **trend curve**, and a
  **Settings modal** (network, allowlist, process transfer function, PID gains,
  setpoint bounds, 8-language i18n).
- **Configuration persisted** in TOML format (`mock_ru_ethernetip.toml`), reloaded
  at startup, with a reset-to-defaults button.

Run it with `cargo run -p mock_bin_ru_ethernetip`; the EtherNet/IP adapter listens
on `0.0.0.0:44818` by default.

## ORPD — the simulated PROFIBUS DP regulator

<p align="center">
  <img src="pic/ru_pbdp-logo.svg" alt="ORPD — Open Regulator Profibus DP" height="120">
</p>

> **ORPD** — *Open Regulator Profibus DP*.
> A process regulator that exists only on your PROFIBUS DP link.

A complete virtual process regulator, same PID + first-order process model as ORME:

- **Software simulator of PROFIBUS DP-V0 frames** over a serial link
  (RS-485/RS-232): frame codec (`SD1`/`SD2`/`SD3`/`SD4`/`SC`, FCS) and the
  slave's state machine (`Power_On → Wait_Prm → Wait_Cfg → Data_Exchange`).
  ⚠️ **Not interoperable with real PROFIBUS DP hardware**: real bus timing
  (slot time, `Tsdr`) requires a dedicated ASIC that this software-only
  simulator does not attempt to emulate — see
  [`reference_profibus.md`](mock_bin_ru_pbdp/docs/en/reference_profibus.md) §6.
- **Serial link is the only transport** (no TCP equivalent for PROFIBUS DP,
  unlike ORME/OSNE where serial is an optional feature next to a
  always-present TCP transport): `tokio-serial` is a direct, non-optional
  dependency. No IP allowlist (inherently point-to-point).
- **Protocol watchdog** — a genuine part of DP-V0 (armed by the master via
  `Set_Prm`), not a homemade addition; forces the safe state on expiry.
- **Single-page graphical interface**: operation, real-time **trend curve**, a
  **frame mini-terminal** (hex log of RX/TX traffic), and a **Settings modal**
  (serial port, baud rate, station address, process transfer function, PID
  gains, setpoint bounds, 8-language i18n).
- **Configuration persisted** in TOML format (`mock_ru_pbdp.toml`), reloaded at
  startup, with a reset-to-defaults button.

Run it with `cargo run -p mock_bin_ru_pbdp`; it tries to open the configured
serial port (default `/dev/ttyUSB0` or `COM3`, 500 kbit/s, station address 3).

## Download

Prebuilt binaries are available on the [**Releases**](https://github.com/CESAMLAB/cesam-tools/releases/latest) page — **no Rust toolchain required**. Each instrument ships its own executable (`orme`, `osne`, `ru_opcua`, `ru_spb`, `ru_s7`, `ru_eip`, `ru_pbdp`).

**ORME** (Modbus controller):

| Platform | GUI | Headless (TCP only, no GUI) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`orme-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-linux-x86_64) | [`orme-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-linux-x86_64-headless) |
| Windows x86_64 | [`orme-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`orme-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-rpi-arm64) | [`orme-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-rpi-arm64-headless) |

**OSNE** (NAMUR lab stirrer):

| Platform | GUI | Headless (TCP only, no GUI) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`osne-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-linux-x86_64) | [`osne-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-linux-x86_64-headless) |
| Windows x86_64 | [`osne-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`osne-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-rpi-arm64) | [`osne-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-rpi-arm64-headless) |

**ORUE** (OPC UA regulator):

| Platform | GUI | Headless (TCP only, no GUI) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`ru_opcua-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-linux-x86_64) | [`ru_opcua-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-linux-x86_64-headless) |
| Windows x86_64 | [`ru_opcua-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`ru_opcua-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-rpi-arm64) | [`ru_opcua-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-rpi-arm64-headless) |

**ORSE** (Sparkplug B edge node):

| Platform | GUI | Headless (client only, no GUI) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`ru_spb-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_spb-linux-x86_64) | [`ru_spb-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_spb-linux-x86_64-headless) |
| Windows x86_64 | [`ru_spb-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_spb-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`ru_spb-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_spb-rpi-arm64) | [`ru_spb-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_spb-rpi-arm64-headless) |

**ORSS** (S7comm regulator):

| Platform | GUI | Headless (TCP only, no GUI) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`ru_s7-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_s7-linux-x86_64) | [`ru_s7-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_s7-linux-x86_64-headless) |
| Windows x86_64 | [`ru_s7-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_s7-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`ru_s7-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_s7-rpi-arm64) | [`ru_s7-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_s7-rpi-arm64-headless) |

**OREE** (EtherNet/IP adapter):

| Platform | GUI | Headless (TCP only, no GUI) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`ru_eip-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_eip-linux-x86_64) | [`ru_eip-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_eip-linux-x86_64-headless) |
| Windows x86_64 | [`ru_eip-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_eip-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`ru_eip-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_eip-rpi-arm64) | [`ru_eip-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_eip-rpi-arm64-headless) |

**ORPD** (PROFIBUS DP regulator):

| Platform | GUI | Headless (serial link, no GUI) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`ru_pbdp-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_pbdp-linux-x86_64) | [`ru_pbdp-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_pbdp-linux-x86_64-headless) |
| Windows x86_64 | [`ru_pbdp-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_pbdp-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`ru_pbdp-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_pbdp-rpi-arm64) | [`ru_pbdp-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_pbdp-rpi-arm64-headless) |

```bash
chmod +x orme-linux-x86_64        # Linux / Raspberry Pi (same for the other instruments)
./orme-linux-x86_64
```

Linux/RPi binaries are dynamically linked to glibc and need a desktop environment (X11/Wayland) for the GUI. On **Wayland**, install the desktop entry for the taskbar icon: `scripts/install-desktop.sh`. Verify integrity with the published checksums:

```bash
sha256sum -c SHA256SUMS
```

## Quick start

```bash
# Prerequisites: Rust stable (2021 edition, >= 1.85).
# Linux system dependencies for the GUI: libxkbcommon, libwayland/xcb, openGL.

cargo run -p mock_bin_ru_modbus
```

The window opens and the Modbus TCP server listens on `0.0.0.0:5502`. The **port**,
the **listening IP** and the **IP allowlist** are set in the **⚙ Settings** modal
(applied at runtime) then are **persisted** in `mock_ru_modbus.toml`. The
**interface language** (French, English, German, Spanish, Italian, Portuguese,
Dutch, Polish) is chosen in this same modal and is persisted. To use another
configuration file:

```bash
MOCK_CONFIG=/path/to/ma_config.toml cargo run -p mock_bin_ru_modbus
```

### Test the Modbus link

With any Modbus client (e.g. `mbpoll`):

```bash
# Start (coil 0) then read the measurement (input registers 0-1, f32)
mbpoll -m tcp -a 1 -t 0 -p 5502 127.0.0.1 1      # write the On/Off coil
mbpoll -m tcp -a 1 -t 3:float -r 1 -p 5502 127.0.0.1   # read PV (f32)
```

The complete address table is documented in
[`mock_bin_ru_modbus/src/map.rs`](mock_bin_ru_modbus/src/map.rs).

## Development

```bash
cargo test --workspace      # unit + integration tests
cargo clippy --workspace    # lint
```

## Documentation

Each instrument carries its own documentation in its `docs/` subfolder, available
in eight languages (`docs/<language>/`). English versions:

**ORME** (Modbus controller):

- [**User manual**](mock_bin_ru_modbus/docs/en/manuel_utilisateur.md) — getting started, GUI, settings, FAQ.
- [Design document](mock_bin_ru_modbus/docs/en/conception.md) — architecture and technical choices.
- [Modbus address table](mock_bin_ru_modbus/docs/en/table_modbus.md) — complete addressing plan.
- [Software maintenance](mock_bin_ru_modbus/docs/en/maintenance.md) — build, configuration, extension, troubleshooting.

**OSNE** (NAMUR lab stirrer):

- [**User manual**](mock_bin_su_namur/docs/en/manuel_utilisateur.md) — getting started, GUI, NAMUR mini-terminal, settings, FAQ.
- [Design document](mock_bin_su_namur/docs/en/conception.md) — motor model, control loop, architecture.
- [NAMUR command set](mock_bin_su_namur/docs/en/commandes_namur.md) — protocol reference (channels, commands, examples).
- [Software maintenance](mock_bin_su_namur/docs/en/maintenance.md) — build, configuration, extension, troubleshooting.

**ORUE** (OPC UA regulator):

- [**User manual**](mock_bin_ru_opcua/docs/en/manuel_utilisateur.md) — getting started, GUI, connecting an OPC UA client, FAQ.
- [Design document](mock_bin_ru_opcua/docs/en/conception.md) — PID + process model, actor architecture, `async-opcua` stack, security.
- [OPC UA reference](mock_bin_ru_opcua/docs/en/reference_opcua.md) — endpoint, namespace, nodes (reads/writes, examples).
- [Software maintenance](mock_bin_ru_opcua/docs/en/maintenance.md) — build, configuration, extension, troubleshooting.

**ORSE** (Sparkplug B edge node):

- [**User manual**](mock_bin_ru_sparkplugb/docs/en/manuel_utilisateur.md) — getting started, GUI, broker connection, FAQ.
- [Design document](mock_bin_ru_sparkplugb/docs/en/conception.md) — actor architecture, protocol layer, library choices.
- [Sparkplug B reference](mock_bin_ru_sparkplugb/docs/en/reference_sparkplugb.md) — topics, metrics, NBIRTH/NDATA/NDEATH, NCMD mapping.
- [Software maintenance](mock_bin_ru_sparkplugb/docs/en/maintenance.md) — build, configuration, extension, troubleshooting.

**ORSS** (S7comm regulator):

- [**User manual**](mock_bin_ru_s7/docs/en/manuel_utilisateur.md) — getting started, GUI, connecting an S7 client, FAQ.
- [Design document](mock_bin_ru_s7/docs/en/conception.md) — actor architecture, protocol layer, session policy.
- [S7comm reference](mock_bin_ru_s7/docs/en/reference_s7.md) — TPKT/COTP/S7comm framing, DB1 image, examples.
- [Software maintenance](mock_bin_ru_s7/docs/en/maintenance.md) — build, configuration, extension, troubleshooting.

**OREE** (EtherNet/IP adapter):

- [**User manual**](mock_bin_ru_ethernetip/docs/en/manuel_utilisateur.md) — getting started, GUI, connecting a CIP client, FAQ.
- [Design document](mock_bin_ru_ethernetip/docs/en/conception.md) — actor architecture, protocol layer, session policy.
- [EtherNet/IP reference](mock_bin_ru_ethernetip/docs/en/reference_ethernetip.md) — encapsulation, CIP Read/Write Tag, examples.
- [Software maintenance](mock_bin_ru_ethernetip/docs/en/maintenance.md) — build, configuration, extension, troubleshooting.

**ORPD** (PROFIBUS DP regulator):

- [**User manual**](mock_bin_ru_pbdp/docs/en/manuel_utilisateur.md) — getting started, GUI, non-interoperability warning, FAQ.
- [Design document](mock_bin_ru_pbdp/docs/en/conception.md) — actor architecture, protocol layer, codec choices.
- [PROFIBUS DP-V0 reference](mock_bin_ru_pbdp/docs/en/reference_profibus.md) — frames, sequencing, I/O blocks, watchdog, example sequence.
- [Software maintenance](mock_bin_ru_pbdp/docs/en/maintenance.md) — build, configuration, extension, troubleshooting.

## Brand & logos

The logos are in [`pic/`](pic/):

- [`orme-icon.svg`](pic/orme-icon.svg) / `orme-icon.png` — ORME icon (dial), also
  embedded as the application's window icon.
- [`orme-logo.svg`](pic/orme-logo.svg) — full ORME logo (icon + text).
- [`osne-icon.svg`](pic/osne-icon.svg) / `osne-icon.png` — OSNE icon (stirrer
  impeller), also embedded as the OSNE window icon.
- [`osne-logo.svg`](pic/osne-logo.svg) — full OSNE logo (icon + text).
- [`ru_opcua-icon.svg`](pic/ru_opcua-icon.svg) / `ru_opcua-icon.png` — ORUE icon
  (regulator dial wrapped in an OPC UA node ring), also embedded as the ORUE window
  icon.
- [`ru_opcua-logo.svg`](pic/ru_opcua-logo.svg) — full ORUE logo (icon + text).
- [`ru_spb-icon.svg`](pic/ru_spb-icon.svg) / `ru_spb-icon.png` — ORSE icon
  (regulator dial + Sparkplug bolt with unlinked pub/sub nodes), also embedded as
  the ORSE window icon.
- [`ru_spb-logo.svg`](pic/ru_spb-logo.svg) — full ORSE logo (icon + text).
- [`ru_s7-icon.svg`](pic/ru_s7-icon.svg) / `ru_s7-icon.png` — ORSS icon (regulator
  dial + open rack of square modules, S7 backplane), also embedded as the ORSS
  window icon.
- [`ru_s7-logo.svg`](pic/ru_s7-logo.svg) — full ORSS logo (icon + text).
- [`ru_eip-icon.svg`](pic/ru_eip-icon.svg) / `ru_eip-icon.png` — OREE icon
  (regulator dial + closed ring of diamonds, DLR EtherNet/IP), also embedded as
  the OREE window icon.
- [`ru_eip-logo.svg`](pic/ru_eip-logo.svg) — full OREE logo (icon + text).
- [`ru_pbdp-icon.svg`](pic/ru_pbdp-icon.svg) / `ru_pbdp-icon.png` — ORPD icon
  (regulator dial with a PROFIBUS DP motif), also embedded as the ORPD window
  icon.
- [`ru_pbdp-logo.svg`](pic/ru_pbdp-logo.svg) — full ORPD logo (icon + text).
- [`Logo-CESAM-Couleur-vect.png`](pic/Logo-CESAM-Couleur-vect.png) — CESAM-Lab logo.

Each icon is **generated** from its `*-logo.gen.py` script
([`pic/orme-logo.gen.py`](pic/orme-logo.gen.py),
[`pic/osne-logo.gen.py`](pic/osne-logo.gen.py),
[`pic/ru_opcua-logo.gen.py`](pic/ru_opcua-logo.gen.py),
[`pic/ru_spb-logo.gen.py`](pic/ru_spb-logo.gen.py),
[`pic/ru_s7-logo.gen.py`](pic/ru_s7-logo.gen.py),
[`pic/ru_eip-logo.gen.py`](pic/ru_eip-logo.gen.py),
[`pic/ru_pbdp-logo.gen.py`](pic/ru_pbdp-logo.gen.py)). All scripts but ORME's
also rasterize their `-icon.png` directly (via Pillow); the ORME `.svg` is
rasterized afterwards.

On **Wayland**, install an instrument's taskbar icon with
`scripts/install-desktop.sh [orme|osne|ru_opcua|ru_spb|ru_s7|ru_eip|ru_pbdp]`.

## License

[MIT](LICENSE) © 2026 CESAM-Lab

Third-party components bundled in some instruments are distributed under their
own licenses (notably the MPL-2.0 OPC UA stack used by `mock_bin_ru_opcua`); see
[NOTICE](NOTICE). They do not change the MIT license of the cesam-tools code.
