# Mantenimiento — Regulador Sparkplug B (ORSE)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · **ES** · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

---

## 1. Build y lanzamiento

```bash
cargo run -p mock_bin_ru_sparkplugb                       # IHM + edge node
cargo build -p mock_bin_ru_sparkplugb --release           # ejecutable IHM
cargo build -p mock_bin_ru_sparkplugb --no-default-features # headless (sin IHM)
```

Features: `gui` (IHM `egui`, por defecto). `--no-default-features` produce un binario
**headless**: edge node Sparkplug B + simulación, sin IHM ni verificación de
actualizaciones.

## 2. Configuración

Archivo TOML `mock_ru_sparkplugb.toml` (directorio actual; ruta sobreescribible por
`MOCK_CONFIG`). Secciones: `language`, `[network]` (broker/Sparkplug), `[process]`,
`[regulation]`, `check_updates`. Véase [`reference_sparkplugb.md`](reference_sparkplugb.md)
para las claves `[network]`. Todo valor es **saneado** al cargar.

## 3. Pruebas

```bash
cargo test -p mock_bin_ru_sparkplugb              # unitarias (sin broker)
cargo test -p mock_bin_ru_sparkplugb -- --ignored # round-trip con broker local
```

- **Unitarias** (sin red): regulación, saneamiento de config y, sobre todo, la capa
  Sparkplug (topics, payloads `NBIRTH`/`NDEATH`, round-trip encode/decode, mapeo
  `NCMD`, rechazo de tipo erróneo, vuelta del `seq` 255→0).
- **Integración `#[ignore]`**: requiere un broker MQTT local —
  `docker run -it --rm -p 1883:1883 eclipse-mosquitto` — y luego lanza el round-trip
  completo (NBIRTH recibido, NCMD aplicado, NDATA reflejado).

## 4. Resolución de problemas

| Síntoma | Pista |
|---|---|
| «Desconectado» permanente | broker inalcanzable (`broker_host`/`broker_port`, cortafuegos, broker detenido) |
| El SCADA no recibe nada | `group_id`/`edge_node_id`; suscripción `spBv1.0/<group>/#`; payloads protobuf |
| Fallo TLS | broker en TLS sobre 8883; certificado raíz reconocido por el sistema |
| NCMD ignorado | métrica no pilotable o tipo erróneo (cf. tabla de métricas) |

## 5. Docker (headless)

La imagen headless se construye vía `scripts/build-prod.sh` (entrada
`mock_bin_ru_sparkplugb:ru_spb:0`). Al ser ORSE un **cliente**, **no expone ningún
puerto** (`PORT=0`, `EXPOSE 0` = metadato inerte) y **ningún `HEALTHCHECK`** TCP es
pertinente: la liveness se constata del lado del broker vía el **Last Will/NDEATH**.
Montar un volumen sobre el directorio de trabajo para proporcionar el
`mock_ru_sparkplugb.toml`.

## 6. Extender

La tabla de métricas y el mapeo `NCMD` son la **fuente de verdad** en
[`sparkplug_node.rs`](../../src/sparkplug_node.rs). Para añadir una métrica:
añadirla a `data_metrics`/`changed_metrics` (lectura) y, si es pilotable, a
`ncmd_to_actions` (escritura → `Command`), luego reflejarlo aquí y en
[`reference_sparkplugb.md`](reference_sparkplugb.md). Añadir una prueba en el módulo.

## 7. Dependencias notables

- `rumqttc` (cliente MQTT, rustls), `sparkplug-rs` (protobuf Tahu, codegen Rust puro).
- MSRV: por verificar tras un build `cross` completo (puede superar el mínimo 1.85 del
  workspace según las dependencias rustls).
