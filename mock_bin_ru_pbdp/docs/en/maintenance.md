# Maintenance Documentation — ORPD / PROFIBUS DP (workspace `cesam-tools`)

*🌍 [FR](../fr/maintenance.md) · **EN** · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

> Crate: `mock_bin_ru_pbdp` · Executable: **ru_pbdp** · Brand: **ORPD**
> Audience: developers who maintain, fix, or extend the project.
> See also: [conception.md](conception.md) · [reference_profibus.md](reference_profibus.md).

---

## 1. Prerequisites

- **Rust stable** (2021 edition, `rust-version` ≥ 1.85). Install via
  <https://rustup.rs>.
- **System dependencies (Linux) for the GUI** (`eframe`/`egui`, OpenGL/winit):
  `libxkbcommon-dev`, `libwayland-dev`, `libxcb1-dev`, `libgl1-mesa-dev` (or
  equivalents), plus a graphical server (X11/Wayland). The GUI needs a
  **display**: in a headless environment, the window doesn't open.
- **Serial link** (port access, `/dev/ttyUSB*`, `dialout` group on Linux):
  unlike ORME/OSNE, **this is not an optional feature** here — `tokio-serial`
  is a direct dependency (see §5), the serial link being this instrument's
  only transport (there is no standard "PROFIBUS over TCP" equivalent).
  Without hardware, the GUI still starts (the opening error is shown in the
  header, the simulation keeps running) — see
  [`manuel_utilisateur.md`](manuel_utilisateur.md) §2.
- Network access to the crates.io registry for the first build.

---

## 2. Common commands

```bash
cargo check -p mock_bin_ru_pbdp          # Quick check (no codegen)
cargo build -p mock_bin_ru_pbdp          # Debug build
cargo build --release -p mock_bin_ru_pbdp   # Optimized build (thin LTO)
cargo test  -p mock_bin_ru_pbdp          # Unit + integration tests
cargo clippy --workspace --all-targets    # Lint (must stay WARNING-FREE)
cargo run   -p mock_bin_ru_pbdp          # Launch the GUI + PROFIBUS DP serial link

# Alternative configuration file:
MOCK_CONFIG=./my_config.toml cargo run -p mock_bin_ru_pbdp
# Verbose logging:
RUST_LOG=debug cargo run -p mock_bin_ru_pbdp
```

Produced binary: `target/debug/ru_pbdp` or `target/release/ru_pbdp` (the
Cargo package stays `mock_bin_ru_pbdp`; the executable and the "ORPD"
commercial name are documentation-only, see `[[bin]]` in the crate's
`Cargo.toml`).

### Cargo features

| Feature | Default | Effect |
|---------|:---------:|-------|
| `gui` | ✅ | `egui`/`eframe` GUI + update check (otherwise a headless binary) |

```bash
cargo build -p mock_bin_ru_pbdp --no-default-features   # headless: serial link + simulation, no GUI
```

> ⚠️ **Difference with ORME/OSNE**: for those two instruments, the serial
> link (RTU/serial) is itself an **optional feature** alongside a TCP
> transport that is always present, and `--no-default-features` can exclude
> it. Here, **there is no "serial-free" variant**: `tokio-serial` is a
> direct dependency (not feature-gated), present in **every** build,
> including headless — it is the instrument's only transport.

---

## 3. Code organization

```
mock_lib_control/        Reusable control library (pure, no IO, testable)
  src/pid.rs             Anti-windup PID
  src/lib.rs             re-exports (optional `serde` feature)

mock_bin_ru_pbdp/        PROFIBUS DP regulator binary (executable `ru_pbdp`)
  src/main.rs            Startup: config, Tokio runtime, actors, GUI/headless
  src/regulator.rs        Synchronous business model (PID + first-order process), Command, step
  src/config.rs           AppConfig (TOML), SerialConfig, ProcessConfig, RegulationConfig, ServerStatus
  src/profibus.rs         PROFIBUS DP-V0 protocol: frame codec + FCS + SlaveFsm (SOURCE OF TRUTH)
  src/profibus_server.rs  Serial session loop (frame read → SlaveFsm → reply) + watchdog
  src/map.rs              Output/Input I/O block layout <-> regulator Command
  src/trace.rs            Circular frame log (GUI mini-terminal)
  src/gui.rs              egui GUI (single page + mini-terminal + Settings modal)
  src/branding.rs         Embedded logos (`gui` feature)
  src/i18n.rs             Typed i18n catalogue (8 languages), no dependency
  src/actors/
    simulation.rs         Control loop (50 ms simulation step)
    network.rs            PROFIBUS DP serial link actor, hot-reconfigurable

docs/                     Design, PROFIBUS reference, manual, maintenance (multilingual)
```

**Golden rule**: the business logic (`mock_lib_control`, `regulator.rs`,
`profibus.rs`, `map.rs`) stays **synchronous and tested**; async is confined
to the actors and serial IO. Control model modelled on **ORME**
(`mock_bin_ru_modbus`) — same invariants.

---

## 4. Configuration

- File: `mock_ru_pbdp.toml` in the current directory, or a path supplied via
  the `MOCK_CONFIG` environment variable.
- Loaded at startup; **default values** if absent or unreadable (a warning
  is logged, the application still starts).
- **Every value coming from the TOML is sanitized**
  (`AppConfig::sanitized`): setpoint/PID bounds reordered, floats forced
  finite, `τ ≥ 1e-3`, `dead_time` bounded, **station address bounded to
  `[0, 125]`**. **Invariant: never call `f32::clamp` with unvalidated
  bounds** (panics if `min > max` or `NaN`).
- Saved from the GUI (*Apply* / *Save* / *Reset to defaults* buttons).

Structure (all sections are optional, filled in with defaults):

```toml
language = "en"
check_updates = true       # check at startup whether a newer release exists (GUI)

[network.serial]
port = "/dev/ttyUSB0"      # "COM3" by default on Windows
baud = 500000              # standard PROFIBUS DP value (9600 .. 12000000)
station_address = 3        # address of the simulated slave (0-125)
watchdog_enabled = true    # allows the watchdog announced by the master (Set_Prm)

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

> The **serial frame format (8E1)** is fixed by the PROFIBUS DP standard and
> is **not** a configuration field — see `SerialConfig::open` in
> [`config.rs`](../../src/config.rs). Unlike ORME/OSNE, **no IP allowlist**
> (the serial link is inherently point-to-point).

### Update check

If `check_updates = true` (default) **and** the binary is built with the
`gui` feature, the GUI queries **at startup** the latest release published
on GitHub (`CESAMLAB/cesam-tools`) via the shared **`mock_lib_update`** crate
(`ureq`/`rustls`, bundled roots, timeout-bounded thread). **Absent from
headless builds** (`--no-default-features`).

---

## 5. Dependencies and version pitfalls

| Crate | Role | Watch out for |
|-------|------|-------------------|
| `tokio` | async runtime | shared features + `io-util` |
| `ractor` | actors | default features |
| `tokio-serial` | PROFIBUS DP link | **direct dependency, not feature-gated** (see §2); `default-features = false` (no `libudev` enumeration) |
| `eframe`/`egui` | GUI | versions tied together, `gui` feature |
| `egui_plot` | trend curve | ⚠️ **versioned one minor ahead of `egui`**: for `egui` 0.33 → `egui_plot` **0.34** |
| `serde`/`toml` | persistence | `mock_lib_control` exposes a `serde` feature enabled by the binary |
| `mock_lib_update` (`ureq`/`rustls`) | update check | `gui` feature only; absent headless |

Shared versions are centralized in `[workspace.dependencies]` of the root
`Cargo.toml`. When bumping `egui`/`eframe`, **check the matching
`egui_plot` version** (otherwise a "two versions of crate egui" error).

---

## 6. Extending the project

### 6.1 Adding a PROFIBUS service (SAP)

Everything happens in **[`profibus.rs`](../../src/profibus.rs)** (the
protocol's source of truth):

1. Add the `SAP_*` constant and the corresponding variant in `enum Request`;
   wire the decoding into `decode_request` (and, for tests, into
   `encode_request`).
2. Handle the new request in `SlaveFsm::handle` (state transition if
   relevant, `Handled` returned).
3. Update the module's doc comment and
   **[reference_profibus.md](reference_profibus.md)**.
4. Add a test in `profibus.rs`'s `tests` module (and, if the full session
   is affected, in `profibus_server.rs`).

### 6.2 Changing the I/O blocks (`Output`/`Input`)

1. Adjust the layout in **[`map.rs`](../../src/map.rs)**
   (`decode_output`/`encode_input`), keeping `OUTPUT_LEN`/`INPUT_LEN`
   consistent with `SlaveProfile` (`profibus_server.rs`).
2. Update the table in **[reference_profibus.md](reference_profibus.md)**
   §3 (documentation source of truth, copied from `map.rs`'s doc comment).
3. Add a round-trip test in `map.rs`.

### 6.3 Adding a business command / GUI setting

1. Variant in `enum Command` (`regulator.rs`) + handling in
   `Regulator::apply` (with sanitization).
2. Field in `RegulatorSnapshot` if the value must be observable.
3. Wire the GUI (`gui.rs`) via a non-blocking `cast`.
4. If persistent: field in `AppConfig` (`config.rs`) + sanitization in
   `sanitized` + reporting in `to_regulator_config`.

### 6.4 Adding an interface string (i18n)

Every GUI string **must** go through a `Msg` key (`i18n.rs`) with its **8
translations** (a fixed-size array checked at compile time). PROFIBUS
service identifiers and unit suffixes stay hardcoded.

### 6.5 Adding a new instrument

1. Create `mock_bin_<name>/` and add it to the root `Cargo.toml`'s
   `members`.
2. Reuse `mock_lib_control`; factor out anything shared into a `mock_lib_*`.
3. Follow the same layout: synchronous model, `ractor` actor(s), protocol
   layer, GUI. Naming convention: `mock_bin_<type>_<protocol>`.

---

## 7. Test strategy

- **Frame codec** (`profibus.rs`): round-trip of `SD1`/`SD2`/`SD3`/`SD4`,
  rejection of incorrect checksum and length, encoding/decoding of the
  requests (`Slave_Diag`/`Set_Prm`/`Chk_Cfg`/`Data_Exchange`) and of the
  mode byte.
- **State machine** (`profibus.rs`): full sequence
  `Power_On → Wait_Prm → Wait_Cfg → Data_Exchange`, rejection of a `Set_Prm`
  with a wrong identifier (stays in `Wait_Prm`).
- **I/O blocks** (`map.rs`): a too-short output block → no command;
  round-trip of setpoint/mode; the input block reflects the snapshot
  (status bits, measurement).
- **Config** (`config.rs`): TOML round-trip, sanitization (inverted bounds,
  non-finite values, out-of-range station address) without a panic, clean
  error when opening a missing serial port.
- **Network session** (`profibus_server.rs`, `#[tokio::test]` on
  `tokio::io::duplex`): full handshake up to `Data_Exchange` with effective
  command application, a frame addressed to another station ignored (no
  activity marked), watchdog expiry forcing the safe state.

Run: `cargo test -p mock_bin_ru_pbdp` (or `--workspace`) — **36 tests**, all
**deterministic and GUI-free**, no slow/`#[ignore]` test (unlike ORUE, where
RSA generation justifies ignored tests).

---

## 8. Troubleshooting

| Symptom | Lead |
|----------|-------|
| "two versions of crate `egui`" | `egui_plot` / `egui` mismatch: align the versions (§5). |
| The GUI doesn't open | No display (headless) or missing system libraries (§1). |
| Serial port opening error (GUI header) | Missing port, wrong path, or permissions (`dialout` group on Linux) — the simulation keeps running without a link. |
| The link stays in `Wait_Prm` | The master isn't sending `Set_Prm` with the expected identifier (`0xEE01`) — see [reference_profibus.md](reference_profibus.md) §2. |
| The link stays in `Wait_Cfg` | The received `Chk_Cfg` doesn't announce `out_len=45`/`in_len=17`. |
| The device stops on its own | Protocol watchdog triggered (prolonged silence from the master) — expected safe state, not a bug. |
| No watchdog even though the master requests one | `watchdog_enabled = false` in local configuration: the master's request is deliberately ignored. |

Increase verbosity: `RUST_LOG=debug` (or `trace`).

---

## 9. Distribution build

```bash
cargo build --release -p mock_bin_ru_pbdp
# Standalone binary:
target/release/ru_pbdp
```

The `release` profile enables `lto = "thin"` and `opt-level = 3` (see the
root `Cargo.toml`). To distribute: ship the binary plus a sample
`mock_ru_pbdp.toml`. License: **MIT** (`LICENSE` file).

### `gui` feature (build with / without the interface)

```bash
cargo build --release -p mock_bin_ru_pbdp                       # with GUI (workstation)
cargo build --release -p mock_bin_ru_pbdp --no-default-features  # "headless": serial link + simulation, no GUI
```

Unlike OSNE, the **headless** mode does not make the serial link optional
(§2): it only removes the GUI. It remains relevant for a screen-less
deployment connected to a real serial/USB port.

### Linux desktop integration (taskbar icon)

The ORPD icon (`pic/ru_pbdp-icon.png`, generated by
[`pic/ru_pbdp-logo.gen.py`](../../../pic/ru_pbdp-logo.gen.py)) is
**embedded** in the binary (`branding.rs` → `window_icon`). This is enough
on **X11, Windows and macOS**. On **Wayland**, the compositor **ignores**
the embedded icon: it associates the window to its **`app_id`** ("ru_pbdp",
set in `main.rs` via `with_app_id`) with a `ru_pbdp.desktop` file of the
same name, and shows the `Icon=ru_pbdp` resolved from the `hicolor` icon
theme.

To get the icon under Wayland, install the desktop entry for the current
user:

```bash
scripts/install-desktop.sh ru_pbdp
```

The script copies:

| Source | Destination |
|--------|-------------|
| `pic/ru_pbdp-icon.png` | `~/.local/share/icons/hicolor/256x256/apps/ru_pbdp.png` |
| `packaging/ru_pbdp.desktop` | `~/.local/share/applications/ru_pbdp.desktop` |

then refreshes the caches. Three names **must stay aligned**: the `app_id`
(`main.rs`), the `ru_pbdp.desktop` file (+ its `StartupWMClass`), and the
`ru_pbdp.png` icon (= `Icon=ru_pbdp`).

---

## 10. "Prod" build — cross-compilation from Linux

Everything is produced **from Linux** by
[`scripts/build-prod.sh`](../../../scripts/build-prod.sh), which builds
**every instrument in the workspace** (`INSTRUMENTS` table, entry
`mock_bin_ru_pbdp:ru_pbdp:0` — port `0`: serial link, no IP port):

| Output | Target | GUI | Method |
|--------|-------|-----|---------|
| `dist/ru_pbdp-linux-x86_64` | `x86_64-unknown-linux-gnu` | ✅ | `cross` |
| `dist/ru_pbdp-windows-x86_64.exe` | `x86_64-pc-windows-gnu` | ✅ | `cross` (mingw) |
| `dist/ru_pbdp-rpi-arm64` | `aarch64-unknown-linux-gnu` (Pi 3/4/5, Pi OS 64-bit) | ✅ | `cross` |
| Headless Docker image `ru_pbdp:headless` | multi-arch `amd64` + `arm64` | ❌ | `docker buildx` |
| `dist/ru_pbdp_<ver>_amd64.deb` / `_arm64.deb` | Debian/Ubuntu package | ✅ | `dpkg-deb` |
| `dist/ru_pbdp-setup-x86_64.exe` | Windows installer | ✅ | NSIS (`makensis`) |

```bash
cargo install cross          # prerequisite (once) — Docker must be running
scripts/build-prod.sh        # every instrument, including ru_pbdp
ONLY=ru_pbdp scripts/build-prod.sh   # this instrument only
```

⚠️ **Don't mix native `cargo` and `cross`** in the same `target/`
(incompatible proc-macros → `can't find crate for …_derive`). The script
always goes through `cross`.

### Headless Docker image: limited usefulness without serial passthrough

The image ([`docker/Dockerfile.headless`](../../../docker/Dockerfile.headless))
is built like for the other instruments (`EXPOSE 0`, inert metadata), but
**is only really useful with a serial device mounted** into the container:

```bash
docker run --rm --device=/dev/ttyUSB0 -v "$PWD/conf:/data" ru_pbdp:headless
```

Without `--device`, the container starts but cannot open any serial port
(same behaviour as missing hardware locally — see §8).

---

## 11. Conventions

- Code and comments in **French** (project-wide convention); logs and error
  messages in **English**.
- `cargo clippy --workspace` **warning-free** before any commit.
- Every new business or protocol behaviour comes with a **test**.
- The PROFIBUS DP-V0 protocol is changed in **`profibus.rs`** (source of
  truth), together with an update to
  **[reference_profibus.md](reference_profibus.md)**.
