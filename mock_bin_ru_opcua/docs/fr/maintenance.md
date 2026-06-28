# Documentation de maintenance — RU/OPC UA (workspace `cesam-tools`)

*🌍 **FR** · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

> Crate : `mock_bin_ru_opcua` · Exécutable : **ru_opcua**

---

## 1. Prérequis

- **Rust** récent. ⚠️ MSRV propre à ce crate : **1.91** (`async-opcua` ne déclare
  aucun `rust-version` et tire des dépendances récentes ; le reste du workspace
  est à 1.85).
- Pour l'IHM : les dépendances système d'`eframe`/`egui` (mêmes que ORME/OSNE).
- Pour le build *headless* : aucune dépendance graphique.

---

## 2. Commandes courantes

```bash
cargo run -p mock_bin_ru_opcua                       # IHM + serveur OPC UA
cargo run -p mock_bin_ru_opcua --no-default-features # headless (sans IHM)
cargo test -p mock_bin_ru_opcua                      # tests unitaires
cargo clippy -p mock_bin_ru_opcua --all-targets      # lint
MOCK_CONFIG=./ma_config.toml cargo run -p mock_bin_ru_opcua  # config alternative
```

### Features Cargo

- **`gui`** (défaut) : interface graphique `egui` + vérification de mise à jour.
- `--no-default-features` : binaire **headless** (serveur OPC UA + simulation,
  sans IHM ni réseau de MAJ).

Le serveur `async-opcua` est **toujours** présent (la feature `server` de
`async-opcua`), car c'est la raison d'être de l'instrument.

---

## 3. Organisation du code

```
mock_bin_ru_opcua/src/
├── main.rs            # Assemble runtime Tokio + acteurs + IHM/headless
├── regulator.rs       # Modèle métier synchrone (PID + procédé), commandes, pas
├── config.rs          # AppConfig (TOML), sanitized(), ServerStatus
├── i18n.rs            # Catalogue i18n (8 langues), Lang + Msg + tr()
├── opcua_server.rs    # Serveur OPC UA : build + espace d'adressage + callbacks
├── gui.rs             # IHM egui (feature gui)
├── branding.rs        # Logos embarqués (feature gui)
└── actors/
    ├── simulation.rs  #   boucle de régulation (tick 0,5 s)
    └── network.rs     #   serveur OPC UA (re)configurable à chaud
```

---

## 4. Configuration

`AppConfig` (langue / réseau / procédé / régulation / `check_updates`) est
sérialisée en **TOML** (`mock_ru_opcua.toml`, surchargeable par `MOCK_CONFIG`),
chargée au démarrage (défauts si absente), sauvegardée depuis l'IHM. Toute valeur
est **assainie** au chargement (`AppConfig::sanitized` : bornes ordonnées,
`τ ≥ 1e-3`, `dead_time ≥ 0`, flottants finis).

**Invariant** : ne jamais appeler `f32::clamp` avec des bornes non validées (panic
si `min > max` ou `NaN`). Les écritures réseau passent aussi par
`Regulator::apply`, qui assainit.

### Vérification de mise à jour

Feature `gui` uniquement : au démarrage, l'IHM interroge la dernière release
GitHub via la lib partagée `mock_lib_update` (thread borné par timeout) et affiche
un bandeau si une version plus récente existe. Réglable par `check_updates`.

---

## 5. Dépendances et pièges de version

- **`async-opcua` 0.18** (serveur). Crypto **100 % Rust** (RustCrypto) : **aucune
  dépendance OpenSSL** → cross-compilation propre. Licence **MPL-2.0** (cf. `NOTICE`).
- ⚠️ `async-opcua` ne déclare **aucun MSRV** : valider sur la toolchain cible avant
  de bumper la version.
- ⚠️ Le certificat d'instance (`create_sample_keypair(true)` + `pki/`) n'est généré
  **qu'en mode chiffré** (`security.encryption`). En mode None (défaut), aucun
  certificat (démarrage instantané). ⚠️ La génération RSA en Rust pur est lente en
  *debug* : compter quelques secondes au premier passage en mode chiffré.
- `egui_plot` reste **en avance d'une mineure** sur `egui` (cf. ORME/OSNE).

---

## 6. Étendre le projet

### 6.1 Ajouter un nœud OPC UA

Dans [`opcua_server.rs`](../../src/opcua_server.rs) : déclarer le nœud
(`add_var`), brancher un callback de lecture (`on_read_*`) et, si inscriptible, un
callback d'écriture (`on_write_*`) qui émet une `Command`. Refléter la table dans
[`reference_opcua.md`](reference_opcua.md).

### 6.2 Ajouter une commande métier

Étendre l'enum `Command` ([`regulator.rs`](../../src/regulator.rs)), gérer le cas
dans `Regulator::apply` (avec assainissement), ajouter un test.

### 6.3 Ajouter une chaîne d'interface (i18n)

Ajouter une variante à `Msg` ([`i18n.rs`](../../src/i18n.rs)) et **les 8
traductions** (tableau de taille fixe vérifié à la compilation).

### 6.4 Sécurité (`SecurityConfig`)

La sécurité est implémentée dans [`opcua_server.rs`](../../src/opcua_server.rs) :
`security.encryption` ajoute un endpoint `Basic256Sha256`/`SignAndEncrypt` avec
certificat auto-généré et jetons anonyme et/ou utilisateur/mot de passe
(`ServerUserToken::user_pass`). Le filtre de log `opcua_crypto::certificate_store=off`
([`main.rs`](../../src/main.rs)) ne concerne que le mode None (pas de certificat) ;
en mode chiffré il est sans effet. La confiance des certificats clients est
**réglable** (`trust_client_certs` : auto par défaut, ou strict via `pki/trusted/`).
Pistes restantes : politiques `Aes256Sha256RsaPss`, jetons X.509.

---

## 7. Stratégie de test

Le cœur métier (`regulator.rs`) et la configuration (`config.rs`) sont **purs et
testés** : convergence PID, clamp de consigne, relaxation à l'arrêt, changement de
procédé sans saut de PV, assainissement TOML, aller-retour TOML. L'i18n vérifie la
non-vacuité et l'aller-retour de langue.

Des **tests d'intégration** couvrent en plus la couche réseau : aller-retour
client↔serveur sur l'endpoint **None** (connexion, écriture, relecture), parité de
l'acteur réseau, et — sur l'endpoint **chiffré** (`Basic256Sha256`) — l'aller-retour
anonyme ainsi que l'**authentification utilisateur/mot de passe** (bon couple accepté,
mauvais mot de passe refusé). Ces deux derniers sont marqués `#[ignore]` car la
**génération RSA est lente en *debug*** ; on les lance explicitement :

```bash
cargo test -p mock_bin_ru_opcua -- --ignored
```

En **CI**, ils tournent en **`--release`** (RSA rapide) et **`--test-threads=1`** (les
serveurs chiffrés partagent le répertoire `pki/` → exécution sérialisée).

---

## 8. Dépannage

| Symptôme | Cause probable | Remède |
|---|---|---|
| `failed to bind` au démarrage | port déjà pris / < 1024 sans droits | changer le port (*Paramètres*) ou lancer en root |
| Client ne voit pas les nœuds | mauvais endpoint / sécurité | `opc.tcp://…:4840/`, None, Anonymous ; *Browse* sous `Objects` |
| Écriture `Bad_TypeMismatch` | type incorrect | `Double` pour les grandeurs, `Boolean` pour `Run`/`Auto` |
| WARN « encrypted endpoints disabled » | aucun certificat (Phase 1b) | normal ; l'endpoint None fonctionne |

---

## 9. Build « prod » — cross-compilation depuis Linux

L'instrument est intégré à [`scripts/build-prod.sh`](../../../scripts/build-prod.sh)
(tableau `INSTRUMENTS`) : exes **avec IHM** pour Linux x86_64, Windows x86_64 et
Raspberry Pi arm64 (via `cross`), plus une image Docker headless.

⚠️ **Cross Windows et `GetHostNameW`** : la pile OPC UA tire `gethostname`, qui fait
référence au symbole winsock `GetHostNameW`. La bibliothèque d'import mingw-w64 de
l'image `cross` **par défaut** (`:0.2.5`) est trop ancienne pour le fournir →
échec à l'édition de liens. Le dépôt fixe donc, dans [`Cross.toml`](../../../Cross.toml),
l'image Windows GNU sur **`:main`** (mingw récent). Validé : builds headless **et**
IHM produisent un `.exe` valide ; ORME/OSNE compilent toujours (image sur-ensemble).

---

## 10. Conventions

- Code et commentaires en **français** ; logs/erreurs en **anglais**.
- Chaînes IHM via `i18n` (8 langues) ; jamais codées en dur.
- Logique métier **synchrone et testable** ; l'asynchrone est cantonné aux acteurs
  et à l'IO. `cargo clippy --workspace` sans avertissement.
- Invariants `ractor` : pas de garde `Mutex` au travers d'un `.await` ; pas de
  timer/`spawn` détaché sans `JoinHandle` abandonné à l'arrêt.
