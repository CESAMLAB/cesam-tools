# Manual de usuario — Regulador EtherNet/IP (OREE)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · **ES** · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

---

## 1. Para qué sirve el instrumento

**OREE** simula una **unidad de regulación** de proceso (PID + proceso térmico de
primer orden) y la expone como un **adaptador EtherNet/IP** (mensajería explícita
CIP). Sirve para probar una supervisión o un cliente EtherNet/IP (pycomm3, RSLinx en
lectura, rseip…) sin material real.

## 2. Primeros pasos

```bash
cargo run -p mock_bin_ru_ethernetip        # IHM + adaptador EtherNet/IP
```

El servidor escucha por defecto en `0.0.0.0:44818` (no se requiere ningún privilegio). El encabezado
indica el estado: **EtherNet/IP ●** (verde) con la dirección de escucha, o un mensaje
de error (rojo). Un banner naranja advierte si el servidor está **expuesto** (todas las
interfaces + lista blanca vacía).

## 3. Interfaz

- **Encabezado**: título, botones *Parámetros* / *Guardar*, estado marcha/parada, estado
  de escucha EtherNet/IP, banner de exposición de red.
- **Panel izquierdo (Comandos)**: *Marcha/Parada*, *Modo automático (PID)*,
  *Consigna*, *Salida manual* (modo manual), ajustes **PID** (Kp/Ki/Kd).
- **Panel central**: tarjetas *Medida / Consigna / Salida* + **curva** en tiempo real.
- **Modal *Parámetros***: idioma, verificación de actualizaciones, **red EtherNet/IP** (IP
  de escucha, puerto, **lista blanca** de IP — un patrón por línea, `*` = comodín),
  **proceso** (K, τ, retardo, ambiente), **límites de consigna**. *Aplicar* reinicia la
  escucha si cambia la IP/puerto y guarda el TOML.

## 4. Conectar un cliente EtherNet/IP

El cliente se conecta a la IP/puerto del servidor (`RegisterSession` automático), luego
lee/escribe los **tags nombrados** por mensajería explícita: `Setpoint`, `ProcessValue`,
`Output`, `ManualOutput`, `Run`, `Auto`, etc. (ver
[`reference_ethernetip.md`](reference_ethernetip.md)). ⚠️ Los valores están en
**little-endian** (REAL = `f32` LE).

## 5. Preguntas frecuentes

- **El cliente no se conecta** → verificar IP/puerto (44818), la **lista blanca**,
  el cortafuegos.
- **Tag no encontrado** → solo existen los tags documentados; los nombres distinguen
  mayúsculas/minúsculas.
- **Mis escrituras no tienen efecto** → solo actúan los tags controlables
  (`Setpoint`, `ManualOutput`, `Run`, `Auto`); los demás son de solo lectura.
- **¿Dónde está el archivo de configuración?** → `mock_ru_ethernetip.toml` (directorio actual;
  reemplazable por `MOCK_CONFIG`).
