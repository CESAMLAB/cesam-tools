# Documentación de mantenimiento — ORPD / PROFIBUS DP (workspace `cesam-tools`)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · **ES** · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

> Crate: `mock_bin_ru_pbdp` · Ejecutable: **ru_pbdp** · Marca: **ORPD**
> Público: desarrolladores que mantienen, corrigen o amplían el proyecto.
> Véase también: [conception.md](conception.md) · [reference_profibus.md](reference_profibus.md).

---

## 1. Requisitos previos

- **Rust stable** (edición 2021, `rust-version` ≥ 1.85). Instalación:
  <https://rustup.rs>.
- **Dependencias de sistema (Linux) para la IU** (`eframe`/`egui`,
  OpenGL/winit): `libxkbcommon-dev`, `libwayland-dev`, `libxcb1-dev`,
  `libgl1-mesa-dev` (o equivalentes), más un servidor gráfico (X11/Wayland).
  La IU necesita una **pantalla**: en un entorno headless, la ventana no
  se abre.
- **Enlace serie** (acceso al puerto, `/dev/ttyUSB*`, grupo `dialout` en
  Linux): a diferencia de ORME/OSNE, **esto no es una función opcional**
  aquí — `tokio-serial` es una dependencia directa (véase §5), siendo el
  enlace serie el único transporte de este instrumento (no existe
  equivalente estándar de «PROFIBUS sobre TCP»). Sin hardware, la IU
  arranca igualmente (el error de apertura se muestra en la cabecera, la
  simulación sigue funcionando) — véase
  [`manuel_utilisateur.md`](manuel_utilisateur.md) §2.
- Acceso de red al registro crates.io para la primera compilación.

---

## 2. Comandos habituales

```bash
cargo check -p mock_bin_ru_pbdp          # Verificación rápida (sin codegen)
cargo build -p mock_bin_ru_pbdp          # Compilación debug
cargo build --release -p mock_bin_ru_pbdp   # Compilación optimizada (LTO thin)
cargo test  -p mock_bin_ru_pbdp          # Pruebas unitarias + de integración
cargo clippy --workspace --all-targets    # Lint (debe quedar SIN advertencias)
cargo run   -p mock_bin_ru_pbdp          # Lanza la IU + el enlace serie PROFIBUS DP

# Archivo de configuración alternativo:
MOCK_CONFIG=./mi_config.toml cargo run -p mock_bin_ru_pbdp
# Registro detallado:
RUST_LOG=debug cargo run -p mock_bin_ru_pbdp
```

Binario producido: `target/debug/ru_pbdp` o `target/release/ru_pbdp` (el
paquete Cargo sigue siendo `mock_bin_ru_pbdp`; el ejecutable y el nombre
comercial «ORPD» son solo documentales, véase `[[bin]]` en el `Cargo.toml`
del crate).

### Features de Cargo

| Feature | Por defecto | Efecto |
|---------|:---------:|-------|
| `gui` | ✅ | IU `egui`/`eframe` + verificación de actualizaciones (si no, binario headless) |

```bash
cargo build -p mock_bin_ru_pbdp --no-default-features   # headless: enlace serie + simulación, sin IU
```

> ⚠️ **Diferencia con ORME/OSNE**: en esos dos instrumentos, el enlace
> serie (RTU/serie) es en sí mismo una **función opcional** junto a un
> transporte TCP siempre presente, y `--no-default-features` puede
> excluirlo. Aquí **no existe una variante «sin serie»**: `tokio-serial`
> es una dependencia directa (no controlada por feature), presente en
> **todas** las compilaciones, incluida la headless — es el único
> transporte del instrumento.

---

## 3. Organización del código

```
mock_lib_control/        Biblioteca de regulación reutilizable (pura, sin IO, comprobable)
  src/pid.rs             PID anti-windup
  src/lib.rs             re-exportaciones (feature `serde` opcional)

mock_bin_ru_pbdp/        Binario regulador PROFIBUS DP (ejecutable `ru_pbdp`)
  src/main.rs            Arranque: configuración, runtime Tokio, actores, IU/headless
  src/regulator.rs        Modelo de negocio síncrono (PID + proceso de 1er orden), Command, paso
  src/config.rs           AppConfig (TOML), SerialConfig, ProcessConfig, RegulationConfig, ServerStatus
  src/profibus.rs         Protocolo PROFIBUS DP-V0: códec de tramas + FCS + SlaveFsm (FUENTE DE VERDAD)
  src/profibus_server.rs  Bucle de sesión serie (leer trama → SlaveFsm → responder) + vigilante
  src/map.rs              Disposición de los bloques de E/S Output/Input <-> Command del regulador
  src/trace.rs            Registro circular de tramas (mini-terminal de la IU)
  src/gui.rs              IU egui (página única + mini-terminal + modal Ajustes)
  src/branding.rs         Logos incrustados (feature `gui`)
  src/i18n.rs             Catálogo i18n tipado (8 idiomas), sin dependencia
  src/actors/
    simulation.rs         Bucle de regulación (paso de simulación 50 ms)
    network.rs            Actor del enlace serie PROFIBUS DP, reconfigurable en caliente

docs/                     Diseño, referencia PROFIBUS, manual, mantenimiento (multilingüe)
```

**Regla de oro**: la lógica de negocio (`mock_lib_control`, `regulator.rs`,
`profibus.rs`, `map.rs`) permanece **síncrona y probada**; lo asíncrono se
confina a los actores y a la E/S serie. Modelo de regulación calcado de
**ORME** (`mock_bin_ru_modbus`) — mismos invariantes.

---

## 4. Configuración

- Archivo: `mock_ru_pbdp.toml` en el directorio actual, o la ruta indicada
  por la variable de entorno `MOCK_CONFIG`.
- Cargado al arrancar; **valores por defecto** si falta o es ilegible (se
  registra una advertencia, la aplicación arranca igualmente).
- **Todo valor procedente del TOML se sanea**
  (`AppConfig::sanitized`): límites de consigna/PID reordenados, valores en
  coma flotante forzados a finitos, `τ ≥ 1e-3`, `dead_time` acotado,
  **dirección de estación acotada a `[0, 125]`**. **Invariante: nunca
  llamar a `f32::clamp` con límites no validados** (entra en pánico si
  `min > max` o `NaN`).
- Se guarda desde la IU (botones *Aplicar* / *Guardar* / *Restablecer por
  defecto*).

Estructura (todas las secciones son opcionales, completadas por defecto):

```toml
language = "es"
check_updates = true       # comprobar al arrancar si existe una versión más reciente (IU)

[network.serial]
port = "/dev/ttyUSB0"      # "COM3" por defecto en Windows
baud = 500000              # valor normalizado PROFIBUS DP (9600 .. 12000000)
station_address = 3        # dirección del esclavo simulado (0-125)
watchdog_enabled = true    # permite el vigilante anunciado por el maestro (Set_Prm)

[process]
gain = 1.6 ; tau = 30.0 ; dead_time = 2.0 ; ambient = 20.0

[regulation]
sp_min = 0.0 ; sp_max = 250.0
hysteresis = 2.0 ; tor_min_cycle = 5.0 ; pwm_period = 10.0
[regulation.pid_heat]
kp = ... ; ki = ... ; kd = ... ; out_min = 0.0 ; out_max = 100.0
[regulation.pid_cool]
kp = ... ; ki = ... ; kd = ... ; out_min = 0.0 ; out_max = 100.0
```

> El **formato de trama serie (8E1)** está fijado por la norma PROFIBUS DP
> y **no** es un campo de configuración — véase `SerialConfig::open` en
> [`config.rs`](../../src/config.rs). A diferencia de ORME/OSNE, **no hay
> lista blanca de IP** (el enlace serie es intrínsecamente punto a punto).

### Verificación de actualizaciones

Si `check_updates = true` (por defecto) **y** el binario se compila con la
feature `gui`, la IU consulta **al arrancar** la última versión publicada
en GitHub (`CESAMLAB/cesam-tools`) mediante el crate compartido
**`mock_lib_update`** (`ureq`/`rustls`, raíces incrustadas, hilo acotado por
tiempo límite). **Ausente en las compilaciones headless**
(`--no-default-features`).

---

## 5. Dependencias y trampas de versión

| Crate | Rol | Punto de atención |
|-------|------|-------------------|
| `tokio` | runtime asíncrono | features compartidas + `io-util` |
| `ractor` | actores | features por defecto |
| `tokio-serial` | enlace PROFIBUS DP | **dependencia directa, no controlada por feature** (véase §2); `default-features = false` (sin enumeración `libudev`) |
| `eframe`/`egui` | IU | versiones ligadas entre sí, feature `gui` |
| `egui_plot` | curva | ⚠️ **versionado una versión menor por delante de `egui`**: para `egui` 0.33 → `egui_plot` **0.34** |
| `serde`/`toml` | persistencia | `mock_lib_control` expone una feature `serde` activada por el binario |
| `mock_lib_update` (`ureq`/`rustls`) | verif. de actualizaciones | solo feature `gui`; ausente en headless |

Las versiones compartidas se centralizan en `[workspace.dependencies]` del
`Cargo.toml` raíz. Al subir `egui`/`eframe`, **comprobar la versión
correspondiente de `egui_plot`** (si no, error «two versions of crate
egui»).

---

## 6. Ampliar el proyecto

### 6.1 Añadir un servicio PROFIBUS (SAP)

Todo ocurre en **[`profibus.rs`](../../src/profibus.rs)** (fuente de
verdad del protocolo):

1. Añadir la constante `SAP_*` y la variante correspondiente en
   `enum Request`; conectar la decodificación en `decode_request` (y, para
   las pruebas, en `encode_request`).
2. Tratar la nueva petición en `SlaveFsm::handle` (transición de estado si
   procede, `Handled` devuelto).
3. Actualizar el comentario de documentación de cabecera y
   **[reference_profibus.md](reference_profibus.md)**.
4. Añadir una prueba en el módulo `tests` de `profibus.rs` (y, si la
   sesión completa está afectada, en `profibus_server.rs`).

### 6.2 Modificar los bloques de E/S (`Output`/`Input`)

1. Ajustar la disposición en **[`map.rs`](../../src/map.rs)**
   (`decode_output`/`encode_input`), manteniendo `OUTPUT_LEN`/`INPUT_LEN`
   coherentes con `SlaveProfile` (`profibus_server.rs`).
2. Actualizar la tabla de
   **[reference_profibus.md](reference_profibus.md)** §3 (fuente de verdad
   documental, copiada del comentario de documentación de `map.rs`).
3. Añadir una prueba de ida y vuelta en `map.rs`.

### 6.3 Añadir un comando de negocio / un ajuste de IU

1. Variante en `enum Command` (`regulator.rs`) + tratamiento en
   `Regulator::apply` (con saneamiento).
2. Campo en `RegulatorSnapshot` si el valor debe ser observable.
3. Conexión en la IU (`gui.rs`) mediante un `cast` no bloqueante.
4. Si es persistente: campo en `AppConfig` (`config.rs`) + saneamiento en
   `sanitized` + reporte en `to_regulator_config`.

### 6.4 Añadir una cadena de interfaz (i18n)

Toda cadena de la IU **debe** pasar por una clave `Msg` (`i18n.rs`) con sus
**8 traducciones** (array de tamaño fijo verificado en tiempo de
compilación). Los identificadores de servicio PROFIBUS y los sufijos de
unidad permanecen codificados de forma fija.

### 6.5 Añadir un nuevo instrumento

1. Crear `mock_bin_<nombre>/` y añadirlo a los `members` del `Cargo.toml`
   raíz.
2. Reutilizar `mock_lib_control`; factorizar lo común en una `mock_lib_*`.
3. Seguir la misma organización: modelo síncrono, actor(es) `ractor`, capa
   de protocolo, IU. Convención de nombre: `mock_bin_<tipo>_<protocolo>`.

---

## 7. Estrategia de pruebas

- **Códec de tramas** (`profibus.rs`): ida y vuelta de
  `SD1`/`SD2`/`SD3`/`SD4`, rechazo de suma de control y longitud
  incorrectas, codificación/decodificación de las peticiones
  (`Slave_Diag`/`Set_Prm`/`Chk_Cfg`/`Data_Exchange`) y del byte de modo.
- **Máquina de estados** (`profibus.rs`): secuencia completa
  `Power_On → Wait_Prm → Wait_Cfg → Data_Exchange`, rechazo de un
  `Set_Prm` con identificador erróneo (permanece en `Wait_Prm`).
- **Bloques de E/S** (`map.rs`): un bloque de salida demasiado corto → sin
  comando; ida y vuelta de consigna/modo; el bloque de entrada refleja la
  instantánea (bits de estado, medida).
- **Configuración** (`config.rs`): ida y vuelta TOML, saneamiento (límites
  invertidos, valores no finitos, dirección de estación fuera de rango)
  sin pánico, error limpio al abrir un puerto serie ausente.
- **Sesión de red** (`profibus_server.rs`, `#[tokio::test]` sobre
  `tokio::io::duplex`): handshake completo hasta `Data_Exchange` con
  aplicación efectiva de los comandos, una trama dirigida a otra estación
  ignorada (sin actividad marcada), vencimiento del vigilante forzando el
  estado seguro.

Ejecutar: `cargo test -p mock_bin_ru_pbdp` (o `--workspace`) — **36
pruebas**, todas **deterministas y sin IU**, ninguna prueba lenta/
`#[ignore]` (a diferencia de ORUE, cuya generación RSA justifica pruebas
ignoradas).

---

## 8. Resolución de problemas

| Síntoma | Pista |
|----------|-------|
| «two versions of crate `egui`» | Discrepancia `egui_plot` / `egui`: alinear las versiones (§5). |
| La IU no se abre | Sin pantalla (headless) o faltan bibliotecas de sistema (§1). |
| Error al abrir el puerto serie (cabecera de la IU) | Puerto ausente, ruta incorrecta, o permisos (grupo `dialout` en Linux) — la simulación sigue funcionando sin enlace. |
| El enlace permanece en `Wait_Prm` | El maestro no envía `Set_Prm` con el identificador esperado (`0xEE01`) — véase [reference_profibus.md](reference_profibus.md) §2. |
| El enlace permanece en `Wait_Cfg` | El `Chk_Cfg` recibido no anuncia `out_len=45`/`in_len=17`. |
| El aparato se detiene solo | Vigilante de protocolo activado (silencio prolongado del maestro) — estado seguro esperado, no un fallo. |
| Ningún vigilante aunque el maestro solicite uno | `watchdog_enabled = false` en la configuración local: la solicitud del maestro se ignora deliberadamente. |

Aumentar la verbosidad: `RUST_LOG=debug` (o `trace`).

---

## 9. Compilación de distribución

```bash
cargo build --release -p mock_bin_ru_pbdp
# Binario independiente:
target/release/ru_pbdp
```

El perfil `release` activa `lto = "thin"` y `opt-level = 3` (véase el
`Cargo.toml` raíz). Para distribuir: proporcionar el binario más un
`mock_ru_pbdp.toml` de ejemplo. Licencia **MIT** (archivo `LICENSE`).

### Feature `gui` (compilación con / sin interfaz)

```bash
cargo build --release -p mock_bin_ru_pbdp                       # con IU (puesto de trabajo)
cargo build --release -p mock_bin_ru_pbdp --no-default-features  # «headless»: enlace serie + simulación, sin IU
```

A diferencia de OSNE, el modo **headless** no hace opcional el enlace
serie (§2): solo retira la IU. Sigue siendo pertinente para un despliegue
sin pantalla conectado a un puerto serie/USB real.

### Integración en el escritorio Linux (icono de la barra de tareas)

El icono ORPD (`pic/ru_pbdp-icon.png`, generado por
[`pic/ru_pbdp-logo.gen.py`](../../../pic/ru_pbdp-logo.gen.py)) está
**incrustado** en el binario (`branding.rs` → `window_icon`). Esto basta en
**X11, Windows y macOS**. En **Wayland**, el compositor **ignora** el
icono incrustado: asocia la ventana a su **`app_id`** («ru_pbdp», definido
en `main.rs` mediante `with_app_id`) a un archivo `ru_pbdp.desktop` del
mismo nombre, y muestra el `Icon=ru_pbdp` resuelto en el tema de iconos
`hicolor`.

Para obtener el icono en Wayland, instale la entrada de escritorio para el
usuario actual:

```bash
scripts/install-desktop.sh ru_pbdp
```

El script copia:

| Origen | Destino |
|--------|-------------|
| `pic/ru_pbdp-icon.png` | `~/.local/share/icons/hicolor/256x256/apps/ru_pbdp.png` |
| `packaging/ru_pbdp.desktop` | `~/.local/share/applications/ru_pbdp.desktop` |

y a continuación refresca las cachés. Tres nombres **deben permanecer
alineados**: el `app_id` (`main.rs`), el archivo `ru_pbdp.desktop` (+ su
`StartupWMClass`) y el icono `ru_pbdp.png` (= `Icon=ru_pbdp`).

---

## 10. Compilación «prod» — compilación cruzada desde Linux

Todo se produce **desde Linux** mediante
[`scripts/build-prod.sh`](../../../scripts/build-prod.sh), que compila
**todos los instrumentos del workspace** (tabla `INSTRUMENTS`, entrada
`mock_bin_ru_pbdp:ru_pbdp:0` — puerto `0`: enlace serie, sin puerto IP):

| Salida | Objetivo | IU | Método |
|--------|-------|-----|---------|
| `dist/ru_pbdp-linux-x86_64` | `x86_64-unknown-linux-gnu` | ✅ | `cross` |
| `dist/ru_pbdp-windows-x86_64.exe` | `x86_64-pc-windows-gnu` | ✅ | `cross` (mingw) |
| `dist/ru_pbdp-rpi-arm64` | `aarch64-unknown-linux-gnu` (Pi 3/4/5, Pi OS 64 bits) | ✅ | `cross` |
| Imagen Docker headless `ru_pbdp:headless` | multi-arch `amd64` + `arm64` | ❌ | `docker buildx` |
| `dist/ru_pbdp_<ver>_amd64.deb` / `_arm64.deb` | paquete Debian/Ubuntu | ✅ | `dpkg-deb` |
| `dist/ru_pbdp-setup-x86_64.exe` | instalador Windows | ✅ | NSIS (`makensis`) |

```bash
cargo install cross          # requisito previo (una vez) — Docker debe estar en ejecución
scripts/build-prod.sh        # todos los instrumentos, incluido ru_pbdp
ONLY=ru_pbdp scripts/build-prod.sh   # solo este instrumento
```

⚠️ **No mezclar `cargo` nativo y `cross`** en el mismo `target/`
(proc-macros incompatibles → `can't find crate for …_derive`). El script
siempre pasa por `cross`.

### Imagen Docker headless: utilidad limitada sin passthrough serie

La imagen ([`docker/Dockerfile.headless`](../../../docker/Dockerfile.headless))
se construye igual que para los demás instrumentos (`EXPOSE 0`, metadato
inerte), pero **solo es realmente útil con un dispositivo serie montado**
en el contenedor:

```bash
docker run --rm --device=/dev/ttyUSB0 -v "$PWD/conf:/data" ru_pbdp:headless
```

Sin `--device`, el contenedor arranca pero no puede abrir ningún puerto
serie (mismo comportamiento que la ausencia de hardware en local — véase
§8).

---

## 11. Convenciones

- Código y comentarios en **francés** (convención de todo el proyecto);
  registros y mensajes de error en **inglés**.
- `cargo clippy --workspace` **sin advertencias** antes de cualquier commit.
- Todo nuevo comportamiento de negocio o de protocolo va acompañado de una
  **prueba**.
- El protocolo PROFIBUS DP-V0 se modifica en **`profibus.rs`** (fuente de
  verdad), junto con una actualización de
  **[reference_profibus.md](reference_profibus.md)**.
