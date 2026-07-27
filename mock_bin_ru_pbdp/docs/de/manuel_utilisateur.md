# Benutzerhandbuch — Simulierter PROFIBUS-DP-Regler (ORPD)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · **DE** · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

> Crate: `mock_bin_ru_pbdp` · Ausführbare Datei: **ru_pbdp** · Marke: **ORPD**

---

## ⚠️ Bevor Sie beginnen: was dieser Simulator NICHT ist

`ru_pbdp` **ist kein** hardwarekonformer PROFIBUS-DP-Slave. PROFIBUS DP ist
ein Token-Bus, dessen Einhaltung der Zeitfenster (*Slot Time*, `Tsdr`,
Watchdog) eine dedizierte Schaltung erfordert (SPC3/VPC3-ASIC,
Hilscher/Softing/Siemens-CP-Masterkarte). Ein gewöhnliches Tokio-Programm,
selbst an einem echten RS-485-Port angeschlossen, **kann diese
Anforderungen nicht einhalten**: eine reale Steuerung (zum Beispiel eine
Siemens S7 mit Masterkarte) wird diesen Simulator **niemals** als gültigen
Slave auf einem echten Bus erkennen.

Was `ru_pbdp` tatsächlich tut: Es implementiert **in Software und ohne
Echtzeitanforderungen** die Telegrammstruktur und die Zustandsmaschine eines
DP-V0-Slaves (Parametrierung, Konfiguration, Diagnose, zyklischer
Austausch). Es ist ein Werkzeug, um das **Protokoll zu verstehen** und eine
**Software-Entwicklung zu testen** (Codec, Zustandsmaschine, Werkzeuge) —
nicht, um Feldgeräte zu steuern. Details der Einschränkungen siehe
[reference_profibus.md](reference_profibus.md) §6.

---

## 1. Wozu dient dieser Simulator

`ru_pbdp` simuliert einen **Prozessregler** (PID-Regelkreis auf einem
thermischen Prozess, Modell identisch zu ORME/Modbus) und stellt ihn über
einen simulierten Satz von PROFIBUS-DP-V0-Telegrammen auf einer seriellen
Verbindung (RS-485/RS-232) bereit. Die grafische Oberfläche ermöglicht es,
die Simulation zu **steuern** und ihre Dynamik zu **visualisieren**; das
Telegrammprotokoll zeigt den ausgetauschten Verkehr hexadezimal an.

---

## 2. Erste Schritte

```bash
cargo run -p mock_bin_ru_pbdp          # GUI + serielle PROFIBUS-DP-Verbindung
```

Beim Start versucht der Simulator, den konfigurierten seriellen Port zu
öffnen (standardmäßig `/dev/ttyUSB0` oder `COM3`, 500 kbit/s,
Stationsadresse 3). Existiert der Port nicht (häufiger Fall ohne serielle
Hardware), zeigt die GUI den Öffnungsfehler im Kopfbereich an — die
Reglersimulation läuft weiter, nur die Verbindung ist nicht verfügbar.
Stellen Sie den **seriellen Port** in den *Einstellungen* so ein, dass er
auf ein verfügbares Pseudo-Terminal oder einen USB-Seriell-Adapter zeigt.

---

## 3. Die Oberfläche

### Kopfbereich

- **Titel** und Schaltflächen **⚙ Einstellungen** / **💾 Einstellungen
  speichern**.
- Rechts: **Gerätestatus** (LÄUFT / GESTOPPT), **Verbindungsstatus**
  (`PROFIBUS ● <Port> [<Status>]` grün, wenn geöffnet — der angezeigte
  Status ist der der DP-V0-Zustandsmaschine: `Power_On`/`Wait_Prm`/
  `Wait_Cfg`/`Data_Exchange`), und das **CESAM-Lab-Logo**.
- Ein **dauerhaftes orangefarbenes Banner** erinnert an die
  Nicht-Interoperabilität mit realer Hardware (siehe Hinweis oben).

### Mini-Terminal (unterer Fensterbereich)

Schreibgeschütztes Protokoll der **empfangenen** (← RX) und **gesendeten**
(→ TX) Telegramme, mit Zeitstempel und hexadezimaler Anzeige. Schaltfläche
**Löschen**, um das Protokoll zu leeren.

### Bedienfeld (links)

Identisch zu ORME: **Start/Stopp**, **Auto/Hand**, Regelungsmodi für
**Richtung 1 (heizen)** / **Richtung 2 (kühlen)** (Aus/PID/Zweipunkt/PWM),
**Sollwerte** (automatisch und manuell), **PID-Einstellungen** beider
Richtungen, **Hysterese**, **minimale Zweipunkt-Zykluszeit**,
**PWM-Periode**.

### Rechtes Feld: PROFIBUS-E/A-Blöcke

Live-Tabelle der *Output*-Blöcke (Master→Slave) und *Input*-Blöcke
(Slave→Master), mit der von diesem Simulator verwendeten Byte-Anordnung —
siehe [reference_profibus.md](reference_profibus.md) §3.

### Mittlerer Bereich

Karten **Messwert**, **Aktiver Sollwert**, **Ausgang**, sowie eine
Trendkurve.

---

## 4. Einstellungen (⚙-Modal)

- Oberflächen-**Sprache** (8 Sprachen), wird gespeichert.
- **Beim Start auf Updates prüfen** + Schaltfläche **Jetzt prüfen**.
- **Serieller Port**, **Baudrate** (einen normierten PROFIBUS-DP-Wert
  verwenden: 9600, 19200, 45450, 93750, 187500, 500000, 1500000, 3000000,
  6000000 oder 12000000), **Stationsadresse** (0-125).
- **Protokoll-Watchdog (zugelassen)**: Kontrollkästchen — wenn deaktiviert,
  wird der vom Master über `Set_Prm` angeforderte Watchdog **ignoriert**
  (nie scharfgeschaltet).
- **Übertragungsfunktion des Prozesses**: Verstärkung `K`, Zeitkonstante
  `τ`, reine Totzeit, Umgebungswert.
- **Sollwertgrenzen**: min / max (bei Vertauschung automatisch neu
  geordnet).
- **Übernehmen** / **Auf Standard zurücksetzen** / **Schließen**.

Eine Änderung von Port/Baudrate/Adresse **schließt und öffnet** die
serielle Verbindung neu. Die Einstellungen werden in `mock_ru_pbdp.toml`
gespeichert (aktuelles Verzeichnis; überschreibbar über die
Umgebungsvariable `MOCK_CONFIG`).

**Das Telegrammformat (8E1) ist durch die PROFIBUS-DP-Norm festgelegt** und
hier nicht einstellbar, im Gegensatz zu Modbus RTU oder seriellem NAMUR.

---

## 5. Das Mini-Terminal als pädagogisches Werkzeug

Ohne echte PROFIBUS-Hardware besteht der beste Weg, das Protokoll zu
beobachten, darin, **zwei Instanzen** dieses Werkzeugs miteinander
kommunizieren zu lassen — oder ein kleines Skript zu schreiben, das eine
Sequenz `Slave_Diag` → `Set_Prm` → `Chk_Cfg` → `Data_Exchange` über ein
Pseudo-Terminal abspielt (`socat -d -d pty,raw,echo=0 pty,raw,echo=0`) — und
das Mini-Terminal zu lesen, um die ausgetauschten Telegramme hexadezimal zu
sehen, mit ihrer Dekodierung in
[reference_profibus.md](reference_profibus.md).

---

## 6. FAQ

**Kann ich diesen Simulator mit einer echten PROFIBUS-DP-Steuerung
verbinden?** Nein — siehe den Hinweis am Anfang dieses Dokuments und §6 von
[reference_profibus.md](reference_profibus.md).

**Der serielle Port öffnet nicht.** Die angegebene Datei/das Gerät
existiert nicht, oder die Rechte reichen nicht aus (Gruppe `dialout` unter
Linux). Der genaue Fehler wird im GUI-Kopfbereich angezeigt.

**Die Verbindung bleibt in `Wait_Prm`.** Der Master hat noch kein
`Set_Prm` mit der erwarteten Kennung gesendet (`0xEE01`, eine **fiktive**,
nicht beim PNO registrierte Kennung). Siehe
[reference_profibus.md](reference_profibus.md) §2.

**Die Verbindung bleibt in `Wait_Cfg`.** Das empfangene `Chk_Cfg` meldet
nicht die erwarteten E/A-Längen (45 Ausgangsbytes, 17 Eingangsbytes für
diesen Simulator).

**Das Gerät stoppt von selbst.** Der Protokoll-Watchdog (vom Master über
`Set_Prm` scharfgeschaltet) ist wegen fehlenden rechtzeitig empfangenen
zyklischen Austauschs abgelaufen — das ist der erwartete sichere Zustand,
kein Fehler.

**Ohne grafische Oberfläche starten?** Kompilieren Sie *headless*:
`cargo run -p mock_bin_ru_pbdp --no-default-features` — die serielle
Verbindung und die Simulation laufen ohne GUI.
