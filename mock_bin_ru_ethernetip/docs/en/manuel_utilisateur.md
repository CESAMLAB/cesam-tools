# User Manual — EtherNet/IP Regulator (OREE)

*🌍 [FR](../fr/manuel_utilisateur.md) · **EN** · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

---

## 1. What the instrument is for

**OREE** simulates a process **regulation unit** (PID + first-order thermal process)
and exposes it as an **EtherNet/IP adapter** (CIP explicit messaging). It is used to
test a supervisor or an EtherNet/IP client (pycomm3, RSLinx for reading, rseip…)
without real hardware.

## 2. Getting started

```bash
cargo run -p mock_bin_ru_ethernetip        # GUI + EtherNet/IP adapter
```

The server listens by default on `0.0.0.0:44818` (no privilege required). The header
shows the state: **EtherNet/IP ●** (green) with the listen address, or an error
message (red). An orange banner warns if the server is **exposed** (all interfaces +
empty allowlist).

## 3. Interface

- **Header**: title, *Settings* / *Save* buttons, run/stop state, EtherNet/IP listen
  state, network exposure banner.
- **Left panel (Commands)**: *Run/Stop*, *Automatic mode (PID)*, *Setpoint*, *Manual
  output* (manual mode), **PID** tuning (Kp/Ki/Kd).
- **Central panel**: *Measurement / Setpoint / Output* cards + real-time **curve**.
- **\*Settings\* modal**: language, update check, **EtherNet/IP network** (listen IP,
  port, IP **allowlist** — one pattern per line, `*` = wildcard), **process** (K, τ,
  delay, ambient), **setpoint bounds**. *Apply* restarts the listener if the IP/port
  changes and saves the TOML.

## 4. Connecting an EtherNet/IP client

The client connects to the server's IP/port (automatic `RegisterSession`), then
reads/writes the **named tags** by explicit messaging: `Setpoint`, `ProcessValue`,
`Output`, `ManualOutput`, `Run`, `Auto`, etc. (see
[`reference_ethernetip.md`](reference_ethernetip.md)). ⚠️ Values are in
**little-endian** (REAL = `f32` LE).

## 5. FAQ

- **The client does not connect** → check IP/port (44818), the **allowlist**, the
  firewall.
- **Tag not found** → only the documented tags exist; names are case-sensitive.
- **My writes have no effect** → only the controllable tags act (`Setpoint`,
  `ManualOutput`, `Run`, `Auto`); the others are read-only.
- **Where is the config file?** → `mock_ru_ethernetip.toml` (current directory;
  overridable via `MOCK_CONFIG`).
