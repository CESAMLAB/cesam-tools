# Référence EtherNet/IP — tags & protocole (RU/EtherNet/IP)

*🌍 **FR** · [EN](../en/reference_ethernetip.md) · [DE](../de/reference_ethernetip.md) · [ES](../es/reference_ethernetip.md) · [IT](../it/reference_ethernetip.md) · [PT](../pt/reference_ethernetip.md) · [NL](../nl/reference_ethernetip.md) · [PL](../pl/reference_ethernetip.md)*

> Source de vérité : [`eip_server.rs`](../../src/eip_server.rs) (encapsulation,
> dispatch CIP, table de tags). Toute évolution se fait **dans ce fichier** et se
> répercute ici.

---

## 1. Endpoint

Adaptateur **EtherNet/IP** (messagerie explicite **CIP** non connectée) sur TCP.
Écoute par défaut sur `0.0.0.0:44818` (port standard EtherNet/IP, > 1024 → aucun
privilège requis). Réglages dans la section `[network]` du TOML / le modal
*Paramètres* :

| Clé | Défaut | Rôle |
|---|---|---|
| `bind_ip` | `0.0.0.0` | IP d'écoute |
| `port` | `44818` | port TCP (EtherNet/IP standard) |
| `allowlist` | *(vide)* | liste blanche d'IP (motifs `*` par octet ; vide = tout autorisé) |

> ⚠️ **Aucune authentification ni chiffrement** (EtherNet/IP « classic »). Le seul
> contrôle d'accès est la **liste blanche d'IP** + la topologie réseau. `0.0.0.0` +
> liste vide = **exposé** : l'IHM affiche un bandeau d'avertissement.

⚠️ EtherNet/IP / CIP est **little-endian** (à l'inverse de Modbus/S7). Les `REAL`
sont des `f32` IEEE-754 little-endian.

## 2. Sessions

Plusieurs clients **simultanés** sont acceptés. Chaque session : `RegisterSession`
(le serveur attribue un *session handle* non nul) → `SendRRData` portant les requêtes
CIP → `UnRegisterSession` (ou déconnexion TCP).

## 3. Sous-ensemble protocole implémenté

- **Encapsulation** : `RegisterSession` (0x0065), `UnRegisterSession` (0x0066),
  `SendRRData` (0x006F, messagerie explicite non connectée, CPF).
- **CIP** : `Read Tag` (service 0x4C) et `Write Tag` (service 0x4D) sur des **tags
  nommés** (segment symbolique ANSI `0x91`).

## 4. Table de tags

| Tag | Type CIP | Accès | Grandeur | Écriture → commande |
|---|---|:--:|---|---|
| `Setpoint` | REAL (0x00CA) | R/W | consigne | `SetSetpoint` |
| `ProcessValue` | REAL | R | mesure | — |
| `Output` | REAL | R | sortie (%) | — |
| `ManualOutput` | REAL | R/W | sortie manuelle (%) | `SetManualOutput` |
| `Run` | BOOL (0x00C1) | R/W | marche | `SetRun` |
| `Auto` | BOOL | R/W | mode auto | `SetAuto` |
| `SetpointMin` | REAL | R | consigne min | — |
| `SetpointMax` | REAL | R | consigne max | — |
| `Kp` / `Ki` / `Kd` | REAL | R | gains PID | — |

Un tag connu en **lecture seule** écrit est **accepté** (statut CIP succès) mais sans
effet ; un **tag inconnu** renvoie le statut CIP `0x05` (*path destination unknown*).
Toute écriture pilotable est **clampée/assainie** par la simulation.

## 5. Exemple client

Avec un client EtherNet/IP (p. ex. `pycomm3`, `rseip`, `rust-ethernet-ip`) pointant
sur l'IP/port du serveur, les tags se lisent/écrivent par leur nom :

```python
from pycomm3 import CIPDriver  # ou LogixDriver selon l'outil
# Lire la mesure, écrire la consigne et démarrer la régulation :
#   read  Tag "ProcessValue" (REAL)
#   write Tag "Setpoint" = 80.0 (REAL)
#   write Tag "Run" = True (BOOL)
```

Le serveur répond aux services génériques Read/Write Tag adressés par segment
symbolique ANSI ; il n'expose pas d'arborescence d'objets CIP au-delà des tags
ci-dessus.
