# Manuel utilisateur — Régulateur Sparkplug B (ORSE)

*🌍 **FR** · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

---

## 1. À quoi sert l'instrument

**ORSE** simule une **unité de régulation** de procédé (PID + procédé thermique du
premier ordre) et publie son état en **MQTT Sparkplug B**, comme un **edge node**
qui se connecte à un **broker** et expose des métriques à un SCADA. Il sert à tester
une chaîne d'acquisition Sparkplug B (Ignition, Chariot, EMQX, Node-RED…) sans
matériel réel.

## 2. Prérequis : un broker MQTT

ORSE étant un **client**, il faut un broker MQTT joignable. En local :

```bash
docker run -it --rm -p 1883:1883 eclipse-mosquitto
```

## 3. Prise en main

```bash
cargo run -p mock_bin_ru_sparkplugb        # IHM + edge node Sparkplug B
```

Au démarrage, l'IHM tente de se connecter au broker (`localhost:1883` par défaut).
L'en-tête indique l'état : **Connecté** (vert) une fois le `NBIRTH` publié, ou
**Déconnecté** (rouge) avec le motif. Un bandeau orange **⚠ MQTT en clair** rappelle
l'absence de TLS.

## 4. Interface

- **En-tête** : titre, boutons *Paramètres* / *Sauvegarder*, état marche/arrêt, état
  de connexion Sparkplug B, bandeau TLS/clair.
- **Panneau gauche (Commandes)** : *Marche/Arrêt*, *Mode automatique (PID)*,
  *Consigne*, *Sortie manuelle* (mode manuel), réglages **PID** (Kp/Ki/Kd).
- **Panneau central** : cartes *Mesure / Consigne / Sortie* + **courbe** temps réel.
- **Modal *Paramètres*** : langue, vérification de MAJ, **Broker MQTT / Sparkplug B**
  (hôte, port, client_id, group_id, edge_node_id, keepalive, TLS, utilisateur/mot de
  passe, publication sur changement/périodique), **procédé** (K, τ, retard, ambiant),
  **bornes de consigne**. *Appliquer* relance la connexion et sauvegarde le TOML.

## 5. Piloter depuis un SCADA

Le SCADA s'abonne à `spBv1.0/<group_id>/#` et reçoit `NBIRTH` puis `NDATA`. Pour
**commander** le régulateur, il publie un `NCMD` sur
`spBv1.0/<group_id>/NCMD/<edge_node_id>` avec les métriques pilotables (`Setpoint`,
`Run`, `Auto`, `ManualOutput`) ou `Node Control/Rebirth = true` pour forcer une
renaissance. Détails : [`reference_sparkplugb.md`](reference_sparkplugb.md).

## 6. FAQ

- **« Déconnecté » en permanence** → broker injoignable : vérifier `broker_host`/
  `broker_port`, le pare-feu, et que le broker tourne.
- **Le SCADA ne voit rien** → vérifier le `group_id`/`edge_node_id` et l'abonnement
  `spBv1.0/<group>/#` ; les payloads sont du **protobuf** (décodeur Sparkplug requis).
- **Mes écritures NCMD sont ignorées** → métrique non pilotable ou mauvais type (cf.
  table des métriques). Seules `Setpoint`/`Run`/`Auto`/`ManualOutput` et `Rebirth`
  sont acceptées.
- **Où est le fichier de config ?** → `mock_ru_sparkplugb.toml` (répertoire courant ;
  surchargeable par `MOCK_CONFIG`).
