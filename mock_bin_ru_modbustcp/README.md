# ORME — simulated Modbus controller

*🌍 **English** · [Français](README.fr.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Italiano](README.it.md) · [Português](README.pt.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md)*

> *Open Regulator Modbus Emulator* · package `mock_bin_ru_modbustcp` · binary `orme`

A **simulated** industrial controller, **Modbus TCP/RTU** slave, with a graphical
interface. Part of the [`cesam-tools`](../README.md) workspace.

## Features

- First-order process + pure dead time (FOPDT transfer function).
- Bidirectional control (heating / cooling), each direction in **PID** or
  **on/off**.
- Run/stop and auto/manual modes; auto (physical) and manual (%) setpoints.
- Modbus TCP server exposing the entire state.
- `egui` GUI with a real-time trend curve and PID gain tuning.
- **Multilingual interface**: French, English, German, Spanish, Italian,
  Portuguese, Dutch, Polish (choice in the *Settings* modal, persisted).

## Run

```bash
cargo run -p mock_bin_ru_modbustcp
# Alternative configuration file:
MOCK_CONFIG=./ma_config.toml cargo run -p mock_bin_ru_modbustcp
```

Listens by default on `0.0.0.0:5502`. The port, the listening IP and the IP
allowlist are set in the **⚙ Settings** modal and are persisted in TOML.

## Modbus address table

Float encoding: 2 registers, big-endian, high word first.

### Coils (FC 1/5/15)

| Addr | Role |
|----|------|
| 0 | Run (1) / Stop (0) |
| 1 | Auto (1) / Manual (0) |

### Discrete inputs (FC 2, read-only)

| Addr | Role |
|----|------|
| 0 | Running |
| 1 | Direction 1 (heating) active |
| 2 | Direction 2 (cooling) active |

### Holding registers (FC 3/6/16)

| Addr | Type | Role |
|-----|------|------|
| 0 | u16 | Direction 1 mode (0=Off, 1=PID, 2=TOR) |
| 1 | u16 | Direction 2 mode (0=Off, 1=PID, 2=TOR) |
| 2–3 | f32 | Automatic setpoint (SP) |
| 4–5 | f32 | Manual setpoint (output %, signed) |
| 6–7 | f32 | Kp direction 1 |
| 8–9 | f32 | Ki direction 1 |
| 10–11 | f32 | Kd direction 1 |
| 12–13 | f32 | Kp direction 2 |
| 14–15 | f32 | Ki direction 2 |
| 16–17 | f32 | Kd direction 2 |
| 18–19 | f32 | TOR hysteresis |

### Input registers (FC 4, read-only)

| Addr | Type | Role |
|-----|------|------|
| 0–1 | f32 | Measurement (PV) |
| 2–3 | f32 | Applied output (signed %: + heating / − cooling) |

The source of truth is the header of [`src/map.rs`](src/map.rs).

## Documentation

Documentation specific to this application (folder [`docs/en/`](docs/en/)):

- [**User manual**](docs/en/manuel_utilisateur.md) — getting started, operation, settings, FAQ.
- [Design document](docs/en/conception.md) — architecture, technical choices, control theory.
- [Modbus address table](docs/en/table_modbus.md) — complete addressing plan, encoding, examples.
- [Software maintenance](docs/en/maintenance.md) — build, configuration, extension, troubleshooting.
