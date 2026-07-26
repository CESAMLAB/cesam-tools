# Manuel utilisateur — Régulateur EtherNet/IP (OREE)

*🌍 **FR** · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

---

## 1. À quoi sert l'instrument

**OREE** simule une **unité de régulation** de procédé (PID + procédé thermique du
premier ordre) et l'expose comme un **adaptateur EtherNet/IP** (messagerie explicite
CIP). Il sert à tester une supervision ou un client EtherNet/IP (pycomm3, RSLinx en
lecture, rseip…) sans matériel réel.

## 2. Prise en main

```bash
cargo run -p mock_bin_ru_ethernetip        # IHM + adaptateur EtherNet/IP
```

Le serveur écoute par défaut sur `0.0.0.0:44818` (aucun privilège requis). L'en-tête
indique l'état : **EtherNet/IP ●** (vert) avec l'adresse d'écoute, ou un message
d'erreur (rouge). Un bandeau orange avertit si le serveur est **exposé** (toutes
interfaces + liste blanche vide).

## 3. Interface

- **En-tête** : titre, boutons *Paramètres* / *Sauvegarder*, état marche/arrêt, état
  d'écoute EtherNet/IP, bandeau d'exposition réseau.
- **Panneau gauche (Commandes)** : *Marche/Arrêt*, *Mode automatique (PID)*,
  *Consigne*, *Sortie manuelle* (mode manuel), réglages **PID** (Kp/Ki/Kd).
- **Panneau central** : cartes *Mesure / Consigne / Sortie* + **courbe** temps réel.
- **Modal *Paramètres*** : langue, vérification de MAJ, **réseau EtherNet/IP** (IP
  d'écoute, port, **liste blanche** d'IP — un motif par ligne, `*` = joker),
  **procédé** (K, τ, retard, ambiant), **bornes de consigne**. *Appliquer* relance
  l'écoute si l'IP/port change et sauvegarde le TOML.

## 4. Connecter un client EtherNet/IP

Le client se connecte à l'IP/port du serveur (`RegisterSession` automatique), puis
lit/écrit les **tags nommés** par messagerie explicite : `Setpoint`, `ProcessValue`,
`Output`, `ManualOutput`, `Run`, `Auto`, etc. (voir
[`reference_ethernetip.md`](reference_ethernetip.md)). ⚠️ Les valeurs sont en
**little-endian** (REAL = `f32` LE).

## 5. FAQ

- **Le client ne se connecte pas** → vérifier IP/port (44818), la **liste blanche**,
  le pare-feu.
- **Tag introuvable** → seuls les tags documentés existent ; les noms sont
  sensibles à la casse.
- **Mes écritures n'ont pas d'effet** → seuls les tags pilotables agissent
  (`Setpoint`, `ManualOutput`, `Run`, `Auto`) ; les autres sont en lecture seule.
- **Où est le fichier de config ?** → `mock_ru_ethernetip.toml` (répertoire courant ;
  surchargeable par `MOCK_CONFIG`).
