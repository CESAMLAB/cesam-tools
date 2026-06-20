# ORME — régulateur simulé Modbus

*🌍 [English](README.md) · **Français** · [Deutsch](README.de.md) · [Español](README.es.md) · [Italiano](README.it.md) · [Português](README.pt.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md)*

> *Open Regulator Modbus Emulator* · paquet `mock_bin_ru_modbustcp` · binaire `orme`

Régulateur industriel **simulé**, esclave **Modbus TCP/RTU**, avec interface
graphique. Fait partie du workspace [`cesam-tools`](../README.fr.md).

## Fonctionnalités

- Procédé du premier ordre + retard pur (fonction de transfert FOPDT).
- Régulation bidirectionnelle (chaud / froid), chaque sens en **PID** ou
  **tout-ou-rien**.
- Modes marche/arrêt et auto/manuel ; consignes auto (physique) et manuelle (%).
- Serveur Modbus TCP exposant l'intégralité de l'état.
- IHM `egui` avec courbe de tendance temps réel et réglage des gains PID.
- **Interface multilingue** : français, anglais, allemand, espagnol, italien,
  portugais, néerlandais, polonais (choix dans le modal *Paramètres*, persisté).

## Lancer

```bash
cargo run -p mock_bin_ru_modbustcp
# Fichier de configuration alternatif :
MOCK_CONFIG=./ma_config.toml cargo run -p mock_bin_ru_modbustcp
```

Écoute par défaut sur `0.0.0.0:5502`. Le port, l'IP d'écoute et la liste blanche
d'IP se règlent dans le modal **⚙ Paramètres** et sont persistés en TOML.

## Table d'adresses Modbus

Encodage des flottants : 2 registres, big-endian, mot de poids fort en premier.

### Bobines (FC 1/5/15)

| Adr | Rôle |
|----|------|
| 0 | Marche (1) / Arrêt (0) |
| 1 | Auto (1) / Manuel (0) |

### Entrées discrètes (FC 2, lecture seule)

| Adr | Rôle |
|----|------|
| 0 | En marche |
| 1 | Sens 1 (chaud) actif |
| 2 | Sens 2 (froid) actif |

### Registres de maintien (FC 3/6/16)

| Adr | Type | Rôle |
|-----|------|------|
| 0 | u16 | Mode sens 1 (0=Off, 1=PID, 2=TOR) |
| 1 | u16 | Mode sens 2 (0=Off, 1=PID, 2=TOR) |
| 2–3 | f32 | Consigne automatique (SP) |
| 4–5 | f32 | Consigne manuelle (% sortie, signée) |
| 6–7 | f32 | Kp sens 1 |
| 8–9 | f32 | Ki sens 1 |
| 10–11 | f32 | Kd sens 1 |
| 12–13 | f32 | Kp sens 2 |
| 14–15 | f32 | Ki sens 2 |
| 16–17 | f32 | Kd sens 2 |
| 18–19 | f32 | Hystérésis TOR |

### Registres d'entrée (FC 4, lecture seule)

| Adr | Type | Rôle |
|-----|------|------|
| 0–1 | f32 | Mesure (PV) |
| 2–3 | f32 | Sortie appliquée (% signé : + chaud / − froid) |

La source de vérité est l'en-tête de [`src/map.rs`](src/map.rs).

## Documentation

Documentation propre à cette application (dossier [`docs/fr/`](docs/fr/)) :

- [**Manuel utilisateur**](docs/fr/manuel_utilisateur.md) — prise en main, pilotage, paramètres, FAQ.
- [Document de conception](docs/fr/conception.md) — architecture, choix techniques, théorie de régulation.
- [Table d'adresses Modbus](docs/fr/table_modbus.md) — plan d'adressage complet, encodage, exemples.
- [Maintenance logicielle](docs/fr/maintenance.md) — build, configuration, extension, dépannage.
