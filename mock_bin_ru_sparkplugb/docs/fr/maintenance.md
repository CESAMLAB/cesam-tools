# Maintenance — Régulateur Sparkplug B (ORSE)

*🌍 **FR** · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

---

## 1. Build & lancement

```bash
cargo run -p mock_bin_ru_sparkplugb                       # IHM + edge node
cargo build -p mock_bin_ru_sparkplugb --release           # exécutable IHM
cargo build -p mock_bin_ru_sparkplugb --no-default-features # headless (sans IHM)
```

Features : `gui` (IHM `egui`, par défaut). `--no-default-features` produit un binaire
**headless** : edge node Sparkplug B + simulation, sans IHM ni vérification de MAJ.

## 2. Configuration

Fichier TOML `mock_ru_sparkplugb.toml` (répertoire courant ; chemin surchargeable par
`MOCK_CONFIG`). Sections : `language`, `[network]` (broker/Sparkplug), `[process]`,
`[regulation]`, `check_updates`. Voir [`reference_sparkplugb.md`](reference_sparkplugb.md)
pour les clés `[network]`. Toute valeur est **assainie** au chargement.

## 3. Tests

```bash
cargo test -p mock_bin_ru_sparkplugb              # unitaires (sans broker)
cargo test -p mock_bin_ru_sparkplugb -- --ignored # round-trip avec broker local
```

- **Unitaires** (sans réseau) : régulation, assainissement de config, et surtout la
  couche Sparkplug (topics, payloads `NBIRTH`/`NDEATH`, round-trip encode/decode,
  mapping `NCMD`, rejet de mauvais type, repli du `seq` 255→0).
- **Intégration `#[ignore]`** : nécessite un broker MQTT local —
  `docker run -it --rm -p 1883:1883 eclipse-mosquitto` — puis lance le round-trip
  complet (NBIRTH reçu, NCMD appliqué, NDATA reflété).

## 4. Dépannage

| Symptôme | Piste |
|---|---|
| « Déconnecté » permanent | broker injoignable (`broker_host`/`broker_port`, pare-feu, broker arrêté) |
| Le SCADA ne reçoit rien | `group_id`/`edge_node_id` ; abonnement `spBv1.0/<group>/#` ; payloads protobuf |
| Échec TLS | broker en TLS sur 8883 ; certificat racine reconnu par le système |
| NCMD ignoré | métrique non pilotable ou mauvais type (cf. table des métriques) |

## 5. Docker (headless)

L'image headless se construit via `scripts/build-prod.sh` (entrée
`mock_bin_ru_sparkplugb:ru_spb:0`). ORSE étant un **client**, il **n'expose aucun
port** (`PORT=0`, `EXPOSE 0` = métadonnée inerte) et **aucun `HEALTHCHECK`** TCP
n'est pertinent : la liveness se constate côté broker via le **Last Will/NDEATH**.
Monter un volume sur le répertoire de travail pour fournir le `mock_ru_sparkplugb.toml`.

## 6. Étendre

La table de métriques et le mapping `NCMD` sont la **source de vérité** dans
[`sparkplug_node.rs`](../../src/sparkplug_node.rs). Pour ajouter une métrique :
l'ajouter à `data_metrics`/`changed_metrics` (lecture) et, si pilotable, à
`ncmd_to_actions` (écriture → `Command`), puis refléter ici et dans
[`reference_sparkplugb.md`](reference_sparkplugb.md). Ajouter un test dans le module.

## 7. Dépendances notables

- `rumqttc` (client MQTT, rustls), `sparkplug-rs` (protobuf Tahu, codegen Rust pur).
- MSRV : à vérifier après un build `cross` complet (peut dépasser le plancher 1.85 du
  workspace selon les dépendances rustls).
