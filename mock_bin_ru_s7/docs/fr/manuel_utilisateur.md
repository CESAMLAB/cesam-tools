# Manuel utilisateur — Régulateur S7 (ORSS)

*🌍 **FR** · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

---

## 1. À quoi sert l'instrument

**ORSS** simule une **unité de régulation** de procédé (PID + procédé thermique du
premier ordre) et l'expose comme un **automate Siemens S7** (serveur S7comm sur
ISO-on-TCP). Il sert à tester une supervision ou un client S7 (Snap7, TIA Portal en
lecture, nodes7…) sans automate réel.

## 2. Prise en main

```bash
cargo run -p mock_bin_ru_s7        # IHM + serveur S7
```

Le serveur écoute par défaut sur `0.0.0.0:102`. ⚠️ Le **port 102 nécessite les
droits root** ; sinon, réglez un port haut (ex. 1102) dans le modal *Paramètres*.

L'en-tête indique l'état : **S7 ●** (vert) avec l'adresse d'écoute, ou un message
d'erreur (rouge) si le bind échoue. Un bandeau orange avertit si le serveur est
**exposé** (toutes interfaces + liste blanche vide).

## 3. Interface

- **En-tête** : titre, boutons *Paramètres* / *Sauvegarder*, état marche/arrêt, état
  d'écoute S7, bandeau d'exposition réseau.
- **Panneau gauche (Commandes)** : *Marche/Arrêt*, *Mode automatique (PID)*,
  *Consigne*, *Sortie manuelle* (mode manuel), réglages **PID** (Kp/Ki/Kd).
- **Panneau central** : cartes *Mesure / Consigne / Sortie* + **courbe** temps réel.
- **Modal *Paramètres*** : langue, vérification de MAJ, **réseau S7** (IP d'écoute,
  port, **liste blanche** d'IP — un motif par ligne, `*` = joker), **procédé**
  (K, τ, retard, ambiant), **bornes de consigne**. *Appliquer* relance l'écoute si
  l'IP/port change et sauvegarde le TOML.

## 4. Connecter un client S7

Le client se connecte à l'IP/port du serveur. Les valeurs **rack/slot** usuelles
(0/1 ou 0/2) fonctionnent : le serveur n'impose pas de TSAP. Les grandeurs sont dans
**DB1** (voir [`reference_s7.md`](reference_s7.md)) : consigne en `DB1.DBD0`, mesure
en `DB1.DBD4`, marche en `DB1.DBX16.0`, etc.

## 5. FAQ

- **« Permission denied » au démarrage** → le port 102 exige les droits root ;
  utilisez un port haut ou lancez avec les privilèges adéquats.
- **Le client ne se connecte pas** → vérifier IP/port, la **liste blanche**, le
  pare-feu. Tester rack/slot 0/1 puis 0/2.
- **Mes écritures n'ont pas d'effet** → seuls les offsets pilotables agissent
  (consigne, sortie manuelle, marche, auto) ; les autres sont en lecture seule.
- **Où est le fichier de config ?** → `mock_ru_s7.toml` (répertoire courant ;
  surchargeable par `MOCK_CONFIG`).
