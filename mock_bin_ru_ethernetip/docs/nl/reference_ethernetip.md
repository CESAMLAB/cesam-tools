# EtherNet/IP-referentie — tags & protocol (RU/EtherNet/IP)

*🌍 [FR](../fr/reference_ethernetip.md) · [EN](../en/reference_ethernetip.md) · [DE](../de/reference_ethernetip.md) · [ES](../es/reference_ethernetip.md) · [IT](../it/reference_ethernetip.md) · [PT](../pt/reference_ethernetip.md) · **NL** · [PL](../pl/reference_ethernetip.md)*

> Bron van waarheid: [`eip_server.rs`](../../src/eip_server.rs) (encapsulatie,
> CIP-dispatch, tagtabel). Elke wijziging gebeurt **in dit bestand** en wordt
> hier doorgevoerd.

---

## 1. Endpoint

**EtherNet/IP**-adapter (expliciete **CIP**-berichten, niet-verbonden) over TCP.
Luistert standaard op `0.0.0.0:44818` (standaard EtherNet/IP-poort, > 1024 → geen
privilege vereist). Instellingen in de sectie `[network]` van de TOML / het modaal
*Parameters*:

| Sleutel | Standaard | Rol |
|---|---|---|
| `bind_ip` | `0.0.0.0` | IP voor luisteren |
| `port` | `44818` | TCP-poort (standaard EtherNet/IP) |
| `allowlist` | *(leeg)* | IP-toegangslijst (`*`-patronen per octet; leeg = alles toegestaan) |

> ⚠️ **Geen authenticatie noch versleuteling** (EtherNet/IP "classic"). De enige
> toegangscontrole is de **IP-toegangslijst** + de netwerktopologie. `0.0.0.0` +
> lege lijst = **blootgesteld**: de GUI toont een waarschuwingsbanner.

⚠️ EtherNet/IP / CIP is **little-endian** (in tegenstelling tot Modbus/S7). De `REAL`'s
zijn `f32`'s IEEE-754 little-endian.

## 2. Sessies

Meerdere **gelijktijdige** clients worden geaccepteerd. Elke sessie: `RegisterSession`
(de server kent een niet-nul *session handle* toe) → `SendRRData` met de CIP-aanvragen
→ `UnRegisterSession` (of TCP-verbreking).

## 3. Geïmplementeerde protocolsubset

- **Encapsulatie**: `RegisterSession` (0x0065), `UnRegisterSession` (0x0066),
  `SendRRData` (0x006F, expliciete niet-verbonden berichten, CPF).
- **CIP**: `Read Tag` (service 0x4C) en `Write Tag` (service 0x4D) op **benoemde
  tags** (ANSI symbolisch segment `0x91`).

## 4. Tagtabel

| Tag | CIP-type | Toegang | Grootheid | Schrijven → opdracht |
|---|---|:--:|---|---|
| `Setpoint` | REAL (0x00CA) | R/W | setpoint | `SetSetpoint` |
| `ProcessValue` | REAL | R | meting | — |
| `Output` | REAL | R | uitgang (%) | — |
| `ManualOutput` | REAL | R/W | handmatige uitgang (%) | `SetManualOutput` |
| `Run` | BOOL (0x00C1) | R/W | aan | `SetRun` |
| `Auto` | BOOL | R/W | automodus | `SetAuto` |
| `SetpointMin` | REAL | R | setpoint min | — |
| `SetpointMax` | REAL | R | setpoint max | — |
| `Kp` / `Ki` / `Kd` | REAL | R | PID-versterkingen | — |

Een bekende **alleen-lezen**-tag die wordt geschreven, wordt **geaccepteerd** (CIP-status succes) maar zonder
effect; een **onbekende tag** geeft de CIP-status `0x05` terug (*path destination unknown*).
Elke aanstuurbare schrijfactie wordt **geklemd/ontsmet** door de simulatie.

## 5. Clientvoorbeeld

Met een EtherNet/IP-client (bv. `pycomm3`, `rseip`, `rust-ethernet-ip`) gericht
op het IP/de poort van de server, worden de tags gelezen/geschreven via hun naam:

```python
from pycomm3 import CIPDriver  # of LogixDriver afhankelijk van de tool
# De meting lezen, de setpoint schrijven en de regeling starten:
#   read  Tag "ProcessValue" (REAL)
#   write Tag "Setpoint" = 80.0 (REAL)
#   write Tag "Run" = True (BOOL)
```

De server beantwoordt de generieke services Read/Write Tag geadresseerd via ANSI
symbolisch segment; hij stelt geen CIP-objectboom beschikbaar buiten de bovenstaande
tags.
