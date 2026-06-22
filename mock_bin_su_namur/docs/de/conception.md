# Entwurfsdokument — Simulierter Laborrührer (OSNE)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · **DE** · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

> Crate: `mock_bin_su_namur` · Ausführbare Datei: **OSNE** (*Open Stirrer NAMUR Emulator*)

Architektur- und Modellierungsdokument. Nach dem Vorbild des Reglers **ORME**
(`mock_bin_ru_modbustcp`): gleiche Aufteilung **synchrones Fachmodell / ractor-
Aktoren / Protokollschicht / egui-IHM**, gleiche Invarianten.

---

## 1. Ziel

Simulation eines **Laborrührers** (im Stil von IKA), gesteuert über das serielle
Protokoll **NAMUR**. Der Motor besitzt eine **Übertragungsfunktion** (Dynamik der
Drehzahl), die durch eine **schnelle Regelung** geführt wird, und die
**Viskosität** des Mediums ist einstellbar und beeinflusst das Drehmoment.

---

## 2. Physikalisches Modell

### Motor ([`motor.rs`](../../src/motor.rs))

Drehmomentbilanz, integriert per explizitem Euler-Verfahren:

```text
J · dω/dt = T_moteur − k · η · ω − T_frottement
```

- `ω`: Drehzahl (tr/min);
- `T_moteur`: Motordrehmoment (Stellgröße, N·cm, ≥ 0);
- `k · η · ω`: **viskoses Lastmoment** (∝ Viskosität `η` und Drehzahl);
- `T_frottement`: verbleibende trockene Reibung;
- `J` (`inertia`): bestimmt die **Reaktivität** (klein ⇒ schnell).

Im stationären Zustand gilt `T_moteur = k·η·ω + T_frottement`: Das zum Halten
einer Drehzahl nötige Drehmoment **wächst mit der Viskosität**. Übersteigt dieses
Drehmoment das **maximale Drehmoment**, ist der Sollwert nicht mehr erreichbar →
**Überlast**.

### Regelung ([`stirrer.rs`](../../src/stirrer.rs))

Ein **PID** ([`mock_lib_control::Pid`], aus ORME wiederverwendet) nimmt den
Drehzahlfehler `Sollwert − Messwert` und erzeugt das **Motordrehmoment**, begrenzt
auf `[0, couple_max]`. Die Standardverstärkungen sind bewusst „steif" gewählt: Der
Ausgang sättigt am maximalen Drehmoment, solange der Fehler groß ist (schneller
Anstieg), anschließend stabilisiert der Integralanteil. Der Simulationsschritt
beträgt **20 ms** (50 Hz), feiner als bei ORME, da die Dynamik eines Motors
schnell ist.

---

## 3. Architektur (Aktoren)

```
IHM (egui) ──Command(cast)──►  SimulationActor ──refresh──► SharedSnapshot ──► IHM
NAMUR-Server ──Command(cast)─►   (Stirrer)     ──refresh──► SharedSnapshot ──► NAMUR-Lesungen
```

- **`SimulationActor`** ([`actors/simulation.rs`](../../src/actors/simulation.rs)):
  alleiniger Eigentümer des `Stirrer`; treibt die Simulation über einen
  neu armierten One-Shot-Timer voran (kein abgekoppelter Timer) und veröffentlicht
  einen `SharedSnapshot`.
- **`NamurServerActor`** ([`actors/network.rs`](../../src/actors/network.rs)):
  besitzt den NAMUR-Server, im laufenden Betrieb neu startbar (`Reconfigure`);
  gemeinsam genutzte IP-Whitelist; für die IHM veröffentlichter Lauschstatus.
- **NAMUR-Server** ([`namur_server.rs`](../../src/namur_server.rs)): liest die
  ASCII-Zeilen, interpretiert sie ([`namur.rs`](../../src/namur.rs)), beantwortet
  Lesungen und leitet Schreibvorgänge/Aktionen an den Aktor weiter. **Ein Master
  gleichzeitig** (Punkt-zu-Punkt). **Watchdog** pro Sitzung.

Die NAMUR-Lesungen greifen auf den `SharedSnapshot` zu (keine separate
Speichertabelle wie beim Modbus von ORME: Das NAMUR-Protokoll ist
„befehlsorientiert", nicht „registerorientiert").

---

## 4. Konfiguration & Sicherheit

- `AppConfig` (Sprache / Netzwerk-Seriell / Motor / Regelung), serialisiert als
  **TOML** ([`config.rs`](../../src/config.rs)), **beim Laden bereinigt**
  (`AppConfig::sanitized`: geordnete Grenzen, endliche Gleitkommawerte) — mit ORME
  geteilte Invariante (niemals `clamp` mit nicht validierten Grenzen).
- NAMUR besitzt **weder Authentifizierung noch Verschlüsselung**: vertrauenswürdiges
  Netzwerk + IP-Whitelist (TCP). Standard `0.0.0.0` + leere Liste ⇒ exponiert: Die
  IHM zeigt ein **Warnbanner** an.

---

## 5. Ausbaupfade

- Drehrichtung (CW/CCW) und Beschleunigungsrampe.
- Temperatursensor (`IN_PV_2/3`), falls ein thermisches Modell hinzukommt.
- Nichtlineares Lastmoment (turbulente Strömung ∝ ω²).
- Hochstufung des Motormodells in `mock_lib_control`, falls es ein zweites
  Instrument bedient.
