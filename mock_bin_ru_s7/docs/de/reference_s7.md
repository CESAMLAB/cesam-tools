# S7-Referenz — Adressbelegung & Protokoll (RU/S7)

*🌍 [FR](../fr/reference_s7.md) · [EN](../en/reference_s7.md) · **DE** · [ES](../es/reference_s7.md) · [IT](../it/reference_s7.md) · [PT](../pt/reference_s7.md) · [NL](../nl/reference_s7.md) · [PL](../pl/reference_s7.md)*

> Quelle der Wahrheit: [`s7_server.rs`](../../src/s7_server.rs) (Telegrammanalyse,
> DB1-Adressbelegung, Mapping der Schreibvorgänge). Jede Weiterentwicklung erfolgt **in
> dieser Datei** und wird hier widergespiegelt.

---

## 1. Endpoint

**S7comm**-Server über **ISO-on-TCP / RFC1006**. Lauscht standardmäßig auf
`0.0.0.0:102` (S7-Standardport; **< 1024 → Root-Rechte** erforderlich, andernfalls einen
hohen Port wählen). Einstellungen im Abschnitt `[network]` der TOML-Datei / im Modal
*Parameter*:

| Schlüssel | Standard | Rolle |
|---|---|---|
| `bind_ip` | `0.0.0.0` | Lausch-IP |
| `port` | `102` | TCP-Port (S7-Standard) |
| `allowlist` | *(leer)* | IP-Whitelist (Muster `*` je Byte; leer = alles erlaubt) |

> ⚠️ **Keine Authentifizierung und keine Verschlüsselung** (S7 „classic“). Die einzige
> Zugangskontrolle ist die **IP-Whitelist** + die Netztopologie. `0.0.0.0` + leere Liste
> = **dem gesamten Netz exponiert**: die IHM zeigt ein Warnbanner an.

## 2. Sitzungen

Im Gegensatz zu ORME (Single-Master) nimmt der S7-Server **mehrere gleichzeitige
Client-Sitzungen** an (übliches Verhalten einer Steuerung). Jede Sitzung verhandelt COTP
(Connection Request → Confirm) dann S7 *Setup Communication*, vor den Austauschen
*Read Var* / *Write Var*.

## 3. Implementierte Protokoll-Teilmenge

- **COTP**: Connection Request (CR) → Connection Confirm (CC); Data (DT).
- **S7comm**: *Setup Communication*, *Read Var* (Funktion `0x04`), *Write Var*
  (Funktion `0x05`) auf dem Datenbaustein **DB1**.

Der Server stellt ein **DB1-Bytes-Abbild** (40 Byte) bereit. Die Lesevorgänge liefern
einen Ausschnitt dieses Abbilds; die Schreibvorgänge auf die steuerbaren Offsets
erzeugen bereinigte Befehle für die Simulation.

## 4. DB1-Adressbelegung

REAL = `f32` big-endian (IEEE-754). Adressierung byteweise (`DBDx`) oder bitweise
(`DBXx.y`).

| Adresse | Typ | Zugriff | Größe | Schreiben → Befehl |
|---|---|:--:|---|---|
| `DB1.DBD0`  | REAL | R/W | Sollwert (Setpoint) | `SetSetpoint` |
| `DB1.DBD4`  | REAL | R   | Messwert (ProcessValue) | — |
| `DB1.DBD8`  | REAL | R   | Ausgang (Output, %) | — |
| `DB1.DBD12` | REAL | R/W | Manueller Ausgang (ManualOutput, %) | `SetManualOutput` |
| `DB1.DBX16.0` | BOOL | R/W | Start (Run) | `SetRun` |
| `DB1.DBX16.1` | BOOL | R/W | Automatikmodus (Auto) | `SetAuto` |
| `DB1.DBD20` | REAL | R | Sollwert min | — |
| `DB1.DBD24` | REAL | R | Sollwert max | — |
| `DB1.DBD28` | REAL | R | PID Kp | — |
| `DB1.DBD32` | REAL | R | PID Ki | — |
| `DB1.DBD36` | REAL | R | PID Kd | — |

Das Schreiben von `DB1.DBB16` (Byte) wird akzeptiert: Bit 0 = Run, Bit 1 = Auto. Jeder
Schreibvorgang auf einen schreibgeschützten Offset wird **akzeptiert, aber ignoriert**
(Erfolgs-Rückgabecode). Ein Lese-/Schreibvorgang außerhalb von DB1 gibt den
S7-Rückgabecode `0x0A` (nicht existierendes Objekt) zurück.

## 5. Client-Beispiel

Mit einem S7-Client (Snap7, `python-snap7`, nodes7 …), der auf die IP/den Port des
Servers konfiguriert ist, **Rack 0 / Slot 1** (übliche Werte; der Server erzwingt keinen
TSAP):

```python
import snap7, struct
c = snap7.client.Client()
c.connect("127.0.0.1", 0, 1, 102)
c.db_write(1, 0, struct.pack(">f", 80.0))   # Sollwert = 80.0
c.db_write(1, 16, bytes([0x01]))            # Run = true (Bit 0)
pv = struct.unpack(">f", c.db_read(1, 4, 4))[0]  # Messwert
```
