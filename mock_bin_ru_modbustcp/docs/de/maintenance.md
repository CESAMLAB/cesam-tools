# Wartungsdokumentation — ORME (Workspace `cesam-tools`)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · **DE** · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

> Zielgruppe: Entwickler, die das Projekt pflegen, korrigieren oder erweitern.
> Siehe auch: [conception.md](conception.md) · [table_modbus.md](table_modbus.md).

---

## 1. Voraussetzungen

- **Rust stable** (Edition 2021, `rust-version` ≥ 1.85). Installation: <https://rustup.rs>.
- **Systemabhängigkeiten (Linux) für die IHM** (`eframe`/`egui`, OpenGL/winit):
  - Debian/Ubuntu: `libxkbcommon-dev`, `libwayland-dev`, `libxcb1-dev`,
    `libgl1-mesa-dev` (oder Äquivalente), plus ein Grafikserver (X11/Wayland).
  - Die IHM benötigt eine **Anzeige**: In einer Headless-Umgebung öffnet sich das
    Fenster nicht (der Modbus-Server hängt hingegen nicht von der Anzeige ab).
- Netzwerkzugang zur crates.io-Registry für die erste Kompilierung.

---

## 2. Gängige Befehle

```bash
cargo check --workspace          # Schnellprüfung (ohne Codegen)
cargo build --workspace          # Debug-Kompilierung
cargo build --release            # Optimierte Kompilierung (LTO thin)
cargo test  --workspace          # Unit- + Integrationstests
cargo clippy --workspace --all-targets   # Lint (muss WARNUNGSFREI bleiben)
cargo run -p mock_bin_ru_modbustcp       # Startet den Regler

# Alternative Konfigurationsdatei:
MOCK_CONFIG=./ma_config.toml cargo run -p mock_bin_ru_modbustcp
# Ausführliche Protokollierung:
RUST_LOG=debug cargo run -p mock_bin_ru_modbustcp
```

Erzeugtes Binary: `target/debug/orme` oder `target/release/orme` (das Cargo-Paket
bleibt `mock_bin_ru_modbustcp`, aber die ausführbare Datei heißt **`orme`** — siehe
`[[bin]]` in der `Cargo.toml` des Crates).

### Cargo-Features

| Feature | Standard | Wirkung |
|---------|:--------:|---------|
| `gui` | ✅ | IHM `egui`/`eframe` (sonst Headless-Binary) |
| `rtu` | ✅ | Serieller Modbus-RTU-Transport (RS485) über `tokio-serial` |

```bash
cargo build --no-default-features                 # headless, nur Modbus TCP
cargo build --no-default-features --features rtu  # headless TCP + serielles RTU
cargo build --no-default-features --features gui  # IHM, nur TCP (ohne seriell)
```

> ⚠️ **`rtu` = native Abhängigkeit.** `tokio-serial` öffnet den Port über termios
> (Linux); die `libudev`-Enumeration ist deaktiviert (`default-features = false`).
> Bei der **Cross-Kompilierung** (`build-prod.sh`, Desktop-Exes mit
> Standard-Features) kann das `cross`-Image des Targets dennoch die seriellen
> Header des Systems verlangen; falls die Toolchain Probleme bereitet, `rtu` aus
> dem betroffenen Build entfernen. Das **Headless-Docker ist nicht betroffen**
> (es baut mit `--no-default-features`).

---

## 3. Code-Organisation

```
mock_lib_control/        Regelungsbibliothek (rein, ohne IO, testbar)
  src/pid.rs             PID Anti-Windup
  src/onoff.rs           Zweipunkt mit symmetrischer Hysterese + Anti-Kurzzyklus
  src/pwm.rs             Taktrelais (PWM / time-proportioning)
  src/process.rs         FOPDT-Übertragungsfunktion
  src/lib.rs             ControllerKind + Re-Exporte (optionale Feature `serde`)

mock_bin_ru_modbustcp/   Regler-Binary
  src/main.rs            Start: Konfiguration, Tokio-Runtime, Aktoren, IHM
  src/regulator.rs       Synchrones Geschäftsmodell (Zustand, Command, step)
  src/config.rs          AppConfig (TOML), Transport/SerialConfig, IpFilter, ServerStatus
  src/map.rs             Modbus-Adressplan (QUELLE DER WAHRHEIT)
  src/modbus_server.rs   RegulatorService (Trait Service) + Single-Master TCP + serve_rtu
  src/gui.rs             IHM egui (Einzelseite + Parameter-Modal)
  src/actors/
    simulation.rs        Regelschleife (tick)
    network.rs           Modbus-TCP/RTU-Server, im laufenden Betrieb (re)konfigurierbar

docs/                    Entwurf, Modbus-Tabelle, Wartung
```

**Goldene Regel**: Die Geschäftslogik (`mock_lib_control`, `regulator.rs`) bleibt
**synchron und getestet**; das Asynchrone bleibt auf die Aktoren und die IO beschränkt.

---

## 4. Konfiguration

- Datei: `mock_ru_modbustcp.toml` im aktuellen Verzeichnis oder Pfad, der durch
  die Umgebungsvariable `MOCK_CONFIG` angegeben wird.
- Beim Start geladen; **Standardwerte**, falls fehlend oder unlesbar (eine
  Warnung wird protokolliert, die Anwendung startet trotzdem).
- Aus der IHM gespeichert (Schaltflächen *Anwenden* / *Einstellungen speichern* /
  *Auf Standard zurücksetzen*).

Struktur (alle Abschnitte sind optional, mit Standardwerten ergänzt):

```toml
[network]
bind_ip = "0.0.0.0"
port = 5502
allowlist = ["192.168.1.*", "127.0.0.1"]   # leer = alle IPs erlaubt

[process]   # Übertragungsfunktion G(s) = K·e^(-L·s)/(1+T·s)
gain = 1.6        # K (Einheit/%)
tau = 30.0        # T (s)
dead_time = 2.0   # L (s)
ambient = 20.0

[regulation]
sp_min = 0.0
sp_max = 250.0
hysteresis = 2.0
[regulation.pid_heat]   # Richtung 1 (heiß)
kp = 4.0 ; ki = 0.25 ; kd = 1.0 ; out_min = 0.0 ; out_max = 100.0
[regulation.pid_cool]   # Richtung 2 (kalt)
kp = 4.0 ; ki = 0.25 ; kd = 1.0 ; out_min = 0.0 ; out_max = 100.0
```

> Die **Standardwerte** haben eine **einzige Quelle**: `RegulatorConfig::default`
> in `regulator.rs`. `ProcessConfig`/`RegulationConfig` (config.rs) leiten sich
> daraus ab. Um einen Standard zu ändern, nur `RegulatorConfig::default` anpassen.

---

## 5. Abhängigkeiten und Versionsfallen

| Crate | Rolle | Hinweis |
|-------|-------|---------|
| `tokio` | async-Runtime | Features: `rt-multi-thread, macros, net, time, sync` |
| `ractor` | Aktoren | Standard-Features (natives async, **kein** `async-trait`) |
| `tokio-serial` | serielles Modbus RTU | optional (Feature `rtu`), `default-features = false` (keine libudev-Enumeration) |
| `tokio-modbus` | Modbus TCP | `default-features = false`, Feature **`tcp-server`** |
| `eframe`/`egui` | IHM | Versionen miteinander verbunden |
| `egui_plot` | Kurve | ⚠️ **versioniert eine Minor-Version vor `egui`**: für `egui` 0.33 → `egui_plot` **0.34** |
| `serde`/`toml` | Persistenz | `mock_lib_control` stellt eine vom Binary aktivierte Feature `serde` bereit |

Die geteilten Versionen sind zentral in `[workspace.dependencies]` der
Root-`Cargo.toml` definiert. Um `egui`/`eframe` anzuheben, **die entsprechende
Version von `egui_plot` prüfen** (sonst Fehler „two versions of crate egui").

---

## 6. Das Projekt erweitern

### 6.1 Einen Modbus-Punkt hinzufügen

Alles passiert in **`map.rs`** (dann Snapshot/Command falls nötig):

1. Die Adresskonstante deklarieren und das `*_COUNT` der betroffenen Tabelle anpassen.
2. Den Wert in `MemoryMap::refresh_from` eintragen (Zustand → Register).
3. Falls der Punkt beschreibbar ist, ihn in `coil_to_command` /
   `holdings_to_commands` dekodieren (Register → `Command`).
4. Den Kopf-Doc-Kommentar **und** [table_modbus.md](table_modbus.md) aktualisieren.
5. Die Zeile in der Live-Tabelle der IHM (`gui.rs::modbus_rows`) hinzufügen.

### 6.2 Einen Befehl / eine Einstellung hinzufügen

1. Variante in `enum Command` (`regulator.rs`) + Verarbeitung in `Regulator::apply`.
2. Feld in `RegulatorSnapshot`, falls der Wert beobachtbar sein soll.
3. IHM-Verdrahtung (`gui.rs`) und/oder Modbus-Dekodierung (`map.rs`).
4. Falls persistent: Feld in `AppConfig` (`config.rs`) + `to_regulator_config`.

### 6.3 Ein neues Instrument hinzufügen

1. `mock_bin_<name>/` erstellen und zu den `members` der Root-`Cargo.toml` hinzufügen.
2. `mock_lib_control` wiederverwenden; alles Gemeinsame in eine `mock_lib_*` auslagern.
3. Demselben Aufbau folgen: synchrones Modell, ractor-Aktor(en), Protokoll-
   schicht, IHM. Namenskonvention: `mock_bin_<typ>_<protokoll>`.

---

## 7. Teststrategie

- **Unit** (`mock_lib_control`): PID (proportional, Begrenzung, Anti-Windup),
  TOR (Totzone), Prozess (Konvergenz im eingeschwungenen Zustand).
- **Domäne** (`regulator.rs`): PID-Konvergenz im Auto, Ausgang im Manuell,
  Rückkehr zur Umgebung beim Stopp.
- **Mapping** (`map.rs`): Round-Trip `f32`↔Register, Dekodierung von
  Schreibvorgängen, Ablehnung eines partiellen `f32`-Schreibvorgangs.
- **Konfiguration / Netzwerk** (`config.rs`, `actors/network.rs`): TOML-Round-Trip,
  IP-Filter (Joker), tatsächlicher Serverstart (Bind auf flüchtigen Port).

Starten: `cargo test --workspace`. Die Tests sind **deterministisch und ohne IHM**.

---

## 8. Fehlerbehebung

| Symptom | Ansatz |
|---------|--------|
| „two versions of crate `egui`" | Unstimmigkeit `egui_plot` / `egui`: Versionen angleichen (§5). |
| Die IHM öffnet sich nicht | Keine Anzeige (headless) oder fehlende Systembibliotheken (§1). |
| `Modbus ✖ Lauschen fehlgeschlagen` in der Kopfzeile | Port bereits belegt oder < 1024 ohne Rechte: Port in *Parameter* ändern. |
| Ein Client wird abgelehnt | IP außerhalb der **Whitelist**: Liste leeren oder Muster hinzufügen (`192.168.1.*`). |
| Unsinnige `f32`-Werte auf Client-Seite | Wortreihenfolge (höchstwertiges Wort zuerst): siehe [table_modbus.md](table_modbus.md). |
| Ein Sollwert-`f32`-Schreibvorgang wird ignoriert | **Beide** Register des Paars in einer Anfrage schreiben. |
| Konfiguration nicht neu geladen | Falsches aktuelles Verzeichnis oder `MOCK_CONFIG`; das Startprotokoll prüfen. |
| Kein Symbol in der Taskleiste (Linux) | **Wayland**-Sitzung: Das eingebettete Symbol wird ignoriert. Den Desktop-Eintrag installieren: `scripts/install-desktop.sh` (§9, Desktop-Integration). |

Ausführlichkeit erhöhen: `RUST_LOG=debug` (oder `trace`).

---

## 9. Distributions-Build

```bash
cargo build --release
# Eigenständiges Binary:
target/release/orme
```

Das Profil `release` aktiviert `lto = "thin"` und `opt-level = 3` (siehe Root-
`Cargo.toml`). Zur Distribution: das Binary + eine beispielhafte
`mock_ru_modbustcp.toml` bereitstellen. Lizenz **MIT** (Datei `LICENSE`).

### Feature `gui` (Build mit / ohne Oberfläche)

Die IHM steckt hinter der Cargo-Feature **`gui`**, standardmäßig aktiviert:

```bash
cargo build --release                       # mit IHM (Arbeitsplatz)
cargo build --release --no-default-features  # „headless": Modbus + Simulation, ohne IHM
```

Der **Headless**-Modus ist für Bereitstellungen ohne Bildschirm gedacht
(Raspberry Pi im Betrieb) und macht die **ARM-Cross-Kompilierung trivial** (keine
zu verlinkende Grafikabhängigkeit).

### Linux-Desktop-Integration (Symbol in der Taskleiste)

Das ORME-Symbol ist im Binary eingebettet (`branding.rs` → `with_icon`). Das
genügt unter **X11, Windows und macOS**. Unter **Wayland** **ignoriert** der
Compositor jedoch das eingebettete Symbol: Er ordnet das Fenster über seine
**`app_id`** („orme", in `main.rs` über `ViewportBuilder::with_app_id` definiert)
einer gleichnamigen Datei `orme.desktop` zu und zeigt das `Icon=` dieser Datei an
(aufgelöst im Icon-Theme `hicolor`).

Um das Symbol unter Wayland zu erhalten, den Desktop-Eintrag für den aktuellen
Benutzer installieren:

```bash
scripts/install-desktop.sh
```

Das Skript kopiert:

| Quelle | Ziel |
|--------|------|
| `pic/orme-icon.png` | `~/.local/share/icons/hicolor/256x256/apps/orme.png` |
| `packaging/orme.desktop` | `~/.local/share/applications/orme.desktop` |

und aktualisiert anschließend die Caches (`gtk-update-icon-cache`,
`update-desktop-database`). Das Symbol erscheint beim nächsten Start von ORME (und
zuverlässig nach einer erneuten Anmeldung der Wayland-Sitzung).

> ⚠️ Drei Namen **müssen übereinstimmen**: die `app_id` (`main.rs`), der Name der
> Datei `orme.desktop` und ihre `StartupWMClass` sowie der Name des Symbols
> `orme.png` (= `Icon=orme`). `packaging/orme.desktop` setzt eine ausführbare
> Datei `orme` im `PATH` voraus (Feld `Exec=`); in der Entwicklung (`cargo run`)
> hat dieses Feld keinen Einfluss auf die Anzeige des Symbols.

---

## 10. „Prod"-Build — Cross-Kompilierung von Linux aus

### Einheitliches Verfahren

Alles wird **von Linux aus** durch
[`scripts/build-prod.sh`](../../../scripts/build-prod.sh) erzeugt, das **alle
Instrumente des Workspace** (ORME *und* OSNE) in einem Durchlauf baut. Für jedes
Instrument (`<bin>` = `orme`, `osne`):

| Ausgabe | Ziel | IHM | Methode |
|---------|------|-----|---------|
| `dist/<bin>-linux-x86_64` | `x86_64-unknown-linux-gnu` | ✅ | `cross` |
| `dist/<bin>-windows-x86_64.exe` | `x86_64-pc-windows-gnu` | ✅ | `cross` (mingw) |
| `dist/<bin>-rpi-arm64` | `aarch64-unknown-linux-gnu` (Pi 3/4/5, Pi OS 64b) | ✅ | `cross` |
| Headless-Docker-Image `<bin>:headless` | multi-arch `linux/amd64` + `linux/arm64` | ❌ | `docker buildx` |
| `dist/<bin>_<ver>_amd64.deb` / `_arm64.deb` | Debian/Ubuntu-Paket | ✅ | `dpkg-deb` |
| `dist/<bin>-setup-x86_64.exe` | Windows-Installer | ✅ | NSIS (`makensis`) |

```bash
# Voraussetzung (einmalig) — Docker muss laufen:
cargo install cross

# Alles erzeugen (Exes ORME + OSNE in dist/ + lokale Docker-Images amd64 geladen):
scripts/build-prod.sh

# Variante: MULTI-ARCH-Docker-Images in eine Registry gepusht (<prefix>/<bin>:latest):
IMAGE_PREFIX=ghcr.io/<konto> scripts/build-prod.sh

# Nur ein einziges Instrument bauen:
ONLY=orme scripts/build-prod.sh
```

### Warum `cross` für ALLE Builds (einschließlich Linux x86_64)

`cross` liefert Docker-Images mit den Toolchains jedes Ziels: weder
`mingw-w64`, noch ARM-Toolchain, noch *sysroot* zu installieren.

⚠️ **Natives `cargo` und `cross` nicht im selben `target/` mischen.** Beide
verwenden unterschiedliche `rustc`-Versionen (Host vs. Container); die von einem
kompilierten **Proc-Macros** werden vom anderen abgelehnt, daher Fehler
`can't find crate for …_derive` (z. B. `zerofrom_derive`, `tracing_attributes`).
Das Skript geht daher **immer über `cross`**, auch für Linux x86_64 — eine einzige
Toolchain, reproduzierbare Builds. (Falls der Fehler dennoch nach einem früheren
nativen Build auftritt: `rm -rf target/release`, dann erneut starten.)

### Nach ARM cross-kompilierte IHM: warum es funktioniert

`eframe`/`egui` laden OpenGL, X11/Wayland und xkbcommon **zur Laufzeit**
(`dlopen`): Das Binary verlinkt beim Build nur die `libc`. Es ist daher keine
ARM-Grafikbibliothek auf der Cross-Seite nötig. Auf dem Raspberry Pi eine
Desktop-Umgebung vorsehen (mesa/X11 oder Wayland) — vorhanden auf Raspberry Pi OS
*Desktop*.

> Für ein **Raspbian 32-Bit** auf `armv7-unknown-linux-gnueabihf` zielen (die
> Ziele im Skript anpassen).

### Headless-Docker-Image „überall"

Das Image ([`docker/Dockerfile.headless`](../../../docker/Dockerfile.headless)) geht
von `debian:bookworm-slim` aus und **kopiert** das Headless-Binary der
gewünschten Architektur (keine Kompilierung im Image → kein QEMU). `docker buildx`
setzt das Multi-Arch `amd64`+`arm64` zusammen. Der Server lauscht auf `5502`. Ein
Volume auf `/data` mounten, um `mock_ru_modbustcp.toml` bereitzustellen/zu persistieren.

```bash
# Ohne Registry: lokales amd64-Image geladen, sofort testbar
docker run --rm -p 5502:5502 -v "$PWD/conf:/data" orme:headless
```

### Installer (`.deb` Linux/RPi + Windows-Setup)

Am Ende des Builds ruft `build-prod.sh`
[`scripts/make-installers.sh <bin>`](../../../scripts/make-installers.sh) auf, das
die Release-Ausführbaren aus `dist/` in **Installer** verwandelt:

| Installer | Quelle | Inhalt | Werkzeug |
|-----------|--------|--------|----------|
| `<bin>_<ver>_amd64.deb` | `dist/<bin>-linux-x86_64` | Binary → `/usr/bin`, Desktop-Eintrag, hicolor-Symbol | `dpkg-deb` |
| `<bin>_<ver>_arm64.deb` | `dist/<bin>-rpi-arm64` | dito (Raspberry Pi OS 64-Bit) | `dpkg-deb` |
| `<bin>-setup-x86_64.exe` | `dist/<bin>-windows-x86_64.exe` | exe + Verknüpfungen (Startmenü/Desktop) + Deinstallationsprogramm | NSIS (`makensis`) |

- Die `.deb` legen das Symbol und die `.desktop`-Datei ab; ein `postinst` frischt
  die Symbol-Caches und die `.desktop`-Datenbank auf. Abhängigkeiten: `libc6`;
  grafische Empfehlungen (`libgl1`, `libxkbcommon0`, `libwayland-client0`).
- Der Windows-Installer stammt aus
  [`packaging/windows/installer.nsi`](../../../packaging/windows/installer.nsi);
  seine Verknüpfungen tragen ein mehrauflösendes `.ico`-Symbol, abgeleitet aus
  `pic/<bin>-icon.png` (via Pillow).
- **Voraussetzungen**: `dpkg-deb` (Debian/Ubuntu) für die `.deb`, **`makensis`**
  (`sudo apt install nsis`) für das Windows-Setup, `python3`+Pillow für das `.ico`.
  Jedes Ziel, dessen Werkzeug/Artefakt fehlt, wird **gewarnt und übersprungen** (der
  Build bricht nicht ab). Per `INSTALLERS=0` deaktivieren, oder die Installer eines
  Instruments allein (neu) erzeugen: `scripts/make-installers.sh orme`.

### Nativer Windows-Build (MSVC) — optional

Die oben erzeugte `.exe` ist **GNU/mingw** (native Windows-Ausführbare, IHM
inbegriffen). Falls ein **MSVC**-Binary erforderlich ist, auf einem Windows-Rechner
mit [`scripts/build-windows.ps1`](../../../scripts/build-windows.ps1) kompilieren
(Voraussetzung: Rust + *Visual Studio Build Tools*, Workload „Desktopentwicklung
mit C++"), oder von Linux aus über `cargo-xwin`
(`cargo xwin build --release --target x86_64-pc-windows-msvc`).

### Hinweise

- Die Binärdateien sind **dynamisch an die glibc gebunden**; über `cross`
  kompiliert (alte glibc-Baseline) laufen sie auf aktuellen Distributionen (und in
  `debian:bookworm-slim`). Für ein vollständig statisches Binary auf `*-musl` zielen.
- `dist/` wird von git ignoriert (Build-Artefakte).

---

## 11. Konventionen

- Code und Kommentare auf **Französisch**.
- `cargo clippy --workspace` **warnungsfrei** vor jedem Commit.
- Jedes neue Geschäfts- oder Mapping-Verhalten geht mit einem **Test** einher.
- Der Adressplan wird in **`map.rs`** (Quelle der Wahrheit) geändert, mit
  gleichzeitiger Aktualisierung der Dokumentation.
