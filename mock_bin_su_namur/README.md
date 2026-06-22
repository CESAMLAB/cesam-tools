# OSNE — Open Stirrer NAMUR Emulator

Simulated **laboratory overhead stirrer** (IKA-style) for the CESAM-Lab toolbox.
A motor with a real transfer function and **fast speed regulation**, an adjustable
**viscous load**, driven over the **NAMUR** serial command protocol — plus a GUI.

Part of the `cesam-tools` workspace; built on the same architecture as the ORME
Modbus regulator (synchronous business model, `ractor` actors, `egui` GUI).

## Highlights

- **Motor transfer function** `J·dω/dt = T − k·η·ω − friction` (Euler), with a
  **fast PID** driving torque to track the speed setpoint.
- **Adjustable viscosity** `η`: raises the load torque (and the displayed torque);
  at high viscosity the motor saturates and the setpoint becomes unreachable
  (**overload**) — like a real stirrer.
- **NAMUR protocol** (ASCII commands) over **TCP** (test without hardware) or
  **RS-232 serial** (feature `serial`), with a **watchdog** (`OUT_WD1@m`).
- **GUI** (feature `gui`): speed setpoint, viscosity, live speed/torque trends,
  settings, 8-language i18n, connection indicator, exposure warning.

## Build & run

```bash
cargo run -p mock_bin_su_namur                 # GUI + NAMUR/TCP (+ serial)
cargo build -p mock_bin_su_namur --no-default-features   # headless, NAMUR/TCP only
cargo test -p mock_bin_su_namur                # unit + integration tests
```

The NAMUR server listens on `0.0.0.0:4001` by default (configurable in the GUI
settings, persisted to `mock_su_namur.toml`; path overridable via `MOCK_CONFIG`).

## NAMUR quick reference

| Command | Effect |
|---------|--------|
| `IN_NAME` / `IN_TYPE` | device identity |
| `IN_PV_4` / `IN_PV_5` | read speed (rpm) / torque (N·cm) |
| `IN_SP_4` / `OUT_SP_4 <v>` | read / set speed setpoint |
| `START_4` / `STOP_4` / `RESET` | start / stop / reset |
| `OUT_WD1@<m>` | watchdog: safe stop if silent for `<m>` s |

See [`docs/fr/commandes_namur.md`](docs/fr/commandes_namur.md) (protocol reference)
and [`docs/fr/conception.md`](docs/fr/conception.md) (design). French is the source
of truth; other languages follow.

## Features

- `gui` (default) — `egui` interface.
- `serial` (default) — NAMUR over RS-232 (`tokio-serial`). Without it: NAMUR/TCP only.

Licensed under MIT.
