# Wartungsdokumentation — RU/OPC UA (Workspace `cesam-tools`)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · **DE** · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

> Crate: `mock_bin_ru_opcua` · Ausführbare Datei: **ru_opcua**

---

## 1. Voraussetzungen

- **Rust** aktuell. ⚠️ Eigene MSRV dieses Crates: **1.91** (`async-opcua` deklariert
  keine `rust-version` und zieht aktuelle Abhängigkeiten; der Rest des Workspace ist
  auf 1.85).
- Für die IHM: die Systemabhängigkeiten von `eframe`/`egui` (dieselben wie ORME/OSNE).
- Für den *Headless*-Build: keine grafische Abhängigkeit.

---

## 2. Gängige Befehle

```bash
cargo run -p mock_bin_ru_opcua                       # IHM + OPC-UA-Server
cargo run -p mock_bin_ru_opcua --no-default-features # headless (ohne IHM)
cargo test -p mock_bin_ru_opcua                      # Unit-Tests
cargo clippy -p mock_bin_ru_opcua --all-targets      # Lint
MOCK_CONFIG=./ma_config.toml cargo run -p mock_bin_ru_opcua  # alternative Konfiguration
```

### Cargo-Features

- **`gui`** (Standard): grafische Oberfläche `egui` + Update-Prüfung.
- `--no-default-features`: **Headless**-Binärdatei (OPC-UA-Server + Simulation, ohne
  IHM und ohne Update-Netzwerk).

Der Server `async-opcua` ist **immer** vorhanden (das Feature `server` von
`async-opcua`), denn er ist der Daseinszweck des Instruments.

---

## 3. Code-Organisation

```
mock_bin_ru_opcua/src/
├── main.rs            # Baut Tokio-Runtime + Aktoren + IHM/headless zusammen
├── regulator.rs       # Synchrones Fachmodell (PID + Prozess), Befehle, Schritt
├── config.rs          # AppConfig (TOML), sanitized(), ServerStatus
├── i18n.rs            # i18n-Katalog (8 Sprachen), Lang + Msg + tr()
├── opcua_server.rs    # OPC-UA-Server: Build + Adressraum + Callbacks
├── gui.rs             # IHM egui (Feature gui)
├── branding.rs        # Eingebettete Logos (Feature gui)
└── actors/
    ├── simulation.rs  #   Regelschleife (Tick 0,5 s)
    └── network.rs     #   OPC-UA-Server (neu) konfigurierbar im laufenden Betrieb
```

---

## 4. Konfiguration

`AppConfig` (Sprache / Netzwerk / Prozess / Regelung / `check_updates`) wird als
**TOML** serialisiert (`mock_ru_opcua.toml`, überschreibbar durch `MOCK_CONFIG`),
beim Start geladen (Standardwerte, wenn nicht vorhanden), aus der IHM gespeichert.
Jeder Wert wird beim Laden **bereinigt** (`AppConfig::sanitized`: geordnete Grenzen,
`τ ≥ 1e-3`, `dead_time ≥ 0`, endliche Gleitkommazahlen).

**Invariante**: niemals `f32::clamp` mit nicht validierten Grenzen aufrufen (Panic
bei `min > max` oder `NaN`). Die Netzwerk-Schreibvorgänge laufen ebenfalls über
`Regulator::apply`, das bereinigt.

### Update-Prüfung

Nur Feature `gui`: beim Start fragt die IHM die letzte GitHub-Release über die
geteilte Lib `mock_lib_update` ab (durch Timeout begrenzter Thread) und zeigt ein
Banner, wenn eine neuere Version existiert. Einstellbar über `check_updates`.

---

## 5. Abhängigkeiten und Versionsfallen

- **`async-opcua` 0.18** (Server). Krypto **100 % Rust** (RustCrypto): **keine
  OpenSSL-Abhängigkeit** → saubere Cross-Kompilierung. Lizenz **MPL-2.0** (vgl. `NOTICE`).
- ⚠️ `async-opcua` deklariert **keine MSRV**: vor dem Anheben der Version auf der
  Ziel-Toolchain validieren.
- ⚠️ Das Instanzzertifikat (`create_sample_keypair(true)` + `pki/`) wird **nur im
  verschlüsselten Modus** erzeugt (`security.encryption`). Im None-Modus (Standard)
  kein Zertifikat (sofortiger Start). ⚠️ Die RSA-Erzeugung in reinem Rust ist im
  *Debug* langsam: beim ersten Wechsel in den verschlüsselten Modus einige Sekunden
  einplanen.
- `egui_plot` bleibt **eine Minor-Version voraus** gegenüber `egui` (vgl. ORME/OSNE).

---

## 6. Das Projekt erweitern

### 6.1 Einen OPC-UA-Knoten hinzufügen

In [`opcua_server.rs`](../../src/opcua_server.rs): den Knoten deklarieren
(`add_var`), einen Lese-Callback verdrahten (`on_read_*`) und, falls beschreibbar,
einen Schreib-Callback (`on_write_*`), der ein `Command` aussendet. Die Tabelle in
[`reference_opcua.md`](reference_opcua.md) nachziehen.

### 6.2 Einen Fachbefehl hinzufügen

Das Enum `Command` ([`regulator.rs`](../../src/regulator.rs)) erweitern, den Fall in
`Regulator::apply` behandeln (mit Bereinigung), einen Test hinzufügen.

### 6.3 Eine Oberflächen-Zeichenkette hinzufügen (i18n)

Eine Variante zu `Msg` ([`i18n.rs`](../../src/i18n.rs)) hinzufügen und **die 8
Übersetzungen** (Array fester Größe, bei der Kompilierung geprüft).

### 6.4 Sicherheit (`SecurityConfig`)

Die Sicherheit ist in [`opcua_server.rs`](../../src/opcua_server.rs) implementiert:
`security.encryption` fügt einen Endpoint `Basic256Sha256`/`SignAndEncrypt` mit
automatisch erzeugtem Zertifikat sowie anonymen und/oder Benutzer-/Passwort-Token
(`ServerUserToken::user_pass`) hinzu. Der Log-Filter
`opcua_crypto::certificate_store=off` ([`main.rs`](../../src/main.rs)) betrifft nur den
None-Modus (kein Zertifikat); im verschlüsselten Modus ist er wirkungslos. Das
Vertrauen in die Client-Zertifikate ist **einstellbar** (`trust_client_certs`:
automatisch als Standard oder streng über `pki/trusted/`). Verbleibende Ansätze:
Richtlinien `Aes256Sha256RsaPss`, X.509-Token.

---

## 7. Teststrategie

Der fachliche Kern (`regulator.rs`) und die Konfiguration (`config.rs`) sind **rein
und getestet**: PID-Konvergenz, Sollwert-Klemmung, Entspannung im Stopp, Prozesswechsel
ohne PV-Sprung, TOML-Bereinigung, TOML-Hin-und-Rückweg. Die i18n prüft die
Nicht-Leere und den Sprach-Hin-und-Rückweg.

**Integrationstests** decken zusätzlich die Netzwerkschicht ab: Client↔Server-Hin-
und-Rückweg am **None**-Endpunkt (Verbinden, Schreiben, Zurücklesen), Netzwerk-Aktor-
Parität und — am **verschlüsselten** Endpunkt (`Basic256Sha256`) — der anonyme
Hin-und-Rückweg sowie die **Benutzer-/Passwort-Authentifizierung** (richtiges Paar
akzeptiert, falsches Passwort abgelehnt). Die letzten beiden sind mit `#[ignore]`
markiert, da die **RSA-Erzeugung im *Debug* langsam** ist; sie werden ausdrücklich
gestartet:

```bash
cargo test -p mock_bin_ru_opcua -- --ignored
```

In der **CI** laufen sie mit **`--release`** (schnelles RSA) und **`--test-threads=1`**
(die verschlüsselten Server teilen sich das Verzeichnis `pki/` → serialisierte
Ausführung).

---

## 8. Fehlerbehebung

| Symptom | Wahrscheinliche Ursache | Abhilfe |
|---|---|---|
| `failed to bind` beim Start | Port bereits belegt / < 1024 ohne Rechte | Port ändern (*Parameter*) oder als Root starten |
| Client sieht die Knoten nicht | falscher Endpoint / Sicherheit | `opc.tcp://…:4840/`, None, Anonymous; *Browse* unter `Objects` |
| Schreibvorgang `Bad_TypeMismatch` | falscher Typ | `Double` für die Größen, `Boolean` für `Run`/`Auto` |
| WARN „encrypted endpoints disabled“ | kein Zertifikat (Phase 1b) | normal; der None-Endpoint funktioniert |

---

## 9. „Prod“-Build — Cross-Kompilierung von Linux aus

Das Instrument ist in [`scripts/build-prod.sh`](../../../scripts/build-prod.sh)
integriert (Array `INSTRUMENTS`): ausführbare Dateien **mit IHM** für Linux x86_64,
Windows x86_64 und Raspberry Pi arm64 (über `cross`), plus ein headless Docker-Image.

⚠️ **Cross Windows und `GetHostNameW`**: der OPC-UA-Stack zieht `gethostname`, das auf
das Winsock-Symbol `GetHostNameW` verweist. Die mingw-w64-Importbibliothek des
**Standard**-`cross`-Images (`:0.2.5`) ist zu alt, um es bereitzustellen → Fehler
beim Linken. Das Repository legt daher in [`Cross.toml`](../../../Cross.toml) das
Windows-GNU-Image auf **`:main`** fest (aktuelles mingw). Validiert: Headless- **und**
IHM-Builds erzeugen eine gültige `.exe`; ORME/OSNE kompilieren weiterhin (Übermengen-Image).

---

## 10. Konventionen

- Code und Kommentare auf **Französisch**; Logs/Fehler auf **Englisch**.
- IHM-Zeichenketten über `i18n` (8 Sprachen); niemals fest codiert.
- Fachlogik **synchron und testbar**; das Asynchrone bleibt auf die Aktoren und die
  IO beschränkt. `cargo clippy --workspace` ohne Warnung.
- `ractor`-Invarianten: keine `Mutex`-Sperre über ein `.await` hinweg; kein
  abgekoppelter Timer/`spawn` ohne beim Stoppen abgebrochenen `JoinHandle`.
