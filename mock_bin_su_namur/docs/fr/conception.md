# Conception — Agitateur de laboratoire simulé (OSNE)

*🌍 **FR** · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

> Crate : `mock_bin_su_namur` · Exécutable : **OSNE** (*Open Stirrer NAMUR Emulator*)

Document d'architecture et de modélisation. Calqué sur le régulateur **ORME**
(`mock_bin_ru_modbustcp`) : même découpage **modèle métier synchrone / acteurs
ractor / couche protocole / IHM egui**, mêmes invariants.

---

## 1. Objet

Simuler un **agitateur de laboratoire** (façon IKA) piloté par le protocole série
**NAMUR**. Le moteur possède une **fonction de transfert** (dynamique de vitesse)
asservie par une **régulation rapide**, et la **viscosité** du milieu est réglable
et influe sur le couple.

---

## 2. Modèle physique

### Moteur ([`motor.rs`](../../src/motor.rs))

Équilibre des couples, intégré par Euler explicite :

```text
J · dω/dt = T_moteur − k · η · ω − T_frottement
```

- `ω` : vitesse (tr/min) ;
- `T_moteur` : couple moteur (commande, N·cm, ≥ 0) ;
- `k · η · ω` : **couple de charge visqueux** (∝ viscosité `η` et vitesse) ;
- `T_frottement` : frottement sec résiduel ;
- `J` (`inertia`) : règle la **réactivité** (petit ⇒ rapide).

En régime établi, `T_moteur = k·η·ω + T_frottement` : le couple nécessaire pour
tenir une vitesse **croît avec la viscosité**. Si ce couple dépasse le **couple
maximal**, la consigne n'est plus atteignable → **surcharge**.

### Asservissement ([`stirrer.rs`](../../src/stirrer.rs))

Un **PID** ([`mock_lib_control::Pid`], réutilisé d'ORME) prend l'erreur de vitesse
`consigne − mesure` et produit le **couple moteur**, borné à `[0, couple_max]`. Les
gains par défaut sont volontairement « raides » : la sortie sature au couple max
tant que l'erreur est grande (montée rapide), puis le terme intégral stabilise.
Le pas de simulation est de **20 ms** (50 Hz), plus fin que celui d'ORME car la
dynamique d'un moteur est rapide.

---

## 3. Architecture (acteurs)

```
IHM (egui) ──Command(cast)──►  SimulationActor ──refresh──► SharedSnapshot ──► IHM
Serveur NAMUR ──Command(cast)─►   (Stirrer)     ──refresh──► SharedSnapshot ──► lectures NAMUR
```

- **`SimulationActor`** ([`actors/simulation.rs`](../../src/actors/simulation.rs)) :
  propriétaire unique du `Stirrer` ; avance la simulation sur un timer one-shot
  ré-armé (pas de timer détaché) et publie un `SharedSnapshot`.
- **`NamurServerActor`** ([`actors/network.rs`](../../src/actors/network.rs)) :
  possède le serveur NAMUR, relançable à chaud (`Reconfigure`) ; liste blanche
  d'IP partagée ; statut d'écoute publié pour l'IHM.
- **Serveur NAMUR** ([`namur_server.rs`](../../src/namur_server.rs)) : lit les
  lignes ASCII, les interprète ([`namur.rs`](../../src/namur.rs)), répond aux
  lectures et relaie les écritures/actions à l'acteur. **Un maître à la fois**
  (point-à-point). **Chien de garde** par session.

Les lectures NAMUR puisent dans le `SharedSnapshot` (pas de table mémoire séparée
comme le Modbus d'ORME : le protocole NAMUR est orienté « commandes », pas
« registres »).

---

## 4. Configuration & sécurité

- `AppConfig` (langue / réseau-série / moteur / régulation) sérialisée en **TOML**
  ([`config.rs`](../../src/config.rs)), **assainie au chargement**
  (`AppConfig::sanitized` : bornes ordonnées, flottants finis) — invariant partagé
  avec ORME (ne jamais `clamp` avec des bornes non validées).
- NAMUR n'a **ni authentification ni chiffrement** : réseau de confiance + liste
  blanche d'IP (TCP). Défaut `0.0.0.0` + liste vide ⇒ exposé : l'IHM affiche un
  **bandeau d'avertissement**.

---

## 5. Pistes d'évolution

- Sens de rotation (CW/CCW) et rampe d'accélération.
- Capteur de température (`IN_PV_2/3`) si un modèle thermique est ajouté.
- Couple de charge non linéaire (régime turbulent ∝ ω²).
- Promotion du modèle moteur dans `mock_lib_control` s'il sert un second instrument.
