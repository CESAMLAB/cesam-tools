# Manual de usuario — Regulador de proceso simulado (RU/OPC UA)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · **ES** · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

> Crate: `mock_bin_ru_opcua` · Ejecutable: **ru_opcua**

---

## 1. Para qué sirve este simulador

`ru_opcua` simula un **regulador de proceso** (bucle PID sobre un proceso
térmico) y lo expone en **OPC UA**, el estándar de supervisión industrial.
Sirve para **probar un cliente OPC UA / un SCADA** (lectura de medidas, escritura de
consignas, suscripciones) sin hardware real.

La interfaz gráfica permite **pilotar** la simulación y **visualizar** la
dinámica; el servidor OPC UA expone las mismas magnitudes a la red.

---

## 2. Primeros pasos

```bash
cargo run -p mock_bin_ru_opcua          # IHM + servidor OPC UA
```

Al arrancar, el servidor escucha por defecto en `opc.tcp://0.0.0.0:4840/`
(seguridad None). La ventana muestra el estado actual y comienza la curva de
tendencia.

Conecte un cliente OPC UA (UaExpert, etc.) a `opc.tcp://127.0.0.1:4840/`,
seguridad **None**, usuario **Anonymous**. Los nodos se describen en la
[referencia OPC UA](reference_opcua.md).

---

## 3. La interfaz

### Cabecera

- **Título** y botones **⚙ Parámetros** / **💾 Guardar los ajustes**.
- A la derecha: **estado del aparato** (EN MARCHA / DETENIDO), **estado del servidor**
  (`OPC UA ● opc.tcp://…` en verde si está a la escucha, ✖ + mensaje en caso de error), y
  el **logo CESAM-Lab**.
- Un **banner naranja** recuerda permanentemente que el endpoint es **anónimo
  (seguridad None)**: exponer únicamente en red de confianza.
- Si hay una actualización disponible, un **banner** propone la descarga.

### Panel de mandos (izquierda)

- **Marcha / Parada**: arranca o detiene la regulación. Al parar, el proceso
  relaja hacia el valor ambiente.
- **Modo automático (PID)**: activado = el PID calcula la salida; desactivado =
  **modo manual** (la salida se impone).
- **Consigna**: deslizador, acotado por los límites de consigna (ajustables en
  *Parámetros*).
- **Salida manual (%)**: deslizador activo únicamente en **modo manual**.
- **Ajustes PID**: ganancias `Kp`, `Ki`, `Kd` editables en caliente.

### Zona central

- **Tarjetas**: Medida, Consigna, Salida.
- **Curva de tendencia**: Medida (PV) y Consigna en el eje izquierdo (unidad
  de proceso), Salida (%) en el eje derecho.

---

## 4. Parámetros (modal ⚙)

- **Idioma** de la interfaz (8 idiomas), persistido.
- **Verificar las actualizaciones al arrancar** + botón **Verificar ahora**.
- **Endpoint**: **IP de escucha** y **puerto** del servidor OPC UA. Un cambio
  **reinicia** el servidor en caliente (las sesiones en curso se cierran limpiamente).
- **Seguridad OPC UA**: **Cifrado** (`Basic256Sha256`), **Permitir anónimo**,
  **Usuario** / **Contraseña** (campos activos cuando el cifrado está marcado).
  Activar el cifrado genera un certificado en el primer arranque (algunos
  segundos) y reinicia el servidor.
- **Proceso (función de transferencia)**: ganancia `K`, constante de tiempo `τ`, retardo
  puro, valor ambiente.
- **Límites de consigna**: mín / máx (reordenados automáticamente si están invertidos).
- **Aplicar** / **Restablecer por defecto** / **Cerrar**.

Los ajustes se guardan en `mock_ru_opcua.toml` (directorio actual;
anulable mediante la variable de entorno `MOCK_CONFIG`).

---

## 5. Seguridad

La seguridad OPC UA es **configurable** en *Parámetros*:

- **Sin cifrado** (por defecto): endpoint **seguridad None**, acceso **anónimo** —
  ninguna protección. **No exponer en una red abierta.** Un banner **naranja**
  lo recuerda.
- **Con cifrado**: endpoint **`Basic256Sha256`** (firmado + cifrado). El
  servidor genera su certificado en el primer arranque y acepta los certificados
  de cliente. Se puede exigir un **usuario / contraseña** o permitir
  el anónimo. Un banner **verde 🔒** confirma el cifrado. Para conectarse, el
  cliente debe entonces usar la política `Basic256Sha256` y confiar en el
  certificado del servidor (primer intercambio).

La contraseña se almacena **en claro** en el archivo TOML: es un
**simulador**, para usar en una red de confianza.

---

## 6. Preguntas frecuentes

**¿Es obligatorio el puerto 4840?** No: se ajusta en *Parámetros* (o mediante el
archivo TOML). Un puerto < 1024 requiere privilegios de root.

**Mi cliente no ve los nodos.** Verifique la conexión a `opc.tcp://…:4840/`,
seguridad **None**, usuario **Anonymous**, y luego *Browse* bajo la carpeta
`Objects` (namespace `urn:cesam-lab:ru-opcua`).

**Se rechaza una escritura.** El tipo debe corresponder (`Double` para las
magnitudes, `Boolean` para `Run`/`Auto`); si no, el servidor devuelve
`Bad_TypeMismatch`.

**¿Ejecutar sin interfaz gráfica?** Compile en *headless*:
`cargo run -p mock_bin_ru_opcua --no-default-features` — el servidor OPC UA y la
simulación funcionan sin IHM.

**Aparece un mensaje «encrypted endpoints disabled».** Es normal en
Fase 1b: no se aprovisiona ningún certificado de instancia (endpoints cifrados
no disponibles). El endpoint None sí funciona.
