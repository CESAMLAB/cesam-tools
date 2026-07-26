# Conception — Régulateur EtherNet/IP (OREE)

*🌍 **FR** · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

---

## 1. Vue d'ensemble

OREE réutilise l'architecture des autres instruments CESAM-Lab : **modèle métier
synchrone et testable** (PID + procédé), **acteurs `ractor`** sur Tokio, **IHM
`egui`** lisant un instantané partagé. Seule la **couche transport** change : un
**adaptateur EtherNet/IP** (encapsulation + CIP) au lieu de Modbus/OPC UA/S7.

```
        Command (cast)                      refresh chaque pas
IHM   ───────────────────►  SimulationActor ───────────────────►  SharedSnapshot (IHM)
CIP Write Tag ───────────►  (Regulator)      ──────────────────►  SharedSnapshot
CIP Read Tag  ◄────────────────────────────────  SharedSnapshot
```

## 2. Acteurs

- **`SimulationActor`** — possède l'unique [`Regulator`] ; applique les `Command`
  (IHM ou écritures CIP) ; publie l'instantané après chaque mutation.
- **`EipServerActor`** — possède la **boucle d'écoute TCP**. Une tâche tokio lie le
  socket et accepte les clients ; chaque session (avec son *session handle*) est
  portée par un `JoinSet` **interne** (abattu avec la boucle — aucune tâche
  détachée). `Reconfigure` relance l'écoute si l'IP/port change et met à jour la
  **liste blanche** partagée.

## 3. Couche protocole

[`eip_server.rs`](../../src/eip_server.rs) est **pur et synchrone** : encapsulation
EtherNet/IP (`RegisterSession`, `SendRRData`/CPF) et CIP (`Read Tag`/`Write Tag` par
segment symbolique). Tout est **little-endian**. Le parsing est **borné** (slices
vérifiés) : un paquet malformé venu du réseau ne provoque **jamais** de panique,
seulement une absence de réponse. C'est l'équivalent du `opcua_server.rs`, isolé pour
être **testable sans socket**.

### Pourquoi un adaptateur fait main

Il n'existe pas de bibliothèque **serveur/adaptateur** EtherNet/IP en Rust (les
crates `rseip`, `rust-ethernet-ip`, `cip` sont orientées **client/scanner**). Le
sous-ensemble nécessaire (encapsulation + CIP Read/Write Tag sur des tags nommés) est
compact : l'implémenter à la main donne un contrôle total et une surface testable,
cohérente avec les autres instruments.

## 4. Politique de sessions

Plusieurs clients **simultanés** sont acceptés (comportement d'un adaptateur), à
l'inverse du mono-maître d'ORME. Chaque session reçoit un *session handle* et lit
l'instantané courant ; le « dernier qui écrit gagne ».

## 5. Posture de sécurité

- **Ni authentification ni chiffrement** (EtherNet/IP « classic ») : seuls la **liste
  blanche d'IP** et la topologie réseau protègent l'accès. `0.0.0.0` + liste vide =
  exposé → bandeau d'avertissement ([`NetworkConfig::is_exposed`](../../src/config.rs)).
- **Assainissement TOML** ([`AppConfig::sanitized`](../../src/config.rs)) : procédé/
  PID/bornes finis et ordonnés. Toute écriture CIP est **clampée/assainie** par
  `Regulator::apply` : la surface réseau ne peut produire ni `NaN`/`Inf` ni valeur
  aberrante.
- **Parsing réseau borné** : aucun paquet ne peut provoquer de panique (cf. §3).
