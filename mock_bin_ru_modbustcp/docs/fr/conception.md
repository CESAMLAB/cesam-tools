# Document de conception — Régulateur simulé Modbus TCP

*🌍 **FR** · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

> Produit : **ORME** · Crate : `mock_bin_ru_modbustcp` · Workspace : `cesam-tools` · Licence : MIT

Ce document décrit l'architecture, les choix techniques et les principes de
fonctionnement du régulateur industriel simulé. Il s'adresse aux développeurs
qui maintiennent ou étendent le projet.

---

## 1. Objectif et périmètre

Fournir un **instrument industriel virtuel** : un régulateur de procédé qui se
comporte de façon réaliste et communique en **Modbus TCP** (esclave), afin de
développer et tester des superviseurs / automates / passerelles **sans matériel**.

Le simulateur couvre :

- un **procédé physique** modélisé par une fonction de transfert ;
- une **régulation** bidirectionnelle (chaud / froid) : PID, tout-ou-rien (TOR) ou
  relais à cycle (PWM) ;
- une **interface Modbus TCP** exposant l'état complet ;
- une **IHM** de pilotage, visualisation et paramétrage ;
- la **persistance** des paramètres.

Hors périmètre actuel : Modbus RTU, redondance, historisation long terme,
authentification forte (seule une liste blanche d'IP est fournie).

---

## 2. Vue d'ensemble

```
┌──────────────────────────────────────────────────────────────────────┐
│                       Processus (thread principal)                     │
│                                                                        │
│   ┌─────────────────────────┐         lit (Mutex)                      │
│   │   IHM  egui / eframe     │◄──────────────── SharedSnapshot         │
│   │   (gui.rs)               │◄──────────────── SharedStatus           │
│   └───────────┬─────────────┘                                          │
│               │ cast (non bloquant)                                    │
└───────────────┼────────────────────────────────────────────────────────┘
                │
   ┌────────────┼──────────── Runtime Tokio (threads de fond) ───────────┐
   │            ▼                                                         │
   │   ┌──────────────────┐  refresh  ┌──────────────┐                   │
   │   │ SimulationActor   ├──────────►│ SharedSnapshot│ (IHM)            │
   │   │  (ractor)         ├──────────►│ SharedMap     │ (Modbus)         │
   │   │  possède le        │           └──────┬───────┘                  │
   │   │  Regulator         │◄── Command ──┐    │ lit                     │
   │   └──────────────────┘              │    ▼                          │
   │          ▲ Command (cast)            │  ┌──────────────────────┐     │
   │          │                           └──┤ RegulatorService      │     │
   │   ┌──────┴───────────┐  gère/rebind     │ (trait Service)       │     │
   │   │ ModbusServerActor ├─────────────────►  serveur Modbus TCP   │◄──── clients
   │   │  (ractor)         │  filtre IP ──────► (tokio-modbus)        │     │
   │   └──────────────────┘   (SharedAllowlist)└──────────────────────┘     │
   └────────────────────────────────────────────────────────────────────┘
```

Principe directeur : **un seul propriétaire de l'état métier**. Le `Regulator`
n'est jamais partagé ; il vit dans `SimulationActor`. Toutes les écritures
(IHM ou Modbus) sont des **messages** `Command`. Les lectures se font sur des
**copies** rafraîchies à chaque pas (`SharedSnapshot`, `SharedMap`), ce qui élimine
les verrous sur la logique et les conditions de course.

---

## 3. Choix techniques

| Besoin | Choix | Justification |
|--------|-------|---------------|
| Concurrence | **`ractor`** (acteurs) sur **Tokio** | Isole l'état mutable dans un acteur ; mutations sérialisées par messages, sans verrou applicatif. Préférence projet. |
| Modbus TCP esclave | **`tokio-modbus`** (`tcp-server`) | Implémentation async mature ; le trait `Service` mappe proprement requête→réponse. |
| IHM | **`egui` / `eframe`** + `egui_plot` | Mode immédiat, multiplateforme, sans état d'UI complexe à synchroniser. |
| Procédé | **FOPDT** (1ᵉʳ ordre + retard) | Modèle standard et suffisant pour un procédé thermique ; peu de paramètres, intuitif. |
| Persistance | **`serde` + `toml`** | Format lisible/éditable à la main, idéal pour des paramètres d'appareil. |

### Pourquoi séparer logique synchrone et asynchrone

`mock_lib_control` et `regulator.rs` sont **purement synchrones** (aucune IO,
aucun async). Avantages : testables unitairement de façon déterministe,
réutilisables par d'autres instruments, et raisonnables à relire. L'asynchrone
est cantonné aux **acteurs** et à la **couche réseau**.

---

## 4. Modèle de données

### État métier (`regulator.rs`)

- `Regulator` — agrégat propriétaire : modes, consignes, régulateurs (`Pid`,
  `OnOff`) et procédé (`FirstOrderProcess`). Non `Clone`, non partagé.
- `RegulatorConfig` — configuration statique (procédé, gains, bornes, `dt`).
  **Source unique** des valeurs par défaut (la config TOML en dérive).
- `RegulatorSnapshot` — **copie immuable** (`Copy`) de l'état observable, publiée
  à chaque pas. C'est le contrat de lecture pour l'IHM et la table Modbus.
- `Command` — énumération des mutations possibles (marche, mode, consignes,
  réglages, procédé, bornes).

### Structures partagées (`actors/mod.rs`, `config.rs`)

| Type | Contenu | Écrit par | Lu par |
|------|---------|-----------|--------|
| `SharedSnapshot` | `RegulatorSnapshot` typé | SimulationActor | IHM |
| `SharedMap` | `MemoryMap` (images des 4 tables Modbus) | SimulationActor | RegulatorService |
| `SharedAllowlist` | `IpFilter` | ModbusServerActor | acceptation connexions |
| `SharedStatus` | `ServerStatus` (écoute / erreur) | ModbusServerActor | IHM |

Tous sont des `Arc<Mutex<…>>` : sections critiques **courtes** (copie / refresh),
jamais tenues pendant un calcul ou une IO.

---

## 5. Composants

### 5.1 `mock_lib_control` (bibliothèque)

- `Pid` — PID à temps discret, dérivée sur l'erreur, **anti-emballement** par
  bornage du terme intégral. API : `step(sp, pv, dt)` ou `step_with_error(err, dt)`
  (réutilisé pour le sens froid).
- `OnOff` — tout-ou-rien à **hystérésis symétrique** (zone morte) **et
  anti-court-cycle** : un temps de cycle minimal (`min_cycle`, s) interdit toute
  commutation tant que le relais n'est pas resté assez longtemps dans son état,
  modélisant la protection d'un actionneur réel. Le relais **latche** son état :
  c'est l'appelant qui doit lui passer l'erreur signée sans le réinitialiser au
  changement de signe (cf. § 5.2).
- `Pwm` — modulateur de largeur d'impulsion (**relais à cycle** /
  *time-proportioning*) : sur une période fixe `T_c`, la sortie tout-ou-rien est
  active la fraction `duty` du cycle (`duty` **échantillonné une fois par cycle**
  pour éviter un biais en régime établi). Permet de réguler finement un organe TOR.
- `FirstOrderProcess` — fonction de transfert `K·e^(-L·s)/(1+T·s)`, intégration
  d'Euler + ligne à retard. `reconfigure(...)` change les paramètres sans saut.
- `ControllerKind` — `Off` / `Pid` / `OnOff` / `Pwm`, avec codage Modbus
  (`to_code`/`from_code`).

### 5.2 `regulator.rs`

Orchestration de la régulation à chaque pas (`step`) :

1. si **arrêté** → sortie 0, régulateurs réinitialisés ;
2. si **manuel** → sortie = consigne manuelle (% signé) ;
3. si **auto** → on calcule **séparément** la contribution du sens chaud (sens 1,
   erreur `SP − PV`) et du sens froid (sens 2, erreur `PV − SP`), chacune ≥ 0,
   puis `sortie = chaud − froid` :
   - **PID** : sortie bornée à `[0, 100]` (`out_min = 0`) — le sens inactif (erreur
     négative) sort 0 et son intégrale se **purge naturellement** par bornage. On
     ne la remet **pas** à zéro de force : avec la forte ondulation du PWM, l'effacer
     à chaque dépassement de consigne introduirait une erreur statique ;
   - **TOR** : le relais est évalué sur l'erreur signée et conserve son état à la
     traversée de la consigne, ce qui restaure une bande d'hystérésis **symétrique**
     `[SP − h/2, SP + h/2]` (les bandes chaud/froid restent disjointes, donc les
     deux relais sont mutuellement exclusifs) ;
   - **PWM** : un PID calcule le rapport cyclique, modulé par le relais à cycle ;
     la sortie physique est strictement 0 % ou 100 %, mais sa moyenne suit le PID.
4. la sortie pilote le procédé qui produit la nouvelle mesure (PV).

> **Historique** : avant cette révision, l'aiguillage chaud/froid se faisait par
> le signe de l'erreur et **réinitialisait** le relais TOR à la traversée de la
> consigne — ce qui tronquait l'hystérésis à `[SP − h/2, SP]` (moitié de bande,
> asymétrique) et rendait la régulation TOR médiocre. Le calcul par sens séparé
> corrige ce défaut.

### 5.3 `actors/simulation.rs`

`SimulationActor` (ractor). `pre_start` arme un `send_interval(dt)` qui émet
`Tick`. `handle` traite `Tick` (avance la simulation) et `Command` (applique une
mutation), puis **publie** l'état dans `SharedSnapshot` et `SharedMap`.

### 5.4 `actors/network.rs`

`ModbusServerActor` possède le serveur Modbus. `Reconfigure(NetworkConfig)` :
- met à jour la **liste blanche** partagée (effet immédiat, sans relance) ;
- si le **transport** (TCP/RTU), le **port / IP** ou les **paramètres série**
  changent, **arrête** la tâche serveur et la **relance** (`start_tcp` ou
  `start_rtu`) ; publie l'état dans `SharedStatus` (succès ou erreur).

Un **seul transport** est actif à la fois (`Transport::Tcp` ou `Rtu`). Le RTU est
derrière la **feature `rtu`** ; sans elle, sélectionner RTU publie une erreur de
statut explicite.

### 5.5 `modbus_server.rs`

`RegulatorService` implémente `tokio_modbus::server::Service` de manière
**synchrone** (`future::Ready`) : lectures = découpe de `SharedMap` ; écritures =
décodage en `Command` (via `map.rs`) puis `cast` vers `SimulationActor`.

**Politique mono-maître.** `serve` (TCP) n'autorise **qu'un maître distant à la
fois** : à chaque nouvelle connexion (IP autorisée par la liste blanche), la
précédente est fermée. Mécanisme : le `TcpStream` est enveloppé dans un
`CancellableStream` qui, sur réception d'un signal `oneshot`, renvoie **EOF en
lecture** — la boucle de traitement de `tokio-modbus` se termine alors et ferme le
socket. `serve_rtu` (feature `rtu`) sert le bus série via
`rtu::Server::serve_forever` : le bus RS485 *est* l'unique maître (rien à évincer).

> ⚠️ L'IHM n'emprunte pas ce chemin : elle envoie ses `Command` directement à
> l'acteur, elle n'est donc jamais comptée comme un maître.
>
> ⚠️ Le serveur RTU de `tokio-modbus` 0.17 ne transmet pas l'adresse esclave au
> service : l'appareil répond donc quelle que soit l'adresse demandée. Une liaison
> **point-à-point** est recommandée. `slave_id` est persisté et affiché, mais non
> utilisé pour filtrer (limitation amont).

### 5.6 `map.rs`

**Source de vérité** du plan d'adressage Modbus. Constantes d'adresses,
`MemoryMap` (images des tables), `refresh_from(snapshot)` (état→registres) et
`*_to_command(s)` (écritures→commandes). Encodage des `f32` sur 2 registres,
big-endian, mot de poids fort en tête.

### 5.7 `config.rs`

`AppConfig` (réseau / procédé / régulation) ⇄ TOML. `IpFilter` (jokers `*` par
octet IPv4). `ServerStatus`. `to_regulator_config()` fait le pont vers le domaine.

### 5.8 `gui.rs`

IHM **page unique** : en-tête (états + boutons), panneau commandes (gauche),
supervision + courbe (centre), table Modbus live (droite), modal Paramètres.
Lit les `Shared*`, envoie des `Command` par `cast` non bloquant.

---

## 6. Scénarios (séquences)

**Lecture Modbus (PV)** : client → `RegulatorService::call(ReadInputRegisters)` →
lecture `SharedMap` → `Response`. Aucune interaction avec l'acteur (latence minimale).

**Écriture Modbus (consigne)** : client → `call(WriteMultipleRegisters)` →
`map::holdings_to_commands` → `cast(Command::SetSpAuto)` → l'acteur applique au
pas suivant → republie `SharedMap`/`SharedSnapshot`.

**Commande IHM** : interaction → `cast(Command)` → idem.

**Reconfiguration réseau** : modal *Appliquer* → `cast(Reconfigure)` →
ModbusServerActor rebinde si nécessaire → `SharedStatus` mis à jour → l'en-tête
de l'IHM reflète l'état.

**Tick** : timer → `Tick` → `Regulator::step` → publication.

---

## 7. Théorie de régulation

**Procédé (FOPDT)** : `v[k+1] = v[k] + (dt/T)·(cible − v[k])`, avec
`cible = ambiant + K·u` et `u` retardée de `L` secondes (ligne à retard).

**PID** : `u = Kp·e + Ki·∫e + Kd·de/dt`, intégrale bornée à `[out_min, out_max]`
(anti-windup). Dérivée sur l'erreur (compromis simplicité/symétrie chaud-froid).

**TOR** : actif si `e > +H/2`, inactif si `e < −H/2`, sinon état conservé.

**Bidirectionnel** : un seul sens agit à la fois, sélectionné par le signe de
l'erreur ; la sortie globale est signée (+ chaud / − froid).

---

## 8. Décisions et compromis

- **Double publication (`Snapshot` + `Map`)** plutôt qu'une seule structure :
  l'IHM manipule des types métier, le Modbus des registres bruts ; les deux
  restent simples et découplés, au prix d'un léger surcoût de copie négligeable.
- **Lectures Modbus sans passer par l'acteur** : on lit `SharedMap` directement
  pour minimiser la latence ; l'acteur reste seul **écrivain**, donc pas de course.
- **Service Modbus synchrone** (`future::Ready`) : tout le travail est non bloquant
  (lock court + cast), inutile de boxer un futur.
- **Rebind sur changement de port** : un socket ne change pas de port ; on
  accepte une courte interruption de service à la reconfiguration.
- **Dérivée sur l'erreur** (et non sur la mesure) : léger « coup de fouet » au
  changement de consigne, accepté pour garder l'algorithme symétrique et simple.

---

## 9. Évolutions envisageables

- Modbus RTU / série (réutiliser `RegulatorService`, changer le transport).
- Rampe de consigne, auto-tuning PID, défauts simulés (capteur HS, saturation).
- Historisation / export CSV de la tendance.
- Bascule de l'IHM en **onglets** si la page unique devient trop dense.
- Nouveaux instruments : créer `mock_bin_<nom>` et factoriser le commun dans des
  `mock_lib_*` (voir [maintenance.md](maintenance.md)).
