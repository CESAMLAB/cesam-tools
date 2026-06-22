# Wartungsdokumentation — OSNE (Workspace `cesam-tools`)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · **DE** · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

> Zielgruppe: Entwickler, die das Projekt pflegen, korrigieren oder erweitern.
> Siehe auch: [conception.md](conception.md) · [commandes_namur.md](commandes_namur.md).

---

## 1. Voraussetzungen

- **Rust stable** (Edition 2021, `rust-version` ≥ 1.85). Installation: <https://rustup.rs>.
- **Systemabhängigkeiten (Linux) für die IHM** (`eframe`/`egui`, OpenGL/winit):
  - Debian/Ubuntu: `libxkbcommon-dev`, `libwayland-dev`, `libxcb1-dev`,
    `libgl1-mesa-dev` (oder Äquivalente), plus ein Grafikserver (X11/Wayland).
  - Die IHM benötigt eine **Anzeige**: In einer Headless-Umgebung öffnet sich das
    Fenster nicht (der NAMUR-Server hängt hingegen nicht von der Anzeige ab).
- **Serielle Verbindung** (Feature `serial`): Zugriff auf den Port (`/dev/ttyUSB*`,
  Gruppe `dialout` unter Linux). Ohne Hardware den Transport **TCP** verwenden.
- Netzwerkzugang zur crates.io-Registry für die erste Kompilierung.

---

## 2. Gängige Befehle

```bash
cargo check -p mock_bin_su_namur          # Schnellprüfung (ohne Codegen)
cargo build -p mock_bin_su_namur          # Debug-Kompilierung
cargo build --release -p mock_bin_su_namur   # Optimierte Kompilierung (LTO thin)
cargo test  -p mock_bin_su_namur          # Unit- + Integrationstests
cargo clippy --workspace --all-targets    # Lint (muss WARNUNGSFREI bleiben)
cargo run   -p mock_bin_su_namur          # Startet den Rührer (IHM + NAMUR/TCP)

# Alternative Konfigurationsdatei:
MOCK_CONFIG=./ma_config.toml cargo run -p mock_bin_su_namur
# Ausführliche Protokollierung:
RUST_LOG=debug cargo run -p mock_bin_su_namur
```

Erzeugtes Binary: `target/debug/osne` oder `target/release/osne` (das Cargo-Paket
bleibt `mock_bin_su_namur`, aber die ausführbare Datei heißt **`osne`** — siehe
`[[bin]]` in der `Cargo.toml` des Crates).

### Cargo-Features

| Feature | Standardmäßig | Wirkung |
|---------|:-------------:|---------|
| `gui` | ✅ | IHM `egui`/`eframe` (sonst Headless-Binary) |
| `serial` | ✅ | NAMUR-Transport über serielle Verbindung RS-232 via `tokio-serial` |

```bash
cargo build -p mock_bin_su_namur --no-default-features                  # headless, nur NAMUR/TCP
cargo build -p mock_bin_su_namur --no-default-features --features serial # headless TCP + seriell
cargo build -p mock_bin_su_namur --no-default-features --features gui    # IHM, nur TCP (ohne seriell)
```

> ⚠️ **`serial` = native Abhängigkeit.** `tokio-serial` öffnet den Port über termios
> (Linux); die `libudev`-Enumeration ist deaktiviert (`default-features = false`).
> Bei **Cross-Kompilierung** (`build-prod.sh`, Desktop-Exes mit Standard-Features)
> kann das `cross`-Image des Targets dennoch serielle Header verlangen; falls die
> Toolchain Probleme bereitet, `serial` aus dem betreffenden Build entfernen. Das
> **Docker-Headless ist nicht betroffen** (es baut mit `--no-default-features`).

---

## 3. Code-Organisation

```
mock_lib_control/        Regelungsbibliothek (pur, ohne IO, testbar)
  src/pid.rs             Anti-Windup-PID (für die Drehzahlregelung wiederverwendet)
  src/lib.rs             Re-Exports (optionale Feature `serde`)

mock_bin_su_namur/       Rührer-Binary (ausführbare Datei `osne`)
  src/main.rs            Start: Konfiguration, Tokio-Runtime, Aktoren, IHM
  src/motor.rs           Physikalisches Motormodell (Rotationsdynamik, Euler)
  src/stirrer.rs         Synchrones Fachmodell (Zustand, Command, step) — besitzt den PID
  src/config.rs          AppConfig (TOML), Transport/SerialConfig, IpFilter, ServerStatus
  src/namur.rs           NAMUR-Protokoll: handle_line (QUELLE DER WAHRHEIT des Befehlssatzes)
  src/namur_server.rs    NAMUR-Dienst (ASCII-Zeilen) + Mono-Master TCP + serielle Bedienung + Watchdog
  src/trace.rs           Ringpuffer-Journal der Rahmen (IHM-Miniterminal)
  src/gui.rs             IHM egui (Einzelseite + Miniterminal + Modal Parameter)
  src/branding.rs        Eingebettete Logos (Feature `gui`)
  src/i18n.rs            Typisierter i18n-Katalog (8 Sprachen), ohne Abhängigkeit
  src/actors/
    simulation.rs        Simulationsschleife (Tick 20 ms)
    network.rs           NAMUR-Server TCP/seriell, im laufenden Betrieb (neu) konfigurierbar

docs/                    Entwurf, NAMUR-Befehle, Handbuch, Wartung (mehrsprachig)
```

**Goldene Regel**: Die Fachlogik (`mock_lib_control`, `motor.rs`, `stirrer.rs`)
bleibt **synchron und getestet**; das Asynchrone bleibt auf die Aktoren und die IO
beschränkt. Exakte Nachbildung des Reglers **ORME** (`mock_bin_ru_modbustcp`) —
gleiche Invarianten.

---

## 4. Konfiguration

- Datei: `mock_su_namur.toml` im aktuellen Verzeichnis, oder durch die
  Umgebungsvariable `MOCK_CONFIG` angegebener Pfad.
- Beim Start geladen; **Standardwerte** bei Fehlen oder Unlesbarkeit (eine Warnung
  wird protokolliert, die Anwendung startet trotzdem).
- **Jeder aus dem TOML stammende Wert wird bereinigt** (`AppConfig::sanitized`):
  neu geordnete Grenzen (`min ≤ max`), endliche Gleitkommawerte erzwungen, Trägheit/
  Drehmoment/Viskosität streng positiv. **Invariante: niemals `f32::clamp` mit nicht
  validierten Grenzen** (Panik bei `min > max` oder `NaN`).
- Aus der IHM gespeichert (Schaltflächen *Anwenden* / *Speichern* / *Zurücksetzen*).

Struktur (alle Abschnitte sind optional, mit Standardwerten ergänzt):

```toml
language = "de"
check_updates = true       # beim Start prüfen, ob eine neuere Release existiert (IHM)

[network]
transport = "tcp"          # "tcp" oder "serial"
bind_ip = "0.0.0.0"
port = 4001
allowlist = ["192.168.1.*", "127.0.0.1"]   # leer = alle IP erlaubt
[network.serial]
port = "/dev/ttyUSB0"
baud = 9600 ; parity = "even" ; data_bits = 7 ; stop_bits = 1   # NAMUR 7E1

[motor]   # J·dω/dt = T − k·η·ω − Reibung
inertia = 0.02      # J (Reaktivität)
load_coeff = 0.05   # k (Gewicht der Viskosität)
friction = 2.0      # N·cm
torque_max = 100.0  # N·cm (Obergrenze der PID-Ausgabe)

[regulation]
speed_min = 0.0 ; speed_max = 2000.0
viscosity = 1.0 ; viscosity_min = 0.1 ; viscosity_max = 20.0
[regulation.pid]
kp = ... ; ki = ... ; kd = ... ; out_min = 0.0 ; out_max = 100.0
```

> Die **Standardwerte** haben eine **einzige Quelle**: `StirrerConfig::default` in
> `stirrer.rs`. `MotorConfig`/`RegulationConfig` (config.rs) leiten sich davon ab.
> Die Ausgangsgrenzen des PID (`out_min`/`out_max`) werden beim Aufbau des Rührers
> auf `[0, couple_max]` **erzwungen** (`to_stirrer_config`).

### Aktualisierungsprüfung

Wenn `check_updates = true` (Standard) **und** das Binary mit der Feature `gui`
kompiliert ist, fragt die IHM **beim Start** die letzte auf GitHub
veröffentlichte Release (`CESAMLAB/cesam-tools`) ab und vergleicht deren Nummer
mit der aktuellen Version. Eine neuere Version zeigt ein klickbares Banner
„🔔 Update verfügbar" an. Die Schaltfläche *Jetzt prüfen* (Modal
*Einstellungen*) startet die Prüfung erneut.

- Die HTTPS-Anfrage läuft in einem **dedizierten Thread**, begrenzt durch einen
  Timeout (5 s): offline oder nicht erreichbares GitHub behindert niemals den
  Start.
- Die Logik liegt in der gemeinsamen Crate **`mock_lib_update`** (`ureq`/`rustls`,
  eingebettete Mozilla-Wurzeln → saubere Cross-Kompilierung unter `cross`).
- **Headless-Build** (`--no-default-features`): die Prüfung — und die gesamte
  Netzwerk-/TLS-Abhängigkeit — ist **nicht vorhanden**. Auf dem Server
  Aktualisierungen über apt/Docker verwalten. Vom Bediener deaktivierbar
  (Kontrollkästchen im Modal).

---

## 5. Abhängigkeiten und Versionsfallen

| Crate | Rolle | Augenmerk |
|-------|-------|-----------|
| `tokio` | Async-Runtime | gemeinsame Features + **`io-util`** (BufReader / NAMUR-ASCII-Zeilen) |
| `ractor` | Aktoren | Standard-Features (natives Async, **kein** `async-trait`) |
| `tokio-serial` | NAMUR seriell | optional (Feature `serial`), `default-features = false` (keine libudev-Enumeration) |
| `eframe`/`egui` | IHM | untereinander verknüpfte Versionen |
| `egui_plot` | Kurve | ⚠️ **um eine Minor-Version vor `egui` versioniert**: für `egui` 0.33 → `egui_plot` **0.34** |
| `serde`/`toml` | Persistenz | `mock_lib_control` stellt eine vom Binary aktivierte Feature `serde` bereit |
| `mock_lib_update` (`ureq`/`rustls`) | Aktualisierungsprüfung | **nur Feature `gui`**; rustls 0.23 (webpki aktuell); fehlt im Headless-Build |

Die gemeinsam genutzten Versionen sind in `[workspace.dependencies]` der Wurzel-
`Cargo.toml` zentralisiert. Um `egui`/`eframe` anzuheben, **die entsprechende
Version von `egui_plot` prüfen** (sonst Fehler „two versions of crate egui").

---

## 6. Das Projekt erweitern

### 6.1 Einen NAMUR-Befehl hinzufügen

Alles geschieht in **`namur.rs`** (Quelle der Wahrheit des Protokolls):

1. Den Zweig in `handle_line` hinzufügen (Lesung → `Reply`, Schreiben/Aktion →
   `Apply(Command)` oder `SetWatchdog`).
2. Handelt es sich um eine **Aktion**, die Variante in `enum Command` (`stirrer.rs`)
   und ihre Behandlung in `Stirrer::apply` hinzufügen.
3. Den Kopf-Doc-Kommentar, **[commandes_namur.md](commandes_namur.md)** und die
   Referenztabelle des Miniterminals (`gui.rs`, Tabelle `rows`) aktualisieren.
4. Einen Test im Modul `tests` von `namur.rs` hinzufügen.

### 6.2 Einen IHM-Befehl / eine IHM-Einstellung hinzufügen

1. Variante in `enum Command` (`stirrer.rs`) + Behandlung in `Stirrer::apply`.
2. Feld in `StirrerSnapshot`, falls der Wert beobachtbar sein soll.
3. IHM-Verdrahtung (`gui.rs`) über einen nicht blockierenden `cast`.
4. Falls persistent: Feld in `AppConfig` (`config.rs`) + Bereinigung in `sanitized`
   + Übertragung in `to_stirrer_config`.

### 6.3 Eine Oberflächen-Zeichenkette hinzufügen (i18n)

Jede IHM-Zeichenkette **muss** über einen `Msg`-Schlüssel (`i18n.rs`) mit ihren **8
Übersetzungen** laufen (Array fester Größe, zur Kompilierzeit geprüft). Die NAMUR-
Akronyme, Einheitensuffixe und Befehlsnamen bleiben fest codiert.

### 6.4 Ein neues Instrument hinzufügen

1. `mock_bin_<nom>/` erstellen und zu den `members` der Wurzel-`Cargo.toml`
   hinzufügen.
2. `mock_lib_control` wiederverwenden; alles Gemeinsame in eine `mock_lib_*`
   herausfaktorisieren (z. B. Hochstufung des Modells `motor.rs`, falls es ein
   zweites Instrument bedient).
3. Derselben Aufteilung folgen: synchrones Modell, ractor-Aktor(en),
   Protokollschicht, IHM. Namenskonvention: `mock_bin_<type>_<protocole>`.

---

## 7. Teststrategie

- **Unit** (`mock_lib_control`): PID (proportional, Begrenzung, Anti-Windup).
- **Motor** (`motor.rs`): Rotationsdynamik, Konvergenz im stationären Zustand,
  Wirkung der Viskosität auf das Drehmoment, Sättigung/Überlast.
- **Domäne** (`stirrer.rs`): Konvergenz der Drehzahl zum Sollwert, Verzögerung beim
  Stopp, Überlasterkennung.
- **Protokoll** (`namur.rs`): Dekodierung der Lesungen (`IN_*`), der Schreibvorgänge
  (`OUT_SP_4`), der Aktionen (`START/STOP/RESET`), des Watchdogs und der unbekannten
  Befehle.
- **Konfiguration / Netzwerk** (`config.rs`, `actors/network.rs`): TOML-Round-Trip,
  IP-Filter (Platzhalter, IPv4-mapped), Bereinigung ohne Panik, serielle Öffnung mit
  Fehler bei fehlendem Port.

Ausführen: `cargo test -p mock_bin_su_namur` (oder `--workspace`). Die Tests sind
**deterministisch und ohne IHM**.

---

## 8. Fehlerbehebung

| Symptom | Ansatz |
|---------|--------|
| „two versions of crate `egui`" | Uneinigkeit `egui_plot` / `egui`: Versionen angleichen (§5). |
| Die IHM öffnet sich nicht | Anzeige fehlt (headless) oder fehlende Systembibliotheken (§1). |
| `NAMUR ✖` im Kopfbereich | TCP-Port bereits belegt / < 1024 ohne Privilegien, oder serieller Port nicht verfügbar: in *Parameter* ändern. |
| Ein TCP-Client wird abgelehnt | IP außerhalb der **Whitelist**: Liste leeren oder ein Muster hinzufügen (`192.168.1.*`). |
| Seriell öffnet sich nicht | Feature `serial` fehlt, falscher Port, oder Berechtigungen (`dialout`). |
| Der Motor stoppt von selbst | **Watchdog** armiert (`OUT_WD1@…`) ohne Verkehr: Rahmen senden oder `OUT_WD1@0`. |
| Dauerhafte Überlast | Viskosität zu hoch gegenüber `torque_max`: Motorparameter anpassen. |
| Konfiguration nicht neu geladen | Falsches aktuelles Verzeichnis oder `MOCK_CONFIG`; das Journal beim Start prüfen. |

Ausführlichkeit erhöhen: `RUST_LOG=debug` (oder `trace`).

---

## 9. Distributionsbuild

```bash
cargo build --release -p mock_bin_su_namur
# Eigenständiges Binary:
target/release/osne
```

Das `release`-Profil aktiviert `lto = "thin"` und `opt-level = 3` (siehe Wurzel-
`Cargo.toml`). Zur Verteilung: das Binary + ein Beispiel-`mock_su_namur.toml`
bereitstellen. **MIT**-Lizenz (Datei `LICENSE`).

### Feature `gui` (Build mit / ohne Oberfläche)

```bash
cargo build --release -p mock_bin_su_namur                       # mit IHM (Arbeitsplatz)
cargo build --release -p mock_bin_su_namur --no-default-features  # „headless": NAMUR + Simulation, ohne IHM
```

Der **Headless**-Modus ist für bildschirmlose Deployments gedacht und macht die
**ARM-Cross-Kompilierung trivial** (keine zu bindende Grafikabhängigkeit).

### Integration in den Linux-Desktop (Symbol in der Taskleiste)

Das OSNE-Symbol (`pic/osne-icon.png`, Rührmotiv, generiert von
[`pic/osne-logo.gen.py`](../../../pic/osne-logo.gen.py)) ist in das Binary
**eingebettet** (`branding.rs` → `window_icon`). Das genügt unter **X11, Windows
und macOS**. Unter **Wayland** **ignoriert** der Compositor das eingebettete
Symbol: Er ordnet das Fenster über seine **`app_id`** („osne", in `main.rs` über
`with_app_id` definiert) einer gleichnamigen `osne.desktop`-Datei zu und zeigt das
im Icon-Theme `hicolor` aufgelöste `Icon=osne` an.

Um das Symbol unter Wayland zu erhalten, den Desktop-Eintrag für den aktuellen
Benutzer installieren:

```bash
scripts/install-desktop.sh osne
```

Das Skript kopiert:

| Quelle | Ziel |
|--------|------|
| `pic/osne-icon.png` | `~/.local/share/icons/hicolor/256x256/apps/osne.png` |
| `packaging/osne.desktop` | `~/.local/share/applications/osne.desktop` |

und aktualisiert anschließend die Caches. Drei Namen **müssen aufeinander
abgestimmt bleiben**: die `app_id` (`main.rs`), die Datei `osne.desktop` (+ deren
`StartupWMClass`) und das Symbol `osne.png` (= `Icon=osne`). Dasselbe Skript
installiert ORME ohne Argument (`scripts/install-desktop.sh`).

---

## 10. „Prod"-Build — Cross-Kompilierung von Linux aus

### Einziges Verfahren

Alles wird **von Linux aus** durch
[`scripts/build-prod.sh`](../../../scripts/build-prod.sh) erzeugt, das **alle
Instrumente des Workspace** baut (ORME *und* OSNE):

| Ausgabe | Ziel | IHM | Methode |
|---------|------|-----|---------|
| `dist/osne-linux-x86_64` | `x86_64-unknown-linux-gnu` | ✅ | `cross` |
| `dist/osne-windows-x86_64.exe` | `x86_64-pc-windows-gnu` | ✅ | `cross` (mingw) |
| `dist/osne-rpi-arm64` | `aarch64-unknown-linux-gnu` (Pi 3/4/5, Pi OS 64b) | ✅ | `cross` |
| Docker-Headless-Image `osne:headless` | multi-arch `amd64` + `arm64` | ❌ | `docker buildx` |
| `dist/osne_<ver>_amd64.deb` / `_arm64.deb` | Debian/Ubuntu-Paket | ✅ | `dpkg-deb` |
| `dist/osne-setup-x86_64.exe` | Windows-Installer | ✅ | NSIS (`makensis`) |

```bash
# Voraussetzungen (einmalig) — Docker muss laufen:
cargo install cross

# Alles erzeugen (Exes ORME + OSNE + Installer in dist/ + Docker-Images amd64):
scripts/build-prod.sh

# Variante: MULTI-ARCH-Docker-Images, in eine Registry gepusht:
IMAGE_PREFIX=ghcr.io/<compte> scripts/build-prod.sh

# Ohne die Installer zu bauen:
INSTALLERS=0 scripts/build-prod.sh
```

### Warum `cross` für ALLE Builds (auch Linux x86_64)

`cross` stellt Docker-Images mit den Toolchains jedes Ziels bereit.
⚠️ **`cargo` nativ und `cross` nicht im selben `target/` mischen.** Die von einem
kompilierten **proc-macros** werden vom anderen abgelehnt (`can't find crate for
…_derive`). Das Skript läuft **immer über `cross`**. (Falls der Fehler auftritt:
`rm -rf target/release` und dann erneut starten.)

### IHM, nach ARM cross-kompiliert: warum das funktioniert

`eframe`/`egui` laden OpenGL, X11/Wayland und xkbcommon **zur Laufzeit** (`dlopen`):
Das Binary bindet beim Build nur die `libc`. Keine ARM-Grafikbibliothek ist
cross-seitig nötig; eine Desktop-Umgebung auf dem Ziel vorsehen.

### Docker-Headless-Image

Das Image ([`docker/Dockerfile.headless`](../../../docker/Dockerfile.headless))
geht von `debian:bookworm-slim` aus und **kopiert** das Headless-Binary der
gewünschten Architektur (keine Kompilierung im Image → kein QEMU). Der Binärname
und der exponierte Port werden per `--build-arg` übergeben (`BIN=osne`, `PORT=4001`).
Ein Volume auf `/data` mounten, um `mock_su_namur.toml` bereitzustellen/zu
persistieren.

```bash
docker run --rm -p 4001:4001 -v "$PWD/conf:/data" osne:headless
```

### Installer (`.deb` Linux/RPi + Windows-Setup)

Am Ende jedes Builds ruft `build-prod.sh`
[`scripts/make-installers.sh <bin>`](../../../scripts/make-installers.sh) auf, das
die Release-Ausführbaren aus `dist/` in **Installer** verwandelt:

| Installer | Quelle | Inhalt | Werkzeug |
|-----------|--------|--------|----------|
| `osne_<ver>_amd64.deb` | `dist/osne-linux-x86_64` | Binary → `/usr/bin`, Desktop-Eintrag, hicolor-Symbol | `dpkg-deb` |
| `osne_<ver>_arm64.deb` | `dist/osne-rpi-arm64` | dito (Raspberry Pi OS 64-Bit) | `dpkg-deb` |
| `osne-setup-x86_64.exe` | `dist/osne-windows-x86_64.exe` | exe + Verknüpfungen (Startmenü/Desktop) + Deinstallationsprogramm | NSIS (`makensis`) |

- Die `.deb` legen das Symbol und die `.desktop`-Datei ab; ein `postinst` frischt
  die Caches auf (`update-desktop-database`, `gtk-update-icon-cache`).
  Abhängigkeiten: `libc6`; grafische Empfehlungen (`libgl1`, `libxkbcommon0`,
  `libwayland-client0`).
- Der Windows-Installer wird aus
  [`packaging/windows/installer.nsi`](../../../packaging/windows/installer.nsi)
  erzeugt; die Verknüpfungen verwenden ein mehrauflösendes `.ico`-Symbol, abgeleitet
  aus `pic/osne-icon.png` (via Pillow).
- **Voraussetzungen**: `dpkg-deb` (auf Debian/Ubuntu vorhanden) für die `.deb`,
  **`makensis`** (`sudo apt install nsis`) für das Windows-Setup, `python3`+Pillow
  für das `.ico`. Jedes Ziel, dessen Werkzeug oder Artefakt fehlt, wird **gewarnt und
  übersprungen** (der Build bricht nicht ab). Per `INSTALLERS=0` deaktivieren. Man
  kann auch die Installer eines Instruments allein (neu) erzeugen:
  `scripts/make-installers.sh osne`.
- Die **Version** der Pakete stammt aus `[workspace.package].version` der
  `Cargo.toml` im Stammverzeichnis.

### Hinweise

- Die Binaries sind **dynamisch mit der glibc gelinkt**; über `cross` kompiliert
  (alte glibc-Baseline) laufen sie auf aktuellen Distributionen.
- `dist/` wird von git ignoriert (Build-Artefakte).

---

## 11. Konventionen

- Code und Kommentare auf **Französisch**; Logs und Fehlermeldungen auf **Englisch**.
- `cargo clippy --workspace` **warnungsfrei** vor jedem Commit.
- Jedes neue Fach-, Motor- oder Protokollverhalten wird von einem **Test**
  begleitet.
- Der NAMUR-Befehlssatz wird in **`namur.rs`** geändert (Quelle der Wahrheit), mit
  gleichzeitiger Aktualisierung der Dokumentation.
