# Tabla de direcciones Modbus — Regulador simulado

*🌍 [FR](../fr/table_modbus.md) · [EN](../en/table_modbus.md) · [DE](../de/table_modbus.md) · **ES** · [IT](../it/table_modbus.md) · [PT](../pt/table_modbus.md) · [NL](../nl/table_modbus.md) · [PL](../pl/table_modbus.md)*

> Crate: `mock_bin_ru_modbustcp` · Protocolo: **Modbus TCP** (esclavo / servidor)

Este documento es la referencia funcional del plan de direccionamiento. La **fuente de
verdad técnica** sigue siendo la cabecera de [`src/map.rs`](../../src/map.rs): toda
divergencia debe corregirse en el código de forma prioritaria.

---

## 1. Generalidades

| Elemento | Valor |
|---------|--------|
| Transporte | Modbus **TCP** o **RTU serie / RS485** (uno solo activo a la vez) |
| Rol | **Esclavo** (servidor) |
| Puerto por defecto | TCP `5502` (configurable, modal *Parámetros*) |
| Serie (RTU) | puerto + baud + paridad + bits, configurables (feature `rtu`) |
| Unit ID / dirección | TCP: indiferente. RTU: `slave_id` configurable pero **no filtrado** (ver nota) |
| Maestros | **un solo maestro remoto a la vez**; en TCP un recién llegado desconecta al anterior (la IHM local no es un maestro) |
| Direccionamiento | **base 0** (la dirección `0` = 1.er elemento de la tabla) |
| Filtrado | lista blanca de IP opcional (comodines `*`, solo TCP) |

> **Nota RTU / dirección esclava**: el servidor RTU responde **sea cual sea
> la dirección** solicitada (la dirección no se transmite al servicio aplicativo).
> Usar un **enlace punto a punto**. El `slave_id` se conserva/muestra pero
> no realiza ningún filtrado.

### Direccionamiento base 0 vs base 1

Las direcciones siguientes son las **direcciones de protocolo (base 0)**, tal
como se envían en la trama. Muchas herramientas muestran una numeración base 1
«convencional» (`4xxxx` para los holdings, `3xxxx` para los inputs…). Así
el registro de mantenimiento de dirección `2` corresponde a la referencia convencional `40003`.

---

## 2. Codificación de los números flotantes (`f32`)

Las magnitudes analógicas son **`f32` IEE-754 en 2 registros consecutivos**:

- **orden de las palabras**: palabra de **mayor peso primero** (big-endian, llamado *ABCD*);
- **orden de los octetos** en cada registro: big-endian (estándar Modbus).

Ejemplo: `80.0` → octetos `42 A0 00 00` → registro `n` = `0x42A0`,
registro `n+1` = `0x0000`.

> Si su cliente lee valores aberrantes, casi siempre es un problema
> de orden de las palabras (probar *word swap* / *CDAB*).

---

## 3. Bobinas — *Coils* (lectura/escritura)

Códigos de función: `0x01` (lectura), `0x05` (escritura simple), `0x0F` (escritura múltiple).

| Dirección | Designación | Valores | Efecto |
|---------|-------------|---------|-------|
| `0` | Marcha / Paro | `0` = paro, `1` = marcha | Activa la regulación |
| `1` | Auto / Manual | `0` = manual, `1` = auto | Elección del modo |

---

## 4. Entradas discretas — *Discrete Inputs* (solo lectura)

Código de función: `0x02`.

| Dirección | Designación | Significado |
|---------|-------------|---------------|
| `0` | En marcha | El equipo está en marcha |
| `1` | Sentido 1 (calor) activo | Salida > 0 |
| `2` | Sentido 2 (frío) activo | Salida < 0 |

---

## 5. Registros de mantenimiento — *Holding Registers* (lectura/escritura)

Códigos de función: `0x03` (lectura), `0x06` (escritura simple), `0x10` (escritura múltiple).

| Dirección | Designación | Tipo | Unidad / valores |
|---------|-------------|------|-----------------|
| `0` | Modo de regulación sentido 1 (calor) | `u16` | `0`=Off, `1`=PID, `2`=TOR, `3`=PWM |
| `1` | Modo de regulación sentido 2 (frío) | `u16` | `0`=Off, `1`=PID, `2`=TOR, `3`=PWM |
| `2`–`3` | Consigna automática (SP) | `f32` | unidad de medida |
| `4`–`5` | Consigna manual | `f32` | % de salida, con signo (−100…+100) |
| `6`–`7` | `Kp` sentido 1 | `f32` | ganancia proporcional |
| `8`–`9` | `Ki` sentido 1 | `f32` | ganancia integral (s⁻¹) |
| `10`–`11` | `Kd` sentido 1 | `f32` | ganancia derivativa (s) |
| `12`–`13` | `Kp` sentido 2 | `f32` | ganancia proporcional |
| `14`–`15` | `Ki` sentido 2 | `f32` | ganancia integral (s⁻¹) |
| `16`–`17` | `Kd` sentido 2 | `f32` | ganancia derivativa (s) |
| `18`–`19` | Histéresis TOR | `f32` | unidad de medida |
| `20`–`21` | Tiempo de ciclo mínimo TOR | `f32` | segundos (anti-ciclo-corto, `0` = desactivado) |
| `22`–`23` | Período del ciclo PWM | `f32` | segundos (> 0) |
| `42`–`46` | Identificador de equipo | `ASCII` | «CESAM-Lab» (solo lectura, 2 car./registro, mayor peso primero) |

> Registros `24`–`41` reservados (leídos a `0`).

> **Escritura parcial de un `f32`**: hay que escribir **los dos registros** de un
> flotante para que se tenga en cuenta. Una escritura de un solo registro de un
> par `f32` se ignora (y devuelve la excepción *Illegal Data Address* si
> no recubre ningún campo válido).
>
> Las ganancias escritas se acotan a valores finitos ≥ 0 (robustez).

---

## 6. Registros de entrada — *Input Registers* (solo lectura)

Código de función: `0x04`.

| Dirección | Designación | Tipo | Unidad |
|---------|-------------|------|-------|
| `0`–`1` | Medida (PV — *process value*) | `f32` | unidad de medida |
| `2`–`3` | Salida aplicada | `f32` | % con signo (+ calor / − frío) |

---

## 7. Excepciones Modbus

| Código | Nombre | Causa en este equipo |
|------|-----|--------------------------|
| `0x01` | Illegal Function | Código de función no gestionado (ej. máscara, FIFO) |
| `0x02` | Illegal Data Address | Dirección / cantidad fuera de tabla, o escritura que no apunta a ningún campo |
| `0x04` | Server Device Failure | Bloqueo interno no disponible (caso anómalo) |

---

## 8. Ejemplos con `mbpoll`

`mbpoll` direcciona en **base 1**; por tanto se añade `1` a las direcciones base 0.

```bash
# Poner en marcha (bobina base0 0 -> -t 0 -r 1) y luego pasar a auto (bobina 1)
mbpoll -m tcp -p 5502 -a 1 -t 0 -r 1 127.0.0.1 1     # On/Off = 1
mbpoll -m tcp -p 5502 -a 1 -t 0 -r 2 127.0.0.1 1     # Auto/Manual = 1 (auto)

# Escribir la consigna auto (HR base0 2-3 -> -t 4:float -r 3) a 80.0
mbpoll -m tcp -p 5502 -a 1 -t 4:float -r 3 127.0.0.1 80.0

# Leer la medida PV (IR base0 0-1 -> -t 3:float -r 1)
mbpoll -m tcp -p 5502 -a 1 -t 3:float -r 1 127.0.0.1

# Leer la salida (IR base0 2-3 -> -t 3:float -r 3)
mbpoll -m tcp -p 5502 -a 1 -t 3:float -r 3 127.0.0.1
```

> Según las versiones de `mbpoll`, el orden de las palabras flotantes puede requerir
> la opción de permutación. En caso de valor incoherente, comprobar el orden de las palabras.

---

## 9. Mapa de memoria condensado

```
Coils (RW)            DiscreteInputs (RO)     Holding (RW)              Input (RO)
0  On/Off             0  En marcha            0  Modo sent1 (u16)       0-1 PV (f32)
1  Auto/Manual        1  Calor activo         1  Modo sent2 (u16)       2-3 Salida (f32)
                      2  Frío activo          2-3  SP auto (f32)
                                              4-5  SP manual (f32)
                                              6-7  Kp1  8-9  Ki1  10-11 Kd1
                                              12-13 Kp2 14-15 Ki2 16-17 Kd2
                                              18-19 Histéresis (f32)
                                              20-21 Ciclo mín. TOR (f32, s)
                                              22-23 Período PWM (f32, s)
                                              42-46 Identificador ASCII "CESAM-Lab"
```

> **Identificador ASCII** (`HR 42-46`): «CESAM-Lab» codificado 2 caracteres por
> registro, carácter de mayor peso primero (`42`=`'CE'`, `43`=`'SA'`, `44`=`'M-'`,
> `45`=`'La'`, `46`=`'b\0'`). Solo lectura. Ejemplo:
> `mbpoll -m tcp -p 5502 -a 1 -t 4 -r 43 -c 5 127.0.0.1` (registros base 1 43..47).
