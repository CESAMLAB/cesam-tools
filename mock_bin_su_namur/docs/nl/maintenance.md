# Onderhoudsdocumentatie — OSNE (workspace `cesam-tools`)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · **NL** · [PL](../pl/maintenance.md)*

> Publiek: ontwikkelaars die het project onderhouden, corrigeren of uitbreiden.
> Zie ook: [conception.md](conception.md) · [commandes_namur.md](commandes_namur.md).

---

## 1. Vereisten

- **Rust stable** (editie 2021, `rust-version` ≥ 1.85). Installatie: <https://rustup.rs>.
- **Systeemafhankelijkheden (Linux) voor de GUI** (`eframe`/`egui`, OpenGL/winit):
  - Debian/Ubuntu: `libxkbcommon-dev`, `libwayland-dev`, `libxcb1-dev`,
    `libgl1-mesa-dev` (of equivalenten), plus een grafische server (X11/Wayland).
  - De GUI vereist een **scherm**: in een headless-omgeving opent het venster niet
    (de NAMUR-server hangt op zijn beurt niet af van het scherm).
- **Seriële verbinding** (feature `serial`): toegang tot de poort (`/dev/ttyUSB*`,
  groep `dialout` onder Linux). Zonder hardware, gebruik het **TCP**-transport.
- Netwerktoegang tot het crates.io-register voor de eerste compilatie.

---

## 2. Veelvoorkomende commando's

```bash
cargo check -p mock_bin_su_namur          # Snelle verificatie (zonder codegen)
cargo build -p mock_bin_su_namur          # Debug-compilatie
cargo build --release -p mock_bin_su_namur   # Geoptimaliseerde compilatie (LTO thin)
cargo test  -p mock_bin_su_namur          # Unit- + integratietests
cargo clippy --workspace --all-targets    # Lint (moet ZONDER waarschuwing blijven)
cargo run   -p mock_bin_su_namur          # Start de roerder (GUI + NAMUR/TCP)

# Alternatief configuratiebestand:
MOCK_CONFIG=./ma_config.toml cargo run -p mock_bin_su_namur
# Gedetailleerde logging:
RUST_LOG=debug cargo run -p mock_bin_su_namur
```

Geproduceerd binair bestand: `target/debug/osne` of `target/release/osne` (het
Cargo-pakket blijft `mock_bin_su_namur`, maar het uitvoerbare bestand heet
**`osne`** — zie `[[bin]]` in de `Cargo.toml` van de crate).

### Cargo-features

| Feature | Standaard | Effect |
|---------|:---------:|--------|
| `gui` | ✅ | GUI `egui`/`eframe` (anders headless-binair) |
| `serial` | ✅ | NAMUR-transport over seriële RS-232-verbinding via `tokio-serial` |

```bash
cargo build -p mock_bin_su_namur --no-default-features                  # headless, alleen NAMUR/TCP
cargo build -p mock_bin_su_namur --no-default-features --features serial # headless TCP + serieel
cargo build -p mock_bin_su_namur --no-default-features --features gui    # GUI, alleen TCP (zonder serieel)
```

> ⚠️ **`serial` = native afhankelijkheid.** `tokio-serial` opent de poort via
> termios (Linux); de `libudev`-enumeratie is uitgeschakeld
> (`default-features = false`). Bij **cross-compilatie** (`build-prod.sh`,
> desktop-exes met standaardfeatures) kan de `cross`-image van het target alsnog
> de seriële headers opvragen; als de toolchain problemen geeft, verwijder
> `serial` uit de betreffende build. De **headless Docker is niet getroffen** (deze
> build met `--no-default-features`).

---

## 3. Organisatie van de code

```
mock_lib_control/        Bibliothèque de régulation (pure, sans IO, testable)
  src/pid.rs             PID anti-emballement (réutilisé pour l'asservissement de vitesse)
  src/lib.rs             ré-exports (feature `serde` optionnelle)

mock_bin_su_namur/       Binaire agitateur (exécutable `osne`)
  src/main.rs            Démarrage : config, runtime Tokio, acteurs, IHM
  src/motor.rs           Modèle physique du moteur (dynamique rotationnelle, Euler)
  src/stirrer.rs         Modèle métier synchrone (état, Command, step) — possède le PID
  src/config.rs          AppConfig (TOML), Transport/SerialConfig, IpFilter, ServerStatus
  src/namur.rs           Protocole NAMUR : handle_line (SOURCE DE VÉRITÉ du jeu de commandes)
  src/namur_server.rs    Service NAMUR (lignes ASCII) + mono-maître TCP + serve série + chien de garde
  src/trace.rs           Journal circulaire des trames (mini-terminal IHM)
  src/gui.rs             IHM egui (page unique + mini-terminal + modal Paramètres)
  src/branding.rs        Logos embarqués (feature `gui`)
  src/i18n.rs            Catalogue i18n typé (8 langues), sans dépendance
  src/actors/
    simulation.rs        Boucle de simulation (tick 20 ms)
    network.rs           Serveur NAMUR TCP/série (re)configurable à chaud

docs/                    Conception, commandes NAMUR, manuel, maintenance (multilingue)
```

**Gulden regel**: de bedrijfslogica (`mock_lib_control`, `motor.rs`, `stirrer.rs`)
blijft **synchroon en getest**; het asynchrone is beperkt tot de acteurs en de IO.
Exacte kopie van de **ORME**-regelaar (`mock_bin_ru_modbustcp`) — dezelfde
invarianten.

---

## 4. Configuratie

- Bestand: `mock_su_namur.toml` in de huidige map, of het pad geleverd door de
  omgevingsvariabele `MOCK_CONFIG`.
- Geladen bij de start; **standaardwaarden** indien afwezig of onleesbaar (er
  wordt een waarschuwing geregistreerd, de applicatie start toch).
- **Elke uit het TOML afkomstige waarde wordt gesaneerd** (`AppConfig::sanitized`):
  herordende grenzen (`min ≤ max`), floats geforceerd eindig, inertie/koppel/
  viscositeit strikt positief. **Invariant: nooit `f32::clamp` met niet-
  gevalideerde grenzen** (paniek bij `min > max` of `NaN`).
- Opgeslagen vanuit de GUI (knoppen *Toepassen* / *Opslaan* / *Resetten*).

Structuur (alle secties zijn optioneel, aangevuld met standaardwaarden):

```toml
language = "fr"

[network]
transport = "tcp"          # "tcp" ou "serial"
bind_ip = "0.0.0.0"
port = 4001
allowlist = ["192.168.1.*", "127.0.0.1"]   # vide = toutes IP autorisées
[network.serial]
port = "/dev/ttyUSB0"
baud = 9600 ; parity = "even" ; data_bits = 7 ; stop_bits = 1   # NAMUR 7E1

[motor]   # J·dω/dt = T − k·η·ω − frottement
inertia = 0.02      # J (réactivité)
load_coeff = 0.05   # k (poids de la viscosité)
friction = 2.0      # N·cm
torque_max = 100.0  # N·cm (plafond de la sortie PID)

[regulation]
speed_min = 0.0 ; speed_max = 2000.0
viscosity = 1.0 ; viscosity_min = 0.1 ; viscosity_max = 20.0
[regulation.pid]
kp = ... ; ki = ... ; kd = ... ; out_min = 0.0 ; out_max = 100.0
```

> De **standaardwaarden** hebben één **enkele bron**: `StirrerConfig::default`
> in `stirrer.rs`. `MotorConfig`/`RegulationConfig` (config.rs) zijn ervan
> afgeleid. De uitgangsgrenzen van de PID (`out_min`/`out_max`) worden **geforceerd**
> naar `[0, couple_max]` op het moment dat de roerder wordt opgebouwd
> (`to_stirrer_config`).

---

## 5. Afhankelijkheden en versievalkuilen

| Crate | Rol | Aandachtspunt |
|-------|-----|---------------|
| `tokio` | async runtime | gedeelde features + **`io-util`** (BufReader / NAMUR ASCII-regels) |
| `ractor` | acteurs | standaardfeatures (native async, **geen** `async-trait`) |
| `tokio-serial` | NAMUR serieel | optioneel (feature `serial`), `default-features = false` (geen libudev-enumeratie) |
| `eframe`/`egui` | GUI | onderling gekoppelde versies |
| `egui_plot` | grafiek | ⚠️ **één minor versie vooruit op `egui`**: voor `egui` 0.33 → `egui_plot` **0.34** |
| `serde`/`toml` | persistentie | `mock_lib_control` biedt een feature `serde` geactiveerd door het binair |

De gedeelde versies zijn gecentraliseerd in `[workspace.dependencies]` van de
root-`Cargo.toml`. Om `egui`/`eframe` op te waarderen, **controleer de
overeenkomstige versie van `egui_plot`** (anders fout « two versions of crate
egui »).

---

## 6. Het project uitbreiden

### 6.1 Een NAMUR-commando toevoegen

Alles gebeurt in **`namur.rs`** (bron van waarheid van het protocol):

1. De tak toevoegen in `handle_line` (lezen → `Reply`, schrijven/actie →
   `Apply(Command)` of `SetWatchdog`).
2. Als het een **actie** is, de variant toevoegen in `enum Command` (`stirrer.rs`)
   en de verwerking ervan in `Stirrer::apply`.
3. De header-doc-commentaar bijwerken, **[commandes_namur.md](commandes_namur.md)**
   en de referentietabel van de miniterminal (`gui.rs`, tabel `rows`).
4. Een test toevoegen in de module `tests` van `namur.rs`.

### 6.2 Een GUI-commando / -instelling toevoegen

1. Variant in `enum Command` (`stirrer.rs`) + verwerking in `Stirrer::apply`.
2. Veld in `StirrerSnapshot` als de waarde waarneembaar moet zijn.
3. GUI-bedrading (`gui.rs`) via een niet-blokkerende `cast`.
4. Indien persistent: veld in `AppConfig` (`config.rs`) + sanering in
   `sanitized` + overdracht in `to_stirrer_config`.

### 6.3 Een interfacestring toevoegen (i18n)

Elke GUI-string **moet** via een sleutel `Msg` (`i18n.rs`) gaan met zijn **8
vertalingen** (array van vaste grootte gecontroleerd bij de compilatie). De
NAMUR-acroniemen, eenheidssuffixen en commandonamen blijven hardcoded.

### 6.4 Een nieuw instrument toevoegen

1. `mock_bin_<nom>/` aanmaken en toevoegen aan de `members` van de
   root-`Cargo.toml`.
2. `mock_lib_control` hergebruiken; alles gemeenschappelijks factoriseren in een
   `mock_lib_*` (bv. promotie van het model `motor.rs` als het een tweede
   instrument dient).
3. Dezelfde opdeling volgen: synchroon model, ractor-acteur(s), protocollaag,
   GUI. Naamconventie: `mock_bin_<type>_<protocole>`.

---

## 7. Teststrategie

- **Unit** (`mock_lib_control`): PID (proportioneel, begrenzing, anti-windup).
- **Motor** (`motor.rs`): rotatiedynamiek, convergentie stationaire toestand,
  effect van de viscositeit op het koppel, verzadiging/overbelasting.
- **Domein** (`stirrer.rs`): convergentie van de snelheid naar het setpoint,
  vertraging bij stop, detectie van overbelasting.
- **Protocol** (`namur.rs`): decodering van de leesopdrachten (`IN_*`), van de
  schrijfopdrachten (`OUT_SP_4`), van de acties (`START/STOP/RESET`), van de
  waakhond en van de onbekende commando's.
- **Config / netwerk** (`config.rs`, `actors/network.rs`): TOML round-trip,
  IP-filter (jokers, IPv4-mapped), sanering zonder paniek, seriële opening in fout
  bij afwezige poort.

Uitvoeren: `cargo test -p mock_bin_su_namur` (of `--workspace`). De tests zijn
**deterministisch en zonder GUI**.

---

## 8. Probleemoplossing

| Symptoom | Spoor |
|----------|-------|
| « two versions of crate `egui` » | Onenigheid `egui_plot` / `egui`: lijn de versies uit (§5). |
| De GUI opent niet | Scherm afwezig (headless) of ontbrekende systeembibliotheken (§1). |
| `NAMUR ✖` in de koptekst | TCP-poort al gebruikt / < 1024 zonder privileges, of seriële poort niet beschikbaar: wijzig in *Parameters*. |
| Een TCP-client wordt geweigerd | IP buiten de **witlijst**: maak de lijst leeg of voeg een patroon toe (`192.168.1.*`). |
| De serieel gaat niet open | Feature `serial` afwezig, verkeerde poort, of permissies (`dialout`). |
| De motor stopt vanzelf | **Waakhond** geactiveerd (`OUT_WD1@…`) zonder verkeer: stuur frames of `OUT_WD1@0`. |
| Permanente overbelasting | Viscositeit te hoog t.o.v. `torque_max`: pas de motorparameters aan. |
| Config niet opnieuw geladen | Verkeerde huidige map of `MOCK_CONFIG`; controleer het journaal bij de start. |

De verbositeit verhogen: `RUST_LOG=debug` (of `trace`).

---

## 9. Distributiebuild

```bash
cargo build --release -p mock_bin_su_namur
# Zelfstandig binair bestand:
target/release/osne
```

Het `release`-profiel activeert `lto = "thin"` en `opt-level = 3` (zie root-
`Cargo.toml`). Om te distribueren: lever het binair + een voorbeeld-
`mock_su_namur.toml`. **MIT**-licentie (bestand `LICENSE`).

### Feature `gui` (build met / zonder interface)

```bash
cargo build --release -p mock_bin_su_namur                       # avec IHM (poste de travail)
cargo build --release -p mock_bin_su_namur --no-default-features  # «headless»: NAMUR + simulatie, zonder GUI
```

De **headless**-modus is bedoeld voor implementaties zonder scherm en maakt de
**ARM-cross-compilatie triviaal** (geen grafische afhankelijkheid te linken).

### Integratie in de Linux-desktop (taakbalkpictogram)

Het OSNE-pictogram (`pic/osne-icon.png`, roerdermotief, gegenereerd door
[`pic/osne-logo.gen.py`](../../../pic/osne-logo.gen.py)) is **ingebed** in het
binair (`branding.rs` → `window_icon`). Dat volstaat onder **X11, Windows en
macOS**. Onder **Wayland** **negeert** de compositor het ingebedde pictogram: hij
koppelt het venster via zijn **`app_id`** (« osne », gedefinieerd in `main.rs` via
`with_app_id`) aan een gelijknamig bestand `osne.desktop`, en toont de in het
pictogramthema `hicolor` opgeloste `Icon=osne`.

Om het pictogram onder Wayland te verkrijgen, installeer het desktopitem voor de
huidige gebruiker:

```bash
scripts/install-desktop.sh osne
```

Het script kopieert:

| Bron | Bestemming |
|------|------------|
| `pic/osne-icon.png` | `~/.local/share/icons/hicolor/256x256/apps/osne.png` |
| `packaging/osne.desktop` | `~/.local/share/applications/osne.desktop` |

en vernieuwt vervolgens de caches. Drie namen **moeten uitgelijnd blijven**: de
`app_id` (`main.rs`), het bestand `osne.desktop` (+ zijn `StartupWMClass`) en het
pictogram `osne.png` (= `Icon=osne`). Hetzelfde script installeert ORME zonder
argument (`scripts/install-desktop.sh`).

---

## 10. « Prod »-build — cross-compilatie vanaf Linux

### Unieke procedure

Alles wordt **vanaf Linux** geproduceerd door
[`scripts/build-prod.sh`](../../../scripts/build-prod.sh), dat **alle instrumenten
van de workspace** bouwt (ORME *en* OSNE):

| Uitvoer | Doel | GUI | Methode |
|---------|------|-----|---------|
| `dist/osne-linux-x86_64` | `x86_64-unknown-linux-gnu` | ✅ | `cross` |
| `dist/osne-windows-x86_64.exe` | `x86_64-pc-windows-gnu` | ✅ | `cross` (mingw) |
| `dist/osne-rpi-arm64` | `aarch64-unknown-linux-gnu` (Pi 3/4/5, Pi OS 64b) | ✅ | `cross` |
| Headless Docker-image `osne:headless` | multi-arch `amd64` + `arm64` | ❌ | `docker buildx` |
| `dist/osne_<ver>_amd64.deb` / `_arm64.deb` | Debian/Ubuntu-pakket | ✅ | `dpkg-deb` |
| `dist/osne-setup-x86_64.exe` | Windows-installer | ✅ | NSIS (`makensis`) |

```bash
# Vereisten (eenmalig) — Docker moet draaien:
cargo install cross

# Alles produceren (exes ORME + OSNE + installers in dist/ + Docker-images amd64):
scripts/build-prod.sh

# Variant: MULTI-ARCH Docker-images gepusht naar een register:
IMAGE_PREFIX=ghcr.io/<account> scripts/build-prod.sh

# Zonder de installers te bouwen:
INSTALLERS=0 scripts/build-prod.sh
```

### Waarom `cross` voor ALLE builds (inclusief Linux x86_64)

`cross` levert Docker-images met de toolchains van elk doel.
⚠️ **Meng geen native `cargo` en `cross` in dezelfde `target/`.** De
**proc-macros** gecompileerd door de ene worden afgewezen door de andere (`can't
find crate for …_derive`). Het script gaat **altijd via `cross`**. (Als de fout
optreedt: `rm -rf target/release` en opnieuw starten.)

### GUI cross-gecompileerd naar ARM: waarom dat werkt

`eframe`/`egui` laden OpenGL, X11/Wayland en xkbcommon **bij de uitvoering**
(`dlopen`): het binair linkt bij de build alleen de `libc`. Geen enkele
ARM-grafische bibliotheek is nodig aan de cross-zijde; voorzie een
desktop-omgeving op het doel.

### Headless Docker-image

De image ([`docker/Dockerfile.headless`](../../../docker/Dockerfile.headless))
vertrekt van `debian:bookworm-slim` en **kopieert** het headless-binair van de
gewenste architectuur (geen compilatie in de image → geen QEMU). De naam van het
binair en de blootgestelde poort worden doorgegeven via `--build-arg` (`BIN=osne`,
`PORT=4001`). Koppel een volume aan `/data` om `mock_su_namur.toml` te leveren/
persisteren.

```bash
docker run --rm -p 4001:4001 -v "$PWD/conf:/data" osne:headless
```

### Installers (`.deb` Linux/RPi + setup Windows)

Aan het einde van elke build roept `build-prod.sh`
[`scripts/make-installers.sh <bin>`](../../../scripts/make-installers.sh) aan, dat
de release-uitvoerbare bestanden uit `dist/` omzet naar **installers**:

| Installer | Bron | Inhoud | Tool |
|-----------|------|--------|------|
| `osne_<ver>_amd64.deb` | `dist/osne-linux-x86_64` | binair → `/usr/bin`, bureaubladvermelding, hicolor-pictogram | `dpkg-deb` |
| `osne_<ver>_arm64.deb` | `dist/osne-rpi-arm64` | idem (Raspberry Pi OS 64-bits) | `dpkg-deb` |
| `osne-setup-x86_64.exe` | `dist/osne-windows-x86_64.exe` | exe + snelkoppelingen (startmenu/bureaublad) + deïnstallatieprogramma | NSIS (`makensis`) |

- De `.deb`-pakketten plaatsen het pictogram en het `.desktop`-bestand; een
  `postinst` vernieuwt de caches (`update-desktop-database`, `gtk-update-icon-cache`).
  Afhankelijkheden: `libc6`; grafische aanbevelingen (`libgl1`, `libxkbcommon0`,
  `libwayland-client0`).
- De Windows-installer wordt gegenereerd op basis van
  [`packaging/windows/installer.nsi`](../../../packaging/windows/installer.nsi); de
  snelkoppelingen gebruiken een `.ico`-pictogram met meerdere resoluties, afgeleid van
  `pic/osne-icon.png` (via Pillow).
- **Vereisten**: `dpkg-deb` (aanwezig op Debian/Ubuntu) voor de `.deb`-pakketten,
  **`makensis`** (`sudo apt install nsis`) voor de Windows-setup, `python3`+Pillow voor
  de `.ico`. Elk doel waarvan de tool of het artefact ontbreekt, wordt **gewaarschuwd
  en overgeslagen** (de build breekt niet). Uitschakelen via `INSTALLERS=0`. Men kan
  ook de installers van één instrument apart (her)genereren:
  `scripts/make-installers.sh osne`.
- De **versie** van de pakketten komt van `[workspace.package].version` van de
  hoofd-`Cargo.toml`.

### Opmerkingen

- De binaire bestanden zijn **dynamisch gelinkt aan de glibc**; gecompileerd via
  `cross` (oude glibc-baseline) draaien ze op recente distributies.
- `dist/` wordt genegeerd door git (build-artefacten).

---

## 11. Conventies

- Code en commentaar in het **Frans**; logs en foutmeldingen in het **Engels**.
- `cargo clippy --workspace` **zonder waarschuwing** vóór elke commit.
- Elk nieuw bedrijfs-, motor- of protocolgedrag gaat gepaard met een **test**.
- De NAMUR-commandoset wordt gewijzigd in **`namur.rs`** (bron van waarheid), met
  gelijktijdige bijwerking van de documentatie.
