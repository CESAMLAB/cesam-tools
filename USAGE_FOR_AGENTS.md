# Guide d'utilisation — pour un agent Claude d'un autre projet

Ce document explique comment **utiliser** les simulateurs d'instruments de ce
workspace (`cesam-tools`) comme *mocks* de test depuis un **autre projet**
(ex. un logiciel de supervision, un pilote de protocole, une suite de tests
d'intégration). Il ne couvre pas l'architecture interne — voir
[`CLAUDE.md`](CLAUDE.md) pour ça.

Sept instruments, chacun un **binaire autonome** qui simule un régulateur (ou
agitateur) et l'expose sur un protocole industriel réel :

| Instrument | Marque | Crate | Binaire | Protocole | Connexion par défaut |
|---|---|---|---|---|---|
| Régulateur Modbus | **ORME** | `mock_bin_ru_modbus` | `orme` | Modbus TCP (+ RTU série en option) | TCP `0.0.0.0:5502` |
| Agitateur NAMUR | **OSNE** | `mock_bin_su_namur` | `osne` | NAMUR (ASCII, TCP ou série) | TCP `0.0.0.0:4001` |
| Régulateur OPC UA | **ORUE** | `mock_bin_ru_opcua` | `ru_opcua` | OPC UA | `opc.tcp://0.0.0.0:4840/` |
| Régulateur Sparkplug B | **ORSE** | `mock_bin_ru_sparkplugb` | `ru_spb` | MQTT Sparkplug B (edge node **sortant**) | se connecte à `localhost:1883`, groupe `CESAM`, nœud `RU1` |
| Régulateur S7 | **ORSS** | `mock_bin_ru_s7` | `ru_s7` | S7comm (ISO-on-TCP / RFC1006) | TCP `0.0.0.0:102` |
| Régulateur EtherNet/IP | **OREE** | `mock_bin_ru_ethernetip` | `ru_eip` | EtherNet/IP (CIP messagerie explicite) | TCP `0.0.0.0:44818` |
| Régulateur PROFIBUS DP | **ORPD** | `mock_bin_ru_pbdp` | `ru_pbdp` | PROFIBUS DP-V0 **simulé** (série) | `/dev/ttyUSB0` (Linux) ou `COM3` (Windows) |

⚠️ **ORPD n'est pas interopérable avec du matériel PROFIBUS DP réel** (aucun
timing de bus respecté) — utile uniquement pour tester un maître logiciel qui
parle DP-V0 sur une liaison série virtuelle (ex. `socat` entre deux ptys).

---

## 1. Obtenir un binaire

Deux options :

### a) Build headless depuis les sources (recommandé pour CI/agent)

```bash
cargo build -p <crate> --release --no-default-features
# binaire produit dans target/release/<binaire>
```

`--no-default-features` retire l'IHM (`egui`) : le binaire ne fait plus que
tourner la simulation + le serveur protocolaire, sans fenêtre — c'est ce qu'il
faut pour un environnement de test automatisé. Aucune dépendance graphique
n'est tirée. Exception : ORPD garde toujours la liaison série (ce n'est pas
une feature optionnelle chez lui), `--no-default-features` retire seulement
son IHM.

### b) Image Docker headless préconstruite

Si `scripts/build-prod.sh` a été exécuté sur ce dépôt, chaque instrument a une
image `<binaire>:headless` (ex. `orme:headless`, `ru_opcua:headless`).

```bash
docker run --rm -p 5502:5502 orme:headless
```

Sparkplug B (ORSE, client sortant) et PROFIBUS DP (ORPD, liaison série) n'ont
pas de port à publier ; pour ORPD, passer le périphérique série au conteneur :
`--device=/dev/ttyUSB0`.

---

## 2. Configurer sans IHM

Chaque instrument charge sa config depuis un fichier **TOML**, dont le chemin
est surchargeable par la variable d'environnement `MOCK_CONFIG` :

```bash
MOCK_CONFIG=/chemin/vers/ma_config.toml ./orme
```

Points importants :

- **Fichier absent → valeurs par défaut** (aucune erreur, aucun fichier créé
  automatiquement). Pour du test « comportement par défaut », il n'y a rien à
  configurer.
- **TOML partiel accepté** : chaque table a `#[serde(default)]`, donc un
  fichier qui ne contient que ce qu'on veut changer suffit (pas besoin de
  recopier tout le schéma). Exemple minimal pour changer juste le port
  d'ORME :
  ```toml
  [network]
  port = 5503
  ```
- **Valeurs assainies au chargement** : bornes réordonnées, flottants non
  finis remplacés par les défauts, etc. — un TOML malformé sur le plan des
  valeurs (mais syntaxiquement TOML valide) ne fait jamais planter le
  binaire, il retombe sur des valeurs saines et logue un `warn`.
- **Pas de hot-reload en headless** : la configuration se lit **au démarrage**
  uniquement (le rechargement à chaud passe par le bouton *Appliquer* de
  l'IHM, absent en headless). Pour changer un paramètre, éditer le TOML puis
  redémarrer le processus.
- Pour connaître le nom exact des clés TOML au-delà des extraits ci-dessous,
  la référence est le fichier `<crate>/src/config.rs` (struct `AppConfig`) —
  ou lancer une fois l'instrument **avec IHM**, régler dans le modal
  *Paramètres*, cliquer *💾 Sauvegarder*, et récupérer le fichier généré.

---

## 3. Détail par instrument

### ORME — Modbus TCP/RTU

```toml
[network]
bind_ip = "0.0.0.0"
port = 5502
allowlist = []   # motifs IP avec jokers "*" par octet, ex. ["192.168.1.*"]
```
- **Un seul maître Modbus servi à la fois** : une nouvelle connexion TCP
  **évince** la précédente (déconnexion immédiate). Un test qui ouvre deux
  connexions concurrentes verra la première coupée.
- Table d'adresses complète (registres, encodage `f32` sur 2 registres
  big-endian, codes fonction) : [`mock_bin_ru_modbus/docs/fr/table_modbus.md`](mock_bin_ru_modbus/docs/fr/table_modbus.md).

### OSNE — NAMUR (ASCII, TCP ou série)

```toml
[network]
bind_ip = "0.0.0.0"
port = 4001
allowlist = []
```
- Protocole en **lignes ASCII** (ex. `IN_SP_4\r\n` → réponse `"500.0 4"`).
- ⚠️ **Pas d'éviction** : contrairement à ORME, un premier client connecté
  bloque les suivants tant qu'il ne se déconnecte pas.
- Jeu de commandes complet : [`mock_bin_su_namur/docs/fr/commandes_namur.md`](mock_bin_su_namur/docs/fr/commandes_namur.md).

### ORUE — OPC UA

```toml
[network]
bind_ip = "0.0.0.0"
port = 4840

[security]
encryption = false   # true -> Basic256Sha256 SignAndEncrypt + certificat auto-signé
allow_anonymous = true
```
- Par défaut : endpoint `SecurityPolicy::None`, anonyme, `opc.tcp://<ip>:4840/`.
- Plusieurs sessions clientes **simultanées** acceptées (pas d'éviction).
- Espace d'adressage (nœuds, mapping) : [`mock_bin_ru_opcua/docs/fr/reference_opcua.md`](mock_bin_ru_opcua/docs/fr/reference_opcua.md).

### ORSE — MQTT Sparkplug B (client sortant)

```toml
[network]
broker_host = "localhost"
broker_port = 1883
client_id = "ru_spb"
group_id = "CESAM"
edge_node_id = "RU1"
```
- C'est l'instrument qui **se connecte** à un broker MQTT — il faut donc un
  broker (ex. Mosquitto) déjà accessible à l'adresse configurée avant de
  lancer ORSE.
- Topics/métriques (`NBIRTH`/`NDATA`/`NDEATH`, mapping `NCMD`) : [`mock_bin_ru_sparkplugb/docs/fr/reference_sparkplugb.md`](mock_bin_ru_sparkplugb/docs/fr/reference_sparkplugb.md).

### ORSS — S7comm (ISO-on-TCP)

```toml
[network]
bind_ip = "0.0.0.0"
port = 102   # < 1024 : peut nécessiter les droits root/CAP_NET_BIND_SERVICE
allowlist = []
```
- Trames TPKT/COTP + S7comm sur une image d'octets DB1 : [`mock_bin_ru_s7/docs/fr/reference_s7.md`](mock_bin_ru_s7/docs/fr/reference_s7.md).

### OREE — EtherNet/IP

```toml
[network]
bind_ip = "0.0.0.0"
port = 44818
allowlist = []
```
- CIP `Read Tag`/`Write Tag`, encodage **little-endian** : [`mock_bin_ru_ethernetip/docs/fr/reference_ethernetip.md`](mock_bin_ru_ethernetip/docs/fr/reference_ethernetip.md).

### ORPD — PROFIBUS DP-V0 (simulé, série)

```toml
[network.serial]
port = "/dev/ttyUSB0"   # ou "COM3" sous Windows
baud = 9600
station_address = 3
watchdog_enabled = false
```
- ⚠️ Simulateur logiciel uniquement, **aucun timing de bus PROFIBUS réel**
  n'est respecté — ne pas l'utiliser face à du matériel de terrain réel.
- Séquencement SAP, blocs I/O, chien de garde : [`mock_bin_ru_pbdp/docs/fr/reference_profibus.md`](mock_bin_ru_pbdp/docs/fr/reference_profibus.md).

---

## 4. Pour aller plus loin

Chaque instrument a un dossier `docs/fr/` (source de vérité) avec quatre
documents : `manuel_utilisateur.md`, `conception.md`, `maintenance.md` et une
référence protocolaire dédiée (table Modbus / commandes NAMUR / espace OPC UA
/ etc., listés ci-dessus). Traductions disponibles dans `docs/<en|de|es|it|pt|nl|pl>/`.

Pour la posture sécurité de chaque instrument (aucun n'a d'authentification
réseau native sauf ORUE en mode chiffré), voir la section « Sécurité &
robustesse » de [`CLAUDE.md`](CLAUDE.md) — pertinent si le projet consommateur
doit décider où exposer ces mocks (réseau de confiance uniquement).
