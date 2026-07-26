# User Manual — S7 Regulator (ORSS)

*🌍 [FR](../fr/manuel_utilisateur.md) · **EN** · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

---

## 1. What the instrument is for

**ORSS** simulates a process **regulation unit** (PID + first-order thermal process)
and exposes it as a **Siemens S7 PLC** (S7comm server over ISO-on-TCP). It is used to
test a supervision system or an S7 client (Snap7, TIA Portal for reading, nodes7…)
without a real PLC.

## 2. Getting started

```bash
cargo run -p mock_bin_ru_s7        # GUI + S7 server
```

The server listens by default on `0.0.0.0:102`. ⚠️ **Port 102 requires root
privileges**; otherwise, set a high port (e.g. 1102) in the *Settings* modal.

The header shows the status: **S7 ●** (green) with the listen address, or an error
message (red) if the bind fails. An orange banner warns if the server is **exposed**
(all interfaces + empty allowlist).

## 3. Interface

- **Header**: title, *Settings* / *Save* buttons, run/stop status, S7 listen status,
  network exposure banner.
- **Left panel (Commands)**: *Run/Stop*, *Automatic mode (PID)*, *Setpoint*, *Manual
  output* (manual mode), **PID** settings (Kp/Ki/Kd).
- **Center panel**: *Measurement / Setpoint / Output* cards + real-time **chart**.
- **\*Settings\* modal**: language, update check, **S7 network** (listen IP, port,
  **IP allowlist** — one pattern per line, `*` = wildcard), **process** (K, τ, delay,
  ambient), **setpoint bounds**. *Apply* restarts listening if the IP/port changes
  and saves the TOML.

## 4. Connecting an S7 client

The client connects to the server's IP/port. The usual **rack/slot** values (0/1 or
0/2) work: the server does not enforce a TSAP. The quantities are in **DB1** (see
[`reference_s7.md`](reference_s7.md)): setpoint at `DB1.DBD0`, measurement at
`DB1.DBD4`, run at `DB1.DBX16.0`, etc.

## 5. FAQ

- **"Permission denied" at startup** → port 102 requires root privileges; use a high
  port or launch with the appropriate privileges.
- **The client does not connect** → check IP/port, the **allowlist**, the firewall.
  Try rack/slot 0/1 then 0/2.
- **My writes have no effect** → only the controllable offsets act (setpoint, manual
  output, run, auto); the others are read-only.
- **Where is the config file?** → `mock_ru_s7.toml` (current directory; overridable
  via `MOCK_CONFIG`).
