# Maintenance — Sparkplug B Regulator (ORSE)

*🌍 [FR](../fr/maintenance.md) · **EN** · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

---

## 1. Build & launch

```bash
cargo run -p mock_bin_ru_sparkplugb                       # GUI + edge node
cargo build -p mock_bin_ru_sparkplugb --release           # GUI executable
cargo build -p mock_bin_ru_sparkplugb --no-default-features # headless (no GUI)
```

Features: `gui` (`egui` GUI, default). `--no-default-features` produces a
**headless** binary: Sparkplug B edge node + simulation, with no GUI and no update
check.

## 2. Configuration

TOML file `mock_ru_sparkplugb.toml` (current directory; path overridable via
`MOCK_CONFIG`). Sections: `language`, `[network]` (broker/Sparkplug), `[process]`,
`[regulation]`, `check_updates`. See [`reference_sparkplugb.md`](reference_sparkplugb.md)
for the `[network]` keys. Every value is **sanitized** on load.

## 3. Tests

```bash
cargo test -p mock_bin_ru_sparkplugb              # unit (no broker)
cargo test -p mock_bin_ru_sparkplugb -- --ignored # round-trip with a local broker
```

- **Unit** (no network): regulation, config sanitization, and above all the
  Sparkplug layer (topics, `NBIRTH`/`NDEATH` payloads, encode/decode round-trip,
  `NCMD` mapping, bad-type rejection, `seq` wraparound 255→0).
- **Integration `#[ignore]`**: requires a local MQTT broker —
  `docker run -it --rm -p 1883:1883 eclipse-mosquitto` — then runs the full
  round-trip (NBIRTH received, NCMD applied, NDATA reflected).

## 4. Troubleshooting

| Symptom | Lead |
|---|---|
| Permanent "Disconnected" | broker unreachable (`broker_host`/`broker_port`, firewall, broker stopped) |
| SCADA receives nothing | `group_id`/`edge_node_id`; subscription `spBv1.0/<group>/#`; protobuf payloads |
| TLS failure | broker in TLS on 8883; root certificate recognized by the system |
| NCMD ignored | non-controllable metric or wrong type (see the metrics table) |

## 5. Docker (headless)

The headless image is built via `scripts/build-prod.sh` (entry
`mock_bin_ru_sparkplugb:ru_spb:0`). As ORSE is a **client**, it **exposes no port**
(`PORT=0`, `EXPOSE 0` = inert metadata) and **no TCP `HEALTHCHECK`** is relevant:
liveness is observed on the broker side via the **Last Will/NDEATH**. Mount a volume
on the working directory to provide the `mock_ru_sparkplugb.toml`.

## 6. Extending

The metrics table and the `NCMD` mapping are the **source of truth** in
[`sparkplug_node.rs`](../../src/sparkplug_node.rs). To add a metric: add it to
`data_metrics`/`changed_metrics` (read) and, if controllable, to `ncmd_to_actions`
(write → `Command`), then reflect it here and in
[`reference_sparkplugb.md`](reference_sparkplugb.md). Add a test in the module.

## 7. Notable dependencies

- `rumqttc` (MQTT client, rustls), `sparkplug-rs` (Tahu protobuf, pure Rust codegen).
- MSRV: to be verified after a full `cross` build (may exceed the workspace floor of
  1.85 depending on the rustls dependencies).
