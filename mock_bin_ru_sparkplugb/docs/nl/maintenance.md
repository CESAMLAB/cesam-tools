# Onderhoud — Sparkplug B-regelaar (ORSE)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · **NL** · [PL](../pl/maintenance.md)*

---

## 1. Build & start

```bash
cargo run -p mock_bin_ru_sparkplugb                       # GUI + edge node
cargo build -p mock_bin_ru_sparkplugb --release           # uitvoerbaar bestand GUI
cargo build -p mock_bin_ru_sparkplugb --no-default-features # headless (zonder GUI)
```

Features: `gui` (GUI `egui`, standaard). `--no-default-features` levert een
**headless** binary: Sparkplug B edge node + simulatie, zonder GUI en zonder
updatecontrole.

## 2. Configuratie

TOML-bestand `mock_ru_sparkplugb.toml` (huidige map; pad overschrijfbaar via
`MOCK_CONFIG`). Secties: `language`, `[network]` (broker/Sparkplug), `[process]`,
`[regulation]`, `check_updates`. Zie [`reference_sparkplugb.md`](reference_sparkplugb.md)
voor de sleutels van `[network]`. Elke waarde wordt bij het laden **gesaneerd**.

## 3. Tests

```bash
cargo test -p mock_bin_ru_sparkplugb              # unit (zonder broker)
cargo test -p mock_bin_ru_sparkplugb -- --ignored # round-trip met lokale broker
```

- **Unit** (zonder netwerk): regeling, configuratiesanering, en vooral de
  Sparkplug-laag (topics, payloads `NBIRTH`/`NDEATH`, round-trip encode/decode,
  mapping `NCMD`, afwijzing van verkeerd type, terugloop van `seq` 255→0).
- **Integratie `#[ignore]`**: vereist een lokale MQTT-broker —
  `docker run -it --rm -p 1883:1883 eclipse-mosquitto` — en voert vervolgens de volledige
  round-trip uit (NBIRTH ontvangen, NCMD toegepast, NDATA weerspiegeld).

## 4. Probleemoplossing

| Symptoom | Spoor |
|---|---|
| Permanent "Verbroken" | broker onbereikbaar (`broker_host`/`broker_port`, firewall, broker gestopt) |
| De SCADA ontvangt niets | `group_id`/`edge_node_id`; abonnement `spBv1.0/<group>/#`; protobuf-payloads |
| TLS-fout | broker in TLS op 8883; root-certificaat erkend door het systeem |
| NCMD genegeerd | niet-aanstuurbare metric of verkeerd type (zie metrictabel) |

## 5. Docker (headless)

De headless image wordt gebouwd via `scripts/build-prod.sh` (entry
`mock_bin_ru_sparkplugb:ru_spb:0`). Aangezien ORSE een **client** is, **exposeert het geen
enkele poort** (`PORT=0`, `EXPOSE 0` = inerte metadata) en is **geen enkele TCP-`HEALTHCHECK`**
relevant: de liveness wordt aan de broker-zijde vastgesteld via de **Last Will/NDEATH**.
Koppel een volume aan de werkmap om de `mock_ru_sparkplugb.toml` te leveren.

## 6. Uitbreiden

De metrictabel en de `NCMD`-mapping zijn de **bron van waarheid** in
[`sparkplug_node.rs`](../../src/sparkplug_node.rs). Om een metric toe te voegen:
voeg hem toe aan `data_metrics`/`changed_metrics` (lezen) en, indien aanstuurbaar, aan
`ncmd_to_actions` (schrijven → `Command`), weerspiegel dit dan hier en in
[`reference_sparkplugb.md`](reference_sparkplugb.md). Voeg een test toe in de module.

## 7. Noemenswaardige afhankelijkheden

- `rumqttc` (MQTT-client, rustls), `sparkplug-rs` (protobuf Tahu, pure Rust-codegen).
- MSRV: te controleren na een volledige `cross`-build (kan de bodem 1.85 van de
  workspace overschrijden, afhankelijk van de rustls-afhankelijkheden).
