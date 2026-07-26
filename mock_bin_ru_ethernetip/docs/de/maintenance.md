# Wartung — EtherNet/IP-Regler (OREE)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · **DE** · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

---

## 1. Build & Start

```bash
cargo run -p mock_bin_ru_ethernetip                        # IHM + EtherNet/IP-Adapter
cargo build -p mock_bin_ru_ethernetip --release            # IHM-Programm
cargo build -p mock_bin_ru_ethernetip --no-default-features # headless (ohne IHM)
```

Features: `gui` (IHM `egui`, standardmäßig). `--no-default-features` erzeugt ein
**headless**-Binary: EtherNet/IP-Adapter + Simulation, ohne IHM und ohne
Update-Prüfung. Der Port 44818 erfordert **keine Privilegien**.

## 2. Konfiguration

TOML-Datei `mock_ru_ethernetip.toml` (aktuelles Verzeichnis; Pfad überschreibbar durch
`MOCK_CONFIG`). Abschnitte: `language`, `[network]` (`bind_ip`, `port`, `allowlist`),
`[process]`, `[regulation]`, `check_updates`. Jeder Wert wird beim Laden **bereinigt**.

## 3. Tests

```bash
cargo test -p mock_bin_ru_ethernetip      # Unit-Tests + lokaler TCP-Round-Trip
```

- **Protokollschicht** (`eip_server`, ohne Netzwerk): RegisterSession, Read/Write Tag,
  BOOL-Schreibvorgang, unbekanntes Tag (`0x05`), Schreiben eines schreibgeschützten Tags,
  **Nicht-Panik** bei fehlerhaften Paketen.
- **Netzwerkaktor**: Bind/Lauschen und ein **echter TCP-Round-Trip** (RegisterSession,
  Write dann Read des Sollwerts) — ohne Abhängigkeit von einem externen Client.

## 4. Fehlerbehebung

| Symptom | Hinweis |
|---|---|
| Client abgewiesen | IP-Whitelist; Firewall; IP/Port (44818) |
| Tag nicht gefunden | ungenauer Name (Groß-/Kleinschreibung); siehe Tag-Tabelle |
| Schreibvorgang ohne Wirkung | schreibgeschütztes Tag |
| Inkonsistente Werte | EtherNet/IP ist **little-endian** (REAL = `f32` LE) |

## 5. Docker (headless)

Headless-Image über `scripts/build-prod.sh` (Eintrag
`mock_bin_ru_ethernetip:ru_eip:44818`, `EXPOSE 44818`). Ein Volume auf das
Arbeitsverzeichnis einhängen, um die `mock_ru_ethernetip.toml` bereitzustellen.

## 6. Tag-Tabelle erweitern

Die Tag-Tabelle und das Mapping der Schreibvorgänge sind die **Quelle der Wahrheit** in
[`eip_server.rs`](../../src/eip_server.rs) (`read_tag` + `write_tag`). Um ein Tag
hinzuzufügen: es zu `read_tag` (Lesen) und, falls steuerbar, zu `write_tag` (Schreiben →
`Command`) hinzufügen, dann hier und in
[`reference_ethernetip.md`](reference_ethernetip.md) widerspiegeln. Einen Test im Modul
hinzufügen.

## 7. Cross / Windows

Wie die anderen Instrumente (vgl. `Cross.toml`). Keine besondere native Abhängigkeit:
die EtherNet/IP-Schicht ist 100 % Rust auf Standard-TCP.
