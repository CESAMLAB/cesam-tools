# Juego de comandos NAMUR — Agitador simulado (OSNE)

*🌍 [FR](../fr/commandes_namur.md) · [EN](../en/commandes_namur.md) · [DE](../de/commandes_namur.md) · **ES** · [IT](../it/commandes_namur.md) · [PT](../pt/commandes_namur.md) · [NL](../nl/commandes_namur.md) · [PL](../pl/commandes_namur.md)*

> Crate: `mock_bin_su_namur` · Ejecutable: **OSNE** · Protocolo: **NAMUR** (ASCII, esclavo)

Referencia funcional del protocolo. La **fuente de verdad técnica** es la cabecera
de [`src/namur.rs`](../../src/namur.rs).

---

## 1. Generalidades

| Elemento | Valor |
|---------|--------|
| Transporte | **TCP** (puerto `4001` por defecto) o **serie RS-232** (feature `serial`) |
| Rol | **Esclavo** (responde a las peticiones del maestro) |
| Trama | una **línea ASCII** por petición, terminada en `CR LF` |
| Lecturas | `IN_*` → devuelven `valor canal` (ej. `1200.0 4`) |
| Escrituras / acciones | `OUT_*`, `START_*`, `STOP_*`, `RESET` → **silenciosas** (sin respuesta) |
| Maestros | **uno solo a la vez** (punto a punto); en TCP un nuevo maestro espera hasta la desconexión del anterior |
| Filtrado | lista blanca de IP opcional (TCP) |

> Ajuste serie NAMUR típico: **9600 baudios, 7 bits, paridad par, 1 stop (7E1)**.

### Canales

| Canal | Magnitud | Unidad |
|-------|----------|-------|
| `4` | Velocidad | tr/min |
| `5` | Par | N·cm |

---

## 2. Comandos

| Comando | Tipo | Efecto | Respuesta |
|----------|------|-------|---------|
| `IN_NAME` | lectura | Nombre del equipo | `CESAM-STIRRER` |
| `IN_TYPE` | lectura | Tipo de equipo | `OSNE` |
| `IN_SW_VERSION` | lectura | Versión del firmware simulado | ej. `0.1.0` |
| `IN_PV_4` | lectura | Velocidad **medida** | `<v> 4` |
| `IN_PV_5` | lectura | Par **medido** | `<c> 5` |
| `IN_SP_4` | lectura | Consigna de velocidad | `<v> 4` |
| `OUT_SP_4 <v>` | escritura | **Ajustar** la consigna de velocidad (tr/min) | — |
| `START_4` | acción | Arrancar el motor | — |
| `STOP_4` | acción | Detener el motor | — |
| `RESET` | acción | Parada + retorno a control local | — |
| `OUT_WD1@<m>` | escritura | **Perro guardián**: parada segura si no hay comandos durante `<m>` s | — |
| `OUT_WD2@<m>` | escritura | Perro guardián (igual que v1: parada segura) | — |

> Todo comando desconocido o argumento inválido se **ignora** (sin respuesta) y se
> registra en `debug`.

### Perro guardián

Tras `OUT_WD1@30`, si **no llega ninguna línea** durante 30 s, el motor se
**detiene** (`STOP`) automáticamente — protección en caso de pérdida de
comunicación con el supervisor. `OUT_WD1@0` desarma el perro guardián. El contador
se **rearma con cada comando recibido**.

---

## 3. Ejemplos (`nc` / netcat)

```text
$ nc 127.0.0.1 4001
IN_NAME
CESAM-STIRRER
OUT_SP_4 1200          (silencieux)
START_4                (silencieux)
IN_PV_4
1200.0 4
IN_PV_5
62.0 5
STOP_4                 (silencieux)
```

> El **par** leído crece con la **viscosidad** ajustada (lado IHM) y la velocidad:
> `par ≈ coef_carga · viscosidad · velocidad + rozamiento`. A alta viscosidad, el
> par satura al máximo del motor: la velocidad de consigna deja de alcanzarse
> (**sobrecarga**), comportamiento que reproduce un agitador real.
