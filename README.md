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
| [`mock_bin_ru_modbustcp`](mock_bin_ru_modbustcp) | **ORME** | Controller (PID / TOR / PWM) over a transfer function | Modbus TCP & RTU (slave) | egui |
| [`mock_bin_su_namur`](mock_bin_su_namur) | **OSNE** | Overhead lab stirrer: motor transfer function, fast speed control, adjustable viscous load | NAMUR over TCP & serial RS-232 (slave) | egui |

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
- **Configuration persisted** in TOML format (`mock_ru_modbustcp.toml`), reloaded
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

## Download

Prebuilt binaries are available on the [**Releases**](https://github.com/CESAMLAB/cesam-tools/releases/latest) page — **no Rust toolchain required**. Each instrument ships its own executable (`orme`, `osne`).

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

```bash
chmod +x orme-linux-x86_64        # Linux / Raspberry Pi (same for osne-*)
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

cargo run -p mock_bin_ru_modbustcp
```

The window opens and the Modbus TCP server listens on `0.0.0.0:5502`. The **port**,
the **listening IP** and the **IP allowlist** are set in the **⚙ Settings** modal
(applied at runtime) then are **persisted** in `mock_ru_modbustcp.toml`. The
**interface language** (French, English, German, Spanish, Italian, Portuguese,
Dutch, Polish) is chosen in this same modal and is persisted. To use another
configuration file:

```bash
MOCK_CONFIG=/path/to/ma_config.toml cargo run -p mock_bin_ru_modbustcp
```

### Test the Modbus link

With any Modbus client (e.g. `mbpoll`):

```bash
# Start (coil 0) then read the measurement (input registers 0-1, f32)
mbpoll -m tcp -a 1 -t 0 -p 5502 127.0.0.1 1      # write the On/Off coil
mbpoll -m tcp -a 1 -t 3:float -r 1 -p 5502 127.0.0.1   # read PV (f32)
```

The complete address table is documented in
[`mock_bin_ru_modbustcp/src/map.rs`](mock_bin_ru_modbustcp/src/map.rs).

## Development

```bash
cargo test --workspace      # unit + integration tests
cargo clippy --workspace    # lint
```

## Documentation

Each instrument carries its own documentation in its `docs/` subfolder, available
in eight languages (`docs/<language>/`). English versions:

**ORME** (Modbus controller):

- [**User manual**](mock_bin_ru_modbustcp/docs/en/manuel_utilisateur.md) — getting started, GUI, settings, FAQ.
- [Design document](mock_bin_ru_modbustcp/docs/en/conception.md) — architecture and technical choices.
- [Modbus address table](mock_bin_ru_modbustcp/docs/en/table_modbus.md) — complete addressing plan.
- [Software maintenance](mock_bin_ru_modbustcp/docs/en/maintenance.md) — build, configuration, extension, troubleshooting.

**OSNE** (NAMUR lab stirrer):

- [**User manual**](mock_bin_su_namur/docs/en/manuel_utilisateur.md) — getting started, GUI, NAMUR mini-terminal, settings, FAQ.
- [Design document](mock_bin_su_namur/docs/en/conception.md) — motor model, control loop, architecture.
- [NAMUR command set](mock_bin_su_namur/docs/en/commandes_namur.md) — protocol reference (channels, commands, examples).
- [Software maintenance](mock_bin_su_namur/docs/en/maintenance.md) — build, configuration, extension, troubleshooting.

## Brand & logos

The logos are in [`pic/`](pic/):

- [`orme-icon.svg`](pic/orme-icon.svg) / `orme-icon.png` — ORME icon (dial), also
  embedded as the application's window icon.
- [`orme-logo.svg`](pic/orme-logo.svg) — full ORME logo (icon + text).
- [`osne-icon.svg`](pic/osne-icon.svg) / `osne-icon.png` — OSNE icon (stirrer
  impeller), also embedded as the OSNE window icon.
- [`osne-logo.svg`](pic/osne-logo.svg) — full OSNE logo (icon + text).
- [`Logo-CESAM-Couleur-vect.png`](pic/Logo-CESAM-Couleur-vect.png) — CESAM-Lab logo.

Each icon is **generated** from its `*-logo.gen.py` script
([`pic/orme-logo.gen.py`](pic/orme-logo.gen.py),
[`pic/osne-logo.gen.py`](pic/osne-logo.gen.py)). The OSNE script also rasterizes
`osne-icon.png` directly (via Pillow); the ORME `.svg` is rasterized afterwards.

On **Wayland**, install an instrument's taskbar icon with
`scripts/install-desktop.sh [orme|osne]`.

## License

[MIT](LICENSE) © 2026 CESAM-Lab
