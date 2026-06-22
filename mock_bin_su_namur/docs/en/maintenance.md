# Maintenance documentation — OSNE (workspace `cesam-tools`)

*🌍 [FR](../fr/maintenance.md) · **EN** · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

> Audience: developers who maintain, fix or extend the project.
> See also: [conception.md](conception.md) · [commandes_namur.md](commandes_namur.md).

---

## 1. Prerequisites

- **Rust stable** (2021 edition, `rust-version` ≥ 1.85). Install: <https://rustup.rs>.
- **System dependencies (Linux) for the GUI** (`eframe`/`egui`, OpenGL/winit):
  - Debian/Ubuntu: `libxkbcommon-dev`, `libwayland-dev`, `libxcb1-dev`,
    `libgl1-mesa-dev` (or equivalents), plus a graphical server (X11/Wayland).
  - The GUI requires a **display**: in a headless environment, the window does
    not open (the NAMUR server, however, does not depend on the display).
- **Serial link** (feature `serial`): access to the port (`/dev/ttyUSB*`,
  `dialout` group on Linux). Without hardware, use the **TCP** transport.
- Network access to the crates.io registry for the first compilation.

---

## 2. Common commands

```bash
cargo check -p mock_bin_su_namur          # Quick check (no codegen)
cargo build -p mock_bin_su_namur          # Debug build
cargo build --release -p mock_bin_su_namur   # Optimized build (thin LTO)
cargo test  -p mock_bin_su_namur          # Unit + integration tests
cargo clippy --workspace --all-targets    # Lint (must stay WARNING-FREE)
cargo run   -p mock_bin_su_namur          # Run the stirrer (GUI + NAMUR/TCP)

# Alternative configuration file:
MOCK_CONFIG=./ma_config.toml cargo run -p mock_bin_su_namur
# Verbose logging:
RUST_LOG=debug cargo run -p mock_bin_su_namur
```

Binary produced: `target/debug/osne` or `target/release/osne` (the Cargo package
remains `mock_bin_su_namur`, but the executable is named **`osne`** — see `[[bin]]`
in the crate's `Cargo.toml`).

### Cargo features

| Feature | Default | Effect |
|---------|:-------:|--------|
| `gui` | ✅ | `egui`/`eframe` GUI (otherwise headless binary) |
| `serial` | ✅ | NAMUR transport over RS-232 serial link via `tokio-serial` |

```bash
cargo build -p mock_bin_su_namur --no-default-features                  # headless, NAMUR/TCP only
cargo build -p mock_bin_su_namur --no-default-features --features serial # headless TCP + serial
cargo build -p mock_bin_su_namur --no-default-features --features gui    # GUI, TCP only (no serial)
```

> ⚠️ **`serial` = native dependency.** `tokio-serial` opens the port via termios
> (Linux); `libudev` enumeration is disabled (`default-features = false`). In
> **cross-compilation** (`build-prod.sh`, desktop exes with default features), the
> target's `cross` image may still require the serial headers; if the toolchain
> causes trouble, drop `serial` from the build concerned. The **headless Docker is
> not affected** (it builds with `--no-default-features`).

---

## 3. Code organization

```
mock_lib_control/        Control library (pure, no IO, testable)
  src/pid.rs             Anti-windup PID (reused for speed feedback control)
  src/lib.rs             re-exports (optional `serde` feature)

mock_bin_su_namur/       Stirrer binary (executable `osne`)
  src/main.rs            Startup: config, Tokio runtime, actors, GUI
  src/motor.rs           Motor physical model (rotational dynamics, Euler)
  src/stirrer.rs         Synchronous business model (state, Command, step) — owns the PID
  src/config.rs          AppConfig (TOML), Transport/SerialConfig, IpFilter, ServerStatus
  src/namur.rs           NAMUR protocol: handle_line (SOURCE OF TRUTH for the command set)
  src/namur_server.rs    NAMUR service (ASCII lines) + TCP single-master + serial serve + watchdog
  src/trace.rs           Circular frame log (GUI mini-terminal)
  src/gui.rs             egui GUI (single page + mini-terminal + Settings modal)
  src/branding.rs        Embedded logos (feature `gui`)
  src/i18n.rs            Typed i18n catalog (8 languages), dependency-free
  src/actors/
    simulation.rs        Simulation loop (20 ms tick)
    network.rs           NAMUR TCP/serial server, hot-(re)configurable

docs/                    Design, NAMUR commands, manual, maintenance (multilingual)
```

**Golden rule**: the business logic (`mock_lib_control`, `motor.rs`, `stirrer.rs`)
stays **synchronous and tested**; async is confined to the actors and IO. Exact
copy of the **ORME** controller (`mock_bin_ru_modbustcp`) — same invariants.

---

## 4. Configuration

- File: `mock_su_namur.toml` in the current directory, or path provided by the
  `MOCK_CONFIG` environment variable.
- Loaded at startup; **default values** if missing or unreadable (a warning is
  logged, the application starts anyway).
- **Every value from the TOML is sanitized** (`AppConfig::sanitized`): reordered
  bounds (`min ≤ max`), floats forced finite, inertia/torque/viscosity strictly
  positive. **Invariant: never `f32::clamp` with unvalidated bounds** (panics if
  `min > max` or `NaN`).
- Saved from the GUI (*Apply* / *Save* / *Reset* buttons).

Structure (all sections are optional, filled in with defaults):

```toml
language = "fr"

[network]
transport = "tcp"          # "tcp" or "serial"
bind_ip = "0.0.0.0"
port = 4001
allowlist = ["192.168.1.*", "127.0.0.1"]   # empty = all IPs allowed
[network.serial]
port = "/dev/ttyUSB0"
baud = 9600 ; parity = "even" ; data_bits = 7 ; stop_bits = 1   # NAMUR 7E1

[motor]   # J·dω/dt = T − k·η·ω − friction
inertia = 0.02      # J (responsiveness)
load_coeff = 0.05   # k (weight of viscosity)
friction = 2.0      # N·cm
torque_max = 100.0  # N·cm (ceiling of the PID output)

[regulation]
speed_min = 0.0 ; speed_max = 2000.0
viscosity = 1.0 ; viscosity_min = 0.1 ; viscosity_max = 20.0
[regulation.pid]
kp = ... ; ki = ... ; kd = ... ; out_min = 0.0 ; out_max = 100.0
```

> The **default values** have a **single source**: `StirrerConfig::default` in
> `stirrer.rs`. `MotorConfig`/`RegulationConfig` (config.rs) derive from it. The
> PID output bounds (`out_min`/`out_max`) are **forced** to `[0, torque_max]` when
> building the stirrer (`to_stirrer_config`).

---

## 5. Dependencies and version pitfalls

| Crate | Role | Point of attention |
|-------|------|--------------------|
| `tokio` | async runtime | shared features + **`io-util`** (BufReader / NAMUR ASCII lines) |
| `ractor` | actors | default features (native async, **not** `async-trait`) |
| `tokio-serial` | serial NAMUR | optional (feature `serial`), `default-features = false` (no libudev enumeration) |
| `eframe`/`egui` | GUI | versions tied together |
| `egui_plot` | chart | ⚠️ **versioned one minor ahead of `egui`**: for `egui` 0.33 → `egui_plot` **0.34** |
| `serde`/`toml` | persistence | `mock_lib_control` exposes a `serde` feature enabled by the binary |

Shared versions are centralized in `[workspace.dependencies]` of the root
`Cargo.toml`. To bump `egui`/`eframe`, **check the matching `egui_plot` version**
(otherwise "two versions of crate egui" error).

---

## 6. Extending the project

### 6.1 Add a NAMUR command

Everything happens in **`namur.rs`** (protocol source of truth):

1. Add the branch in `handle_line` (read → `Reply`, write/action →
   `Apply(Command)` or `SetWatchdog`).
2. If it is an **action**, add the variant in `enum Command` (`stirrer.rs`) and its
   handling in `Stirrer::apply`.
3. Update the header doc-comment, **[commandes_namur.md](commandes_namur.md)** and
   the mini-terminal reference table (`gui.rs`, `rows` table).
4. Add a test in the `tests` module of `namur.rs`.

### 6.2 Add a GUI command / setting

1. Variant in `enum Command` (`stirrer.rs`) + handling in `Stirrer::apply`.
2. Field in `StirrerSnapshot` if the value must be observable.
3. GUI wiring (`gui.rs`) via a non-blocking `cast`.
4. If persistent: field in `AppConfig` (`config.rs`) + sanitization in `sanitized`
   + carry-over in `to_stirrer_config`.

### 6.3 Add an interface string (i18n)

Every GUI string **must** go through a `Msg` key (`i18n.rs`) with its **8
translations** (fixed-size array checked at compile time). NAMUR acronyms, unit
suffixes and command names stay hard-coded.

### 6.4 Add a new instrument

1. Create `mock_bin_<name>/` and add it to the `members` of the root `Cargo.toml`.
2. Reuse `mock_lib_control`; factor out anything common into a `mock_lib_*` (e.g.
   promote the `motor.rs` model if it serves a second instrument).
3. Follow the same split: synchronous model, ractor actor(s), protocol layer, GUI.
   Naming convention: `mock_bin_<type>_<protocol>`.

---

## 7. Test strategy

- **Unit** (`mock_lib_control`): PID (proportional, bounding, anti-windup).
- **Motor** (`motor.rs`): rotational dynamics, steady-state convergence, effect of
  viscosity on torque, saturation/overload.
- **Domain** (`stirrer.rs`): convergence of the speed toward the setpoint,
  deceleration on stop, overload detection.
- **Protocol** (`namur.rs`): decoding of reads (`IN_*`), writes (`OUT_SP_4`),
  actions (`START/STOP/RESET`), watchdog and unknown commands.
- **Config / network** (`config.rs`, `actors/network.rs`): TOML round-trip, IP
  filter (wildcards, IPv4-mapped), sanitization without panic, serial open erroring
  on a missing port.

Run: `cargo test -p mock_bin_su_namur` (or `--workspace`). The tests are
**deterministic and GUI-free**.

---

## 8. Troubleshooting

| Symptom | Lead |
|---------|------|
| "two versions of crate `egui`" | `egui_plot` / `egui` mismatch: align the versions (§5). |
| The GUI does not open | Display absent (headless) or missing system libs (§1). |
| `NAMUR ✖` in the header | TCP port already in use / < 1024 without privileges, or serial port unavailable: change it in *Settings*. |
| A TCP client is refused | IP outside the **allowlist**: empty the list or add a pattern (`192.168.1.*`). |
| The serial does not open | `serial` feature absent, wrong port, or permissions (`dialout`). |
| The motor stops on its own | **Watchdog** armed (`OUT_WD1@…`) without traffic: send frames or `OUT_WD1@0`. |
| Permanent overload | Viscosity too high vs `torque_max`: adjust the motor parameters. |
| Config not reloaded | Wrong current directory or `MOCK_CONFIG`; check the startup log. |

Increase verbosity: `RUST_LOG=debug` (or `trace`).

---

## 9. Distribution build

```bash
cargo build --release -p mock_bin_su_namur
# Standalone binary:
target/release/osne
```

The `release` profile enables `lto = "thin"` and `opt-level = 3` (see the root
`Cargo.toml`). To distribute: provide the binary + a sample `mock_su_namur.toml`.
**MIT** License (`LICENSE` file).

### `gui` feature (build with / without interface)

```bash
cargo build --release -p mock_bin_su_namur                       # with GUI (workstation)
cargo build --release -p mock_bin_su_namur --no-default-features  # "headless": NAMUR + simulation, no GUI
```

The **headless** mode is intended for screenless deployments and makes **ARM
cross-compilation trivial** (no graphics dependency to link).

### Linux desktop integration (taskbar icon)

The OSNE icon (`pic/osne-icon.png`, stirrer motif, generated by
[`pic/osne-logo.gen.py`](../../../pic/osne-logo.gen.py)) is **embedded** in the
binary (`branding.rs` → `window_icon`). This is enough under **X11, Windows and
macOS**. Under **Wayland**, the compositor **ignores** the embedded icon: it
matches the window to its **`app_id`** ("osne", set in `main.rs` via
`with_app_id`) against an `osne.desktop` file of the same name, and shows the
`Icon=osne` resolved from the `hicolor` icon theme.

To get the icon under Wayland, install the desktop entry for the current user:

```bash
scripts/install-desktop.sh osne
```

The script copies:

| Source | Destination |
|--------|-------------|
| `pic/osne-icon.png` | `~/.local/share/icons/hicolor/256x256/apps/osne.png` |
| `packaging/osne.desktop` | `~/.local/share/applications/osne.desktop` |

then refreshes the caches. Three names **must stay aligned**: the `app_id`
(`main.rs`), the `osne.desktop` file (+ its `StartupWMClass`) and the `osne.png`
icon (= `Icon=osne`). The same script installs ORME with no argument
(`scripts/install-desktop.sh`).

---

## 10. "Prod" build — cross-compilation from Linux

### Single procedure

Everything is produced **from Linux** by
[`scripts/build-prod.sh`](../../../scripts/build-prod.sh), which builds **all
workspace instruments** (ORME *and* OSNE):

| Output | Target | GUI | Method |
|--------|--------|-----|--------|
| `dist/osne-linux-x86_64` | `x86_64-unknown-linux-gnu` | ✅ | `cross` |
| `dist/osne-windows-x86_64.exe` | `x86_64-pc-windows-gnu` | ✅ | `cross` (mingw) |
| `dist/osne-rpi-arm64` | `aarch64-unknown-linux-gnu` (Pi 3/4/5, Pi OS 64b) | ✅ | `cross` |
| Headless Docker image `osne:headless` | multi-arch `amd64` + `arm64` | ❌ | `docker buildx` |
| `dist/osne_<ver>_amd64.deb` / `_arm64.deb` | Debian/Ubuntu package | ✅ | `dpkg-deb` |
| `dist/osne-setup-x86_64.exe` | Windows installer | ✅ | NSIS (`makensis`) |

```bash
# Prerequisites (once) — Docker must be running:
cargo install cross

# Produce everything (ORME + OSNE exes + installers in dist/ + amd64 Docker images):
scripts/build-prod.sh

# Variant: MULTI-ARCH Docker images pushed to a registry:
IMAGE_PREFIX=ghcr.io/<account> scripts/build-prod.sh

# Without building the installers:
INSTALLERS=0 scripts/build-prod.sh
```

### Why `cross` for ALL builds (including Linux x86_64)

`cross` provides Docker images containing the toolchains of each target.
⚠️ **Do not mix native `cargo` and `cross` in the same `target/`.** The
**proc-macros** compiled by one are rejected by the other (`can't find crate for
…_derive`). The script **always goes through `cross`**. (If the error occurs:
`rm -rf target/release` then retry.)

### GUI cross-compiled to ARM: why it works

`eframe`/`egui` load OpenGL, X11/Wayland and xkbcommon **at runtime** (`dlopen`):
the binary only links `libc` at build time. No ARM graphics lib is needed on the
cross side; provide a desktop environment on the target.

### Headless Docker image

The image ([`docker/Dockerfile.headless`](../../../docker/Dockerfile.headless))
starts from `debian:bookworm-slim` and **copies** the headless binary of the
desired architecture (no compilation in the image → no QEMU). The binary name and
the exposed port are passed via `--build-arg` (`BIN=osne`, `PORT=4001`). Mount a
volume on `/data` to provide/persist `mock_su_namur.toml`.

```bash
docker run --rm -p 4001:4001 -v "$PWD/conf:/data" osne:headless
```

### Installers (`.deb` Linux/RPi + Windows setup)

At the end of each build, `build-prod.sh` calls
[`scripts/make-installers.sh <bin>`](../../../scripts/make-installers.sh), which
turns the release executables in `dist/` into **installers**:

| Installer | Source | Contents | Tool |
|-----------|--------|----------|------|
| `osne_<ver>_amd64.deb` | `dist/osne-linux-x86_64` | binary → `/usr/bin`, desktop entry, hicolor icon | `dpkg-deb` |
| `osne_<ver>_arm64.deb` | `dist/osne-rpi-arm64` | same (Raspberry Pi OS 64-bit) | `dpkg-deb` |
| `osne-setup-x86_64.exe` | `dist/osne-windows-x86_64.exe` | exe + shortcuts (Start menu/desktop) + uninstaller | NSIS (`makensis`) |

- The `.deb` packages install the icon and the `.desktop` entry; a `postinst`
  refreshes the caches (`update-desktop-database`, `gtk-update-icon-cache`).
  Dependencies: `libc6`; graphics recommendations (`libgl1`, `libxkbcommon0`,
  `libwayland-client0`).
- The Windows installer is generated from
  [`packaging/windows/installer.nsi`](../../../packaging/windows/installer.nsi);
  the shortcuts use a multi-resolution `.ico` derived from `pic/osne-icon.png`
  (via Pillow).
- **Prerequisites**: `dpkg-deb` (present on Debian/Ubuntu) for the `.deb` packages,
  **`makensis`** (`sudo apt install nsis`) for the Windows setup, `python3`+Pillow
  for the `.ico`. Any target whose tool or artifact is missing is **warned about
  and skipped** (the build does not break). Disable via `INSTALLERS=0`. You can
  also (re)generate the installers of a single instrument only:
  `scripts/make-installers.sh osne`.
- The package **version** comes from `[workspace.package].version` of the root
  `Cargo.toml`.

### Notes

- The binaries are **dynamically linked to glibc**; compiled via `cross` (old
  glibc baseline) they run on recent distributions.
- `dist/` is ignored by git (build artifacts).

---

## 11. Conventions

- Code and comments in **French**; logs and error messages in **English**.
- `cargo clippy --workspace` **warning-free** before any commit.
- Any new business, motor or protocol behaviour comes with a **test**.
- The NAMUR command set is modified in **`namur.rs`** (source of truth), with joint
  update of the documentation.
