# EtherNet/IP Reference — tags & protocol (RU/EtherNet/IP)

*🌍 [FR](../fr/reference_ethernetip.md) · **EN** · [DE](../de/reference_ethernetip.md) · [ES](../es/reference_ethernetip.md) · [IT](../it/reference_ethernetip.md) · [PT](../pt/reference_ethernetip.md) · [NL](../nl/reference_ethernetip.md) · [PL](../pl/reference_ethernetip.md)*

> Source of truth: [`eip_server.rs`](../../src/eip_server.rs) (encapsulation,
> CIP dispatch, tag table). Every change is made **in that file** and reflected
> here.

---

## 1. Endpoint

**EtherNet/IP** adapter (unconnected **CIP** explicit messaging) over TCP. Listens by
default on `0.0.0.0:44818` (standard EtherNet/IP port, > 1024 → no privilege
required). Settings in the `[network]` section of the TOML / the *Settings* modal:

| Key | Default | Role |
|---|---|---|
| `bind_ip` | `0.0.0.0` | listen IP |
| `port` | `44818` | TCP port (standard EtherNet/IP) |
| `allowlist` | *(empty)* | IP allowlist (`*` patterns per octet; empty = all allowed) |

> ⚠️ **No authentication or encryption** (EtherNet/IP "classic"). The only access
> control is the **IP allowlist** + network topology. `0.0.0.0` + empty list =
> **exposed**: the GUI shows a warning banner.

⚠️ EtherNet/IP / CIP is **little-endian** (unlike Modbus/S7). The `REAL` values are
IEEE-754 little-endian `f32`.

## 2. Sessions

Multiple **simultaneous** clients are accepted. Each session: `RegisterSession`
(the server assigns a non-zero *session handle*) → `SendRRData` carrying the CIP
requests → `UnRegisterSession` (or TCP disconnect).

## 3. Implemented protocol subset

- **Encapsulation**: `RegisterSession` (0x0065), `UnRegisterSession` (0x0066),
  `SendRRData` (0x006F, unconnected explicit messaging, CPF).
- **CIP**: `Read Tag` (service 0x4C) and `Write Tag` (service 0x4D) on **named tags**
  (ANSI symbolic segment `0x91`).

## 4. Tag table

| Tag | CIP type | Access | Quantity | Write → command |
|---|---|:--:|---|---|
| `Setpoint` | REAL (0x00CA) | R/W | setpoint | `SetSetpoint` |
| `ProcessValue` | REAL | R | measurement | — |
| `Output` | REAL | R | output (%) | — |
| `ManualOutput` | REAL | R/W | manual output (%) | `SetManualOutput` |
| `Run` | BOOL (0x00C1) | R/W | run | `SetRun` |
| `Auto` | BOOL | R/W | auto mode | `SetAuto` |
| `SetpointMin` | REAL | R | min setpoint | — |
| `SetpointMax` | REAL | R | max setpoint | — |
| `Kp` / `Ki` / `Kd` | REAL | R | PID gains | — |

A known **read-only** tag that is written is **accepted** (CIP success status) but has
no effect; an **unknown tag** returns CIP status `0x05` (*path destination unknown*).
Every controllable write is **clamped/sanitized** by the simulation.

## 5. Client example

With an EtherNet/IP client (e.g. `pycomm3`, `rseip`, `rust-ethernet-ip`) pointing at
the server's IP/port, the tags are read/written by their name:

```python
from pycomm3 import CIPDriver  # or LogixDriver depending on the tool
# Read the measurement, write the setpoint and start the regulation:
#   read  Tag "ProcessValue" (REAL)
#   write Tag "Setpoint" = 80.0 (REAL)
#   write Tag "Run" = True (BOOL)
```

The server responds to the generic Read/Write Tag services addressed by ANSI symbolic
segment; it does not expose a CIP object tree beyond the tags above.
