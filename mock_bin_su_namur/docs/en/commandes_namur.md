# NAMUR command set — Simulated stirrer (OSNE)

*🌍 [FR](../fr/commandes_namur.md) · **EN** · [DE](../de/commandes_namur.md) · [ES](../es/commandes_namur.md) · [IT](../it/commandes_namur.md) · [PT](../pt/commandes_namur.md) · [NL](../nl/commandes_namur.md) · [PL](../pl/commandes_namur.md)*

> Crate: `mock_bin_su_namur` · Executable: **OSNE** · Protocol: **NAMUR** (ASCII, slave)

Functional reference for the protocol. The **technical source of truth** is the
header of [`src/namur.rs`](../../src/namur.rs).

---

## 1. General

| Item | Value |
|------|-------|
| Transport | **TCP** (port `4001` by default) or **serial RS-232** (feature `serial`) |
| Role | **Slave** (answers master requests) |
| Frame | one **ASCII line** per request, terminated by `CR LF` |
| Reads | `IN_*` → return `value channel` (e.g. `1200.0 4`) |
| Writes / actions | `OUT_*`, `START_*`, `STOP_*`, `RESET` → **silent** (no reply) |
| Masters | **one at a time** (point-to-point); over TCP a new master waits until the previous one disconnects |
| Filtering | optional IP allowlist (TCP) |

> Typical NAMUR serial setting: **9600 baud, 7 bits, even parity, 1 stop (7E1)**.

### Channels

| Channel | Quantity | Unit |
|---------|----------|------|
| `4` | Speed | tr/min |
| `5` | Torque | N·cm |

---

## 2. Commands

| Command | Type | Effect | Reply |
|---------|------|--------|-------|
| `IN_NAME` | read | Device name | `CESAM-STIRRER` |
| `IN_TYPE` | read | Device type | `OSNE` |
| `IN_SW_VERSION` | read | Simulated firmware version | e.g. `0.1.0` |
| `IN_PV_4` | read | **Measured** speed | `<v> 4` |
| `IN_PV_5` | read | **Measured** torque | `<c> 5` |
| `IN_SP_4` | read | Speed setpoint | `<v> 4` |
| `OUT_SP_4 <v>` | write | **Set** the speed setpoint (tr/min) | — |
| `START_4` | action | Start the motor | — |
| `STOP_4` | action | Stop the motor | — |
| `RESET` | action | Stop + return to local control | — |
| `OUT_WD1@<m>` | write | **Watchdog**: safe stop if no command for `<m>` s | — |
| `OUT_WD2@<m>` | write | Watchdog (same as v1: safe stop) | — |

> Any unknown command or invalid argument is **ignored** (no reply) and logged at
> `debug` level.

### Watchdog

After `OUT_WD1@30`, if **no line** arrives for 30 s, the motor is automatically
**stopped** (`STOP`) — protection against loss of communication with the
supervisor. `OUT_WD1@0` disarms the watchdog. The counter is **rearmed on every
command received**.

---

## 3. Examples (`nc` / netcat)

```text
$ nc 127.0.0.1 4001
IN_NAME
CESAM-STIRRER
OUT_SP_4 1200          (silent)
START_4                (silent)
IN_PV_4
1200.0 4
IN_PV_5
62.0 5
STOP_4                 (silent)
```

> The **torque** read grows with the **viscosity** set (in the GUI) and the speed:
> `torque ≈ load_coeff · viscosity · speed + friction`. At high viscosity, the
> torque saturates at the motor maximum: the setpoint speed is no longer reached
> (**overload**), a behaviour that reproduces a real stirrer.
