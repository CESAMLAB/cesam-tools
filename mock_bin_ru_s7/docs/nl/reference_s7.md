# S7-referentie — adresseringsplan & protocol (RU/S7)

*🌍 [FR](../fr/reference_s7.md) · [EN](../en/reference_s7.md) · [DE](../de/reference_s7.md) · [ES](../es/reference_s7.md) · [IT](../it/reference_s7.md) · [PT](../pt/reference_s7.md) · **NL** · [PL](../pl/reference_s7.md)*

> Bron van waarheid: [`s7_server.rs`](../../src/s7_server.rs) (analyse van de frames,
> DB1-adresseringsplan, mapping van schrijfacties). Elke evolutie gebeurt **in dit
> bestand** en wordt hier weerspiegeld.

---

## 1. Endpoint

**S7comm**-server op **ISO-on-TCP / RFC1006**. Luistert standaard op `0.0.0.0:102`
(standaard S7-poort; **< 1024 → root-rechten** vereist, kies anders een hoge poort).
Instellingen in de sectie `[network]` van het TOML / het modaal *Parameters*:

| Sleutel | Standaard | Rol |
|---|---|---|
| `bind_ip` | `0.0.0.0` | luister-IP |
| `port` | `102` | TCP-poort (S7-standaard) |
| `allowlist` | *(leeg)* | IP-toelatingslijst (patronen `*` per byte; leeg = alles toegestaan) |

> ⚠️ **Geen authenticatie noch versleuteling** (S7 "classic"). De enige
> toegangscontrole is de **IP-toelatingslijst** + de netwerktopologie. `0.0.0.0` + lege
> lijst = **blootgesteld aan het hele netwerk**: de GUI toont een waarschuwingsbanner.

## 2. Sessies

In tegenstelling tot ORME (single-master) accepteert de S7-server **meerdere
gelijktijdige clientsessies** (gebruikelijk gedrag van een PLC). Elke sessie
onderhandelt COTP (Connection Request → Confirm) en daarna S7 *Setup Communication*,
vóór de uitwisselingen *Read Var* / *Write Var*.

## 3. Geïmplementeerde protocol-deelverzameling

- **COTP**: Connection Request (CR) → Connection Confirm (CC); Data (DT).
- **S7comm**: *Setup Communication*, *Read Var* (functie `0x04`), *Write Var*
  (functie `0x05`) op het datablok **DB1**.

De server stelt een **byte-image van DB1** beschikbaar (40 bytes). De leesacties dienen
een segment van deze image; de schrijfacties op de stuurbare offsets produceren
gesaneerde commando's voor de simulatie.

## 4. DB1-adresseringsplan

REAL = `f32` big-endian (IEEE-754). Adressering per byte (`DBDx`) of per bit
(`DBXx.y`).

| Adres | Type | Toegang | Grootheid | Schrijven → commando |
|---|---|:--:|---|---|
| `DB1.DBD0`  | REAL | R/W | Setpoint | `SetSetpoint` |
| `DB1.DBD4`  | REAL | R   | Meting (ProcessValue) | — |
| `DB1.DBD8`  | REAL | R   | Uitgang (Output, %) | — |
| `DB1.DBD12` | REAL | R/W | Handmatige uitgang (ManualOutput, %) | `SetManualOutput` |
| `DB1.DBX16.0` | BOOL | R/W | Start (Run) | `SetRun` |
| `DB1.DBX16.1` | BOOL | R/W | Automodus (Auto) | `SetAuto` |
| `DB1.DBD20` | REAL | R | Setpoint min | — |
| `DB1.DBD24` | REAL | R | Setpoint max | — |
| `DB1.DBD28` | REAL | R | PID Kp | — |
| `DB1.DBD32` | REAL | R | PID Ki | — |
| `DB1.DBD36` | REAL | R | PID Kd | — |

Schrijven van `DB1.DBB16` (byte) geaccepteerd: bit 0 = Run, bit 1 = Auto. Elke
schrijfactie op een alleen-lezen offset wordt **geaccepteerd maar genegeerd**
(retourcode succes). Een lees-/schrijfactie buiten DB1 retourneert de S7-retourcode
`0x0A` (object bestaat niet).

## 5. Clientvoorbeeld

Met een S7-client (Snap7, `python-snap7`, nodes7…) geconfigureerd op het IP/de poort
van de server, **rack 0 / slot 1** (gebruikelijke waarden; de server legt de TSAP niet
op):

```python
import snap7, struct
c = snap7.client.Client()
c.connect("127.0.0.1", 0, 1, 102)
c.db_write(1, 0, struct.pack(">f", 80.0))   # Setpoint = 80.0
c.db_write(1, 16, bytes([0x01]))            # Run = true (bit 0)
pv = struct.unpack(">f", c.db_read(1, 4, 4))[0]  # Meting
```
