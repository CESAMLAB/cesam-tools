# Referencia OPC UA — espacio de direcciones (RU/OPC UA)

*🌍 [FR](../fr/reference_opcua.md) · [EN](../en/reference_opcua.md) · [DE](../de/reference_opcua.md) · **ES** · [IT](../it/reference_opcua.md) · [PT](../pt/reference_opcua.md) · [NL](../nl/reference_opcua.md) · [PL](../pl/reference_opcua.md)*

> Fuente de verdad: [`opcua_server.rs`](../../src/opcua_server.rs) (declaración de los
> nodos + callbacks). Toda evolución de la tabla se hace **en este archivo** y se
> refleja aquí.

---

## 1. Endpoint

| Elemento | Valor |
|---|---|
| URL | `opc.tcp://<bind_ip>:<port>/` (por defecto `opc.tcp://0.0.0.0:4840/`) |
| Transporte | OPC UA TCP binario |
| Política de seguridad | `None` |
| Modo de seguridad | `None` |
| Token de usuario | `Anonymous` |

⚠️ **Seguridad None**: ni autenticación ni cifrado (Fase 1b). Exponer
únicamente en una **red de confianza**. Seguridad real (`Basic256Sha256`, certificados,
auth) prevista en la **Fase 2**.

---

## 2. Namespace

| Índice | URI |
|---|---|
| `0` | `http://opcfoundation.org/UA/` (namespace núcleo OPC UA) |
| `ns` | `urn:cesam-lab:ru-opcua` (namespace de aplicación) |

El índice `ns` del namespace de aplicación se asigna dinámicamente al arrancar;
un cliente lo resuelve mediante `IN GetNamespaceArray` / el servicio *Browse*. Los nodos
de negocio descritos abajo residen allí.

---

## 3. Nodos (bajo la carpeta `Objects`)

Cada nodo es una `Variable`; su `NodeId` tiene la forma `ns=<ns>;s=<nombre>`.

| BrowseName | NodeId (`s=`) | Tipo | Acceso | Magnitud |
|---|---|---|:--:|---|
| `Setpoint` | `Setpoint` | `Double` | R/W | Consigna (unidad de proceso) |
| `ProcessValue` | `ProcessValue` | `Double` | R | Medida (PV) |
| `Output` | `Output` | `Double` | R | Salida de mando (%) |
| `ManualOutput` | `ManualOutput` | `Double` | R/W | Salida impuesta en modo manual (%) |
| `Run` | `Run` | `Boolean` | R/W | Marcha / parada de la regulación |
| `Auto` | `Auto` | `Boolean` | R/W | Modo automático (PID) vs manual |

- **Lecturas**: servidas por un callback que lee la **instantánea compartida**; son
  por tanto «vivas» y **muestreables** por las suscripciones (*Subscription*
  / *MonitoredItem*).
- **Escrituras**: encaminadas hacia el actor de simulación. Los valores se **sanean**
  (no finitos rechazados, consigna acotada, salida manual acotada a `[0, 100]`).

---

## 4. Mapeo hacia el estado de negocio

| Nodo | Efecto de una escritura | Fuente de una lectura |
|---|---|---|
| `Setpoint` | `Command::SetSetpoint` (acotada `[sp_min, sp_max]`) | `snapshot.setpoint` |
| `ManualOutput` | `Command::SetManualOutput` (acotada `[0, 100]`) | `snapshot.manual_output` |
| `Run` | `Command::SetRun` | `snapshot.run` |
| `Auto` | `Command::SetAuto` | `snapshot.auto` |
| `ProcessValue` | — (solo lectura) | `snapshot.pv` |
| `Output` | — (solo lectura) | `snapshot.output` |

Una escritura de un tipo inesperado devuelve `Bad_TypeMismatch`; una escritura sin
valor, `Bad_NothingToDo`. El `Float` se acepta además del `Double` para los
nodos numéricos.

---

## 5. Ejemplos (cliente OPC UA)

Con un cliente genérico (UaExpert, `opcua` CLI, etc.), conectarse a
`opc.tcp://127.0.0.1:4840/`, seguridad **None**, usuario **Anonymous**, y luego:

```text
# Lectura de la medida y de la consigna
Read  ns=<ns>;s=ProcessValue   → 60.0
Read  ns=<ns>;s=Setpoint       → 60.0

# Arranque + nueva consigna
Write ns=<ns>;s=Run        = true
Write ns=<ns>;s=Setpoint   = 80.0

# Cambio a manual y salida impuesta al 40 %
Write ns=<ns>;s=Auto         = false
Write ns=<ns>;s=ManualOutput = 40.0
```

Suscribirse (*Subscribe* / *MonitoredItem*) a `ProcessValue` y `Output` permite
seguir la dinámica del proceso en tiempo real.
