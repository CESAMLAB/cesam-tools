# Referencia PROFIBUS DP-V0 — Regulador simulado (ORPD)

*🌍 [FR](../fr/reference_profibus.md) · [EN](../en/reference_profibus.md) · [DE](../de/reference_profibus.md) · **ES** · [IT](../it/reference_profibus.md) · [PT](../pt/reference_profibus.md) · [NL](../nl/reference_profibus.md) · [PL](../pl/reference_profibus.md)*

> Crate: `mock_bin_ru_pbdp` · Ejecutable: **ru_pbdp** · Protocolo: **PROFIBUS DP-V0** (esclavo serie)

Este documento es la referencia funcional del subconjunto PROFIBUS DP-V0
simulado. La **fuente de verdad técnica** sigue siendo la cabecera de
[`src/profibus.rs`](../../src/profibus.rs) (códec + máquina de estados) y
de [`src/map.rs`](../../src/map.rs) (bloques de E/S): cualquier
discrepancia debe corregirse en el código en primer lugar.

---

## ⚠️ 0. Alcance y límites — leer antes de cualquier uso

`ru_pbdp` implementa un **subconjunto educativo** de DP-V0, **sin ninguna
pretensión de conformidad binaria estricta** con las tablas normativas
(IEC 61158 / EN 50170) más allá de los elementos más universalmente
documentados:

- **conformes**: delimitadores de trama (`SD1`/`SD2`/`SD3`/`SD4`/`SC`/`ED`),
  FCS (suma módulo 256), números SAP de los servicios de parametrización
  (`Slave_Diag` = 61, `Set_Prm` = 62, `Chk_Cfg` = 63).
- **convenciones propias de este simulador, no un perfil GSD real
  registrado en el PNO** (PROFIBUS & PROFINET International): codificación
  exacta de los bits del campo `FC`, disposición precisa de los bytes de
  diagnóstico, disposición de los bloques de entrada/salida (§3), el
  identificador `Ident_Number` (§4).
- **ningún temporizado de bus real**: ni ventana de respuesta (*slot time*,
  `Tsdr` mín/máx), ni testigo entre maestros, ni arbitraje multi-maestro.
  Solo un ASIC dedicado (SPC3/VPC3) o una tarjeta maestra hardware
  (Hilscher/Softing/Siemens CP) pueden cumplir estas restricciones a nivel
  de bit.

**Consecuencia directa: este simulador nunca será reconocido por un
maestro PROFIBUS DP real** (autómata + tarjeta maestra). Sirve para
comprender la estructura del protocolo y probar un desarrollo software
(códec, máquina de estados, herramientas), no para pilotar equipos de
campo — véase [`manuel_utilisateur.md`](manuel_utilisateur.md).

---

## 1. Tramas — delimitadores y FCS

| Delimitador | Valor | Uso |
|---|:--:|---|
| `SD1` | `0x10` | Petición fija sin datos (6 bytes: `SD1 DA SA FC FCS ED`) |
| `SD2` | `0x68` | Trama de longitud variable con datos (`SD2 LE LEr SD2 DA SA FC [datos…] FCS ED`) |
| `SD3` | `0xA2` | Trama de datos fijos, 8 bytes (14 bytes en total) — **no utilizada** por este simulador (véase §0), incluida por completitud del códec y sus pruebas |
| `SD4` | `0xDC` | Trama de testigo, 3 bytes, sin FCS ni ED — fuera de alcance para un esclavo mono-maestro simulado, incluida por completitud del códec |
| `SC` | `0xE5` | Acuse de recibo corto, 1 byte |
| `ED` | `0x16` | Delimitador de fin |

- **`FCS`**: suma módulo 256 de los bytes útiles de la trama (véase
  `profibus::checksum`). Una trama recibida con un FCS incorrecto se
  rechaza (`FrameError::BadChecksum`) sin respuesta — el maestro debe
  retransmitir.
- **`DA`/`SA`**: dirección destino / origen. Bit 7 de `DA` = **extensión de
  dirección (DAE)**: presencia de un byte de SAP justo después de `DA` en
  la carga útil. Ausente = intercambio de datos por defecto
  (`Data_Exchange`). La dirección de estación ocupa los 7 bits restantes
  (`0`-`125`; `126`/`127` reservadas por la norma, no utilizadas aquí).
- **Este simulador favorece sistemáticamente `SD2`** para todos los
  intercambios `Data_Exchange`, incluso cuando `SD3` (8 bytes fijos)
  bastaría en un perfil real — elección que simplifica el códec sin perder
  cobertura de los conceptos del protocolo (véase
  [`conception.md`](conception.md) §4).
- **Trama mal formada / delimitador desconocido (ruido de línea)**:
  rechazada silenciosamente (`log::debug!`), la sesión continúa — permite
  resincronizar el flujo de bytes sin colapsar el enlace.

---

## 2. Secuenciación — servicios y máquina de estados

El esclavo simulado (`SlaveFsm`, [`profibus.rs`](../../src/profibus.rs))
atraviesa cuatro estados:

```
PowerOn ──Slave_Diag──► WaitPrm ──Set_Prm (id OK)──► WaitCfg ──Chk_Cfg (tamaños OK)──► DataExchange
```

| Estado | Significado | Respuesta típica |
|---|---|---|
| `Power_On` | Justo tras el arranque, antes de la primera consulta de diagnóstico | — |
| `Wait_Prm` | Esperando un `Set_Prm` válido | `Diag` con `Stat_1 = STAT1_PRM_REQ` |
| `Wait_Cfg` | Parametrizado, esperando un `Chk_Cfg` válido | `Diag` con `Stat_1 = STAT1_CFG_FAULT` |
| `Data_Exchange` | Parametrizado y configurado: intercambio cíclico activo | bloque de entrada (§3) |

### `Slave_Diag` (SAP 61)

Petición sin datos (o trama `SD1`, siempre interpretada como `Slave_Diag`
por convención de este simulador — ninguna extensión de dirección posible
en `SD1`, al no haber byte disponible para portar un SAP). Respuesta
`Diag` (6 bytes):

| Byte | Símbolo | Contenido |
|:--:|---|---|
| `0` | `Stat_1` | `0x01` (`STAT1_PRM_REQ`, mientras no esté parametrizado) o `0x02` (`STAT1_CFG_FAULT`, mientras no esté configurado) o `0x00` (`Data_Exchange`) |
| `1` | `Stat_2` | siempre `0x00` (no simulado) |
| `2` | `Stat_3` | siempre `0x00` (no simulado) |
| `3` | `Master_Add` | `0xFF` (ningún maestro conocido — no rastreado por este simulador) |
| `4-5` | `Ident_Number` | identificador fijo del esclavo, big-endian (§4) |

El primer `Slave_Diag` recibido hace pasar `Power_On` → `Wait_Prm`; los
siguientes no cambian el estado (solo una lectura de diagnóstico).

### `Set_Prm` (SAP 62)

Petición: `SAP(62) Ident_Number(2, BE) WD_Fact_1(1) WD_Fact_2(1)`. El
vigilante anunciado, si está presente, se calcula como
`watchdog_ms = WD_Fact_1 × WD_Fact_2 × 10` (unidad 10 ms, convención
estándar DP); `WD_Fact_1 = 0` **o** `WD_Fact_2 = 0` significa «sin
vigilante». Respuesta: `ShortAck` (`SC`) en todos los casos.

- Si `Ident_Number` **coincide** con el perfil fijo del esclavo (§4):
  estado → `Wait_Cfg`, y el eventual vigilante se transmite a la sesión
  (armado solo si el ajuste local `watchdog_enabled` lo permite — véase
  [`manuel_utilisateur.md`](manuel_utilisateur.md) §4).
- Si el identificador **no coincide**: la parametrización se rechaza
  silenciosamente (se devuelve `ShortAck` de todos modos, como prescribe
  DP-V0 para este servicio, pero sin efecto sobre el estado interno) — el
  esclavo permanece en `Wait_Prm`.

### `Chk_Cfg` (SAP 63)

Petición: `SAP(63) Out_Len(1) In_Len(1)`. Respuesta: `ShortAck`. El estado
pasa a `Data_Exchange` **solo si** `Out_Len == 45` e `In_Len == 17`
(tamaños fijos del perfil simulado, §3) **y** el esclavo estaba en
`Wait_Cfg`; en caso contrario el estado no cambia (el maestro debe
retransmitir un `Chk_Cfg` correcto).

### `Data_Exchange` (sin SAP — dirección por defecto, bit DAE ausente)

Petición: el bloque de salida en bruto (45 bytes, §3). Respuesta: el
bloque de entrada (17 bytes, §3), recalculado al vuelo desde la instantánea
compartida en el momento de responder (sin tabla de memoria persistente, a
diferencia de Modbus/ORME).

Si el maestro envía un `Data_Exchange` **antes** de alcanzar el estado
`Data_Exchange` (secuenciación no respetada), el esclavo responde con el
diagnóstico actual (`Diag`) en lugar de colapsar o ignorar la trama.

---

## 3. Bloques de E/S — disposición de bytes

Copiado de la cabecera de [`map.rs`](../../src/map.rs), única fuente de
verdad en caso de discrepancia. Todos los valores en coma flotante (`f32`)
ocupan **4 bytes consecutivos, big-endian**.

### Bloque de salida — *Output* (maestro → esclavo, `OUTPUT_LEN` = 45 bytes)

| Byte(s) | Símbolo | Tipo | Descripción |
|---|---|:--:|---|
| `0` | `OUT_MODE` | bits | bit0 = marcha, bit1 = auto, [3:2] = modo sentido 1, [5:4] = modo sentido 2 |
| `1-4` | `OUT_SP_AUTO` | f32 | Consigna automática |
| `5-8` | `OUT_SP_MANUAL` | f32 | Consigna manual (% salida, con signo) |
| `9-12` | `OUT_KP1` | f32 | Ganancia proporcional Kp sentido 1 |
| `13-16` | `OUT_KI1` | f32 | Ganancia integral Ki sentido 1 |
| `17-20` | `OUT_KD1` | f32 | Ganancia derivativa Kd sentido 1 |
| `21-24` | `OUT_KP2` | f32 | Ganancia proporcional Kp sentido 2 |
| `25-28` | `OUT_KI2` | f32 | Ganancia integral Ki sentido 2 |
| `29-32` | `OUT_KD2` | f32 | Ganancia derivativa Kd sentido 2 |
| `33-36` | `OUT_HYSTERESIS` | f32 | Histéresis de los reguladores todo-o-nada |
| `37-40` | `OUT_TOR_MIN_CYCLE` | f32 | Tiempo de ciclo mínimo todo-o-nada (s) |
| `41-44` | `OUT_PWM_PERIOD` | f32 | Periodo del ciclo de modulación PWM (s) |

Los códigos de modo (`[3:2]`/`[5:4]`) siguen `ControllerKind`: `0` = Off,
`1` = PID, `2` = Todo-o-nada, `3` = PWM (véase `mock_lib_control`).

### Bloque de entrada — *Input* (esclavo → maestro, `INPUT_LEN` = 17 bytes)

| Byte(s) | Símbolo | Tipo | Descripción |
|---|---|:--:|---|
| `0` | `IN_STATUS` | bits | bit0 = en marcha, bit1 = sentido 1 activo (salida > 0), bit2 = sentido 2 activo (salida < 0) |
| `1-4` | `IN_PV` | f32 | Medida / *process value* |
| `5-8` | `IN_OUTPUT` | f32 | Salida aplicada (% con signo) |
| `9-12` | `IN_SP_AUTO` | f32 | Reflejo (solo lectura) de la consigna automática |
| `13-16` | `IN_SP_MANUAL` | f32 | Reflejo (solo lectura) de la consigna manual |

Un bloque de salida **demasiado corto** (< 45 bytes) se ignora sin
colapsar: no se produce ningún `Command`, el regulador conserva su último
estado válido.

---

## 4. Perfil fijo del esclavo

| Parámetro | Valor | Observación |
|---|---|---|
| `Ident_Number` | `0xEE01` | **Ficticio**, no registrado ante el PNO — no representa ningún dispositivo de catálogo real |
| `Out_Len` | `45` | Esperado en `Chk_Cfg.out_len` |
| `In_Len` | `17` | Esperado en `Chk_Cfg.in_len` |
| Dirección de estación | `0`-`125`, configurable | Ajuste local (modal *Ajustes*), véase [`manuel_utilisateur.md`](manuel_utilisateur.md) §4 |
| Formato de trama serie | `8E1` (8 bits, paridad par, 1 bit de parada) | **Fijado por la norma PROFIBUS DP**, no ajustable |
| Velocidades normalizadas | `9600` a `12.000.000` bit/s | No verificado al abrir: un valor no estándar se transmite tal cual al puerto serie |

---

## 5. Vigilante de protocolo

A diferencia del vigilante NAMUR de OSNE (añadido casero), este es una
**parte real del protocolo DP**: es **anunciado por el maestro** en
`Set_Prm` (factores `WD_Fact_1`/`WD_Fact_2`, §2) y solo se **arma del lado
del esclavo** si el ajuste local `watchdog_enabled` lo permite (en caso
contrario la solicitud del maestro se ignora, nunca se arma). Al vencer, sin
haber recibido una nueva trama para la estación, el esclavo fuerza el
estado seguro (`Command::SetOnOff(false)`) — simplificación documentada:
un perfil DP-V0 real podría exigir un retorno completo mediante
`Set_Prm`/`Chk_Cfg` antes de reanudar el intercambio, algo que este
simulador no exige explícitamente (basta con reanudar el envío de tramas
`Data_Exchange`, ya que el estado `Data_Exchange` no se abandona por el
vencimiento del vigilante).

---

## 6. No-interoperabilidad — por qué

| Requisito del PROFIBUS DP real | Este simulador |
|---|---|
| Ventana de respuesta a nivel de bit (*slot time*, `Tsdr` mín/máx) | Ausente — responde en cuanto la trama se decodifica, sin restricción de tiempo |
| Circuito dedicado (ASIC SPC3/VPC3) para el temporizado | Ausente — software Tokio ordinario |
| Testigo entre maestros, arbitraje multi-maestro | Ausente — esclavo mono-maestro, enlace punto a punto |
| Perfil GSD registrado ante el PNO | Ausente — perfil de E/S propio de este simulador (§3) |
| Codificación bit a bit exacta de los campos FC/diagnóstico | Convención de simulación, no garantizada conforme |

**Un autómata real (un Siemens S7 con tarjeta maestra, por ejemplo) nunca
reconocerá este simulador como esclavo válido en un bus real
PROFIBUS DP RS-485.** Dos instancias de este simulador (o un script que
reproduzca la secuencia siguiente), en cambio, pueden dialogar entre sí
para ilustrar el protocolo — véase
[`manuel_utilisateur.md`](manuel_utilisateur.md) §5.

---

## 7. Ejemplo de secuencia (hexadecimal)

Secuencia completa estación `5`, maestro `3`, hasta el intercambio cíclico
(valores ilustrativos, `FCS` calculado sobre los bytes útiles):

```text
# 1. Slave_Diag (SD2, DAE=1, SAP=61)
→ TX  68 03 03 68 85 03 C0 3D FC 16
← RX  68 06 06 68 03 85 00 01 00 00 FF EE 01 F5 16   (Diag: Stat_1=0x01, Ident=0xEE01)

# 2. Set_Prm (SD2, DAE=1, SAP=62, Ident=0xEE01, WD=1×30×10ms=300ms)
→ TX  68 07 07 68 85 03 C0 3E EE 01 01 1E … 16
← RX  E5                                              (ShortAck)

# 3. Chk_Cfg (SD2, DAE=1, SAP=63, out_len=45, in_len=17)
→ TX  68 05 05 68 85 03 C0 3F 2D 11 … 16
← RX  E5                                              (ShortAck)

# 4. Data_Exchange (SD2, sin SAP, bloque de salida de 45 bytes)
→ TX  68 30 30 68 05 03 C0 [45 bytes] … 16
← RX  68 14 14 68 03 85 00 [17 bytes]  … 16          (bloque de entrada)
```

Los bytes exactos de FCS/longitud dependen de los valores de carga útil;
este esquema ilustra el **orden de los servicios**, no una trama para
reproducir literalmente. Véanse las pruebas de
[`profibus.rs`](../../src/profibus.rs) y
[`profibus_server.rs`](../../src/profibus_server.rs) para secuencias
verificadas bit a bit.
