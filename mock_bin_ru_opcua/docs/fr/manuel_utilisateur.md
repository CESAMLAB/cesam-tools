# Manuel utilisateur — Régulateur de procédé simulé (RU/OPC UA)

*🌍 **FR** · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

> Crate : `mock_bin_ru_opcua` · Exécutable : **ru_opcua**

---

## 1. À quoi sert ce simulateur

`ru_opcua` simule un **régulateur de procédé** (boucle PID sur un procédé
thermique) et l'expose en **OPC UA**, le standard de supervision industrielle.
Il sert à **tester un client OPC UA / un SCADA** (lecture de mesures, écriture de
consignes, abonnements) sans matériel réel.

L'interface graphique permet de **piloter** la simulation et de **visualiser** la
dynamique ; le serveur OPC UA expose les mêmes grandeurs au réseau.

---

## 2. Prise en main

```bash
cargo run -p mock_bin_ru_opcua          # IHM + serveur OPC UA
```

Au lancement, le serveur écoute par défaut sur `opc.tcp://0.0.0.0:4840/`
(sécurité None). La fenêtre affiche l'état courant et démarre la courbe de
tendance.

Connectez un client OPC UA (UaExpert, etc.) à `opc.tcp://127.0.0.1:4840/`,
sécurité **None**, utilisateur **Anonymous**. Les nœuds sont décrits dans la
[référence OPC UA](reference_opcua.md).

---

## 3. L'interface

### En-tête

- **Titre** et boutons **⚙ Paramètres** / **💾 Sauvegarder les réglages**.
- À droite : **état de l'appareil** (EN MARCHE / À L'ARRÊT), **état du serveur**
  (`OPC UA ● opc.tcp://…` en vert si à l'écoute, ✖ + message en cas d'erreur), et
  le **logo CESAM-Lab**.
- Un **bandeau orange** rappelle en permanence que l'endpoint est **anonyme
  (sécurité None)** : à n'exposer que sur réseau de confiance.
- Si une mise à jour est disponible, un **bandeau** propose le téléchargement.

### Panneau de commandes (gauche)

- **Marche / Arrêt** : démarre ou arrête la régulation. À l'arrêt, le procédé
  relaxe vers la valeur ambiante.
- **Mode automatique (PID)** : activé = le PID calcule la sortie ; désactivé =
  **mode manuel** (la sortie est imposée).
- **Consigne** : curseur, borné par les bornes de consigne (réglables dans
  *Paramètres*).
- **Sortie manuelle (%)** : curseur actif uniquement en **mode manuel**.
- **Réglages PID** : gains `Kp`, `Ki`, `Kd` éditables à chaud.

### Zone centrale

- **Cartes** : Mesure, Consigne, Sortie.
- **Courbe de tendance** : Mesure (PV) et Consigne sur l'axe de gauche (unité
  procédé), Sortie (%) sur l'axe de droite.

---

## 4. Paramètres (modal ⚙)

- **Langue** de l'interface (8 langues), persistée.
- **Vérifier les mises à jour au démarrage** + bouton **Vérifier maintenant**.
- **Endpoint** : **IP d'écoute** et **port** du serveur OPC UA. Un changement
  **relance** le serveur à chaud (les sessions en cours sont fermées proprement).
- **Procédé (fonction de transfert)** : gain `K`, constante de temps `τ`, retard
  pur, valeur ambiante.
- **Bornes de consigne** : min / max (réordonnées automatiquement si inversées).
- **Appliquer** / **Réinitialiser par défaut** / **Fermer**.

Les réglages sont sauvegardés dans `mock_ru_opcua.toml` (répertoire courant ;
surchargeable via la variable d'environnement `MOCK_CONFIG`).

---

## 5. Sécurité

OPC UA **peut** être sécurisé (certificats, chiffrement, authentification), mais
en l'état (**Phase 1b**) le simulateur n'expose qu'un endpoint **sécurité None**
**anonyme** : aucune protection. **Ne pas exposer sur un réseau ouvert.** Le
bandeau d'avertissement le rappelle en permanence. La sécurité réelle est prévue
en **Phase 2**.

---

## 6. FAQ

**Le port 4840 est-il imposé ?** Non : il se règle dans *Paramètres* (ou via le
fichier TOML). Un port < 1024 nécessite les droits root.

**Mon client ne voit pas les nœuds.** Vérifiez la connexion à `opc.tcp://…:4840/`,
sécurité **None**, utilisateur **Anonymous**, puis *Browse* sous le dossier
`Objects` (namespace `urn:cesam-lab:ru-opcua`).

**Une écriture est refusée.** Le type doit correspondre (`Double` pour les
grandeurs, `Boolean` pour `Run`/`Auto`) ; sinon le serveur renvoie
`Bad_TypeMismatch`.

**Lancer sans interface graphique ?** Compilez en *headless* :
`cargo run -p mock_bin_ru_opcua --no-default-features` — le serveur OPC UA et la
simulation tournent sans IHM.

**Un message « encrypted endpoints disabled » apparaît.** C'est normal en
Phase 1b : aucun certificat d'instance n'est provisionné (endpoints chiffrés
indisponibles). L'endpoint None, lui, fonctionne.
