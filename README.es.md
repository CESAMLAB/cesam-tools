<p align="center">
  <img src="pic/Logo-CESAM-Couleur-vect-card.png" alt="CESAM-Lab" height="84">
</p>

# cesam-tools — Caja de herramientas CESAM-Lab

*🌍 [English](README.md) · [Français](README.fr.md) · [Deutsch](README.de.md) · **Español** · [Italiano](README.it.md) · [Português](README.pt.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md)*

<p align="center">
  <a href="https://github.com/CESAMLAB/cesam-tools/releases/latest"><img src="https://img.shields.io/github/v/release/CESAMLAB/cesam-tools?label=release" alt="Latest release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue" alt="License: MIT"></a>
</p>

Workspace Rust que agrupa las **herramientas de CESAM-Lab**, empezando por
**simuladores de instrumentos industriales**: equipos virtuales que
reproducen un comportamiento físico realista y se comunican mediante protocolos
de campo. Útil para desarrollar, probar y demostrar supervisores, autómatas
o pasarelas **sin hardware real**.

> Distribuido gratuitamente bajo licencia [MIT](LICENSE).

## Instrumentos disponibles

| Crate | Producto | Descripción | Protocolo | IHM |
|-------|---------|-------------|-----------|-----|
| [`mock_bin_ru_modbus`](mock_bin_ru_modbus) | **ORME** | Regulador (PID / TOR / PWM) sobre función de transferencia | Modbus TCP & RTU (esclavo) | egui |
| [`mock_bin_su_namur`](mock_bin_su_namur) | **OSNE** | Agitador de laboratorio suspendido: función de transferencia del motor, control rápido de velocidad, carga viscosa ajustable | NAMUR sobre TCP & serie RS-232 (esclavo) | egui |
| [`mock_bin_ru_opcua`](mock_bin_ru_opcua) | **ORUE** | Regulador de proceso (PID anti-windup) sobre un proceso de primer orden, con seguridad OPC UA configurable | OPC UA (servidor) | egui |
| [`mock_bin_ru_sparkplugb`](mock_bin_ru_sparkplugb) | **ORSE** | Regulador de proceso expuesto como nodo perimetral MQTT Sparkplug B (saliente) | Sparkplug B / MQTT (cliente) | egui |
| [`mock_bin_ru_s7`](mock_bin_ru_s7) | **ORSS** | Regulador de proceso expuesto como servidor S7comm sobre ISO-on-TCP (RFC1006) | S7comm (servidor) | egui |
| [`mock_bin_ru_ethernetip`](mock_bin_ru_ethernetip) | **OREE** | Regulador de proceso expuesto como adaptador EtherNet/IP (mensajería explícita CIP) | EtherNet/IP (adaptador) | egui |
| [`mock_bin_ru_pbdp`](mock_bin_ru_pbdp) | **ORPD** | Regulador de proceso expuesto como esclavo PROFIBUS DP-V0 simulado sobre enlace serie | PROFIBUS DP (esclavo, serie) | egui |

Bibliotecas compartidas:

| Crate | Descripción |
|-------|-------------|
| [`mock_lib_control`](mock_lib_control) | Bloques de regulación reutilizables: PID anti-windup, todo-o-nada con histéresis, proceso de 1.er orden + retardo puro (FOPDT). |
| [`mock_lib_regulator`](mock_lib_regulator) | Regulador PID listo para usar (estado, configuración TOML, actor `ractor`), compartido tal cual por ORUE, ORSE, ORSS y OREE. |

## ORME — el regulador simulado

<p align="center">
  <img src="pic/orme-logo.svg" alt="ORME — Open Regulator Modbus Emulator" height="120">
</p>

> **ORME** — *Open Regulator Modbus Emulator*. **«Abra el bus.»**
> Un regulador de campo que solo existe en su bus Modbus.

Un regulador industrial virtual completo:

- **Proceso** modelado mediante una función de transferencia de primer orden con
  retardo puro `K·e^(-Ls) / (1 + T·s)` (típico de un horno o baño termostatizado).
- **Regulación** bidireccional: sentido 1 (calor) y sentido 2 (frío),
  cada uno configurable en **PID**, **todo-o-nada (TOR)** o **relé de ciclo (PWM)**.
- **Modos** marcha/paro y automático/manual.
- **Servidor Modbus** en **TCP** o **RTU serie / RS485** (feature `rtu`), a elección.
  Tabla de direcciones (consigna, medida, salida, modos…), **lista blanca de IP**
  (comodines `*`) configurable en caliente, y **política de maestro único** (un solo maestro
  remoto a la vez; en TCP un recién llegado desconecta al anterior).
- **Interfaz gráfica** en una página: control, **curva de tendencia**
  en tiempo real, **tabla de direcciones Modbus en vivo**, y un **modal de Parámetros**
  (transporte TCP/RTU, puerto, IP autorizadas, parámetros serie, función de
  transferencia, límites de consigna).
- **Configuración persistida** en formato TOML (`mock_ru_modbus.toml`),
  recargada al arranque, con botón de restablecimiento a los valores por defecto.

### Arquitectura asíncrona

```
        Command (cast no bloqueante)            instantánea compartida
  IHM (egui) ──────────────────────►  SimulationActor  ──────────►  IHM (lectura)
  Modbus escritura ────────────────►   (ractor)         ──────────►  imagen Modbus
  Modbus lectura  ◄──────────────────────────────────────  imagen Modbus
```

- **`ractor`**: un actor único posee el estado del regulador; todas las
  mutaciones pasan por mensajes (sin bloqueo sobre la lógica de negocio).
- **`tokio-modbus`**: servidor Modbus TCP y RTU serie (trait `Service`).
- **`eframe`/`egui`**: interfaz gráfica en el hilo principal.

## OSNE — el agitador de laboratorio simulado

<p align="center">
  <img src="pic/osne-logo.svg" alt="OSNE — Open Stirrer NAMUR Emulator" height="120">
</p>

> **OSNE** — *Open Stirrer NAMUR Emulator*.
> Un agitador de laboratorio suspendido (estilo IKA) que solo existe en su enlace NAMUR.

Un agitador de laboratorio virtual completo:

- **Motor** modelado mediante una función de transferencia rotacional `J·dω/dt = T
  − k·η·ω − fricción` (Euler explícito), con un **PID rápido** que regula el par
  para seguir la consigna de velocidad.
- **Viscosidad ajustable** `η`: aumenta el par de carga; con viscosidad elevada el
  motor satura y la consigna se vuelve inalcanzable (**sobrecarga**) — como un
  agitador real.
- **Servidor NAMUR** (protocolo de comandos ASCII) sobre **TCP** (prueba sin
  hardware) o **serie RS-232** (feature `serial`), con un **watchdog** por sesión
  (`OUT_WD1@<m>`), política de **maestro único** y una **lista blanca de IP** (TCP).
- **Interfaz gráfica** en una página: consigna de velocidad, viscosidad, **curva de
  tendencia** de velocidad/par en vivo, un **miniterminal NAMUR** embebido
  (enviar/inspeccionar tramas con historial de comandos), y un **modal de
  Parámetros** (transporte TCP/serie, parámetros del motor, límites, i18n en
  8 idiomas).
- **Configuración persistida** en formato TOML (`mock_su_namur.toml`), recargada al
  arranque, con botón de restablecimiento a los valores por defecto.

Comparte la arquitectura de ORME (modelo de negocio síncrono, actores `ractor`,
IHM `egui`). Lánzalo con `cargo run -p mock_bin_su_namur`; el servidor NAMUR escucha
en `0.0.0.0:4001` por defecto.

## ORUE — el regulador OPC UA simulado

<p align="center">
  <img src="pic/ru_opcua-logo.svg" alt="ORUE — Open Regulator UA Emulator" height="120">
</p>

> **ORUE** — *Open Regulator UA Emulator*. **«Unifique el proceso.»**
> Un regulador de proceso que solo existe en su espacio de direcciones OPC UA.

Un regulador de proceso virtual completo:

- **Proceso** modelado mediante una función de transferencia de primer orden
  gobernada por un **PID anti-windup**, con paso cada 0,5 s.
- **Servidor OPC UA** (`async-opcua`, nativo de Tokio, criptografía 100 % Rust —
  sin OpenSSL, pila MPL-2.0). **Seguridad configurable** (`SecurityConfig`):
  `None`/anónimo por defecto (arranque instantáneo) **o** `Basic256Sha256` /
  SignAndEncrypt con un certificado autofirmado (`pki/`, generado en el primer
  arranque cifrado), más tokens anónimo o de **usuario/contraseña**.
- **Una postura que difiere de ORME/OSNE**: la seguridad OPC UA se basa en
  **certificado + autenticación**, no en una lista blanca de IP (no hay **ninguna**);
  el servidor acepta **varias sesiones cliente simultáneas** (sin maestro único, gana
  el último que escribe). El valor por defecto `None`/anónimo en `0.0.0.0:4840` es el
  más abierto del workspace — un banner de la IHM avisa cuando el cifrado está
  desactivado.
- **Interfaz gráfica** en una página: control, **curva de tendencia** en tiempo
  real, y un **modal de Parámetros** (red, función de transferencia del proceso,
  ganancias PID, límites de consigna, seguridad, i18n en 8 idiomas).
- **Configuración persistida** en formato TOML (`mock_ru_opcua.toml`), recargada al
  arranque, con botón de restablecimiento a los valores por defecto.

Comparte la arquitectura de ORME (modelo de negocio síncrono, actores `ractor`,
IHM `egui`). Lánzalo con `cargo run -p mock_bin_ru_opcua`; el servidor OPC UA escucha
en `0.0.0.0:4840` por defecto. El espacio de direcciones está documentado en
[`mock_bin_ru_opcua/docs/es/reference_opcua.md`](mock_bin_ru_opcua/docs/es/reference_opcua.md).

## ORSE — el nodo perimetral Sparkplug B simulado

<p align="center">
  <img src="pic/ru_spb-logo.svg" alt="ORSE — Open Regulator Sparkplug Emulator" height="120">
</p>

> **ORSE** — *Open Regulator Sparkplug Emulator*.
> Un regulador de proceso que solo existe como nodo perimetral MQTT Sparkplug B.

Un regulador de proceso virtual completo, mismo modelo PID + proceso de primer orden que ORME:

- **Nodo perimetral MQTT Sparkplug B** (cliente saliente, `rumqttc` +
  `sparkplug-rs`, protobuf Eclipse Tahu, 100 % Rust — sin `protoc`). Publica
  `NBIRTH`/`NDATA` y un `NDEATH` portado por el **testamento MQTT** (*Last
  Will*, robusto ante cualquier pérdida de enlace); reacciona a las
  escrituras `NCMD` del broker. Contadores `bdSeq`/`seq` poseídos y probados
  en una capa de protocolo pura, no delegados a un framework.
- **Una postura diferente de ORME/OSNE**: al ser un cliente y no un servidor,
  **no hay lista blanca de IP**. **MQTT en texto plano por defecto** (puerto
  1883, sin cifrar, sin autenticación) — un banner de la IHM avisa mientras
  no se activen TLS + credenciales para salir de una red de confianza.
- **Interfaz gráfica** en una página: control, **curva de tendencia** en
  tiempo real, y un **modal de Parámetros** (dirección/credenciales/TLS del
  broker, función de transferencia del proceso, ganancias PID, límites de
  consigna, i18n en 8 idiomas).
- **Configuración persistida** en formato TOML (`mock_ru_sparkplugb.toml`),
  recargada al arranque, con botón de restablecimiento a los valores por
  defecto.

Lánzalo con `cargo run -p mock_bin_ru_sparkplugb`; se conecta de salida al
broker configurado en *Parámetros* (`localhost:1883` por defecto) — ningún
puerto en escucha.

## ORSS — el regulador S7 simulado

<p align="center">
  <img src="pic/ru_s7-logo.svg" alt="ORSS — Open Regulator S7 Server" height="120">
</p>

> **ORSS** — *Open Regulator S7 Server*.
> Un regulador de proceso que solo existe en su enlace S7comm.

Un regulador de proceso virtual completo, mismo modelo PID + proceso de primer orden que ORME:

- **Servidor S7comm hecho a mano** sobre ISO-on-TCP (RFC1006), puerto 102:
  tramas TPKT, COTP (CR→CC, DT) y S7comm (Setup, Read/Write Var) sobre una
  **imagen de bytes DB1**. No existe ningún crate de **servidor** S7 en
  Rust (solo orientados a cliente): el subconjunto requerido se implementa
  por tanto directamente — análisis acotado, sin pánico ante una trama
  malformada.
- **Se aceptan varios clientes simultáneos** (comportamiento de un autómata
  real), a diferencia de la política de maestro único con desalojo de
  ORME — gana el último que escribe.
- **Sin autenticación ni cifrado** (S7 «clásico»): solo la **lista blanca de
  IP** y la topología de red protegen el acceso; un banner de la IHM avisa
  en caso de exposición (`0.0.0.0` + lista blanca vacía).
- **Interfaz gráfica** en una página: control, **curva de tendencia** en
  tiempo real, y un **modal de Parámetros** (red, lista blanca, función de
  transferencia del proceso, ganancias PID, límites de consigna, i18n en 8
  idiomas).
- **Configuración persistida** en formato TOML (`mock_ru_s7.toml`), recargada
  al arranque, con botón de restablecimiento a los valores por defecto.

Lánzalo con `cargo run -p mock_bin_ru_s7`; el servidor S7comm escucha por
defecto en `0.0.0.0:102` (puerto < 1024 requiere privilegios root).

## OREE — el regulador EtherNet/IP simulado

<p align="center">
  <img src="pic/ru_eip-logo.svg" alt="OREE — Open Regulator EtherNet/IP Emulator" height="120">
</p>

> **OREE** — *Open Regulator EtherNet/IP Emulator*.
> Un regulador de proceso que solo existe en su enlace EtherNet/IP.

Un regulador de proceso virtual completo, mismo modelo PID + proceso de primer orden que ORME:

- **Adaptador EtherNet/IP hecho a mano** (encapsulación `RegisterSession`,
  `SendRRData`/CPF, y CIP `Read Tag`/`Write Tag` por segmento simbólico,
  **little-endian**), puerto 44818. No existe ningún crate de **adaptador**
  EtherNet/IP en Rust (solo orientados a cliente/escáner): el subconjunto
  requerido se implementa por tanto directamente — análisis acotado, sin
  pánico ante un paquete malformado.
- **Se aceptan varios clientes simultáneos** (comportamiento de un
  adaptador), a diferencia de la política de maestro único con desalojo de
  ORME — cada sesión recibe un *session handle*, gana el último que escribe.
- **Sin autenticación ni cifrado** (EtherNet/IP «clásico»): solo la **lista
  blanca de IP** y la topología de red protegen el acceso; un banner de la
  IHM avisa en caso de exposición.
- **Interfaz gráfica** en una página: control, **curva de tendencia** en
  tiempo real, y un **modal de Parámetros** (red, lista blanca, función de
  transferencia del proceso, ganancias PID, límites de consigna, i18n en 8
  idiomas).
- **Configuración persistida** en formato TOML (`mock_ru_ethernetip.toml`),
  recargada al arranque, con botón de restablecimiento a los valores por
  defecto.

Lánzalo con `cargo run -p mock_bin_ru_ethernetip`; el adaptador EtherNet/IP
escucha por defecto en `0.0.0.0:44818`.

## ORPD — el regulador PROFIBUS DP simulado

<p align="center">
  <img src="pic/ru_pbdp-logo.svg" alt="ORPD — Open Regulator Profibus DP" height="120">
</p>

> **ORPD** — *Open Regulator Profibus DP*.
> Un regulador de proceso que solo existe en su enlace PROFIBUS DP.

Un regulador de proceso virtual completo, mismo modelo PID + proceso de primer orden que ORME:

- **Simulador software de tramas PROFIBUS DP-V0** sobre enlace serie
  (RS-485/RS-232): códec de tramas (`SD1`/`SD2`/`SD3`/`SD4`/`SC`, FCS) y
  máquina de estados del esclavo
  (`Power_On → Wait_Prm → Wait_Cfg → Data_Exchange`). ⚠️ **No interoperable
  con hardware PROFIBUS DP real**: el temporizado de bus real (*slot time*,
  `Tsdr`) exige un ASIC dedicado que este simulador puramente software no
  pretende emular — véase
  [`reference_profibus.md`](mock_bin_ru_pbdp/docs/es/reference_profibus.md) §6.
- **El enlace serie es el único transporte** (sin equivalente TCP para
  PROFIBUS DP, a diferencia de ORME/OSNE donde el enlace serie es una
  función opcional junto a un transporte TCP siempre presente):
  `tokio-serial` es una dependencia directa, no opcional. Sin lista blanca
  de IP (intrínsecamente punto a punto).
- **Vigilante de protocolo** — una parte real de DP-V0 (armada por el
  maestro mediante `Set_Prm`), no un añadido casero; fuerza el estado seguro
  al vencer.
- **Interfaz gráfica** en una página: control, **curva de tendencia** en
  tiempo real, un **mini-terminal de tramas** (registro hexadecimal del
  tráfico RX/TX), y un **modal de Parámetros** (puerto serie, velocidad,
  dirección de estación, función de transferencia del proceso, ganancias
  PID, límites de consigna, i18n en 8 idiomas).
- **Configuración persistida** en formato TOML (`mock_ru_pbdp.toml`),
  recargada al arranque, con botón de restablecimiento a los valores por
  defecto.

Lánzalo con `cargo run -p mock_bin_ru_pbdp`; intenta abrir el puerto serie
configurado (por defecto `/dev/ttyUSB0` o `COM3`, 500 kbit/s, dirección de
estación 3).

## Descarga

Hay binarios precompilados disponibles en la página de [**Releases**](https://github.com/CESAMLAB/cesam-tools/releases/latest) — **sin necesidad de la cadena de herramientas Rust**. Cada instrumento incluye su propio ejecutable (`orme`, `osne`, `ru_opcua`, `ru_spb`, `ru_s7`, `ru_eip`, `ru_pbdp`).

**ORME** (regulador Modbus):

| Plataforma | IHM | Headless (solo TCP, sin IHM) |
|------------|-----|------------------------------|
| Linux x86_64 | [`orme-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-linux-x86_64) | [`orme-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-linux-x86_64-headless) |
| Windows x86_64 | [`orme-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64 bits) | [`orme-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-rpi-arm64) | [`orme-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-rpi-arm64-headless) |

**OSNE** (agitador de laboratorio NAMUR):

| Plataforma | IHM | Headless (solo TCP, sin IHM) |
|------------|-----|------------------------------|
| Linux x86_64 | [`osne-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-linux-x86_64) | [`osne-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-linux-x86_64-headless) |
| Windows x86_64 | [`osne-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64 bits) | [`osne-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-rpi-arm64) | [`osne-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-rpi-arm64-headless) |

**ORUE** (regulador OPC UA):

| Plataforma | IHM | Headless (solo TCP, sin IHM) |
|------------|-----|------------------------------|
| Linux x86_64 | [`ru_opcua-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-linux-x86_64) | [`ru_opcua-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-linux-x86_64-headless) |
| Windows x86_64 | [`ru_opcua-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64 bits) | [`ru_opcua-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-rpi-arm64) | [`ru_opcua-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-rpi-arm64-headless) |

**ORSE** (nodo perimetral Sparkplug B):

| Plataforma | IHM | Headless (solo cliente, sin IHM) |
|------------|-----|------------------------------|
| Linux x86_64 | [`ru_spb-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_spb-linux-x86_64) | [`ru_spb-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_spb-linux-x86_64-headless) |
| Windows x86_64 | [`ru_spb-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_spb-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64 bits) | [`ru_spb-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_spb-rpi-arm64) | [`ru_spb-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_spb-rpi-arm64-headless) |

**ORSS** (regulador S7comm):

| Plataforma | IHM | Headless (solo TCP, sin IHM) |
|------------|-----|------------------------------|
| Linux x86_64 | [`ru_s7-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_s7-linux-x86_64) | [`ru_s7-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_s7-linux-x86_64-headless) |
| Windows x86_64 | [`ru_s7-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_s7-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64 bits) | [`ru_s7-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_s7-rpi-arm64) | [`ru_s7-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_s7-rpi-arm64-headless) |

**OREE** (adaptador EtherNet/IP):

| Plataforma | IHM | Headless (solo TCP, sin IHM) |
|------------|-----|------------------------------|
| Linux x86_64 | [`ru_eip-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_eip-linux-x86_64) | [`ru_eip-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_eip-linux-x86_64-headless) |
| Windows x86_64 | [`ru_eip-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_eip-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64 bits) | [`ru_eip-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_eip-rpi-arm64) | [`ru_eip-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_eip-rpi-arm64-headless) |

**ORPD** (regulador PROFIBUS DP):

| Plataforma | IHM | Headless (enlace serie, sin IHM) |
|------------|-----|------------------------------|
| Linux x86_64 | [`ru_pbdp-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_pbdp-linux-x86_64) | [`ru_pbdp-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_pbdp-linux-x86_64-headless) |
| Windows x86_64 | [`ru_pbdp-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_pbdp-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64 bits) | [`ru_pbdp-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_pbdp-rpi-arm64) | [`ru_pbdp-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_pbdp-rpi-arm64-headless) |

```bash
chmod +x orme-linux-x86_64        # Linux / Raspberry Pi (igual para el resto de instrumentos)
./orme-linux-x86_64
```

Los binarios Linux/RPi están enlazados dinámicamente con glibc y necesitan un entorno de escritorio (X11/Wayland) para la IHM. En **Wayland**, instala la entrada de escritorio para el icono en la barra de tareas: `scripts/install-desktop.sh`. Verifica la integridad con las sumas de comprobación publicadas:

```bash
sha256sum -c SHA256SUMS
```

## Arranque rápido

```bash
# Requisitos previos: Rust stable (edición 2021, >= 1.85).
# Dependencias del sistema Linux para la IHM: libxkbcommon, libwayland/xcb, openGL.

cargo run -p mock_bin_ru_modbus
```

La ventana se abre y el servidor Modbus TCP escucha en `0.0.0.0:5502`.
El **puerto**, la **IP de escucha** y la **lista blanca de IP** se ajustan en el
modal **⚙ Parámetros** (aplicado en caliente) y luego se **persisten** en
`mock_ru_modbus.toml`. El **idioma de la interfaz** (francés, inglés,
alemán, español, italiano, portugués, neerlandés, polaco) se elige en ese
mismo modal y se persiste. Para usar otro archivo de configuración:

```bash
MOCK_CONFIG=/ruta/a/ma_config.toml cargo run -p mock_bin_ru_modbus
```

### Probar el enlace Modbus

Con cualquier cliente Modbus (ej. `mbpoll`):

```bash
# Poner en marcha (bobina 0) y luego leer la medida (input registers 0-1, f32)
mbpoll -m tcp -a 1 -t 0 -p 5502 127.0.0.1 1      # escribir la bobina On/Off
mbpoll -m tcp -a 1 -t 3:float -r 1 -p 5502 127.0.0.1   # leer PV (f32)
```

La tabla de direcciones completa está documentada en
[`mock_bin_ru_modbus/src/map.rs`](mock_bin_ru_modbus/src/map.rs).

## Desarrollo

```bash
cargo test --workspace      # tests unitarios + integración
cargo clippy --workspace    # lint
```

## Documentación

Cada instrumento tiene su propia documentación en su subcarpeta `docs/`,
disponible en ocho idiomas (`docs/<idioma>/`). Versiones españolas:

**ORME** (regulador Modbus):

- [**Manual de usuario**](mock_bin_ru_modbus/docs/es/manuel_utilisateur.md) — primeros pasos, IHM, parámetros, FAQ.
- [Documento de diseño](mock_bin_ru_modbus/docs/es/conception.md) — arquitectura y decisiones técnicas.
- [Tabla de direcciones Modbus](mock_bin_ru_modbus/docs/es/table_modbus.md) — plan de direccionamiento completo.
- [Mantenimiento del software](mock_bin_ru_modbus/docs/es/maintenance.md) — build, configuración, ampliación, resolución de problemas.

**OSNE** (agitador de laboratorio NAMUR):

- [**Manual de usuario**](mock_bin_su_namur/docs/es/manuel_utilisateur.md) — primeros pasos, IHM, miniterminal NAMUR, parámetros, FAQ.
- [Documento de diseño](mock_bin_su_namur/docs/es/conception.md) — modelo del motor, bucle de control, arquitectura.
- [Conjunto de comandos NAMUR](mock_bin_su_namur/docs/es/commandes_namur.md) — referencia del protocolo (canales, comandos, ejemplos).
- [Mantenimiento del software](mock_bin_su_namur/docs/es/maintenance.md) — build, configuración, ampliación, resolución de problemas.

**ORUE** (regulador OPC UA):

- [**Manual de usuario**](mock_bin_ru_opcua/docs/es/manuel_utilisateur.md) — primeros pasos, IHM, conexión de un cliente OPC UA, FAQ.
- [Documento de diseño](mock_bin_ru_opcua/docs/es/conception.md) — modelo PID + proceso, arquitectura de actores, pila `async-opcua`, seguridad.
- [Referencia OPC UA](mock_bin_ru_opcua/docs/es/reference_opcua.md) — endpoint, namespace, nodos (lecturas/escrituras, ejemplos).
- [Mantenimiento del software](mock_bin_ru_opcua/docs/es/maintenance.md) — build, configuración, ampliación, resolución de problemas.

**ORSE** (nodo perimetral Sparkplug B):

- [**Manual de usuario**](mock_bin_ru_sparkplugb/docs/es/manuel_utilisateur.md) — primeros pasos, IHM, conexión al broker, FAQ.
- [Documento de diseño](mock_bin_ru_sparkplugb/docs/es/conception.md) — arquitectura de actores, capa de protocolo, elección de bibliotecas.
- [Referencia Sparkplug B](mock_bin_ru_sparkplugb/docs/es/reference_sparkplugb.md) — topics, métricas, NBIRTH/NDATA/NDEATH, mapeo de NCMD.
- [Mantenimiento del software](mock_bin_ru_sparkplugb/docs/es/maintenance.md) — build, configuración, ampliación, resolución de problemas.

**ORSS** (regulador S7comm):

- [**Manual de usuario**](mock_bin_ru_s7/docs/es/manuel_utilisateur.md) — primeros pasos, IHM, conexión de un cliente S7, FAQ.
- [Documento de diseño](mock_bin_ru_s7/docs/es/conception.md) — arquitectura de actores, capa de protocolo, política de sesión.
- [Referencia S7comm](mock_bin_ru_s7/docs/es/reference_s7.md) — tramas TPKT/COTP/S7comm, imagen DB1, ejemplos.
- [Mantenimiento del software](mock_bin_ru_s7/docs/es/maintenance.md) — build, configuración, ampliación, resolución de problemas.

**OREE** (adaptador EtherNet/IP):

- [**Manual de usuario**](mock_bin_ru_ethernetip/docs/es/manuel_utilisateur.md) — primeros pasos, IHM, conexión de un cliente CIP, FAQ.
- [Documento de diseño](mock_bin_ru_ethernetip/docs/es/conception.md) — arquitectura de actores, capa de protocolo, política de sesión.
- [Referencia EtherNet/IP](mock_bin_ru_ethernetip/docs/es/reference_ethernetip.md) — encapsulación, CIP Read/Write Tag, ejemplos.
- [Mantenimiento del software](mock_bin_ru_ethernetip/docs/es/maintenance.md) — build, configuración, ampliación, resolución de problemas.

**ORPD** (regulador PROFIBUS DP):

- [**Manual de usuario**](mock_bin_ru_pbdp/docs/es/manuel_utilisateur.md) — primeros pasos, IHM, aviso de no-interoperabilidad, FAQ.
- [Documento de diseño](mock_bin_ru_pbdp/docs/es/conception.md) — arquitectura de actores, capa de protocolo, decisiones de códec.
- [Referencia PROFIBUS DP-V0](mock_bin_ru_pbdp/docs/es/reference_profibus.md) — tramas, secuenciación, bloques de E/S, vigilante, ejemplo de secuencia.
- [Mantenimiento del software](mock_bin_ru_pbdp/docs/es/maintenance.md) — build, configuración, ampliación, resolución de problemas.

## Marca y logotipos

Los logotipos están en [`pic/`](pic/):

- [`orme-icon.svg`](pic/orme-icon.svg) / `orme-icon.png` — icono ORME (esfera),
  también embebido como icono de ventana de la aplicación.
- [`orme-logo.svg`](pic/orme-logo.svg) — logotipo ORME completo (icono + texto).
- [`osne-icon.svg`](pic/osne-icon.svg) / `osne-icon.png` — icono OSNE (impulsor de
  agitador), también embebido como icono de ventana de OSNE.
- [`osne-logo.svg`](pic/osne-logo.svg) — logotipo OSNE completo (icono + texto).
- [`ru_opcua-icon.svg`](pic/ru_opcua-icon.svg) / `ru_opcua-icon.png` — icono ORUE
  (esfera de regulador rodeada por un anillo de nodo OPC UA), también embebido como
  icono de ventana de ORUE.
- [`ru_opcua-logo.svg`](pic/ru_opcua-logo.svg) — logotipo ORUE completo (icono + texto).
- [`ru_spb-icon.svg`](pic/ru_spb-icon.svg) / `ru_spb-icon.png` — icono ORSE
  (esfera de regulador + rayo Sparkplug con nodos pub/sub sin conectar), también
  embebido como icono de ventana de ORSE.
- [`ru_spb-logo.svg`](pic/ru_spb-logo.svg) — logotipo ORSE completo (icono + texto).
- [`ru_s7-icon.svg`](pic/ru_s7-icon.svg) / `ru_s7-icon.png` — icono ORSS (esfera de
  regulador + rack abierto de módulos cuadrados, backplane S7), también embebido
  como icono de ventana de ORSS.
- [`ru_s7-logo.svg`](pic/ru_s7-logo.svg) — logotipo ORSS completo (icono + texto).
- [`ru_eip-icon.svg`](pic/ru_eip-icon.svg) / `ru_eip-icon.png` — icono OREE (esfera
  de regulador + anillo cerrado de rombos, DLR EtherNet/IP), también embebido como
  icono de ventana de OREE.
- [`ru_eip-logo.svg`](pic/ru_eip-logo.svg) — logotipo OREE completo (icono + texto).
- [`ru_pbdp-icon.svg`](pic/ru_pbdp-icon.svg) / `ru_pbdp-icon.png` — icono ORPD
  (esfera de regulador con motivo PROFIBUS DP), también embebido como icono de
  ventana de ORPD.
- [`ru_pbdp-logo.svg`](pic/ru_pbdp-logo.svg) — logotipo ORPD completo (icono + texto).
- [`Logo-CESAM-Couleur-vect.png`](pic/Logo-CESAM-Couleur-vect.png) — logotipo CESAM-Lab.

Cada icono se **genera** a partir de su script `*-logo.gen.py`
([`pic/orme-logo.gen.py`](pic/orme-logo.gen.py),
[`pic/osne-logo.gen.py`](pic/osne-logo.gen.py),
[`pic/ru_opcua-logo.gen.py`](pic/ru_opcua-logo.gen.py),
[`pic/ru_spb-logo.gen.py`](pic/ru_spb-logo.gen.py),
[`pic/ru_s7-logo.gen.py`](pic/ru_s7-logo.gen.py),
[`pic/ru_eip-logo.gen.py`](pic/ru_eip-logo.gen.py),
[`pic/ru_pbdp-logo.gen.py`](pic/ru_pbdp-logo.gen.py)). Todos los scripts salvo
el de ORME también rasterizan su `-icon.png` directamente (vía Pillow); el
`.svg` de ORME se rasteriza después.

En **Wayland**, instala el icono de la barra de tareas de un instrumento con
`scripts/install-desktop.sh [orme|osne|ru_opcua|ru_spb|ru_s7|ru_eip|ru_pbdp]`.

## Licencia

[MIT](LICENSE) © 2026 CESAM-Lab

Los componentes de terceros integrados en algunos instrumentos se distribuyen bajo sus propias licencias (en particular, la pila OPC UA bajo MPL-2.0 utilizada por `mock_bin_ru_opcua`); consulte [NOTICE](NOTICE). No modifican la licencia MIT del código de cesam-tools.
