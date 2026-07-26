# Maintenance — EtherNet/IP Regulator (OREE)

*🌍 [FR](../fr/maintenance.md) · **EN** · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

---

## 1. Build & run

```bash
cargo run -p mock_bin_ru_ethernetip                        # GUI + EtherNet/IP adapter
cargo build -p mock_bin_ru_ethernetip --release            # GUI executable
cargo build -p mock_bin_ru_ethernetip --no-default-features # headless (no GUI)
```

Features: `gui` (`egui` GUI, default). `--no-default-features` produces a
**headless** binary: EtherNet/IP adapter + simulation, without GUI or update check.
Port 44818 requires **no privilege**.

## 2. Configuration

TOML file `mock_ru_ethernetip.toml` (current directory; path overridable via
`MOCK_CONFIG`). Sections: `language`, `[network]` (`bind_ip`, `port`, `allowlist`),
`[process]`, `[regulation]`, `check_updates`. Every value is **sanitized** on load.

## 3. Tests

```bash
cargo test -p mock_bin_ru_ethernetip      # unit tests + local TCP round-trip
```

- **Protocol layer** (`eip_server`, no network): RegisterSession, Read/Write Tag,
  BOOL write, unknown tag (`0x05`), write to a read-only tag, **no panic** on
  malformed packets.
- **Network actor**: bind/listen and a **real TCP round-trip** (RegisterSession,
  Write then Read of the setpoint) — without any dependency on an external client.

## 4. Troubleshooting

| Symptom | Lead |
|---|---|
| Client refused | IP allowlist; firewall; IP/port (44818) |
| Tag not found | inexact name (case); see the tag table |
| Write with no effect | read-only tag |
| Inconsistent values | EtherNet/IP is **little-endian** (REAL = `f32` LE) |

## 5. Docker (headless)

Headless image via `scripts/build-prod.sh` (entry
`mock_bin_ru_ethernetip:ru_eip:44818`, `EXPOSE 44818`). Mount a volume on the working
directory to provide the `mock_ru_ethernetip.toml`.

## 6. Extending the tag table

The tag table and the write mapping are the **source of truth** in
[`eip_server.rs`](../../src/eip_server.rs) (`read_tag` + `write_tag`). To add a tag:
add it to `read_tag` (read) and, if controllable, to `write_tag` (write →
`Command`), then reflect it here and in
[`reference_ethernetip.md`](reference_ethernetip.md). Add a test in the module.

## 7. Cross / Windows

Like the other instruments (cf. `Cross.toml`). No particular native dependency: the
EtherNet/IP layer is 100% Rust over standard TCP.
