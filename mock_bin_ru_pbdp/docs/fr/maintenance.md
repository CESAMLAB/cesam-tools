# Documentation de maintenance — ORPD / PROFIBUS DP (workspace `cesam-tools`)

*🌍 **FR** · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

> Crate : `mock_bin_ru_pbdp` · Exécutable : **ru_pbdp** · Marque : **ORPD**
> Public : développeurs qui maintiennent, corrigent ou étendent le projet.
> Voir aussi : [conception.md](conception.md) · [reference_profibus.md](reference_profibus.md).

---

## 1. Prérequis

- **Rust stable** (édition 2021, `rust-version` ≥ 1.85). Installation : <https://rustup.rs>.
- **Dépendances système (Linux) pour l'IHM** (`eframe`/`egui`, OpenGL/winit) :
  `libxkbcommon-dev`, `libwayland-dev`, `libxcb1-dev`, `libgl1-mesa-dev` (ou
  équivalents), plus un serveur graphique (X11/Wayland). L'IHM nécessite un
  **affichage** : en environnement headless, la fenêtre ne s'ouvre pas.
- **Liaison série** (accès au port, `/dev/ttyUSB*`, groupe `dialout` sous
  Linux) : contrairement à ORME/OSNE, **ce n'est pas une feature optionnelle**
  ici — `tokio-serial` est une dépendance directe (voir §5), la liaison série
  étant l'unique transport de cet instrument (pas d'équivalent standard
  « PROFIBUS sur TCP »). Sans port matériel, l'IHM démarre quand même (l'erreur
  d'ouverture s'affiche dans l'en-tête, la simulation continue) — voir
  [`manuel_utilisateur.md`](manuel_utilisateur.md) §2.
- Accès réseau au registre crates.io pour la première compilation.

---

## 2. Commandes courantes

```bash
cargo check -p mock_bin_ru_pbdp          # Vérification rapide (sans codegen)
cargo build -p mock_bin_ru_pbdp          # Compilation debug
cargo build --release -p mock_bin_ru_pbdp   # Compilation optimisée (LTO thin)
cargo test  -p mock_bin_ru_pbdp          # Tests unitaires + intégration
cargo clippy --workspace --all-targets    # Lint (doit rester SANS avertissement)
cargo run   -p mock_bin_ru_pbdp          # Lance l'IHM + la liaison série PROFIBUS DP

# Fichier de configuration alternatif :
MOCK_CONFIG=./ma_config.toml cargo run -p mock_bin_ru_pbdp
# Journalisation détaillée :
RUST_LOG=debug cargo run -p mock_bin_ru_pbdp
```

Binaire produit : `target/debug/ru_pbdp` ou `target/release/ru_pbdp` (le
paquet Cargo reste `mock_bin_ru_pbdp` ; l'exécutable et le nom commercial
« ORPD » sont documentaires, voir `[[bin]]` du `Cargo.toml` du crate).

### Features Cargo

| Feature | Par défaut | Effet |
|---------|:---------:|-------|
| `gui` | ✅ | IHM `egui`/`eframe` + vérification de mise à jour (sinon binaire headless) |

```bash
cargo build -p mock_bin_ru_pbdp --no-default-features   # headless : liaison série + simulation, sans IHM
```

> ⚠️ **Différence avec ORME/OSNE** : chez ces deux instruments, la liaison
> série (RTU/serial) est elle-même une **feature optionnelle** à côté d'un
> transport TCP toujours présent, et `--no-default-features` peut l'exclure.
> Ici, **il n'existe pas de variante « sans série »** : `tokio-serial` est une
> dépendance directe (non feature-gated), présente dans **tous** les builds y
> compris headless — c'est le seul transport de l'instrument.

---

## 3. Organisation du code

```
mock_lib_control/        Bibliothèque de régulation (pure, sans IO, testable)
  src/pid.rs             PID anti-emballement
  src/lib.rs             ré-exports (feature `serde` optionnelle)

mock_bin_ru_pbdp/        Binaire régulateur PROFIBUS DP (exécutable `ru_pbdp`)
  src/main.rs            Démarrage : config, runtime Tokio, acteurs, IHM/headless
  src/regulator.rs        Modèle métier synchrone (PID + procédé 1er ordre), Command, pas
  src/config.rs           AppConfig (TOML), SerialConfig, ProcessConfig, RegulationConfig, ServerStatus
  src/profibus.rs         Protocole PROFIBUS DP-V0 : codec de trames + FCS + SlaveFsm (SOURCE DE VÉRITÉ)
  src/profibus_server.rs  Boucle de session série (lecture trame → SlaveFsm → réponse) + chien de garde
  src/map.rs              Disposition des blocs I/O Output/Input <-> Command du régulateur
  src/trace.rs            Journal circulaire des trames (mini-terminal IHM)
  src/gui.rs              IHM egui (page unique + mini-terminal + modal Paramètres)
  src/branding.rs         Logos embarqués (feature `gui`)
  src/i18n.rs             Catalogue i18n typé (8 langues), sans dépendance
  src/actors/
    simulation.rs         Boucle de régulation (pas de simulation 50 ms)
    network.rs            Acteur liaison série PROFIBUS DP (re)configurable à chaud

docs/                     Conception, référence PROFIBUS, manuel, maintenance (multilingue)
```

**Règle d'or** : la logique métier (`mock_lib_control`, `regulator.rs`,
`profibus.rs`, `map.rs`) reste **synchrone et testée** ; l'asynchrone est
cantonné aux acteurs et à l'IO série. Modèle de régulation calqué sur **ORME**
(`mock_bin_ru_modbus`) — mêmes invariants.

---

## 4. Configuration

- Fichier : `mock_ru_pbdp.toml` dans le répertoire courant, ou chemin fourni
  par la variable d'environnement `MOCK_CONFIG`.
- Chargé au démarrage ; **valeurs par défaut** si absent ou illisible (un
  avertissement est journalisé, l'application démarre quand même).
- **Toute valeur issue du TOML est assainie** (`AppConfig::sanitized`) : bornes
  de consigne/PID réordonnées, flottants forcés finis, `τ ≥ 1e-3`,
  `dead_time` borné, **adresse de station bornée `[0, 125]`**. **Invariant :
  ne jamais `f32::clamp` avec des bornes non validées** (panique si `min > max`
  ou `NaN`).
- Sauvegardé depuis l'IHM (boutons *Appliquer* / *Sauvegarder* / *Réinitialiser*).

Structure (toutes les sections sont optionnelles, complétées par défaut) :

```toml
language = "fr"
check_updates = true       # vérifier au démarrage si une release plus récente existe (IHM)

[network.serial]
port = "/dev/ttyUSB0"      # "COM3" par défaut sous Windows
baud = 500000              # valeur normalisée PROFIBUS DP (9600 .. 12000000)
station_address = 3        # adresse de l'esclave simulé (0-125)
watchdog_enabled = true    # autorise le chien de garde annoncé par le maître (Set_Prm)

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

> Le **format de trame série (8E1)** est fixé par la norme PROFIBUS DP et n'est
> **pas** un champ de configuration — voir `SerialConfig::open` dans
> [`config.rs`](../../src/config.rs). Contrairement à ORME/OSNE, **pas de liste
> blanche d'IP** (liaison série intrinsèquement point-à-point).

### Vérification de mise à jour

Si `check_updates = true` (défaut) **et** que le binaire est compilé avec la
feature `gui`, l'IHM interroge **au démarrage** la dernière release publiée sur
GitHub (`CESAMLAB/cesam-tools`) via la crate partagée **`mock_lib_update`**
(`ureq`/`rustls`, racines embarquées, thread borné par timeout). **Absente des
builds headless** (`--no-default-features`).

---

## 5. Dépendances et pièges de version

| Crate | Rôle | Point d'attention |
|-------|------|-------------------|
| `tokio` | runtime async | features partagées + `io-util` |
| `ractor` | acteurs | features par défaut |
| `tokio-serial` | liaison PROFIBUS DP | **dépendance directe, non feature-gated** (voir §2) ; `default-features = false` (pas d'énumération `libudev`) |
| `eframe`/`egui` | IHM | versions liées entre elles, feature `gui` |
| `egui_plot` | courbe | ⚠️ **versionné une mineure en avance sur `egui`** : pour `egui` 0.33 → `egui_plot` **0.34** |
| `serde`/`toml` | persistance | `mock_lib_control` expose une feature `serde` activée par le binaire |
| `mock_lib_update` (`ureq`/`rustls`) | vérif. de MAJ | feature `gui` uniquement ; absent en headless |

Les versions partagées sont centralisées dans `[workspace.dependencies]` du
`Cargo.toml` racine. Pour monter `egui`/`eframe`, **vérifier la version
correspondante d'`egui_plot`** (sinon erreur « two versions of crate egui »).

---

## 6. Étendre le projet

### 6.1 Ajouter un service PROFIBUS (SAP)

Tout se passe dans **[`profibus.rs`](../../src/profibus.rs)** (source de
vérité du protocole) :

1. Ajouter la constante `SAP_*` et la variante correspondante dans `enum
   Request` ; brancher le décodage dans `decode_request` (et, pour les tests,
   dans `encode_request`).
2. Traiter la nouvelle requête dans `SlaveFsm::handle` (transition d'état si
   pertinent, `Handled` renvoyé).
3. Mettre à jour le doc-commentaire d'en-tête et
   **[reference_profibus.md](reference_profibus.md)**.
4. Ajouter un test dans le module `tests` de `profibus.rs` (et, si la session
   complète est concernée, dans `profibus_server.rs`).

### 6.2 Modifier les blocs I/O (`Output`/`Input`)

1. Ajuster la disposition dans **[`map.rs`](../../src/map.rs)**
   (`decode_output`/`encode_input`), en conservant `OUTPUT_LEN`/`INPUT_LEN`
   cohérents avec `SlaveProfile` (`profibus_server.rs`).
2. Mettre à jour la table de **[reference_profibus.md](reference_profibus.md)**
   §3 (source de vérité documentaire, recopiée depuis le doc-commentaire de
   `map.rs`).
3. Ajouter un test de round-trip dans `map.rs`.

### 6.3 Ajouter une commande / un réglage IHM

1. Variante dans `enum Command` (`regulator.rs`) + traitement dans
   `Regulator::apply` (avec assainissement).
2. Champ dans `RegulatorSnapshot` si la valeur doit être observable.
3. Câblage IHM (`gui.rs`) via un `cast` non bloquant.
4. Si persistant : champ dans `AppConfig` (`config.rs`) + assainissement dans
   `sanitized` + report dans `to_regulator_config`.

### 6.4 Ajouter une chaîne d'interface (i18n)

Toute chaîne IHM **doit** passer par une clé `Msg` (`i18n.rs`) avec ses **8
traductions** (tableau de taille fixe vérifié à la compilation). Les
identifiants de service PROFIBUS et suffixes d'unité restent codés en dur.

### 6.5 Ajouter un nouvel instrument

1. Créer `mock_bin_<nom>/` et l'ajouter aux `members` du `Cargo.toml` racine.
2. Réutiliser `mock_lib_control` ; factoriser tout commun dans une `mock_lib_*`.
3. Suivre le même découpage : modèle synchrone, acteur(s) ractor, couche
   protocole, IHM. Convention de nom : `mock_bin_<type>_<protocole>`.

---

## 7. Stratégie de test

- **Codec de trames** (`profibus.rs`) : round-trip `SD1`/`SD2`/`SD3`/`SD4`,
  rejet de checksum et de longueur incorrects, encodage/décodage des requêtes
  (`Slave_Diag`/`Set_Prm`/`Chk_Cfg`/`Data_Exchange`) et de l'octet de mode.
- **Machine à états** (`profibus.rs`) : séquence complète
  `Power_On → Wait_Prm → Wait_Cfg → Data_Exchange`, refus d'un `Set_Prm` avec
  identifiant erroné (reste en `Wait_Prm`).
- **Blocs I/O** (`map.rs`) : bloc de sortie trop court → aucune commande ;
  round-trip consigne/mode ; le bloc d'entrée reflète l'instantané (bits de
  statut, mesure).
- **Config** (`config.rs`) : round-trip TOML, assainissement (bornes
  inversées, valeurs non finies, adresse de station hors plage) sans panic,
  erreur propre à l'ouverture d'un port série absent.
- **Session réseau** (`profibus_server.rs`, `#[tokio::test]` sur
  `tokio::io::duplex`) : handshake complet jusqu'à `Data_Exchange` avec
  application effective des commandes, trame adressée à une autre station
  ignorée (aucune activité marquée), déclenchement du chien de garde forçant
  l'état sûr.

Lancer : `cargo test -p mock_bin_ru_pbdp` (ou `--workspace`) — **36 tests**,
tous **déterministes et sans IHM**, aucun test lent/`#[ignore]` (contrairement
à RU/OPC UA dont la génération RSA justifie des tests ignorés).

---

## 8. Dépannage

| Symptôme | Piste |
|----------|-------|
| « two versions of crate `egui` » | Désaccord `egui_plot` / `egui` : aligner les versions (§5). |
| L'IHM ne s'ouvre pas | Affichage absent (headless) ou libs système manquantes (§1). |
| Erreur d'ouverture du port série (en-tête IHM) | Port absent, mauvais chemin, ou permissions (`dialout` sous Linux) — la simulation continue sans liaison. |
| La liaison reste en `Wait_Prm` | Le maître n'envoie pas `Set_Prm` avec l'identifiant attendu (`0xEE01`) — voir [reference_profibus.md](reference_profibus.md) §2. |
| La liaison reste en `Wait_Cfg` | `Chk_Cfg` reçu n'annonce pas `out_len=45`/`in_len=17`. |
| L'appareil s'arrête tout seul | Chien de garde protocolaire déclenché (silence prolongé du maître) — état sûr attendu, pas un bug. |
| Pas de chien de garde alors que le maître en demande un | `watchdog_enabled = false` en configuration locale : la demande du maître est ignorée par choix. |

Augmenter la verbosité : `RUST_LOG=debug` (ou `trace`).

---

## 9. Build de distribution

```bash
cargo build --release -p mock_bin_ru_pbdp
# Binaire autonome :
target/release/ru_pbdp
```

Le profil `release` active `lto = "thin"` et `opt-level = 3` (voir `Cargo.toml`
racine). Pour distribuer : fournir le binaire + un `mock_ru_pbdp.toml`
d'exemple. Licence **MIT** (fichier `LICENSE`).

### Feature `gui` (build avec / sans interface)

```bash
cargo build --release -p mock_bin_ru_pbdp                       # avec IHM (poste de travail)
cargo build --release -p mock_bin_ru_pbdp --no-default-features  # « headless » : liaison série + simulation, sans IHM
```

Contrairement à OSNE, le mode **headless** ne rend pas la liaison série
optionnelle (§2) : il retire uniquement l'IHM. Il reste pertinent pour un
déploiement sans écran relié à un vrai port série/USB.

### Intégration au bureau Linux (icône de la barre des tâches)

L'icône ORPD (`pic/ru_pbdp-icon.png`, générée par
[`pic/ru_pbdp-logo.gen.py`](../../../pic/ru_pbdp-logo.gen.py)) est **embarquée**
dans le binaire (`branding.rs` → `window_icon`). Cela suffit sous **X11,
Windows et macOS**. Sous **Wayland**, le compositeur **ignore** l'icône
embarquée : il associe la fenêtre à son **`app_id`** (« ru_pbdp », défini dans
`main.rs` via `with_app_id`) à un fichier `ru_pbdp.desktop` du même nom, et
affiche l'`Icon=ru_pbdp` résolue dans le thème d'icônes `hicolor`.

Pour obtenir l'icône sous Wayland, installer l'entrée de bureau pour
l'utilisateur courant :

```bash
scripts/install-desktop.sh ru_pbdp
```

Le script copie :

| Source | Destination |
|--------|-------------|
| `pic/ru_pbdp-icon.png` | `~/.local/share/icons/hicolor/256x256/apps/ru_pbdp.png` |
| `packaging/ru_pbdp.desktop` | `~/.local/share/applications/ru_pbdp.desktop` |

puis rafraîchit les caches. Trois noms **doivent rester alignés** : l'`app_id`
(`main.rs`), le fichier `ru_pbdp.desktop` (+ son `StartupWMClass`) et l'icône
`ru_pbdp.png` (= `Icon=ru_pbdp`).

---

## 10. Build « prod » — cross-compilation depuis Linux

Tout est produit **depuis Linux** par
[`scripts/build-prod.sh`](../../../scripts/build-prod.sh), qui construit
**tous les instruments du workspace** (tableau `INSTRUMENTS`, entrée
`mock_bin_ru_pbdp:ru_pbdp:0` — port `0` : liaison série, aucun port IP) :

| Sortie | Cible | IHM | Méthode |
|--------|-------|-----|---------|
| `dist/ru_pbdp-linux-x86_64` | `x86_64-unknown-linux-gnu` | ✅ | `cross` |
| `dist/ru_pbdp-windows-x86_64.exe` | `x86_64-pc-windows-gnu` | ✅ | `cross` (mingw) |
| `dist/ru_pbdp-rpi-arm64` | `aarch64-unknown-linux-gnu` (Pi 3/4/5, Pi OS 64b) | ✅ | `cross` |
| Image Docker headless `ru_pbdp:headless` | multi-arch `amd64` + `arm64` | ❌ | `docker buildx` |
| `dist/ru_pbdp_<ver>_amd64.deb` / `_arm64.deb` | paquet Debian/Ubuntu | ✅ | `dpkg-deb` |
| `dist/ru_pbdp-setup-x86_64.exe` | installeur Windows | ✅ | NSIS (`makensis`) |

```bash
cargo install cross          # prérequis (une fois) — Docker doit tourner
scripts/build-prod.sh        # tous les instruments, dont ru_pbdp
ONLY=ru_pbdp scripts/build-prod.sh   # ce seul instrument
```

⚠️ **Ne pas mélanger `cargo` natif et `cross`** dans le même `target/` (proc-macros
incompatibles → `can't find crate for …_derive`). Le script passe toujours par
`cross`.

### Image Docker headless : utilité limitée sans passthrough série

L'image ([`docker/Dockerfile.headless`](../../../docker/Dockerfile.headless))
est construite comme pour les autres instruments (`EXPOSE 0`, métadonnée
inerte), mais **n'a d'intérêt réel qu'avec un périphérique série monté** dans
le conteneur :

```bash
docker run --rm --device=/dev/ttyUSB0 -v "$PWD/conf:/data" ru_pbdp:headless
```

Sans `--device`, le conteneur démarre mais ne peut ouvrir aucun port série
(comportement identique à l'absence de matériel en local — voir §8).

---

## 11. Conventions

- Code et commentaires en **français** ; logs et messages d'erreur en **anglais**.
- `cargo clippy --workspace` **sans avertissement** avant tout commit.
- Tout nouveau comportement métier ou de protocole s'accompagne d'un **test**.
- Le protocole PROFIBUS DP-V0 se modifie dans **`profibus.rs`** (source de
  vérité), avec mise à jour conjointe de
  **[reference_profibus.md](reference_profibus.md)**.
