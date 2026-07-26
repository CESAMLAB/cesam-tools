# Conception — Régulateur Sparkplug B (ORSE)

*🌍 **FR** · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

---

## 1. Vue d'ensemble

ORSE réutilise l'architecture des autres instruments CESAM-Lab : un **modèle métier
synchrone et testable** (régulateur PID + procédé), piloté par des **acteurs
`ractor`** sur Tokio, et une **IHM `egui`** qui lit un instantané partagé. Seule la
**couche transport** change : ici, un **edge node MQTT Sparkplug B** (client sortant)
au lieu d'un serveur Modbus/OPC UA.

```
        Command (cast)                      refresh chaque pas
IHM   ───────────────────►  SimulationActor ───────────────────►  SharedSnapshot (IHM)
NCMD (broker) ───────────►  (Regulator)      ──────────────────►  SharedSnapshot (publication)
NBIRTH/NDATA (broker) ◄──────────────────────  SharedSnapshot
```

## 2. Acteurs

- **`SimulationActor`** — possède l'unique [`Regulator`]. Boucle à pas fixe (`Tick`
  toutes les 0,5 s) ; applique les `Command` (IHM ou NCMD) ; publie l'instantané
  après chaque mutation. Identique aux autres instruments.
- **`SparkplugActor`** — possède le **client MQTT** (`rumqttc`) et exécute le **cycle
  de vie Sparkplug B** dans une tâche tokio dédiée (dont le `JoinHandle` est abattu à
  l'arrêt). Un message `Reconfigure` relance le client si le broker/les identifiants/
  TLS changent.

## 3. Couche protocole

[`sparkplug_node.rs`](../../src/sparkplug_node.rs) est **pur et synchrone** (aucune
dépendance tokio/rumqttc) : construction des **topics**, table de **métriques**,
fabrication des **payloads** (`NBIRTH`/`NDATA`/`NDEATH`), (dé)sérialisation protobuf,
mapping **`NCMD` → commandes**, et le compteur `seq`. C'est l'équivalent du
`opcua_server.rs` d'ORUE, isolé pour être **testable sans broker**.

### Choix des bibliothèques

- **`rumqttc`** — client MQTT async Tokio (Last Will, reconnexion automatique, TLS
  via rustls — déjà dans l'arbre via OPC UA, **sans OpenSSL**).
- **`sparkplug-rs`** — structs protobuf Eclipse Tahu (`Payload`/`Metric`/`Value`),
  générés en **100 % Rust** (rust-protobuf, **pas de `protoc`** → cross propre). La
  crate re-exporte `protobuf` (runtime), utilisé pour `write_to_bytes`/`parse_from_bytes`.
- **Alternative écartée : `srad`** — cadre haut niveau d'edge node Sparkplug qui gère
  lui-même `bdSeq`/`seq`/rebirth. Écarté volontairement : on **possède** la machine
  d'état dans l'acteur réseau pour la rendre explicite et testable (cohérence avec
  les autres instruments).

## 4. Cycle de vie & invariants

- **`bdSeq`** incrémenté à chaque (re)démarrage du client ; **même** valeur dans le
  Last Will `NDEATH` et le `NBIRTH` d'une session.
- **`seq`** roulant 0–255, remis à 0 à chaque `NBIRTH`.
- **`NDEATH`** porté par le **Last Will MQTT** : robuste à toute perte de lien.
- **Publication `NDATA`** par **diff** d'instantané (cadence = pas de simulation en
  mode *sur changement*, ou périodique). Le verrou du snapshot n'est **jamais** tenu
  à travers un `.await`.

## 5. Posture de sécurité

- **Pas de liste blanche d'IP** (l'instrument est un client, pas un serveur) : écart
  de parité **assumé** avec ORME/OSNE.
- **MQTT en clair par défaut** (port 1883) — non chiffré, non authentifié réseau.
  Bandeau d'avertissement dans l'IHM. Activer **TLS** + identifiants pour sortir d'un
  réseau de confiance.
- **Mot de passe en clair** dans le TOML — **simulateur uniquement**.
- **Assainissement TOML** ([`AppConfig::sanitized`](../../src/config.rs)) : procédé/
  PID/bornes finis et ordonnés, identifiants Sparkplug non vides, temporisations
  bornées. Toute écriture NCMD est **clampée/assainie** par `Regulator::apply`.
