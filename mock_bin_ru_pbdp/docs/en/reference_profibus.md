# PROFIBUS DP-V0 Reference — Simulated Regulator (ORPD)

*🌍 [FR](../fr/reference_profibus.md) · **EN** · [DE](../de/reference_profibus.md) · [ES](../es/reference_profibus.md) · [IT](../it/reference_profibus.md) · [PT](../pt/reference_profibus.md) · [NL](../nl/reference_profibus.md) · [PL](../pl/reference_profibus.md)*

> Crate: `mock_bin_ru_pbdp` · Executable: **ru_pbdp** · Protocol: **PROFIBUS DP-V0** (serial slave)

This document is the functional reference for the simulated PROFIBUS DP-V0
subset. The **technical source of truth** remains the header of
[`src/profibus.rs`](../../src/profibus.rs) (codec + state machine) and
[`src/map.rs`](../../src/map.rs) (I/O blocks): any discrepancy must be fixed
in the code first.

---

## ⚠️ 0. Scope and limitations — read before any use

`ru_pbdp` implements an **educational subset** of DP-V0, with **no claim of
strict binary compliance** with the normative tables (IEC 61158 / EN 50170)
beyond the most universally documented elements:

- **compliant**: frame delimiters (`SD1`/`SD2`/`SD3`/`SD4`/`SC`/`ED`), FCS
  (modulo-256 sum), SAP numbers of the parametrization services
  (`Slave_Diag` = 61, `Set_Prm` = 62, `Chk_Cfg` = 63).
- **conventions specific to this simulator, not a real GSD profile
  registered with the PNO** (PROFIBUS & PROFINET International): exact
  encoding of the `FC` field bits, precise layout of the diagnostic bytes,
  layout of the input/output blocks (§3), the `Ident_Number` identifier (§4).
- **no real bus timing** whatsoever: neither a response window (*slot time*,
  `Tsdr` min/max), nor an inter-master token, nor multi-master arbitration.
  Only a dedicated ASIC (SPC3/VPC3) or hardware master card (Hilscher/
  Softing/Siemens CP) can meet these bit-level constraints.

**Direct consequence: this simulator will never be recognized by a real
PROFIBUS DP master** (PLC + master card). It serves to understand the
protocol's structure and to test software development (codec, state
machine, tooling), not to drive field equipment — see
[`manuel_utilisateur.md`](manuel_utilisateur.md).

---

## 1. Frames — delimiters and FCS

| Delimiter | Value | Usage |
|---|:--:|---|
| `SD1` | `0x10` | Fixed request without data (6 bytes: `SD1 DA SA FC FCS ED`) |
| `SD2` | `0x68` | Variable-length data frame (`SD2 LE LEr SD2 DA SA FC [data…] FCS ED`) |
| `SD3` | `0xA2` | Fixed-data frame, 8 bytes (14 bytes total) — **not used** by this simulator (see §0), provided for codec/test completeness |
| `SD4` | `0xDC` | Token frame, 3 bytes, no FCS or ED — out of scope for a simulated single-master slave, provided for codec completeness |
| `SC` | `0xE5` | Short acknowledgement, 1 byte |
| `ED` | `0x16` | End delimiter |

- **`FCS`**: modulo-256 sum of the frame's payload bytes (see
  `profibus::checksum`). A frame received with an incorrect FCS is rejected
  (`FrameError::BadChecksum`) without a reply — the master must retransmit.
- **`DA`/`SA`**: destination / source address. Bit 7 of `DA` = **address
  extension (DAE)**: presence of a SAP byte right after `DA` in the payload.
  Absent = default data exchange (`Data_Exchange`). The station address
  occupies the remaining 7 bits (`0`-`125`; `126`/`127` reserved by the
  standard, unused here).
- **This simulator always favours `SD2`** for all `Data_Exchange`
  exchanges, even when `SD3` (8 fixed bytes) would suffice in a real
  profile — a choice that simplifies the codec without losing any coverage
  of the protocol concepts (see [`conception.md`](conception.md) §4).
- **Malformed frame / unknown delimiter (line noise)**: silently rejected
  (`log::debug!`), the session continues — allows resynchronizing on the
  byte stream without crashing the link.

---

## 2. Sequencing — services and state machine

The simulated slave (`SlaveFsm`, [`profibus.rs`](../../src/profibus.rs))
goes through four states:

```
PowerOn ──Slave_Diag──► WaitPrm ──Set_Prm (ident OK)──► WaitCfg ──Chk_Cfg (sizes OK)──► DataExchange
```

| State | Meaning | Typical reply |
|---|---|---|
| `Power_On` | Right after startup, before the first diagnostic poll | — |
| `Wait_Prm` | Waiting for a valid `Set_Prm` | `Diag` with `Stat_1 = STAT1_PRM_REQ` |
| `Wait_Cfg` | Parametrized, waiting for a valid `Chk_Cfg` | `Diag` with `Stat_1 = STAT1_CFG_FAULT` |
| `Data_Exchange` | Parametrized and configured: cyclic exchange active | input block (§3) |

### `Slave_Diag` (SAP 61)

A request without data (or an `SD1` frame, always interpreted as
`Slave_Diag` by this simulator's convention — no address extension possible
on `SD1`, for lack of a spare byte to carry a SAP). `Diag` reply (6 bytes):

| Byte | Symbol | Content |
|:--:|---|---|
| `0` | `Stat_1` | `0x01` (`STAT1_PRM_REQ`, while not parametrized) or `0x02` (`STAT1_CFG_FAULT`, while not configured) or `0x00` (`Data_Exchange`) |
| `1` | `Stat_2` | always `0x00` (not simulated) |
| `2` | `Stat_3` | always `0x00` (not simulated) |
| `3` | `Master_Add` | `0xFF` (no known master — not tracked by this simulator) |
| `4-5` | `Ident_Number` | the slave's fixed identifier, big-endian (§4) |

The first `Slave_Diag` received moves `Power_On` → `Wait_Prm`; subsequent
ones don't change the state (just a diagnostic read).

### `Set_Prm` (SAP 62)

Request: `SAP(62) Ident_Number(2, BE) WD_Fact_1(1) WD_Fact_2(1)`. The
announced watchdog, if present, is computed as
`watchdog_ms = WD_Fact_1 × WD_Fact_2 × 10` (10 ms unit, standard DP
convention); `WD_Fact_1 = 0` **or** `WD_Fact_2 = 0` means "no watchdog".
Reply: `ShortAck` (`SC`) in all cases.

- If `Ident_Number` **matches** the slave's fixed profile (§4): state →
  `Wait_Cfg`, and any watchdog is passed on to the session (armed only if
  the local `watchdog_enabled` setting allows it — see
  [`manuel_utilisateur.md`](manuel_utilisateur.md) §4).
- If the identifier **doesn't match**: the parametrization is silently
  rejected (`ShortAck` returned anyway, as DP-V0 prescribes for this
  service, but without effect on the internal state) — the slave stays in
  `Wait_Prm`.

### `Chk_Cfg` (SAP 63)

Request: `SAP(63) Out_Len(1) In_Len(1)`. Reply: `ShortAck`. The state moves
to `Data_Exchange` **only if** `Out_Len == 45` and `In_Len == 17` (the
fixed sizes of the simulated profile, §3) **and** the slave was in
`Wait_Cfg`; otherwise the state doesn't change (the master must retransmit
a correct `Chk_Cfg`).

### `Data_Exchange` (no SAP — default address, DAE bit absent)

Request: the raw output block (45 bytes, §3). Reply: the input block (17
bytes, §3), recomputed on the fly from the shared snapshot at the moment of
replying (no persistent memory table, unlike Modbus/ORME).

If the master sends a `Data_Exchange` **before** reaching the
`Data_Exchange` state (sequencing not respected), the slave replies with the
current diagnostic (`Diag`) rather than crashing or ignoring the frame.

---

## 3. I/O blocks — byte layout

Copied from the header of [`map.rs`](../../src/map.rs), the sole source of
truth in case of discrepancy. All floats (`f32`) occupy **4 consecutive
bytes, big-endian**.

### Output block (master → slave, `OUTPUT_LEN` = 45 bytes)

| Byte(s) | Symbol | Type | Description |
|---|---|:--:|---|
| `0` | `OUT_MODE` | bits | bit0 = run, bit1 = auto, [3:2] = direction 1 mode, [5:4] = direction 2 mode |
| `1-4` | `OUT_SP_AUTO` | f32 | Automatic setpoint |
| `5-8` | `OUT_SP_MANUAL` | f32 | Manual setpoint (output %, signed) |
| `9-12` | `OUT_KP1` | f32 | Proportional gain Kp, direction 1 |
| `13-16` | `OUT_KI1` | f32 | Integral gain Ki, direction 1 |
| `17-20` | `OUT_KD1` | f32 | Derivative gain Kd, direction 1 |
| `21-24` | `OUT_KP2` | f32 | Proportional gain Kp, direction 2 |
| `25-28` | `OUT_KI2` | f32 | Integral gain Ki, direction 2 |
| `29-32` | `OUT_KD2` | f32 | Derivative gain Kd, direction 2 |
| `33-36` | `OUT_HYSTERESIS` | f32 | On-off controller hysteresis |
| `37-40` | `OUT_TOR_MIN_CYCLE` | f32 | Minimum on-off cycle time (s) |
| `41-44` | `OUT_PWM_PERIOD` | f32 | PWM modulation cycle period (s) |

The mode codes (`[3:2]`/`[5:4]`) follow `ControllerKind`: `0` = Off,
`1` = PID, `2` = On-off, `3` = PWM (see `mock_lib_control`).

### Input block (slave → master, `INPUT_LEN` = 17 bytes)

| Byte(s) | Symbol | Type | Description |
|---|---|:--:|---|
| `0` | `IN_STATUS` | bits | bit0 = running, bit1 = direction 1 active (output > 0), bit2 = direction 2 active (output < 0) |
| `1-4` | `IN_PV` | f32 | Measurement / process value |
| `5-8` | `IN_OUTPUT` | f32 | Applied output (signed %) |
| `9-12` | `IN_SP_AUTO` | f32 | Read-back (read-only) of the automatic setpoint |
| `13-16` | `IN_SP_MANUAL` | f32 | Read-back (read-only) of the manual setpoint |

An output block that is **too short** (< 45 bytes) is ignored without a
panic: no `Command` is produced, the regulator keeps its last valid state.

---

## 4. The slave's fixed profile

| Parameter | Value | Note |
|---|---|---|
| `Ident_Number` | `0xEE01` | **Fictitious**, not registered with the PNO — does not represent any real catalogue device |
| `Out_Len` | `45` | Expected in `Chk_Cfg.out_len` |
| `In_Len` | `17` | Expected in `Chk_Cfg.in_len` |
| Station address | `0`-`125`, configurable | Local setting (*Settings* modal), see [`manuel_utilisateur.md`](manuel_utilisateur.md) §4 |
| Serial frame format | `8E1` (8 bits, even parity, 1 stop bit) | **Fixed by the PROFIBUS DP standard**, not adjustable |
| Standard baud rates | `9600` to `12,000,000` bit/s | Not checked on open: a non-standard value is passed through to the serial port as-is |

---

## 5. Protocol watchdog

Unlike OSNE's NAMUR watchdog (a homemade addition), this one is a **genuine
part of the DP protocol**: it is **announced by the master** in `Set_Prm`
(`WD_Fact_1`/`WD_Fact_2` factors, §2) and is only **armed on the slave side**
if the local `watchdog_enabled` setting allows it (otherwise the master's
request is ignored, never armed). On expiry, with no new frame received for
the station, the slave forces the safe state
(`Command::SetOnOff(false)`) — a documented simplification: a real DP-V0
profile might require a full return through `Set_Prm`/`Chk_Cfg` before
resuming the exchange, which this simulator does not explicitly demand
(simply resuming to send `Data_Exchange` frames is enough, since the
`Data_Exchange` state is not left on watchdog expiry).

---

## 6. Non-interoperability — why

| Real PROFIBUS DP requirement | This simulator |
|---|---|
| Bit-level response window (*slot time*, `Tsdr` min/max) | Absent — replies as soon as the frame is decoded, with no time constraint |
| Dedicated circuit (SPC3/VPC3 ASIC) for timing | Absent — ordinary Tokio software |
| Inter-master token, multi-master arbitration | Absent — single-master slave, point-to-point link |
| GSD profile registered with the PNO | Absent — I/O profile specific to this simulator (§3) |
| Bit-exact encoding of the FC/diagnostic fields | Simulation convention, not guaranteed compliant |

**A real PLC (a Siemens S7 with a master card, for example) will never
recognize this simulator as a valid slave on a real PROFIBUS DP RS-485
bus.** Two instances of this simulator (or a script replaying the sequence
below), on the other hand, can talk to each other to illustrate the
protocol — see [`manuel_utilisateur.md`](manuel_utilisateur.md) §5.

---

## 7. Example sequence (hexadecimal)

Full sequence for station `5`, master `3`, up to the cyclic exchange
(illustrative values, `FCS` computed over the payload bytes):

```text
# 1. Slave_Diag (SD2, DAE=1, SAP=61)
→ TX  68 03 03 68 85 03 C0 3D FC 16
← RX  68 06 06 68 03 85 00 01 00 00 FF EE 01 F5 16   (Diag: Stat_1=0x01, Ident=0xEE01)

# 2. Set_Prm (SD2, DAE=1, SAP=62, Ident=0xEE01, WD=1×30×10ms=300ms)
→ TX  68 07 07 68 85 03 C0 3E EE 01 01 1E … 16
← RX  E5                                              (ShortAck)

# 3. Chk_Cfg (SD2, DAE=1, SAP=63, out_len=45, in_len=17)
→ TX  68 05 05 68 85 03 C0 3F 2D 11 … 16
← RX  E5                                              (ShortAck)

# 4. Data_Exchange (SD2, no SAP, 45-byte output block)
→ TX  68 30 30 68 05 03 C0 [45 bytes] … 16
← RX  68 14 14 68 03 85 00 [17 bytes]  … 16          (input block)
```

The exact FCS/length bytes depend on the payload values; this diagram
illustrates the **order of the services**, not a frame to replay verbatim.
See the tests in [`profibus.rs`](../../src/profibus.rs) and
[`profibus_server.rs`](../../src/profibus_server.rs) for bit-exact,
verified sequences.
