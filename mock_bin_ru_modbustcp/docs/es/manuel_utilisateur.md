# Manual de usuario — ORME (regulador simulado Modbus)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · **ES** · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

> **ORME** — *Open Regulator Modbus Emulator* · binario `mock_bin_ru_modbustcp` ·
> Licencia MIT · Editor: **CESAM-Lab** · Identificador de equipo Modbus: **CESAM-Lab**
>
> *«Abra el bus.»* Un regulador de campo que solo existe en su bus
> Modbus (TCP/RTU) — para probar SCADA, autómatas e IHM sin hardware real.

Este manual está dirigido al **usuario** del regulador simulado: cómo lanzarlo,
controlarlo desde la interfaz, configurarlo y conectarlo en Modbus TCP.
No se necesita ningún conocimiento de programación.

---

## 1. ¿Para qué sirve este software?

Simula un **regulador industrial** (tipo horno o baño termostatizado):

- un **proceso físico** realista (la «medida» sube/baja según el comando);
- una **regulación** automática o manual, en **calor** y/o en **frío**;
- un **servidor Modbus TCP** para controlarlo/supervisarlo desde otro software
  (autómata, SCADA, pasarela…);
- una **interfaz gráfica** de conducción y de visualización.

Es una herramienta de **prueba**: permite poner a punto y demostrar un
supervisor o un autómata **sin hardware real**.

---

## 2. Iniciar el software

Lanzar el ejecutable correspondiente a su sistema:

| Sistema | Archivo |
|---------|---------|
| Windows | `orme-windows-x86_64.exe` (doble clic) |
| Linux PC | `./orme-linux-x86_64` |
| Raspberry Pi (pantalla) | `./orme-rpi-arm64` |

La ventana se abre y el **servidor Modbus arranca automáticamente** (puerto `5502`
por defecto). El encabezado indica el estado:

- **● EN MARCHA / ● DETENIDO**: estado del equipo;
- **Modbus ● 0.0.0.0:5502** (verde): servidor a la escucha; **✖ …** (rojo) en caso
  de problema de red.

> Sin pantalla (solo servidor), ver el **§ 9 (Uso sin pantalla)**.

---

## 3. La interfaz de un vistazo

La ventana consta de cuatro zonas:

```
┌───────────────────────────── Encabezado: título, ⚙ Parámetros, 💾 Guardar, estados ─────────────────────────────┐
├──────────────────┬─────────────────────────────────────────────────┬───────────────────────────────────────────┤
│  COMANDOS         │   SUPERVISIÓN                                   │   TABLA DE DIRECCIONES MODBUS             │
│  (izquierda)      │   - valores instantáneos (Medida / Consigna /   │   (derecha)                               │
│  Marcha/Paro      │     Salida)                                     │   lista en vivo: designación, tabla,      │
│  Auto/Manual      │   - CURVA de tendencia en tiempo real           │   dirección, valor, acceso                │
│  Modos, consignas │                                                 │                                           │
│  ajustes PID…     │                                                 │                                           │
└──────────────────┴─────────────────────────────────────────────────┴───────────────────────────────────────────┘
```

---

## 4. Controlar el regulador (panel de la izquierda)

### 4.1 Marcha / Paro
Botón **Marcha / Paro**. Detenido, la salida es nula y la medida vuelve
suavemente hacia el valor ambiente.

### 4.2 Auto / Manual
- **Manual**: *usted* impone la salida mediante la **consigna manual** (en %).
- **Auto**: el regulador calcula la salida para alcanzar la **consigna auto**.

### 4.3 Las consignas
Cada consigna dispone de un **campo numérico** (entrada precisa con el teclado) y
de un **deslizador**. Ambos son siempre modificables; la consigna **activa**
(según el modo) se muestra en negrita.

| Consigna | Unidad | Rol |
|----------|-------|------|
| **SP auto** | unidad de medida (ej. °C) | objetivo a alcanzar en modo Auto |
| **SP manual** | % de salida, de −100 a +100 | salida impuesta en modo Manual (**+** calor / **−** frío) |

### 4.4 Modos de regulación — sentido 1 (calor) y sentido 2 (frío)
Cada sentido se ajusta de forma independiente:

- **Desactivado** — el sentido no actúa;
- **PID** — regulación continua (salida 0…100 %), precisa y suave;
- **Todo-o-nada (TOR)** — relé con histéresis: salida 0 % o 100 %, simple pero
  oscilante alrededor de la consigna;
- **Relé de ciclo (PWM)** — un PID calcula un ciclo de trabajo, *troceado* sobre un
  período fijo: la salida física permanece todo-o-nada (0/100 %), pero su
  **media** sigue al PID. Es el mejor compromiso para controlar finamente un
  órgano que solo sabe abrirse o cerrarse (relé, válvula TOR).

> 👉 **Importante — ver **§ 6 (Comprender la regulación)****: elegir
> PID/TOR/PWM para el frío *arma* el frío, pero este solo **entrega cuando
> la medida supera la consigna**.

### 4.5 Ajustes PID (Kp, Ki, Kd)
Para cada sentido, tres ganancias ajustables en directo:

- **Kp** (proporcional): cuanto mayor sea, más viva es la reacción (riesgo de oscilación);
- **Ki** (integral): anula la desviación residual con el tiempo (demasiado fuerte → sobreoscilación);
- **Kd** (derivativo): amortigua/anticipa (demasiado fuerte → sensible al ruido).

### 4.6 Ajustes TOR / PWM
- **Histéresis TOR** — anchura de la **zona muerta** del modo Todo-o-nada, centrada
  en la consigna (`[SP − h/2, SP + h/2]`): evita que la salida conmute sin
  parar. Cuanto más ancha, mayor es la ondulación pero las conmutaciones
  más espaciadas.
- **Ciclo mín. TOR (s)** — duración mínima durante la cual el relé permanece en un
  estado antes de poder reconmutar (**anti-ciclo-corto**). Protege un actuador
  real (relé, compresor) y suaviza el comportamiento. `0` = desactivado.
- **Período PWM (s)** — duración de un ciclo del **relé de ciclo**. Corto → media
  más fiel pero conmutaciones frecuentes; largo → menos desgaste pero ondulación
  más marcada. A elegir mucho más pequeño que la constante de tiempo del proceso.

---

## 5. Leer la curva de tendencia

La curva (en el centro) traza en tiempo real tres magnitudes. La **leyenda, arriba
a la izquierda**, recuerda el color **y el último valor** de cada serie:

| Color | Serie | Significado |
|---------|-------|---------------|
| 🔵 azul | **Consigna (SP)** | objetivo (en Auto) |
| 🔴 rojo | **Medida (PV)** | valor del proceso |
| 🟢 verde | **Salida (%)** | comando aplicado (**+** calor / **−** frío) |

Encima de la curva, tres tarjetas muestran los valores instantáneos
(Medida, Consigna activa, Salida). Se puede hacer zoom/desplazar la curva con el ratón.

---

## 6. Comprender la regulación (calor / frío)

El regulador actúa en **un solo sentido a la vez**, elegido según la desviación
`Consigna − Medida`:

| Situación | Sentido que actúa | Salida | Indicador |
|-----------|---------------|--------|--------|
| Medida **< ** Consigna (hay que calentar) | **Sentido 1 (calor)** | **positiva** (0…+100 %) | **Calor activo = 1** |
| Medida **> ** Consigna (hay que enfriar) | **Sentido 2 (frío)** | **negativa** (−100…0 %) | **Frío activo = 1** |

Consecuencias prácticas:

- Seleccionar **PID/TOR para el frío** no basta para encender «Frío activo»:
  hace falta que **la medida esté por encima de la consigna**. Mientras la medida esté
  por debajo, es el **calor** el que trabaja.
- Para ver el frío entregar: en **Auto**, sentido 2 en PID/TOR, **baje la
  consigna por debajo de la medida actual** (o espere una superación). La salida
  se vuelve negativa y **Frío activo** pasa a 1.
- En **TOR**, el relé conmuta sobre la **media histéresis** a ambos lados de la
  consigna (zona muerta simétrica) y respeta el **ciclo mínimo** entre dos
  conmutaciones. En **PWM**, la salida trocea a 0/100 % pero su media sigue al PID.

---

## 7. Parámetros (botón ⚙)

El botón **⚙ Parámetros** abre una ventana para configurar:

### Transporte Modbus
Elección del bus de comunicación — **uno solo activo a la vez**:

**TCP (Ethernet)**
- **IP de escucha** (`0.0.0.0` = todas las interfaces) y **Puerto** (por defecto 5502);
- **IP autorizadas**: una por línea, comodines `*` aceptados (ej. `192.168.1.*`).
  **Lista vacía = todas las IP autorizadas.** Las demás se rechazan.

**RTU (RS485)** — necesita un binario compilado con la feature `rtu`
- **Puerto serie**: `/dev/ttyUSB0`, `/dev/ttyAMA0` (Raspberry Pi), `COM3` (Windows)…;
- **Baud** (por defecto 19200), **Paridad** (por defecto Par), **Bits de datos** (8),
  **Bits de stop** (1) — a acordar con el maestro;
- **Dirección esclava** (1–247).

> ⚠️ **Un solo maestro remoto a la vez.** En TCP, la conexión de un nuevo
> maestro **desconecta automáticamente** al anterior. La IHM local **no** es
> un maestro: permanece siempre activa. En RTU, dar preferencia a un **enlace
> punto a punto** (el equipo responde sea cual sea la dirección solicitada).

### Función de transferencia (proceso)
Comportamiento físico simulado `G(s) = K·e^(−L·s) / (1 + T·s)`:
- **Ganancia K**: variación de medida por % de salida;
- **Constante T** (s): inercia/rapidez;
- **Retardo L** (s): tiempo muerto antes de la reacción;
- **Ambiente**: valor de reposo.

### Límites de consigna
Límites mín/máx de la consigna auto.

Botones: **Aplicar** (surte efecto de inmediato **y** guarda),
**Restablecer por defecto**, **Cerrar**.

### Guardado de los ajustes
Los ajustes se **guardan** en un archivo `mock_ru_modbustcp.toml` (junto
al software) y se **recargan en el próximo arranque**. El botón **💾 Guardar
ajustes** del encabezado guarda también las ganancias PID, la histéresis, el ciclo
mínimo TOR y el período PWM modificados desde el panel de la izquierda.

---

## 8. Conectar un cliente Modbus

El software es un **esclavo Modbus** (TCP puerto 5502 por defecto, o RTU serie
según el transporte elegido en el § 7). Un cliente (autómata, SCADA, `mbpoll`…) puede
**leer** el estado y **escribir** las consignas/modos. Recordatorio: **un solo maestro
remoto a la vez** (en TCP, un recién llegado desconecta al anterior).

Referencias principales (direcciones **base 0**):

| Dato | Tabla | Dirección | Tipo | Acceso |
|--------|-------|---------|------|-------|
| Marcha/Paro | Bobina | 0 | bit | L/E |
| Auto/Manual | Bobina | 1 | bit | L/E |
| Modo sentido 1 / sentido 2 | Holding | 0 / 1 | 0=Off,1=PID,2=TOR,3=PWM | L/E |
| Consigna auto | Holding | 2–3 | flotante | L/E |
| Consigna manual | Holding | 4–5 | flotante | L/E |
| Ciclo mín. TOR (s) | Holding | 20–21 | flotante | L/E |
| Período PWM (s) | Holding | 22–23 | flotante | L/E |
| Medida (PV) | Input | 0–1 | flotante | L |
| Salida (%) | Input | 2–3 | flotante | L |
| Identificador «CESAM-Lab» | Holding | 42–46 | texto ASCII | L |

> La **tabla completa** (ganancias PID, histéresis, codificación de los flotantes, códigos
> de función, ejemplos `mbpoll`) está en **[table_modbus.md](table_modbus.md)**.
> La misma tabla también es visible **en directo** en el panel de la derecha de la IHM.

---

## 9. Uso sin pantalla («headless» / Docker)

Para un despliegue en segundo plano (Raspberry Pi sin pantalla, servidor), existe una
versión **sin interfaz**: hace funcionar la simulación y el servidor
Modbus, controlables **únicamente por Modbus**.

```bash
# Imagen Docker (desplegable en cualquier lugar):
docker run --rm -p 5502:5502 -v "$PWD/conf:/data" orme:headless
```

La carpeta montada en `/data` permite proporcionar/conservar `mock_ru_modbustcp.toml`.

---

## 10. Preguntas frecuentes

| Pregunta / síntoma | Respuesta |
|---------------------|---------|
| **«Frío activo» no pasa a 1 aunque he puesto PID/TOR.** | Normal: el frío solo entrega si **la medida supera la consigna**. Baje la consigna por debajo de la medida (modo Auto). Ver **§ 6 (Comprender la regulación)**. |
| La medida no se mueve. | Compruebe que el equipo está **En marcha**, y que la consigna/salida no son nulas. |
| En manual, cambiar los modos sentido 1/2 no hace nada. | Normal: los modos solo se aplican en **Auto**. |
| El encabezado muestra **Modbus ✖**. | Puerto ya en uso o < 1024 sin derechos: cambie el **puerto** en ⚙ Parámetros. |
| Mi cliente Modbus es rechazado. | Su IP no está en la **lista blanca**: vacíe la lista o añada un patrón (`192.168.1.*`). |
| Los flotantes leídos son incoherentes. | Problema de **orden de las palabras** en el cliente (palabra de mayor peso primero). Ver table_modbus.md. |
| Una consigna escrita en Modbus se ignora. | Un flotante ocupa **2 registros**: escríbalos **juntos**. |
| Mis ajustes no se conservan. | Haga clic en **Aplicar** / **💾 Guardar**. El archivo `mock_ru_modbustcp.toml` debe ser accesible para escritura. |

---

*Documentación técnica asociada: [conception.md](conception.md) ·
[table_modbus.md](table_modbus.md) · [maintenance.md](maintenance.md).*
