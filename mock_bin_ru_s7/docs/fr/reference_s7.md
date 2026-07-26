# Référence S7 — plan d'adressage & protocole (RU/S7)

*🌍 **FR** · [EN](../en/reference_s7.md) · [DE](../de/reference_s7.md) · [ES](../es/reference_s7.md) · [IT](../it/reference_s7.md) · [PT](../pt/reference_s7.md) · [NL](../nl/reference_s7.md) · [PL](../pl/reference_s7.md)*

> Source de vérité : [`s7_server.rs`](../../src/s7_server.rs) (analyse des trames,
> plan d'adressage DB1, mapping des écritures). Toute évolution se fait **dans ce
> fichier** et se répercute ici.

---

## 1. Endpoint

Serveur **S7comm** sur **ISO-on-TCP / RFC1006**. Écoute par défaut sur
`0.0.0.0:102` (port standard S7 ; **< 1024 → droits root** requis, sinon choisir un
port haut). Réglages dans la section `[network]` du TOML / le modal *Paramètres* :

| Clé | Défaut | Rôle |
|---|---|---|
| `bind_ip` | `0.0.0.0` | IP d'écoute |
| `port` | `102` | port TCP (S7 standard) |
| `allowlist` | *(vide)* | liste blanche d'IP (motifs `*` par octet ; vide = tout autorisé) |

> ⚠️ **Aucune authentification ni chiffrement** (S7 « classic »). Le seul contrôle
> d'accès est la **liste blanche d'IP** + la topologie réseau. `0.0.0.0` + liste vide
> = **exposé à tout le réseau** : l'IHM affiche un bandeau d'avertissement.

## 2. Sessions

À l'inverse d'ORME (mono-maître), le serveur S7 accepte **plusieurs sessions
clientes simultanées** (comportement usuel d'un automate). Chaque session négocie
COTP (Connection Request → Confirm) puis S7 *Setup Communication*, avant les
échanges *Read Var* / *Write Var*.

## 3. Sous-ensemble protocole implémenté

- **COTP** : Connection Request (CR) → Connection Confirm (CC) ; Data (DT).
- **S7comm** : *Setup Communication*, *Read Var* (fonction `0x04`), *Write Var*
  (fonction `0x05`) sur le bloc de données **DB1**.

Le serveur expose une **image d'octets de DB1** (40 octets). Les lectures servent
une tranche de cette image ; les écritures sur les offsets pilotables produisent des
commandes assainies pour la simulation.

## 4. Plan d'adressage DB1

REAL = `f32` big-endian (IEEE-754). Adressage par octet (`DBDx`) ou par bit
(`DBXx.y`).

| Adresse | Type | Accès | Grandeur | Écriture → commande |
|---|---|:--:|---|---|
| `DB1.DBD0`  | REAL | R/W | Consigne (Setpoint) | `SetSetpoint` |
| `DB1.DBD4`  | REAL | R   | Mesure (ProcessValue) | — |
| `DB1.DBD8`  | REAL | R   | Sortie (Output, %) | — |
| `DB1.DBD12` | REAL | R/W | Sortie manuelle (ManualOutput, %) | `SetManualOutput` |
| `DB1.DBX16.0` | BOOL | R/W | Marche (Run) | `SetRun` |
| `DB1.DBX16.1` | BOOL | R/W | Mode auto (Auto) | `SetAuto` |
| `DB1.DBD20` | REAL | R | Consigne min | — |
| `DB1.DBD24` | REAL | R | Consigne max | — |
| `DB1.DBD28` | REAL | R | PID Kp | — |
| `DB1.DBD32` | REAL | R | PID Ki | — |
| `DB1.DBD36` | REAL | R | PID Kd | — |

Écriture de `DB1.DBB16` (octet) acceptée : bit 0 = Run, bit 1 = Auto. Toute écriture
sur un offset en lecture seule est **acceptée mais ignorée** (code retour succès).
Une lecture/écriture hors DB1 renvoie le code retour S7 `0x0A` (objet inexistant).

## 5. Exemple client

Avec un client S7 (Snap7, `python-snap7`, nodes7…) configuré sur l'IP/port du
serveur, **rack 0 / slot 1** (valeurs usuelles ; le serveur n'impose pas le TSAP) :

```python
import snap7, struct
c = snap7.client.Client()
c.connect("127.0.0.1", 0, 1, 102)
c.db_write(1, 0, struct.pack(">f", 80.0))   # Consigne = 80.0
c.db_write(1, 16, bytes([0x01]))            # Run = true (bit 0)
pv = struct.unpack(">f", c.db_read(1, 4, 4))[0]  # Mesure
```
