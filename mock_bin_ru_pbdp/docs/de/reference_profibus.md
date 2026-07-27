# PROFIBUS-DP-V0-Referenz — Simulierter Regler (ORPD)

*🌍 [FR](../fr/reference_profibus.md) · [EN](../en/reference_profibus.md) · **DE** · [ES](../es/reference_profibus.md) · [IT](../it/reference_profibus.md) · [PT](../pt/reference_profibus.md) · [NL](../nl/reference_profibus.md) · [PL](../pl/reference_profibus.md)*

> Crate: `mock_bin_ru_pbdp` · Ausführbare Datei: **ru_pbdp** · Protokoll: **PROFIBUS DP-V0** (serieller Slave)

Dieses Dokument ist die funktionale Referenz für die simulierte
PROFIBUS-DP-V0-Teilmenge. Die **technische Quelle der Wahrheit** bleibt der
Kopf von [`src/profibus.rs`](../../src/profibus.rs) (Codec + Zustandsmaschine)
und [`src/map.rs`](../../src/map.rs) (E/A-Blöcke): jede Abweichung muss
zuerst im Code korrigiert werden.

---

## ⚠️ 0. Umfang und Grenzen — vor jeder Nutzung lesen

`ru_pbdp` implementiert eine **pädagogische Teilmenge** von DP-V0, **ohne
jeglichen Anspruch auf strikte binäre Konformität** mit den normativen
Tabellen (IEC 61158 / EN 50170), über die am universellsten dokumentierten
Elemente hinaus:

- **konform**: Telegrammbegrenzer (`SD1`/`SD2`/`SD3`/`SD4`/`SC`/`ED`), FCS
  (Summe modulo 256), SAP-Nummern der Parametrierungsdienste
  (`Slave_Diag` = 61, `Set_Prm` = 62, `Chk_Cfg` = 63).
- **simulatorspezifische Konventionen, kein beim PNO registriertes echtes
  GSD-Profil** (PROFIBUS & PROFINET International): exakte Kodierung der
  `FC`-Feld-Bits, genaue Anordnung der Diagnosebytes, Anordnung der
  Eingangs-/Ausgangsblöcke (§3), die Kennung `Ident_Number` (§4).
- **kein reales Bus-Timing**: weder ein Antwortfenster (*Slot Time*, `Tsdr`
  min/max), noch ein Inter-Master-Token, noch eine
  Multi-Master-Arbitrierung. Nur ein dedizierter ASIC (SPC3/VPC3) oder eine
  Hardware-Masterkarte (Hilscher/Softing/Siemens CP) kann diese
  Bit-Ebene-Anforderungen erfüllen.

**Direkte Konsequenz: Dieser Simulator wird von einem echten
PROFIBUS-DP-Master niemals erkannt** (Steuerung + Masterkarte). Er dient
dazu, die Struktur des Protokolls zu verstehen und eine
Software-Entwicklung zu testen (Codec, Zustandsmaschine, Werkzeuge), nicht
dazu, Feldgeräte zu steuern — siehe
[`manuel_utilisateur.md`](manuel_utilisateur.md).

---

## 1. Telegramme — Begrenzer und FCS

| Begrenzer | Wert | Verwendung |
|---|:--:|---|
| `SD1` | `0x10` | Feste Anfrage ohne Daten (6 Bytes: `SD1 DA SA FC FCS ED`) |
| `SD2` | `0x68` | Telegramm variabler Länge mit Daten (`SD2 LE LEr SD2 DA SA FC [Daten…] FCS ED`) |
| `SD3` | `0xA2` | Telegramm mit festen Daten, 8 Bytes (14 Bytes insgesamt) — von diesem Simulator **nicht verwendet** (siehe §0), zur Vollständigkeit des Codecs und seiner Tests bereitgestellt |
| `SD4` | `0xDC` | Token-Telegramm, 3 Bytes, ohne FCS oder ED — für einen simulierten Single-Master-Slave nicht relevant, zur Vollständigkeit des Codecs bereitgestellt |
| `SC` | `0xE5` | Kurzquittung, 1 Byte |
| `ED` | `0x16` | Endebegrenzer |

- **`FCS`**: Summe modulo 256 der Nutzdatenbytes des Telegramms (siehe
  `profibus::checksum`). Ein Telegramm mit falscher FCS wird ohne Antwort
  verworfen (`FrameError::BadChecksum`) — der Master muss erneut senden.
- **`DA`/`SA`**: Ziel-/Quelladresse. Bit 7 von `DA` = **Adresserweiterung
  (DAE)**: Vorhandensein eines SAP-Bytes direkt nach `DA` in den Nutzdaten.
  Fehlt es, bedeutet dies Standard-Datenaustausch (`Data_Exchange`). Die
  Stationsadresse belegt die verbleibenden 7 Bits (`0`-`125`; `126`/`127`
  durch die Norm reserviert, hier ungenutzt).
- **Dieser Simulator bevorzugt systematisch `SD2`** für alle
  `Data_Exchange`-Austausche, auch wenn `SD3` (8 feste Bytes) in einem
  echten Profil ausreichen würde — eine Entscheidung, die den Codec
  vereinfacht, ohne an Abdeckung der Protokollkonzepte zu verlieren (siehe
  [`conception.md`](conception.md) §4).
- **Fehlerhaftes Telegramm / unbekannter Begrenzer (Leitungsrauschen)**:
  wird stillschweigend verworfen (`log::debug!`), die Sitzung läuft weiter
  — ermöglicht eine Resynchronisierung auf den Byte-Strom, ohne die
  Verbindung abstürzen zu lassen.

---

## 2. Ablauf — Dienste und Zustandsmaschine

Der simulierte Slave (`SlaveFsm`, [`profibus.rs`](../../src/profibus.rs))
durchläuft vier Zustände:

```
PowerOn ──Slave_Diag──► WaitPrm ──Set_Prm (Kennung OK)──► WaitCfg ──Chk_Cfg (Längen OK)──► DataExchange
```

| Zustand | Bedeutung | Typische Antwort |
|---|---|---|
| `Power_On` | Direkt nach dem Start, vor der ersten Diagnoseabfrage | — |
| `Wait_Prm` | Wartet auf ein gültiges `Set_Prm` | `Diag` mit `Stat_1 = STAT1_PRM_REQ` |
| `Wait_Cfg` | Parametriert, wartet auf ein gültiges `Chk_Cfg` | `Diag` mit `Stat_1 = STAT1_CFG_FAULT` |
| `Data_Exchange` | Parametriert und konfiguriert: zyklischer Austausch aktiv | Eingangsblock (§3) |

### `Slave_Diag` (SAP 61)

Anfrage ohne Daten (oder ein `SD1`-Telegramm, nach der Konvention dieses
Simulators stets als `Slave_Diag` interpretiert — auf `SD1` ist keine
Adresserweiterung möglich, da kein Byte übrig ist, um ein SAP zu tragen).
`Diag`-Antwort (6 Bytes):

| Byte | Symbol | Inhalt |
|:--:|---|---|
| `0` | `Stat_1` | `0x01` (`STAT1_PRM_REQ`, solange nicht parametriert) oder `0x02` (`STAT1_CFG_FAULT`, solange nicht konfiguriert) oder `0x00` (`Data_Exchange`) |
| `1` | `Stat_2` | immer `0x00` (nicht simuliert) |
| `2` | `Stat_3` | immer `0x00` (nicht simuliert) |
| `3` | `Master_Add` | `0xFF` (kein bekannter Master — von diesem Simulator nicht verfolgt) |
| `4-5` | `Ident_Number` | feste Kennung des Slaves, Big-Endian (§4) |

Das erste empfangene `Slave_Diag` bewirkt den Übergang `Power_On` →
`Wait_Prm`; nachfolgende ändern den Zustand nicht (nur ein Diagnoselesen).

### `Set_Prm` (SAP 62)

Anfrage (Standard-DP-V0-Format, **entspricht** dem, was ein echter Master
sendet — z. B. `profirust` — keine simulatorspezifische Konvention, im
Unterschied zum I/O-Blocklayout in §3):

```
SAP(62) Station_Status(1) WD_Fact_1(1) WD_Fact_2(1) Min_Tsdr(1) Ident_Number(2, BE) Groups(1) [User_Prm_Data...]
```

`Station_Status` (Bits Lock_Req/Sync_Req/Freeze_Req/WD_On), `Min_Tsdr`,
`Groups` und `User_Prm_Data` werden von diesem Simulator **nicht ausgewertet**
(keine Sperre, kein Sync-/Freeze-Modus, keine Gruppen modelliert); nur
`WD_Fact_1`/`WD_Fact_2` und `Ident_Number` werden gelesen. Der angekündigte
Watchdog wird, falls vorhanden, als
`watchdog_ms = WD_Fact_1 × WD_Fact_2 × 10` berechnet (Einheit 10 ms,
Standard-DP-Konvention); `WD_Fact_1 = 0` **oder** `WD_Fact_2 = 0` bedeutet
„kein Watchdog“. Antwort: in jedem Fall `ShortAck` (`SC`).

- Entspricht `Ident_Number` dem festen Profil des Slaves (§4): Zustand →
  `Wait_Cfg`, und ein etwaiger Watchdog wird an die Sitzung übergeben (nur
  scharfgeschaltet, wenn die lokale Einstellung `watchdog_enabled` es
  erlaubt — siehe [`manuel_utilisateur.md`](manuel_utilisateur.md) §4).
- Entspricht die Kennung **nicht**: Die Parametrierung wird stillschweigend
  abgelehnt (`ShortAck` wird trotzdem zurückgegeben, wie DP-V0 es für
  diesen Dienst vorschreibt, jedoch ohne Auswirkung auf den internen
  Zustand) — der Slave bleibt in `Wait_Prm`.

### `Chk_Cfg` (SAP 63)

Anfrage: `SAP(63) Out_Len(1) In_Len(1)`. Antwort: `ShortAck`. Der Zustand
geht **nur dann** zu `Data_Exchange` über, wenn `Out_Len == 45` und
`In_Len == 17` (die festen Größen des simulierten Profils, §3) **und** sich
der Slave in `Wait_Cfg` befand; andernfalls ändert sich der Zustand nicht
(der Master muss ein korrektes `Chk_Cfg` erneut senden).

### `Data_Exchange` (kein SAP — Standardadresse, DAE-Bit fehlt)

Anfrage: der rohe Ausgangsblock (45 Bytes, §3). Antwort: der Eingangsblock
(17 Bytes, §3), der im Moment der Antwort spontan aus dem gemeinsamen
Snapshot neu berechnet wird (keine persistente Speichertabelle, anders als
bei Modbus/ORME).

Sendet der Master ein `Data_Exchange` **bevor** der Zustand `Data_Exchange`
erreicht ist (Ablauf nicht eingehalten), antwortet der Slave mit der
aktuellen Diagnose (`Diag`), anstatt abzustürzen oder das Telegramm zu
ignorieren.

---

## 3. E/A-Blöcke — Byte-Anordnung

Übernommen aus dem Kopf von [`map.rs`](../../src/map.rs), der alleinigen
Quelle der Wahrheit im Falle einer Abweichung. Alle Gleitkommawerte (`f32`)
belegen **4 aufeinanderfolgende Bytes, Big-Endian**.

### Ausgangsblock — *Output* (Master → Slave, `OUTPUT_LEN` = 45 Bytes)

| Byte(s) | Symbol | Typ | Beschreibung |
|---|---|:--:|---|
| `0` | `OUT_MODE` | Bits | Bit0 = Lauf, Bit1 = Auto, [3:2] = Modus Richtung 1, [5:4] = Modus Richtung 2 |
| `1-4` | `OUT_SP_AUTO` | f32 | Automatischer Sollwert |
| `5-8` | `OUT_SP_MANUAL` | f32 | Manueller Sollwert (% Ausgang, vorzeichenbehaftet) |
| `9-12` | `OUT_KP1` | f32 | Proportionalverstärkung Kp, Richtung 1 |
| `13-16` | `OUT_KI1` | f32 | Integralverstärkung Ki, Richtung 1 |
| `17-20` | `OUT_KD1` | f32 | Differentialverstärkung Kd, Richtung 1 |
| `21-24` | `OUT_KP2` | f32 | Proportionalverstärkung Kp, Richtung 2 |
| `25-28` | `OUT_KI2` | f32 | Integralverstärkung Ki, Richtung 2 |
| `29-32` | `OUT_KD2` | f32 | Differentialverstärkung Kd, Richtung 2 |
| `33-36` | `OUT_HYSTERESIS` | f32 | Hysterese der Zweipunktregler |
| `37-40` | `OUT_TOR_MIN_CYCLE` | f32 | Minimale Zweipunkt-Zykluszeit (s) |
| `41-44` | `OUT_PWM_PERIOD` | f32 | Periode des PWM-Modulationszyklus (s) |

Die Moduscodes (`[3:2]`/`[5:4]`) folgen `ControllerKind`: `0` = Aus,
`1` = PID, `2` = Zweipunkt, `3` = PWM (siehe `mock_lib_control`).

### Eingangsblock — *Input* (Slave → Master, `INPUT_LEN` = 17 Bytes)

| Byte(s) | Symbol | Typ | Beschreibung |
|---|---|:--:|---|
| `0` | `IN_STATUS` | Bits | Bit0 = in Betrieb, Bit1 = Richtung 1 aktiv (Ausgang > 0), Bit2 = Richtung 2 aktiv (Ausgang < 0) |
| `1-4` | `IN_PV` | f32 | Messwert / *Process Value* |
| `5-8` | `IN_OUTPUT` | f32 | Angewendeter Ausgang (vorzeichenbehaftetes %) |
| `9-12` | `IN_SP_AUTO` | f32 | Rückmeldung (nur lesend) des automatischen Sollwerts |
| `13-16` | `IN_SP_MANUAL` | f32 | Rückmeldung (nur lesend) des manuellen Sollwerts |

Ein zu kurzer Ausgangsblock (< 45 Bytes) wird ohne Absturz ignoriert: es
wird kein `Command` erzeugt, der Regler behält seinen letzten gültigen
Zustand bei.

---

## 4. Festes Profil des Slaves

| Parameter | Wert | Anmerkung |
|---|---|---|
| `Ident_Number` | `0xEE01` | **Fiktiv**, nicht beim PNO registriert — stellt kein reales Katalog-Gerät dar |
| `Out_Len` | `45` | Erwartet in `Chk_Cfg.out_len` |
| `In_Len` | `17` | Erwartet in `Chk_Cfg.in_len` |
| Stationsadresse | `0`-`125`, konfigurierbar | Lokale Einstellung (*Einstellungen*-Modal), siehe [`manuel_utilisateur.md`](manuel_utilisateur.md) §4 |
| Serielles Telegrammformat | `8E1` (8 Bit, gerade Parität, 1 Stoppbit) | **Durch die PROFIBUS-DP-Norm festgelegt**, nicht einstellbar |
| Normierte Baudraten | `9600` bis `12.000.000` bit/s | Beim Öffnen nicht geprüft: ein nicht normierter Wert wird unverändert an den seriellen Port weitergegeben |

---

## 5. Protokoll-Watchdog

Anders als OSNEs NAMUR-Watchdog (ein selbstgemachter Zusatz) ist dieser ein
**echter Bestandteil des DP-Protokolls**: er wird **vom Master** in
`Set_Prm` angekündigt (Faktoren `WD_Fact_1`/`WD_Fact_2`, §2) und wird
slaveseitig nur dann **scharfgeschaltet**, wenn die lokale Einstellung
`watchdog_enabled` es erlaubt (andernfalls wird die Anforderung des Masters
ignoriert, nie scharfgeschaltet). Bei Ablauf, ohne dass für die Station ein
neues Telegramm empfangen wurde, erzwingt der Slave den sicheren Zustand
(`Command::SetOnOff(false)`) — eine dokumentierte Vereinfachung: ein echtes
DP-V0-Profil könnte eine vollständige Rückkehr über `Set_Prm`/`Chk_Cfg`
verlangen, bevor der Austausch fortgesetzt wird, was dieser Simulator nicht
ausdrücklich verlangt (es genügt, das Senden von `Data_Exchange`-Telegrammen
wieder aufzunehmen, da der Zustand `Data_Exchange` beim Ablauf des
Watchdogs nicht verlassen wird).

---

## 6. Nicht-Interoperabilität — warum

| Anforderung des echten PROFIBUS DP | Dieser Simulator |
|---|---|
| Antwortfenster auf Bit-Ebene (*Slot Time*, `Tsdr` min/max) | Fehlt — antwortet, sobald das Telegramm dekodiert ist, ohne Zeitzwang |
| Dedizierte Schaltung (SPC3/VPC3-ASIC) für das Timing | Fehlt — gewöhnliche Tokio-Software |
| Inter-Master-Token, Multi-Master-Arbitrierung | Fehlt — Single-Master-Slave, Punkt-zu-Punkt-Verbindung |
| Beim PNO registriertes GSD-Profil | Fehlt — simulatorspezifisches E/A-Profil (§3) |
| Bit-exakte Kodierung der FC-/Diagnosefelder | Simulationskonvention, nicht garantiert konform |

**Eine reale Steuerung (zum Beispiel eine Siemens S7 mit Masterkarte) wird
diesen Simulator niemals als gültigen Slave auf einem echten
PROFIBUS-DP-RS-485-Bus erkennen.** Zwei Instanzen dieses Simulators (oder
ein Skript, das die untenstehende Sequenz abspielt) können hingegen
miteinander kommunizieren, um das Protokoll zu veranschaulichen — siehe
[`manuel_utilisateur.md`](manuel_utilisateur.md) §5.

---

## 7. Beispielsequenz (hexadezimal)

Vollständige Sequenz für Station `5`, Master `3`, bis zum zyklischen
Austausch (illustrative Werte, `FCS` über die Nutzdatenbytes berechnet):

```text
# 1. Slave_Diag (SD2, DAE=1, SAP=61)
→ TX  68 03 03 68 85 03 C0 3D FC 16
← RX  68 06 06 68 03 85 00 01 00 00 FF EE 01 F5 16   (Diag: Stat_1=0x01, Ident=0xEE01)

# 2. Set_Prm (SD2, DAE=1, SAP=62, Standard-DP-V0-Format: Station_Status
#    Lock_Req+WD_On=0x88, WD_Fact_1=1, WD_Fact_2=30 (300ms), Min_Tsdr=0,
#    Ident=0xEE01, Groups=0)
→ TX  68 0B 0B 68 85 03 C0 3E 88 01 1E 00 EE 01 00 … 16
← RX  E5                                              (ShortAck)

# 3. Chk_Cfg (SD2, DAE=1, SAP=63, out_len=45, in_len=17)
→ TX  68 05 05 68 85 03 C0 3F 2D 11 … 16
← RX  E5                                              (ShortAck)

# 4. Data_Exchange (SD2, kein SAP, 45-Byte-Ausgangsblock)
→ TX  68 30 30 68 05 03 C0 [45 Bytes] … 16
← RX  68 14 14 68 03 85 00 [17 Bytes]  … 16          (Eingangsblock)
```

Die genauen FCS-/Längenbytes hängen von den Nutzdatenwerten ab; dieses
Schema veranschaulicht die **Reihenfolge der Dienste**, kein wortwörtlich
abzuspielendes Telegramm. Siehe die Tests in
[`profibus.rs`](../../src/profibus.rs) und
[`profibus_server.rs`](../../src/profibus_server.rs) für bit-genau
verifizierte Sequenzen.
