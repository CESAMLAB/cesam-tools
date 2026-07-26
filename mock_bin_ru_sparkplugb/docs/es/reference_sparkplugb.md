# Referencia MQTT Sparkplug B — métricas y ciclo de vida (RU/Sparkplug B)

*🌍 [FR](../fr/reference_sparkplugb.md) · [EN](../en/reference_sparkplugb.md) · [DE](../de/reference_sparkplugb.md) · **ES** · [IT](../it/reference_sparkplugb.md) · [PT](../pt/reference_sparkplugb.md) · [NL](../nl/reference_sparkplugb.md) · [PL](../pl/reference_sparkplugb.md)*

> Fuente de verdad: [`sparkplug_node.rs`](../../src/sparkplug_node.rs) (topics, tabla
> de métricas, payloads, mapeo NCMD). Toda evolución se hace **en este archivo** y se
> repercute aquí.

---

## 1. Rol y conexión

El instrumento es un **edge node Sparkplug B**: **no escucha en ningún puerto**, se
**conecta en salida** a un **broker MQTT externo** (mosquitto, EMQX, HiveMQ…) y
publica el estado del regulador. Ajustes en la sección `[network]` del TOML / el modal
*Parámetros*:

| Clave | Por defecto | Rol |
|---|---|---|
| `broker_host` | `localhost` | host del broker MQTT |
| `broker_port` | `1883` | puerto (`8883` en TLS) |
| `client_id` | `ru_spb` | identificador de cliente MQTT |
| `group_id` | `CESAM` | grupo Sparkplug (`spBv1.0/<group_id>/…`) |
| `edge_node_id` | `RU1` | nodo edge (`…/<edge_node_id>`) |
| `username` / `password` | *(vacío)* | auth MQTT (contraseña **en claro**, solo simulador) |
| `tls` | `false` | cifrado TLS (rustls) hacia el broker |
| `keepalive_secs` | `30` | keepalive MQTT |
| `publish_on_change` | `true` | `true`: `NDATA` en cuanto cambia una métrica (cadencia = paso de simulación, 0,5 s); `false`: periódico |
| `publish_period_secs` | `5` | cadencia periódica cuando `publish_on_change = false` |

> ⚠️ **MQTT en claro por defecto**: sin TLS, el tráfico no está cifrado ni autenticado
> en red. Usar únicamente sobre una **red de confianza**. La IHM muestra un banner de
> advertencia mientras `tls` esté desactivado.

---

## 2. Espacio de nombres (topics)

Namespace `spBv1.0`. Topics del nodo:

```
spBv1.0/<group_id>/NBIRTH/<edge_node_id>
spBv1.0/<group_id>/NDATA/<edge_node_id>
spBv1.0/<group_id>/NDEATH/<edge_node_id>
spBv1.0/<group_id>/NCMD/<edge_node_id>
```

Con los valores por defecto: `spBv1.0/CESAM/NBIRTH/RU1`, etc.

---

## 3. Tabla de métricas

Todas las métricas de datos viven bajo el **nodo edge** (sin *device* en esta
versión). Tipo Sparkplug (Eclipse Tahu): `Float` (9), `Boolean` (11), `UInt64` (8).

| Métrica | Tipo | Lectura/Escritura | Campo instantánea (lectura) | NCMD → comando (escritura) |
|---|---|:--:|---|---|
| `Setpoint` | Float | R/W | `setpoint` | `SetSetpoint` |
| `ProcessValue` | Float | R | `pv` | — |
| `Output` | Float | R | `output` | — |
| `ManualOutput` | Float | R/W | `manual_output` | `SetManualOutput` |
| `Run` | Boolean | R/W | `run` | `SetRun` |
| `Auto` | Boolean | R/W | `auto` | `SetAuto` |
| `SetpointMin` | Float | R | `sp_min` | *(ajustado vía IHM/TOML)* |
| `SetpointMax` | Float | R | `sp_max` | *(ajustado vía IHM/TOML)* |
| `PID/Kp` | Float | R | `pid.kp` | *(ajustado vía IHM/TOML)* |
| `PID/Ki` | Float | R | `pid.ki` | *(ajustado vía IHM/TOML)* |
| `PID/Kd` | Float | R | `pid.kd` | *(ajustado vía IHM/TOML)* |
| `bdSeq` | UInt64 | R | *(contador de sesión)* | — |
| `Node Control/Rebirth` | Boolean | W | — | republica un `NBIRTH` |

**Superficie pilotable por `NCMD`**: `Setpoint`, `ManualOutput`, `Run`, `Auto`, más
`Node Control/Rebirth` (paridad con las escrituras OPC UA del instrumento ORUE). Los
límites de consigna y las ganancias PID se **publican** (observables por un SCADA)
pero se ajustan vía la IHM/TOML. Una métrica desconocida o de **tipo erróneo** en un
`NCMD` se **ignora** (nunca un error, nunca un valor aberrante: la simulación sanea
toda escritura).

---

## 4. Ciclo de vida

- **`NBIRTH`** — publicado en cada conexión (ConnAck). Contiene **todas** las métricas
  (con valores), `bdSeq` y `Node Control/Rebirth`. `seq = 0`.
- **`NDATA`** — métricas **modificadas** únicamente, `seq` rodante **0–255**.
- **`NDEATH`** — contiene `bdSeq` **solo**, **sin** `seq`. Depositado como **Last Will
  MQTT** en la conexión: el **broker** lo publica automáticamente al perder el enlace
  (parada, reconfiguración, fallo). Sin `NDEATH` explícito del lado del nodo.
- **`NCMD`** — suscripción `spBv1.0/<group>/NCMD/<node>` (QoS 1) suscrita justo
  después del `NBIRTH`. Decodificado → comandos aplicados a la simulación.
- **`bdSeq`** — incrementado en cada (re)arranque del cliente; el `NDEATH` (Last Will)
  y el `NBIRTH` de una **misma sesión** llevan el **mismo** valor (invariante
  Sparkplug). Mostrado en la IHM (diagnóstico).
- **`seq`** — puesto a 0 en cada `NBIRTH`, incrementado (rodante) en cada `NDATA`.
- **Renacimiento** (`Node Control/Rebirth = true` vía `NCMD`) → republicación de un
  `NBIRTH` (resincronización SCADA).

---

## 5. Ejemplo cliente (SCADA)

Suscripción a todo el grupo, luego envío de una consigna:

```bash
# Observar los mensajes del nodo
mosquitto_sub -h localhost -t 'spBv1.0/CESAM/#' -v

# (los payloads son protobuf Sparkplug B — usar un decodificador Tahu para leerlos)
```

Un `NCMD` publicado en `spBv1.0/CESAM/NCMD/RU1` con las métricas `Run=true` y
`Setpoint=80.0` arranca la regulación y fija la consigna; un `NDATA` posterior refleja
el cambio.
