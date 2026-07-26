# Manual de usuario — Regulador Sparkplug B (ORSE)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · **ES** · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

---

## 1. Para qué sirve el instrumento

**ORSE** simula una **unidad de regulación** de proceso (PID + proceso térmico de
primer orden) y publica su estado en **MQTT Sparkplug B**, como un **edge node** que
se conecta a un **broker** y expone métricas a un SCADA. Sirve para probar una cadena
de adquisición Sparkplug B (Ignition, Chariot, EMQX, Node-RED…) sin material real.

## 2. Requisito previo: un broker MQTT

Al ser ORSE un **cliente**, hace falta un broker MQTT alcanzable. En local:

```bash
docker run -it --rm -p 1883:1883 eclipse-mosquitto
```

## 3. Primeros pasos

```bash
cargo run -p mock_bin_ru_sparkplugb        # IHM + edge node Sparkplug B
```

Al arrancar, la IHM intenta conectarse al broker (`localhost:1883` por defecto). El
encabezado indica el estado: **Conectado** (verde) una vez publicado el `NBIRTH`, o
**Desconectado** (rojo) con el motivo. Un banner naranja **⚠ MQTT en claro** recuerda
la ausencia de TLS.

## 4. Interfaz

- **Encabezado**: título, botones *Parámetros* / *Guardar*, estado marcha/parada,
  estado de conexión Sparkplug B, banner TLS/claro.
- **Panel izquierdo (Comandos)**: *Marcha/Parada*, *Modo automático (PID)*, *Consigna*,
  *Salida manual* (modo manual), ajustes **PID** (Kp/Ki/Kd).
- **Panel central**: tarjetas *Medida / Consigna / Salida* + **curva** en tiempo real.
- **Modal *Parámetros***: idioma, verificación de actualizaciones, **Broker MQTT /
  Sparkplug B** (host, puerto, client_id, group_id, edge_node_id, keepalive, TLS,
  usuario/contraseña, publicación al cambiar/periódica), **proceso** (K, τ, retardo,
  ambiente), **límites de consigna**. *Aplicar* reinicia la conexión y guarda el TOML.

## 5. Pilotar desde un SCADA

El SCADA se suscribe a `spBv1.0/<group_id>/#` y recibe `NBIRTH` y luego `NDATA`. Para
**comandar** el regulador, publica un `NCMD` en
`spBv1.0/<group_id>/NCMD/<edge_node_id>` con las métricas pilotables (`Setpoint`,
`Run`, `Auto`, `ManualOutput`) o `Node Control/Rebirth = true` para forzar un
renacimiento. Detalles: [`reference_sparkplugb.md`](reference_sparkplugb.md).

## 6. FAQ

- **«Desconectado» de forma permanente** → broker inalcanzable: verificar
  `broker_host`/`broker_port`, el cortafuegos y que el broker esté en marcha.
- **El SCADA no ve nada** → verificar el `group_id`/`edge_node_id` y la suscripción
  `spBv1.0/<group>/#`; los payloads son **protobuf** (se requiere un decodificador
  Sparkplug).
- **Mis escrituras NCMD se ignoran** → métrica no pilotable o tipo erróneo (cf. tabla
  de métricas). Solo se aceptan `Setpoint`/`Run`/`Auto`/`ManualOutput` y `Rebirth`.
- **¿Dónde está el archivo de configuración?** → `mock_ru_sparkplugb.toml` (directorio
  actual; sobreescribible por `MOCK_CONFIG`).
