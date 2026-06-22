# Konzeption — Simulierter Prozessregler (RU/OPC UA)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · **DE** · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

> Crate: `mock_bin_ru_opcua` · Ausführbare Datei: **ru_opcua** (*Regulation Unit over OPC UA*)

Architektur- und Modellierungsdokument. Angelehnt an den Regler **ORME**
(`mock_bin_ru_modbustcp`): gleiche Aufteilung in **synchrones Fachmodell / ractor-Aktoren
/ Protokollschicht / egui-IHM**, dieselben Invarianten. Nur der **Transport**
ändert sich: **OPC UA** statt Modbus.

---

## 1. Zweck

Einen **Prozessregler** simulieren (PID-Regelkreis auf einem thermischen Prozess
erster Ordnung) und ihn über **OPC UA** bereitstellen, dem Standard der
industriellen Leittechnik (Industrie 4.0). Im Gegensatz zu ORME (Modbus) und OSNE
(NAMUR) — **Feldprotokolle ohne Sicherheit** — trägt OPC UA von Haus aus
Authentifizierung, Signatur und Verschlüsselung (vorgesehen in Phase 2).

---

## 2. Physikalisches Modell ([`regulator.rs`](../../src/regulator.rs))

Der **Prozess** verwendet [`mock_lib_control::FirstOrderProcess`] erneut (geteilt mit
ORME): Übertragungsfunktion erster Ordnung mit reiner Totzeit

```text
PV(s) / U(s) = K · e^(−L·s) / (1 + τ·s)
```

- `PV`: Messwert (Prozesseinheit, z. B. °C);
- `U`: Stellgröße / Ausgang (0-100 %);
- `K`: statische Verstärkung; `τ`: Zeitkonstante; `L`: reine Totzeit;
- `ambient`: Ruhewert (Ausgang null).

Ein **PID** ([`mock_lib_control::Pid`], ebenfalls von ORME wiederverwendet) führt
den Messwert auf den **Sollwert**, indem er den Ausgang steuert, begrenzt auf
`[0, 100]`. Zwei Modi: **automatisch** (der PID berechnet den Ausgang) und
**manuell** (Ausgang vorgegeben). Der Simulationsschritt beträgt **0,5 s**
(langsamer thermischer Prozess).

Alle Schreibvorgänge (Netzwerk oder IHM) werden in `Regulator::apply` **bereinigt**:
nicht endliche Gleitkommazahlen ignoriert, Sollwert begrenzt, Grenzen neu geordnet
(`min ≤ max`), PID-Verstärkungen geklemmt. **Invariante: niemals `f32::clamp` mit
nicht validierten Grenzen** (Panic bei `min > max` oder `NaN`).

---

## 3. Architektur (Aktoren)

```
IHM (egui) ───Command(cast)──►  SimulationActor ──refresh──► SharedSnapshot ──► IHM
OPC-UA-Server ─Command(cast)─►   (Regulator)    ──refresh──► SharedSnapshot ──► OPC-UA-Lesevorgänge
```

- **`SimulationActor`** ([`actors/simulation.rs`](../../src/actors/simulation.rs)):
  **alleiniger** Eigentümer des `Regulator`; treibt die Simulation über einen
  neu armierten One-Shot-Timer voran (kein abgekoppelter Timer) und veröffentlicht
  bei jedem Schritt einen `SharedSnapshot`.
- **`OpcuaServerActor`** ([`actors/network.rs`](../../src/actors/network.rs)):
  besitzt den OPC-UA-Server (tokio-Task `server.run()`); im laufenden Betrieb
  neustartbar (`Reconfigure`: erneutes Binden, wenn sich IP/Port ändert); behält
  den `JoinHandle` (Abbruch beim Stoppen) und den `ServerHandle` (sauberes
  Beenden der Sitzungen); veröffentlicht seinen Lauschstatus für die IHM.
- **OPC-UA-Server** ([`opcua_server.rs`](../../src/opcua_server.rs)): baut den
  Server [`async-opcua`](https://crates.io/crates/async-opcua) auf, deklariert den
  Adressraum und verdrahtet die Callbacks. Die **Lesevorgänge** schöpfen aus dem
  `SharedSnapshot`; die **Schreibvorgänge** senden ein `Command` per nicht
  blockierendem `cast` an den `SimulationActor`.

Wie bei NAMUR (OSNE) und anders als beim Modbus von ORME gibt es **keine separate
Speichertabelle**: die OPC-UA-Knoten lesen direkt aus dem geteilten Schnappschuss.

---

## 4. OPC-UA-Stack — technische Entscheidungen

- **`async-opcua`** (Server, Feature `server`): **tokio-native** Implementierung
  (eine Task pro Verbindung), die sich in den ractor/tokio-Stack einfügt. Krypto
  **100 % Rust** (RustCrypto: `rsa`, `aes`, `sha2`, `x509-cert`) — **keine
  OpenSSL-Abhängigkeit**, was die Cross-Kompilierung bewahrt (Linux/Windows/RPi).
- **Adressraum**: ein `SimpleNodeManager` im Speicher; `Variable`-Knoten unter
  `Objects` organisiert (vgl. [`reference_opcua.md`](reference_opcua.md)).
- **Callbacks**: `add_read_callback` (lebendiger Wert, abgetastet für Abonnements)
  und `add_write_callback` (leitet zur Simulation weiter).
- **Lizenz**: `async-opcua` steht unter **MPL-2.0** (die gesamte OPC-UA-Linie in
  Rust ist es). Copyleft **pro Datei**: unveränderte Nutzung → der CESAM-Lab-Code
  bleibt MIT (vgl. Datei `NOTICE` im Wurzelverzeichnis).

---

## 5. Sicherheit

- **Phase 1b (aktueller Zustand)**: ein **einziger Endpoint**, `SecurityPolicy::None`,
  **anonymes** Token. Keine Authentifizierung, keine Verschlüsselung: **nur
  vertrauenswürdiges Netzwerk**. Die IHM zeigt dauerhaft ein **Warnbanner**. Es wird
  kein Zertifikat erzeugt (die RSA-Erzeugung in reinem Rust ist im Debug langsam).
- **Phase 2 (geplant)**: verschlüsselte Endpoints (`Basic256Sha256`),
  Instanzzertifikat, Benutzerauthentifizierung. Das ist das
  **Unterscheidungsmerkmal** von OPC UA gegenüber den Feldprotokollen.

---

## 6. Konfiguration & Persistenz

`AppConfig` (Sprache / Netzwerk / Prozess / Regelung / Update-Prüfung) als **TOML**
serialisiert ([`config.rs`](../../src/config.rs)), **beim Laden bereinigt**
(`AppConfig::sanitized`: geordnete Grenzen, `τ ≥ 1e-3`, `dead_time ≥ 0`, endliche
Gleitkommazahlen). Datei: `mock_ru_opcua.toml` (überschreibbar durch `MOCK_CONFIG`).

---

## 7. Entwicklungsperspektiven

- **Phase 2**: OPC-UA-Sicherheit (Zertifikate, Verschlüsselung, Auth).
- OPC-UA-Methoden (`Reset`, `Autotune`) zusätzlich zu den Variablen.
- Typisiertes Informationsmodell (Regler-ObjectType) statt flacher Variablen.
- Historisierung / `HistoryRead` auf dem Messwert.
- Hochstufung des Reglermodells von ORME in eine geteilte `mock_lib_*` (es ist
  heute zwischen ORME und diesem Instrument dupliziert).
