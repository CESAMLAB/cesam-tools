# Référence OPC UA — espace d'adressage (RU/OPC UA)

*🌍 **FR** · [EN](../en/reference_opcua.md) · [DE](../de/reference_opcua.md) · [ES](../es/reference_opcua.md) · [IT](../it/reference_opcua.md) · [PT](../pt/reference_opcua.md) · [NL](../nl/reference_opcua.md) · [PL](../pl/reference_opcua.md)*

> Source de vérité : [`opcua_server.rs`](../../src/opcua_server.rs) (déclaration des
> nœuds + callbacks). Toute évolution de la table se fait **dans ce fichier** et se
> répercute ici.

---

## 1. Endpoint

| Élément | Valeur |
|---|---|
| URL | `opc.tcp://<bind_ip>:<port>/` (défaut `opc.tcp://0.0.0.0:4840/`) |
| Transport | OPC UA TCP binaire |
| Politique de sécurité | `None` |
| Mode de sécurité | `None` |
| Jeton utilisateur | `Anonymous` |

⚠️ **Sécurité None** : ni authentification ni chiffrement (Phase 1b). À n'exposer
que sur un **réseau de confiance**. Sécurité réelle (`Basic256Sha256`, certificats,
auth) prévue en **Phase 2**.

---

## 2. Namespace

| Index | URI |
|---|---|
| `0` | `http://opcfoundation.org/UA/` (namespace cœur OPC UA) |
| `ns` | `urn:cesam-lab:ru-opcua` (namespace applicatif) |

L'index `ns` du namespace applicatif est attribué dynamiquement au démarrage ;
un client le résout via `IN GetNamespaceArray` / le service *Browse*. Les nœuds
métier ci-dessous y vivent.

---

## 3. Nœuds (sous le dossier `Objects`)

Chaque nœud est une `Variable` ; son `NodeId` est de la forme `ns=<ns>;s=<nom>`.

| BrowseName | NodeId (`s=`) | Type | Accès | Grandeur |
|---|---|---|:--:|---|
| `Setpoint` | `Setpoint` | `Double` | R/W | Consigne (unité procédé) |
| `ProcessValue` | `ProcessValue` | `Double` | R | Mesure (PV) |
| `Output` | `Output` | `Double` | R | Sortie de commande (%) |
| `ManualOutput` | `ManualOutput` | `Double` | R/W | Sortie imposée en mode manuel (%) |
| `Run` | `Run` | `Boolean` | R/W | Marche / arrêt de la régulation |
| `Auto` | `Auto` | `Boolean` | R/W | Mode automatique (PID) vs manuel |

- **Lectures** : servies par un callback qui lit l'**instantané partagé** ; elles
  sont donc « vivantes » et **échantillonnables** par les abonnements (*Subscription*
  / *MonitoredItem*).
- **Écritures** : routées vers l'acteur de simulation. Les valeurs sont **assainies**
  (non finies rejetées, consigne bornée, sortie manuelle bornée à `[0, 100]`).

---

## 4. Mapping vers l'état métier

| Nœud | Effet d'une écriture | Source d'une lecture |
|---|---|---|
| `Setpoint` | `Command::SetSetpoint` (bornée `[sp_min, sp_max]`) | `snapshot.setpoint` |
| `ManualOutput` | `Command::SetManualOutput` (bornée `[0, 100]`) | `snapshot.manual_output` |
| `Run` | `Command::SetRun` | `snapshot.run` |
| `Auto` | `Command::SetAuto` | `snapshot.auto` |
| `ProcessValue` | — (lecture seule) | `snapshot.pv` |
| `Output` | — (lecture seule) | `snapshot.output` |

Une écriture d'un type inattendu renvoie `Bad_TypeMismatch` ; une écriture sans
valeur, `Bad_NothingToDo`. Le `Float` est accepté en plus du `Double` pour les
nœuds numériques.

---

## 5. Exemples (client OPC UA)

Avec un client générique (UaExpert, `opcua` CLI, etc.), se connecter à
`opc.tcp://127.0.0.1:4840/`, sécurité **None**, utilisateur **Anonymous**, puis :

```text
# Lecture de la mesure et de la consigne
Read  ns=<ns>;s=ProcessValue   → 60.0
Read  ns=<ns>;s=Setpoint       → 60.0

# Démarrage + nouvelle consigne
Write ns=<ns>;s=Run        = true
Write ns=<ns>;s=Setpoint   = 80.0

# Bascule en manuel et sortie imposée à 40 %
Write ns=<ns>;s=Auto         = false
Write ns=<ns>;s=ManualOutput = 40.0
```

S'abonner (*Subscribe* / *MonitoredItem*) à `ProcessValue` et `Output` permet de
suivre la dynamique du procédé en temps réel.
