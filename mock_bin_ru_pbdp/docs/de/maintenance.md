# Wartungsdokumentation — ORPD / PROFIBUS DP (Workspace `cesam-tools`)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · **DE** · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

> Crate: `mock_bin_ru_pbdp` · Ausführbare Datei: **ru_pbdp** · Marke: **ORPD**
> Zielgruppe: Entwickler, die das Projekt warten, korrigieren oder erweitern.
> Siehe auch: [conception.md](conception.md) · [reference_profibus.md](reference_profibus.md).

---

## 1. Voraussetzungen

- **Rust stable** (Edition 2021, `rust-version` ≥ 1.85). Installation:
  <https://rustup.rs>.
- **Systemabhängigkeiten (Linux) für die GUI** (`eframe`/`egui`,
  OpenGL/winit): `libxkbcommon-dev`, `libwayland-dev`, `libxcb1-dev`,
  `libgl1-mesa-dev` (oder Äquivalente), plus ein grafischer Server
  (X11/Wayland). Die GUI benötigt eine **Anzeige**: in einer
  Headless-Umgebung öffnet sich das Fenster nicht.
- **Serielle Verbindung** (Portzugriff, `/dev/ttyUSB*`, Gruppe `dialout`
  unter Linux): anders als bei ORME/OSNE ist dies hier **keine optionale
  Funktion** — `tokio-serial` ist eine direkte Abhängigkeit (siehe §5), da
  die serielle Verbindung der einzige Transport dieses Instruments ist (es
  gibt kein Standardäquivalent „PROFIBUS über TCP“). Ohne Hardware startet
  die GUI trotzdem (der Öffnungsfehler wird im Kopfbereich angezeigt, die
  Simulation läuft weiter) — siehe
  [`manuel_utilisateur.md`](manuel_utilisateur.md) §2.
- Netzwerkzugriff auf die crates.io-Registry für den ersten Build.

---

## 2. Übliche Befehle

```bash
cargo check -p mock_bin_ru_pbdp          # Schnelle Prüfung (ohne Codegen)
cargo build -p mock_bin_ru_pbdp          # Debug-Build
cargo build --release -p mock_bin_ru_pbdp   # Optimierter Build (Thin-LTO)
cargo test  -p mock_bin_ru_pbdp          # Unit- + Integrationstests
cargo clippy --workspace --all-targets    # Lint (muss OHNE Warnung bleiben)
cargo run   -p mock_bin_ru_pbdp          # GUI + serielle PROFIBUS-DP-Verbindung starten

# Alternative Konfigurationsdatei:
MOCK_CONFIG=./meine_config.toml cargo run -p mock_bin_ru_pbdp
# Ausführliche Protokollierung:
RUST_LOG=debug cargo run -p mock_bin_ru_pbdp
```

Erzeugte ausführbare Datei: `target/debug/ru_pbdp` oder
`target/release/ru_pbdp` (das Cargo-Paket bleibt `mock_bin_ru_pbdp`; die
ausführbare Datei und der kommerzielle Name „ORPD“ sind nur dokumentarisch,
siehe `[[bin]]` in der `Cargo.toml` der Crate).

### Cargo-Features

| Feature | Standard | Wirkung |
|---------|:---------:|-------|
| `gui` | ✅ | `egui`/`eframe`-GUI + Update-Prüfung (sonst eine Headless-Binärdatei) |

```bash
cargo build -p mock_bin_ru_pbdp --no-default-features   # headless: serielle Verbindung + Simulation, ohne GUI
```

> ⚠️ **Unterschied zu ORME/OSNE**: Bei diesen beiden Instrumenten ist die
> serielle Verbindung (RTU/seriell) selbst eine **optionale Funktion**
> neben einem stets vorhandenen TCP-Transport, und
> `--no-default-features` kann sie ausschließen. Hier gibt es **keine
> „serielle-freie“ Variante**: `tokio-serial` ist eine direkte, nicht
> Feature-gesteuerte Abhängigkeit, in **jedem** Build vorhanden,
> einschließlich Headless — sie ist der einzige Transport des Instruments.

---

## 3. Code-Organisation

```
mock_lib_control/        Wiederverwendbare Regelungsbibliothek (rein, ohne IO, testbar)
  src/pid.rs             Anti-Windup-PID
  src/lib.rs             Re-Exporte (optionales `serde`-Feature)

mock_bin_ru_pbdp/        PROFIBUS-DP-Regler-Binärdatei (ausführbare Datei `ru_pbdp`)
  src/main.rs            Start: Konfiguration, Tokio-Runtime, Akteure, GUI/Headless
  src/regulator.rs        Synchrones Fachmodell (PID + Prozess 1. Ordnung), Command, Schritt
  src/config.rs           AppConfig (TOML), SerialConfig, ProcessConfig, RegulationConfig, ServerStatus
  src/profibus.rs         PROFIBUS-DP-V0-Protokoll: Telegramm-Codec + FCS + SlaveFsm (QUELLE DER WAHRHEIT)
  src/profibus_server.rs  Serielle Sitzungsschleife (Telegramm lesen → SlaveFsm → Antwort) + Watchdog
  src/map.rs              Anordnung der Output-/Input-E/A-Blöcke <-> Regler-Command
  src/trace.rs            Zirkuläres Telegrammprotokoll (GUI-Mini-Terminal)
  src/gui.rs              egui-GUI (Einzelseite + Mini-Terminal + Einstellungen-Modal)
  src/branding.rs         Eingebettete Logos (Feature `gui`)
  src/i18n.rs             Typisierter i18n-Katalog (8 Sprachen), ohne Abhängigkeit
  src/actors/
    simulation.rs         Regelschleife (Simulationsschritt 50 ms)
    network.rs            Akteur der PROFIBUS-DP-Serialverbindung, zur Laufzeit neu konfigurierbar

docs/                     Konzeption, PROFIBUS-Referenz, Handbuch, Wartung (mehrsprachig)
```

**Goldene Regel**: Die Fachlogik (`mock_lib_control`, `regulator.rs`,
`profibus.rs`, `map.rs`) bleibt **synchron und getestet**; asynchrone
Abläufe sind auf die Akteure und die serielle IO beschränkt. Reglermodell
angelehnt an **ORME** (`mock_bin_ru_modbus`) — dieselben Invarianten.

---

## 4. Konfiguration

- Datei: `mock_ru_pbdp.toml` im aktuellen Verzeichnis, oder ein über die
  Umgebungsvariable `MOCK_CONFIG` angegebener Pfad.
- Beim Start geladen; **Standardwerte**, falls fehlend oder unlesbar (eine
  Warnung wird protokolliert, die Anwendung startet trotzdem).
- **Jeder aus dem TOML stammende Wert wird bereinigt**
  (`AppConfig::sanitized`): Sollwert-/PID-Grenzen neu geordnet,
  Gleitkommawerte auf endlich erzwungen, `τ ≥ 1e-3`, `dead_time` begrenzt,
  **Stationsadresse auf `[0, 125]` begrenzt**. **Invariante: niemals
  `f32::clamp` mit ungeprüften Grenzen aufrufen** (Panik bei `min > max`
  oder `NaN`).
- Wird über die GUI gespeichert (Schaltflächen *Übernehmen* / *Speichern* /
  *Auf Standard zurücksetzen*).

Struktur (alle Abschnitte sind optional, mit Standardwerten aufgefüllt):

```toml
language = "de"
check_updates = true       # beim Start prüfen, ob eine neuere Version existiert (GUI)

[network.serial]
port = "/dev/ttyUSB0"      # unter Windows standardmäßig "COM3"
baud = 500000              # normierter PROFIBUS-DP-Wert (9600 .. 12000000)
station_address = 3        # Adresse des simulierten Slaves (0-125)
watchdog_enabled = true    # erlaubt den vom Master angekündigten Watchdog (Set_Prm)

[process]
gain = 1.6 ; tau = 30.0 ; dead_time = 2.0 ; ambient = 20.0

[regulation]
sp_min = 0.0 ; sp_max = 250.0
hysteresis = 2.0 ; tor_min_cycle = 5.0 ; pwm_period = 10.0
[regulation.pid_heat]
kp = ... ; ki = ... ; kd = ... ; out_min = 0.0 ; out_max = 100.0
[regulation.pid_cool]
kp = ... ; ki = ... ; kd = ... ; out_min = 0.0 ; out_max = 100.0
```

> Das **serielle Telegrammformat (8E1)** ist durch die PROFIBUS-DP-Norm
> festgelegt und ist **kein** Konfigurationsfeld — siehe
> `SerialConfig::open` in [`config.rs`](../../src/config.rs). Anders als
> bei ORME/OSNE gibt es **keine IP-Positivliste** (die serielle Verbindung
> ist von Natur aus Punkt-zu-Punkt).

### Update-Prüfung

Wenn `check_updates = true` (Standard) **und** die Binärdatei mit dem
Feature `gui` kompiliert ist, fragt die GUI **beim Start** die neueste auf
GitHub veröffentlichte Version (`CESAMLAB/cesam-tools`) über die
gemeinsame Crate **`mock_lib_update`** ab (`ureq`/`rustls`, eingebettete
Wurzelzertifikate, durch Timeout begrenzter Thread). **Fehlt in
Headless-Builds** (`--no-default-features`).

---

## 5. Abhängigkeiten und Versionsfallstricke

| Crate | Rolle | Zu beachten |
|-------|------|-------------------|
| `tokio` | Async-Runtime | gemeinsame Features + `io-util` |
| `ractor` | Akteure | Standard-Features |
| `tokio-serial` | PROFIBUS-DP-Verbindung | **direkte, nicht Feature-gesteuerte Abhängigkeit** (siehe §2); `default-features = false` (keine `libudev`-Enumeration) |
| `eframe`/`egui` | GUI | Versionen aneinander gebunden, Feature `gui` |
| `egui_plot` | Trendkurve | ⚠️ **um eine Nebenversion vor `egui` versioniert**: für `egui` 0.33 → `egui_plot` **0.34** |
| `serde`/`toml` | Persistenz | `mock_lib_control` stellt ein von der Binärdatei aktiviertes `serde`-Feature bereit |
| `mock_lib_update` (`ureq`/`rustls`) | Update-Prüfung | nur Feature `gui`; fehlt headless |

Gemeinsame Versionen sind in `[workspace.dependencies]` der Root-`Cargo.toml`
zentralisiert. Beim Anheben von `egui`/`eframe` **die passende
`egui_plot`-Version prüfen** (sonst Fehler „two versions of crate egui“).

---

## 6. Projekt erweitern

### 6.1 Einen PROFIBUS-Dienst (SAP) hinzufügen

Alles geschieht in **[`profibus.rs`](../../src/profibus.rs)** (Quelle der
Wahrheit des Protokolls):

1. Die `SAP_*`-Konstante und die entsprechende Variante in `enum Request`
   hinzufügen; die Dekodierung in `decode_request` (und, für Tests, in
   `encode_request`) verdrahten.
2. Die neue Anfrage in `SlaveFsm::handle` behandeln (Zustandsübergang, falls
   relevant, `Handled` zurückgeben).
3. Den Doc-Kommentar des Moduls und
   **[reference_profibus.md](reference_profibus.md)** aktualisieren.
4. Einen Test im `tests`-Modul von `profibus.rs` hinzufügen (und, falls die
   vollständige Sitzung betroffen ist, in `profibus_server.rs`).

### 6.2 Die E/A-Blöcke (`Output`/`Input`) ändern

1. Die Anordnung in **[`map.rs`](../../src/map.rs)**
   (`decode_output`/`encode_input`) anpassen, dabei `OUTPUT_LEN`/`INPUT_LEN`
   konsistent mit `SlaveProfile` (`profibus_server.rs`) halten.
2. Die Tabelle in **[reference_profibus.md](reference_profibus.md)** §3
   aktualisieren (Quelle der Wahrheit der Dokumentation, aus dem
   Doc-Kommentar von `map.rs` übernommen).
3. Einen Round-Trip-Test in `map.rs` hinzufügen.

### 6.3 Einen Fachbefehl / eine GUI-Einstellung hinzufügen

1. Variante in `enum Command` (`regulator.rs`) + Behandlung in
   `Regulator::apply` (mit Bereinigung).
2. Feld in `RegulatorSnapshot`, falls der Wert beobachtbar sein muss.
3. GUI-Verdrahtung (`gui.rs`) über ein nicht blockierendes `cast`.
4. Falls persistent: Feld in `AppConfig` (`config.rs`) + Bereinigung in
   `sanitized` + Übertragung in `to_regulator_config`.

### 6.4 Eine Oberflächenzeichenkette (i18n) hinzufügen

Jede GUI-Zeichenkette **muss** über einen `Msg`-Schlüssel (`i18n.rs`) mit
seinen **8 Übersetzungen** (zur Kompilierzeit geprüftes Array fester
Größe) laufen. PROFIBUS-Dienstbezeichner und Einheitensuffixe bleiben fest
codiert.

### 6.5 Ein neues Instrument hinzufügen

1. `mock_bin_<name>/` erstellen und zu den `members` der Root-`Cargo.toml`
   hinzufügen.
2. `mock_lib_control` wiederverwenden; alles Gemeinsame in eine
   `mock_lib_*` auslagern.
3. Derselben Gliederung folgen: synchrones Modell, `ractor`-Akteur(e),
   Protokollschicht, GUI. Namenskonvention: `mock_bin_<Typ>_<Protokoll>`.

---

## 7. Teststrategie

- **Telegramm-Codec** (`profibus.rs`): Round-Trip von
  `SD1`/`SD2`/`SD3`/`SD4`, Ablehnung falscher Prüfsumme und Länge,
  Kodierung/Dekodierung der Anfragen
  (`Slave_Diag`/`Set_Prm`/`Chk_Cfg`/`Data_Exchange`) und des Modus-Bytes.
- **Zustandsmaschine** (`profibus.rs`): vollständige Sequenz
  `Power_On → Wait_Prm → Wait_Cfg → Data_Exchange`, Ablehnung eines
  `Set_Prm` mit falscher Kennung (bleibt in `Wait_Prm`).
- **E/A-Blöcke** (`map.rs`): ein zu kurzer Ausgangsblock → kein Befehl;
  Round-Trip von Sollwert/Modus; der Eingangsblock spiegelt den Snapshot
  wider (Statusbits, Messwert).
- **Konfiguration** (`config.rs`): TOML-Round-Trip, Bereinigung
  (vertauschte Grenzen, nicht endliche Werte, Stationsadresse außerhalb des
  Bereichs) ohne Panik, sauberer Fehler beim Öffnen eines fehlenden
  seriellen Ports.
- **Netzwerksitzung** (`profibus_server.rs`, `#[tokio::test]` auf
  `tokio::io::duplex`): vollständiger Handshake bis `Data_Exchange` mit
  tatsächlicher Befehlsanwendung, ein an eine andere Station adressiertes
  Telegramm wird ignoriert (keine Aktivität vermerkt), Watchdog-Ablauf
  erzwingt den sicheren Zustand.

Ausführen: `cargo test -p mock_bin_ru_pbdp` (oder `--workspace`) — **36
Tests**, alle **deterministisch und ohne GUI**, kein langsamer/`#[ignore]`-
Test (anders als bei ORUE, wo die RSA-Erzeugung ignorierte Tests
rechtfertigt).

---

## 8. Fehlerbehebung

| Symptom | Ansatz |
|----------|-------|
| „two versions of crate `egui`“ | Diskrepanz `egui_plot` / `egui`: Versionen angleichen (§5). |
| Die GUI öffnet sich nicht | Keine Anzeige (Headless) oder fehlende Systembibliotheken (§1). |
| Fehler beim Öffnen des seriellen Ports (GUI-Kopfbereich) | Fehlender Port, falscher Pfad oder Rechte (Gruppe `dialout` unter Linux) — die Simulation läuft ohne Verbindung weiter. |
| Die Verbindung bleibt in `Wait_Prm` | Der Master sendet kein `Set_Prm` mit der erwarteten Kennung (`0xEE01`) — siehe [reference_profibus.md](reference_profibus.md) §2. |
| Die Verbindung bleibt in `Wait_Cfg` | Das empfangene `Chk_Cfg` meldet nicht `out_len=45`/`in_len=17`. |
| Das Gerät stoppt von selbst | Protokoll-Watchdog ausgelöst (anhaltendes Schweigen des Masters) — erwarteter sicherer Zustand, kein Fehler. |
| Kein Watchdog, obwohl der Master einen anfordert | `watchdog_enabled = false` in der lokalen Konfiguration: die Anforderung des Masters wird bewusst ignoriert. |

Ausführlichkeit erhöhen: `RUST_LOG=debug` (oder `trace`).

---

## 9. Distributions-Build

```bash
cargo build --release -p mock_bin_ru_pbdp
# Eigenständige Binärdatei:
target/release/ru_pbdp
```

Das `release`-Profil aktiviert `lto = "thin"` und `opt-level = 3` (siehe
Root-`Cargo.toml`). Zum Verteilen: die Binärdatei plus eine Beispiel-
`mock_ru_pbdp.toml` bereitstellen. Lizenz: **MIT** (Datei `LICENSE`).

### Feature `gui` (Build mit / ohne Oberfläche)

```bash
cargo build --release -p mock_bin_ru_pbdp                       # mit GUI (Arbeitsplatz)
cargo build --release -p mock_bin_ru_pbdp --no-default-features  # „headless“: serielle Verbindung + Simulation, ohne GUI
```

Anders als bei OSNE macht der **Headless**-Modus die serielle Verbindung
nicht optional (§2): er entfernt nur die GUI. Er bleibt relevant für eine
bildschirmlose Bereitstellung, die an einen echten seriellen/USB-Port
angeschlossen ist.

### Linux-Desktop-Integration (Taskleisten-Symbol)

Das ORPD-Symbol (`pic/ru_pbdp-icon.png`, erzeugt von
[`pic/ru_pbdp-logo.gen.py`](../../../pic/ru_pbdp-logo.gen.py)) ist in die
Binärdatei **eingebettet** (`branding.rs` → `window_icon`). Das genügt
unter **X11, Windows und macOS**. Unter **Wayland** **ignoriert** der
Compositor das eingebettete Symbol: Er ordnet das Fenster über seine
**`app_id`** („ru_pbdp“, in `main.rs` über `with_app_id` festgelegt) einer
gleichnamigen `ru_pbdp.desktop`-Datei zu und zeigt das über das
Icon-Theme `hicolor` aufgelöste `Icon=ru_pbdp` an.

Um das Symbol unter Wayland zu erhalten, installieren Sie den
Desktop-Eintrag für den aktuellen Benutzer:

```bash
scripts/install-desktop.sh ru_pbdp
```

Das Skript kopiert:

| Quelle | Ziel |
|--------|-------------|
| `pic/ru_pbdp-icon.png` | `~/.local/share/icons/hicolor/256x256/apps/ru_pbdp.png` |
| `packaging/ru_pbdp.desktop` | `~/.local/share/applications/ru_pbdp.desktop` |

und aktualisiert danach die Caches. Drei Namen **müssen übereinstimmen**:
die `app_id` (`main.rs`), die Datei `ru_pbdp.desktop` (+ ihr
`StartupWMClass`) und das Symbol `ru_pbdp.png` (= `Icon=ru_pbdp`).

---

## 10. „Prod“-Build — Cross-Kompilierung von Linux aus

Alles wird **von Linux aus** durch
[`scripts/build-prod.sh`](../../../scripts/build-prod.sh) erzeugt, das
**jedes Instrument des Workspace** baut (Tabelle `INSTRUMENTS`, Eintrag
`mock_bin_ru_pbdp:ru_pbdp:0` — Port `0`: serielle Verbindung, kein
IP-Port):

| Ausgabe | Ziel | GUI | Methode |
|--------|-------|-----|---------|
| `dist/ru_pbdp-linux-x86_64` | `x86_64-unknown-linux-gnu` | ✅ | `cross` |
| `dist/ru_pbdp-windows-x86_64.exe` | `x86_64-pc-windows-gnu` | ✅ | `cross` (mingw) |
| `dist/ru_pbdp-rpi-arm64` | `aarch64-unknown-linux-gnu` (Pi 3/4/5, Pi OS 64-Bit) | ✅ | `cross` |
| Headless-Docker-Image `ru_pbdp:headless` | Multi-Arch `amd64` + `arm64` | ❌ | `docker buildx` |
| `dist/ru_pbdp_<ver>_amd64.deb` / `_arm64.deb` | Debian/Ubuntu-Paket | ✅ | `dpkg-deb` |
| `dist/ru_pbdp-setup-x86_64.exe` | Windows-Installer | ✅ | NSIS (`makensis`) |

```bash
cargo install cross          # Voraussetzung (einmalig) — Docker muss laufen
scripts/build-prod.sh        # jedes Instrument, einschließlich ru_pbdp
ONLY=ru_pbdp scripts/build-prod.sh   # nur dieses Instrument
```

⚠️ **Nicht natives `cargo` und `cross` im selben `target/` mischen**
(inkompatible Proc-Makros → `can't find crate for …_derive`). Das Skript
verwendet immer `cross`.

### Headless-Docker-Image: begrenzter Nutzen ohne seriellen Passthrough

Das Image ([`docker/Dockerfile.headless`](../../../docker/Dockerfile.headless))
wird wie bei den anderen Instrumenten gebaut (`EXPOSE 0`, träge
Metadaten), ist aber **nur wirklich nützlich, wenn ein serielles Gerät**
in den Container eingebunden wird:

```bash
docker run --rm --device=/dev/ttyUSB0 -v "$PWD/conf:/data" ru_pbdp:headless
```

Ohne `--device` startet der Container, kann aber keinen seriellen Port
öffnen (dasselbe Verhalten wie fehlende Hardware lokal — siehe §8).

---

## 11. Konventionen

- Code und Kommentare auf **Französisch** (projektweite Konvention); Logs
  und Fehlermeldungen auf **Englisch**.
- `cargo clippy --workspace` **ohne Warnung** vor jedem Commit.
- Jedes neue Fach- oder Protokollverhalten geht mit einem **Test** einher.
- Das PROFIBUS-DP-V0-Protokoll wird in **`profibus.rs`** geändert (Quelle
  der Wahrheit), zusammen mit einer Aktualisierung von
  **[reference_profibus.md](reference_profibus.md)**.
