<p align="center">
  <img src="pic/Logo-CESAM-Couleur-vect-card.png" alt="CESAM-Lab" height="84">
</p>

# cesam-tools — Boîte à outils CESAM-Lab

*🌍 [English](README.md) · **Français** · [Deutsch](README.de.md) · [Español](README.es.md) · [Italiano](README.it.md) · [Português](README.pt.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md)*

<p align="center">
  <a href="https://github.com/CESAMLAB/cesam-tools/releases/latest"><img src="https://img.shields.io/github/v/release/CESAMLAB/cesam-tools?label=release" alt="Latest release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue" alt="License: MIT"></a>
</p>

Workspace Rust regroupant les **outils de CESAM-Lab**, à commencer par des
**simulateurs d'instruments industriels** : des appareils virtuels qui
reproduisent un comportement physique réaliste et communiquent via des protocoles
de terrain. Utile pour développer, tester et démontrer des superviseurs, automates
ou passerelles **sans matériel réel**.

> Distribué gratuitement sous licence [MIT](LICENSE).

## Instruments disponibles

| Crate | Produit | Description | Protocole | IHM |
|-------|---------|-------------|-----------|-----|
| [`mock_bin_ru_modbustcp`](mock_bin_ru_modbustcp) | **ORME** | Régulateur (PID / TOR / PWM) sur fonction de transfert | Modbus TCP & RTU (esclave) | egui |
| [`mock_bin_su_namur`](mock_bin_su_namur) | **OSNE** | Agitateur de laboratoire à hélice : fonction de transfert du moteur, asservissement de vitesse rapide, charge visqueuse ajustable | NAMUR sur TCP & série RS-232 (esclave) | egui |
| [`mock_bin_ru_opcua`](mock_bin_ru_opcua) | **ORUE** | Régulateur de procédé (PID anti-emballement) sur procédé du premier ordre, avec sécurité OPC UA configurable | OPC UA (serveur) | egui |

Bibliothèque partagée :

| Crate | Description |
|-------|-------------|
| [`mock_lib_control`](mock_lib_control) | Briques de régulation réutilisables : PID anti-emballement, tout-ou-rien à hystérésis, procédé du 1ᵉʳ ordre + retard pur (FOPDT). |

## ORME — le régulateur simulé

<p align="center">
  <img src="pic/orme-logo.svg" alt="ORME — Open Regulator Modbus Emulator" height="120">
</p>

> **ORME** — *Open Regulator Modbus Emulator*. **« Ouvrez le bus. »**
> Un régulateur de terrain qui n'existe que sur votre bus Modbus.

Un régulateur industriel virtuel complet :

- **Procédé** modélisé par une fonction de transfert du premier ordre avec
  retard pur `K·e^(-Ls) / (1 + T·s)` (typique d'un four ou bain thermostaté).
- **Régulation** bidirectionnelle : sens 1 (chaud) et sens 2 (froid),
  chacun configurable en **PID**, **tout-ou-rien (TOR)** ou **relais à cycle (PWM)**.
- **Modes** marche/arrêt et automatique/manuel.
- **Serveur Modbus** en **TCP** ou **RTU série / RS485** (feature `rtu`), au choix.
  Table d'adresses (consigne, mesure, sortie, modes…), **liste blanche d'IP**
  (jokers `*`) configurable à chaud, et **politique mono-maître** (un seul maître
  distant à la fois ; en TCP un nouveau venu déconnecte le précédent).
- **Interface graphique** sur une page : pilotage, **courbe de tendance**
  temps réel, **table d'adresses Modbus live**, et un **modal Paramètres**
  (transport TCP/RTU, port, IP autorisées, paramètres série, fonction de
  transfert, bornes de consigne).
- **Configuration persistée** au format TOML (`mock_ru_modbustcp.toml`),
  rechargée au démarrage, avec bouton de réinitialisation aux valeurs par défaut.

### Architecture asynchrone

```
        Command (cast non bloquant)            instantané partagé
  IHM (egui) ──────────────────────►  SimulationActor  ──────────►  IHM (lecture)
  Modbus écriture ─────────────────►   (ractor)         ──────────►  image Modbus
  Modbus lecture  ◄──────────────────────────────────────  image Modbus
```

- **`ractor`** : un acteur unique possède l'état du régulateur ; toutes les
  mutations passent par messages (pas de verrou sur la logique métier).
- **`tokio-modbus`** : serveur Modbus TCP et RTU série (trait `Service`).
- **`eframe`/`egui`** : interface graphique sur le thread principal.

## OSNE — l'agitateur de laboratoire simulé

<p align="center">
  <img src="pic/osne-logo.svg" alt="OSNE — Open Stirrer NAMUR Emulator" height="120">
</p>

> **OSNE** — *Open Stirrer NAMUR Emulator*.
> Un agitateur de laboratoire à hélice (style IKA) qui n'existe que sur votre
> liaison NAMUR.

Un agitateur de laboratoire virtuel complet :

- **Moteur** modélisé par une fonction de transfert rotationnelle `J·dω/dt = T −
  k·η·ω − frottement` (Euler explicite), avec un **PID rapide** pilotant le couple
  pour suivre la consigne de vitesse.
- **Viscosité ajustable** `η` : augmente le couple de charge ; à forte viscosité
  le moteur sature et la consigne devient inatteignable (**surcharge**) — comme un
  vrai agitateur.
- **Serveur NAMUR** (protocole de commandes ASCII) sur **TCP** (test sans
  matériel) ou **série RS-232** (feature `serial`), avec un **chien de garde** par
  session (`OUT_WD1@<m>`), une **politique mono-maître** et une **liste blanche
  d'IP** (TCP).
- **Interface graphique** sur une page : consigne de vitesse, viscosité, **courbe
  de tendance** vitesse/couple live, un **mini-terminal NAMUR** embarqué
  (envoyer/inspecter des trames avec historique des commandes), et un **modal
  Paramètres** (transport TCP/série, paramètres moteur, bornes, i18n 8 langues).
- **Configuration persistée** au format TOML (`mock_su_namur.toml`), rechargée au
  démarrage, avec bouton de réinitialisation aux valeurs par défaut.

Il partage l'architecture d'ORME (modèle métier synchrone, acteurs `ractor`, IHM
`egui`). Lancez-le avec `cargo run -p mock_bin_su_namur` ; le serveur NAMUR écoute
par défaut sur `0.0.0.0:4001`.

## ORUE — le régulateur OPC UA simulé

<p align="center">
  <img src="pic/ru_opcua-logo.svg" alt="ORUE — Open Regulator UA Emulator" height="120">
</p>

> **ORUE** — *Open Regulator UA Emulator*. **« Unifiez le procédé. »**
> Un régulateur de procédé qui n'existe que sur votre espace d'adressage OPC UA.

Un régulateur de procédé virtuel complet :

- **Procédé** modélisé par une fonction de transfert du premier ordre piloté par un
  **PID anti-emballement**, calculé toutes les 0,5 s.
- **Serveur OPC UA** (`async-opcua`, natif Tokio, crypto 100 % Rust — sans OpenSSL,
  pile MPL-2.0). **Sécurité configurable** (`SecurityConfig`) : `None`/anonyme par
  défaut (démarrage instantané) **ou** `Basic256Sha256` / SignAndEncrypt avec un
  certificat auto-signé (`pki/`, généré au premier passage en chiffré), plus des
  jetons anonyme et/ou **utilisateur/mot de passe**.
- **Une posture différente d'ORME/OSNE** : la sécurité OPC UA repose sur
  **certificat + authentification**, pas sur une liste blanche d'IP (il n'y en a
  **aucune**) ; le serveur accepte **plusieurs sessions clientes simultanées** (pas
  de mono-maître, dernier gagnant). Le `None`/anonyme par défaut sur `0.0.0.0:4840`
  est le plus ouvert du workspace — un bandeau IHM avertit dès que le chiffrement
  est désactivé.
- **Interface graphique** sur une page : pilotage, **courbe de tendance** temps
  réel, et un **modal Paramètres** (réseau, fonction de transfert du procédé, gains
  PID, bornes de consigne, sécurité, i18n 8 langues).
- **Configuration persistée** au format TOML (`mock_ru_opcua.toml`), rechargée au
  démarrage, avec bouton de réinitialisation aux valeurs par défaut.

Il partage l'architecture d'ORME (modèle métier synchrone, acteurs `ractor`, IHM
`egui`). Lancez-le avec `cargo run -p mock_bin_ru_opcua` ; le serveur OPC UA écoute
par défaut sur `0.0.0.0:4840`. L'espace d'adressage est documenté dans
[`mock_bin_ru_opcua/docs/fr/reference_opcua.md`](mock_bin_ru_opcua/docs/fr/reference_opcua.md).

## Téléchargement

Des binaires précompilés sont disponibles sur la page [**Releases**](https://github.com/CESAMLAB/cesam-tools/releases/latest) — **aucune chaîne d'outils Rust requise**. Chaque instrument fournit son propre exécutable (`orme`, `osne`, `ru_opcua`).

**ORME** (régulateur Modbus) :

| Plateforme | IHM | Headless (TCP seul, sans IHM) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`orme-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-linux-x86_64) | [`orme-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-linux-x86_64-headless) |
| Windows x86_64 | [`orme-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`orme-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-rpi-arm64) | [`orme-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-rpi-arm64-headless) |

**OSNE** (agitateur de laboratoire NAMUR) :

| Plateforme | IHM | Headless (TCP seul, sans IHM) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`osne-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-linux-x86_64) | [`osne-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-linux-x86_64-headless) |
| Windows x86_64 | [`osne-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`osne-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-rpi-arm64) | [`osne-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-rpi-arm64-headless) |

**ORUE** (régulateur OPC UA) :

| Plateforme | IHM | Headless (TCP seul, sans IHM) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`ru_opcua-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-linux-x86_64) | [`ru_opcua-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-linux-x86_64-headless) |
| Windows x86_64 | [`ru_opcua-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`ru_opcua-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-rpi-arm64) | [`ru_opcua-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-rpi-arm64-headless) |

```bash
chmod +x orme-linux-x86_64        # Linux / Raspberry Pi (idem pour osne-*, ru_opcua-*)
./orme-linux-x86_64
```

Les binaires Linux/RPi sont liés dynamiquement à la glibc et nécessitent un environnement de bureau (X11/Wayland) pour l'IHM. Sous **Wayland**, installez l'entrée de bureau pour l'icône de la barre des tâches : `scripts/install-desktop.sh`. Vérifiez l'intégrité avec les sommes de contrôle publiées :

```bash
sha256sum -c SHA256SUMS
```

## Démarrage rapide

```bash
# Prérequis : Rust stable (édition 2021, >= 1.85).
# Dépendances système Linux pour l'IHM : libxkbcommon, libwayland/xcb, openGL.

cargo run -p mock_bin_ru_modbustcp
```

La fenêtre s'ouvre et le serveur Modbus TCP écoute sur `0.0.0.0:5502`.
Le **port**, l'**IP d'écoute** et la **liste blanche d'IP** se règlent dans le
modal **⚙ Paramètres** (appliqué à chaud) puis sont **persistés** dans
`mock_ru_modbustcp.toml`. La **langue de l'interface** (français, anglais,
allemand, espagnol, italien, portugais, néerlandais, polonais) se choisit dans ce
même modal et est persistée. Pour utiliser un autre fichier de configuration :

```bash
MOCK_CONFIG=/chemin/vers/ma_config.toml cargo run -p mock_bin_ru_modbustcp
```

### Tester la liaison Modbus

Avec n'importe quel client Modbus (ex. `mbpoll`) :

```bash
# Mettre en marche (bobine 0) puis lire la mesure (input registers 0-1, f32)
mbpoll -m tcp -a 1 -t 0 -p 5502 127.0.0.1 1      # écrire la bobine On/Off
mbpoll -m tcp -a 1 -t 3:float -r 1 -p 5502 127.0.0.1   # lire PV (f32)
```

La table d'adresses complète est documentée dans
[`mock_bin_ru_modbustcp/src/map.rs`](mock_bin_ru_modbustcp/src/map.rs).

## Développement

```bash
cargo test --workspace      # tests unitaires + intégration
cargo clippy --workspace    # lint
```

## Documentation

Chaque instrument porte sa propre documentation dans son sous-dossier `docs/`,
disponible en huit langues (`docs/<langue>/`). Versions françaises :

**ORME** (régulateur Modbus) :

- [**Manuel utilisateur**](mock_bin_ru_modbustcp/docs/fr/manuel_utilisateur.md) — prise en main, IHM, paramètres, FAQ.
- [Document de conception](mock_bin_ru_modbustcp/docs/fr/conception.md) — architecture et choix techniques.
- [Table d'adresses Modbus](mock_bin_ru_modbustcp/docs/fr/table_modbus.md) — plan d'adressage complet.
- [Maintenance logicielle](mock_bin_ru_modbustcp/docs/fr/maintenance.md) — build, configuration, extension, dépannage.

**OSNE** (agitateur de laboratoire NAMUR) :

- [**Manuel utilisateur**](mock_bin_su_namur/docs/fr/manuel_utilisateur.md) — prise en main, IHM, mini-terminal NAMUR, paramètres, FAQ.
- [Document de conception](mock_bin_su_namur/docs/fr/conception.md) — modèle moteur, boucle d'asservissement, architecture.
- [Jeu de commandes NAMUR](mock_bin_su_namur/docs/fr/commandes_namur.md) — référence du protocole (canaux, commandes, exemples).
- [Maintenance logicielle](mock_bin_su_namur/docs/fr/maintenance.md) — build, configuration, extension, dépannage.

**ORUE** (régulateur OPC UA) :

- [**Manuel utilisateur**](mock_bin_ru_opcua/docs/fr/manuel_utilisateur.md) — prise en main, IHM, connexion d'un client OPC UA, FAQ.
- [Document de conception](mock_bin_ru_opcua/docs/fr/conception.md) — modèle PID + procédé, architecture acteurs, pile `async-opcua`, sécurité.
- [Référence OPC UA](mock_bin_ru_opcua/docs/fr/reference_opcua.md) — endpoint, namespace, nœuds (lectures/écritures, exemples).
- [Maintenance logicielle](mock_bin_ru_opcua/docs/fr/maintenance.md) — build, configuration, extension, dépannage.

## Marque & logos

Les logos sont dans [`pic/`](pic/) :

- [`orme-icon.svg`](pic/orme-icon.svg) / `orme-icon.png` — icône ORME (cadran),
  aussi embarquée comme icône de fenêtre de l'application.
- [`orme-logo.svg`](pic/orme-logo.svg) — logo ORME complet (icône + texte).
- [`osne-icon.svg`](pic/osne-icon.svg) / `osne-icon.png` — icône OSNE (hélice
  d'agitateur), aussi embarquée comme icône de fenêtre d'OSNE.
- [`osne-logo.svg`](pic/osne-logo.svg) — logo OSNE complet (icône + texte).
- [`ru_opcua-icon.svg`](pic/ru_opcua-icon.svg) / `ru_opcua-icon.png` — icône ORUE
  (cadran de régulateur entouré d'un anneau de nœud OPC UA), aussi embarquée comme
  icône de fenêtre d'ORUE.
- [`ru_opcua-logo.svg`](pic/ru_opcua-logo.svg) — logo ORUE complet (icône + texte).
- [`Logo-CESAM-Couleur-vect.png`](pic/Logo-CESAM-Couleur-vect.png) — logo CESAM-Lab.

Chaque icône est **générée** depuis son script `*-logo.gen.py`
([`pic/orme-logo.gen.py`](pic/orme-logo.gen.py),
[`pic/osne-logo.gen.py`](pic/osne-logo.gen.py),
[`pic/ru_opcua-logo.gen.py`](pic/ru_opcua-logo.gen.py)). Les scripts OSNE et ORUE
rastérisent aussi leur `-icon.png` directement (via Pillow) ; le `.svg` d'ORME est
rasterisé ensuite.

Sous **Wayland**, installer l'icône de barre des tâches d'un instrument avec
`scripts/install-desktop.sh [orme|osne|ru_opcua]`.

## Licence

[MIT](LICENSE) © 2026 CESAM-Lab

Les composants tiers intégrés dans certains instruments sont distribués sous leurs propres licences (notamment la pile OPC UA sous MPL-2.0 utilisée par `mock_bin_ru_opcua`) ; voir [NOTICE](NOTICE). Ils ne modifient pas la licence MIT du code de cesam-tools.
