# Référence PROFIBUS DP-V0 — Régulateur simulé (ORPD)

*🌍 **FR** · [EN](../en/reference_profibus.md) · [DE](../de/reference_profibus.md) · [ES](../es/reference_profibus.md) · [IT](../it/reference_profibus.md) · [PT](../pt/reference_profibus.md) · [NL](../nl/reference_profibus.md) · [PL](../pl/reference_profibus.md)*

> Crate : `mock_bin_ru_pbdp` · Exécutable : **ru_pbdp** · Protocole : **PROFIBUS DP-V0** (esclave série)

Ce document est la référence fonctionnelle du sous-ensemble PROFIBUS DP-V0
simulé. La **source de vérité technique** reste l'en-tête de
[`src/profibus.rs`](../../src/profibus.rs) (codec + machine à états) et de
[`src/map.rs`](../../src/map.rs) (blocs I/O) : toute divergence doit être
corrigée dans le code en priorité.

---

## ⚠️ 0. Portée et limites — lire avant tout usage

`ru_pbdp` implémente un **sous-ensemble éducatif** de DP-V0, **sans aucune
prétention de conformité binaire stricte** aux tables normatives (IEC 61158 /
EN 50170) au-delà des éléments les plus universellement documentés :

- **conformes** : délimiteurs de trame (`SD1`/`SD2`/`SD3`/`SD4`/`SC`/`ED`), FCS
  (somme modulo 256), numéros de SAP des services de paramétrage
  (`Slave_Diag` = 61, `Set_Prm` = 62, `Chk_Cfg` = 63).
- **conventions propres à ce simulateur, pas un profil GSD réel enregistré au
  PNO** (PROFIBUS & PROFINET International) : encodage exact des bits du champ
  `FC`, disposition précise des octets de diagnostic, disposition des blocs
  d'entrées/sorties (§3), identifiant `Ident_Number` (§4).
- **aucun timing de bus réel** n'est respecté : ni fenêtre de réponse
  (*slot time*, `Tsdr` min/max), ni jeton inter-maîtres, ni arbitrage
  multi-maître. Un ASIC dédié (SPC3/VPC3) ou une carte maître matérielle
  (Hilscher/Softing/Siemens CP) sont seuls capables de tenir ces contraintes au
  niveau bit.

**Conséquence directe : ce simulateur ne sera jamais reconnu par un vrai
maître PROFIBUS DP** (automate + carte maître). Il sert à comprendre la
structure du protocole et à tester un développement logiciel (codec, machine à
états, outillage), pas à piloter du matériel de terrain — voir
[`manuel_utilisateur.md`](manuel_utilisateur.md).

---

## 1. Trames — délimiteurs et FCS

| Délimiteur | Valeur | Usage |
|---|:--:|---|
| `SD1` | `0x10` | Requête fixe sans données (6 octets : `SD1 DA SA FC FCS ED`) |
| `SD2` | `0x68` | Trame à données de longueur variable (`SD2 LE LEr SD2 DA SA FC [data…] FCS ED`) |
| `SD3` | `0xA2` | Trame à données fixes, 8 octets (14 octets au total) — **non utilisée** par ce simulateur (voir §0), fournie pour la complétude du codec et ses tests |
| `SD4` | `0xDC` | Trame jeton, 3 octets, sans FCS ni ED — hors sujet pour un esclave mono-maître simulé, fournie pour la complétude du codec |
| `SC` | `0xE5` | Accusé de réception court, 1 octet |
| `ED` | `0x16` | Délimiteur de fin |

- **`FCS`** : somme modulo 256 des octets utiles de la trame (voir
  `profibus::checksum`). Une trame reçue avec un FCS incorrect est rejetée
  (`FrameError::BadChecksum`) sans réponse — le maître doit retransmettre.
- **`DA`/`SA`** : adresse destination / source. Bit 7 de `DA` = **extension
  d'adresse (DAE)** : présence d'un octet de SAP juste après `DA` dans la
  charge utile. Absent = échange de données par défaut (`Data_Exchange`).
  L'adresse de station occupe les 7 bits restants (`0`-`125` ; `126`/`127`
  réservées par la norme, non utilisées ici).
- **Ce simulateur privilégie systématiquement `SD2`** pour tous les échanges
  `Data_Exchange`, y compris quand `SD3` (8 octets fixes) suffirait dans un
  vrai profil GSD — choix qui simplifie le codec sans rien perdre en
  couverture des concepts protocolaires (voir
  [`conception.md`](conception.md) §4).
- **Trame mal formée / délimiteur inconnu (bruit de ligne)** : rejetée
  silencieusement (`log::debug!`), la session continue — permet de
  resynchroniser sur le flux d'octets sans planter la liaison.

---

## 2. Séquencement — services et machine à états

L'esclave simulé (`SlaveFsm`, [`profibus.rs`](../../src/profibus.rs)) traverse
quatre états :

```
PowerOn ──Slave_Diag──► WaitPrm ──Set_Prm (ident OK)──► WaitCfg ──Chk_Cfg (tailles OK)──► DataExchange
```

| État | Signification | Réponse type |
|---|---|---|
| `Power_On` | Juste après démarrage, avant la première interrogation | — |
| `Wait_Prm` | En attente d'un `Set_Prm` valide | `Diag` avec `Stat_1 = STAT1_PRM_REQ` |
| `Wait_Cfg` | Paramétré, en attente d'un `Chk_Cfg` valide | `Diag` avec `Stat_1 = STAT1_CFG_FAULT` |
| `Data_Exchange` | Paramétré et configuré : échange cyclique actif | bloc d'entrée (§3) |

### `Slave_Diag` (SAP 61)

Requête sans donnée (ou trame `SD1`, toujours interprétée comme `Slave_Diag`
par convention de ce simulateur — aucune extension d'adresse possible sur
`SD1`, faute d'octet disponible pour porter un SAP). Réponse `Diag` (6 octets) :

| Octet | Symbole | Contenu |
|:--:|---|---|
| `0` | `Stat_1` | `0x01` (`STAT1_PRM_REQ`, tant que non paramétré) ou `0x02` (`STAT1_CFG_FAULT`, tant que non configuré) ou `0x00` (`Data_Exchange`) |
| `1` | `Stat_2` | toujours `0x00` (non simulé) |
| `2` | `Stat_3` | toujours `0x00` (non simulé) |
| `3` | `Master_Add` | `0xFF` (aucun maître connu — non tracé par ce simulateur) |
| `4-5` | `Ident_Number` | identifiant figé de l'esclave, big-endian (§4) |

Le premier `Slave_Diag` reçu fait passer `Power_On` → `Wait_Prm` ; les suivants
ne changent pas l'état (juste une lecture de diagnostic).

### `Set_Prm` (SAP 62)

Requête : `SAP(62) Ident_Number(2, BE) WD_Fact_1(1) WD_Fact_2(1)`. Le chien de
garde annoncé, s'il est présent, se calcule `watchdog_ms = WD_Fact_1 ×
WD_Fact_2 × 10` (unité 10 ms, convention DP standard) ; `WD_Fact_1 = 0` **ou**
`WD_Fact_2 = 0` signifie « pas de chien de garde ». Réponse : `ShortAck` (`SC`)
dans tous les cas.

- Si `Ident_Number` **correspond** au profil figé de l'esclave (§4) : état →
  `Wait_Cfg`, et le chien de garde éventuel est transmis à la session
  (armé seulement si le réglage local `watchdog_enabled` l'autorise — voir
  [`manuel_utilisateur.md`](manuel_utilisateur.md) §4).
- Si l'identifiant **ne correspond pas** : l'esclave reste (ou retourne) en
  `Wait_Cfg` → non, reste en `Wait_Prm` — le paramétrage est refusé
  silencieusement (`ShortAck` renvoyé quand même, comme le prescrit DP-V0 pour
  ce service, mais sans effet sur l'état interne).

### `Chk_Cfg` (SAP 63)

Requête : `SAP(63) Out_Len(1) In_Len(1)`. Réponse : `ShortAck`. L'état passe à
`Data_Exchange` **seulement si** `Out_Len == 45` et `In_Len == 17` (tailles
figées du profil simulé, §3) **et** l'esclave était en `Wait_Cfg` ; sinon
l'état ne change pas (le maître doit retransmettre un `Chk_Cfg` correct).

### `Data_Exchange` (pas de SAP — adresse par défaut, bit DAE absent)

Requête : le bloc de sortie brut (45 octets, §3). Réponse : le bloc d'entrée
(17 octets, §3), recalculé à la volée depuis l'instantané partagé au moment de
répondre (pas de table mémoire persistante, contrairement à Modbus/ORME).

Si le maître envoie un `Data_Exchange` **avant** d'avoir atteint l'état
`Data_Exchange` (séquencement non respecté), l'esclave répond par le
diagnostic courant (`Diag`) plutôt que de planter ou d'ignorer la trame.

---

## 3. Blocs I/O — disposition des octets

Recopié depuis l'en-tête de [`map.rs`](../../src/map.rs), seule source de
vérité en cas de divergence. Tous les flottants (`f32`) occupent **4 octets
consécutifs, big-endian**.

### Bloc de sortie — *Output* (maître → esclave, `OUTPUT_LEN` = 45 octets)

| Octet(s) | Symbole | Type | Description |
|---|---|:--:|---|
| `0` | `OUT_MODE` | bits | bit0 = marche, bit1 = auto, [3:2] = mode sens 1, [5:4] = mode sens 2 |
| `1-4` | `OUT_SP_AUTO` | f32 | Consigne automatique |
| `5-8` | `OUT_SP_MANUAL` | f32 | Consigne manuelle (% sortie, signée) |
| `9-12` | `OUT_KP1` | f32 | Gain proportionnel Kp sens 1 |
| `13-16` | `OUT_KI1` | f32 | Gain intégral Ki sens 1 |
| `17-20` | `OUT_KD1` | f32 | Gain dérivé Kd sens 1 |
| `21-24` | `OUT_KP2` | f32 | Gain proportionnel Kp sens 2 |
| `25-28` | `OUT_KI2` | f32 | Gain intégral Ki sens 2 |
| `29-32` | `OUT_KD2` | f32 | Gain dérivé Kd sens 2 |
| `33-36` | `OUT_HYSTERESIS` | f32 | Hystérésis des régulateurs TOR |
| `37-40` | `OUT_TOR_MIN_CYCLE` | f32 | Temps de cycle minimal TOR (s) |
| `41-44` | `OUT_PWM_PERIOD` | f32 | Période du cycle de modulation PWM (s) |

Les codes de mode (`[3:2]`/`[5:4]`) suivent `ControllerKind` : `0` = Off,
`1` = PID, `2` = TOR, `3` = PWM (voir `mock_lib_control`).

### Bloc d'entrée — *Input* (esclave → maître, `INPUT_LEN` = 17 octets)

| Octet(s) | Symbole | Type | Description |
|---|---|:--:|---|
| `0` | `IN_STATUS` | bits | bit0 = en marche, bit1 = sens 1 actif (sortie > 0), bit2 = sens 2 actif (sortie < 0) |
| `1-4` | `IN_PV` | f32 | Mesure / *process value* |
| `5-8` | `IN_OUTPUT` | f32 | Sortie appliquée (% signé) |
| `9-12` | `IN_SP_AUTO` | f32 | Recopie (lecture seule) de la consigne automatique |
| `13-16` | `IN_SP_MANUAL` | f32 | Recopie (lecture seule) de la consigne manuelle |

Un bloc de sortie **trop court** (< 45 octets) est ignoré sans panique : aucune
`Command` n'est produite, le régulateur conserve son dernier état valide.

---

## 4. Profil figé de l'esclave

| Paramètre | Valeur | Remarque |
|---|---|---|
| `Ident_Number` | `0xEE01` | **Fictif**, non enregistré auprès du PNO — ne représente aucun appareil catalogue réel |
| `Out_Len` | `45` | Attendu dans `Chk_Cfg.out_len` |
| `In_Len` | `17` | Attendu dans `Chk_Cfg.in_len` |
| Adresse de station | `0`-`125`, configurable | Réglage local (modal *Paramètres*), voir [`manuel_utilisateur.md`](manuel_utilisateur.md) §4 |
| Format de trame série | `8E1` (8 bits, parité paire, 1 stop) | **Fixé par la norme PROFIBUS DP**, non réglable |
| Débits normalisés | `9600` à `12 000 000` bit/s | Non contrôlé à l'ouverture : une valeur non standard est transmise telle quelle au port série |

---

## 5. Chien de garde protocolaire

Contrairement au chien de garde NAMUR d'OSNE (ajout maison), celui-ci est une
**vraie partie du protocole DP** : il est **annoncé par le maître** dans
`Set_Prm` (facteurs `WD_Fact_1`/`WD_Fact_2`, §2) et n'est **armé côté esclave**
que si le réglage local `watchdog_enabled` l'autorise (sinon la demande du
maître est ignorée, jamais armée). À l'échéance, sans nouvelle trame reçue pour
la station, l'esclave force l'état sûr (`Command::SetOnOff(false)`) —
simplification documentée : un vrai profil DP-V0 pourrait exiger un retour
complet par `Set_Prm`/`Chk_Cfg` avant de reprendre l'échange, ce que ce
simulateur ne redemande pas explicitement (il suffit de reprendre l'envoi de
trames `Data_Exchange`, l'état `Data_Exchange` n'étant pas quitté par
l'expiration du chien de garde).

---

## 6. Non-interopérabilité — pourquoi

| Exigence PROFIBUS DP réel | Ce simulateur |
|---|---|
| Fenêtre de réponse (*slot time*, `Tsdr` min/max) au niveau bit | Absente — répond dès que la trame est décodée, sans contrainte de temps |
| Circuit dédié (ASIC SPC3/VPC3) pour le timing | Absent — logiciel Tokio ordinaire |
| Jeton inter-maîtres, arbitrage multi-maître | Absent — esclave mono-maître, liaison point-à-point |
| Profil GSD enregistré au PNO | Absent — profil I/O propre à ce simulateur (§3) |
| Encodage bit-exact des champs FC/diagnostic | Convention de simulation, non garantie conforme |

**Un automate réel (Siemens S7 + carte maître, par exemple) ne reconnaîtra
jamais ce simulateur comme esclave valide sur un vrai bus RS-485 PROFIBUS DP.**
Deux instances de ce simulateur (ou un script rejouant la séquence ci-dessous)
peuvent en revanche dialoguer entre elles pour illustrer le protocole — voir
[`manuel_utilisateur.md`](manuel_utilisateur.md) §5.

---

## 7. Exemple de séquence (hexadécimal)

Séquence complète station `5`, maître `3`, jusqu'à l'échange cyclique
(valeurs illustratives, `FCS` calculé sur les octets utiles) :

```text
# 1. Slave_Diag (SD2, DAE=1, SAP=61)
→ TX  68 03 03 68 85 03 C0 3D FC 16
← RX  68 06 06 68 03 85 00 01 00 00 FF EE 01 F5 16   (Diag : Stat_1=0x01, Ident=0xEE01)

# 2. Set_Prm (SD2, DAE=1, SAP=62, Ident=0xEE01, WD=1×30×10ms=300ms)
→ TX  68 07 07 68 85 03 C0 3E EE 01 01 1E … 16
← RX  E5                                              (ShortAck)

# 3. Chk_Cfg (SD2, DAE=1, SAP=63, out_len=45, in_len=17)
→ TX  68 05 05 68 85 03 C0 3F 2D 11 … 16
← RX  E5                                              (ShortAck)

# 4. Data_Exchange (SD2, pas de SAP, bloc de sortie 45 octets)
→ TX  68 30 30 68 05 03 C0 [45 octets] … 16
← RX  68 14 14 68 03 85 00 [17 octets]  … 16          (bloc d'entrée)
```

Les octets exacts de FCS/longueur dépendent des valeurs de charge utile ; ce
schéma illustre l'**ordre des services**, pas une trame à rejouer telle quelle.
Voir les tests de [`profibus.rs`](../../src/profibus.rs) et
[`profibus_server.rs`](../../src/profibus_server.rs) pour des séquences
vérifiées bit à bit.
