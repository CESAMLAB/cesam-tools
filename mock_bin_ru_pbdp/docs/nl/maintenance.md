# Onderhoudsdocumentatie — ORPD / PROFIBUS DP (workspace `cesam-tools`)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · **NL** · [PL](../pl/maintenance.md)*

> Crate: `mock_bin_ru_pbdp` · Uitvoerbaar bestand: **ru_pbdp** · Merk: **ORPD**
> Doelgroep: ontwikkelaars die het project onderhouden, repareren of uitbreiden.
> Zie ook: [conception.md](conception.md) · [reference_profibus.md](reference_profibus.md).

---

## 1. Vereisten

- **Rust stable** (editie 2021, `rust-version` ≥ 1.85). Installatie:
  <https://rustup.rs>.
- **Systeemafhankelijkheden (Linux) voor de GUI** (`eframe`/`egui`,
  OpenGL/winit): `libxkbcommon-dev`, `libwayland-dev`, `libxcb1-dev`,
  `libgl1-mesa-dev` (of equivalenten), plus een grafische server
  (X11/Wayland). De GUI heeft een **beeldscherm** nodig: in een headless
  omgeving opent het venster niet.
- **Seriële verbinding** (poorttoegang, `/dev/ttyUSB*`, `dialout`-groep
  onder Linux): in tegenstelling tot ORME/OSNE is dit hier **geen
  optionele functie** — `tokio-serial` is een directe afhankelijkheid
  (zie §5), aangezien de seriële verbinding het enige transport van dit
  instrument is (er bestaat geen standaardequivalent van «PROFIBUS over
  TCP»). Zonder hardware start de GUI toch (de openingsfout wordt
  getoond in de kop, de simulatie blijft draaien) — zie
  [`manuel_utilisateur.md`](manuel_utilisateur.md) §2.
- Netwerktoegang tot het crates.io-register voor de eerste build.

---

## 2. Gangbare commando's

```bash
cargo check -p mock_bin_ru_pbdp          # Snelle controle (zonder codegen)
cargo build -p mock_bin_ru_pbdp          # Debug-build
cargo build --release -p mock_bin_ru_pbdp   # Geoptimaliseerde build (thin LTO)
cargo test  -p mock_bin_ru_pbdp          # Unit- + integratietests
cargo clippy --workspace --all-targets    # Lint (moet ZONDER waarschuwingen blijven)
cargo run   -p mock_bin_ru_pbdp          # Start de GUI + de seriële PROFIBUS-DP-verbinding

# Alternatief configuratiebestand:
MOCK_CONFIG=./mijn_config.toml cargo run -p mock_bin_ru_pbdp
# Uitgebreide logging:
RUST_LOG=debug cargo run -p mock_bin_ru_pbdp
```

Geproduceerd binair bestand: `target/debug/ru_pbdp` of
`target/release/ru_pbdp` (het Cargo-pakket blijft `mock_bin_ru_pbdp`; het
uitvoerbare bestand en de commerciële naam «ORPD» zijn louter
documentair, zie `[[bin]]` in de `Cargo.toml` van de crate).

### Cargo-features

| Feature | Standaard | Effect |
|---------|:---------:|-------|
| `gui` | ✅ | `egui`/`eframe`-GUI + update-controle (anders een headless binair bestand) |

```bash
cargo build -p mock_bin_ru_pbdp --no-default-features   # headless: seriële verbinding + simulatie, zonder GUI
```

> ⚠️ **Verschil met ORME/OSNE**: bij deze twee instrumenten is de seriële
> verbinding (RTU/serieel) zelf een **optionele functie** naast een
> steeds aanwezig TCP-transport, en `--no-default-features` kan deze
> uitsluiten. Hier bestaat **geen «seriële-vrije» variant**:
> `tokio-serial` is een directe afhankelijkheid (niet feature-gestuurd),
> aanwezig in **elke** build, inclusief headless — het is het enige
> transport van het instrument.

---

## 3. Code-organisatie

```
mock_lib_control/        Herbruikbare regelbibliotheek (puur, zonder IO, testbaar)
  src/pid.rs             Anti-windup PID
  src/lib.rs             re-exports (optionele feature `serde`)

mock_bin_ru_pbdp/        PROFIBUS-DP-regelaarbinary (uitvoerbaar bestand `ru_pbdp`)
  src/main.rs            Opstart: configuratie, Tokio-runtime, actoren, GUI/headless
  src/regulator.rs        Synchroon bedrijfsmodel (PID + proces 1e orde), Command, stap
  src/config.rs           AppConfig (TOML), SerialConfig, ProcessConfig, RegulationConfig, ServerStatus
  src/profibus.rs         PROFIBUS-DP-V0-protocol: framecodec + FCS + SlaveFsm (BRON VAN WAARHEID)
  src/profibus_server.rs  Seriële sessielus (frame lezen → SlaveFsm → antwoord) + watchdog
  src/map.rs              Indeling van de Output/Input-I/O-blokken <-> Command van de regelaar
  src/trace.rs            Circulair framelog (mini-terminal van de GUI)
  src/gui.rs              egui-GUI (enkele pagina + mini-terminal + modaal Instellingen)
  src/branding.rs         Ingebedde logo's (feature `gui`)
  src/i18n.rs             Getypeerde i18n-catalogus (8 talen), zonder afhankelijkheid
  src/actors/
    simulation.rs         Regellus (simulatiestap 50 ms)
    network.rs            Actor van de seriële PROFIBUS-DP-verbinding, hot-herconfigureerbaar

docs/                     Ontwerp, PROFIBUS-referentie, handleiding, onderhoud (meertalig)
```

**Gouden regel**: de bedrijfslogica (`mock_lib_control`, `regulator.rs`,
`profibus.rs`, `map.rs`) blijft **synchroon en getest**; asynchroniciteit
is beperkt tot de actoren en de seriële IO. Regelaarmodel naar het
voorbeeld van **ORME** (`mock_bin_ru_modbus`) — dezelfde invarianten.

---

## 4. Configuratie

- Bestand: `mock_ru_pbdp.toml` in de huidige map, of een pad opgegeven
  via de omgevingsvariabele `MOCK_CONFIG`.
- Geladen bij het opstarten; **standaardwaarden** indien afwezig of
  onleesbaar (een waarschuwing wordt gelogd, de applicatie start toch).
- **Elke waarde afkomstig uit de TOML wordt gesaneerd**
  (`AppConfig::sanitized`): setpoint-/PID-grenzen herordend,
  zwevendekommawaarden geforceerd eindig, `τ ≥ 1e-3`, `dead_time`
  begrensd, **stationsadres begrensd tot `[0, 125]`**. **Invariant: nooit
  `f32::clamp` aanroepen met ongevalideerde grenzen** (paniekt bij
  `min > max` of `NaN`).
- Opgeslagen vanuit de GUI (knoppen *Toepassen* / *Opslaan* / *Standaard
  herstellen*).

Structuur (alle secties zijn optioneel, standaard aangevuld):

```toml
language = "nl"
check_updates = true       # bij opstarten controleren of er een nieuwere versie bestaat (GUI)

[network.serial]
port = "/dev/ttyUSB0"      # standaard "COM3" onder Windows
baud = 500000              # genormaliseerde PROFIBUS-DP-waarde (9600 .. 12000000)
station_address = 3        # adres van de gesimuleerde slave (0-125)
watchdog_enabled = true    # staat de door de master aangekondigde watchdog toe (Set_Prm)

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

> Het **seriële frameformaat (8E1)** is vastgelegd door de PROFIBUS-DP-
> norm en is **geen** configuratieveld — zie `SerialConfig::open` in
> [`config.rs`](../../src/config.rs). In tegenstelling tot ORME/OSNE,
> **geen IP-whitelist** (de seriële verbinding is inherent
> punt-naar-punt).

### Updatecontrole

Als `check_updates = true` (standaard) **en** het binaire bestand is
gecompileerd met de feature `gui`, vraagt de GUI **bij het opstarten** de
laatste op GitHub gepubliceerde release op (`CESAMLAB/cesam-tools`) via
de gedeelde crate **`mock_lib_update`** (`ureq`/`rustls`, ingebedde
roots, door timeout begrensde thread). **Afwezig in headless builds**
(`--no-default-features`).

---

## 5. Afhankelijkheden en versievalkuilen

| Crate | Rol | Aandachtspunt |
|-------|------|-------------------|
| `tokio` | asynchrone runtime | gedeelde features + `io-util` |
| `ractor` | actoren | standaardfeatures |
| `tokio-serial` | PROFIBUS-DP-verbinding | **directe, niet feature-gestuurde afhankelijkheid** (zie §2); `default-features = false` (geen `libudev`-enumeratie) |
| `eframe`/`egui` | GUI | onderling gekoppelde versies, feature `gui` |
| `egui_plot` | trendcurve | ⚠️ **een minor versie vooruit op `egui`**: voor `egui` 0.33 → `egui_plot` **0.34** |
| `serde`/`toml` | persistentie | `mock_lib_control` biedt een door het binaire bestand geactiveerde feature `serde` |
| `mock_lib_update` (`ureq`/`rustls`) | update-controle | alleen feature `gui`; afwezig headless |

Gedeelde versies zijn gecentraliseerd in `[workspace.dependencies]` van
de root-`Cargo.toml`. Bij het opwaarderen van `egui`/`eframe`, **de
overeenkomstige versie van `egui_plot` controleren** (anders fout «two
versions of crate egui»).

---

## 6. Het project uitbreiden

### 6.1 Een PROFIBUS-dienst (SAP) toevoegen

Alles gebeurt in **[`profibus.rs`](../../src/profibus.rs)** (bron van
waarheid van het protocol):

1. De constante `SAP_*` en de overeenkomstige variant in `enum Request`
   toevoegen; de decodering bekabelen in `decode_request` (en, voor
   tests, in `encode_request`).
2. Het nieuwe verzoek behandelen in `SlaveFsm::handle` (toestandsovergang
   indien relevant, `Handled` teruggegeven).
3. Het documentatiecommentaar van de module en
   **[reference_profibus.md](reference_profibus.md)** bijwerken.
4. Een test toevoegen in de `tests`-module van `profibus.rs` (en, als de
   volledige sessie is betrokken, in `profibus_server.rs`).

### 6.2 De I/O-blokken (`Output`/`Input`) wijzigen

1. De indeling aanpassen in **[`map.rs`](../../src/map.rs)**
   (`decode_output`/`encode_input`), waarbij `OUTPUT_LEN`/`INPUT_LEN`
   consistent blijven met `SlaveProfile` (`profibus_server.rs`).
2. De tabel van **[reference_profibus.md](reference_profibus.md)** §3
   bijwerken (documentaire bron van waarheid, overgenomen uit het
   documentatiecommentaar van `map.rs`).
3. Een round-trip-test toevoegen in `map.rs`.

### 6.3 Een bedrijfscommando / een GUI-instelling toevoegen

1. Variant in `enum Command` (`regulator.rs`) + behandeling in
   `Regulator::apply` (met sanering).
2. Veld in `RegulatorSnapshot` als de waarde observeerbaar moet zijn.
3. Bekabeling van de GUI (`gui.rs`) via een niet-blokkerende `cast`.
4. Indien persistent: veld in `AppConfig` (`config.rs`) + sanering in
   `sanitized` + terugkoppeling in `to_regulator_config`.

### 6.4 Een interfacetekenreeks (i18n) toevoegen

Elke GUI-tekenreeks **moet** via een `Msg`-sleutel (`i18n.rs`) gaan met
de **8 vertalingen** (array met vaste grootte, gecontroleerd tijdens het
compileren). PROFIBUS-dienstidentificatoren en eenheidssuffixen blijven
hardgecodeerd.

### 6.5 Een nieuw instrument toevoegen

1. `mock_bin_<naam>/` aanmaken en toevoegen aan de `members` van de
   root-`Cargo.toml`.
2. `mock_lib_control` hergebruiken; alles wat gemeenschappelijk is
   factoriseren in een `mock_lib_*`.
3. Dezelfde indeling volgen: synchroon model, `ractor`-actor(en),
   protocollaag, GUI. Naamconventie: `mock_bin_<type>_<protocol>`.

---

## 7. Teststrategie

- **Framecodec** (`profibus.rs`): round-trip van
  `SD1`/`SD2`/`SD3`/`SD4`, afwijzing van onjuiste checksum en lengte,
  codering/decodering van de verzoeken
  (`Slave_Diag`/`Set_Prm`/`Chk_Cfg`/`Data_Exchange`) en van de modusbyte.
- **Toestandsmachine** (`profibus.rs`): volledige sequentie
  `Power_On → Wait_Prm → Wait_Cfg → Data_Exchange`, afwijzing van een
  `Set_Prm` met verkeerde identificatie (blijft in `Wait_Prm`).
- **I/O-blokken** (`map.rs`): een te kort uitvoerblok → geen commando;
  round-trip van setpoint/modus; het invoerblok weerspiegelt de snapshot
  (statusbits, meetwaarde).
- **Configuratie** (`config.rs`): TOML-round-trip, sanering (omgekeerde
  grenzen, niet-eindige waarden, stationsadres buiten bereik) zonder
  paniek, nette fout bij het openen van een ontbrekende seriële poort.
- **Netwerksessie** (`profibus_server.rs`, `#[tokio::test]` op
  `tokio::io::duplex`): volledige handshake tot `Data_Exchange` met
  effectieve toepassing van de commando's, een aan een ander station
  geadresseerd frame genegeerd (geen activiteit gemarkeerd), verlopen
  watchdog die de veilige toestand afdwingt.

Uitvoeren: `cargo test -p mock_bin_ru_pbdp` (of `--workspace`) — **36
tests**, allemaal **deterministisch en zonder GUI**, geen enkele
trage/`#[ignore]`-test (in tegenstelling tot ORUE, waar RSA-generatie
genegeerde tests rechtvaardigt).

---

## 8. Probleemoplossing

| Symptoom | Aanwijzing |
|----------|-------|
| «two versions of crate `egui`» | Discrepantie `egui_plot` / `egui`: versies uitlijnen (§5). |
| De GUI opent niet | Geen beeldscherm (headless) of ontbrekende systeembibliotheken (§1). |
| Fout bij het openen van de seriële poort (GUI-kop) | Ontbrekende poort, verkeerd pad, of rechten (`dialout`-groep onder Linux) — de simulatie blijft draaien zonder verbinding. |
| De verbinding blijft in `Wait_Prm` | De master stuurt geen `Set_Prm` met de verwachte identificatie (`0xEE01`) — zie [reference_profibus.md](reference_profibus.md) §2. |
| De verbinding blijft in `Wait_Cfg` | De ontvangen `Chk_Cfg` kondigt niet `out_len=45`/`in_len=17` aan. |
| Het apparaat stopt vanzelf | Protocolwatchdog geactiveerd (langdurige stilte van de master) — verwachte veilige toestand, geen bug. |
| Geen watchdog terwijl de master er een aanvraagt | `watchdog_enabled = false` in de lokale configuratie: het verzoek van de master wordt bewust genegeerd. |

Verhoog de verbositeit: `RUST_LOG=debug` (of `trace`).

---

## 9. Distributiebuild

```bash
cargo build --release -p mock_bin_ru_pbdp
# Zelfstandig binair bestand:
target/release/ru_pbdp
```

Het `release`-profiel activeert `lto = "thin"` en `opt-level = 3` (zie de
root-`Cargo.toml`). Om te distribueren: lever het binaire bestand plus
een voorbeeld-`mock_ru_pbdp.toml`. Licentie: **MIT** (bestand `LICENSE`).

### Feature `gui` (build met / zonder interface)

```bash
cargo build --release -p mock_bin_ru_pbdp                       # met GUI (werkplek)
cargo build --release -p mock_bin_ru_pbdp --no-default-features  # «headless»: seriële verbinding + simulatie, zonder GUI
```

In tegenstelling tot OSNE maakt de **headless**-modus de seriële
verbinding niet optioneel (§2): deze verwijdert alleen de GUI. Blijft
relevant voor een schermloze inzet die is aangesloten op een echte
seriële/USB-poort.

### Integratie in het Linux-bureaublad (taakbalkicoon)

Het ORPD-icoon (`pic/ru_pbdp-icon.png`, gegenereerd door
[`pic/ru_pbdp-logo.gen.py`](../../../pic/ru_pbdp-logo.gen.py)) is
**ingebed** in het binaire bestand (`branding.rs` → `window_icon`). Dit
volstaat onder **X11, Windows en macOS**. Onder **Wayland** **negeert**
de compositor het ingebedde icoon: hij koppelt het venster via zijn
**`app_id`** («ru_pbdp», ingesteld in `main.rs` via `with_app_id`) aan een
gelijknamig `ru_pbdp.desktop`-bestand, en toont het via het `hicolor`-
icoonthema opgeloste `Icon=ru_pbdp`.

Om het icoon onder Wayland te verkrijgen, installeer de bureaubladinvoer
voor de huidige gebruiker:

```bash
scripts/install-desktop.sh ru_pbdp
```

Het script kopieert:

| Bron | Bestemming |
|--------|-------------|
| `pic/ru_pbdp-icon.png` | `~/.local/share/icons/hicolor/256x256/apps/ru_pbdp.png` |
| `packaging/ru_pbdp.desktop` | `~/.local/share/applications/ru_pbdp.desktop` |

en ververst vervolgens de caches. Drie namen **moeten op elkaar
afgestemd blijven**: de `app_id` (`main.rs`), het bestand
`ru_pbdp.desktop` (+ zijn `StartupWMClass`) en het icoon `ru_pbdp.png`
(= `Icon=ru_pbdp`).

---

## 10. «Prod»-build — cross-compilatie vanaf Linux

Alles wordt **vanaf Linux** geproduceerd door
[`scripts/build-prod.sh`](../../../scripts/build-prod.sh), dat **elk
instrument van de workspace** bouwt (tabel `INSTRUMENTS`, item
`mock_bin_ru_pbdp:ru_pbdp:0` — poort `0`: seriële verbinding, geen
IP-poort):

| Output | Doel | GUI | Methode |
|--------|-------|-----|---------|
| `dist/ru_pbdp-linux-x86_64` | `x86_64-unknown-linux-gnu` | ✅ | `cross` |
| `dist/ru_pbdp-windows-x86_64.exe` | `x86_64-pc-windows-gnu` | ✅ | `cross` (mingw) |
| `dist/ru_pbdp-rpi-arm64` | `aarch64-unknown-linux-gnu` (Pi 3/4/5, Pi OS 64-bits) | ✅ | `cross` |
| Headless Docker-image `ru_pbdp:headless` | multi-arch `amd64` + `arm64` | ❌ | `docker buildx` |
| `dist/ru_pbdp_<ver>_amd64.deb` / `_arm64.deb` | Debian/Ubuntu-pakket | ✅ | `dpkg-deb` |
| `dist/ru_pbdp-setup-x86_64.exe` | Windows-installer | ✅ | NSIS (`makensis`) |

```bash
cargo install cross          # vereiste (eenmalig) — Docker moet draaien
scripts/build-prod.sh        # elk instrument, inclusief ru_pbdp
ONLY=ru_pbdp scripts/build-prod.sh   # alleen dit instrument
```

⚠️ **Meng geen native `cargo` en `cross`** in dezelfde `target/`
(incompatibele proc-macro's → `can't find crate for …_derive`). Het
script gaat altijd via `cross`.

### Headless Docker-image: beperkt nut zonder seriële passthrough

De image ([`docker/Dockerfile.headless`](../../../docker/Dockerfile.headless))
wordt gebouwd zoals voor de andere instrumenten (`EXPOSE 0`, inert
metadata), maar is **pas echt nuttig met een seriëel apparaat gemount**
in de container:

```bash
docker run --rm --device=/dev/ttyUSB0 -v "$PWD/conf:/data" ru_pbdp:headless
```

Zonder `--device` start de container, maar kan geen enkele seriële
poort openen (hetzelfde gedrag als lokaal ontbrekende hardware — zie
§8).

---

## 11. Conventies

- Code en commentaar in het **Frans** (projectbrede conventie); logs en
  foutmeldingen in het **Engels**.
- `cargo clippy --workspace` **zonder waarschuwingen** vóór elke commit.
- Elk nieuw bedrijfs- of protocolgedrag gaat gepaard met een **test**.
- Het PROFIBUS-DP-V0-protocol wordt gewijzigd in **`profibus.rs`** (bron
  van waarheid), samen met een update van
  **[reference_profibus.md](reference_profibus.md)**.
