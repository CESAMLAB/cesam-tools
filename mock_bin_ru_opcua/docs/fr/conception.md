# Conception — Régulateur de procédé simulé (RU/OPC UA)

*🌍 **FR** · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

> Crate : `mock_bin_ru_opcua` · Exécutable : **ru_opcua** (*Regulation Unit over OPC UA*)

Document d'architecture et de modélisation. Calqué sur le régulateur **ORME**
(`mock_bin_ru_modbustcp`) : même découpage **modèle métier synchrone / acteurs
ractor / couche protocole / IHM egui**, mêmes invariants. Seul le **transport**
change : **OPC UA** au lieu de Modbus.

---

## 1. Objet

Simuler un **régulateur de procédé** (boucle PID sur un procédé thermique du
premier ordre) et l'exposer via **OPC UA**, le standard de supervision
industrielle (Industrie 4.0). Contrairement à ORME (Modbus) et OSNE (NAMUR) —
protocoles **de terrain sans sécurité** — OPC UA porte nativement
l'authentification, la signature et le chiffrement (prévus en Phase 2).

---

## 2. Modèle physique ([`regulator.rs`](../../src/regulator.rs))

Le **procédé** réutilise [`mock_lib_control::FirstOrderProcess`] (partagé avec
ORME) : fonction de transfert du premier ordre avec retard pur

```text
PV(s) / U(s) = K · e^(−L·s) / (1 + τ·s)
```

- `PV` : mesure (unité procédé, p. ex. °C) ;
- `U` : commande / sortie (0-100 %) ;
- `K` : gain statique ; `τ` : constante de temps ; `L` : retard pur ;
- `ambient` : valeur au repos (sortie nulle).

Un **PID** ([`mock_lib_control::Pid`], lui aussi réutilisé d'ORME) asservit la
mesure vers la **consigne** en pilotant la sortie, bornée à `[0, 100]`. Deux modes :
**automatique** (le PID calcule la sortie) et **manuel** (sortie imposée). Le pas
de simulation est de **0,5 s** (procédé thermique lent).

Toutes les écritures (réseau ou IHM) sont **assainies** dans `Regulator::apply` :
flottants non finis ignorés, consigne bornée, bornes réordonnées (`min ≤ max`),
gains PID clampés. **Invariant : jamais de `f32::clamp` avec des bornes non
validées** (panic si `min > max` ou `NaN`).

---

## 3. Architecture (acteurs)

```
IHM (egui) ───Command(cast)──►  SimulationActor ──refresh──► SharedSnapshot ──► IHM
Serveur OPC UA ─Command(cast)─►   (Regulator)    ──refresh──► SharedSnapshot ──► lectures OPC UA
```

- **`SimulationActor`** ([`actors/simulation.rs`](../../src/actors/simulation.rs)) :
  propriétaire **unique** du `Regulator` ; avance la simulation sur un timer
  one-shot ré-armé (pas de timer détaché) et publie un `SharedSnapshot` à chaque
  pas.
- **`OpcuaServerActor`** ([`actors/network.rs`](../../src/actors/network.rs)) :
  possède le serveur OPC UA (tâche tokio `server.run()`) ; relançable à chaud
  (`Reconfigure` : rebind si l'IP/port change) ; conserve le `JoinHandle` (abandon
  à l'arrêt) et le `ServerHandle` (annulation propre des sessions) ; publie son
  statut d'écoute pour l'IHM.
- **Serveur OPC UA** ([`opcua_server.rs`](../../src/opcua_server.rs)) : construit le
  serveur [`async-opcua`](https://crates.io/crates/async-opcua), déclare l'espace
  d'adressage et branche les callbacks. Les **lectures** puisent dans le
  `SharedSnapshot` ; les **écritures** émettent une `Command` vers le
  `SimulationActor` par `cast` non bloquant.

Comme NAMUR (OSNE) et contrairement au Modbus d'ORME, il n'y a **pas de table
mémoire séparée** : les nœuds OPC UA lisent directement l'instantané partagé.

---

## 4. Pile OPC UA — choix techniques

- **`async-opcua`** (serveur, feature `server`) : implémentation **tokio-native**
  (une tâche par connexion), qui s'imbrique dans la stack ractor/tokio. Crypto
  **100 % Rust** (RustCrypto : `rsa`, `aes`, `sha2`, `x509-cert`) — **aucune
  dépendance OpenSSL**, ce qui préserve la cross-compilation (Linux/Windows/RPi).
- **Espace d'adressage** : un `SimpleNodeManager` en mémoire ; nœuds `Variable`
  organisés sous `Objects` (cf. [`reference_opcua.md`](reference_opcua.md)).
- **Callbacks** : `add_read_callback` (valeur vivante, échantillonnée pour les
  abonnements) et `add_write_callback` (route vers la simulation).
- **Licence** : `async-opcua` est sous **MPL-2.0** (toute la lignée OPC UA en Rust
  l'est). Copyleft **par fichier** : usage non modifié → le code CESAM-Lab reste
  MIT (cf. fichier `NOTICE` à la racine).

---

## 5. Sécurité

La sécurité est **réglable** (`SecurityConfig`) et constitue le différenciateur
d'OPC UA face aux protocoles de terrain (Modbus/NAMUR, sans sécurité).

- **Mode non chiffré (défaut)** : un endpoint `SecurityPolicy::None`, jeton
  **anonyme** — réseau de confiance uniquement, démarrage instantané, aucun
  certificat. L'IHM affiche un **bandeau orange** d'avertissement.
- **Mode chiffré (Phase 2)** : endpoint `Basic256Sha256` / `SignAndEncrypt`. Un
  **certificat d'instance** auto-signé est généré au premier lancement (`pki/`) ;
  le serveur fait confiance aux certificats clients. **Authentification** par
  utilisateur/mot de passe (`ServerUserToken::user_pass`) et/ou anonyme. L'IHM
  affiche un **bandeau vert** 🔒.

Le mode se règle dans le modal *Paramètres* ; un changement **relance** le serveur
à chaud (`OpcuaServerActor`).

---

## 6. Configuration & persistance

`AppConfig` (langue / réseau / procédé / régulation / vérif. MAJ) sérialisée en
**TOML** ([`config.rs`](../../src/config.rs)), **assainie au chargement**
(`AppConfig::sanitized` : bornes ordonnées, `τ ≥ 1e-3`, `dead_time ≥ 0`, flottants
finis). Fichier : `mock_ru_opcua.toml` (surchargeable par `MOCK_CONFIG`).

---

## 7. Pistes d'évolution

- **Phase 2** : sécurité OPC UA (certificats, chiffrement, auth).
- Méthodes OPC UA (`Reset`, `Autotune`) en plus des variables.
- Modèle d'information typé (ObjectType régulateur) plutôt que des variables à plat.
- Historisation / `HistoryRead` sur la mesure.
- Promotion du modèle régulateur d'ORME dans une `mock_lib_*` partagée (il est
  aujourd'hui dupliqué entre ORME et cet instrument).
