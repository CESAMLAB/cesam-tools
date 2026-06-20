# Table d'adresses Modbus — Régulateur simulé

*🌍 **FR** · [EN](../en/table_modbus.md) · [DE](../de/table_modbus.md) · [ES](../es/table_modbus.md) · [IT](../it/table_modbus.md) · [PT](../pt/table_modbus.md) · [NL](../nl/table_modbus.md) · [PL](../pl/table_modbus.md)*

> Crate : `mock_bin_ru_modbustcp` · Protocole : **Modbus TCP** (esclave / serveur)

Ce document est la référence fonctionnelle du plan d'adressage. La **source de
vérité technique** reste l'en-tête de [`src/map.rs`](../../src/map.rs) : toute
divergence doit être corrigée dans le code en priorité.

---

## 1. Généralités

| Élément | Valeur |
|---------|--------|
| Transport | Modbus **TCP** ou **RTU série / RS485** (un seul actif à la fois) |
| Rôle | **Esclave** (serveur) |
| Port par défaut | TCP `5502` (configurable, modal *Paramètres*) |
| Série (RTU) | port + baud + parité + bits, configurables (feature `rtu`) |
| Unit ID / adresse | TCP : indifférent. RTU : `slave_id` configurable mais **non filtré** (voir note) |
| Maîtres | **un seul maître distant à la fois** ; en TCP un nouveau venu déconnecte le précédent (l'IHM locale n'est pas un maître) |
| Adressage | **base 0** (l'adresse `0` = 1ᵉʳ élément de la table) |
| Filtrage | liste blanche d'IP optionnelle (jokers `*`, TCP uniquement) |

> **Note RTU / adresse esclave** : le serveur RTU répond **quelle que soit
> l'adresse** demandée (l'adresse n'est pas transmise au service applicatif).
> Utiliser une **liaison point-à-point**. Le `slave_id` est conservé/affiché mais
> n'effectue pas de filtrage.

### Adressage base 0 vs base 1

Les adresses ci-dessous sont les **adresses protocolaires (base 0)**, telles
qu'envoyées dans la trame. Beaucoup d'outils affichent une numérotation base 1
« conventionnelle » (`4xxxx` pour les holdings, `3xxxx` pour les inputs…). Ainsi
le registre de maintien d'adresse `2` correspond au repère conventionnel `40003`.

---

## 2. Encodage des nombres flottants (`f32`)

Les grandeurs analogiques sont des **`f32` IEE-754 sur 2 registres consécutifs** :

- **ordre des mots** : mot de **poids fort en premier** (big-endian, dit *ABCD*) ;
- **ordre des octets** dans chaque registre : big-endian (standard Modbus).

Exemple : `80.0` → octets `42 A0 00 00` → registre `n` = `0x42A0`,
registre `n+1` = `0x0000`.

> Si votre client lit des valeurs aberrantes, c'est presque toujours un problème
> d'ordre des mots (essayer *word swap* / *CDAB*).

---

## 3. Bobines — *Coils* (lecture/écriture)

Codes fonction : `0x01` (lecture), `0x05` (écriture simple), `0x0F` (écriture multiple).

| Adresse | Désignation | Valeurs | Effet |
|---------|-------------|---------|-------|
| `0` | Marche / Arrêt | `0` = arrêt, `1` = marche | Active la régulation |
| `1` | Auto / Manuel | `0` = manuel, `1` = auto | Choix du mode |

---

## 4. Entrées discrètes — *Discrete Inputs* (lecture seule)

Code fonction : `0x02`.

| Adresse | Désignation | Signification |
|---------|-------------|---------------|
| `0` | En marche | L'appareil est en marche |
| `1` | Sens 1 (chaud) actif | Sortie > 0 |
| `2` | Sens 2 (froid) actif | Sortie < 0 |

---

## 5. Registres de maintien — *Holding Registers* (lecture/écriture)

Codes fonction : `0x03` (lecture), `0x06` (écriture simple), `0x10` (écriture multiple).

| Adresse | Désignation | Type | Unité / valeurs |
|---------|-------------|------|-----------------|
| `0` | Mode de régulation sens 1 (chaud) | `u16` | `0`=Off, `1`=PID, `2`=TOR, `3`=PWM |
| `1` | Mode de régulation sens 2 (froid) | `u16` | `0`=Off, `1`=PID, `2`=TOR, `3`=PWM |
| `2`–`3` | Consigne automatique (SP) | `f32` | unité de mesure |
| `4`–`5` | Consigne manuelle | `f32` | % de sortie, signée (−100…+100) |
| `6`–`7` | `Kp` sens 1 | `f32` | gain proportionnel |
| `8`–`9` | `Ki` sens 1 | `f32` | gain intégral (s⁻¹) |
| `10`–`11` | `Kd` sens 1 | `f32` | gain dérivé (s) |
| `12`–`13` | `Kp` sens 2 | `f32` | gain proportionnel |
| `14`–`15` | `Ki` sens 2 | `f32` | gain intégral (s⁻¹) |
| `16`–`17` | `Kd` sens 2 | `f32` | gain dérivé (s) |
| `18`–`19` | Hystérésis TOR | `f32` | unité de mesure |
| `20`–`21` | Temps de cycle minimal TOR | `f32` | secondes (anti-court-cycle, `0` = désactivé) |
| `22`–`23` | Période du cycle PWM | `f32` | secondes (> 0) |
| `42`–`46` | Identifiant appareil | `ASCII` | « CESAM-Lab » (lecture seule, 2 car./registre, poids fort d'abord) |

> Registres `24`–`41` réservés (lus à `0`).

> **Écriture partielle d'un `f32`** : il faut écrire **les deux registres** d'un
> flottant pour qu'il soit pris en compte. Une écriture d'un seul registre d'une
> paire `f32` est ignorée (et renvoie l'exception *Illegal Data Address* si elle
> ne recouvre aucun champ valide).
>
> Les gains écrits sont bornés à des valeurs finies ≥ 0 (robustesse).

---

## 6. Registres d'entrée — *Input Registers* (lecture seule)

Code fonction : `0x04`.

| Adresse | Désignation | Type | Unité |
|---------|-------------|------|-------|
| `0`–`1` | Mesure (PV — *process value*) | `f32` | unité de mesure |
| `2`–`3` | Sortie appliquée | `f32` | % signé (+ chaud / − froid) |

---

## 7. Exceptions Modbus

| Code | Nom | Cause dans cet appareil |
|------|-----|--------------------------|
| `0x01` | Illegal Function | Code fonction non géré (ex. masque, FIFO) |
| `0x02` | Illegal Data Address | Adresse / quantité hors table, ou écriture ne ciblant aucun champ |
| `0x04` | Server Device Failure | Verrou interne indisponible (cas anormal) |

---

## 8. Exemples avec `mbpoll`

`mbpoll` adresse en **base 1** ; on ajoute donc `1` aux adresses base 0.

```bash
# Mettre en marche (bobine base0 0 -> -t 0 -r 1) puis passer en auto (bobine 1)
mbpoll -m tcp -p 5502 -a 1 -t 0 -r 1 127.0.0.1 1     # On/Off = 1
mbpoll -m tcp -p 5502 -a 1 -t 0 -r 2 127.0.0.1 1     # Auto/Manuel = 1 (auto)

# Écrire la consigne auto (HR base0 2-3 -> -t 4:float -r 3) à 80.0
mbpoll -m tcp -p 5502 -a 1 -t 4:float -r 3 127.0.0.1 80.0

# Lire la mesure PV (IR base0 0-1 -> -t 3:float -r 1)
mbpoll -m tcp -p 5502 -a 1 -t 3:float -r 1 127.0.0.1

# Lire la sortie (IR base0 2-3 -> -t 3:float -r 3)
mbpoll -m tcp -p 5502 -a 1 -t 3:float -r 3 127.0.0.1
```

> Selon les versions de `mbpoll`, l'ordre des mots flottants peut nécessiter
> l'option de permutation. En cas de valeur incohérente, vérifier l'ordre des mots.

---

## 9. Carte mémoire condensée

```
Coils (RW)            DiscreteInputs (RO)     Holding (RW)              Input (RO)
0  On/Off             0  En marche            0  Mode sens1 (u16)       0-1 PV (f32)
1  Auto/Manuel        1  Chaud actif          1  Mode sens2 (u16)       2-3 Sortie (f32)
                      2  Froid actif          2-3  SP auto (f32)
                                              4-5  SP manuel (f32)
                                              6-7  Kp1  8-9  Ki1  10-11 Kd1
                                              12-13 Kp2 14-15 Ki2 16-17 Kd2
                                              18-19 Hystérésis (f32)
                                              20-21 Cycle min. TOR (f32, s)
                                              22-23 Période PWM (f32, s)
                                              42-46 Identifiant ASCII "CESAM-Lab"
```

> **Identifiant ASCII** (`HR 42-46`) : « CESAM-Lab » encodé 2 caractères par
> registre, caractère de poids fort d'abord (`42`=`'CE'`, `43`=`'SA'`, `44`=`'M-'`,
> `45`=`'La'`, `46`=`'b\0'`). Lecture seule. Exemple :
> `mbpoll -m tcp -p 5502 -a 1 -t 4 -r 43 -c 5 127.0.0.1` (registres base 1 43..47).
