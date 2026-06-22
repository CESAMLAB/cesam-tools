# Documentación de mantenimiento — ORME (workspace `cesam-tools`)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · **ES** · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

> Público: desarrolladores que mantienen, corrigen o amplían el proyecto.
> Ver también: [conception.md](conception.md) · [table_modbus.md](table_modbus.md).

---

## 1. Requisitos previos

- **Rust stable** (edición 2021, `rust-version` ≥ 1.85). Instalación: <https://rustup.rs>.
- **Dependencias del sistema (Linux) para la IHM** (`eframe`/`egui`, OpenGL/winit):
  - Debian/Ubuntu: `libxkbcommon-dev`, `libwayland-dev`, `libxcb1-dev`,
    `libgl1-mesa-dev` (o equivalentes), más un servidor gráfico (X11/Wayland).
  - La IHM necesita una **pantalla**: en entorno headless, la ventana no
    se abre (el servidor Modbus, en cambio, no depende de la pantalla).
- Acceso de red al registro crates.io para la primera compilación.

---

## 2. Comandos habituales

```bash
cargo check --workspace          # Verificación rápida (sin codegen)
cargo build --workspace          # Compilación debug
cargo build --release            # Compilación optimizada (LTO thin)
cargo test  --workspace          # Tests unitarios + integración
cargo clippy --workspace --all-targets   # Lint (debe quedar SIN advertencias)
cargo run -p mock_bin_ru_modbustcp       # Lanza el regulador

# Archivo de configuración alternativo:
MOCK_CONFIG=./ma_config.toml cargo run -p mock_bin_ru_modbustcp
# Registro detallado:
RUST_LOG=debug cargo run -p mock_bin_ru_modbustcp
```

Binario producido: `target/debug/orme` o `target/release/orme` (el paquete Cargo
sigue siendo `mock_bin_ru_modbustcp`, pero el ejecutable se llama **`orme`** — ver
`[[bin]]` en el `Cargo.toml` del crate).

### Features de Cargo

| Feature | Por defecto | Efecto |
|---------|:---------:|-------|
| `gui` | ✅ | IHM `egui`/`eframe` (si no, binario headless) |
| `rtu` | ✅ | Transporte Modbus RTU serie (RS485) mediante `tokio-serial` |

```bash
cargo build --no-default-features                 # headless, solo Modbus TCP
cargo build --no-default-features --features rtu  # headless TCP + RTU serie
cargo build --no-default-features --features gui  # IHM, solo TCP (sin serie)
```

> ⚠️ **`rtu` = dependencia nativa.** `tokio-serial` abre el puerto mediante termios
> (Linux); la enumeración `libudev` está desactivada (`default-features = false`).
> En **compilación cruzada** (`build-prod.sh`, ejecutables desktop con features por
> defecto), la imagen `cross` del target puede aun así requerir las cabeceras serie
> del sistema; si la cadena da problemas, retirar `rtu` del build correspondiente. El
> **Docker headless no se ve afectado** (compila en `--no-default-features`).

---

## 3. Organización del código

```
mock_lib_control/        Biblioteca de regulación (pura, sin IO, comprobable)
  src/pid.rs             PID anti-windup
  src/onoff.rs           Todo-o-nada con histéresis simétrica + anti-ciclo-corto
  src/pwm.rs             Relé de ciclo (PWM / time-proportioning)
  src/process.rs         Función de transferencia FOPDT
  src/lib.rs             ControllerKind + reexportaciones (feature `serde` opcional)

mock_bin_ru_modbustcp/   Binario regulador
  src/main.rs            Arranque: config, runtime Tokio, actores, IHM
  src/regulator.rs       Modelo de negocio síncrono (estado, Command, step)
  src/config.rs          AppConfig (TOML), Transport/SerialConfig, IpFilter, ServerStatus
  src/map.rs             Plan de direccionamiento Modbus (FUENTE DE VERDAD)
  src/modbus_server.rs   RegulatorService (trait Service) + maestro único TCP + serve_rtu
  src/gui.rs             IHM egui (página única + modal de Parámetros)
  src/actors/
    simulation.rs        Bucle de regulación (tick)
    network.rs           Servidor Modbus TCP/RTU (re)configurable en caliente

docs/                    Diseño, tabla Modbus, mantenimiento
```

**Regla de oro**: la lógica de negocio (`mock_lib_control`, `regulator.rs`) permanece
**síncrona y probada**; la parte asíncrona queda confinada a los actores y a la IO.

---

## 4. Configuración

- Archivo: `mock_ru_modbustcp.toml` en el directorio actual, o ruta
  indicada por la variable de entorno `MOCK_CONFIG`.
- Cargado al arranque; **valores por defecto** si está ausente o es ilegible (se
  registra una advertencia, la aplicación arranca de todos modos).
- Guardado desde la IHM (botones *Aplicar* / *Guardar ajustes* /
  *Restablecer por defecto*).

Estructura (todas las secciones son opcionales, completadas por defecto):

```toml
language = "es"
check_updates = true       # comprobar al arranque si existe una release más reciente (IHM)

[network]
bind_ip = "0.0.0.0"
port = 5502
allowlist = ["192.168.1.*", "127.0.0.1"]   # vacía = todas las IP autorizadas

[process]   # función de transferencia G(s) = K·e^(-L·s)/(1+T·s)
gain = 1.6        # K (unidad/%)
tau = 30.0        # T (s)
dead_time = 2.0   # L (s)
ambient = 20.0

[regulation]
sp_min = 0.0
sp_max = 250.0
hysteresis = 2.0
[regulation.pid_heat]   # sentido 1 (calor)
kp = 4.0 ; ki = 0.25 ; kd = 1.0 ; out_min = 0.0 ; out_max = 100.0
[regulation.pid_cool]   # sentido 2 (frío)
kp = 4.0 ; ki = 0.25 ; kd = 1.0 ; out_min = 0.0 ; out_max = 100.0
```

> Los **valores por defecto** tienen una **fuente única**: `RegulatorConfig::default`
> en `regulator.rs`. `ProcessConfig`/`RegulationConfig` (config.rs) derivan de ella.
> Para cambiar un valor por defecto, modificar únicamente `RegulatorConfig::default`.

### Comprobación de actualización

Si `check_updates = true` (por defecto) **y** el binario está compilado con la
feature `gui`, la IHM consulta **al arranque** la última release publicada en
GitHub (`CESAMLAB/cesam-tools`) y compara su número con la versión actual. Una
versión más reciente muestra un banner clicable «🔔 Actualización disponible».
El botón *Comprobar ahora* (modal *Ajustes*) relanza la comprobación.

- La petición HTTPS se ejecuta en un **hilo dedicado**, acotada por un timeout
  (5 s): sin conexión o GitHub inaccesible nunca obstaculiza el arranque.
- La lógica vive en la crate compartida **`mock_lib_update`** (`ureq`/`rustls`,
  raíces Mozilla embebidas → compilación cruzada limpia con `cross`).
- **Build headless** (`--no-default-features`): la comprobación —y toda la
  dependencia de red/TLS— está **ausente**. En servidor, gestionar las
  actualizaciones mediante apt/Docker. Desactivable por el operador (casilla del
  modal).

---

## 5. Dependencias y trampas de versión

| Crate | Rol | Punto de atención |
|-------|------|-------------------|
| `tokio` | runtime async | features: `rt-multi-thread, macros, net, time, sync` |
| `ractor` | actores | features por defecto (async nativo, **no** `async-trait`) |
| `tokio-serial` | Modbus RTU serie | opcional (feature `rtu`), `default-features = false` (sin enumeración libudev) |
| `tokio-modbus` | Modbus TCP | `default-features = false`, feature **`tcp-server`** |
| `eframe`/`egui` | IHM | versiones ligadas entre sí |
| `egui_plot` | curva | ⚠️ **versionado una menor por delante de `egui`**: para `egui` 0.33 → `egui_plot` **0.34** |
| `serde`/`toml` | persistencia | `mock_lib_control` expone una feature `serde` activada por el binario |
| `mock_lib_update` (`ureq`/`rustls`) | comprob. de actualización | **solo feature `gui`**; rustls 0.23 (webpki actualizado); ausente en headless |

Las versiones compartidas están centralizadas en `[workspace.dependencies]` del
`Cargo.toml` raíz. Para subir `egui`/`eframe`, **comprobar la versión
correspondiente de `egui_plot`** (de lo contrario error «two versions of crate egui»).

---

## 6. Ampliar el proyecto

### 6.1 Añadir un punto Modbus

Todo ocurre en **`map.rs`** (luego el snapshot/Command si es necesario):

1. Declarar la constante de dirección y ajustar el `*_COUNT` de la tabla afectada.
2. Rellenar el valor en `MemoryMap::refresh_from` (estado → registro).
3. Si el punto es escribible, decodificarlo en `coil_to_command` /
   `holdings_to_commands` (registro → `Command`).
4. Actualizar el comentario de documentación de cabecera **y** [table_modbus.md](table_modbus.md).
5. Añadir la fila en la tabla en vivo de la IHM (`gui.rs::modbus_rows`).

### 6.2 Añadir un comando / un ajuste

1. Variante en `enum Command` (`regulator.rs`) + tratamiento en `Regulator::apply`.
2. Campo en `RegulatorSnapshot` si el valor debe ser observable.
3. Cableado IHM (`gui.rs`) y/o decodificación Modbus (`map.rs`).
4. Si es persistente: campo en `AppConfig` (`config.rs`) + `to_regulator_config`.

### 6.3 Añadir un nuevo instrumento

1. Crear `mock_bin_<nom>/` y añadirlo a los `members` del `Cargo.toml` raíz.
2. Reutilizar `mock_lib_control`; factorizar todo lo común en una `mock_lib_*`.
3. Seguir el mismo reparto: modelo síncrono, actor(es) ractor, capa
   de protocolo, IHM. Convención de nombre: `mock_bin_<type>_<protocole>`.

---

## 7. Estrategia de pruebas

- **Unitarias** (`mock_lib_control`): PID (proporcional, acotación, anti-windup),
  TOR (zona muerta), proceso (convergencia en régimen permanente).
- **Dominio** (`regulator.rs`): convergencia PID en auto, salida en manual,
  retorno al ambiente al detener.
- **Mapping** (`map.rs`): round-trip `f32`↔registros, decodificación de escritura,
  rechazo de escritura `f32` parcial.
- **Config / red** (`config.rs`, `actors/network.rs`): round-trip TOML, filtro
  IP (comodines), arranque efectivo del servidor (bind en puerto efímero).

Lanzar: `cargo test --workspace`. Las pruebas son **deterministas y sin IHM**.

---

## 8. Resolución de problemas

| Síntoma | Pista |
|----------|-------|
| «two versions of crate `egui`» | Desajuste `egui_plot` / `egui`: alinear las versiones (§5). |
| La IHM no se abre | Pantalla ausente (headless) o libs del sistema faltantes (§1). |
| `Modbus ✖ fallo en la escucha` en el encabezado | Puerto ya en uso o < 1024 sin privilegios: cambiar el puerto en *Parámetros*. |
| Un cliente es rechazado | IP fuera de la **lista blanca**: vaciar la lista o añadir un patrón (`192.168.1.*`). |
| Valores `f32` aberrantes en el cliente | Orden de las palabras (palabra de mayor peso al inicio): ver [table_modbus.md](table_modbus.md). |
| Una escritura de consigna `f32` se ignora | Escribir **los dos** registros del par en una sola petición. |
| Config no recargada | Directorio actual incorrecto o `MOCK_CONFIG`; comprobar el registro al arranque. |
| Sin icono en la barra de tareas (Linux) | Sesión **Wayland**: el icono embebido se ignora. Instalar la entrada de escritorio: `scripts/install-desktop.sh` (§9). |

Aumentar la verbosidad: `RUST_LOG=debug` (o `trace`).

---

## 9. Build de distribución

```bash
cargo build --release
# Binario autónomo:
target/release/orme
```

El perfil `release` activa `lto = "thin"` y `opt-level = 3` (ver `Cargo.toml`
raíz). Para distribuir: proporcionar el binario + un `mock_ru_modbustcp.toml`
de ejemplo. Licencia **MIT** (archivo `LICENSE`).

### Feature `gui` (build con / sin interfaz)

La IHM está detrás de la feature de Cargo **`gui`**, activada por defecto:

```bash
cargo build --release                       # con IHM (puesto de trabajo)
cargo build --release --no-default-features  # «headless»: Modbus + simulación, sin IHM
```

El modo **headless** está destinado a despliegues sin pantalla (Raspberry Pi en
servicio) y hace la **compilación cruzada ARM trivial** (ninguna dependencia
gráfica que enlazar).

### Integración con el escritorio Linux (icono de la barra de tareas)

El icono ORME está embebido en el binario (`branding.rs` → `with_icon`). Esto basta
en **X11, Windows y macOS**. Pero bajo **Wayland**, el compositor **ignora** el icono
embebido: asocia la ventana a su **`app_id`** («orme», definido en `main.rs` mediante
`ViewportBuilder::with_app_id`) a un archivo `orme.desktop` del mismo nombre, y muestra
el `Icon=` de ese archivo (resuelto en el tema de iconos `hicolor`).

Para obtener el icono bajo Wayland, instalar la entrada de escritorio para el usuario
actual:

```bash
scripts/install-desktop.sh
```

El script copia:

| Fuente | Destino |
|--------|-------------|
| `pic/orme-icon.png` | `~/.local/share/icons/hicolor/256x256/apps/orme.png` |
| `packaging/orme.desktop` | `~/.local/share/applications/orme.desktop` |

luego refresca las cachés (`gtk-update-icon-cache`, `update-desktop-database`). El icono
aparece en el próximo lanzamiento de ORME (y de forma fiable tras un nuevo inicio de
sesión de la sesión Wayland).

> ⚠️ Tres nombres **deben permanecer alineados**: el `app_id` (`main.rs`), el nombre del
> archivo `orme.desktop` y su `StartupWMClass`, y el nombre del icono `orme.png`
> (= `Icon=orme`). `packaging/orme.desktop` supone un ejecutable `orme` en el `PATH`
> (campo `Exec=`); en dev (`cargo run`) este campo no incide en la visualización del
> icono.

---

## 10. Build «prod» — compilación cruzada desde Linux

### Procedimiento único

Todo se produce **desde Linux** mediante
[`scripts/build-prod.sh`](../../../scripts/build-prod.sh), que construye **todos los
instrumentos del workspace** (ORME *y* OSNE) en una sola pasada. Para cada instrumento
(`<bin>` = `orme`, `osne`):

| Salida | Destino | IHM | Método |
|--------|-------|-----|---------|
| `dist/<bin>-linux-x86_64` | `x86_64-unknown-linux-gnu` | ✅ | `cross` |
| `dist/<bin>-windows-x86_64.exe` | `x86_64-pc-windows-gnu` | ✅ | `cross` (mingw) |
| `dist/<bin>-rpi-arm64` | `aarch64-unknown-linux-gnu` (Pi 3/4/5, Pi OS 64b) | ✅ | `cross` |
| Imagen Docker headless `<bin>:headless` | multi-arch `linux/amd64` + `linux/arm64` | ❌ | `docker buildx` |
| `dist/<bin>_<ver>_amd64.deb` / `_arm64.deb` | paquete Debian/Ubuntu | ✅ | `dpkg-deb` |
| `dist/<bin>-setup-x86_64.exe` | instalador Windows | ✅ | NSIS (`makensis`) |

```bash
# Requisitos previos (una vez) — Docker debe estar en ejecución:
cargo install cross

# Producir todo (ejecutables ORME + OSNE en dist/ + imágenes Docker locales amd64 cargadas):
scripts/build-prod.sh

# Variante: imágenes Docker MULTI-ARCH enviadas a un registro (<prefix>/<bin>:latest):
IMAGE_PREFIX=ghcr.io/<cuenta> scripts/build-prod.sh

# Construir un solo instrumento:
ONLY=orme scripts/build-prod.sh
```

### Por qué `cross` para TODOS los builds (incluido Linux x86_64)

`cross` proporciona imágenes Docker que contienen las toolchains de cada destino: ni
`mingw-w64`, ni toolchain ARM, ni *sysroot* que instalar.

⚠️ **No mezclar `cargo` nativo y `cross` en el mismo `target/`.** Ambos
usan versiones de `rustc` diferentes (host vs contenedor); las
**proc-macros** compiladas por uno son rechazadas por el otro, de ahí los errores
`can't find crate for …_derive` (ej. `zerofrom_derive`, `tracing_attributes`).
El script pasa por tanto **siempre por `cross`**, incluso para Linux x86_64 — una sola
toolchain, builds reproducibles. (Si el error surge a pesar de todo tras un
build nativo anterior: `rm -rf target/release` y relanzar.)

### IHM compilada de forma cruzada hacia ARM: por qué funciona

`eframe`/`egui` cargan OpenGL, X11/Wayland y xkbcommon **en ejecución**
(`dlopen`): el binario solo enlaza al build la `libc`. Ninguna lib gráfica ARM
es por tanto necesaria del lado cross. En la Raspberry Pi, prever un entorno
de escritorio (mesa/X11 o Wayland) — presente en Raspberry Pi OS *Desktop*.

> Para un **Raspbian de 32 bits**, apuntar a `armv7-unknown-linux-gnueabihf` (adaptar
> los destinos en el script).

### Imagen Docker headless «en cualquier lugar»

La imagen ([`docker/Dockerfile.headless`](../../../docker/Dockerfile.headless)) parte de
`debian:bookworm-slim` y **copia** el binario headless de la arquitectura deseada
(ninguna compilación en la imagen → sin QEMU). `docker buildx` ensambla el
multi-arch `amd64`+`arm64`. El servidor escucha en `5502`. Montar un volumen en
`/data` para proporcionar/persistir `mock_ru_modbustcp.toml`.

```bash
# Sin registro: imagen local amd64 cargada, probable de inmediato
docker run --rm -p 5502:5502 -v "$PWD/conf:/data" orme:headless
```

### Instaladores (`.deb` Linux/RPi + setup Windows)

Al final del build, `build-prod.sh` llama a
[`scripts/make-installers.sh <bin>`](../../../scripts/make-installers.sh), que
transforma los ejecutables release de `dist/` en **instaladores**:

| Instalador | Origen | Contenido | Herramienta |
|------------|--------|-----------|-------------|
| `<bin>_<ver>_amd64.deb` | `dist/<bin>-linux-x86_64` | binario → `/usr/bin`, entrada de escritorio, icono hicolor | `dpkg-deb` |
| `<bin>_<ver>_arm64.deb` | `dist/<bin>-rpi-arm64` | ídem (Raspberry Pi OS de 64 bits) | `dpkg-deb` |
| `<bin>-setup-x86_64.exe` | `dist/<bin>-windows-x86_64.exe` | exe + accesos directos (menú Inicio/escritorio) + desinstalador | NSIS (`makensis`) |

- Los `.deb` colocan el icono y el `.desktop`; un `postinst` refresca las cachés
  de iconos y la base `.desktop`. Dependencias: `libc6`; recomendaciones
  gráficas (`libgl1`, `libxkbcommon0`, `libwayland-client0`).
- El instalador Windows proviene de
  [`packaging/windows/installer.nsi`](../../../packaging/windows/installer.nsi);
  sus accesos directos llevan un icono `.ico` multi-resolución derivado de
  `pic/<bin>-icon.png` (mediante Pillow).
- **Requisitos previos**: `dpkg-deb` (Debian/Ubuntu) para los `.deb`, **`makensis`**
  (`sudo apt install nsis`) para el setup Windows, `python3`+Pillow para el `.ico`.
  Todo objetivo cuya herramienta/artefacto falte es **avisado y omitido** (el build no
  se rompe). Desactivar mediante `INSTALLERS=0`, o (re)generar solo los instaladores
  de un instrumento: `scripts/make-installers.sh orme`.

### Build nativo Windows (MSVC) — opcional

El `.exe` producido arriba es **GNU/mingw** (ejecutable Windows nativo, IHM
incluida). Si se requiere un binario **MSVC**, compilar en una máquina Windows
con [`scripts/build-windows.ps1`](../../../scripts/build-windows.ps1) (requisitos:
Rust + *Visual Studio Build Tools*, carga «Desarrollo Desktop en C++»), o
desde Linux mediante `cargo-xwin` (`cargo xwin build --release --target x86_64-pc-windows-msvc`).

### Notas

- Los binarios están **enlazados dinámicamente a la glibc**; compilados mediante `cross`
  (baseline glibc antigua) funcionan en distribuciones recientes (y en
  `debian:bookworm-slim`). Para un binario totalmente estático, apuntar a `*-musl`.
- `dist/` está ignorado por git (artefactos de build).

---

## 11. Convenciones

- Código y comentarios en **francés**.
- `cargo clippy --workspace` **sin advertencias** antes de cualquier commit.
- Todo nuevo comportamiento de negocio o de mapping se acompaña de un **test**.
- El plan de direccionamiento se modifica en **`map.rs`** (fuente de verdad), con
  actualización conjunta de la documentación.
