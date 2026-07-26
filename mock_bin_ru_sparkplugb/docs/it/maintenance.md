# Manutenzione — Regolatore Sparkplug B (ORSE)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · **IT** · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

---

## 1. Build e avvio

```bash
cargo run -p mock_bin_ru_sparkplugb                       # IHM + edge node
cargo build -p mock_bin_ru_sparkplugb --release           # eseguibile IHM
cargo build -p mock_bin_ru_sparkplugb --no-default-features # headless (senza IHM)
```

Feature: `gui` (IHM `egui`, predefinita). `--no-default-features` produce un binario
**headless**: edge node Sparkplug B + simulazione, senza IHM né verifica degli
aggiornamenti.

## 2. Configurazione

File TOML `mock_ru_sparkplugb.toml` (directory corrente; percorso sovrascrivibile con
`MOCK_CONFIG`). Sezioni: `language`, `[network]` (broker/Sparkplug), `[process]`,
`[regulation]`, `check_updates`. Vedere [`reference_sparkplugb.md`](reference_sparkplugb.md)
per le chiavi `[network]`. Ogni valore viene **sanificato** al caricamento.

## 3. Test

```bash
cargo test -p mock_bin_ru_sparkplugb              # unitari (senza broker)
cargo test -p mock_bin_ru_sparkplugb -- --ignored # round-trip con broker locale
```

- **Unitari** (senza rete): regolazione, sanificazione della config, e soprattutto lo
  strato Sparkplug (topic, payload `NBIRTH`/`NDEATH`, round-trip encode/decode,
  mapping `NCMD`, rifiuto del tipo errato, riavvolgimento del `seq` 255→0).
- **Integrazione `#[ignore]`**: richiede un broker MQTT locale —
  `docker run -it --rm -p 1883:1883 eclipse-mosquitto` — quindi esegue il round-trip
  completo (NBIRTH ricevuto, NCMD applicato, NDATA riflesso).

## 4. Risoluzione dei problemi

| Sintomo | Pista |
|---|---|
| «Disconnesso» permanente | broker irraggiungibile (`broker_host`/`broker_port`, firewall, broker arrestato) |
| Lo SCADA non riceve nulla | `group_id`/`edge_node_id`; sottoscrizione `spBv1.0/<group>/#`; payload protobuf |
| Errore TLS | broker in TLS su 8883; certificato radice riconosciuto dal sistema |
| NCMD ignorato | metrica non pilotabile o tipo errato (cfr. tabella delle metriche) |

## 5. Docker (headless)

L'immagine headless si costruisce tramite `scripts/build-prod.sh` (voce
`mock_bin_ru_sparkplugb:ru_spb:0`). Essendo ORSE un **client**, **non espone alcuna
porta** (`PORT=0`, `EXPOSE 0` = metadato inerte) e **nessun `HEALTHCHECK`** TCP è
pertinente: la liveness si constata lato broker tramite il **Last Will/NDEATH**.
Montare un volume sulla directory di lavoro per fornire il `mock_ru_sparkplugb.toml`.

## 6. Estendere

La tabella delle metriche e il mapping `NCMD` sono la **fonte di verità** in
[`sparkplug_node.rs`](../../src/sparkplug_node.rs). Per aggiungere una metrica:
aggiungerla a `data_metrics`/`changed_metrics` (lettura) e, se pilotabile, a
`ncmd_to_actions` (scrittura → `Command`), poi rifletterla qui e in
[`reference_sparkplugb.md`](reference_sparkplugb.md). Aggiungere un test nel modulo.

## 7. Dipendenze notevoli

- `rumqttc` (client MQTT, rustls), `sparkplug-rs` (protobuf Tahu, codegen Rust puro).
- MSRV: da verificare dopo un build `cross` completo (può superare il minimo 1.85 del
  workspace a seconda delle dipendenze rustls).
