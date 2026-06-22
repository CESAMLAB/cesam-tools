# Maintenance documentation — ORME (workspace `cesam-tools`)

*🌍 [FR](../fr/maintenance.md) · **EN** · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

> Audience: developers who maintain, fix or extend the project.
> See also: [conception.md](conception.md) · [table_modbus.md](table_modbus.md).

---

## 1. Prerequisites

- **Rust stable** (2021 edition, `rust-version` ≥ 1.85). Install: <https://rustup.rs>.
- **System dependencies (Linux) for the GUI** (`eframe`/`egui`, OpenGL/winit):
  - Debian/Ubuntu: `libxkbcommon-dev`, `libwayland-dev`, `libxcb1-dev`,
    `libgl1-mesa-dev` (or equivalents), plus a graphical server (X11/Wayland).
  - The GUI requires a **display**: in a headless environment, the window does
    not open (the Modbus server, however, does not depend on the display).
- Network access to the crates.io registry for the first compilation.

---

## 2. Common commands

```bash
cargo check --workspace          # Quick check (no codegen)
cargo build --workspace          # Debug build
cargo build --release            # Optimized build (thin LTO)
cargo test  --workspace          # Unit + integration tests
cargo clippy --workspace --all-targets   # Lint (must stay WARNING-FREE)
cargo run -p mock_bin_ru_modbustcp       # Run the controller

# Alternative configuration file:
MOCK_CONFIG=./ma_config.toml cargo run -p mock_bin_ru_modbustcp
# Verbose logging:
RUST_LOG=debug cargo run -p mock_bin_ru_modbustcp
```

Binary produced: `target/debug/orme` or `target/release/orme` (the Cargo package
remains `mock_bin_ru_modbustcp`, but the executable is named **`orme`** — see
`[[bin]]` in the crate's `Cargo.toml`).

### Cargo features

| Feature | Default | Effect |
|---------|:---------:|-------|
| `gui` | ✅ | `egui`/`eframe` GUI (otherwise headless binary) |
| `rtu` | ✅ | Serial Modbus RTU transport (RS485) via `tokio-serial` |

```bash
cargo build --no-default-features                 # headless, Modbus TCP only
cargo build --no-default-features --features rtu  # headless TCP + serial RTU
cargo build --no-default-features --features gui  # GUI, TCP only (no serial)
```

> ⚠️ **`rtu` = native dependency.** `tokio-serial` opens the port via termios
> (Linux); `libudev` enumeration is disabled (`default-features = false`). When
> **cross-compiling** (`build-prod.sh`, desktop exes with default features), the
> `cross` image of the target may still require the system serial headers; if the
> toolchain causes trouble, remove `rtu` from the affected build. The **headless
> Docker is not impacted** (it builds with `--no-default-features`).

---

## 3. Code organization

```
mock_lib_control/        Control library (pure, no IO, testable)
  src/pid.rs             Anti-windup PID
  src/onoff.rs           On/off with symmetric hysteresis + anti-short-cycle
  src/pwm.rs             Cycle relay (PWM / time-proportioning)
  src/process.rs         FOPDT transfer function
  src/lib.rs             ControllerKind + re-exports (optional `serde` feature)

mock_bin_ru_modbustcp/   Controller binary
  src/main.rs            Startup: config, Tokio runtime, actors, GUI
  src/regulator.rs       Synchronous business model (state, Command, step)
  src/config.rs          AppConfig (TOML), Transport/SerialConfig, IpFilter, ServerStatus
  src/map.rs             Modbus addressing plan (SOURCE OF TRUTH)
  src/modbus_server.rs   RegulatorService (Service trait) + TCP single-master + serve_rtu
  src/gui.rs             egui GUI (single page + Settings modal)
  src/actors/
    simulation.rs        Control loop (tick)
    network.rs           Modbus TCP/RTU server, reconfigurable at runtime

docs/                    Design, Modbus table, maintenance
```

**Golden rule**: the business logic (`mock_lib_control`, `regulator.rs`) stays
**synchronous and tested**; the asynchronous part is confined to the actors and
the IO.

---

## 4. Configuration

- File: `mock_ru_modbustcp.toml` in the current directory, or the path provided by
  the `MOCK_CONFIG` environment variable.
- Loaded at startup; **default values** if absent or unreadable (a warning is
  logged, the application starts anyway).
- Saved from the GUI (*Apply* / *Save settings* / *Reset to defaults* buttons).

Structure (all sections are optional, completed by defaults):

```toml
language = "en"
check_updates = true       # check at startup whether a newer release exists (GUI)

[network]
bind_ip = "0.0.0.0"
port = 5502
allowlist = ["192.168.1.*", "127.0.0.1"]   # empty = all IPs allowed

[process]   # transfer function G(s) = K·e^(-L·s)/(1+T·s)
gain = 1.6        # K (unit/%)
tau = 30.0        # T (s)
dead_time = 2.0   # L (s)
ambient = 20.0

[regulation]
sp_min = 0.0
sp_max = 250.0
hysteresis = 2.0
[regulation.pid_heat]   # direction 1 (heating)
kp = 4.0 ; ki = 0.25 ; kd = 1.0 ; out_min = 0.0 ; out_max = 100.0
[regulation.pid_cool]   # direction 2 (cooling)
kp = 4.0 ; ki = 0.25 ; kd = 1.0 ; out_min = 0.0 ; out_max = 100.0
```

> The **default values** have a **single source**: `RegulatorConfig::default` in
> `regulator.rs`. `ProcessConfig`/`RegulationConfig` (config.rs) derive from it.
> To change a default, modify `RegulatorConfig::default` only.

### Update check

If `check_updates = true` (default) **and** the binary is compiled with the `gui`
feature, the GUI queries **at startup** the latest release published on GitHub
(`CESAMLAB/cesam-tools`) and compares its number with the current version. A newer
version displays a clickable "🔔 Update available" banner. The *Check now* button
(*Settings* modal) re-runs the check.

- The HTTPS request runs in a **dedicated thread**, bounded by a timeout (5 s):
  being offline or having GitHub unreachable never hinders startup.
- The logic lives in the shared crate **`mock_lib_update`** (`ureq`/`rustls`,
  embedded Mozilla roots → clean cross-compilation under `cross`).
- **Headless build** (`--no-default-features`): the check — and the whole
  network/TLS dependency — is **absent**. On a server, manage updates via
  apt/Docker. Disableable by the operator (modal checkbox).

---

## 5. Dependencies and version pitfalls

| Crate | Role | Point of attention |
|-------|------|-------------------|
| `tokio` | async runtime | features: `rt-multi-thread, macros, net, time, sync` |
| `ractor` | actors | default features (native async, **not** `async-trait`) |
| `tokio-serial` | serial Modbus RTU | optional (`rtu` feature), `default-features = false` (no libudev enumeration) |
| `tokio-modbus` | Modbus TCP | `default-features = false`, feature **`tcp-server`** |
| `eframe`/`egui` | GUI | versions tied to each other |
| `egui_plot` | curve | ⚠️ **versioned one minor ahead of `egui`**: for `egui` 0.33 → `egui_plot` **0.34** |
| `serde`/`toml` | persistence | `mock_lib_control` exposes a `serde` feature enabled by the binary |
| `mock_lib_update` (`ureq`/`rustls`) | update check | **`gui` feature only**; rustls 0.23 (webpki up to date); absent in headless |

The shared versions are centralized in `[workspace.dependencies]` of the root
`Cargo.toml`. To bump `egui`/`eframe`, **check the corresponding `egui_plot`
version** (otherwise "two versions of crate egui" error).

---

## 6. Extending the project

### 6.1 Add a Modbus point

Everything happens in **`map.rs`** (then the snapshot/Command if needed):

1. Declare the address constant and adjust the `*_COUNT` of the relevant table.
2. Fill in the value in `MemoryMap::refresh_from` (state → register).
3. If the point is writable, decode it in `coil_to_command` /
   `holdings_to_commands` (register → `Command`).
4. Update the header doc-comment **and** [table_modbus.md](table_modbus.md).
5. Add the row in the GUI's live table (`gui.rs::modbus_rows`).

### 6.2 Add a command / a setting

1. Variant in `enum Command` (`regulator.rs`) + handling in `Regulator::apply`.
2. Field in `RegulatorSnapshot` if the value must be observable.
3. GUI wiring (`gui.rs`) and/or Modbus decoding (`map.rs`).
4. If persistent: field in `AppConfig` (`config.rs`) + `to_regulator_config`.

### 6.3 Add a new instrument

1. Create `mock_bin_<name>/` and add it to the `members` of the root `Cargo.toml`.
2. Reuse `mock_lib_control`; factor anything common into a `mock_lib_*`.
3. Follow the same split: synchronous model, ractor actor(s), protocol layer, GUI.
   Naming convention: `mock_bin_<type>_<protocol>`.

---

## 7. Test strategy

- **Unit** (`mock_lib_control`): PID (proportional, clamping, anti-windup), TOR
  (dead band), process (steady-state convergence).
- **Domain** (`regulator.rs`): PID convergence in auto, output in manual, return
  to ambient when stopped.
- **Mapping** (`map.rs`): `f32`↔registers round-trip, write decoding, rejection of
  a partial `f32` write.
- **Config / network** (`config.rs`, `actors/network.rs`): TOML round-trip, IP
  filter (wildcards), effective server startup (bind on an ephemeral port).

Run: `cargo test --workspace`. The tests are **deterministic and GUI-free**.

---

## 8. Troubleshooting

| Symptom | Lead |
|----------|-------|
| "two versions of crate `egui`" | `egui_plot` / `egui` mismatch: align the versions (§5). |
| The GUI does not open | Display absent (headless) or missing system libs (§1). |
| `Modbus ✖ listening failed` in the header | Port already in use or < 1024 without privileges: change the port in *Settings*. |
| A client is refused | IP outside the **allowlist**: empty the list or add a pattern (`192.168.1.*`). |
| Aberrant `f32` values on the client side | Word order (high word first): see [table_modbus.md](table_modbus.md). |
| A `f32` setpoint write is ignored | Write **both** registers of the pair in one request. |
| Config not reloaded | Wrong current directory or `MOCK_CONFIG`; check the startup log. |
| No icon in the taskbar (Linux) | **Wayland** session: the embedded icon is ignored. Install the desktop entry: `scripts/install-desktop.sh` (§9, *Desktop integration*). |

Increase verbosity: `RUST_LOG=debug` (or `trace`).

---

## 9. Distribution build

```bash
cargo build --release
# Standalone binary:
target/release/orme
```

The `release` profile enables `lto = "thin"` and `opt-level = 3` (see the root
`Cargo.toml`). To distribute: provide the binary + an example
`mock_ru_modbustcp.toml`. License **MIT** (`LICENSE` file).

### `gui` feature (build with / without interface)

The GUI is behind the **`gui`** Cargo feature, enabled by default:

```bash
cargo build --release                       # with GUI (workstation)
cargo build --release --no-default-features  # "headless": Modbus + simulation, no GUI
```

The **headless** mode is intended for displayless deployments (Raspberry Pi in
service) and makes **ARM cross-compilation trivial** (no graphics dependency to
link).

### Linux desktop integration (taskbar icon)

The ORME icon is embedded in the binary (`branding.rs` → `with_icon`). This is
enough on **X11, Windows and macOS**. But on **Wayland**, the compositor
**ignores** the embedded icon: it associates the window through its **`app_id`**
("orme", set in `main.rs` via `ViewportBuilder::with_app_id`) with an
`orme.desktop` file of the same name, and displays the `Icon=` of that file
(resolved in the `hicolor` icon theme).

To get the icon under Wayland, install the desktop entry for the current user:

```bash
scripts/install-desktop.sh
```

The script copies:

| Source | Destination |
|--------|-------------|
| `pic/orme-icon.png` | `~/.local/share/icons/hicolor/256x256/apps/orme.png` |
| `packaging/orme.desktop` | `~/.local/share/applications/orme.desktop` |

then refreshes the caches (`gtk-update-icon-cache`, `update-desktop-database`).
The icon appears the next time ORME is launched (and reliably after a relogin of
the Wayland session).

> ⚠️ Three names **must stay aligned** for the association to work: the `app_id`
> (`main.rs`), the `orme.desktop` file name and its `StartupWMClass`, and the
> `orme.png` icon name (= `Icon=orme`). `packaging/orme.desktop` assumes an `orme`
> executable in the `PATH` (`Exec=` field); in dev (`cargo run`) this field has no
> bearing on the icon display, the association being done through
> `app_id`/`StartupWMClass`.

---

## 10. "Prod" build — cross-compilation from Linux

### Single procedure

Everything is produced **from Linux** by
[`scripts/build-prod.sh`](../../../scripts/build-prod.sh), which builds **every
workspace instrument** (ORME *and* OSNE) in one pass. For each instrument
(`<bin>` = `orme`, `osne`):

| Output | Target | GUI | Method |
|--------|-------|-----|---------|
| `dist/<bin>-linux-x86_64` | `x86_64-unknown-linux-gnu` | ✅ | `cross` |
| `dist/<bin>-windows-x86_64.exe` | `x86_64-pc-windows-gnu` | ✅ | `cross` (mingw) |
| `dist/<bin>-rpi-arm64` | `aarch64-unknown-linux-gnu` (Pi 3/4/5, Pi OS 64b) | ✅ | `cross` |
| Headless Docker image `<bin>:headless` | multi-arch `linux/amd64` + `linux/arm64` | ❌ | `docker buildx` |
| `dist/<bin>_<ver>_amd64.deb` / `_arm64.deb` | Debian/Ubuntu package | ✅ | `dpkg-deb` |
| `dist/<bin>-setup-x86_64.exe` | Windows installer | ✅ | NSIS (`makensis`) |

```bash
# Prerequisites (once) — Docker must be running:
cargo install cross

# Produce everything (ORME + OSNE exes in dist/ + local amd64 Docker images loaded):
scripts/build-prod.sh

# Variant: MULTI-ARCH Docker images pushed to a registry (<prefix>/<bin>:latest):
IMAGE_PREFIX=ghcr.io/<account> scripts/build-prod.sh

# Build only one instrument:
ONLY=orme scripts/build-prod.sh
```

### Why `cross` for ALL builds (including Linux x86_64)

`cross` provides Docker images containing the toolchains of each target: no
`mingw-w64`, no ARM toolchain, no *sysroot* to install.

⚠️ **Do not mix native `cargo` and `cross` in the same `target/`.** Both use
different versions of `rustc` (host vs container); the **proc-macros** compiled by
one are rejected by the other, hence `can't find crate for …_derive` errors (e.g.
`zerofrom_derive`, `tracing_attributes`). The script therefore **always goes
through `cross`**, even for Linux x86_64 — a single toolchain, reproducible builds.
(If the error occurs anyway after a previous native build: `rm -rf target/release`
then re-run.)

### GUI cross-compiled to ARM: why it works

`eframe`/`egui` load OpenGL, X11/Wayland and xkbcommon **at runtime** (`dlopen`):
the binary only links `libc` at build time. No ARM graphics lib is therefore
needed on the cross side. On the Raspberry Pi, provide a desktop environment
(mesa/X11 or Wayland) — present on Raspberry Pi OS *Desktop*.

> For a **32-bit Raspbian**, target `armv7-unknown-linux-gnueabihf` (adapt the
> targets in the script).

### Headless Docker image, "anywhere"

The image ([`docker/Dockerfile.headless`](../../../docker/Dockerfile.headless))
starts from `debian:bookworm-slim` and **copies** the headless binary of the
desired architecture (no compilation in the image → no QEMU). `docker buildx`
assembles the multi-arch `amd64`+`arm64`. The server listens on `5502`. Mount a
volume on `/data` to provide/persist `mock_ru_modbustcp.toml`.

```bash
# Without a registry: local amd64 image loaded, immediately testable
docker run --rm -p 5502:5502 -v "$PWD/conf:/data" orme:headless
```

### Installers (`.deb` Linux/RPi + Windows setup)

At the end of the build, `build-prod.sh` calls
[`scripts/make-installers.sh <bin>`](../../../scripts/make-installers.sh), which
turns the release executables in `dist/` into **installers**:

| Installer | Source | Contents | Tool |
|-----------|--------|----------|------|
| `<bin>_<ver>_amd64.deb` | `dist/<bin>-linux-x86_64` | binary → `/usr/bin`, desktop entry, hicolor icon | `dpkg-deb` |
| `<bin>_<ver>_arm64.deb` | `dist/<bin>-rpi-arm64` | same (Raspberry Pi OS 64-bit) | `dpkg-deb` |
| `<bin>-setup-x86_64.exe` | `dist/<bin>-windows-x86_64.exe` | exe + shortcuts (Start menu/desktop) + uninstaller | NSIS (`makensis`) |

- The `.deb` packages install the icon and the `.desktop` entry; a `postinst`
  refreshes the icon caches and the `.desktop` database. Dependencies: `libc6`;
  graphics recommendations (`libgl1`, `libxkbcommon0`, `libwayland-client0`).
- The Windows installer comes from
  [`packaging/windows/installer.nsi`](../../../packaging/windows/installer.nsi);
  its shortcuts carry a multi-resolution `.ico` derived from `pic/<bin>-icon.png`
  (via Pillow).
- **Prerequisites**: `dpkg-deb` (Debian/Ubuntu) for the `.deb` packages,
  **`makensis`** (`sudo apt install nsis`) for the Windows setup, `python3`+Pillow
  for the `.ico`. Any target whose tool/artifact is missing is **warned about and
  skipped** (the build does not break). Disable via `INSTALLERS=0`, or
  (re)generate the installers of a single instrument only:
  `scripts/make-installers.sh orme`.

### Native Windows build (MSVC) — optional

The `.exe` produced above is **GNU/mingw** (native Windows executable, GUI
included). If an **MSVC** binary is required, build on a Windows machine with
[`scripts/build-windows.ps1`](../../../scripts/build-windows.ps1) (prerequisites:
Rust + *Visual Studio Build Tools*, "Desktop development with C++" workload), or
from Linux via `cargo-xwin` (`cargo xwin build --release --target x86_64-pc-windows-msvc`).

### Notes

- The binaries are **dynamically linked to glibc**; compiled via `cross` (old
  glibc baseline) they run on recent distributions (and in `debian:bookworm-slim`).
  For a fully static binary, target `*-musl`.
- `dist/` is ignored by git (build artifacts).

---

## 11. Conventions

- Code and comments in **French**.
- `cargo clippy --workspace` **warning-free** before any commit.
- Any new business or mapping behavior comes with a **test**.
- The addressing plan is modified in **`map.rs`** (source of truth), with a joint
  documentation update.
