# Wartung — Sparkplug-B-Regler (ORSE)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · **DE** · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

---

## 1. Build & Start

```bash
cargo run -p mock_bin_ru_sparkplugb                       # IHM + Edge-Node
cargo build -p mock_bin_ru_sparkplugb --release           # ausführbare Datei mit IHM
cargo build -p mock_bin_ru_sparkplugb --no-default-features # headless (ohne IHM)
```

Features: `gui` (IHM `egui`, standardmäßig). `--no-default-features` erzeugt ein
**headless**-Binary: Sparkplug-B-Edge-Node + Simulation, ohne IHM und ohne
Update-Prüfung.

## 2. Konfiguration

TOML-Datei `mock_ru_sparkplugb.toml` (aktuelles Verzeichnis; Pfad überschreibbar durch
`MOCK_CONFIG`). Abschnitte: `language`, `[network]` (Broker/Sparkplug), `[process]`,
`[regulation]`, `check_updates`. Siehe [`reference_sparkplugb.md`](reference_sparkplugb.md)
für die `[network]`-Schlüssel. Jeder Wert wird beim Laden **bereinigt**.

## 3. Tests

```bash
cargo test -p mock_bin_ru_sparkplugb              # Unit-Tests (ohne Broker)
cargo test -p mock_bin_ru_sparkplugb -- --ignored # Round-Trip mit lokalem Broker
```

- **Unit-Tests** (ohne Netzwerk): Regelung, Bereinigung der Konfiguration und vor allem
  die Sparkplug-Schicht (Topics, Payloads `NBIRTH`/`NDEATH`, Round-Trip encode/decode,
  Abbildung `NCMD`, Ablehnung falschen Typs, Überlauf des `seq` 255→0).
- **Integration `#[ignore]`**: benötigt einen lokalen MQTT-Broker —
  `docker run -it --rm -p 1883:1883 eclipse-mosquitto` — und führt dann den vollständigen
  Round-Trip aus (NBIRTH empfangen, NCMD angewendet, NDATA gespiegelt).

## 4. Fehlerbehebung

| Symptom | Ansatz |
|---|---|
| Dauerhaft „Getrennt" | Broker nicht erreichbar (`broker_host`/`broker_port`, Firewall, Broker gestoppt) |
| Das SCADA empfängt nichts | `group_id`/`edge_node_id`; Abonnement `spBv1.0/<group>/#`; protobuf-Payloads |
| TLS-Fehler | Broker mit TLS auf 8883; vom System anerkanntes Wurzelzertifikat |
| NCMD ignoriert | nicht steuerbare Metrik oder falscher Typ (siehe Metriktabelle) |

## 5. Docker (headless)

Das headless-Image wird über `scripts/build-prod.sh` erstellt (Eintrag
`mock_bin_ru_sparkplugb:ru_spb:0`). Da ORSE ein **Client** ist, **exponiert er keinen
Port** (`PORT=0`, `EXPOSE 0` = inerte Metadaten) und **kein TCP-`HEALTHCHECK`** ist
relevant: Die Liveness wird brokerseitig über den **Last Will/NDEATH** festgestellt.
Ein Volume auf das Arbeitsverzeichnis einhängen, um die `mock_ru_sparkplugb.toml`
bereitzustellen.

## 6. Erweitern

Die Metriktabelle und die `NCMD`-Abbildung sind die **Quelle der Wahrheit** in
[`sparkplug_node.rs`](../../src/sparkplug_node.rs). Um eine Metrik hinzuzufügen: sie zu
`data_metrics`/`changed_metrics` (Lesen) und, falls steuerbar, zu `ncmd_to_actions`
(Schreiben → `Command`) hinzufügen, dann hier und in
[`reference_sparkplugb.md`](reference_sparkplugb.md) widerspiegeln. Einen Test im Modul
ergänzen.

## 7. Bemerkenswerte Abhängigkeiten

- `rumqttc` (MQTT-Client, rustls), `sparkplug-rs` (protobuf Tahu, reine Rust-Codegen).
- MSRV: nach einem vollständigen `cross`-Build zu prüfen (kann die Workspace-Untergrenze
  1.85 je nach rustls-Abhängigkeiten überschreiten).
