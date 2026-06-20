# Documento de diseño — Regulador simulado Modbus TCP

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · **ES** · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

> Producto: **ORME** · Crate: `mock_bin_ru_modbustcp` · Workspace: `cesam-tools` · Licencia: MIT

Este documento describe la arquitectura, las decisiones técnicas y los principios de
funcionamiento del regulador industrial simulado. Está dirigido a los desarrolladores
que mantienen o amplían el proyecto.

---

## 1. Objetivo y alcance

Proporcionar un **instrumento industrial virtual**: un regulador de proceso que se
comporta de forma realista y se comunica por **Modbus TCP** (esclavo), con el fin de
desarrollar y probar supervisores / autómatas / pasarelas **sin hardware**.

El simulador cubre:

- un **proceso físico** modelado mediante una función de transferencia;
- una **regulación** bidireccional (calor / frío): PID, todo-o-nada (TOR) o
  relé de ciclo (PWM);
- una **interfaz Modbus TCP** que expone el estado completo;
- una **IHM** de control, visualización y configuración;
- la **persistencia** de los parámetros.

Fuera del alcance actual: Modbus RTU, redundancia, historización a largo plazo,
autenticación fuerte (solo se proporciona una lista blanca de IP).

---

## 2. Visión general

```
┌──────────────────────────────────────────────────────────────────────┐
│                       Proceso (hilo principal)                         │
│                                                                        │
│   ┌─────────────────────────┐         lee (Mutex)                      │
│   │   IHM  egui / eframe     │◄──────────────── SharedSnapshot         │
│   │   (gui.rs)               │◄──────────────── SharedStatus           │
│   └───────────┬─────────────┘                                          │
│               │ cast (no bloqueante)                                   │
└───────────────┼────────────────────────────────────────────────────────┘
                │
   ┌────────────┼──────────── Runtime Tokio (hilos de fondo) ───────────┐
   │            ▼                                                         │
   │   ┌──────────────────┐  refresh  ┌──────────────┐                   │
   │   │ SimulationActor   ├──────────►│ SharedSnapshot│ (IHM)            │
   │   │  (ractor)         ├──────────►│ SharedMap     │ (Modbus)         │
   │   │  posee el          │           └──────┬───────┘                  │
   │   │  Regulator         │◄── Command ──┐    │ lee                     │
   │   └──────────────────┘              │    ▼                          │
   │          ▲ Command (cast)            │  ┌──────────────────────┐     │
   │          │                           └──┤ RegulatorService      │     │
   │   ┌──────┴───────────┐  gestiona/rebind │ (trait Service)       │     │
   │   │ ModbusServerActor ├─────────────────►  servidor Modbus TCP  │◄──── clientes
   │   │  (ractor)         │  filtro IP ──────► (tokio-modbus)        │     │
   │   └──────────────────┘   (SharedAllowlist)└──────────────────────┘     │
   └────────────────────────────────────────────────────────────────────┘
```

Principio rector: **un único propietario del estado de negocio**. El `Regulator`
nunca se comparte; vive dentro de `SimulationActor`. Todas las escrituras
(IHM o Modbus) son **mensajes** `Command`. Las lecturas se realizan sobre
**copias** refrescadas en cada paso (`SharedSnapshot`, `SharedMap`), lo que elimina
los bloqueos sobre la lógica y las condiciones de carrera.

---

## 3. Decisiones técnicas

| Necesidad | Elección | Justificación |
|--------|-------|---------------|
| Concurrencia | **`ractor`** (actores) sobre **Tokio** | Aísla el estado mutable en un actor; mutaciones serializadas por mensajes, sin bloqueo aplicativo. Preferencia del proyecto. |
| Modbus TCP esclavo | **`tokio-modbus`** (`tcp-server`) | Implementación async madura; el trait `Service` mapea limpiamente petición→respuesta. |
| IHM | **`egui` / `eframe`** + `egui_plot` | Modo inmediato, multiplataforma, sin estado de UI complejo que sincronizar. |
| Proceso | **FOPDT** (1.er orden + retardo) | Modelo estándar y suficiente para un proceso térmico; pocos parámetros, intuitivo. |
| Persistencia | **`serde` + `toml`** | Formato legible/editable a mano, ideal para parámetros de equipo. |

### Por qué separar lógica síncrona y asíncrona

`mock_lib_control` y `regulator.rs` son **puramente síncronos** (sin IO,
sin async). Ventajas: comprobables unitariamente de forma determinista,
reutilizables por otros instrumentos y fáciles de revisar. La parte asíncrona
queda confinada a los **actores** y a la **capa de red**.

---

## 4. Modelo de datos

### Estado de negocio (`regulator.rs`)

- `Regulator` — agregado propietario: modos, consignas, reguladores (`Pid`,
  `OnOff`) y proceso (`FirstOrderProcess`). No `Clone`, no compartido.
- `RegulatorConfig` — configuración estática (proceso, ganancias, límites, `dt`).
  **Fuente única** de los valores por defecto (la configuración TOML deriva de ella).
- `RegulatorSnapshot` — **copia inmutable** (`Copy`) del estado observable, publicada
  en cada paso. Es el contrato de lectura para la IHM y la tabla Modbus.
- `Command` — enumeración de las mutaciones posibles (marcha, modo, consignas,
  ajustes, proceso, límites).

### Estructuras compartidas (`actors/mod.rs`, `config.rs`)

| Tipo | Contenido | Escrito por | Leído por |
|------|---------|-----------|--------|
| `SharedSnapshot` | `RegulatorSnapshot` tipado | SimulationActor | IHM |
| `SharedMap` | `MemoryMap` (imágenes de las 4 tablas Modbus) | SimulationActor | RegulatorService |
| `SharedAllowlist` | `IpFilter` | ModbusServerActor | aceptación de conexiones |
| `SharedStatus` | `ServerStatus` (escucha / error) | ModbusServerActor | IHM |

Todos son `Arc<Mutex<…>>`: secciones críticas **cortas** (copia / refresh),
nunca retenidas durante un cálculo o una IO.

---

## 5. Componentes

### 5.1 `mock_lib_control` (biblioteca)

- `Pid` — PID en tiempo discreto, derivada sobre el error, **anti-windup** mediante
  acotación del término integral. API: `step(sp, pv, dt)` o `step_with_error(err, dt)`
  (reutilizado para el sentido frío).
- `OnOff` — todo-o-nada con **histéresis simétrica** (zona muerta) **y
  anti-ciclo-corto**: un tiempo de ciclo mínimo (`min_cycle`, s) prohíbe toda
  conmutación mientras el relé no haya permanecido suficiente tiempo en su estado,
  modelando la protección de un actuador real. El relé **enclava** su estado:
  es quien llama quien debe pasarle el error con signo sin reinicializarlo al
  cambio de signo (cf. § 5.2).
- `Pwm` — modulador de ancho de impulso (**relé de ciclo** /
  *time-proportioning*): sobre un período fijo `T_c`, la salida todo-o-nada está
  activa la fracción `duty` del ciclo (`duty` **muestreado una vez por ciclo**
  para evitar un sesgo en régimen permanente). Permite regular finamente un órgano TOR.
- `FirstOrderProcess` — función de transferencia `K·e^(-L·s)/(1+T·s)`, integración
  de Euler + línea de retardo. `reconfigure(...)` cambia los parámetros sin salto.
- `ControllerKind` — `Off` / `Pid` / `OnOff` / `Pwm`, con codificación Modbus
  (`to_code`/`from_code`).

### 5.2 `regulator.rs`

Orquestación de la regulación en cada paso (`step`):

1. si está **detenido** → salida 0, reguladores reinicializados;
2. si está en **manual** → salida = consigna manual (% con signo);
3. si está en **auto** → se calcula **por separado** la contribución del sentido calor (sentido 1,
   error `SP − PV`) y la del sentido frío (sentido 2, error `PV − SP`), cada una ≥ 0,
   luego `salida = calor − frío`:
   - **PID**: salida acotada a `[0, 100]` (`out_min = 0`) — el sentido inactivo (error
     negativo) entrega 0 y su integral se **purga naturalmente** por acotación. No
     se pone a cero por la fuerza: con la fuerte ondulación del PWM, borrarla
     en cada superación de consigna introduciría un error estático;
   - **TOR**: el relé se evalúa sobre el error con signo y conserva su estado al
     atravesar la consigna, lo que restaura una banda de histéresis **simétrica**
     `[SP − h/2, SP + h/2]` (las bandas calor/frío permanecen disjuntas, por lo que los
     dos relés son mutuamente excluyentes);
   - **PWM**: un PID calcula el ciclo de trabajo, modulado por el relé de ciclo;
     la salida física es estrictamente 0 % o 100 %, pero su media sigue al PID.
4. la salida acciona el proceso que produce la nueva medida (PV).

> **Histórico**: antes de esta revisión, el reparto calor/frío se hacía por
> el signo del error y **reinicializaba** el relé TOR al atravesar la
> consigna — lo que truncaba la histéresis a `[SP − h/2, SP]` (mitad de banda,
> asimétrica) y hacía mediocre la regulación TOR. El cálculo por sentido separado
> corrige este defecto.

### 5.3 `actors/simulation.rs`

`SimulationActor` (ractor). `pre_start` arma un `send_interval(dt)` que emite
`Tick`. `handle` procesa `Tick` (avanza la simulación) y `Command` (aplica una
mutación), luego **publica** el estado en `SharedSnapshot` y `SharedMap`.

### 5.4 `actors/network.rs`

`ModbusServerActor` posee el servidor Modbus. `Reconfigure(NetworkConfig)`:
- actualiza la **lista blanca** compartida (efecto inmediato, sin reinicio);
- si el **transporte** (TCP/RTU), el **puerto / IP** o los **parámetros serie**
  cambian, **detiene** la tarea servidor y la **reinicia** (`start_tcp` o
  `start_rtu`); publica el estado en `SharedStatus` (éxito o error).

Un **único transporte** está activo a la vez (`Transport::Tcp` o `Rtu`). El RTU está
detrás de la **feature `rtu`**; sin ella, seleccionar RTU publica un error de
estado explícito.

### 5.5 `modbus_server.rs`

`RegulatorService` implementa `tokio_modbus::server::Service` de manera
**síncrona** (`future::Ready`): lecturas = recorte de `SharedMap`; escrituras =
decodificación en `Command` (mediante `map.rs`) y luego `cast` hacia `SimulationActor`.

**Política de maestro único.** `serve` (TCP) solo autoriza **un maestro remoto a la
vez**: en cada nueva conexión (IP autorizada por la lista blanca), la
anterior se cierra. Mecanismo: el `TcpStream` se envuelve en un
`CancellableStream` que, al recibir una señal `oneshot`, devuelve **EOF en
lectura** — el bucle de procesamiento de `tokio-modbus` termina entonces y cierra el
socket. `serve_rtu` (feature `rtu`) sirve el bus serie mediante
`rtu::Server::serve_forever`: el bus RS485 *es* el maestro único (nada que expulsar).

> ⚠️ La IHM no toma este camino: envía sus `Command` directamente al
> actor, por lo que nunca se cuenta como un maestro.
>
> ⚠️ El servidor RTU de `tokio-modbus` 0.17 no transmite la dirección esclava al
> servicio: el equipo responde por tanto sea cual sea la dirección solicitada. Se
> recomienda un enlace **punto a punto**. `slave_id` se persiste y se muestra, pero no
> se usa para filtrar (limitación de origen).

### 5.6 `map.rs`

**Fuente de verdad** del plan de direccionamiento Modbus. Constantes de direcciones,
`MemoryMap` (imágenes de las tablas), `refresh_from(snapshot)` (estado→registros) y
`*_to_command(s)` (escrituras→comandos). Codificación de los `f32` en 2 registros,
big-endian, palabra de mayor peso al inicio.

### 5.7 `config.rs`

`AppConfig` (red / proceso / regulación) ⇄ TOML. `IpFilter` (comodines `*` por
octeto IPv4). `ServerStatus`. `to_regulator_config()` hace de puente hacia el dominio.

### 5.8 `gui.rs`

IHM de **página única**: encabezado (estados + botones), panel de comandos (izquierda),
supervisión + curva (centro), tabla Modbus en vivo (derecha), modal de Parámetros.
Lee los `Shared*`, envía `Command` mediante `cast` no bloqueante.

---

## 6. Escenarios (secuencias)

**Lectura Modbus (PV)**: cliente → `RegulatorService::call(ReadInputRegisters)` →
lectura de `SharedMap` → `Response`. Ninguna interacción con el actor (latencia mínima).

**Escritura Modbus (consigna)**: cliente → `call(WriteMultipleRegisters)` →
`map::holdings_to_commands` → `cast(Command::SetSpAuto)` → el actor lo aplica en el
paso siguiente → republica `SharedMap`/`SharedSnapshot`.

**Comando IHM**: interacción → `cast(Command)` → ídem.

**Reconfiguración de red**: modal *Aplicar* → `cast(Reconfigure)` →
ModbusServerActor reenlaza si es necesario → `SharedStatus` actualizado → el encabezado
de la IHM refleja el estado.

**Tick**: temporizador → `Tick` → `Regulator::step` → publicación.

---

## 7. Teoría de regulación

**Proceso (FOPDT)**: `v[k+1] = v[k] + (dt/T)·(cible − v[k])`, con
`cible = ambiant + K·u` y `u` retardada `L` segundos (línea de retardo).

**PID**: `u = Kp·e + Ki·∫e + Kd·de/dt`, integral acotada a `[out_min, out_max]`
(anti-windup). Derivada sobre el error (compromiso simplicidad/simetría calor-frío).

**TOR**: activo si `e > +H/2`, inactivo si `e < −H/2`, en caso contrario estado conservado.

**Bidireccional**: un solo sentido actúa a la vez, seleccionado por el signo del
error; la salida global tiene signo (+ calor / − frío).

---

## 8. Decisiones y compromisos

- **Doble publicación (`Snapshot` + `Map`)** en lugar de una única estructura:
  la IHM manipula tipos de negocio, el Modbus registros en bruto; ambos
  permanecen simples y desacoplados, a costa de un ligero sobrecoste de copia despreciable.
- **Lecturas Modbus sin pasar por el actor**: se lee `SharedMap` directamente
  para minimizar la latencia; el actor sigue siendo el único **escritor**, por lo que no hay carrera.
- **Servicio Modbus síncrono** (`future::Ready`): todo el trabajo es no bloqueante
  (lock corto + cast), no hace falta boxear un futuro.
- **Reenlace al cambiar de puerto**: un socket no cambia de puerto; se
  acepta una breve interrupción de servicio en la reconfiguración.
- **Derivada sobre el error** (y no sobre la medida): ligero «latigazo» al
  cambio de consigna, aceptado para mantener el algoritmo simétrico y simple.

---

## 9. Evoluciones posibles

- Modbus RTU / serie (reutilizar `RegulatorService`, cambiar el transporte).
- Rampa de consigna, auto-tuning PID, fallos simulados (sensor averiado, saturación).
- Historización / exportación CSV de la tendencia.
- Cambio de la IHM a **pestañas** si la página única se vuelve demasiado densa.
- Nuevos instrumentos: crear `mock_bin_<nom>` y factorizar lo común en
  `mock_lib_*` (ver [maintenance.md](maintenance.md)).
