# Benutzerhandbuch — ORME (simulierter Modbus-Regler)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · **DE** · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

> **ORME** — *Open Regulator Modbus Emulator* · Binary `mock_bin_ru_modbustcp` ·
> MIT-Lizenz · Hersteller: **CESAM-Lab** · Modbus-Gerätekennung: **CESAM-Lab**
>
> *„Öffnen Sie den Bus."* Ein Feldregler, der nur auf Ihrem Modbus-Bus (TCP/RTU)
> existiert — zum Testen von SCADA, SPS und IHM ohne reale Hardware.

Dieses Handbuch richtet sich an den **Benutzer** des simulierten Reglers: wie man
ihn startet, ihn über die Oberfläche steuert, ihn parametriert und ihn per
Modbus TCP anbindet. Es sind keine Programmierkenntnisse erforderlich.

---

## 1. Wozu dient diese Software?

Sie simuliert einen **Industrieregler** (Typ Ofen oder Thermostatbad):

- einen realistischen **physikalischen Prozess** (der „Messwert" steigt/fällt je nach Befehl);
- eine automatische oder manuelle **Regelung**, in **Heizen** und/oder **Kühlen**;
- einen **Modbus-TCP-Server** zur Steuerung/Überwachung von einer anderen Software
  aus (SPS, SCADA, Gateway…);
- eine **grafische Oberfläche** für Bedienung und Visualisierung.

Es ist ein **Test**-Werkzeug: Es ermöglicht das Entwickeln und Vorführen eines
Leitsystems oder einer SPS **ohne reale Hardware**.

---

## 2. Die Software starten

Die zu Ihrem System passende ausführbare Datei starten:

| System | Datei |
|--------|-------|
| Windows | `orme-windows-x86_64.exe` (Doppelklick) |
| Linux PC | `./orme-linux-x86_64` |
| Raspberry Pi (Bildschirm) | `./orme-rpi-arm64` |

Das Fenster öffnet sich und der **Modbus-Server startet automatisch** (Port `5502`
standardmäßig). Die Kopfzeile zeigt den Zustand an:

- **● IN BETRIEB / ● GESTOPPT**: Zustand des Geräts;
- **Modbus ● 0.0.0.0:5502** (grün): Server lauscht; **✖ …** (rot) bei einem
  Netzwerkproblem.

> Ohne Bildschirm (nur Server) siehe **§ 9 (Verwendung ohne Bildschirm)**.

---

## 3. Die Oberfläche auf einen Blick

Das Fenster umfasst vier Bereiche:

```
┌───────────────────────────── Kopfzeile: Titel, ⚙ Parameter, 💾 Speichern, Zustände ─────────────────────────────┐
├──────────────────┬─────────────────────────────────────────────────┬───────────────────────────────────────────┤
│  BEFEHLE          │   ÜBERWACHUNG                                   │   MODBUS-ADRESSTABELLE                    │
│  (links)          │   - Momentanwerte (Messwert / Sollwert /        │   (rechts)                                │
│  Start/Stopp      │     Ausgang)                                    │   Live-Liste: Bezeichnung, Tabelle,       │
│  Auto/Manuell     │   - TREND-Kurve in Echtzeit                     │   Adresse, Wert, Zugriff                  │
│  Modi, Sollwerte  │                                                 │                                           │
│  PID-Einstellung…│                                                 │                                           │
└──────────────────┴─────────────────────────────────────────────────┴───────────────────────────────────────────┘
```

---

## 4. Den Regler steuern (linkes Panel)

### 4.1 Start / Stopp
Schaltfläche **Start / Stopp**. Im Stopp ist der Ausgang null und der Messwert
kehrt langsam zum Umgebungswert zurück.

### 4.2 Auto / Manuell
- **Manuell**: *Sie* geben den Ausgang über den **manuellen Sollwert** (in %) vor.
- **Auto**: Der Regler berechnet den Ausgang, um den **Auto-Sollwert** zu erreichen.

### 4.3 Die Sollwerte
Jeder Sollwert verfügt über ein **Zahlenfeld** (präzise Tastatureingabe) und
einen **Schieberegler**. Beide sind jederzeit veränderbar; der **aktive** Sollwert
(je nach Modus) wird fett angezeigt.

| Sollwert | Einheit | Rolle |
|----------|---------|-------|
| **SP auto** | Messeinheit (z. B. °C) | im Auto-Modus zu erreichendes Ziel |
| **SP manuell** | % Ausgang, von −100 bis +100 | im Manuell-Modus vorgegebener Ausgang (**+** heiß / **−** kalt) |

### 4.4 Regelungsmodi — Richtung 1 (heiß) und Richtung 2 (kalt)
Jede Richtung wird unabhängig eingestellt:

- **Deaktiviert** — die Richtung wirkt nicht;
- **PID** — kontinuierliche Regelung (Ausgang 0…100 %), präzise und sanft;
- **Zweipunkt (TOR)** — Relais mit Hysterese: Ausgang 0 % oder 100 %, einfach, aber
  um den Sollwert oszillierend;
- **Taktrelais (PWM)** — ein PID berechnet ein Tastverhältnis, *zerhackt* über eine
  feste Periode: Der physikalische Ausgang bleibt Zweipunkt (0/100 %), aber sein
  **Mittelwert** folgt dem PID. Es ist der beste Kompromiss, um ein Organ fein zu
  steuern, das nur öffnen oder schließen kann (Relais, TOR-Ventil).

> 👉 **Wichtig — siehe **§ 6 (Die Regelung verstehen)****: Die Wahl von
> PID/TOR/PWM für das Kühlen *aktiviert* das Kühlen, aber dieses **liefert nur,
> wenn der Messwert den Sollwert überschreitet**.

### 4.5 PID-Einstellungen (Kp, Ki, Kd)
Für jede Richtung drei direkt einstellbare Verstärkungen:

- **Kp** (proportional): je größer, desto lebhafter die Reaktion (Oszillationsrisiko);
- **Ki** (integral): beseitigt die bleibende Regelabweichung mit der Zeit (zu stark → Überschwingen);
- **Kd** (differenziell): dämpft/antizipiert (zu stark → rauschempfindlich).

### 4.6 TOR-/PWM-Einstellungen
- **TOR-Hysterese** — Breite der **Totzone** des Zweipunkt-Modus, zentriert auf
  den Sollwert (`[SP − h/2, SP + h/2]`): verhindert ein ständiges Klappern des
  Ausgangs. Je breiter, desto größer die Welligkeit, aber die Schaltvorgänge
  weiter auseinander.
- **Min. Zyklus TOR (s)** — Mindestdauer, während der das Relais in einem Zustand
  verbleibt, bevor es erneut schalten kann (**Anti-Kurzzyklus**). Schützt ein
  reales Stellglied (Relais, Kompressor) und glättet das Verhalten. `0` = deaktiviert.
- **PWM-Periode (s)** — Dauer eines Zyklus des **Taktrelais**. Kurz → treuerer
  Mittelwert, aber häufige Schaltvorgänge; lang → weniger Verschleiß, aber
  ausgeprägtere Welligkeit. Deutlich kleiner als die Zeitkonstante des Prozesses zu wählen.

---

## 5. Die Trendkurve lesen

Die Kurve (in der Mitte) zeichnet in Echtzeit drei Größen. Die **Legende, oben
links**, erinnert an die Farbe **und den letzten Wert** jeder Reihe:

| Farbe | Reihe | Bedeutung |
|-------|-------|-----------|
| 🔵 blau | **Sollwert (SP)** | Ziel (im Auto) |
| 🔴 rot | **Messwert (PV)** | Prozesswert |
| 🟢 grün | **Ausgang (%)** | angewandter Befehl (**+** heiß / **−** kalt) |

Über der Kurve zeigen drei Karten die Momentanwerte (Messwert, aktiver Sollwert,
Ausgang). Man kann die Kurve mit der Maus zoomen/verschieben.

---

## 6. Die Regelung verstehen (Heizen / Kühlen)

Der Regler wirkt in **nur einer Richtung gleichzeitig**, gewählt nach der
Abweichung `Sollwert − Messwert`:

| Situation | Wirkende Richtung | Ausgang | Anzeige |
|-----------|-------------------|---------|---------|
| Messwert **< ** Sollwert (es muss geheizt werden) | **Richtung 1 (heiß)** | **positiv** (0…+100 %) | **Heiß aktiv = 1** |
| Messwert **> ** Sollwert (es muss gekühlt werden) | **Richtung 2 (kalt)** | **negativ** (−100…0 %) | **Kalt aktiv = 1** |

Praktische Konsequenzen:

- Die Auswahl von **PID/TOR für das Kühlen** genügt nicht, um „Kalt aktiv" zu
  aktivieren: Es muss **der Messwert über dem Sollwert** liegen. Solange der
  Messwert darunter liegt, arbeitet das **Heizen**.
- Um das Kühlen liefern zu sehen: im **Auto**, Richtung 2 auf PID/TOR, **den
  Sollwert unter den aktuellen Messwert absenken** (oder ein Überschwingen
  abwarten). Der Ausgang wird negativ und **Kalt aktiv** geht auf 1.
- Im **TOR** schaltet das Relais auf der **Halbhysterese** beiderseits des
  Sollwerts (symmetrische Totzone) und respektiert den **Mindestzyklus** zwischen
  zwei Schaltvorgängen. Im **PWM** zerhackt der Ausgang auf 0/100 %, aber sein
  Mittelwert folgt dem PID.

---

## 7. Parameter (Schaltfläche ⚙)

Die Schaltfläche **⚙ Parameter** öffnet ein Fenster zur Konfiguration von:

### Modbus-Transport
Wahl des Kommunikationsbusses — **nur einer gleichzeitig aktiv**:

**TCP (Ethernet)**
- **Lausch-IP** (`0.0.0.0` = alle Schnittstellen) und **Port** (Standard 5502);
- **Erlaubte IPs**: eine pro Zeile, Joker `*` akzeptiert (z. B. `192.168.1.*`).
  **Leere Liste = alle IPs erlaubt.** Die anderen werden abgelehnt.

**RTU (RS485)** — erfordert ein mit der Feature `rtu` kompiliertes Binary
- **Serieller Port**: `/dev/ttyUSB0`, `/dev/ttyAMA0` (Raspberry Pi), `COM3` (Windows)…;
- **Baud** (Standard 19200), **Parität** (Standard Gerade), **Datenbits** (8),
  **Stoppbits** (1) — mit dem Master abzustimmen;
- **Slave-Adresse** (1–247).

> ⚠️ **Nur ein entfernter Master gleichzeitig.** In TCP **trennt** die Verbindung
> eines neuen Masters automatisch den vorherigen. Die lokale IHM ist **kein**
> Master: Sie bleibt immer aktiv. In RTU eine **Punkt-zu-Punkt-Verbindung**
> bevorzugen (das Gerät antwortet unabhängig von der angeforderten Adresse).

### Übertragungsfunktion (Prozess)
Simuliertes physikalisches Verhalten `G(s) = K·e^(−L·s) / (1 + T·s)`:
- **Verstärkung K**: Messwertänderung pro % Ausgang;
- **Konstante T** (s): Trägheit/Schnelligkeit;
- **Totzeit L** (s): Verzögerung vor der Reaktion;
- **Umgebung**: Ruhewert.

### Sollwertgrenzen
Min-/Max-Grenzen des Auto-Sollwerts.

Schaltflächen: **Anwenden** (wirkt sofort **und** speichert),
**Auf Standard zurücksetzen**, **Schließen**.

### Speicherung der Einstellungen
Die Einstellungen werden in einer Datei `mock_ru_modbustcp.toml` (neben der
Software) **gespeichert** und **beim nächsten Start neu geladen**. Die Schaltfläche
**💾 Einstellungen speichern** in der Kopfzeile speichert auch die vom linken Panel
aus geänderten PID-Verstärkungen, die Hysterese, den TOR-Mindestzyklus und die
PWM-Periode.

---

## 8. Einen Modbus-Client anbinden

Die Software ist ein **Modbus-Slave** (TCP Port 5502 standardmäßig oder seriell
RTU je nach in § 7 gewähltem Transport). Ein Client (SPS, SCADA, `mbpoll`…) kann
den Zustand **lesen** und die Sollwerte/Modi **schreiben**. Erinnerung: **nur ein
entfernter Master gleichzeitig** (in TCP trennt ein Neuankömmling den vorherigen).

Wichtigste Bezugspunkte (Adressen **Basis 0**):

| Daten | Tabelle | Adresse | Typ | Zugriff |
|-------|---------|---------|-----|---------|
| Start/Stopp | Coil | 0 | bit | L/S |
| Auto/Manuell | Coil | 1 | bit | L/S |
| Modus Richtung 1 / Richtung 2 | Holding | 0 / 1 | 0=Off,1=PID,2=TOR,3=PWM | L/S |
| Auto-Sollwert | Holding | 2–3 | Gleitkomma | L/S |
| Manueller Sollwert | Holding | 4–5 | Gleitkomma | L/S |
| Min. Zyklus TOR (s) | Holding | 20–21 | Gleitkomma | L/S |
| PWM-Periode (s) | Holding | 22–23 | Gleitkomma | L/S |
| Messwert (PV) | Input | 0–1 | Gleitkomma | L |
| Ausgang (%) | Input | 2–3 | Gleitkomma | L |
| Kennung „CESAM-Lab" | Holding | 42–46 | ASCII-Text | L |

> Die **vollständige Tabelle** (PID-Verstärkungen, Hysterese, Gleitkomma-Kodierung,
> Funktionscodes, `mbpoll`-Beispiele) steht in **[table_modbus.md](table_modbus.md)**.
> Dieselbe Tabelle ist auch **live** im rechten Panel der IHM sichtbar.

---

## 9. Verwendung ohne Bildschirm („headless" / Docker)

Für eine Bereitstellung im Hintergrund (Raspberry Pi ohne Bildschirm, Server)
existiert eine Version **ohne Oberfläche**: Sie lässt die Simulation und den
Modbus-Server laufen, steuerbar **nur über Modbus**.

```bash
# Docker-Image (überall bereitstellbar):
docker run --rm -p 5502:5502 -v "$PWD/conf:/data" orme:headless
```

Der auf `/data` gemountete Ordner ermöglicht das Bereitstellen/Aufbewahren von
`mock_ru_modbustcp.toml`.

---

## 10. Häufige Fragen

| Frage / Symptom | Antwort |
|-----------------|---------|
| **„Kalt aktiv" geht nicht auf 1, obwohl ich PID/TOR gesetzt habe.** | Normal: Das Kühlen liefert nur, wenn **der Messwert den Sollwert überschreitet**. Den Sollwert unter den Messwert absenken (Auto-Modus). Siehe **§ 6 (Die Regelung verstehen)**. |
| Der Messwert bewegt sich nicht. | Prüfen, ob das Gerät **In Betrieb** ist und Sollwert/Ausgang nicht null sind. |
| Im Manuell ändert das Wechseln der Modi Richtung 1/2 nichts. | Normal: Die Modi gelten nur im **Auto**. |
| Die Kopfzeile zeigt **Modbus ✖**. | Port bereits belegt oder < 1024 ohne Rechte: den **Port** in ⚙ Parameter ändern. |
| Mein Modbus-Client wird abgelehnt. | Seine IP ist nicht in der **Whitelist**: die Liste leeren oder ein Muster hinzufügen (`192.168.1.*`). |
| Die gelesenen Gleitkommawerte sind inkonsistent. | **Wortreihenfolge**-Problem auf Client-Seite (höchstwertiges Wort zuerst). Siehe table_modbus.md. |
| Ein per Modbus geschriebener Sollwert wird ignoriert. | Ein Gleitkommawert belegt **2 Register**: sie **gemeinsam** schreiben. |
| Meine Einstellungen werden nicht beibehalten. | **Anwenden** / **💾 Speichern** klicken. Die Datei `mock_ru_modbustcp.toml` muss beschreibbar sein. |

---

*Zugehörige technische Dokumentation: [conception.md](conception.md) ·
[table_modbus.md](table_modbus.md) · [maintenance.md](maintenance.md).*
