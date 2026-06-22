# NAMUR-Befehlssatz — Simulierter Rührer (OSNE)

*🌍 [FR](../fr/commandes_namur.md) · [EN](../en/commandes_namur.md) · **DE** · [ES](../es/commandes_namur.md) · [IT](../it/commandes_namur.md) · [PT](../pt/commandes_namur.md) · [NL](../nl/commandes_namur.md) · [PL](../pl/commandes_namur.md)*

> Crate: `mock_bin_su_namur` · Ausführbare Datei: **OSNE** · Protokoll: **NAMUR** (ASCII, Slave)

Funktionale Referenz des Protokolls. Die **technische Quelle der Wahrheit** ist
der Kopf von [`src/namur.rs`](../../src/namur.rs).

---

## 1. Allgemeines

| Element | Wert |
|---------|------|
| Transport | **TCP** (Port `4001` standardmäßig) oder **seriell RS-232** (Feature `serial`) |
| Rolle | **Slave** (beantwortet Anfragen des Masters) |
| Rahmen | eine **ASCII-Zeile** pro Anfrage, abgeschlossen mit `CR LF` |
| Lesungen | `IN_*` → liefern `Wert Kanal` (z. B. `1200.0 4`) |
| Schreibvorgänge / Aktionen | `OUT_*`, `START_*`, `STOP_*`, `RESET` → **stumm** (keine Antwort) |
| Master | **nur einer gleichzeitig** (Punkt-zu-Punkt); bei TCP wartet ein neuer Master bis zur Trennung des vorherigen |
| Filterung | optionale IP-Whitelist (TCP) |

> Typische serielle NAMUR-Einstellung: **9600 Baud, 7 Bit, gerade Parität, 1 Stopp (7E1)**.

### Kanäle

| Kanal | Größe | Einheit |
|-------|-------|---------|
| `4` | Drehzahl | tr/min |
| `5` | Drehmoment | N·cm |

---

## 2. Befehle

| Befehl | Typ | Wirkung | Antwort |
|--------|-----|---------|---------|
| `IN_NAME` | Lesung | Gerätename | `CESAM-STIRRER` |
| `IN_TYPE` | Lesung | Gerätetyp | `OSNE` |
| `IN_SW_VERSION` | Lesung | Version der simulierten Firmware | z. B. `0.1.0` |
| `IN_PV_4` | Lesung | **gemessene** Drehzahl | `<v> 4` |
| `IN_PV_5` | Lesung | **gemessenes** Drehmoment | `<c> 5` |
| `IN_SP_4` | Lesung | Drehzahl-Sollwert | `<v> 4` |
| `OUT_SP_4 <v>` | Schreiben | Drehzahl-Sollwert **einstellen** (tr/min) | — |
| `START_4` | Aktion | Motor starten | — |
| `STOP_4` | Aktion | Motor stoppen | — |
| `RESET` | Aktion | Stopp + Rückkehr in lokale Steuerung | — |
| `OUT_WD1@<m>` | Schreiben | **Watchdog**: sicherer Stopp, wenn `<m>` s lang kein Befehl eintrifft | — |
| `OUT_WD2@<m>` | Schreiben | Watchdog (wie v1: sicherer Stopp) | — |

> Jeder unbekannte Befehl oder ungültige Parameter wird **ignoriert** (keine
> Antwort) und auf `debug`-Ebene protokolliert.

### Watchdog

Trifft nach `OUT_WD1@30` 30 s lang **keine Zeile** ein, wird der Motor automatisch
**gestoppt** (`STOP`) — Schutz bei Verlust der Kommunikation mit dem Leitsystem.
`OUT_WD1@0` deaktiviert den Watchdog. Der Zähler wird **bei jedem empfangenen
Befehl neu armiert**.

---

## 3. Beispiele (`nc` / netcat)

```text
$ nc 127.0.0.1 4001
IN_NAME
CESAM-STIRRER
OUT_SP_4 1200          (stumm)
START_4                (stumm)
IN_PV_4
1200.0 4
IN_PV_5
62.0 5
STOP_4                 (stumm)
```

> Das gelesene **Drehmoment** wächst mit der eingestellten **Viskosität** (IHM-
> seitig) und der Drehzahl: `Drehmoment ≈ Lastkoeffizient · Viskosität · Drehzahl +
> Reibung`. Bei hoher Viskosität sättigt das Drehmoment am Motormaximum: Die
> Solldrehzahl wird nicht mehr erreicht (**Überlast**), ein Verhalten, das einen
> realen Rührer nachbildet.
