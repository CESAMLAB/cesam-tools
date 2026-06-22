# Benutzerhandbuch — Simulierter Prozessregler (RU/OPC UA)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · **DE** · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

> Crate: `mock_bin_ru_opcua` · Ausführbare Datei: **ru_opcua**

---

## 1. Wozu dieser Simulator dient

`ru_opcua` simuliert einen **Prozessregler** (PID-Regelkreis auf einem thermischen
Prozess) und stellt ihn über **OPC UA** bereit, dem Standard der industriellen
Leittechnik. Er dient dazu, **einen OPC-UA-Client / ein SCADA zu testen** (Lesen von
Messwerten, Schreiben von Sollwerten, Abonnements) ohne reale Hardware.

Die grafische Oberfläche erlaubt es, die Simulation zu **steuern** und die Dynamik
zu **visualisieren**; der OPC-UA-Server stellt dieselben Größen dem Netzwerk bereit.

---

## 2. Erste Schritte

```bash
cargo run -p mock_bin_ru_opcua          # IHM + OPC-UA-Server
```

Beim Start lauscht der Server standardmäßig auf `opc.tcp://0.0.0.0:4840/`
(Sicherheit None). Das Fenster zeigt den aktuellen Zustand und startet die
Trendkurve.

Verbinden Sie einen OPC-UA-Client (UaExpert usw.) mit `opc.tcp://127.0.0.1:4840/`,
Sicherheit **None**, Benutzer **Anonymous**. Die Knoten sind in der
[OPC-UA-Referenz](reference_opcua.md) beschrieben.

---

## 3. Die Oberfläche

### Kopfzeile

- **Titel** und Schaltflächen **⚙ Parameter** / **💾 Einstellungen speichern**.
- Rechts: **Gerätezustand** (IN BETRIEB / GESTOPPT), **Serverzustand**
  (`OPC UA ● opc.tcp://…` in Grün, wenn lauschend, ✖ + Meldung bei Fehler) und das
  **CESAM-Lab-Logo**.
- Ein **oranges Banner** erinnert dauerhaft daran, dass der Endpoint **anonym
  (Sicherheit None)** ist: nur in einem vertrauenswürdigen Netzwerk bereitstellen.
- Wenn ein Update verfügbar ist, schlägt ein **Banner** den Download vor.

### Bedienfeld (links)

- **Start / Stopp**: startet oder stoppt die Regelung. Im Stoppzustand entspannt
  sich der Prozess zum Umgebungswert.
- **Automatikmodus (PID)**: aktiviert = der PID berechnet den Ausgang; deaktiviert =
  **Manuellmodus** (der Ausgang ist vorgegeben).
- **Sollwert**: Schieberegler, begrenzt durch die Sollwertgrenzen (einstellbar unter
  *Parameter*).
- **Manueller Ausgang (%)**: Schieberegler nur im **Manuellmodus** aktiv.
- **PID-Einstellungen**: Verstärkungen `Kp`, `Ki`, `Kd` im laufenden Betrieb
  editierbar.

### Zentralbereich

- **Karten**: Messwert, Sollwert, Ausgang.
- **Trendkurve**: Messwert (PV) und Sollwert auf der linken Achse (Prozesseinheit),
  Ausgang (%) auf der rechten Achse.

---

## 4. Parameter (Modal ⚙)

- **Sprache** der Oberfläche (8 Sprachen), persistiert.
- **Beim Start auf Updates prüfen** + Schaltfläche **Jetzt prüfen**.
- **Endpoint**: **Lausch-IP** und **Port** des OPC-UA-Servers. Eine Änderung
  **startet** den Server im laufenden Betrieb neu (laufende Sitzungen werden sauber
  geschlossen).
- **Prozess (Übertragungsfunktion)**: Verstärkung `K`, Zeitkonstante `τ`, reine
  Totzeit, Umgebungswert.
- **Sollwertgrenzen**: min / max (automatisch neu geordnet, wenn vertauscht).
- **Anwenden** / **Auf Standard zurücksetzen** / **Schließen**.

Die Einstellungen werden in `mock_ru_opcua.toml` gespeichert (aktuelles Verzeichnis;
überschreibbar über die Umgebungsvariable `MOCK_CONFIG`).

---

## 5. Sicherheit

OPC UA **kann** abgesichert werden (Zertifikate, Verschlüsselung, Authentifizierung),
aber im aktuellen Zustand (**Phase 1b**) stellt der Simulator nur einen Endpoint
**Sicherheit None** **anonym** bereit: keinerlei Schutz. **Nicht in einem offenen
Netzwerk bereitstellen.** Das Warnbanner erinnert dauerhaft daran. Echte Sicherheit
ist für **Phase 2** vorgesehen.

---

## 6. FAQ

**Ist der Port 4840 vorgeschrieben?** Nein: er wird unter *Parameter* (oder über die
TOML-Datei) eingestellt. Ein Port < 1024 erfordert Root-Rechte.

**Mein Client sieht die Knoten nicht.** Prüfen Sie die Verbindung zu
`opc.tcp://…:4840/`, Sicherheit **None**, Benutzer **Anonymous**, dann *Browse* unter
dem Ordner `Objects` (Namespace `urn:cesam-lab:ru-opcua`).

**Ein Schreibvorgang wird abgelehnt.** Der Typ muss übereinstimmen (`Double` für die
Größen, `Boolean` für `Run`/`Auto`); andernfalls liefert der Server
`Bad_TypeMismatch`.

**Ohne grafische Oberfläche starten?** *Headless* kompilieren:
`cargo run -p mock_bin_ru_opcua --no-default-features` — der OPC-UA-Server und die
Simulation laufen ohne IHM.

**Eine Meldung „encrypted endpoints disabled“ erscheint.** Das ist in Phase 1b
normal: es wird kein Instanzzertifikat bereitgestellt (verschlüsselte Endpoints nicht
verfügbar). Der None-Endpoint funktioniert hingegen.
