# Manual de usuario — Regulador PROFIBUS DP simulado (ORPD)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · **ES** · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

> Crate: `mock_bin_ru_pbdp` · Ejecutable: **ru_pbdp** · Marca: **ORPD**

---

## ⚠️ Antes de empezar: lo que este simulador NO es

`ru_pbdp` **no es** un esclavo PROFIBUS DP conforme al hardware real.
PROFIBUS DP es un bus de testigo cuyo cumplimiento de las ventanas
temporales (*slot time*, `Tsdr`, vigilante) exige un circuito dedicado
(ASIC SPC3/VPC3, tarjeta maestra Hilscher/Softing/Siemens CP). Un programa
Tokio ordinario, incluso conectado a un puerto RS-485 real, **no puede
cumplir estas restricciones**: un autómata real (un Siemens S7 con tarjeta
maestra, por ejemplo) **nunca** reconocerá este simulador como esclavo
válido en un bus real.

Lo que `ru_pbdp` hace realmente: implementa, **en software y sin
restricciones de tiempo real**, la estructura de tramas y la máquina de
estados de un esclavo DP-V0 (parametrización, configuración, diagnóstico,
intercambio cíclico). Es una herramienta para **comprender el protocolo** y
**probar un desarrollo software** (códec, máquina de estados, herramientas)
— no para pilotar equipos de campo. Véase
[reference_profibus.md](reference_profibus.md) §6 para el detalle de las
limitaciones.

---

## 1. Para qué sirve este simulador

`ru_pbdp` simula un **regulador de proceso** (bucle PID sobre un proceso
térmico, modelo idéntico a ORME/Modbus) y lo expone mediante un conjunto
simulado de tramas PROFIBUS DP-V0, sobre un enlace serie (RS-485/RS-232).
La interfaz gráfica permite **pilotar** la simulación y **visualizar** su
dinámica; el registro de tramas muestra el tráfico intercambiado en
hexadecimal.

---

## 2. Primeros pasos

```bash
cargo run -p mock_bin_ru_pbdp          # IU + enlace serie PROFIBUS DP
```

Al arrancar, el simulador intenta abrir el puerto serie configurado (por
defecto `/dev/ttyUSB0` o `COM3`, 500 kbit/s, dirección de estación 3). Si
el puerto no existe (caso frecuente sin hardware serie), la IU muestra el
error de apertura en la cabecera — la simulación del regulador sigue
funcionando, solo el enlace no está disponible. Ajuste el **puerto serie**
en *Ajustes* para apuntar a un pseudo-terminal o a un adaptador
USB-serie disponible.

---

## 3. La interfaz

### Cabecera

- **Título** y botones **⚙ Ajustes** / **💾 Guardar ajustes**.
- A la derecha: **estado del aparato** (EN MARCHA / DETENIDO), **estado del
  enlace** (`PROFIBUS ● <puerto> [<estado>]` en verde si está abierto — el
  estado mostrado es el de la máquina de estados DP-V0:
  `Power_On`/`Wait_Prm`/`Wait_Cfg`/`Data_Exchange`), y el **logo
  CESAM-Lab**.
- Un **banner naranja permanente** recuerda la no-interoperabilidad con
  hardware real (véase la advertencia anterior).

### Mini-terminal (parte inferior de la ventana)

Registro de solo lectura de las tramas **recibidas** (← RX) y **emitidas**
(→ TX), con marca de tiempo y visualización hexadecimal. Botón **Borrar**
para vaciar el registro.

### Panel de comandos (izquierda)

Idéntico a ORME: **Marcha/Paro**, **Auto/Manual**, modos de regulación
**sentido 1 (calor)** / **sentido 2 (frío)** (Off/PID/Todo-o-nada/PWM),
**consignas** (automática y manual), **ajustes PID** de ambos sentidos,
**histéresis**, **ciclo mínimo todo-o-nada**, **periodo PWM**.

### Panel derecho: bloques de E/S PROFIBUS

Tabla en vivo de los bloques *Output* (maestro→esclavo) e *Input*
(esclavo→maestro), con la disposición de bytes utilizada por este
simulador — véase [reference_profibus.md](reference_profibus.md) §3.

### Zona central

Tarjetas **Medida**, **Consigna activa**, **Salida**, y curva de tendencia.

---

## 4. Ajustes (modal ⚙)

- **Idioma** de la interfaz (8 idiomas), persistido.
- **Comprobar actualizaciones al arrancar** + botón **Comprobar ahora**.
- **Puerto serie**, **velocidad** (baudios — usar un valor normalizado
  PROFIBUS DP: 9600, 19200, 45450, 93750, 187500, 500000, 1500000, 3000000,
  6000000 o 12000000), **dirección de estación** (0-125).
- **Vigilante de protocolo (permitido)**: casilla — si está desmarcada, el
  vigilante solicitado por el maestro mediante `Set_Prm` se **ignora**
  (nunca se arma).
- **Función de transferencia del proceso**: ganancia `K`, constante de
  tiempo `τ`, retardo puro, valor ambiente.
- **Límites de consigna**: mín / máx (reordenados automáticamente si están
  invertidos).
- **Aplicar** / **Restablecer por defecto** / **Cerrar**.

Un cambio de puerto/velocidad/dirección **cierra y reabre** el enlace
serie. Los ajustes se guardan en `mock_ru_pbdp.toml` (directorio actual;
sobrescribible mediante la variable de entorno `MOCK_CONFIG`).

**El formato de trama (8E1) está fijado por la norma PROFIBUS DP** y no es
ajustable aquí, a diferencia de Modbus RTU o NAMUR serie.

---

## 5. El mini-terminal como herramienta pedagógica

Sin hardware PROFIBUS real, la mejor manera de observar el protocolo es
hacer dialogar **dos instancias** de esta herramienta entre sí — o escribir
un pequeño script que reproduzca una secuencia `Slave_Diag` → `Set_Prm` →
`Chk_Cfg` → `Data_Exchange` sobre un pseudo-terminal
(`socat -d -d pty,raw,echo=0 pty,raw,echo=0`) — y leer el mini-terminal para
ver las tramas intercambiadas en hexadecimal, con su decodificación en
[reference_profibus.md](reference_profibus.md).

---

## 6. Preguntas frecuentes

**¿Puedo conectar este simulador a un autómata PROFIBUS DP real?** No —
véase la advertencia al inicio de este documento y el §6 de
[reference_profibus.md](reference_profibus.md).

**El puerto serie no se abre.** El archivo/dispositivo indicado no existe o
los permisos son insuficientes (grupo `dialout` en Linux). El error exacto
se muestra en la cabecera de la IU.

**El enlace permanece en `Wait_Prm`.** El maestro aún no ha enviado un
`Set_Prm` con el identificador esperado (`0xEE01`, identificador
**ficticio**, no registrado ante el PNO). Véase
[reference_profibus.md](reference_profibus.md) §2.

**El enlace permanece en `Wait_Cfg`.** El `Chk_Cfg` recibido no anuncia las
longitudes de E/S esperadas (45 bytes de salida, 17 de entrada para este
simulador).

**El aparato se detiene solo.** El vigilante de protocolo (armado por el
maestro mediante `Set_Prm`) se ha activado por falta de intercambio
cíclico recibido a tiempo — es el estado seguro esperado, no un fallo.

**¿Lanzar sin interfaz gráfica?** Compile en modo *headless*:
`cargo run -p mock_bin_ru_pbdp --no-default-features` — el enlace serie y
la simulación funcionan sin IU.
