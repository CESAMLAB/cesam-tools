# S7 Reference — addressing plan & protocol (RU/S7)

*🌍 [FR](../fr/reference_s7.md) · **EN** · [DE](../de/reference_s7.md) · [ES](../es/reference_s7.md) · [IT](../it/reference_s7.md) · [PT](../pt/reference_s7.md) · [NL](../nl/reference_s7.md) · [PL](../pl/reference_s7.md)*

> Source of truth: [`s7_server.rs`](../../src/s7_server.rs) (frame parsing, DB1
> addressing plan, write mapping). Any change is made **in this file** and
> propagated here.

---

## 1. Endpoint

**S7comm** server over **ISO-on-TCP / RFC1006**. Listens by default on
`0.0.0.0:102` (standard S7 port; **< 1024 → root privileges** required, otherwise
choose a high port). Settings in the `[network]` section of the TOML / the *Settings*
modal:

| Key | Default | Role |
|---|---|---|
| `bind_ip` | `0.0.0.0` | listen IP |
| `port` | `102` | TCP port (S7 standard) |
| `allowlist` | *(empty)* | IP allowlist (`*` patterns per octet; empty = all allowed) |

> ⚠️ **No authentication or encryption** (S7 "classic"). The only access control is
> the **IP allowlist** + the network topology. `0.0.0.0` + empty list = **exposed to
> the whole network**: the GUI displays a warning banner.

## 2. Sessions

Unlike ORME (single-master), the S7 server accepts **several simultaneous client
sessions** (the usual behavior of a PLC). Each session negotiates COTP (Connection
Request → Confirm) then S7 *Setup Communication*, before the *Read Var* / *Write Var*
exchanges.

## 3. Implemented protocol subset

- **COTP**: Connection Request (CR) → Connection Confirm (CC); Data (DT).
- **S7comm**: *Setup Communication*, *Read Var* (function `0x04`), *Write Var*
  (function `0x05`) over the **DB1** data block.

The server exposes a **DB1 byte image** (40 bytes). Reads serve a slice of this
image; writes on the controllable offsets produce sanitized commands for the
simulation.

## 4. DB1 addressing plan

REAL = big-endian `f32` (IEEE-754). Addressing by byte (`DBDx`) or by bit (`DBXx.y`).

| Address | Type | Access | Quantity | Write → command |
|---|---|:--:|---|---|
| `DB1.DBD0`  | REAL | R/W | Setpoint | `SetSetpoint` |
| `DB1.DBD4`  | REAL | R   | Measurement (ProcessValue) | — |
| `DB1.DBD8`  | REAL | R   | Output (Output, %) | — |
| `DB1.DBD12` | REAL | R/W | Manual output (ManualOutput, %) | `SetManualOutput` |
| `DB1.DBX16.0` | BOOL | R/W | Run | `SetRun` |
| `DB1.DBX16.1` | BOOL | R/W | Auto mode (Auto) | `SetAuto` |
| `DB1.DBD20` | REAL | R | Setpoint min | — |
| `DB1.DBD24` | REAL | R | Setpoint max | — |
| `DB1.DBD28` | REAL | R | PID Kp | — |
| `DB1.DBD32` | REAL | R | PID Ki | — |
| `DB1.DBD36` | REAL | R | PID Kd | — |

A write to `DB1.DBB16` (byte) is accepted: bit 0 = Run, bit 1 = Auto. Any write to a
read-only offset is **accepted but ignored** (success return code). A read/write
outside DB1 returns the S7 return code `0x0A` (object does not exist).

## 5. Client example

With an S7 client (Snap7, `python-snap7`, nodes7…) configured on the server's
IP/port, **rack 0 / slot 1** (usual values; the server does not enforce the TSAP):

```python
import snap7, struct
c = snap7.client.Client()
c.connect("127.0.0.1", 0, 1, 102)
c.db_write(1, 0, struct.pack(">f", 80.0))   # Setpoint = 80.0
c.db_write(1, 16, bytes([0x01]))            # Run = true (bit 0)
pv = struct.unpack(">f", c.db_read(1, 4, 4))[0]  # Measurement
```
