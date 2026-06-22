# Onderhoudsdocumentatie — ORME (workspace `cesam-tools`)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · **NL** · [PL](../pl/maintenance.md)*

> Publiek: ontwikkelaars die het project onderhouden, corrigeren of uitbreiden.
> Zie ook: [conception.md](conception.md) · [table_modbus.md](table_modbus.md).

---

## 1. Vereisten

- **Rust stable** (editie 2021, `rust-version` ≥ 1.85). Installatie: <https://rustup.rs>.
- **Systeemafhankelijkheden (Linux) voor de GUI** (`eframe`/`egui`, OpenGL/winit):
  - Debian/Ubuntu: `libxkbcommon-dev`, `libwayland-dev`, `libxcb1-dev`,
    `libgl1-mesa-dev` (of equivalenten), plus een grafische server (X11/Wayland).
  - De GUI vereist een **scherm**: in een headless-omgeving opent het venster niet
    (de Modbus-server hangt op zijn beurt niet af van het scherm).
- Netwerktoegang tot het crates.io-register voor de eerste compilatie.

---

## 2. Veelvoorkomende commando's

```bash
cargo check --workspace          # Snelle verificatie (zonder codegen)
cargo build --workspace          # Debug-compilatie
cargo build --release            # Geoptimaliseerde compilatie (LTO thin)
cargo test  --workspace          # Unit- + integratietests
cargo clippy --workspace --all-targets   # Lint (moet ZONDER waarschuwing blijven)
cargo run -p mock_bin_ru_modbustcp       # Start de regelaar

# Alternatief configuratiebestand:
MOCK_CONFIG=./ma_config.toml cargo run -p mock_bin_ru_modbustcp
# Gedetailleerde logging:
RUST_LOG=debug cargo run -p mock_bin_ru_modbustcp
```

Geproduceerd binair bestand: `target/debug/orme` of `target/release/orme` (het
Cargo-pakket blijft `mock_bin_ru_modbustcp`, maar het uitvoerbare bestand heet
**`orme`** — zie `[[bin]]` in de `Cargo.toml` van de crate).

### Cargo-features

| Feature | Standaard | Effect |
|---------|:---------:|--------|
| `gui` | ✅ | GUI `egui`/`eframe` (anders headless-binair) |
| `rtu` | ✅ | Modbus RTU serieel transport (RS485) via `tokio-serial` |

```bash
cargo build --no-default-features                 # headless, alleen Modbus TCP
cargo build --no-default-features --features rtu  # headless TCP + RTU serieel
cargo build --no-default-features --features gui  # GUI, alleen TCP (zonder serieel)
```

> ⚠️ **`rtu` = native afhankelijkheid.** `tokio-serial` opent de poort via termios
> (Linux); de `libudev`-enumeratie is uitgeschakeld (`default-features = false`).
> Bij **cross-compilatie** (`build-prod.sh`, desktop-exes met standaardfeatures) kan
> de `cross`-image van het target alsnog de seriële headers van het systeem
> opvragen; als de toolchain problemen geeft, verwijder `rtu` uit de betreffende
> build. De **headless Docker is niet getroffen** (deze build met
> `--no-default-features`).

---

## 3. Code-organisatie

```
mock_lib_control/        Regelbibliotheek (zuiver, zonder IO, testbaar)
  src/pid.rs             PID met anti-windup
  src/onoff.rs           Aan-uit met symmetrische hysterese + anti-kortsluitcyclus
  src/pwm.rs             Cyclusrelais (PWM / time-proportioning)
  src/process.rs         FOPDT-overdrachtsfunctie
  src/lib.rs             ControllerKind + re-exports (optionele feature `serde`)

mock_bin_ru_modbustcp/   Regelaar-binair
  src/main.rs            Opstart: config, Tokio-runtime, actoren, GUI
  src/regulator.rs       Synchroon bedrijfsmodel (toestand, Command, step)
  src/config.rs          AppConfig (TOML), Transport/SerialConfig, IpFilter, ServerStatus
  src/map.rs             Modbus-adresseringsplan (BRON VAN WAARHEID)
  src/modbus_server.rs   RegulatorService (trait Service) + single-master TCP + serve_rtu
  src/gui.rs             GUI egui (enkele pagina + Parameters-modaal)
  src/actors/
    simulation.rs        Regellus (tick)
    network.rs           Modbus TCP/RTU-server, herconfigureerbaar tijdens werking

docs/                    Ontwerp, Modbus-tabel, onderhoud
```

**Gulden regel**: de bedrijfslogica (`mock_lib_control`, `regulator.rs`) blijft
**synchroon en getest**; het asynchrone deel blijft beperkt tot de actoren en de IO.

---

## 4. Configuratie

- Bestand: `mock_ru_modbustcp.toml` in de huidige map, of het pad dat door de
  omgevingsvariabele `MOCK_CONFIG` wordt geleverd.
- Geladen bij opstart; **standaardwaarden** indien afwezig of onleesbaar (een
  waarschuwing wordt gelogd, de applicatie start toch).
- Opgeslagen vanuit de GUI (knoppen *Toepassen* / *Instellingen opslaan* /
  *Standaardwaarden herstellen*).

Structuur (alle secties zijn optioneel, aangevuld met standaardwaarden):

```toml
[network]
bind_ip = "0.0.0.0"
port = 5502
allowlist = ["192.168.1.*", "127.0.0.1"]   # leeg = alle IP's toegestaan

[process]   # overdrachtsfunctie G(s) = K·e^(-L·s)/(1+T·s)
gain = 1.6        # K (eenheid/%)
tau = 30.0        # T (s)
dead_time = 2.0   # L (s)
ambient = 20.0

[regulation]
sp_min = 0.0
sp_max = 250.0
hysteresis = 2.0
[regulation.pid_heat]   # richting 1 (warm)
kp = 4.0 ; ki = 0.25 ; kd = 1.0 ; out_min = 0.0 ; out_max = 100.0
[regulation.pid_cool]   # richting 2 (koud)
kp = 4.0 ; ki = 0.25 ; kd = 1.0 ; out_min = 0.0 ; out_max = 100.0
```

> De **standaardwaarden** hebben één **enkele bron**: `RegulatorConfig::default`
> in `regulator.rs`. `ProcessConfig`/`RegulationConfig` (config.rs) zijn hiervan
> afgeleid. Om een standaard te wijzigen, pas alleen `RegulatorConfig::default` aan.

---

## 5. Afhankelijkheden en versievalkuilen

| Crate | Rol | Aandachtspunt |
|-------|-----|---------------|
| `tokio` | async-runtime | features: `rt-multi-thread, macros, net, time, sync` |
| `ractor` | actoren | standaardfeatures (native async, **geen** `async-trait`) |
| `tokio-serial` | Modbus RTU serieel | optioneel (feature `rtu`), `default-features = false` (geen libudev-enumeratie) |
| `tokio-modbus` | Modbus TCP | `default-features = false`, feature **`tcp-server`** |
| `eframe`/`egui` | GUI | onderling verbonden versies |
| `egui_plot` | grafiek | ⚠️ **één minor-versie vooruit ten opzichte van `egui`**: voor `egui` 0.33 → `egui_plot` **0.34** |
| `serde`/`toml` | persistentie | `mock_lib_control` biedt een feature `serde` die door het binair wordt geactiveerd |

De gedeelde versies zijn gecentraliseerd in `[workspace.dependencies]` van de
root-`Cargo.toml`. Om `egui`/`eframe` op te waarderen, **verifieer de
overeenkomstige versie van `egui_plot`** (anders fout « two versions of crate
egui »).

---

## 6. Het project uitbreiden

### 6.1 Een Modbus-punt toevoegen

Alles gebeurt in **`map.rs`** (daarna de snapshot/Command indien nodig):

1. Declareer de adresconstante en pas de `*_COUNT` van de betreffende tabel aan.
2. Vul de waarde in `MemoryMap::refresh_from` (toestand → register).
3. Indien het punt schrijfbaar is, decodeer het in `coil_to_command` /
   `holdings_to_commands` (register → `Command`).
4. Werk het kopcommentaar bij **en** [table_modbus.md](table_modbus.md).
5. Voeg de regel toe in de live tabel van de GUI (`gui.rs::modbus_rows`).

### 6.2 Een commando / een instelling toevoegen

1. Variant in `enum Command` (`regulator.rs`) + verwerking in `Regulator::apply`.
2. Veld in `RegulatorSnapshot` als de waarde waarneembaar moet zijn.
3. GUI-bedrading (`gui.rs`) en/of Modbus-decodering (`map.rs`).
4. Indien persistent: veld in `AppConfig` (`config.rs`) + `to_regulator_config`.

### 6.3 Een nieuw instrument toevoegen

1. Maak `mock_bin_<naam>/` en voeg het toe aan de `members` van de root-`Cargo.toml`.
2. Hergebruik `mock_lib_control`; factoriseer al het gemeenschappelijke in een
   `mock_lib_*`.
3. Volg dezelfde indeling: synchroon model, ractor-actor(en), protocollaag, GUI.
   Naamgevingsconventie: `mock_bin_<type>_<protocol>`.

---

## 7. Teststrategie

- **Unit** (`mock_lib_control`): PID (proportioneel, begrenzing, anti-windup),
  TOR (dode zone), proces (convergentie in ingeschakelde toestand).
- **Domein** (`regulator.rs`): PID-convergentie in auto, uitgang in handmatig,
  terugkeer naar de omgevingswaarde bij stop.
- **Mapping** (`map.rs`): round-trip `f32`↔registers, decodering van schrijfactie,
  weigering van gedeeltelijke `f32`-schrijfactie.
- **Config / netwerk** (`config.rs`, `actors/network.rs`): TOML round-trip,
  IP-filter (jokers), effectieve start van de server (bind op een efemere poort).

Uitvoeren: `cargo test --workspace`. De tests zijn **deterministisch en zonder GUI**.

---

## 8. Probleemoplossing

| Symptoom | Aanwijzing |
|----------|------------|
| « two versions of crate `egui` » | Onenigheid `egui_plot` / `egui`: lijn de versies uit (§5). |
| De GUI opent niet | Scherm afwezig (headless) of ontbrekende systeembibliotheken (§1). |
| `Modbus ✖ luisteren mislukt` in de koptekst | Poort al in gebruik of < 1024 zonder rechten: wijzig de poort in *Parameters*. |
| Een client wordt geweigerd | IP buiten de **witte lijst**: maak de lijst leeg of voeg een patroon toe (`192.168.1.*`). |
| Afwijkende `f32`-waarden aan clientzijde | Woordvolgorde (hoogwaardig woord eerst): zie [table_modbus.md](table_modbus.md). |
| Een `f32`-setpointschrijfactie wordt genegeerd | Schrijf **beide** registers van het paar in één verzoek. |
| Config niet herladen | Verkeerde huidige map of `MOCK_CONFIG`; controleer het logboek bij opstart. |
| Geen pictogram in de taakbalk (Linux) | **Wayland**-sessie: het ingebedde pictogram wordt genegeerd. Installeer de bureaubladvermelding: `scripts/install-desktop.sh` (§9). |

Verhoog de uitvoerigheid: `RUST_LOG=debug` (of `trace`).

---

## 9. Distributie-build

```bash
cargo build --release
# Zelfstandig binair bestand:
target/release/orme
```

Het `release`-profiel activeert `lto = "thin"` en `opt-level = 3` (zie root-
`Cargo.toml`). Om te distribueren: lever het binair + een voorbeeld
`mock_ru_modbustcp.toml`. **MIT**-licentie (bestand `LICENSE`).

### Feature `gui` (build met / zonder interface)

De GUI zit achter de Cargo-feature **`gui`**, standaard geactiveerd:

```bash
cargo build --release                       # met GUI (werkstation)
cargo build --release --no-default-features  # «headless»: Modbus + simulatie, zonder GUI
```

De **headless**-modus is bedoeld voor implementaties zonder scherm (Raspberry Pi in
dienst) en maakt **ARM-cross-compilatie triviaal** (geen grafische afhankelijkheid
te linken).

### Integratie in het Linux-bureaublad (taakbalkpictogram)

Het ORME-pictogram is ingebed in het binair (`branding.rs` → `with_icon`). Dat
volstaat onder **X11, Windows en macOS**. Maar onder **Wayland** **negeert** de
compositor het ingebedde pictogram: hij koppelt het venster via zijn **`app_id`**
(« orme », gedefinieerd in `main.rs` via `ViewportBuilder::with_app_id`) aan een
gelijknamig bestand `orme.desktop`, en toont de `Icon=` uit dat bestand (opgelost
in het pictogramthema `hicolor`).

Om het pictogram onder Wayland te verkrijgen, installeer de bureaubladvermelding
voor de huidige gebruiker:

```bash
scripts/install-desktop.sh
```

Het script kopieert:

| Bron | Bestemming |
|------|------------|
| `pic/orme-icon.png` | `~/.local/share/icons/hicolor/256x256/apps/orme.png` |
| `packaging/orme.desktop` | `~/.local/share/applications/orme.desktop` |

en vernieuwt vervolgens de caches (`gtk-update-icon-cache`,
`update-desktop-database`). Het pictogram verschijnt bij de volgende start van ORME
(en betrouwbaar na een herlogin van de Wayland-sessie).

> ⚠️ Drie namen **moeten op één lijn blijven**: de `app_id` (`main.rs`), de naam van
> het bestand `orme.desktop` en zijn `StartupWMClass`, en de naam van het pictogram
> `orme.png` (= `Icon=orme`). `packaging/orme.desktop` veronderstelt een uitvoerbaar
> bestand `orme` in het `PATH` (veld `Exec=`); in dev (`cargo run`) heeft dit veld
> geen invloed op de weergave van het pictogram.

---

## 10. « Prod »-build — cross-compilatie vanaf Linux

### Eenduidige procedure

Alles wordt **vanaf Linux** geproduceerd door
[`scripts/build-prod.sh`](../../../scripts/build-prod.sh), dat **alle instrumenten
van het workspace** (ORME *en* OSNE) in één passe bouwt. Voor elk instrument
(`<bin>` = `orme`, `osne`):

| Uitvoer | Doel | GUI | Methode |
|---------|------|-----|---------|
| `dist/<bin>-linux-x86_64` | `x86_64-unknown-linux-gnu` | ✅ | `cross` |
| `dist/<bin>-windows-x86_64.exe` | `x86_64-pc-windows-gnu` | ✅ | `cross` (mingw) |
| `dist/<bin>-rpi-arm64` | `aarch64-unknown-linux-gnu` (Pi 3/4/5, Pi OS 64b) | ✅ | `cross` |
| Headless Docker-image `<bin>:headless` | multi-arch `linux/amd64` + `linux/arm64` | ❌ | `docker buildx` |
| `dist/<bin>_<ver>_amd64.deb` / `_arm64.deb` | Debian/Ubuntu-pakket | ✅ | `dpkg-deb` |
| `dist/<bin>-setup-x86_64.exe` | Windows-installer | ✅ | NSIS (`makensis`) |

```bash
# Vereisten (eenmalig) — Docker moet draaien:
cargo install cross

# Alles produceren (exes ORME + OSNE in dist/ + lokale Docker-images amd64 geladen):
scripts/build-prod.sh

# Variant: MULTI-ARCH Docker-images gepusht naar een register (<prefix>/<bin>:latest):
IMAGE_PREFIX=ghcr.io/<account> scripts/build-prod.sh

# Slechts één instrument bouwen:
ONLY=orme scripts/build-prod.sh
```

### Waarom `cross` voor ALLE builds (ook Linux x86_64)

`cross` levert Docker-images die de toolchains van elk doel bevatten: geen
`mingw-w64`, geen ARM-toolchain, geen *sysroot* te installeren.

⚠️ **Meng geen native `cargo` en `cross` in dezelfde `target/`.** Beide gebruiken
verschillende `rustc`-versies (host vs container); de **proc-macros** gecompileerd
door de een worden door de ander geweigerd, vandaar fouten `can't find crate for
…_derive` (bv. `zerofrom_derive`, `tracing_attributes`). Het script gaat dus
**altijd via `cross`**, zelfs voor Linux x86_64 — één enkele toolchain,
reproduceerbare builds. (Als de fout toch optreedt na een eerdere native build:
`rm -rf target/release` en herstart.)

### GUI gecross-compileerd naar ARM: waarom het werkt

`eframe`/`egui` laden OpenGL, X11/Wayland en xkbcommon **tijdens runtime**
(`dlopen`): het binair linkt bij de build alleen de `libc`. Geen enkele
ARM-grafische bibliotheek is dus nodig aan de cross-zijde. Voorzie op de Raspberry
Pi een desktopomgeving (mesa/X11 of Wayland) — aanwezig op Raspberry Pi OS
*Desktop*.

> Voor een **32-bits Raspbian**, richt op `armv7-unknown-linux-gnueabihf` (pas de
> doelen in het script aan).

### Headless Docker-image « overal »

De image ([`docker/Dockerfile.headless`](../../../docker/Dockerfile.headless))
vertrekt van `debian:bookworm-slim` en **kopieert** het headless-binair van de
gewenste architectuur (geen compilatie in de image → geen QEMU). `docker buildx`
assembleert de multi-arch `amd64`+`arm64`. De server luistert op `5502`. Mount een
volume op `/data` om `mock_ru_modbustcp.toml` te leveren/behouden.

```bash
# Zonder register: lokale image amd64 geladen, onmiddellijk testbaar
docker run --rm -p 5502:5502 -v "$PWD/conf:/data" orme:headless
```

### Installers (`.deb` Linux/RPi + setup Windows)

Aan het einde van de build roept `build-prod.sh`
[`scripts/make-installers.sh <bin>`](../../../scripts/make-installers.sh) aan, dat
de release-uitvoerbare bestanden uit `dist/` omzet naar **installers**:

| Installer | Bron | Inhoud | Tool |
|-----------|------|--------|------|
| `<bin>_<ver>_amd64.deb` | `dist/<bin>-linux-x86_64` | binair → `/usr/bin`, bureaubladvermelding, hicolor-pictogram | `dpkg-deb` |
| `<bin>_<ver>_arm64.deb` | `dist/<bin>-rpi-arm64` | idem (Raspberry Pi OS 64-bits) | `dpkg-deb` |
| `<bin>-setup-x86_64.exe` | `dist/<bin>-windows-x86_64.exe` | exe + snelkoppelingen (startmenu/bureaublad) + deïnstallatieprogramma | NSIS (`makensis`) |

- De `.deb`-pakketten plaatsen het pictogram en het `.desktop`-bestand; een
  `postinst` vernieuwt de pictogramcaches en de `.desktop`-database. Afhankelijkheden:
  `libc6`; grafische aanbevelingen (`libgl1`, `libxkbcommon0`, `libwayland-client0`).
- De Windows-installer komt van
  [`packaging/windows/installer.nsi`](../../../packaging/windows/installer.nsi); zijn
  snelkoppelingen dragen een `.ico`-pictogram met meerdere resoluties, afgeleid van
  `pic/<bin>-icon.png` (via Pillow).
- **Vereisten**: `dpkg-deb` (Debian/Ubuntu) voor de `.deb`-pakketten, **`makensis`**
  (`sudo apt install nsis`) voor de Windows-setup, `python3`+Pillow voor de `.ico`.
  Elk doel waarvan de tool of het artefact ontbreekt, wordt **gewaarschuwd en
  overgeslagen** (de build breekt niet). Uitschakelen via `INSTALLERS=0`, of de
  installers van één instrument apart (her)genereren: `scripts/make-installers.sh orme`.
- De **versie** van de pakketten komt van `[workspace.package].version` van de
  hoofd-`Cargo.toml`.

### Native Windows-build (MSVC) — optioneel

De hierboven geproduceerde `.exe` is **GNU/mingw** (native Windows uitvoerbaar
bestand, GUI inbegrepen). Als een **MSVC**-binair vereist is, compileer op een
Windows-machine met [`scripts/build-windows.ps1`](../../../scripts/build-windows.ps1)
(vereisten: Rust + *Visual Studio Build Tools*, werkbelasting « Desktop-ontwikkeling
in C++ »), of vanaf Linux via `cargo-xwin` (`cargo xwin build --release --target x86_64-pc-windows-msvc`).

### Opmerkingen

- De binaire bestanden zijn **dynamisch gelinkt aan glibc**; gecompileerd via
  `cross` (oude glibc-baseline) draaien ze op recente distributies (en in
  `debian:bookworm-slim`). Voor een volledig statisch binair, richt op `*-musl`.
- `dist/` wordt door git genegeerd (build-artefacten).

---

## 11. Conventies

- Code en commentaar in het **Frans**.
- `cargo clippy --workspace` **zonder waarschuwing** vóór elke commit.
- Elk nieuw bedrijfs- of mappinggedrag gaat gepaard met een **test**.
- Het adresseringsplan wordt gewijzigd in **`map.rs`** (bron van waarheid), met
  gelijktijdige bijwerking van de documentatie.
