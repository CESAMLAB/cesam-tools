# Référence MQTT Sparkplug B — métriques & cycle de vie (RU/Sparkplug B)

*🌍 **FR** · [EN](../en/reference_sparkplugb.md) · [DE](../de/reference_sparkplugb.md) · [ES](../es/reference_sparkplugb.md) · [IT](../it/reference_sparkplugb.md) · [PT](../pt/reference_sparkplugb.md) · [NL](../nl/reference_sparkplugb.md) · [PL](../pl/reference_sparkplugb.md)*

> Source de vérité : [`sparkplug_node.rs`](../../src/sparkplug_node.rs) (topics, table
> de métriques, payloads, mapping NCMD). Toute évolution se fait **dans ce fichier**
> et se répercute ici.

---

## 1. Rôle & connexion

L'instrument est un **edge node Sparkplug B** : il ne **n'écoute aucun port**, il se
**connecte en sortie** à un **broker MQTT externe** (mosquitto, EMQX, HiveMQ…) et
publie l'état du régulateur. Réglages dans la section `[network]` du TOML / le modal
*Paramètres* :

| Clé | Défaut | Rôle |
|---|---|---|
| `broker_host` | `localhost` | hôte du broker MQTT |
| `broker_port` | `1883` | port (`8883` en TLS) |
| `client_id` | `ru_spb` | identifiant de client MQTT |
| `group_id` | `CESAM` | groupe Sparkplug (`spBv1.0/<group_id>/…`) |
| `edge_node_id` | `RU1` | nœud edge (`…/<edge_node_id>`) |
| `username` / `password` | *(vide)* | auth MQTT (mot de passe **en clair**, simulateur uniquement) |
| `tls` | `false` | chiffrement TLS (rustls) vers le broker |
| `keepalive_secs` | `30` | keepalive MQTT |
| `publish_on_change` | `true` | `true` : `NDATA` dès qu'une métrique change (cadence = pas de simulation, 0,5 s) ; `false` : périodique |
| `publish_period_secs` | `5` | cadence périodique quand `publish_on_change = false` |

> ⚠️ **MQTT en clair par défaut** : sans TLS, le trafic n'est ni chiffré ni
> authentifié réseau. À n'utiliser que sur un **réseau de confiance**. L'IHM affiche
> un bandeau d'avertissement tant que `tls` est désactivé.

---

## 2. Espace de noms (topics)

Namespace `spBv1.0`. Topics du nœud :

```
spBv1.0/<group_id>/NBIRTH/<edge_node_id>
spBv1.0/<group_id>/NDATA/<edge_node_id>
spBv1.0/<group_id>/NDEATH/<edge_node_id>
spBv1.0/<group_id>/NCMD/<edge_node_id>
```

Avec les valeurs par défaut : `spBv1.0/CESAM/NBIRTH/RU1`, etc.

---

## 3. Table de métriques

Toutes les métriques de données vivent sous le **nœud edge** (pas de *device* dans
cette version). Type Sparkplug (Eclipse Tahu) : `Float` (9), `Boolean` (11),
`UInt64` (8).

| Métrique | Type | Lecture/Écriture | Champ instantané (lecture) | NCMD → commande (écriture) |
|---|---|:--:|---|---|
| `Setpoint` | Float | R/W | `setpoint` | `SetSetpoint` |
| `ProcessValue` | Float | R | `pv` | — |
| `Output` | Float | R | `output` | — |
| `ManualOutput` | Float | R/W | `manual_output` | `SetManualOutput` |
| `Run` | Boolean | R/W | `run` | `SetRun` |
| `Auto` | Boolean | R/W | `auto` | `SetAuto` |
| `SetpointMin` | Float | R | `sp_min` | *(réglé via IHM/TOML)* |
| `SetpointMax` | Float | R | `sp_max` | *(réglé via IHM/TOML)* |
| `PID/Kp` | Float | R | `pid.kp` | *(réglé via IHM/TOML)* |
| `PID/Ki` | Float | R | `pid.ki` | *(réglé via IHM/TOML)* |
| `PID/Kd` | Float | R | `pid.kd` | *(réglé via IHM/TOML)* |
| `bdSeq` | UInt64 | R | *(compteur de session)* | — |
| `Node Control/Rebirth` | Boolean | W | — | republie un `NBIRTH` |

**Surface pilotable par `NCMD`** : `Setpoint`, `ManualOutput`, `Run`, `Auto`, plus
`Node Control/Rebirth` (parité avec les écritures OPC UA de l'instrument ORUE). Les
bornes de consigne et les gains PID sont **publiés** (observables par un SCADA) mais
se règlent via l'IHM/TOML. Une métrique inconnue ou de **mauvais type** dans un
`NCMD` est **ignorée** (jamais d'erreur, jamais de valeur aberrante : la simulation
assainit toute écriture).

---

## 4. Cycle de vie

- **`NBIRTH`** — publié à chaque connexion (ConnAck). Contient **toutes** les
  métriques (avec valeurs), `bdSeq`, et `Node Control/Rebirth`. `seq = 0`.
- **`NDATA`** — métriques **modifiées** uniquement, `seq` roulant **0–255**.
- **`NDEATH`** — contient `bdSeq` **seul**, **sans** `seq`. Déposé comme **Last Will
  MQTT** à la connexion : le **broker** le publie automatiquement à la perte du lien
  (arrêt, reconfiguration, panne). Pas de `NDEATH` explicite côté nœud.
- **`NCMD`** — abonnement `spBv1.0/<group>/NCMD/<node>` (QoS 1) souscrit juste après
  le `NBIRTH`. Décodé → commandes appliquées à la simulation.
- **`bdSeq`** — incrémenté à chaque (re)démarrage du client ; le `NDEATH` (Last Will)
  et le `NBIRTH` d'une **même session** portent la **même** valeur (invariant
  Sparkplug). Affiché dans l'IHM (diagnostic).
- **`seq`** — remis à 0 à chaque `NBIRTH`, incrémenté (roulant) à chaque `NDATA`.
- **Renaissance** (`Node Control/Rebirth = true` via `NCMD`) → republication d'un
  `NBIRTH` (resynchronisation SCADA).

---

## 5. Exemple client (SCADA)

Abonnement à tout le groupe, puis envoi d'une consigne :

```bash
# Observer les messages du nœud
mosquitto_sub -h localhost -t 'spBv1.0/CESAM/#' -v

# (les payloads sont du protobuf Sparkplug B — utiliser un décodeur Tahu pour les lire)
```

Un `NCMD` publié sur `spBv1.0/CESAM/NCMD/RU1` avec les métriques `Run=true` et
`Setpoint=80.0` démarre la régulation et fixe la consigne ; un `NDATA` ultérieur
reflète le changement.
