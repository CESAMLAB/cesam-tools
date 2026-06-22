# Documentation de maintenance — OSNE (workspace `cesam-tools`)

*🌍 **FR** · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

> Public : développeurs qui maintiennent, corrigent ou étendent le projet.
> Voir aussi : [conception.md](conception.md) · [commandes_namur.md](commandes_namur.md).

---

## 1. Prérequis

- **Rust stable** (édition 2021, `rust-version` ≥ 1.85). Installation : <https://rustup.rs>.
- **Dépendances système (Linux) pour l'IHM** (`eframe`/`egui`, OpenGL/winit) :
  - Debian/Ubuntu : `libxkbcommon-dev`, `libwayland-dev`, `libxcb1-dev`,
    `libgl1-mesa-dev` (ou équivalents), plus un serveur graphique (X11/Wayland).
  - L'IHM nécessite un **affichage** : en environnement headless, la fenêtre ne
    s'ouvre pas (le serveur NAMUR, lui, ne dépend pas de l'affichage).
- **Liaison série** (feature `serial`) : accès au port (`/dev/ttyUSB*`, groupe
  `dialout` sous Linux). Sans matériel, utiliser le transport **TCP**.
- Accès réseau au registre crates.io pour la première compilation.

---

## 2. Commandes courantes

```bash
cargo check -p mock_bin_su_namur          # Vérification rapide (sans codegen)
cargo build -p mock_bin_su_namur          # Compilation debug
cargo build --release -p mock_bin_su_namur   # Compilation optimisée (LTO thin)
cargo test  -p mock_bin_su_namur          # Tests unitaires + intégration
cargo clippy --workspace --all-targets    # Lint (doit rester SANS avertissement)
cargo run   -p mock_bin_su_namur          # Lance l'agitateur (IHM + NAMUR/TCP)

# Fichier de configuration alternatif :
MOCK_CONFIG=./ma_config.toml cargo run -p mock_bin_su_namur
# Journalisation détaillée :
RUST_LOG=debug cargo run -p mock_bin_su_namur
```

Binaire produit : `target/debug/osne` ou `target/release/osne` (le paquet Cargo
reste `mock_bin_su_namur`, mais l'exécutable s'appelle **`osne`** — voir `[[bin]]`
dans le `Cargo.toml` du crate).

### Features Cargo

| Feature | Par défaut | Effet |
|---------|:---------:|-------|
| `gui` | ✅ | IHM `egui`/`eframe` (sinon binaire headless) |
| `serial` | ✅ | Transport NAMUR sur liaison série RS-232 via `tokio-serial` |

```bash
cargo build -p mock_bin_su_namur --no-default-features                  # headless, NAMUR/TCP seul
cargo build -p mock_bin_su_namur --no-default-features --features serial # headless TCP + série
cargo build -p mock_bin_su_namur --no-default-features --features gui    # IHM, TCP seul (sans série)
```

> ⚠️ **`serial` = dépendance native.** `tokio-serial` ouvre le port via termios
> (Linux) ; l'énumération `libudev` est désactivée (`default-features = false`).
> En **cross-compilation** (`build-prod.sh`, exes desktop avec features par
> défaut), l'image `cross` du target peut tout de même réclamer les en-têtes série
> ; si la chaîne pose problème, retirer `serial` du build concerné. Le **Docker
> headless n'est pas impacté** (il build en `--no-default-features`).

---

## 3. Organisation du code

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

**Règle d'or** : la logique métier (`mock_lib_control`, `motor.rs`, `stirrer.rs`)
reste **synchrone et testée** ; l'asynchrone est cantonné aux acteurs et à l'IO.
Calque exact du régulateur **ORME** (`mock_bin_ru_modbustcp`) — mêmes invariants.

---

## 4. Configuration

- Fichier : `mock_su_namur.toml` dans le répertoire courant, ou chemin fourni par
  la variable d'environnement `MOCK_CONFIG`.
- Chargé au démarrage ; **valeurs par défaut** si absent ou illisible (un
  avertissement est journalisé, l'application démarre quand même).
- **Toute valeur issue du TOML est assainie** (`AppConfig::sanitized`) : bornes
  réordonnées (`min ≤ max`), flottants forcés finis, inertie/couple/viscosité
  strictement positifs. **Invariant : ne jamais `f32::clamp` avec des bornes non
  validées** (panique si `min > max` ou `NaN`).
- Sauvegardé depuis l'IHM (boutons *Appliquer* / *Sauvegarder* / *Réinitialiser*).

Structure (toutes les sections sont optionnelles, complétées par défaut) :

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

> Les **valeurs par défaut** ont une **source unique** : `StirrerConfig::default`
> dans `stirrer.rs`. `MotorConfig`/`RegulationConfig` (config.rs) en dérivent. Les
> bornes de sortie du PID (`out_min`/`out_max`) sont **forcées** à `[0, couple_max]`
> au moment de construire l'agitateur (`to_stirrer_config`).

---

## 5. Dépendances et pièges de version

| Crate | Rôle | Point d'attention |
|-------|------|-------------------|
| `tokio` | runtime async | features partagées + **`io-util`** (BufReader / lignes ASCII NAMUR) |
| `ractor` | acteurs | features par défaut (async natif, **pas** `async-trait`) |
| `tokio-serial` | NAMUR série | optionnel (feature `serial`), `default-features = false` (pas d'énumération libudev) |
| `eframe`/`egui` | IHM | versions liées entre elles |
| `egui_plot` | courbe | ⚠️ **versionné une mineure en avance sur `egui`** : pour `egui` 0.33 → `egui_plot` **0.34** |
| `serde`/`toml` | persistance | `mock_lib_control` expose une feature `serde` activée par le binaire |

Les versions partagées sont centralisées dans `[workspace.dependencies]` du
`Cargo.toml` racine. Pour monter `egui`/`eframe`, **vérifier la version
correspondante d'`egui_plot`** (sinon erreur « two versions of crate egui »).

---

## 6. Étendre le projet

### 6.1 Ajouter une commande NAMUR

Tout se passe dans **`namur.rs`** (source de vérité du protocole) :

1. Ajouter la branche dans `handle_line` (lecture → `Reply`, écriture/action →
   `Apply(Command)` ou `SetWatchdog`).
2. Si c'est une **action**, ajouter la variante dans `enum Command` (`stirrer.rs`)
   et son traitement dans `Stirrer::apply`.
3. Mettre à jour le doc-commentaire d'en-tête, **[commandes_namur.md](commandes_namur.md)**
   et la table de référence du mini-terminal (`gui.rs`, tableau `rows`).
4. Ajouter un test dans le module `tests` de `namur.rs`.

### 6.2 Ajouter une commande / un réglage IHM

1. Variante dans `enum Command` (`stirrer.rs`) + traitement dans `Stirrer::apply`.
2. Champ dans `StirrerSnapshot` si la valeur doit être observable.
3. Câblage IHM (`gui.rs`) via un `cast` non bloquant.
4. Si persistant : champ dans `AppConfig` (`config.rs`) + assainissement dans
   `sanitized` + report dans `to_stirrer_config`.

### 6.3 Ajouter une chaîne d'interface (i18n)

Toute chaîne IHM **doit** passer par une clé `Msg` (`i18n.rs`) avec ses **8
traductions** (tableau de taille fixe vérifié à la compilation). Les acronymes
NAMUR, suffixes d'unité et noms de commandes restent codés en dur.

### 6.4 Ajouter un nouvel instrument

1. Créer `mock_bin_<nom>/` et l'ajouter aux `members` du `Cargo.toml` racine.
2. Réutiliser `mock_lib_control` ; factoriser tout commun dans une `mock_lib_*`
   (ex. promotion du modèle `motor.rs` s'il sert un second instrument).
3. Suivre le même découpage : modèle synchrone, acteur(s) ractor, couche
   protocole, IHM. Convention de nom : `mock_bin_<type>_<protocole>`.

---

## 7. Stratégie de test

- **Unitaires** (`mock_lib_control`) : PID (proportionnel, bornage, anti-windup).
- **Moteur** (`motor.rs`) : dynamique rotationnelle, convergence régime établi,
  effet de la viscosité sur le couple, saturation/surcharge.
- **Domaine** (`stirrer.rs`) : convergence de la vitesse vers la consigne,
  décélération à l'arrêt, détection de surcharge.
- **Protocole** (`namur.rs`) : décodage des lectures (`IN_*`), des écritures
  (`OUT_SP_4`), des actions (`START/STOP/RESET`), du chien de garde et des
  commandes inconnues.
- **Config / réseau** (`config.rs`, `actors/network.rs`) : round-trip TOML, filtre
  IP (jokers, IPv4-mapped), assainissement sans panic, ouverture série en erreur
  sur port absent.

Lancer : `cargo test -p mock_bin_su_namur` (ou `--workspace`). Les tests sont
**déterministes et sans IHM**.

---

## 8. Dépannage

| Symptôme | Piste |
|----------|-------|
| « two versions of crate `egui` » | Désaccord `egui_plot` / `egui` : aligner les versions (§5). |
| L'IHM ne s'ouvre pas | Affichage absent (headless) ou libs système manquantes (§1). |
| `NAMUR ✖` dans l'en-tête | Port TCP déjà utilisé / < 1024 sans privilèges, ou port série indisponible : changer dans *Paramètres*. |
| Un client TCP est refusé | IP hors **liste blanche** : vider la liste ou ajouter un motif (`192.168.1.*`). |
| La série ne s'ouvre pas | Feature `serial` absente, mauvais port, ou permissions (`dialout`). |
| Le moteur s'arrête seul | **Chien de garde** armé (`OUT_WD1@…`) sans trafic : envoyer des trames ou `OUT_WD1@0`. |
| Surcharge permanente | Viscosité trop élevée vs `torque_max` : ajuster les paramètres moteur. |
| Config non rechargée | Mauvais répertoire courant ou `MOCK_CONFIG` ; vérifier le journal au démarrage. |

Augmenter la verbosité : `RUST_LOG=debug` (ou `trace`).

---

## 9. Build de distribution

```bash
cargo build --release -p mock_bin_su_namur
# Binaire autonome :
target/release/osne
```

Le profil `release` active `lto = "thin"` et `opt-level = 3` (voir `Cargo.toml`
racine). Pour distribuer : fournir le binaire + un `mock_su_namur.toml` d'exemple.
Licence **MIT** (fichier `LICENSE`).

### Feature `gui` (build avec / sans interface)

```bash
cargo build --release -p mock_bin_su_namur                       # avec IHM (poste de travail)
cargo build --release -p mock_bin_su_namur --no-default-features  # « headless » : NAMUR + simulation, sans IHM
```

Le mode **headless** est destiné aux déploiements sans écran et rend la
**cross-compilation ARM triviale** (aucune dépendance graphique à lier).

### Intégration au bureau Linux (icône de la barre des tâches)

L'icône OSNE (`pic/osne-icon.png`, motif agitateur, générée par
[`pic/osne-logo.gen.py`](../../../pic/osne-logo.gen.py)) est **embarquée** dans le
binaire (`branding.rs` → `window_icon`). Cela suffit sous **X11, Windows et
macOS**. Sous **Wayland**, le compositeur **ignore** l'icône embarquée : il associe
la fenêtre à son **`app_id`** (« osne », défini dans `main.rs` via `with_app_id`) à
un fichier `osne.desktop` du même nom, et affiche l'`Icon=osne` résolue dans le
thème d'icônes `hicolor`.

Pour obtenir l'icône sous Wayland, installer l'entrée de bureau pour l'utilisateur
courant :

```bash
scripts/install-desktop.sh osne
```

Le script copie :

| Source | Destination |
|--------|-------------|
| `pic/osne-icon.png` | `~/.local/share/icons/hicolor/256x256/apps/osne.png` |
| `packaging/osne.desktop` | `~/.local/share/applications/osne.desktop` |

puis rafraîchit les caches. Trois noms **doivent rester alignés** : l'`app_id`
(`main.rs`), le fichier `osne.desktop` (+ son `StartupWMClass`) et l'icône
`osne.png` (= `Icon=osne`). Le même script installe ORME sans argument
(`scripts/install-desktop.sh`).

---

## 10. Build « prod » — cross-compilation depuis Linux

### Procédure unique

Tout est produit **depuis Linux** par
[`scripts/build-prod.sh`](../../../scripts/build-prod.sh), qui construit **tous les
instruments du workspace** (ORME *et* OSNE) :

| Sortie | Cible | IHM | Méthode |
|--------|-------|-----|---------|
| `dist/osne-linux-x86_64` | `x86_64-unknown-linux-gnu` | ✅ | `cross` |
| `dist/osne-windows-x86_64.exe` | `x86_64-pc-windows-gnu` | ✅ | `cross` (mingw) |
| `dist/osne-rpi-arm64` | `aarch64-unknown-linux-gnu` (Pi 3/4/5, Pi OS 64b) | ✅ | `cross` |
| Image Docker headless `osne:headless` | multi-arch `amd64` + `arm64` | ❌ | `docker buildx` |
| `dist/osne_<ver>_amd64.deb` / `_arm64.deb` | paquet Debian/Ubuntu | ✅ | `dpkg-deb` |
| `dist/osne-setup-x86_64.exe` | installeur Windows | ✅ | NSIS (`makensis`) |

```bash
# Prérequis (une fois) — Docker doit tourner :
cargo install cross

# Tout produire (exes ORME + OSNE + installeurs dans dist/ + images Docker amd64) :
scripts/build-prod.sh

# Variante : images Docker MULTI-ARCH poussées vers un registre :
IMAGE_PREFIX=ghcr.io/<compte> scripts/build-prod.sh

# Sans construire les installeurs :
INSTALLERS=0 scripts/build-prod.sh
```

### Pourquoi `cross` pour TOUS les builds (y compris Linux x86_64)

`cross` fournit des images Docker contenant les toolchains de chaque cible.
⚠️ **Ne pas mélanger `cargo` natif et `cross` dans le même `target/`.** Les
**proc-macros** compilées par l'un sont rejetées par l'autre (`can't find crate
for …_derive`). Le script passe **toujours par `cross`**. (Si l'erreur survient :
`rm -rf target/release` puis relancer.)

### IHM cross-compilée vers ARM : pourquoi ça marche

`eframe`/`egui` chargent OpenGL, X11/Wayland et xkbcommon **à l'exécution**
(`dlopen`) : le binaire ne lie au build que la `libc`. Aucune lib graphique ARM
n'est nécessaire côté cross ; prévoir un environnement de bureau sur la cible.

### Image Docker headless

L'image ([`docker/Dockerfile.headless`](../../../docker/Dockerfile.headless)) part
de `debian:bookworm-slim` et **copie** le binaire headless de l'architecture
voulue (aucune compilation dans l'image → pas de QEMU). Le nom du binaire et le
port exposé sont passés par `--build-arg` (`BIN=osne`, `PORT=4001`). Monter un
volume sur `/data` pour fournir/persister `mock_su_namur.toml`.

```bash
docker run --rm -p 4001:4001 -v "$PWD/conf:/data" osne:headless
```

### Installeurs (`.deb` Linux/RPi + setup Windows)

À la fin de chaque build, `build-prod.sh` appelle
[`scripts/make-installers.sh <bin>`](../../../scripts/make-installers.sh), qui
transforme les exécutables release de `dist/` en **installeurs** :

| Installeur | Source | Contenu | Outil |
|------------|--------|---------|-------|
| `osne_<ver>_amd64.deb` | `dist/osne-linux-x86_64` | binaire → `/usr/bin`, entrée de bureau, icône hicolor | `dpkg-deb` |
| `osne_<ver>_arm64.deb` | `dist/osne-rpi-arm64` | idem (Raspberry Pi OS 64 bits) | `dpkg-deb` |
| `osne-setup-x86_64.exe` | `dist/osne-windows-x86_64.exe` | exe + raccourcis (menu Démarrer/bureau) + désinstalleur | NSIS (`makensis`) |

- Les `.deb` posent l'icône et le `.desktop` ; un `postinst` rafraîchit les caches
  (`update-desktop-database`, `gtk-update-icon-cache`). Dépendances : `libc6` ;
  recommandations graphiques (`libgl1`, `libxkbcommon0`, `libwayland-client0`).
- L'installeur Windows est généré à partir de
  [`packaging/windows/installer.nsi`](../../../packaging/windows/installer.nsi) ;
  les raccourcis utilisent une icône `.ico` multi-résolution dérivée de
  `pic/osne-icon.png` (via Pillow).
- **Prérequis** : `dpkg-deb` (présent sur Debian/Ubuntu) pour les `.deb`,
  **`makensis`** (`sudo apt install nsis`) pour le setup Windows, `python3`+Pillow
  pour le `.ico`. Chaque cible dont l'outil ou l'artefact manque est **avertie et
  sautée** (le build ne casse pas). Désactiver via `INSTALLERS=0`. On peut aussi
  (re)générer seuls les installeurs d'un instrument :
  `scripts/make-installers.sh osne`.
- La **version** des paquets vient de `[workspace.package].version` du `Cargo.toml`
  racine.

### Notes

- Les binaires sont **dynamiquement liés à la glibc** ; compilés via `cross`
  (baseline glibc ancienne) ils tournent sur des distributions récentes.
- `dist/` est ignoré par git (artefacts de build).

---

## 11. Conventions

- Code et commentaires en **français** ; logs et messages d'erreur en **anglais**.
- `cargo clippy --workspace` **sans avertissement** avant tout commit.
- Tout nouveau comportement métier, de moteur ou de protocole s'accompagne d'un
  **test**.
- Le jeu de commandes NAMUR se modifie dans **`namur.rs`** (source de vérité), avec
  mise à jour conjointe de la documentation.
