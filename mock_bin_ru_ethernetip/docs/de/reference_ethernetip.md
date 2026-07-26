# EtherNet/IP-Referenz — Tags & Protokoll (RU/EtherNet/IP)

*🌍 [FR](../fr/reference_ethernetip.md) · [EN](../en/reference_ethernetip.md) · **DE** · [ES](../es/reference_ethernetip.md) · [IT](../it/reference_ethernetip.md) · [PT](../pt/reference_ethernetip.md) · [NL](../nl/reference_ethernetip.md) · [PL](../pl/reference_ethernetip.md)*

> Quelle der Wahrheit: [`eip_server.rs`](../../src/eip_server.rs) (Encapsulation,
> CIP-Dispatch, Tag-Tabelle). Jede Weiterentwicklung erfolgt **in dieser Datei** und
> wird hier widergespiegelt.

---

## 1. Endpoint

**EtherNet/IP**-Adapter (nicht verbundenes explizites **CIP**-Messaging) über TCP.
Lauscht standardmäßig auf `0.0.0.0:44818` (Standard-EtherNet/IP-Port, > 1024 → keine
Privilegien erforderlich). Einstellungen im Abschnitt `[network]` des TOML / im Modal
*Parameter*:

| Schlüssel | Standard | Rolle |
|---|---|---|
| `bind_ip` | `0.0.0.0` | Lausch-IP |
| `port` | `44818` | TCP-Port (EtherNet/IP-Standard) |
| `allowlist` | *(leer)* | IP-Whitelist (Muster `*` pro Oktett; leer = alles erlaubt) |

> ⚠️ **Weder Authentifizierung noch Verschlüsselung** (EtherNet/IP „classic“). Die
> einzige Zugangskontrolle ist die **IP-Whitelist** + die Netztopologie. `0.0.0.0` +
> leere Liste = **exponiert**: die IHM zeigt ein Warnbanner an.

⚠️ EtherNet/IP / CIP ist **little-endian** (im Gegensatz zu Modbus/S7). Die `REAL`
sind `f32` IEEE-754 little-endian.

## 2. Sitzungen

Mehrere **gleichzeitige** Clients werden angenommen. Jede Sitzung: `RegisterSession`
(der Server weist ein nicht null *session handle* zu) → `SendRRData`, das die
CIP-Anfragen trägt → `UnRegisterSession` (oder TCP-Trennung).

## 3. Implementierte Protokoll-Teilmenge

- **Encapsulation**: `RegisterSession` (0x0065), `UnRegisterSession` (0x0066),
  `SendRRData` (0x006F, nicht verbundenes explizites Messaging, CPF).
- **CIP**: `Read Tag` (Service 0x4C) und `Write Tag` (Service 0x4D) auf **benannten
  Tags** (symbolisches ANSI-Segment `0x91`).

## 4. Tag-Tabelle

| Tag | CIP-Typ | Zugriff | Größe | Schreiben → Befehl |
|---|---|:--:|---|---|
| `Setpoint` | REAL (0x00CA) | R/W | Sollwert | `SetSetpoint` |
| `ProcessValue` | REAL | R | Messwert | — |
| `Output` | REAL | R | Ausgabe (%) | — |
| `ManualOutput` | REAL | R/W | manuelle Ausgabe (%) | `SetManualOutput` |
| `Run` | BOOL (0x00C1) | R/W | Lauf | `SetRun` |
| `Auto` | BOOL | R/W | Automatikmodus | `SetAuto` |
| `SetpointMin` | REAL | R | Sollwert min | — |
| `SetpointMax` | REAL | R | Sollwert max | — |
| `Kp` / `Ki` / `Kd` | REAL | R | PID-Verstärkungen | — |

Ein bekanntes **schreibgeschütztes** Tag, das geschrieben wird, wird **angenommen**
(CIP-Status Erfolg), aber ohne Wirkung; ein **unbekanntes Tag** liefert den CIP-Status
`0x05` (*path destination unknown*). Jeder steuerbare Schreibvorgang wird von der
Simulation **geklemmt/bereinigt**.

## 5. Client-Beispiel

Mit einem EtherNet/IP-Client (z. B. `pycomm3`, `rseip`, `rust-ethernet-ip`), der auf
IP/Port des Servers zeigt, werden die Tags über ihren Namen gelesen/geschrieben:

```python
from pycomm3 import CIPDriver  # oder LogixDriver je nach Werkzeug
# Messwert lesen, Sollwert schreiben und Regelung starten:
#   read  Tag "ProcessValue" (REAL)
#   write Tag "Setpoint" = 80.0 (REAL)
#   write Tag "Run" = True (BOOL)
```

Der Server antwortet auf die generischen Read/Write-Tag-Dienste, die per symbolischem
ANSI-Segment adressiert werden; er stellt keinen CIP-Objektbaum über die obigen Tags
hinaus bereit.
