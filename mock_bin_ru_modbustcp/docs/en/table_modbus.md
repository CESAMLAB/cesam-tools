# Modbus address table — Simulated controller

*🌍 [FR](../fr/table_modbus.md) · **EN** · [DE](../de/table_modbus.md) · [ES](../es/table_modbus.md) · [IT](../it/table_modbus.md) · [PT](../pt/table_modbus.md) · [NL](../nl/table_modbus.md) · [PL](../pl/table_modbus.md)*

> Crate: `mock_bin_ru_modbustcp` · Protocol: **Modbus TCP** (slave / server)

This document is the functional reference of the addressing plan. The **technical
source of truth** remains the header of [`src/map.rs`](../../src/map.rs): any
divergence must be fixed in the code first.

---

## 1. Generalities

| Element | Value |
|---------|--------|
| Transport | Modbus **TCP** or **serial RTU / RS485** (only one active at a time) |
| Role | **Slave** (server) |
| Default port | TCP `5502` (configurable, *Settings* modal) |
| Serial (RTU) | port + baud + parity + bits, configurable (`rtu` feature) |
| Unit ID / address | TCP: irrelevant. RTU: `slave_id` configurable but **not filtered** (see note) |
| Masters | **only one remote master at a time**; in TCP a newcomer disconnects the previous one (the local GUI is not a master) |
| Addressing | **base 0** (address `0` = 1st element of the table) |
| Filtering | optional IP allowlist (`*` wildcards, TCP only) |

> **RTU / slave address note**: the RTU server responds **whatever the address**
> requested (the address is not passed to the application service). Use a
> **point-to-point** link. The `slave_id` is kept/displayed but does not perform
> any filtering.

### Base 0 vs base 1 addressing

The addresses below are the **protocol addresses (base 0)**, as sent in the frame.
Many tools display a "conventional" base 1 numbering (`4xxxx` for holdings,
`3xxxx` for inputs…). Thus the holding register at address `2` corresponds to the
conventional landmark `40003`.

---

## 2. Encoding of floating-point numbers (`f32`)

Analog quantities are **IEEE-754 `f32` over 2 consecutive registers**:

- **word order**: **high word first** (big-endian, called *ABCD*);
- **byte order** within each register: big-endian (Modbus standard).

Example: `80.0` → bytes `42 A0 00 00` → register `n` = `0x42A0`, register `n+1` =
`0x0000`.

> If your client reads aberrant values, it is almost always a word-order problem
> (try *word swap* / *CDAB*).

---

## 3. Coils (read/write)

Function codes: `0x01` (read), `0x05` (single write), `0x0F` (multiple write).

| Address | Label | Values | Effect |
|---------|-------------|---------|-------|
| `0` | Run / Stop | `0` = stop, `1` = run | Enables control |
| `1` | Auto / Manual | `0` = manual, `1` = auto | Mode choice |

---

## 4. Discrete Inputs (read-only)

Function code: `0x02`.

| Address | Label | Meaning |
|---------|-------------|---------------|
| `0` | Running | The device is running |
| `1` | Direction 1 (heating) active | Output > 0 |
| `2` | Direction 2 (cooling) active | Output < 0 |

---

## 5. Holding Registers (read/write)

Function codes: `0x03` (read), `0x06` (single write), `0x10` (multiple write).

| Address | Label | Type | Unit / values |
|---------|-------------|------|-----------------|
| `0` | Control mode direction 1 (heating) | `u16` | `0`=Off, `1`=PID, `2`=TOR, `3`=PWM |
| `1` | Control mode direction 2 (cooling) | `u16` | `0`=Off, `1`=PID, `2`=TOR, `3`=PWM |
| `2`–`3` | Automatic setpoint (SP) | `f32` | measurement unit |
| `4`–`5` | Manual setpoint | `f32` | output %, signed (−100…+100) |
| `6`–`7` | `Kp` direction 1 | `f32` | proportional gain |
| `8`–`9` | `Ki` direction 1 | `f32` | integral gain (s⁻¹) |
| `10`–`11` | `Kd` direction 1 | `f32` | derivative gain (s) |
| `12`–`13` | `Kp` direction 2 | `f32` | proportional gain |
| `14`–`15` | `Ki` direction 2 | `f32` | integral gain (s⁻¹) |
| `16`–`17` | `Kd` direction 2 | `f32` | derivative gain (s) |
| `18`–`19` | TOR hysteresis | `f32` | measurement unit |
| `20`–`21` | Minimum TOR cycle time | `f32` | seconds (anti-short-cycle, `0` = disabled) |
| `22`–`23` | PWM cycle period | `f32` | seconds (> 0) |
| `42`–`46` | Device identifier | `ASCII` | "CESAM-Lab" (read-only, 2 chars/register, high byte first) |

> Registers `24`–`41` reserved (read as `0`).

> **Partial write of an `f32`**: you must write **both registers** of a float for
> it to be taken into account. A write of a single register of an `f32` pair is
> ignored (and returns the *Illegal Data Address* exception if it overlaps no
> valid field).
>
> Written gains are clamped to finite values ≥ 0 (robustness).

---

## 6. Input Registers (read-only)

Function code: `0x04`.

| Address | Label | Type | Unit |
|---------|-------------|------|-------|
| `0`–`1` | Measurement (PV — *process value*) | `f32` | measurement unit |
| `2`–`3` | Applied output | `f32` | signed % (+ heating / − cooling) |
| `4`–`5` | Auto setpoint readback (read-only) | `f32` | measurement unit |
| `6`–`7` | Manual setpoint readback (read-only) | `f32` | output %, signed (−100…+100) |

> **Setpoint readbacks**: registers `4`–`7` expose **read-only** the current value
> of the auto/manual setpoints (mirrors of holdings `2`–`5`). Handy for a supervisor
> that only **monitors** without writing.

---

## 7. Modbus exceptions

| Code | Name | Cause in this device |
|------|-----|--------------------------|
| `0x01` | Illegal Function | Unhandled function code (e.g. mask, FIFO) |
| `0x02` | Illegal Data Address | Address / quantity out of table, or write targeting no field |
| `0x04` | Server Device Failure | Internal lock unavailable (abnormal case) |

---

## 8. Examples with `mbpoll`

`mbpoll` addresses in **base 1**; we therefore add `1` to base 0 addresses.

```bash
# Start (coil base0 0 -> -t 0 -r 1) then switch to auto (coil 1)
mbpoll -m tcp -p 5502 -a 1 -t 0 -r 1 127.0.0.1 1     # On/Off = 1
mbpoll -m tcp -p 5502 -a 1 -t 0 -r 2 127.0.0.1 1     # Auto/Manual = 1 (auto)

# Write the auto setpoint (HR base0 2-3 -> -t 4:float -r 3) to 80.0
mbpoll -m tcp -p 5502 -a 1 -t 4:float -r 3 127.0.0.1 80.0

# Read the measurement PV (IR base0 0-1 -> -t 3:float -r 1)
mbpoll -m tcp -p 5502 -a 1 -t 3:float -r 1 127.0.0.1

# Read the output (IR base0 2-3 -> -t 3:float -r 3)
mbpoll -m tcp -p 5502 -a 1 -t 3:float -r 3 127.0.0.1
```

> Depending on the `mbpoll` version, the float word order may require the swap
> option. In case of an inconsistent value, check the word order.

---

## 9. Condensed memory map

```
Coils (RW)            DiscreteInputs (RO)     Holding (RW)              Input (RO)
0  On/Off             0  Running              0  Mode dir1 (u16)        0-1 PV (f32)
1  Auto/Manual        1  Heating active       1  Mode dir2 (u16)        2-3 Output (f32)
                      2  Cooling active       2-3  SP auto (f32)         4-5 SP auto (readback, RO)
                                              4-5  SP manual (f32)        6-7 SP manual (readback, RO)
                                              6-7  Kp1  8-9  Ki1  10-11 Kd1
                                              12-13 Kp2 14-15 Ki2 16-17 Kd2
                                              18-19 Hysteresis (f32)
                                              20-21 Min. TOR cycle (f32, s)
                                              22-23 PWM period (f32, s)
                                              42-46 ASCII identifier "CESAM-Lab"
```

> **ASCII identifier** (`HR 42-46`): "CESAM-Lab" encoded 2 characters per
> register, high byte character first (`42`=`'CE'`, `43`=`'SA'`, `44`=`'M-'`,
> `45`=`'La'`, `46`=`'b\0'`). Read-only. Example:
> `mbpoll -m tcp -p 5502 -a 1 -t 4 -r 43 -c 5 127.0.0.1` (base 1 registers 43..47).
