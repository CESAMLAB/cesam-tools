# Documentation de maintenance — ORME (workspace `cesam-tools`)

*🌍 **FR** · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

> Public : développeurs qui maintiennent, corrigent ou étendent le projet.
> Voir aussi : [conception.md](conception.md) · [table_modbus.md](table_modbus.md).

---

## 1. Prérequis

- **Rust stable** (édition 2021, `rust-version` ≥ 1.85). Installation : <https://rustup.rs>.
- **Dépendances système (Linux) pour l'IHM** (`eframe`/`egui`, OpenGL/winit) :
  - Debian/Ubuntu : `libxkbcommon-dev`, `libwayland-dev`, `libxcb1-dev`,
    `libgl1-mesa-dev` (ou équivalents), plus un serveur graphique (X11/Wayland).
  - L'IHM nécessite un **affichage** : en environnement headless, la fenêtre ne
    s'ouvre pas (le serveur Modbus, lui, ne dépend pas de l'affichage).
- Accès réseau au registre crates.io pour la première compilation.

---

## 2. Commandes courantes

```bash
cargo check --workspace          # Vérification rapide (sans codegen)
cargo build --workspace          # Compilation debug
cargo build --release            # Compilation optimisée (LTO thin)
cargo test  --workspace          # Tests unitaires + intégration
cargo clippy --workspace --all-targets   # Lint (doit rester SANS avertissement)
cargo run -p mock_bin_ru_modbustcp       # Lance le régulateur

# Fichier de configuration alternatif :
MOCK_CONFIG=./ma_config.toml cargo run -p mock_bin_ru_modbustcp
# Journalisation détaillée :
RUST_LOG=debug cargo run -p mock_bin_ru_modbustcp
```

Binaire produit : `target/debug/orme` ou `target/release/orme` (le paquet Cargo
reste `mock_bin_ru_modbustcp`, mais l'exécutable s'appelle **`orme`** — voir
`[[bin]]` dans le `Cargo.toml` du crate).

### Features Cargo

| Feature | Par défaut | Effet |
|---------|:---------:|-------|
| `gui` | ✅ | IHM `egui`/`eframe` (sinon binaire headless) |
| `rtu` | ✅ | Transport Modbus RTU série (RS485) via `tokio-serial` |

```bash
cargo build --no-default-features                 # headless, Modbus TCP seul
cargo build --no-default-features --features rtu  # headless TCP + RTU série
cargo build --no-default-features --features gui  # IHM, TCP seul (sans série)
```

> ⚠️ **`rtu` = dépendance native.** `tokio-serial` ouvre le port via termios
> (Linux) ; l'énumération `libudev` est désactivée (`default-features = false`).
> En **cross-compilation** (`build-prod.sh`, exes desktop avec features par
> défaut), l'image `cross` du target peut tout de même réclamer les en-têtes série
> du système ; si la chaîne pose problème, retirer `rtu` du build concerné. Le
> **Docker headless n'est pas impacté** (il build en `--no-default-features`).

---

## 3. Organisation du code

```
mock_lib_control/        Bibliothèque de régulation (pure, sans IO, testable)
  src/pid.rs             PID anti-emballement
  src/onoff.rs           Tout-ou-rien à hystérésis symétrique + anti-court-cycle
  src/pwm.rs             Relais à cycle (PWM / time-proportioning)
  src/process.rs         Fonction de transfert FOPDT
  src/lib.rs             ControllerKind + ré-exports (feature `serde` optionnelle)

mock_bin_ru_modbustcp/   Binaire régulateur
  src/main.rs            Démarrage : config, runtime Tokio, acteurs, IHM
  src/regulator.rs       Modèle métier synchrone (état, Command, step)
  src/config.rs          AppConfig (TOML), Transport/SerialConfig, IpFilter, ServerStatus
  src/map.rs             Plan d'adressage Modbus (SOURCE DE VÉRITÉ)
  src/modbus_server.rs   RegulatorService (trait Service) + mono-maître TCP + serve_rtu
  src/gui.rs             IHM egui (page unique + modal Paramètres)
  src/actors/
    simulation.rs        Boucle de régulation (tick)
    network.rs           Serveur Modbus TCP/RTU (re)configurable à chaud

docs/                    Conception, table Modbus, maintenance
```

**Règle d'or** : la logique métier (`mock_lib_control`, `regulator.rs`) reste
**synchrone et testée** ; l'asynchrone est cantonné aux acteurs et à l'IO.

---

## 4. Configuration

- Fichier : `mock_ru_modbustcp.toml` dans le répertoire courant, ou chemin
  fourni par la variable d'environnement `MOCK_CONFIG`.
- Chargé au démarrage ; **valeurs par défaut** si absent ou illisible (un
  avertissement est journalisé, l'application démarre quand même).
- Sauvegardé depuis l'IHM (boutons *Appliquer* / *Sauvegarder réglages* /
  *Réinitialiser par défaut*).

Structure (toutes les sections sont optionnelles, complétées par défaut) :

```toml
language = "fr"
check_updates = true       # vérifier au démarrage si une release plus récente existe (IHM)

[network]
bind_ip = "0.0.0.0"
port = 5502
allowlist = ["192.168.1.*", "127.0.0.1"]   # vide = toutes IP autorisées

[process]   # fonction de transfert G(s) = K·e^(-L·s)/(1+T·s)
gain = 1.6        # K (unité/%)
tau = 30.0        # T (s)
dead_time = 2.0   # L (s)
ambient = 20.0

[regulation]
sp_min = 0.0
sp_max = 250.0
hysteresis = 2.0
[regulation.pid_heat]   # sens 1 (chaud)
kp = 4.0 ; ki = 0.25 ; kd = 1.0 ; out_min = 0.0 ; out_max = 100.0
[regulation.pid_cool]   # sens 2 (froid)
kp = 4.0 ; ki = 0.25 ; kd = 1.0 ; out_min = 0.0 ; out_max = 100.0
```

> Les **valeurs par défaut** ont une **source unique** : `RegulatorConfig::default`
> dans `regulator.rs`. `ProcessConfig`/`RegulationConfig` (config.rs) en dérivent.
> Pour changer un défaut, modifier `RegulatorConfig::default` uniquement.

### Vérification de mise à jour

Si `check_updates = true` (défaut) **et** que le binaire est compilé avec la
feature `gui`, l'IHM interroge **au démarrage** la dernière release publiée sur
GitHub (`CESAMLAB/cesam-tools`) et compare son numéro à la version courante. Une
version plus récente affiche un bandeau cliquable « 🔔 Mise à jour disponible ».
Le bouton *Vérifier maintenant* (modal *Paramètres*) relance la vérification.

- La requête HTTPS s'exécute dans un **thread dédié**, bornée par un timeout
  (5 s) : hors ligne ou GitHub injoignable n'entrave jamais le démarrage.
- La logique vit dans la crate partagée **`mock_lib_update`** (`ureq`/`rustls`,
  racines Mozilla embarquées → cross-compilation propre sous `cross`).
- **Build headless** (`--no-default-features`) : la vérification — et toute la
  dépendance réseau/TLS — est **absente**. Sur serveur, gérer les MAJ via
  apt/Docker. Désactivable par l'opérateur (case à cocher du modal).

---

## 5. Dépendances et pièges de version

| Crate | Rôle | Point d'attention |
|-------|------|-------------------|
| `tokio` | runtime async | features : `rt-multi-thread, macros, net, time, sync` |
| `ractor` | acteurs | features par défaut (async natif, **pas** `async-trait`) |
| `tokio-serial` | Modbus RTU série | optionnel (feature `rtu`), `default-features = false` (pas d'énumération libudev) |
| `tokio-modbus` | Modbus TCP | `default-features = false`, feature **`tcp-server`** |
| `eframe`/`egui` | IHM | versions liées entre elles |
| `egui_plot` | courbe | ⚠️ **versionné une mineure en avance sur `egui`** : pour `egui` 0.33 → `egui_plot` **0.34** |
| `serde`/`toml` | persistance | `mock_lib_control` expose une feature `serde` activée par le binaire |
| `mock_lib_update` (`ureq`/`rustls`) | vérif. de MAJ | **feature `gui` uniquement** ; rustls 0.23 (webpki à jour) ; absent en headless |

Les versions partagées sont centralisées dans `[workspace.dependencies]` du
`Cargo.toml` racine. Pour monter `egui`/`eframe`, **vérifier la version
correspondante d'`egui_plot`** (sinon erreur « two versions of crate egui »).

---

## 6. Étendre le projet

### 6.1 Ajouter un point Modbus

Tout se passe dans **`map.rs`** (puis le snapshot/Command si nécessaire) :

1. Déclarer la constante d'adresse et ajuster le `*_COUNT` de la table concernée.
2. Renseigner la valeur dans `MemoryMap::refresh_from` (état → registre).
3. Si le point est inscriptible, le décoder dans `coil_to_command` /
   `holdings_to_commands` (registre → `Command`).
4. Mettre à jour le doc-commentaire d'en-tête **et** [table_modbus.md](table_modbus.md).
5. Ajouter la ligne dans la table live de l'IHM (`gui.rs::modbus_rows`).

### 6.2 Ajouter une commande / un réglage

1. Variante dans `enum Command` (`regulator.rs`) + traitement dans `Regulator::apply`.
2. Champ dans `RegulatorSnapshot` si la valeur doit être observable.
3. Câblage IHM (`gui.rs`) et/ou décodage Modbus (`map.rs`).
4. Si persistant : champ dans `AppConfig` (`config.rs`) + `to_regulator_config`.

### 6.3 Ajouter un nouvel instrument

1. Créer `mock_bin_<nom>/` et l'ajouter aux `members` du `Cargo.toml` racine.
2. Réutiliser `mock_lib_control` ; factoriser tout commun dans une `mock_lib_*`.
3. Suivre le même découpage : modèle synchrone, acteur(s) ractor, couche
   protocole, IHM. Convention de nom : `mock_bin_<type>_<protocole>`.

---

## 7. Stratégie de test

- **Unitaires** (`mock_lib_control`) : PID (proportionnel, bornage, anti-windup),
  TOR (zone morte), procédé (convergence régime établi).
- **Domaine** (`regulator.rs`) : convergence PID en auto, sortie en manuel,
  retour à l'ambiant à l'arrêt.
- **Mapping** (`map.rs`) : round-trip `f32`↔registres, décodage d'écriture,
  rejet d'écriture `f32` partielle.
- **Config / réseau** (`config.rs`, `actors/network.rs`) : round-trip TOML, filtre
  IP (jokers), démarrage effectif du serveur (bind sur port éphémère).

Lancer : `cargo test --workspace`. Les tests sont **déterministes et sans IHM**.

---

## 8. Dépannage

| Symptôme | Piste |
|----------|-------|
| « two versions of crate `egui` » | Désaccord `egui_plot` / `egui` : aligner les versions (§5). |
| L'IHM ne s'ouvre pas | Affichage absent (headless) ou libs système manquantes (§1). |
| `Modbus ✖ échec de l'écoute` dans l'en-tête | Port déjà utilisé ou < 1024 sans privilèges : changer le port dans *Paramètres*. |
| Un client est refusé | IP hors **liste blanche** : vider la liste ou ajouter un motif (`192.168.1.*`). |
| Valeurs `f32` aberrantes côté client | Ordre des mots (mot fort en tête) : voir [table_modbus.md](table_modbus.md). |
| Une écriture de consigne `f32` est ignorée | Écrire **les deux** registres de la paire en une requête. |
| Config non rechargée | Mauvais répertoire courant ou `MOCK_CONFIG` ; vérifier le journal au démarrage. |
| Pas d'icône dans la barre des tâches (Linux) | Session **Wayland** : l'icône embarquée est ignorée. Installer l'entrée de bureau : `scripts/install-desktop.sh` (§9, *Intégration au bureau*). |

Augmenter la verbosité : `RUST_LOG=debug` (ou `trace`).

---

## 9. Build de distribution

```bash
cargo build --release
# Binaire autonome :
target/release/orme
```

Le profil `release` active `lto = "thin"` et `opt-level = 3` (voir `Cargo.toml`
racine). Pour distribuer : fournir le binaire + un `mock_ru_modbustcp.toml`
d'exemple. Licence **MIT** (fichier `LICENSE`).

### Feature `gui` (build avec / sans interface)

L'IHM est derrière la feature Cargo **`gui`**, activée par défaut :

```bash
cargo build --release                       # avec IHM (poste de travail)
cargo build --release --no-default-features  # « headless » : Modbus + simulation, sans IHM
```

Le mode **headless** est destiné aux déploiements sans écran (Raspberry Pi en
service) et rend la **cross-compilation ARM triviale** (aucune dépendance
graphique à lier).

### Intégration au bureau Linux (icône de la barre des tâches)

L'icône ORME est embarquée dans le binaire (`branding.rs` → `with_icon`). Cela
suffit sous **X11, Windows et macOS**. Mais sous **Wayland**, le compositeur
**ignore** l'icône embarquée : il associe la fenêtre à son **`app_id`** (« orme »,
défini dans `main.rs` via `ViewportBuilder::with_app_id`) à un fichier
`orme.desktop` du même nom, et affiche l'`Icon=` de ce fichier (résolu dans le
thème d'icônes `hicolor`).

Pour obtenir l'icône sous Wayland, installer l'entrée de bureau pour
l'utilisateur courant :

```bash
scripts/install-desktop.sh
```

Le script copie :

| Source | Destination |
|--------|-------------|
| `pic/orme-icon.png` | `~/.local/share/icons/hicolor/256x256/apps/orme.png` |
| `packaging/orme.desktop` | `~/.local/share/applications/orme.desktop` |

puis rafraîchit les caches (`gtk-update-icon-cache`, `update-desktop-database`).
L'icône apparaît au prochain lancement d'ORME (et de façon fiable après un
relogin de la session Wayland).

> ⚠️ Trois noms **doivent rester alignés** pour que l'association fonctionne :
> l'`app_id` (`main.rs`), le nom du fichier `orme.desktop` et son `StartupWMClass`,
> et le nom de l'icône `orme.png` (= `Icon=orme`). `packaging/orme.desktop`
> suppose un exécutable `orme` dans le `PATH` (champ `Exec=`) ; en dev (`cargo
> run`) ce champ n'a pas d'incidence sur l'affichage de l'icône, l'association se
> faisant par `app_id`/`StartupWMClass`.

---

## 10. Build « prod » — cross-compilation depuis Linux

### Procédure unique

Tout est produit **depuis Linux** par
[`scripts/build-prod.sh`](../../../scripts/build-prod.sh), qui construit **tous les
instruments du workspace** (ORME *et* OSNE) en une passe. Pour chaque instrument
(`<bin>` = `orme`, `osne`) :

| Sortie | Cible | IHM | Méthode |
|--------|-------|-----|---------|
| `dist/<bin>-linux-x86_64` | `x86_64-unknown-linux-gnu` | ✅ | `cross` |
| `dist/<bin>-windows-x86_64.exe` | `x86_64-pc-windows-gnu` | ✅ | `cross` (mingw) |
| `dist/<bin>-rpi-arm64` | `aarch64-unknown-linux-gnu` (Pi 3/4/5, Pi OS 64b) | ✅ | `cross` |
| Image Docker headless `<bin>:headless` | multi-arch `linux/amd64` + `linux/arm64` | ❌ | `docker buildx` |
| `dist/<bin>_<ver>_amd64.deb` / `_arm64.deb` | paquet Debian/Ubuntu | ✅ | `dpkg-deb` |
| `dist/<bin>-setup-x86_64.exe` | installeur Windows | ✅ | NSIS (`makensis`) |

```bash
# Prérequis (une fois) — Docker doit tourner :
cargo install cross

# Tout produire (exes ORME + OSNE dans dist/ + images Docker locales amd64 chargées) :
scripts/build-prod.sh

# Variante : images Docker MULTI-ARCH poussées vers un registre (<prefix>/<bin>:latest) :
IMAGE_PREFIX=ghcr.io/<compte> scripts/build-prod.sh

# Ne construire qu'un seul instrument :
ONLY=orme scripts/build-prod.sh
```

### Pourquoi `cross` pour TOUS les builds (y compris Linux x86_64)

`cross` fournit des images Docker contenant les toolchains de chaque cible : ni
`mingw-w64`, ni toolchain ARM, ni *sysroot* à installer.

⚠️ **Ne pas mélanger `cargo` natif et `cross` dans le même `target/`.** Les deux
utilisent des versions de `rustc` différentes (hôte vs conteneur) ; les
**proc-macros** compilées par l'un sont rejetées par l'autre, d'où des erreurs
`can't find crate for …_derive` (ex. `zerofrom_derive`, `tracing_attributes`).
Le script passe donc **toujours par `cross`**, même pour Linux x86_64 — un seul
toolchain, builds reproductibles. (Si l'erreur survient malgré tout après un
build natif antérieur : `rm -rf target/release` puis relancer.)

### IHM cross-compilée vers ARM : pourquoi ça marche

`eframe`/`egui` chargent OpenGL, X11/Wayland et xkbcommon **à l'exécution**
(`dlopen`) : le binaire ne lie au build que la `libc`. Aucune lib graphique ARM
n'est donc nécessaire côté cross. Sur le Raspberry Pi, prévoir un environnement
de bureau (mesa/X11 ou Wayland) — présent sur Raspberry Pi OS *Desktop*.

> Pour un **Raspbian 32 bits**, viser `armv7-unknown-linux-gnueabihf` (adapter
> les cibles dans le script).

### Image Docker headless « n'importe où »

L'image ([`docker/Dockerfile.headless`](../../../docker/Dockerfile.headless)) part de
`debian:bookworm-slim` et **copie** le binaire headless de l'architecture voulue
(aucune compilation dans l'image → pas de QEMU). `docker buildx` assemble le
multi-arch `amd64`+`arm64`. Le serveur écoute sur `5502`. Monter un volume sur
`/data` pour fournir/persister `mock_ru_modbustcp.toml`.

```bash
# Sans registre : image locale amd64 chargée, testable immédiatement
docker run --rm -p 5502:5502 -v "$PWD/conf:/data" orme:headless
```

### Installeurs (`.deb` Linux/RPi + setup Windows)

En fin de build, `build-prod.sh` appelle
[`scripts/make-installers.sh <bin>`](../../../scripts/make-installers.sh), qui
transforme les exécutables release de `dist/` en **installeurs** :

| Installeur | Source | Contenu | Outil |
|------------|--------|---------|-------|
| `<bin>_<ver>_amd64.deb` | `dist/<bin>-linux-x86_64` | binaire → `/usr/bin`, entrée de bureau, icône hicolor | `dpkg-deb` |
| `<bin>_<ver>_arm64.deb` | `dist/<bin>-rpi-arm64` | idem (Raspberry Pi OS 64 bits) | `dpkg-deb` |
| `<bin>-setup-x86_64.exe` | `dist/<bin>-windows-x86_64.exe` | exe + raccourcis (menu Démarrer/bureau) + désinstalleur | NSIS (`makensis`) |

- Les `.deb` posent l'icône et le `.desktop` ; un `postinst` rafraîchit les caches
  d'icônes et la base `.desktop`. Dépendances : `libc6` ; recommandations
  graphiques (`libgl1`, `libxkbcommon0`, `libwayland-client0`).
- L'installeur Windows vient de
  [`packaging/windows/installer.nsi`](../../../packaging/windows/installer.nsi) ;
  ses raccourcis portent une icône `.ico` multi-résolution dérivée de
  `pic/<bin>-icon.png` (via Pillow).
- **Prérequis** : `dpkg-deb` (Debian/Ubuntu) pour les `.deb`, **`makensis`**
  (`sudo apt install nsis`) pour le setup Windows, `python3`+Pillow pour le `.ico`.
  Toute cible dont l'outil/artefact manque est **avertie et sautée** (le build ne
  casse pas). Désactiver via `INSTALLERS=0`, ou (re)générer seuls les installeurs
  d'un instrument : `scripts/make-installers.sh orme`.

### Build natif Windows (MSVC) — optionnel

Le `.exe` produit ci-dessus est **GNU/mingw** (exécutable Windows natif, IHM
incluse). Si un binaire **MSVC** est requis, compiler sur une machine Windows
avec [`scripts/build-windows.ps1`](../../../scripts/build-windows.ps1) (prérequis :
Rust + *Visual Studio Build Tools*, charge « Développement Desktop en C++ »), ou
depuis Linux via `cargo-xwin` (`cargo xwin build --release --target x86_64-pc-windows-msvc`).

### Notes

- Les binaires sont **dynamiquement liés à la glibc** ; compilés via `cross`
  (baseline glibc ancienne) ils tournent sur des distributions récentes (et dans
  `debian:bookworm-slim`). Pour un binaire totalement statique, viser `*-musl`.
- `dist/` est ignoré par git (artefacts de build).

---

## 11. Conventions

- Code et commentaires en **français**.
- `cargo clippy --workspace` **sans avertissement** avant tout commit.
- Tout nouveau comportement métier ou de mapping s'accompagne d'un **test**.
- Le plan d'adressage se modifie dans **`map.rs`** (source de vérité), avec mise
  à jour conjointe de la documentation.
