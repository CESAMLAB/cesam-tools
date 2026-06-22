# Documentación de mantenimiento — OSNE (workspace `cesam-tools`)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · **ES** · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

> Público: desarrolladores que mantienen, corrigen o amplían el proyecto.
> Ver también: [conception.md](conception.md) · [commandes_namur.md](commandes_namur.md).

---

## 1. Requisitos previos

- **Rust stable** (edición 2021, `rust-version` ≥ 1.85). Instalación: <https://rustup.rs>.
- **Dependencias del sistema (Linux) para la IHM** (`eframe`/`egui`, OpenGL/winit):
  - Debian/Ubuntu: `libxkbcommon-dev`, `libwayland-dev`, `libxcb1-dev`,
    `libgl1-mesa-dev` (o equivalentes), más un servidor gráfico (X11/Wayland).
  - La IHM necesita una **pantalla**: en entorno headless, la ventana no se abre
    (el servidor NAMUR, en cambio, no depende de la pantalla).
- **Enlace serie** (feature `serial`): acceso al puerto (`/dev/ttyUSB*`, grupo
  `dialout` en Linux). Sin equipo, usar el transporte **TCP**.
- Acceso de red al registro crates.io para la primera compilación.

---

## 2. Comandos habituales

```bash
cargo check -p mock_bin_su_namur          # Verificación rápida (sin codegen)
cargo build -p mock_bin_su_namur          # Compilación debug
cargo build --release -p mock_bin_su_namur   # Compilación optimizada (LTO thin)
cargo test  -p mock_bin_su_namur          # Tests unitarios + integración
cargo clippy --workspace --all-targets    # Lint (debe quedar SIN advertencias)
cargo run   -p mock_bin_su_namur          # Lanza el agitador (IHM + NAMUR/TCP)

# Archivo de configuración alternativo:
MOCK_CONFIG=./ma_config.toml cargo run -p mock_bin_su_namur
# Registro detallado:
RUST_LOG=debug cargo run -p mock_bin_su_namur
```

Binario producido: `target/debug/osne` o `target/release/osne` (el paquete Cargo
sigue siendo `mock_bin_su_namur`, pero el ejecutable se llama **`osne`** — ver
`[[bin]]` en el `Cargo.toml` del crate).

### Features Cargo

| Feature | Por defecto | Efecto |
|---------|:---------:|-------|
| `gui` | ✅ | IHM `egui`/`eframe` (si no, binario headless) |
| `serial` | ✅ | Transporte NAMUR sobre enlace serie RS-232 vía `tokio-serial` |

```bash
cargo build -p mock_bin_su_namur --no-default-features                  # headless, solo NAMUR/TCP
cargo build -p mock_bin_su_namur --no-default-features --features serial # headless TCP + serie
cargo build -p mock_bin_su_namur --no-default-features --features gui    # IHM, solo TCP (sin serie)
```

> ⚠️ **`serial` = dependencia nativa.** `tokio-serial` abre el puerto vía termios
> (Linux); la enumeración `libudev` está desactivada (`default-features = false`).
> En **compilación cruzada** (`build-prod.sh`, ejecutables de escritorio con las
> features por defecto), la imagen `cross` del target puede reclamar de todos modos
> las cabeceras serie; si la cadena da problemas, retirar `serial` del build
> correspondiente. El **Docker headless no se ve afectado** (compila en
> `--no-default-features`).

---

## 3. Organización del código

```
mock_lib_control/        Bibliothèque de régulation (pure, sans IO, testable)
  src/pid.rs             PID anti-emballement (réutilisé pour l'asservissement de vitesse)
  src/lib.rs             ré-exports (feature `serde` optionnelle)

mock_bin_su_namur/       Binaire agitateur (exécutable `osne`)
  src/main.rs            Démarrage : config, runtime Tokio, acteurs, IHM
  src/motor.rs           Modèle physique du moteur (dynamique rotationnelle, Euler)
  src/stirrer.rs         Modèle métier synchrone (état, Command, step) — possède le PID
  src/config.rs          AppConfig (TOML), Transport/SerialConfig, IpFilter, ServerStatus
  src/namur.rs           Protocole NAMUR : handle_line (SOURCE DE VÉRITÉ du jeu de commandes)
  src/namur_server.rs    Service NAMUR (lignes ASCII) + mono-maître TCP + serve série + chien de garde
  src/trace.rs           Journal circulaire des trames (mini-terminal IHM)
  src/gui.rs             IHM egui (page unique + mini-terminal + modal Paramètres)
  src/branding.rs        Logos embarqués (feature `gui`)
  src/i18n.rs            Catalogue i18n typé (8 langues), sans dépendance
  src/actors/
    simulation.rs        Boucle de simulation (tick 20 ms)
    network.rs           Serveur NAMUR TCP/série (re)configurable à chaud

docs/                    Conception, commandes NAMUR, manuel, maintenance (multilingue)
```

**Regla de oro**: la lógica de negocio (`mock_lib_control`, `motor.rs`,
`stirrer.rs`) sigue siendo **síncrona y probada**; lo asíncrono se limita a los
actores y a la IO. Calco exacto del regulador **ORME** (`mock_bin_ru_modbustcp`) —
mismos invariantes.

---

## 4. Configuración

- Archivo: `mock_su_namur.toml` en el directorio actual, o ruta proporcionada por
  la variable de entorno `MOCK_CONFIG`.
- Cargado al arrancar; **valores por defecto** si está ausente o es ilegible (se
  registra una advertencia, la aplicación arranca de todos modos).
- **Todo valor procedente del TOML se sanea** (`AppConfig::sanitized`): límites
  reordenados (`min ≤ max`), flotantes forzados a finitos, inercia/par/viscosidad
  estrictamente positivos. **Invariante: nunca `f32::clamp` con límites no
  validados** (entra en pánico si `min > max` o `NaN`).
- Guardado desde la IHM (botones *Aplicar* / *Guardar* / *Restablecer*).

Estructura (todas las secciones son opcionales, se completan por defecto):

```toml
language = "fr"

[network]
transport = "tcp"          # "tcp" ou "serial"
bind_ip = "0.0.0.0"
port = 4001
allowlist = ["192.168.1.*", "127.0.0.1"]   # vide = toutes IP autorisées
[network.serial]
port = "/dev/ttyUSB0"
baud = 9600 ; parity = "even" ; data_bits = 7 ; stop_bits = 1   # NAMUR 7E1

[motor]   # J·dω/dt = T − k·η·ω − frottement
inertia = 0.02      # J (réactivité)
load_coeff = 0.05   # k (poids de la viscosité)
friction = 2.0      # N·cm
torque_max = 100.0  # N·cm (plafond de la sortie PID)

[regulation]
speed_min = 0.0 ; speed_max = 2000.0
viscosity = 1.0 ; viscosity_min = 0.1 ; viscosity_max = 20.0
[regulation.pid]
kp = ... ; ki = ... ; kd = ... ; out_min = 0.0 ; out_max = 100.0
```

> Los **valores por defecto** tienen una **fuente única**: `StirrerConfig::default`
> en `stirrer.rs`. `MotorConfig`/`RegulationConfig` (config.rs) derivan de ella.
> Los límites de salida del PID (`out_min`/`out_max`) se **fuerzan** a
> `[0, couple_max]` en el momento de construir el agitador (`to_stirrer_config`).

---

## 5. Dependencias y trampas de versión

| Crate | Rol | Punto de atención |
|-------|------|-------------------|
| `tokio` | runtime async | features compartidas + **`io-util`** (BufReader / líneas ASCII NAMUR) |
| `ractor` | actores | features por defecto (async nativo, **no** `async-trait`) |
| `tokio-serial` | NAMUR serie | opcional (feature `serial`), `default-features = false` (sin enumeración libudev) |
| `eframe`/`egui` | IHM | versiones ligadas entre sí |
| `egui_plot` | curva | ⚠️ **versionado una menor por delante de `egui`**: para `egui` 0.33 → `egui_plot` **0.34** |
| `serde`/`toml` | persistencia | `mock_lib_control` expone una feature `serde` activada por el binario |

Las versiones compartidas están centralizadas en `[workspace.dependencies]` del
`Cargo.toml` raíz. Para subir `egui`/`eframe`, **comprobar la versión
correspondiente de `egui_plot`** (de lo contrario error «two versions of crate
egui»).

---

## 6. Ampliar el proyecto

### 6.1 Añadir un comando NAMUR

Todo ocurre en **`namur.rs`** (fuente de verdad del protocolo):

1. Añadir la rama en `handle_line` (lectura → `Reply`, escritura/acción →
   `Apply(Command)` o `SetWatchdog`).
2. Si es una **acción**, añadir la variante en `enum Command` (`stirrer.rs`) y su
   tratamiento en `Stirrer::apply`.
3. Actualizar el comentario de cabecera, **[commandes_namur.md](commandes_namur.md)**
   y la tabla de referencia del mini-terminal (`gui.rs`, tabla `rows`).
4. Añadir un test en el módulo `tests` de `namur.rs`.

### 6.2 Añadir un comando / un ajuste IHM

1. Variante en `enum Command` (`stirrer.rs`) + tratamiento en `Stirrer::apply`.
2. Campo en `StirrerSnapshot` si el valor debe ser observable.
3. Cableado IHM (`gui.rs`) vía un `cast` no bloqueante.
4. Si es persistente: campo en `AppConfig` (`config.rs`) + saneamiento en
   `sanitized` + traslado a `to_stirrer_config`.

### 6.3 Añadir una cadena de interfaz (i18n)

Toda cadena IHM **debe** pasar por una clave `Msg` (`i18n.rs`) con sus **8
traducciones** (tabla de tamaño fijo verificada en compilación). Los acrónimos
NAMUR, los sufijos de unidad y los nombres de comandos siguen codificados en duro.

### 6.4 Añadir un nuevo instrumento

1. Crear `mock_bin_<nom>/` y añadirlo a los `members` del `Cargo.toml` raíz.
2. Reutilizar `mock_lib_control`; factorizar todo lo común en una `mock_lib_*`
   (ej. promoción del modelo `motor.rs` si sirve a un segundo instrumento).
3. Seguir el mismo reparto: modelo síncrono, actor(es) ractor, capa de protocolo,
   IHM. Convención de nombre: `mock_bin_<type>_<protocole>`.

---

## 7. Estrategia de pruebas

- **Unitarias** (`mock_lib_control`): PID (proporcional, acotado, anti-windup).
- **Motor** (`motor.rs`): dinámica rotacional, convergencia en régimen permanente,
  efecto de la viscosidad sobre el par, saturación/sobrecarga.
- **Dominio** (`stirrer.rs`): convergencia de la velocidad hacia la consigna,
  deceleración al detenerse, detección de sobrecarga.
- **Protocolo** (`namur.rs`): decodificación de las lecturas (`IN_*`), de las
  escrituras (`OUT_SP_4`), de las acciones (`START/STOP/RESET`), del perro guardián
  y de los comandos desconocidos.
- **Config / red** (`config.rs`, `actors/network.rs`): round-trip TOML, filtro IP
  (comodines, IPv4-mapped), saneamiento sin pánico, apertura serie con error en
  puerto ausente.

Lanzar: `cargo test -p mock_bin_su_namur` (o `--workspace`). Las pruebas son
**deterministas y sin IHM**.

---

## 8. Resolución de problemas

| Síntoma | Pista |
|----------|-------|
| «two versions of crate `egui`» | Desacuerdo `egui_plot` / `egui`: alinear las versiones (§5). |
| La IHM no se abre | Pantalla ausente (headless) o libs del sistema faltantes (§1). |
| `NAMUR ✖` en el encabezado | Puerto TCP ya en uso / < 1024 sin privilegios, o puerto serie no disponible: cambiar en *Parámetros*. |
| Un cliente TCP es rechazado | IP fuera de la **lista blanca**: vaciar la lista o añadir un patrón (`192.168.1.*`). |
| La serie no se abre | Feature `serial` ausente, puerto erróneo, o permisos (`dialout`). |
| El motor se detiene solo | **Perro guardián** armado (`OUT_WD1@…`) sin tráfico: enviar tramas o `OUT_WD1@0`. |
| Sobrecarga permanente | Viscosidad demasiado elevada vs `torque_max`: ajustar los parámetros del motor. |
| Config no recargada | Directorio actual erróneo o `MOCK_CONFIG`; comprobar el registro al arrancar. |

Aumentar la verbosidad: `RUST_LOG=debug` (o `trace`).

---

## 9. Build de distribución

```bash
cargo build --release -p mock_bin_su_namur
# Binario autónomo:
target/release/osne
```

El perfil `release` activa `lto = "thin"` y `opt-level = 3` (ver `Cargo.toml`
raíz). Para distribuir: proporcionar el binario + un `mock_su_namur.toml` de
ejemplo. Licencia **MIT** (archivo `LICENSE`).

### Feature `gui` (build con / sin interfaz)

```bash
cargo build --release -p mock_bin_su_namur                       # avec IHM (poste de travail)
cargo build --release -p mock_bin_su_namur --no-default-features  # «headless»: NAMUR + simulación, sin IHM
```

El modo **headless** está destinado a despliegues sin pantalla y hace que la
**compilación cruzada ARM sea trivial** (ninguna dependencia gráfica que enlazar).

### Integración en el escritorio Linux (icono de la barra de tareas)

El icono OSNE (`pic/osne-icon.png`, motivo de agitador, generado por
[`pic/osne-logo.gen.py`](../../../pic/osne-logo.gen.py)) está **embebido** en el
binario (`branding.rs` → `window_icon`). Esto basta en **X11, Windows y macOS**.
Bajo **Wayland**, el compositor **ignora** el icono embebido: asocia la ventana a su
**`app_id`** («osne», definido en `main.rs` vía `with_app_id`) con un archivo
`osne.desktop` del mismo nombre, y muestra el `Icon=osne` resuelto en el tema de
iconos `hicolor`.

Para obtener el icono bajo Wayland, instalar la entrada de escritorio para el
usuario actual:

```bash
scripts/install-desktop.sh osne
```

El script copia:

| Origen | Destino |
|--------|---------|
| `pic/osne-icon.png` | `~/.local/share/icons/hicolor/256x256/apps/osne.png` |
| `packaging/osne.desktop` | `~/.local/share/applications/osne.desktop` |

y luego refresca las cachés. Tres nombres **deben permanecer alineados**: el
`app_id` (`main.rs`), el archivo `osne.desktop` (+ su `StartupWMClass`) y el icono
`osne.png` (= `Icon=osne`). El mismo script instala ORME sin argumento
(`scripts/install-desktop.sh`).

---

## 10. Build «prod» — compilación cruzada desde Linux

### Procedimiento único

Todo se produce **desde Linux** mediante
[`scripts/build-prod.sh`](../../../scripts/build-prod.sh), que construye **todos los
instrumentos del workspace** (ORME *y* OSNE):

| Salida | Objetivo | IHM | Método |
|--------|-------|-----|---------|
| `dist/osne-linux-x86_64` | `x86_64-unknown-linux-gnu` | ✅ | `cross` |
| `dist/osne-windows-x86_64.exe` | `x86_64-pc-windows-gnu` | ✅ | `cross` (mingw) |
| `dist/osne-rpi-arm64` | `aarch64-unknown-linux-gnu` (Pi 3/4/5, Pi OS 64b) | ✅ | `cross` |
| Imagen Docker headless `osne:headless` | multi-arch `amd64` + `arm64` | ❌ | `docker buildx` |
| `dist/osne_<ver>_amd64.deb` / `_arm64.deb` | paquete Debian/Ubuntu | ✅ | `dpkg-deb` |
| `dist/osne-setup-x86_64.exe` | instalador Windows | ✅ | NSIS (`makensis`) |

```bash
# Requisitos previos (una vez) — Docker debe estar en ejecución:
cargo install cross

# Producir todo (ejecutables ORME + OSNE + instaladores en dist/ + imágenes Docker amd64):
scripts/build-prod.sh

# Variante: imágenes Docker MULTI-ARCH enviadas a un registro:
IMAGE_PREFIX=ghcr.io/<cuenta> scripts/build-prod.sh

# Sin construir los instaladores:
INSTALLERS=0 scripts/build-prod.sh
```

### Por qué `cross` para TODOS los builds (incluido Linux x86_64)

`cross` proporciona imágenes Docker que contienen las toolchains de cada objetivo.
⚠️ **No mezclar `cargo` nativo y `cross` en el mismo `target/`.** Los **proc-macros**
compilados por uno son rechazados por el otro (`can't find crate for …_derive`). El
script pasa **siempre por `cross`**. (Si surge el error: `rm -rf target/release` y
relanzar.)

### IHM compilada de forma cruzada hacia ARM: por qué funciona

`eframe`/`egui` cargan OpenGL, X11/Wayland y xkbcommon **en tiempo de ejecución**
(`dlopen`): el binario solo enlaza en el build con la `libc`. No se necesita
ninguna lib gráfica ARM del lado de la compilación cruzada; prever un entorno de
escritorio en el objetivo.

### Imagen Docker headless

La imagen ([`docker/Dockerfile.headless`](../../../docker/Dockerfile.headless))
parte de `debian:bookworm-slim` y **copia** el binario headless de la arquitectura
deseada (ninguna compilación en la imagen → sin QEMU). El nombre del binario y el
puerto expuesto se pasan por `--build-arg` (`BIN=osne`, `PORT=4001`). Montar un
volumen en `/data` para proporcionar/persistir `mock_su_namur.toml`.

```bash
docker run --rm -p 4001:4001 -v "$PWD/conf:/data" osne:headless
```

### Instaladores (`.deb` Linux/RPi + setup Windows)

Al final de cada build, `build-prod.sh` llama a
[`scripts/make-installers.sh <bin>`](../../../scripts/make-installers.sh), que
transforma los ejecutables release de `dist/` en **instaladores**:

| Instalador | Origen | Contenido | Herramienta |
|------------|--------|-----------|-------------|
| `osne_<ver>_amd64.deb` | `dist/osne-linux-x86_64` | binario → `/usr/bin`, entrada de escritorio, icono hicolor | `dpkg-deb` |
| `osne_<ver>_arm64.deb` | `dist/osne-rpi-arm64` | ídem (Raspberry Pi OS de 64 bits) | `dpkg-deb` |
| `osne-setup-x86_64.exe` | `dist/osne-windows-x86_64.exe` | exe + accesos directos (menú Inicio/escritorio) + desinstalador | NSIS (`makensis`) |

- Los `.deb` colocan el icono y el `.desktop`; un `postinst` refresca las cachés
  (`update-desktop-database`, `gtk-update-icon-cache`). Dependencias: `libc6`;
  recomendaciones gráficas (`libgl1`, `libxkbcommon0`, `libwayland-client0`).
- El instalador Windows se genera a partir de
  [`packaging/windows/installer.nsi`](../../../packaging/windows/installer.nsi);
  los accesos directos usan un icono `.ico` multi-resolución derivado de
  `pic/osne-icon.png` (mediante Pillow).
- **Requisitos previos**: `dpkg-deb` (presente en Debian/Ubuntu) para los `.deb`,
  **`makensis`** (`sudo apt install nsis`) para el setup Windows, `python3`+Pillow
  para el `.ico`. Cada objetivo cuya herramienta o artefacto falte es **avisado y
  omitido** (el build no se rompe). Desactivar mediante `INSTALLERS=0`. También se
  pueden (re)generar solo los instaladores de un instrumento:
  `scripts/make-installers.sh osne`.
- La **versión** de los paquetes proviene de `[workspace.package].version` del
  `Cargo.toml` raíz.

### Notas

- Los binarios están **enlazados dinámicamente a glibc**; compilados vía `cross`
  (baseline glibc antigua) funcionan en distribuciones recientes.
- `dist/` está ignorado por git (artefactos de build).

---

## 11. Convenciones

- Código y comentarios en **francés**; logs y mensajes de error en **inglés**.
- `cargo clippy --workspace` **sin advertencias** antes de todo commit.
- Todo nuevo comportamiento de negocio, de motor o de protocolo se acompaña de un
  **test**.
- El juego de comandos NAMUR se modifica en **`namur.rs`** (fuente de verdad), con
  actualización conjunta de la documentación.
