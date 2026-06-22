# Maintenance documentation — RU/OPC UA (workspace `cesam-tools`)

*🌍 [FR](../fr/maintenance.md) · **EN** · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

> Crate: `mock_bin_ru_opcua` · Executable: **ru_opcua**

---

## 1. Prerequisites

- Recent **Rust**. ⚠️ MSRV specific to this crate: **1.91** (`async-opcua`
  declares no `rust-version` and pulls in recent dependencies; the rest of the
  workspace is at 1.85).
- For the GUI: the system dependencies of `eframe`/`egui` (same as ORME/OSNE).
- For the *headless* build: no graphical dependency.

---

## 2. Common commands

```bash
cargo run -p mock_bin_ru_opcua                       # GUI + OPC UA server
cargo run -p mock_bin_ru_opcua --no-default-features # headless (no GUI)
cargo test -p mock_bin_ru_opcua                      # unit tests
cargo clippy -p mock_bin_ru_opcua --all-targets      # lint
MOCK_CONFIG=./ma_config.toml cargo run -p mock_bin_ru_opcua  # alternative config
```

### Cargo features

- **`gui`** (default): `egui` graphical interface + update check.
- `--no-default-features`: **headless** binary (OPC UA server + simulation, no GUI
  nor update network).

The `async-opcua` server is **always** present (the `server` feature of
`async-opcua`), since it is the instrument's reason for being.

---

## 3. Code organization

```
mock_bin_ru_opcua/src/
├── main.rs            # Assembles Tokio runtime + actors + GUI/headless
├── regulator.rs       # Synchronous business model (PID + process), commands, step
├── config.rs          # AppConfig (TOML), sanitized(), ServerStatus
├── i18n.rs            # i18n catalog (8 languages), Lang + Msg + tr()
├── opcua_server.rs    # OPC UA server: build + address space + callbacks
├── gui.rs             # egui GUI (gui feature)
├── branding.rs        # Embedded logos (gui feature)
└── actors/
    ├── simulation.rs  #   regulation loop (tick 0.5 s)
    └── network.rs     #   OPC UA server (re)configurable at runtime
```

---

## 4. Configuration

`AppConfig` (language / network / process / regulation / `check_updates`) is
serialized as **TOML** (`mock_ru_opcua.toml`, overridable via `MOCK_CONFIG`),
loaded at startup (defaults if absent), saved from the GUI. Every value is
**sanitized** at load (`AppConfig::sanitized`: ordered bounds, `τ ≥ 1e-3`,
`dead_time ≥ 0`, finite floats).

**Invariant**: never call `f32::clamp` with unvalidated bounds (panics if
`min > max` or `NaN`). Network writes also go through `Regulator::apply`, which
sanitizes.

### Update check

`gui` feature only: at startup, the GUI queries the latest GitHub release via the
shared `mock_lib_update` library (timeout-bounded thread) and shows a banner if a
newer version exists. Configurable via `check_updates`.

---

## 5. Dependencies and version pitfalls

- **`async-opcua` 0.18** (server). **100 % Rust** crypto (RustCrypto): **no
  OpenSSL dependency** → clean cross-compilation. License **MPL-2.0** (see
  `NOTICE`).
- ⚠️ `async-opcua` declares **no MSRV**: validate on the target toolchain before
  bumping the version.
- ⚠️ Certificate generation (`create_sample_keypair(true)`) is **intentionally
  disabled**: pure-Rust RSA generation is very slow in *debug* and would write
  into `pki/`. In Phase 1b (None endpoint), no certificate is required.
- `egui_plot` stays **one minor ahead** of `egui` (see ORME/OSNE).

---

## 6. Extending the project

### 6.1 Add an OPC UA node

In [`opcua_server.rs`](../../src/opcua_server.rs): declare the node (`add_var`),
wire up a read callback (`on_read_*`) and, if writable, a write callback
(`on_write_*`) that emits a `Command`. Mirror the table in
[`reference_opcua.md`](reference_opcua.md).

### 6.2 Add a business command

Extend the `Command` enum ([`regulator.rs`](../../src/regulator.rs)), handle the
case in `Regulator::apply` (with sanitization), add a test.

### 6.3 Add an interface string (i18n)

Add a variant to `Msg` ([`i18n.rs`](../../src/i18n.rs)) and **all 8
translations** (fixed-size array verified at compile time).

### 6.4 Phase 2 — security

Enable an encrypted endpoint (`Basic256Sha256`), provision an instance
certificate, add user authentication. Then remove the log filter
`opcua_crypto::certificate_store=off` set in [`main.rs`](../../src/main.rs).

---

## 7. Test strategy

The business core (`regulator.rs`) and the configuration (`config.rs`) are **pure
and tested**: PID convergence, setpoint clamp, relaxation at stop, process change
without a PV jump, TOML sanitization, TOML round-trip. The i18n checks
non-emptiness and the language round-trip. The async logic (actors, server) stays
thin and relies on these tested building blocks.

---

## 8. Troubleshooting

| Symptom | Likely cause | Remedy |
|---|---|---|
| `failed to bind` at startup | port already taken / < 1024 without privileges | change the port (*Settings*) or run as root |
| Client does not see the nodes | wrong endpoint / security | `opc.tcp://…:4840/`, None, Anonymous; *Browse* under `Objects` |
| `Bad_TypeMismatch` write | incorrect type | `Double` for the quantities, `Boolean` for `Run`/`Auto` |
| WARN "encrypted endpoints disabled" | no certificate (Phase 1b) | normal; the None endpoint works |

---

## 9. "Prod" build — cross-compilation from Linux

The instrument is integrated into [`scripts/build-prod.sh`](../../../scripts/build-prod.sh)
(`INSTRUMENTS` table): exes **with GUI** for Linux x86_64, Windows x86_64 and
Raspberry Pi arm64 (via `cross`), plus a headless Docker image.

⚠️ **Cross Windows and `GetHostNameW`**: the OPC UA stack pulls in `gethostname`,
which references the winsock symbol `GetHostNameW`. The mingw-w64 import library of
the **default** `cross` image (`:0.2.5`) is too old to provide it → link-time
failure. The repository therefore pins, in [`Cross.toml`](../../../Cross.toml),
the Windows GNU image to **`:main`** (recent mingw). Validated: headless **and**
GUI builds produce a valid `.exe`; ORME/OSNE still compile (superset image).

---

## 10. Conventions

- Code and comments in **French**; logs/errors in **English**.
- GUI strings via `i18n` (8 languages); never hard-coded.
- Business logic **synchronous and testable**; the async is confined to the actors
  and IO. `cargo clippy --workspace` without a warning.
- `ractor` invariants: no `Mutex` guard held across an `.await`; no detached
  timer/`spawn` without a `JoinHandle` aborted at shutdown.
