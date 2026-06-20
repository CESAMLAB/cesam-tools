<p align="center">
  <img src="pic/Logo-CESAM-Couleur-vect.png" alt="CESAM-Lab" height="84">
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

## Download

Prebuilt binaries are available on the [**Releases**](https://github.com/CESAMLAB/cesam-tools/releases/latest) page — **no Rust toolchain required**.

| Platform | GUI | Headless (TCP only, no GUI) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`orme-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-linux-x86_64) | [`orme-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-linux-x86_64-headless) |
| Windows x86_64 | [`orme-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`orme-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-rpi-arm64) | [`orme-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-rpi-arm64-headless) |

```bash
chmod +x orme-linux-x86_64        # Linux / Raspberry Pi
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

See [CLAUDE.md](CLAUDE.md) for the conventions and the detailed architecture.

## Documentation

Each instrument carries its own documentation in its `docs/` subfolder, available
in eight languages (`docs/<language>/`). For the controller (English version):

- [**User manual**](mock_bin_ru_modbustcp/docs/en/manuel_utilisateur.md) — getting started, GUI, settings, FAQ.
- [Design document](mock_bin_ru_modbustcp/docs/en/conception.md) — architecture and technical choices.
- [Modbus address table](mock_bin_ru_modbustcp/docs/en/table_modbus.md) — complete addressing plan.
- [Software maintenance](mock_bin_ru_modbustcp/docs/en/maintenance.md) — build, configuration, extension, troubleshooting.

## Brand & logos

The logos are in [`pic/`](pic/):

- [`orme-icon.svg`](pic/orme-icon.svg) / `orme-icon.png` — ORME icon (dial), also
  embedded as the application's window icon.
- [`orme-logo.svg`](pic/orme-logo.svg) — full ORME logo (icon + text).
- [`Logo-CESAM-Couleur-vect.png`](pic/Logo-CESAM-Couleur-vect.png) — CESAM-Lab logo.

The ORME icon is **generated** from [`pic/orme-logo.gen.py`](pic/orme-logo.gen.py)
(`python3 pic/orme-logo.gen.py` produces the `.svg` files, to be rasterized
afterwards).

## License

[MIT](LICENSE) © 2026 CESAM-Lab
