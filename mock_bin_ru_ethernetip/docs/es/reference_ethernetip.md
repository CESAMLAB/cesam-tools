# Referencia EtherNet/IP — tags y protocolo (RU/EtherNet/IP)

*🌍 [FR](../fr/reference_ethernetip.md) · [EN](../en/reference_ethernetip.md) · [DE](../de/reference_ethernetip.md) · **ES** · [IT](../it/reference_ethernetip.md) · [PT](../pt/reference_ethernetip.md) · [NL](../nl/reference_ethernetip.md) · [PL](../pl/reference_ethernetip.md)*

> Fuente de verdad: [`eip_server.rs`](../../src/eip_server.rs) (encapsulación,
> dispatch CIP, tabla de tags). Toda evolución se hace **en este archivo** y se
> refleja aquí.

---

## 1. Endpoint

Adaptador **EtherNet/IP** (mensajería explícita **CIP** no conectada) sobre TCP.
Escucha por defecto en `0.0.0.0:44818` (puerto estándar EtherNet/IP, > 1024 → no se
requiere ningún privilegio). Ajustes en la sección `[network]` del TOML / el modal
*Parámetros*:

| Clave | Por defecto | Función |
|---|---|---|
| `bind_ip` | `0.0.0.0` | IP de escucha |
| `port` | `44818` | puerto TCP (EtherNet/IP estándar) |
| `allowlist` | *(vacía)* | lista blanca de IP (patrones `*` por octeto; vacía = todo permitido) |

> ⚠️ **Ninguna autenticación ni cifrado** (EtherNet/IP «classic»). El único
> control de acceso es la **lista blanca de IP** + la topología de red. `0.0.0.0` +
> lista vacía = **expuesto**: la IHM muestra un banner de advertencia.

⚠️ EtherNet/IP / CIP es **little-endian** (al contrario que Modbus/S7). Los `REAL`
son `f32` IEEE-754 little-endian.

## 2. Sesiones

Se aceptan varios clientes **simultáneos**. Cada sesión: `RegisterSession`
(el servidor asigna un *session handle* no nulo) → `SendRRData` portando las solicitudes
CIP → `UnRegisterSession` (o desconexión TCP).

## 3. Subconjunto de protocolo implementado

- **Encapsulación**: `RegisterSession` (0x0065), `UnRegisterSession` (0x0066),
  `SendRRData` (0x006F, mensajería explícita no conectada, CPF).
- **CIP**: `Read Tag` (servicio 0x4C) y `Write Tag` (servicio 0x4D) sobre **tags
  nombrados** (segmento simbólico ANSI `0x91`).

## 4. Tabla de tags

| Tag | Tipo CIP | Acceso | Magnitud | Escritura → comando |
|---|---|:--:|---|---|
| `Setpoint` | REAL (0x00CA) | R/W | consigna | `SetSetpoint` |
| `ProcessValue` | REAL | R | medida | — |
| `Output` | REAL | R | salida (%) | — |
| `ManualOutput` | REAL | R/W | salida manual (%) | `SetManualOutput` |
| `Run` | BOOL (0x00C1) | R/W | marcha | `SetRun` |
| `Auto` | BOOL | R/W | modo auto | `SetAuto` |
| `SetpointMin` | REAL | R | consigna mín. | — |
| `SetpointMax` | REAL | R | consigna máx. | — |
| `Kp` / `Ki` / `Kd` | REAL | R | ganancias PID | — |

Un tag conocido de **solo lectura** que se escribe es **aceptado** (estado CIP éxito) pero sin
efecto; un **tag desconocido** devuelve el estado CIP `0x05` (*path destination unknown*).
Toda escritura controlable es **acotada/saneada** por la simulación.

## 5. Ejemplo de cliente

Con un cliente EtherNet/IP (p. ej. `pycomm3`, `rseip`, `rust-ethernet-ip`) apuntando
a la IP/puerto del servidor, los tags se leen/escriben por su nombre:

```python
from pycomm3 import CIPDriver  # o LogixDriver según la herramienta
# Leer la medida, escribir la consigna y arrancar la regulación:
#   read  Tag "ProcessValue" (REAL)
#   write Tag "Setpoint" = 80.0 (REAL)
#   write Tag "Run" = True (BOOL)
```

El servidor responde a los servicios genéricos Read/Write Tag direccionados por segmento
simbólico ANSI; no expone un árbol de objetos CIP más allá de los tags
anteriores.
