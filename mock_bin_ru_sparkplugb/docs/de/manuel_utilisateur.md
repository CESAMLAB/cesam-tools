# Benutzerhandbuch — Sparkplug-B-Regler (ORSE)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · **DE** · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

---

## 1. Wozu dient das Instrument

**ORSE** simuliert eine **Regeleinheit** eines Prozesses (PID + thermischer Prozess
erster Ordnung) und veröffentlicht seinen Zustand per **MQTT Sparkplug B**, wie ein
**Edge-Node**, der sich mit einem **Broker** verbindet und Metriken einem SCADA bereitstellt.
Es dient zum Testen einer Sparkplug-B-Erfassungskette (Ignition, Chariot, EMQX,
Node-RED…) ohne echte Hardware.

## 2. Voraussetzung: ein MQTT-Broker

Da ORSE ein **Client** ist, wird ein erreichbarer MQTT-Broker benötigt. Lokal:

```bash
docker run -it --rm -p 1883:1883 eclipse-mosquitto
```

## 3. Erste Schritte

```bash
cargo run -p mock_bin_ru_sparkplugb        # IHM + Sparkplug-B-Edge-Node
```

Beim Start versucht die IHM, sich mit dem Broker zu verbinden (`localhost:1883`
standardmäßig). Die Kopfzeile zeigt den Zustand an: **Verbunden** (grün), sobald der
`NBIRTH` veröffentlicht ist, oder **Getrennt** (rot) mit dem Grund. Ein orangefarbenes
Banner **⚠ MQTT unverschlüsselt** weist auf das Fehlen von TLS hin.

## 4. Oberfläche

- **Kopfzeile**: Titel, Schaltflächen *Parameter* / *Speichern*, Betriebs-/Stoppzustand,
  Verbindungszustand Sparkplug B, Banner TLS/Klartext.
- **Linkes Panel (Befehle)**: *Start/Stopp*, *Automatikmodus (PID)*, *Sollwert*,
  *Manuelle Ausgabe* (manueller Modus), **PID**-Einstellungen (Kp/Ki/Kd).
- **Mittleres Panel**: Karten *Messwert / Sollwert / Ausgabe* + **Echtzeitkurve**.
- **Modal *Parameter***: Sprache, Update-Prüfung, **MQTT-Broker / Sparkplug B** (Host,
  Port, client_id, group_id, edge_node_id, keepalive, TLS, Benutzer/Passwort,
  Veröffentlichung bei Änderung/periodisch), **Prozess** (K, τ, Verzögerung, Umgebung),
  **Sollwertgrenzen**. *Anwenden* startet die Verbindung neu und speichert die TOML.

## 5. Steuerung von einem SCADA aus

Das SCADA abonniert `spBv1.0/<group_id>/#` und empfängt `NBIRTH`, dann `NDATA`. Um den
Regler zu **befehligen**, veröffentlicht es ein `NCMD` auf
`spBv1.0/<group_id>/NCMD/<edge_node_id>` mit den steuerbaren Metriken (`Setpoint`,
`Run`, `Auto`, `ManualOutput`) oder `Node Control/Rebirth = true`, um eine Wiedergeburt
zu erzwingen. Details: [`reference_sparkplugb.md`](reference_sparkplugb.md).

## 6. FAQ

- **Dauerhaft „Getrennt"** → Broker nicht erreichbar: `broker_host`/`broker_port`, die
  Firewall und ob der Broker läuft prüfen.
- **Das SCADA sieht nichts** → `group_id`/`edge_node_id` und das Abonnement
  `spBv1.0/<group>/#` prüfen; die Payloads sind **protobuf** (Sparkplug-Decoder
  erforderlich).
- **Meine NCMD-Schreibvorgänge werden ignoriert** → nicht steuerbare Metrik oder falscher
  Typ (siehe Metriktabelle). Nur `Setpoint`/`Run`/`Auto`/`ManualOutput` und `Rebirth`
  werden akzeptiert.
- **Wo ist die Konfigurationsdatei?** → `mock_ru_sparkplugb.toml` (aktuelles Verzeichnis;
  überschreibbar durch `MOCK_CONFIG`).
