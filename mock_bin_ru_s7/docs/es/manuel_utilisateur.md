# Manual de usuario — Regulador S7 (ORSS)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · **ES** · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

---

## 1. ¿Para qué sirve el instrumento?

**ORSS** simula una **unidad de regulación** de proceso (PID + proceso térmico de
primer orden) y la expone como un **autómata Siemens S7** (servidor S7comm sobre
ISO-on-TCP). Sirve para probar una supervisión o un cliente S7 (Snap7, TIA Portal en
lectura, nodes7…) sin un autómata real.

## 2. Primeros pasos

```bash
cargo run -p mock_bin_ru_s7        # IHM + servidor S7
```

El servidor escucha por defecto en `0.0.0.0:102`. ⚠️ El **puerto 102 requiere
derechos root**; en caso contrario, configure un puerto alto (p. ej. 1102) en el modal
*Parámetros*.

El encabezado indica el estado: **S7 ●** (verde) con la dirección de escucha, o un
mensaje de error (rojo) si el bind falla. Un banner naranja advierte si el servidor está
**expuesto** (todas las interfaces + lista blanca vacía).

## 3. Interfaz

- **Encabezado**: título, botones *Parámetros* / *Guardar*, estado marcha/parada, estado
  de escucha S7, banner de exposición de red.
- **Panel izquierdo (Comandos)**: *Marcha/Parada*, *Modo automático (PID)*,
  *Consigna*, *Salida manual* (modo manual), ajustes **PID** (Kp/Ki/Kd).
- **Panel central**: tarjetas *Medida / Consigna / Salida* + **curva** en tiempo real.
- **Modal *Parámetros***: idioma, verificación de actualizaciones, **red S7** (IP de
  escucha, puerto, **lista blanca** de IP — un patrón por línea, `*` = comodín),
  **proceso** (K, τ, retardo, ambiente), **límites de consigna**. *Aplicar* reinicia la
  escucha si cambia la IP/puerto y guarda el TOML.

## 4. Conectar un cliente S7

El cliente se conecta a la IP/puerto del servidor. Los valores **rack/slot** habituales
(0/1 o 0/2) funcionan: el servidor no impone TSAP. Las magnitudes están en
**DB1** (ver [`reference_s7.md`](reference_s7.md)): consigna en `DB1.DBD0`, medida
en `DB1.DBD4`, marcha en `DB1.DBX16.0`, etc.

## 5. Preguntas frecuentes

- **«Permission denied» al arrancar** → el puerto 102 exige derechos root;
  utilice un puerto alto o lance con los privilegios adecuados.
- **El cliente no se conecta** → verificar IP/puerto, la **lista blanca**, el
  cortafuegos. Probar rack/slot 0/1 y luego 0/2.
- **Mis escrituras no tienen efecto** → solo los offsets pilotables actúan
  (consigna, salida manual, marcha, auto); los demás son de solo lectura.
- **¿Dónde está el archivo de configuración?** → `mock_ru_s7.toml` (directorio actual;
  reemplazable mediante `MOCK_CONFIG`).
