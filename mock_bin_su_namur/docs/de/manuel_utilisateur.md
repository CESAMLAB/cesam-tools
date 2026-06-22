# Benutzerhandbuch — OSNE (simulierter NAMUR-Laborrührer)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · **DE** · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

> **OSNE** — *Open Stirrer NAMUR Emulator* · Binary `mock_bin_su_namur`
> (ausführbare Datei `osne`) · MIT-Lizenz · Herausgeber: **CESAM-Lab** · NAMUR-
> Identität: Name `CESAM-STIRRER`, Typ `OSNE`.
>
> *Ein Laborrührer (im Stil von IKA), der nur auf Ihrer NAMUR-Verbindung existiert
> — zum Testen von Leitsystemen, Skripten und Gateways ohne reale Hardware.*

Dieses Handbuch richtet sich an den **Benutzer** des simulierten Rührers: wie man
ihn startet, über die Oberfläche steuert, parametriert und per **NAMUR** (TCP oder
seriell RS-232) anbindet. Programmierkenntnisse sind nicht erforderlich.

---

## 1. Wozu dient diese Software?

Sie simuliert einen **Laborrührer** (Tischrührer mit Propeller, im Stil von IKA):

- einen realistischen **physikalischen Motor**: Die Drehzahl steigt/fällt je nach
  angelegtem Drehmoment, mit einer **schnellen Drehzahlregelung**;
- eine **einstellbare viskose Last**: je viskoser das Medium, desto höher das nötige
  Drehmoment — bis zur **Überlast** (Sollwert nicht erreichbar);
- einen **NAMUR-Server** (serielles ASCII-Protokoll der Laborgeräte), um ihn von
  einer anderen Software oder einem Skript aus zu steuern/überwachen;
- eine **grafische Oberfläche** zur Bedienung, Visualisierung und zum **Test des
  Protokolls** (integriertes NAMUR-Miniterminal).

Es ist ein **Test**werkzeug: Es ermöglicht, ein Leitsystem, ein
Erfassungsskript oder ein Gateway **ohne reale Hardware** zu entwickeln und
vorzuführen.

---

## 2. Software starten

Starten Sie die ausführbare Datei für Ihr System:

| System | Datei |
|--------|-------|
| Windows | `osne-windows-x86_64.exe` (Doppelklick) |
| Linux-PC | `./osne-linux-x86_64` |
| Raspberry Pi (Bildschirm) | `./osne-rpi-arm64` |

Das Fenster öffnet sich und der **NAMUR-Server startet automatisch** (Port `4001`
standardmäßig). Der Kopfbereich zeigt den Zustand an:

- **● IN BETRIEB / ● GESTOPPT**: Motorzustand;
- **NAMUR ● 0.0.0.0:4001** (grün): Server lauscht; **✖ …** (rot) bei einem Problem
  (Port belegt, seriell nicht verfügbar …);
- eine **Verbindungsanzeige**: bei TCP zeigt sie den verbundenen Master (oder „kein
  Master"), seriell einen einfachen Punkt. Sie wird **grün**, wenn kürzlich ein
  Rahmen empfangen wurde (Verbindung aktiv), sonst grau.

> Ohne Bildschirm (nur Server) siehe **§ 9 (Verwendung ohne Bildschirm)**.

---

## 3. Die Oberfläche auf einen Blick

```
┌──────────────── Kopfbereich: Titel OSNE, ⚙ Parameter, 💾 Speichern, Zustände & Anzeigen ────────────────┐
├──────────────────┬──────────────────────────────────────────────────────────────────────────────────┤
│  BEFEHLE          │   ÜBERWACHUNG                                                                      │
│  (links)          │   - Wertkarten (Drehzahl / Drehmoment / Viskosität / Überlast)                    │
│  Start/Stopp      │   - TREND-Kurve in Echtzeit (Sollwert / Drehzahl / Drehmoment)                    │
│  Drehzahl-Sollwert│                                                                                   │
│  Viskosität       │                                                                                   │
│  PID-Einstellungen│                                                                                   │
├──────────────────┴──────────────────────────────────────────────────────────────────────────────────┤
│  ⇄ NAMUR-RAHMEN: Miniterminal (RX/TX) + Befehlszeile + Protokollreferenz (rechts)                     │
└───────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 4. Den Rührer steuern (linkes Feld)

### 4.1 Start / Stopp
Schaltfläche **Start / Stopp**. Im gestoppten Zustand verlangsamt sich der Motor
frei bis zum Stillstand (Reibung + Last), Motordrehmoment null.

### 4.2 Drehzahl-Sollwert
Schieberegler **Drehzahl-Sollwert** (in `tr/min`), begrenzt durch die in den
*Parametern* eingestellten Min-/Max-Grenzen. Es ist dieselbe Größe wie der NAMUR-
Befehl `OUT_SP_4` (Kanal 4). Im Betrieb führt die Regelung die gemessene Drehzahl
an diesen Sollwert heran.

### 4.3 Viskosität des Mediums
Schieberegler **Viskosität** (logarithmische Skala). Er stellt die **Last** des
gerührten Mediums dar:

- **niedrige** Viskosität → geringes Drehmoment, der Sollwert wird schnell erreicht;
- **hohe** Viskosität → großes Lastmoment; übersteigt das nötige Drehmoment das
  **maximale Motordrehmoment**, wird die Solldrehzahl **nicht mehr erreicht** → die
  Anzeige **Überlast ⚠** leuchtet auf (Verhalten eines realen Rührers gegenüber
  einem zu dickflüssigen Medium).

### 4.4 PID-Einstellungen (Kp, Ki, Kd)
Die drei Verstärkungen der Drehzahlregelung, live einstellbar:

- **Kp** (proportional): je größer, desto lebhafter der Drehzahlanstieg
  (Überschwing-/Schwingungsrisiko);
- **Ki** (integral): hebt die verbleibende Drehzahlabweichung mit der Zeit auf;
- **Kd** (differenzial): dämpft/antizipiert (zu stark → empfindlich gegen Rauschen).

> Die Standardverstärkungen sind bewusst „steif" gewählt: Der Ausgang sättigt am
> maximalen Drehmoment, solange der Fehler groß ist (schneller Anstieg),
> anschließend stabilisiert der Integralanteil. Der PID-Ausgang **ist** das
> Motordrehmoment, begrenzt auf `[0, couple_max]`.

---

## 5. Die Trendkurve lesen

Die Kurve (in der Mitte) zeichnet drei Größen in Echtzeit. Die **Legende oben
links** erinnert an die Farbe **und den letzten Wert** jeder Reihe:

| Farbe | Reihe | Bedeutung |
|-------|-------|-----------|
| 🔵 blau | **Sollwert** | Drehzahl-Sollwert (im Betrieb) |
| 🔴 rot | **Drehzahl** | gemessene Drehzahl (`tr/min`, linke Achse) |
| 🟢 grün | **Drehmoment** | gemessenes Drehmoment (`N·cm`, **rechte Achse**) |

> Die Kurve hat **zwei vertikale Achsen**: die **Drehzahl** (`tr/min`) links, das
> **Drehmoment** (`N·cm`) rechts. Das Drehmoment wird skaliert, um sich den Graphen
> zu teilen, aber die rechte Achse zeigt tatsächlich `N·cm` an.

Über der Kurve zeigen **Karten** die Momentanwerte an: **Drehzahl**,
**Drehmoment**, **Viskosität** und **Überlast ⚠**, wenn der Motor sättigt. Die
Kurve lässt sich mit der Maus zoomen/verschieben.

---

## 6. Das NAMUR-Miniterminal (Fensterunterseite)

Das Feld **⇄ NAMUR-Rahmen** ermöglicht es, das **Protokoll** direkt aus der IHM
heraus zu **testen**, ohne externen Client:

- das **Journal** zeigt die **empfangenen** (`← RX`, blau) und **gesendeten**
  (`→ TX`, grün) Rahmen mit Zeitstempel an;
- die **Befehlszeile** sendet einen NAMUR-Rahmen an den Simulator (Taste
  **Eingabe** oder Schaltfläche **▶ Senden**). Die Pfeile **↑/↓** rufen vorherige
  Befehle ab (Verlauf);
- die **Protokollreferenz** (rechtes Feld) listet die Befehle auf: Ein **Klick**
  fügt den Befehl in die Eingabezeile ein;
- die Schaltfläche **🗑 Leeren** leert das Journal.

> Die hier eingegebenen Rahmen werden genauso interpretiert wie die eines
> Netzwerk-Masters: `OUT_SP_4 500` stellt den Sollwert ein, `START_4`/`STOP_4`
> starten/stoppen, `IN_PV_4` liest die Drehzahl usw. Der **Watchdog**
> (`OUT_WD1@…`) wirkt jedoch nur innerhalb einer echten Netzwerksitzung (siehe § 8).

---

## 7. Parameter (Schaltfläche ⚙)

Die Schaltfläche **⚙ Parameter** öffnet ein Fenster zur Konfiguration von:

### Sprache der Oberfläche
Auswahl oben: **Français, English, Deutsch, Español, Italiano, Português,
Nederlands, Polski** (8 Sprachen). Die Sprache wird gespeichert.

### NAMUR-Transport
Wahl der Verbindung — **nur eine gleichzeitig aktiv**:

**TCP (Ethernet)**
- **Lausch-IP** (`0.0.0.0` = alle Schnittstellen) und **Port** (Standard 4001);
- **Erlaubte IPs**: eine pro Zeile, Platzhalter `*` zulässig (z. B. `192.168.1.*`).
  **Leere Liste = alle IPs erlaubt.** Die übrigen werden abgelehnt.

**Seriell (RS-232)** — erfordert ein mit dem Feature `serial` kompiliertes Binary
- **Serieller Port**: `/dev/ttyUSB0` (Linux), `COM3` (Windows) …;
- **Baud** (Standard 9600), **Parität** (Standard Gerade), **Datenbits** (7),
  **Stoppbits** (1) — typische serielle NAMUR-Laboreinstellung: **9600 7E1**.

> ⚠️ **Nur ein Master gleichzeitig.** Bei TCP **wartet** ein neuer Master bis zur
> Trennung des vorherigen (Punkt-zu-Punkt-Verbindung). Die lokale IHM ist **kein**
> Master. Seriell *ist* der Bus der einzige Master; eine **Punkt-zu-Punkt-
> Verbindung** ist zu bevorzugen (der Server antwortet unabhängig von der
> angefragten Adresse).

### Motorparameter
Simuliertes physikalisches Verhalten `J·dω/dt = T − k·η·ω − Reibung`:
- **Trägheit** (`J`): Reaktivität des Motors (klein ⇒ schnell);
- **Lastkoeffizient** (`k`): Gewicht der Viskosität auf das Drehmoment;
- **Reibung** (`N·cm`): verbleibende trockene Reibung;
- **Max. Drehmoment** (`N·cm`): maximales Motordrehmoment (Obergrenze des PID-
  Ausgangs).

### Drehzahlgrenzen
Min-/Max-Grenzen des Drehzahl-Sollwerts (`tr/min`).

### Viskositätsgrenzen
Min-/Max-Grenzen des Viskositäts-Schiebereglers.

Schaltflächen: **Anwenden** (wirkt sofort **und** speichert), **Auf Standard
zurücksetzen**, **Schließen**.

### Speichern der Einstellungen
Die Einstellungen werden in einer Datei `mock_su_namur.toml` (neben der Software)
**gespeichert** und **beim nächsten Start neu geladen**. Die Schaltfläche **💾
Speichern** im Kopfbereich speichert ebenfalls die PID-Verstärkungen und die
Viskosität, die im linken Feld geändert wurden.

---

## 8. Einen NAMUR-Client anbinden

Die Software ist ein **NAMUR-Slave** (TCP Port 4001 standardmäßig oder seriell je
nach in § 7 gewähltem Transport). Ein Client (Skript, Terminal, Gateway) **sendet
eine ASCII-Zeile pro Anfrage**, abgeschlossen mit `CR LF`. Die **Lesungen**
(`IN_*`) liefern einen Wert; die **Schreibvorgänge/Aktionen** (`OUT_*`, `START_*`,
`STOP_*`, `RESET`) sind **stumm** (keine Antwort), gemäß NAMUR-Praxis.

Wichtige Referenzpunkte:

| Befehl | Wirkung |
|--------|---------|
| `IN_NAME` / `IN_TYPE` | Identität (`CESAM-STIRRER` / `OSNE`) |
| `IN_PV_4` / `IN_PV_5` | Drehzahl (`tr/min`) / Drehmoment (`N·cm`) lesen |
| `IN_SP_4` | Drehzahl-Sollwert lesen |
| `OUT_SP_4 <v>` | Drehzahl-Sollwert **einstellen** |
| `START_4` / `STOP_4` / `RESET` | starten / stoppen / zurücksetzen |
| `OUT_WD1@<m>` | **Watchdog**: sicherer Stopp bei Stille von `<m>` s |

Beispiel mit `nc` (netcat):

```text
$ nc 127.0.0.1 4001
IN_NAME
CESAM-STIRRER
OUT_SP_4 1200          (silencieux)
START_4                (silencieux)
IN_PV_4
1200.0 4
STOP_4                 (silencieux)
```

> Der **Watchdog** `OUT_WD1@30` stoppt den Motor automatisch, wenn 30 s lang
> **keine Zeile** eintrifft (Schutz bei Kommunikationsverlust). `OUT_WD1@0`
> deaktiviert ihn. Der Zähler wird bei jedem empfangenen Befehl neu armiert.

> Die **vollständige Protokollreferenz** (Kanäle, Kodierung, Beispiele) befindet
> sich in **[commandes_namur.md](commandes_namur.md)**. Dieselbe Liste wird **live**
> im rechten Feld des Miniterminals angezeigt.

---

## 9. Verwendung ohne Bildschirm („headless" / Docker)

Für einen Hintergrundeinsatz (Raspberry Pi ohne Bildschirm, Server) existiert eine
Version **ohne Oberfläche**: Sie lässt die Simulation und den NAMUR-Server laufen,
steuerbar **ausschließlich über NAMUR**.

```bash
# Docker-Image (überall einsetzbar):
docker run --rm -p 4001:4001 -v "$PWD/conf:/data" osne:headless
```

Der auf `/data` gemountete Ordner ermöglicht es, `mock_su_namur.toml`
bereitzustellen/aufzubewahren.

---

## 10. Häufige Fragen

| Frage / Symptom | Antwort |
|-----------------|---------|
| **Überlast ⚠** leuchtet auf und die Drehzahl erreicht den Sollwert nicht. | Normal: Die **Viskosität** verlangt mehr Drehmoment, als der Motor liefert. Senken Sie die Viskosität oder den Sollwert, oder erhöhen Sie das **max. Drehmoment** (Parameter). |
| Die Drehzahl bewegt sich nicht. | Prüfen Sie, ob der Rührer **In Betrieb** ist und der Sollwert ungleich null ist. |
| Der Kopfbereich zeigt **NAMUR ✖**. | Port bereits belegt oder < 1024 ohne Rechte (TCP), oder serieller Port nicht verfügbar: ändern Sie die Einstellung unter ⚙ Parameter. |
| Mein NAMUR/TCP-Client wird abgelehnt. | Seine IP ist nicht in der **Whitelist**: leeren Sie die Liste oder fügen Sie ein Muster hinzu (`192.168.1.*`). |
| `OUT_SP_4 …` liefert nichts zurück. | Normal: NAMUR-Schreibvorgänge/Aktionen sind **stumm**. Lesen Sie mit `IN_SP_4` / `IN_PV_4`. |
| Der Motor stoppt von selbst. | Ein **Watchdog** ist armiert (`OUT_WD1@…`) und es ist rechtzeitig kein Befehl eingetroffen. Deaktivieren Sie ihn (`OUT_WD1@0`) oder senden Sie regelmäßig Rahmen. |
| Die serielle Verbindung öffnet sich nicht. | Binary **ohne** das Feature `serial` kompiliert, oder Port/Berechtigungen falsch (Gruppe `dialout` unter Linux). |
| Meine Einstellungen bleiben nicht erhalten. | Klicken Sie **Anwenden** / **💾 Speichern**. Die Datei `mock_su_namur.toml` muss beschreibbar sein. |

---

*Zugehörige technische Dokumentation: [conception.md](conception.md) ·
[commandes_namur.md](commandes_namur.md) · [maintenance.md](maintenance.md).*
