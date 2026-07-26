# Diseño — Regulador Sparkplug B (ORSE)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · **ES** · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

---

## 1. Visión de conjunto

ORSE reutiliza la arquitectura de los demás instrumentos CESAM-Lab: un **modelo de
negocio síncrono y comprobable** (regulador PID + proceso), gobernado por **actores
`ractor`** sobre Tokio, y una **IHM `egui`** que lee una instantánea compartida. Solo
cambia la **capa de transporte**: aquí, un **edge node MQTT Sparkplug B** (cliente
saliente) en lugar de un servidor Modbus/OPC UA.

```
        Command (cast)                      refresh en cada paso
IHM   ───────────────────►  SimulationActor ───────────────────►  SharedSnapshot (IHM)
NCMD (broker) ───────────►  (Regulator)      ──────────────────►  SharedSnapshot (publicación)
NBIRTH/NDATA (broker) ◄──────────────────────  SharedSnapshot
```

## 2. Actores

- **`SimulationActor`** — posee el único [`Regulator`]. Bucle de paso fijo (`Tick`
  cada 0,5 s); aplica los `Command` (IHM o NCMD); publica la instantánea después de
  cada mutación. Idéntico a los demás instrumentos.
- **`SparkplugActor`** — posee el **cliente MQTT** (`rumqttc`) y ejecuta el **ciclo
  de vida Sparkplug B** en una tarea tokio dedicada (cuyo `JoinHandle` se aborta al
  detenerse). Un mensaje `Reconfigure` reinicia el cliente si cambian el broker, los
  identificadores o TLS.

## 3. Capa de protocolo

[`sparkplug_node.rs`](../../src/sparkplug_node.rs) es **puro y síncrono** (sin
dependencia de tokio/rumqttc): construcción de los **topics**, tabla de **métricas**,
fabricación de los **payloads** (`NBIRTH`/`NDATA`/`NDEATH`), (des)serialización
protobuf, mapeo **`NCMD` → comandos** y el contador `seq`. Es el equivalente del
`opcua_server.rs` de ORUE, aislado para ser **comprobable sin broker**.

### Elección de las bibliotecas

- **`rumqttc`** — cliente MQTT async Tokio (Last Will, reconexión automática, TLS vía
  rustls — ya presente en el árbol vía OPC UA, **sin OpenSSL**).
- **`sparkplug-rs`** — structs protobuf Eclipse Tahu (`Payload`/`Metric`/`Value`),
  generados en **100 % Rust** (rust-protobuf, **sin `protoc`** → cross limpio). La
  crate reexporta `protobuf` (runtime), usado para `write_to_bytes`/`parse_from_bytes`.
- **Alternativa descartada: `srad`** — marco de alto nivel de edge node Sparkplug que
  gestiona él mismo `bdSeq`/`seq`/rebirth. Descartado deliberadamente: **poseemos** la
  máquina de estados en el actor de red para hacerla explícita y comprobable
  (coherencia con los demás instrumentos).

## 4. Ciclo de vida e invariantes

- **`bdSeq`** incrementado en cada (re)arranque del cliente; **el mismo** valor en el
  Last Will `NDEATH` y en el `NBIRTH` de una sesión.
- **`seq`** rodante 0–255, puesto a 0 en cada `NBIRTH`.
- **`NDEATH`** llevado por el **Last Will MQTT**: robusto ante cualquier pérdida de
  enlace.
- **Publicación `NDATA`** por **diff** de instantánea (cadencia = paso de simulación en
  modo *al cambiar*, o periódica). El cerrojo de la instantánea **nunca** se mantiene a
  través de un `.await`.

## 5. Postura de seguridad

- **Sin lista blanca de IP** (el instrumento es un cliente, no un servidor): desvío de
  paridad **asumido** con ORME/OSNE.
- **MQTT en claro por defecto** (puerto 1883) — sin cifrar, sin autenticación de red.
  Banner de advertencia en la IHM. Activar **TLS** + identificadores para salir de una
  red de confianza.
- **Contraseña en claro** en el TOML — **solo simulador**.
- **Saneamiento TOML** ([`AppConfig::sanitized`](../../src/config.rs)): proceso/PID/
  límites finitos y ordenados, identificadores Sparkplug no vacíos, temporizaciones
  acotadas. Toda escritura NCMD es **acotada/saneada** por `Regulator::apply`.
