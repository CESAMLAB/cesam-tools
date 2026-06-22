# Manual de usuario — OSNE (agitador de laboratorio simulado NAMUR)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · **ES** · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

> **OSNE** — *Open Stirrer NAMUR Emulator* · binario `mock_bin_su_namur`
> (ejecutable `osne`) · Licencia MIT · Editor: **CESAM-Lab** · Identidad NAMUR:
> nombre `CESAM-STIRRER`, tipo `OSNE`.
>
> *Un agitador de laboratorio (estilo IKA) que solo existe en su enlace NAMUR —
> para probar supervisores, scripts y pasarelas sin equipo real.*

Este manual está dirigido al **usuario** del agitador simulado: cómo lanzarlo,
controlarlo desde la interfaz, configurarlo y conectarlo en **NAMUR** (TCP o serie
RS-232). No se necesita ningún conocimiento de programación.

---

## 1. ¿Para qué sirve este software?

Simula un **agitador de laboratorio** (agitador de sobremesa de hélice, estilo
IKA):

- un **motor físico** realista: la velocidad sube/baja según el par aplicado, con
  una **regulación de velocidad rápida**;
- una **carga viscosa ajustable**: cuanto más viscoso es el medio, mayor es el par
  necesario — hasta la **sobrecarga** (consigna inalcanzable);
- un **servidor NAMUR** (protocolo serie ASCII de los equipos de laboratorio) para
  controlarlo/supervisarlo desde otro software o un script;
- una **interfaz gráfica** de conducción, de visualización y de **prueba del
  protocolo** (mini-terminal NAMUR integrado).

Es una herramienta de **prueba**: permite poner a punto y demostrar un supervisor,
un script de adquisición o una pasarela **sin equipo real**.

---

## 2. Iniciar el software

Lanzar el ejecutable correspondiente a su sistema:

| Sistema | Archivo |
|---------|---------|
| Windows | `osne-windows-x86_64.exe` (doble clic) |
| Linux PC | `./osne-linux-x86_64` |
| Raspberry Pi (pantalla) | `./osne-rpi-arm64` |

La ventana se abre y el **servidor NAMUR arranca automáticamente** (puerto `4001`
por defecto). El encabezado indica el estado:

- **● EN MARCHA / ● DETENIDO**: estado del motor;
- **NAMUR ● 0.0.0.0:4001** (verde): servidor a la escucha; **✖ …** (rojo) en caso
  de problema (puerto ocupado, serie no disponible…);
- un **piloto de conexión**: en TCP muestra el maestro conectado (o «ningún
  maestro»), en serie un simple punto. Pasa a **verde** cuando se ha recibido una
  trama recientemente (enlace activo), gris si no.

> Sin pantalla (solo servidor), ver el **§ 9 (Uso sin pantalla)**.

---

## 3. La interfaz de un vistazo

```
┌──────────────── En-tête : titre OSNE, ⚙ Paramètres, 💾 Sauvegarder, états & voyants ────────────────┐
├──────────────────┬──────────────────────────────────────────────────────────────────────────────────┤
│  COMMANDES        │   SUPERVISION                                                                      │
│  (gauche)         │   - cartes de valeurs (Vitesse / Couple / Viscosité / Surcharge)                  │
│  Marche/Arrêt     │   - COURBE de tendance temps réel (Consigne / Vitesse / Couple)                   │
│  Consigne vitesse │                                                                                   │
│  Viscosité        │                                                                                   │
│  Réglages PID     │                                                                                   │
├──────────────────┴──────────────────────────────────────────────────────────────────────────────────┤
│  ⇄ TRAMES NAMUR : mini-terminal (RX/TX) + ligne de commande + référence du protocole (à droite)       │
└───────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 4. Controlar el agitador (panel de la izquierda)

### 4.1 Marcha / Parada
Botón **Marcha / Parada**. Al detenerse, el motor decelera libremente hasta
inmovilizarse (rozamiento + carga), con par motor nulo.

### 4.2 Consigna de velocidad
Deslizador **Consigna de velocidad** (en `tr/min`), acotado por los límites
mín/máx ajustados en los *Parámetros*. Es la misma magnitud que el comando NAMUR
`OUT_SP_4` (canal 4). En marcha, el lazo de regulación lleva la velocidad medida
hacia esa consigna.

### 4.3 Viscosidad del medio
Deslizador **Viscosidad** (escala logarítmica). Representa la **carga** del medio
agitado:

- viscosidad **baja** → par bajo, la consigna se alcanza rápidamente;
- viscosidad **elevada** → par de carga importante; si el par necesario supera el
  **par motor máximo**, la velocidad de consigna **deja de alcanzarse** → el
  indicador **Sobrecarga ⚠** se enciende (comportamiento de un agitador real ante
  un medio demasiado espeso).

### 4.4 Ajustes PID (Kp, Ki, Kd)
Las tres ganancias del lazo de regulación de velocidad, ajustables en directo:

- **Kp** (proporcional): cuanto mayor es, más viva es la subida de velocidad
  (riesgo de sobreoscilación/oscilación);
- **Ki** (integral): anula la desviación residual de velocidad con el tiempo;
- **Kd** (derivativo): amortigua/anticipa (demasiado fuerte → sensible al ruido).

> Las ganancias por defecto son deliberadamente «duras»: la salida satura al par
> máximo mientras el error es grande (subida rápida), y luego el término integral
> estabiliza. La salida del PID **es** el par motor, acotado a `[0, couple_max]`.

---

## 5. Leer la curva de tendencia

La curva (en el centro) traza tres magnitudes en tiempo real. La **leyenda, arriba
a la izquierda**, recuerda el color **y el último valor** de cada serie:

| Color | Serie | Significado |
|---------|-------|---------------|
| 🔵 azul | **Consigna** | consigna de velocidad (en marcha) |
| 🔴 rojo | **Velocidad** | velocidad medida (`tr/min`, eje izquierdo) |
| 🟢 verde | **Par** | par medido (`N·cm`, **eje derecho**) |

> La curva tiene **dos ejes verticales**: la **velocidad** (`tr/min`) a la
> izquierda, el **par** (`N·cm`) a la derecha. El par se escala para compartir el
> gráfico, pero el eje derecho muestra efectivamente `N·cm`.

Encima de la curva, unas **tarjetas** muestran los valores instantáneos:
**Velocidad**, **Par**, **Viscosidad**, y **Sobrecarga ⚠** cuando el motor satura.
Se puede hacer zoom/desplazar la curva con el ratón.

---

## 6. El mini-terminal NAMUR (parte inferior de la ventana)

El panel **⇄ Tramas NAMUR** permite **probar el protocolo** directamente desde la
IHM, sin cliente externo:

- el **registro** muestra las tramas **recibidas** (`← RX`, azul) y **emitidas**
  (`→ TX`, verde), con marca de tiempo;
- la **línea de comando** envía una trama NAMUR al simulador (tecla **Intro** o
  botón **▶ Enviar**). Las flechas **↑/↓** recuperan los comandos anteriores
  (historial);
- la **referencia del protocolo** (panel de la derecha) lista los comandos: un
  **clic** inserta el comando en la línea de entrada;
- el botón **🗑 Borrar** vacía el registro.

> Las tramas tecleadas aquí se interpretan exactamente como las de un maestro de
> red: `OUT_SP_4 500` ajusta la consigna, `START_4`/`STOP_4` arrancan/detienen,
> `IN_PV_4` lee la velocidad, etc. El **perro guardián** (`OUT_WD1@…`) solo surte
> efecto, no obstante, dentro de una verdadera sesión de red (cf. § 8).

---

## 7. Parámetros (botón ⚙)

El botón **⚙ Parámetros** abre una ventana para configurar:

### Idioma de la interfaz
Selector en la parte superior: **Français, English, Deutsch, Español, Italiano,
Português, Nederlands, Polski** (8 idiomas). El idioma se conserva.

### Transporte NAMUR
Elección del enlace — **uno solo activo a la vez**:

**TCP (Ethernet)**
- **IP de escucha** (`0.0.0.0` = todas las interfaces) y **Puerto** (defecto 4001);
- **IP autorizadas**: una por línea, comodines `*` aceptados (ej. `192.168.1.*`).
  **Lista vacía = todas las IP autorizadas.** Las demás se rechazan.

**Serie (RS-232)** — requiere un binario compilado con la feature `serial`
- **Puerto serie**: `/dev/ttyUSB0` (Linux), `COM3` (Windows)…;
- **Baud** (defecto 9600), **Paridad** (defecto Par), **Bits de datos** (7),
  **Bits de stop** (1) — ajuste NAMUR de laboratorio típico: **9600 7E1**.

> ⚠️ **Un solo maestro a la vez.** En TCP, un nuevo maestro **espera** hasta la
> desconexión del anterior (enlace punto a punto). La IHM local **no** es un
> maestro. En serie, el bus *es* el maestro único; conviene un **enlace punto a
> punto** (el servidor responde sea cual sea la dirección solicitada).

### Parámetros del motor
Comportamiento físico simulado `J·dω/dt = T − k·η·ω − rozamiento`:
- **Inercia** (`J`): reactividad del motor (pequeño ⇒ rápido);
- **Coeficiente de carga** (`k`): peso de la viscosidad sobre el par;
- **Rozamiento** (`N·cm`): rozamiento seco residual;
- **Par máx** (`N·cm`): par motor máximo (techo de la salida PID).

### Límites de velocidad
Límites mín/máx de la consigna de velocidad (`tr/min`).

### Límites de viscosidad
Límites mín/máx del deslizador de viscosidad.

Botones: **Aplicar** (surte efecto inmediatamente **y** guarda),
**Restablecer por defecto**, **Cerrar**.

### Guardado de los ajustes
Los ajustes se **guardan** en un archivo `mock_su_namur.toml` (junto al software) y
se **recargan en el siguiente arranque**. El botón **💾 Guardar** del encabezado
guarda también las ganancias PID y la viscosidad modificadas desde el panel de la
izquierda.

---

## 8. Conectar un cliente NAMUR

El software es un **esclavo NAMUR** (TCP puerto 4001 por defecto, o serie según el
transporte elegido en el § 7). Un cliente (script, terminal, pasarela) **envía una
línea ASCII por petición**, terminada en `CR LF`. Las **lecturas** (`IN_*`)
devuelven un valor; las **escrituras/acciones** (`OUT_*`, `START_*`, `STOP_*`,
`RESET`) son **silenciosas** (sin respuesta), conforme al uso NAMUR.

Referencias principales:

| Comando | Efecto |
|----------|-------|
| `IN_NAME` / `IN_TYPE` | identidad (`CESAM-STIRRER` / `OSNE`) |
| `IN_PV_4` / `IN_PV_5` | leer la velocidad (`tr/min`) / el par (`N·cm`) |
| `IN_SP_4` | leer la consigna de velocidad |
| `OUT_SP_4 <v>` | **ajustar** la consigna de velocidad |
| `START_4` / `STOP_4` / `RESET` | arrancar / detener / reiniciar |
| `OUT_WD1@<m>` | **perro guardián**: parada segura si hay silencio durante `<m>` s |

Ejemplo con `nc` (netcat):

```text
$ nc 127.0.0.1 4001
IN_NAME
CESAM-STIRRER
OUT_SP_4 1200          (silencieux)
START_4                (silencieux)
IN_PV_4
1200.0 4
STOP_4                 (silencieux)
```

> El **perro guardián** `OUT_WD1@30` detiene automáticamente el motor si **no llega
> ninguna línea** durante 30 s (protección en caso de pérdida de comunicación).
> `OUT_WD1@0` lo desarma. El contador se rearma con cada comando recibido.

> La **referencia completa del protocolo** (canales, codificación, ejemplos) está
> en **[commandes_namur.md](commandes_namur.md)**. La misma lista se recuerda **en
> directo** en el panel de la derecha del mini-terminal.

---

## 9. Uso sin pantalla («headless» / Docker)

Para un despliegue en segundo plano (Raspberry Pi sin pantalla, servidor), existe
una versión **sin interfaz**: ejecuta la simulación y el servidor NAMUR,
controlables **únicamente por NAMUR**.

```bash
# Image Docker (déployable n'importe où) :
docker run --rm -p 4001:4001 -v "$PWD/conf:/data" osne:headless
```

La carpeta montada en `/data` permite proporcionar/conservar `mock_su_namur.toml`.

---

## 10. Preguntas frecuentes

| Pregunta / síntoma | Respuesta |
|---------------------|---------|
| **Sobrecarga ⚠** se enciende y la velocidad no alcanza la consigna. | Normal: la **viscosidad** exige más par del que el motor proporciona. Baje la viscosidad o la consigna, o aumente el **par máx** (Parámetros). |
| La velocidad no se mueve. | Compruebe que el agitador está **En marcha** y la consigna no es nula. |
| El encabezado muestra **NAMUR ✖**. | Puerto ya en uso o < 1024 sin privilegios (TCP), o puerto serie no disponible: cambie el ajuste en ⚙ Parámetros. |
| Mi cliente NAMUR/TCP es rechazado. | Su IP no está en la **lista blanca**: vacíe la lista o añada un patrón (`192.168.1.*`). |
| `OUT_SP_4 …` no devuelve nada. | Normal: las escrituras/acciones NAMUR son **silenciosas**. Lea con `IN_SP_4` / `IN_PV_4`. |
| El motor se detiene solo. | Hay un **perro guardián** armado (`OUT_WD1@…`) y no llegó ningún comando a tiempo. Desármelo (`OUT_WD1@0`) o envíe tramas con regularidad. |
| El enlace serie no se abre. | Binario compilado **sin** la feature `serial`, o puerto/permisos incorrectos (grupo `dialout` en Linux). |
| Mis ajustes no se conservan. | Pulse **Aplicar** / **💾 Guardar**. El archivo `mock_su_namur.toml` debe ser accesible para escritura. |

---

*Documentación técnica asociada: [conception.md](conception.md) ·
[commandes_namur.md](commandes_namur.md) · [maintenance.md](maintenance.md).*
