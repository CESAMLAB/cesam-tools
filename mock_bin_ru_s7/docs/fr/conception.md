# Conception — Régulateur S7 (ORSS)

*🌍 **FR** · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

---

## 1. Vue d'ensemble

ORSS réutilise l'architecture des autres instruments CESAM-Lab : **modèle métier
synchrone et testable** (PID + procédé), **acteurs `ractor`** sur Tokio, **IHM
`egui`** lisant un instantané partagé. Seule la **couche transport** change : un
**serveur S7comm** (ISO-on-TCP / RFC1006) au lieu de Modbus/OPC UA.

```
        Command (cast)                      refresh chaque pas
IHM   ───────────────────►  SimulationActor ───────────────────►  SharedSnapshot (IHM)
S7 Write Var ────────────►  (Regulator)      ──────────────────►  SharedSnapshot
S7 Read Var  ◄────────────────────────────────  SharedSnapshot (image DB1)
```

## 2. Acteurs

- **`SimulationActor`** — possède l'unique [`Regulator`]. Boucle à pas fixe ;
  applique les `Command` (IHM ou écritures S7) ; publie l'instantané après chaque
  mutation.
- **`S7ServerActor`** — possède la **boucle d'écoute TCP**. Une tâche tokio dédiée
  lie le socket et accepte les clients ; chaque session est portée par un `JoinSet`
  **interne** (donc abattue avec la boucle — aucune tâche détachée). `Reconfigure`
  relance l'écoute si l'IP/port change et met à jour la **liste blanche** partagée.

## 3. Couche protocole

[`s7_server.rs`](../../src/s7_server.rs) est **pur et synchrone** (aucune dépendance
réseau) : framing TPKT, COTP (CR→CC, DT) et S7comm (Setup, Read Var, Write Var) sur
une **image d'octets DB1**. Le parsing est **borné** (accès par `get`/slices
vérifiés) : une trame malformée venue du réseau ne provoque **jamais** de panique,
seulement une absence de réponse. C'est l'équivalent S7 du `opcua_server.rs`, isolé
pour être **testable sans socket**.

### Pourquoi un serveur fait main

Il n'existe pas de bibliothèque **serveur** S7 en Rust (les crates `s7`/`s7-comm`
sont orientées **client**). Le sous-ensemble nécessaire (COTP classe 0 + S7 Read/
Write Var sur un DB) est compact et bien spécifié : l'implémenter à la main donne un
contrôle total et une surface testable, cohérente avec les autres instruments.

## 4. Politique de sessions

Plusieurs clients S7 **simultanés** sont acceptés (comportement d'automate), à
l'inverse du mono-maître d'ORME (éviction) et du point-à-point d'OSNE (squat).
Chaque session lit l'image DB1 courante et route ses écritures vers la simulation ;
le « dernier qui écrit gagne », comme un automate réel.

## 5. Posture de sécurité

- **Ni authentification ni chiffrement** (S7 « classic ») : seuls la **liste blanche
  d'IP** et la topologie réseau protègent l'accès. `0.0.0.0` + liste vide = exposé →
  bandeau d'avertissement dans l'IHM ([`NetworkConfig::is_exposed`](../../src/config.rs)).
- **Assainissement TOML** ([`AppConfig::sanitized`](../../src/config.rs)) : procédé/
  PID/bornes finis et ordonnés. Toute écriture S7 est **clampée/assainie** par
  `Regulator::apply` : la surface réseau ne peut produire ni `NaN`/`Inf` ni valeur
  aberrante.
- **Parsing réseau borné** : aucune trame ne peut provoquer de panique (cf. §3).
