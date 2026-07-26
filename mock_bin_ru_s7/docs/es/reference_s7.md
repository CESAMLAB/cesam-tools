# Referencia S7 — plan de direccionamiento y protocolo (RU/S7)

*🌍 [FR](../fr/reference_s7.md) · [EN](../en/reference_s7.md) · [DE](../de/reference_s7.md) · **ES** · [IT](../it/reference_s7.md) · [PT](../pt/reference_s7.md) · [NL](../nl/reference_s7.md) · [PL](../pl/reference_s7.md)*

> Fuente de verdad: [`s7_server.rs`](../../src/s7_server.rs) (análisis de las tramas,
> plan de direccionamiento DB1, mapeo de las escrituras). Toda evolución se realiza **en
> ese archivo** y se repercute aquí.

---

## 1. Endpoint

Servidor **S7comm** sobre **ISO-on-TCP / RFC1006**. Escucha por defecto en
`0.0.0.0:102` (puerto estándar S7; **< 1024 → derechos root** requeridos, en caso
contrario elegir un puerto alto). Ajustes en la sección `[network]` del TOML / el modal
*Parámetros*:

| Clave | Por defecto | Función |
|---|---|---|
| `bind_ip` | `0.0.0.0` | IP de escucha |
| `port` | `102` | puerto TCP (S7 estándar) |
| `allowlist` | *(vacía)* | lista blanca de IP (patrones `*` por byte; vacía = todo permitido) |

> ⚠️ **Ninguna autenticación ni cifrado** (S7 «classic»). El único control de
> acceso es la **lista blanca de IP** + la topología de red. `0.0.0.0` + lista vacía
> = **expuesto a toda la red**: la IHM muestra un banner de advertencia.

## 2. Sesiones

Al contrario que ORME (maestro único), el servidor S7 acepta **varias sesiones
cliente simultáneas** (comportamiento habitual de un autómata). Cada sesión negocia
COTP (Connection Request → Confirm) y luego S7 *Setup Communication*, antes de los
intercambios *Read Var* / *Write Var*.

## 3. Subconjunto de protocolo implementado

- **COTP**: Connection Request (CR) → Connection Confirm (CC); Data (DT).
- **S7comm**: *Setup Communication*, *Read Var* (función `0x04`), *Write Var*
  (función `0x05`) sobre el bloque de datos **DB1**.

El servidor expone una **imagen de bytes de DB1** (40 bytes). Las lecturas sirven
una porción de esa imagen; las escrituras sobre los offsets pilotables producen
comandos saneados para la simulación.

## 4. Plan de direccionamiento DB1

REAL = `f32` big-endian (IEEE-754). Direccionamiento por byte (`DBDx`) o por bit
(`DBXx.y`).

| Dirección | Tipo | Acceso | Magnitud | Escritura → comando |
|---|---|:--:|---|---|
| `DB1.DBD0`  | REAL | R/W | Consigna (Setpoint) | `SetSetpoint` |
| `DB1.DBD4`  | REAL | R   | Medida (ProcessValue) | — |
| `DB1.DBD8`  | REAL | R   | Salida (Output, %) | — |
| `DB1.DBD12` | REAL | R/W | Salida manual (ManualOutput, %) | `SetManualOutput` |
| `DB1.DBX16.0` | BOOL | R/W | Marcha (Run) | `SetRun` |
| `DB1.DBX16.1` | BOOL | R/W | Modo auto (Auto) | `SetAuto` |
| `DB1.DBD20` | REAL | R | Consigna mín | — |
| `DB1.DBD24` | REAL | R | Consigna máx | — |
| `DB1.DBD28` | REAL | R | PID Kp | — |
| `DB1.DBD32` | REAL | R | PID Ki | — |
| `DB1.DBD36` | REAL | R | PID Kd | — |

La escritura de `DB1.DBB16` (byte) se acepta: bit 0 = Run, bit 1 = Auto. Toda escritura
sobre un offset de solo lectura es **aceptada pero ignorada** (código de retorno éxito).
Una lectura/escritura fuera de DB1 devuelve el código de retorno S7 `0x0A` (objeto
inexistente).

## 5. Ejemplo de cliente

Con un cliente S7 (Snap7, `python-snap7`, nodes7…) configurado en la IP/puerto del
servidor, **rack 0 / slot 1** (valores habituales; el servidor no impone el TSAP):

```python
import snap7, struct
c = snap7.client.Client()
c.connect("127.0.0.1", 0, 1, 102)
c.db_write(1, 0, struct.pack(">f", 80.0))   # Consigna = 80.0
c.db_write(1, 16, bytes([0x01]))            # Run = true (bit 0)
pv = struct.unpack(">f", c.db_read(1, 4, 4))[0]  # Medida
```
