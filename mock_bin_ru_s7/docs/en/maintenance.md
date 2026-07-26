# Maintenance — S7 Regulator (ORSS)

*🌍 [FR](../fr/maintenance.md) · **EN** · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

---

## 1. Build & launch

```bash
cargo run -p mock_bin_ru_s7                        # GUI + S7 server
cargo build -p mock_bin_ru_s7 --release            # GUI executable
cargo build -p mock_bin_ru_s7 --no-default-features # headless (no GUI)
```

Features: `gui` (`egui` GUI, default). `--no-default-features` produces a
**headless** binary: S7 server + simulation, without GUI or update check.

⚠️ Port **102** (S7 standard) is preferred (< 1024): run with the appropriate
privileges or choose a high port in the configuration.

## 2. Configuration

TOML file `mock_ru_s7.toml` (current directory; path overridable via `MOCK_CONFIG`).
Sections: `language`, `[network]` (`bind_ip`, `port`, `allowlist`), `[process]`,
`[regulation]`, `check_updates`. Every value is **sanitized** at load time.

## 3. Tests

```bash
cargo test -p mock_bin_ru_s7      # unit + local TCP round-trip
```

- **Protocol layer** (`s7_server`, no network): CR→CC, Setup, Read/Write Var, bit
  write, out-of-area return code, **no panic** on malformed frames, DB image
  round-trip.
- **Network actor**: bind/listen, and a **real TCP round-trip** (COTP connection,
  write then read-back of the setpoint via raw S7 frames) — without any dependency on
  an external client.

## 4. Troubleshooting

| Symptom | Lead |
|---|---|
| Bind fails (`permission denied`) | port 102 < 1024 → root privileges or high port |
| Client refused | IP allowlist; firewall; IP/port |
| No response | rack/slot (try 0/1, 0/2); frames outside the subset are ignored |
| Write has no effect | read-only offset (see addressing plan) |

## 5. Docker (headless)

Headless image via `scripts/build-prod.sh` (entry `mock_bin_ru_s7:ru_s7:102`,
`EXPOSE 102`). Mount a volume on the working directory to provide the
`mock_ru_s7.toml`. The container publishes port 102; map it to a high port on the
host side if needed.

## 6. Extend the addressing plan

The DB1 plan and the write mapping are the **source of truth** in
[`s7_server.rs`](../../src/s7_server.rs) (`db_image` + `handle_write`). To add a
quantity: write it into `db_image` (read) and, if controllable, add it to the
`match` in `handle_write` (write → `Command`), then reflect it here and in
[`reference_s7.md`](reference_s7.md). Add a test in the module.

## 7. Cross / Windows

Like the other instruments (see `Cross.toml`). No particular native dependency: the
S7 layer is 100% Rust over standard TCP.
