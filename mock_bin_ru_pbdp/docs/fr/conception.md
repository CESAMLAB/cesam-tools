# Conception — Régulateur PROFIBUS DP simulé (ORPD)

*🌍 **FR** · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

> Crate : `mock_bin_ru_pbdp` · Exécutable : **ru_pbdp** (*Regulation Unit over PROFIBUS DP*)

Document d'architecture et de modélisation. Calqué sur le régulateur **ORME**
(`mock_bin_ru_modbus`) pour le modèle métier et les acteurs, et sur **OSNE**
(`mock_bin_su_namur`) pour la liaison série. Seule la **couche protocole**
change : un **simulateur logiciel de trames PROFIBUS DP-V0**, développé from
scratch (aucune crate `profibus`/`profibus-dp` publiée n'existe dans
l'écosystème Rust à ce jour).

---

## 1. Objet

Simuler un **régulateur de procédé** (boucle PID sur un procédé thermique du
premier ordre, modèle **identique** à ORME) et l'exposer via une **structure de
trames PROFIBUS DP-V0** sur liaison série (RS-485/RS-232).

**Ce document assume que le lecteur a lu l'avertissement de non-interopérabilité**
(voir [`manuel_utilisateur.md`](manuel_utilisateur.md) et
[`reference_profibus.md`](reference_profibus.md) §6) : PROFIBUS DP réel exige un
respect de timing bus au niveau bit (slot time, `Tsdr` min/max, chien de garde en
dizaines de ms) que seul un ASIC dédié (SPC3/VPC3) peut garantir. Ce simulateur
n'y prétend pas — c'est un outil pédagogique et de test logiciel, pas un pilote
de bus.

---

## 2. Modèle physique ([`regulator.rs`](../../src/regulator.rs))

Repris à l'identique du régulateur ORME : [`mock_lib_control::FirstOrderProcess`]
(fonction de transfert du premier ordre avec retard pur) et
[`mock_lib_control::Pid`] (PID à anti-emballement), avec les mêmes modes
(Off/PID/TOR/PWM) sur deux sens (chaud/froid). Pas de simulation : **50 ms**.
Toutes les écritures sont **assainies** dans `Regulator::apply` (bornes
réordonnées, flottants non finis ignorés, gains PID clampés) — même invariant que
partout ailleurs dans le workspace : jamais de `f32::clamp` avec des bornes non
validées.

---

## 3. Architecture (acteurs)

```
IHM (egui) ──Command(cast)──►  SimulationActor  ──refresh──► SharedSnapshot ──► IHM
Maître PROFIBUS (simulé) ────►  (Regulator)      ──refresh──► SharedSnapshot ──► réponses Data_Exchange
```

- **`SimulationActor`** ([`actors/simulation.rs`](../../src/actors/simulation.rs)) :
  identique en forme à celui d'ORME/OSNE — propriétaire unique du `Regulator`,
  timer one-shot ré-armé, publication du `SharedSnapshot` à chaque pas.
- **`ProfibusServerActor`** ([`actors/network.rs`](../../src/actors/network.rs)) :
  possède la liaison série ; `Reconfigure` referme/rouvre le port si le
  port/débit/adresse de station change ; conserve le `JoinHandle` de la session
  (abandon à l'arrêt) ; publie l'état de la liaison (`ServerStatus`, incluant
  l'état courant de la machine DP-V0) pour l'IHM.
- **[`profibus.rs`](../../src/profibus.rs)** — **source de vérité** du protocole :
  codec des trames (`SD1`/`SD2`/`SD3`/`SD4`/`SC`, FCS), décodage des services
  (`Slave_Diag`/`Set_Prm`/`Chk_Cfg`/`Data_Exchange`) et machine à états de
  l'esclave `SlaveFsm` (`PowerOn → WaitPrm → WaitCfg → DataExchange`).
- **[`map.rs`](../../src/map.rs)** — conversion des blocs d'octets I/O
  `Data_Exchange` vers/depuis les `Command` du régulateur (voir
  [`reference_profibus.md`](reference_profibus.md) §3).
- **[`profibus_server.rs`](../../src/profibus_server.rs)** — boucle de session sur
  un flux `AsyncRead + AsyncWrite` quelconque (le port série en production, un
  `tokio::io::duplex` en test) : lit une trame, la décode, appelle
  `SlaveFsm::handle`, applique les `Command` résultantes, encode et renvoie la
  réponse. Gère aussi le **chien de garde protocolaire** (`tokio::select!` entre
  lecture de trame et délai, comme le chien de garde NAMUR d'OSNE — mais ici
  c'est une **vraie partie du protocole DP**, armée par `Set_Prm`, pas un ajout
  maison).

Contrairement à Modbus (ORME, table mémoire séparée régénérée à chaque tick) et
à l'image d'OPC UA/NAMUR, il n'y a **pas de table mémoire persistante** : le bloc
d'entrée `Data_Exchange` est recalculé à la volée depuis le `SharedSnapshot` au
moment de répondre.

**Pas de politique multi-maître à gérer** : la liaison série *est* l'unique
maître (comme le RTU Modbus ou le port série NAMUR), contrairement au Modbus TCP
d'ORME (éviction) ou même au NAMUR TCP d'OSNE (point-à-point sans éviction).

---

## 4. Codec PROFIBUS DP-V0 — choix et limites assumées

- **Délimiteurs de trame** (`SD1=0x10`, `SD2=0x68`, `SD3=0xA2`, `SD4=0xDC`,
  `SC=0xE5`, `ED=0x16`) et **FCS** (somme modulo 256) : conformes à la norme,
  bien documentés publiquement.
- **SAP des services de paramétrage** (`Slave_Diag=61`, `Set_Prm=62`,
  `Chk_Cfg=63`) : conformes.
- **Encodage exact des bits du champ FC**, **disposition précise des octets de
  diagnostic**, et **disposition des blocs I/O** (`map.rs`) : ce sont des
  **conventions propres à ce simulateur**, pas un profil GSD enregistré au PNO.
  Le simulateur utilise systématiquement des trames **SD2** (longueur variable)
  pour `Data_Exchange`, y compris quand `SD3` (8 octets fixes) suffirait dans un
  vrai profil — choix qui simplifie le codec sans rien perdre en couverture des
  concepts protocolaires.
- **Identifiant PROFIBUS** (`Ident_Number = 0xEE01`) : **fictif**, non enregistré
  auprès du PNO (PROFIBUS & PROFINET International) — ne représente aucun
  appareil catalogue réel.
- **Aucun timing de bus** : ni fenêtre de réponse (`Tsdr`), ni jeton, ni
  arbitrage multi-maître ne sont implémentés — voir §1.

Détail complet dans [`reference_profibus.md`](reference_profibus.md).

---

## 5. Configuration & persistance

`AppConfig` (langue / liaison série / procédé / régulation / vérif. MAJ)
sérialisée en **TOML** ([`config.rs`](../../src/config.rs)), **assainie au
chargement** (`AppConfig::sanitized` : bornes ordonnées, `τ ≥ 1e-3`,
`dead_time ≥ 0`, flottants finis, adresse de station bornée `[0, 125]`). Fichier :
`mock_ru_pbdp.toml` (surchargeable par `MOCK_CONFIG`). Contrairement à
ORME/OSNE, **pas de liste blanche d'IP** (liaison série intrinsèquement
point-à-point, pas de notion d'adresse réseau).

---

## 6. Pistes d'évolution

- Un véritable outil de **maître PROFIBUS DP simulé** (binaire séparé), utilisant
  les mêmes fonctions d'encodage/décodage exposées en test dans `profibus.rs`,
  pour piloter ce simulateur ou tout autre esclave logiciel sans dépendre d'un
  script ad hoc.
- Génération d'un fichier **GSD** illustratif (non fonctionnel côté simulateur)
  documentant le profil I/O simulé, à titre pédagogique.
- Support de **DP-V1** (accès acyclique, alarmes) si le besoin pédagogique
  émerge — hors périmètre initial (DP-V0 seul).
- Promotion du modèle régulateur dans une `mock_lib_*` partagée (aujourd'hui
  dupliqué entre ORME et cet instrument, comme pour RU/OPC UA).
