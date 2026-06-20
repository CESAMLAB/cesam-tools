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

## Téléchargement

Des binaires précompilés sont disponibles sur la page [**Releases**](https://github.com/CESAMLAB/cesam-tools/releases/latest) — **aucune chaîne d'outils Rust requise**.

| Plateforme | IHM | Headless (TCP seul, sans IHM) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`orme-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-linux-x86_64) | [`orme-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-linux-x86_64-headless) |
| Windows x86_64 | [`orme-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`orme-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-rpi-arm64) | [`orme-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-rpi-arm64-headless) |

```bash
chmod +x orme-linux-x86_64        # Linux / Raspberry Pi
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

Voir [CLAUDE.md](CLAUDE.md) pour les conventions et l'architecture détaillée.

## Documentation

Chaque instrument porte sa propre documentation dans son sous-dossier `docs/`,
disponible en huit langues (`docs/<langue>/`). Pour le régulateur (version
française) :

- [**Manuel utilisateur**](mock_bin_ru_modbustcp/docs/fr/manuel_utilisateur.md) — prise en main, IHM, paramètres, FAQ.
- [Document de conception](mock_bin_ru_modbustcp/docs/fr/conception.md) — architecture et choix techniques.
- [Table d'adresses Modbus](mock_bin_ru_modbustcp/docs/fr/table_modbus.md) — plan d'adressage complet.
- [Maintenance logicielle](mock_bin_ru_modbustcp/docs/fr/maintenance.md) — build, configuration, extension, dépannage.

## Marque & logos

Les logos sont dans [`pic/`](pic/) :

- [`orme-icon.svg`](pic/orme-icon.svg) / `orme-icon.png` — icône ORME (cadran),
  aussi embarquée comme icône de fenêtre de l'application.
- [`orme-logo.svg`](pic/orme-logo.svg) — logo ORME complet (icône + texte).
- [`Logo-CESAM-Couleur-vect.png`](pic/Logo-CESAM-Couleur-vect.png) — logo CESAM-Lab.

L'icône ORME est **générée** depuis [`pic/orme-logo.gen.py`](pic/orme-logo.gen.py)
(`python3 pic/orme-logo.gen.py` produit les `.svg`, à rasteriser ensuite).

## Licence

[MIT](LICENSE) © 2026 CESAM-Lab
