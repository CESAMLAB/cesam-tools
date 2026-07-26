# Onderhoud — EtherNet/IP-regelaar (OREE)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · **NL** · [PL](../pl/maintenance.md)*

---

## 1. Build & start

```bash
cargo run -p mock_bin_ru_ethernetip                        # GUI + EtherNet/IP-adapter
cargo build -p mock_bin_ru_ethernetip --release            # uitvoerbaar bestand GUI
cargo build -p mock_bin_ru_ethernetip --no-default-features # headless (zonder GUI)
```

Features: `gui` (GUI `egui`, standaard). `--no-default-features` produceert een binary
**headless**: EtherNet/IP-adapter + simulatie, zonder GUI noch updatecontrole.
Poort 44818 vereist **geen enkel privilege**.

## 2. Configuratie

TOML-bestand `mock_ru_ethernetip.toml` (huidige map; pad overschrijfbaar via
`MOCK_CONFIG`). Secties: `language`, `[network]` (`bind_ip`, `port`, `allowlist`),
`[process]`, `[regulation]`, `check_updates`. Elke waarde wordt **ontsmet** bij het
laden.

## 3. Tests

```bash
cargo test -p mock_bin_ru_ethernetip      # unit + lokale TCP-roundtrip
```

- **Protocollaag** (`eip_server`, zonder netwerk): RegisterSession, Read/Write Tag,
  BOOL-schrijfactie, onbekende tag (`0x05`), schrijfactie op een alleen-lezen-tag, **niet-panic**
  bij misvormde pakketten.
- **Netwerkactor**: bind/luisteren en een **echte TCP-roundtrip** (RegisterSession,
  Write en daarna Read van de setpoint) — zonder afhankelijkheid van een externe client.

## 4. Probleemoplossing

| Symptoom | Spoor |
|---|---|
| Client geweigerd | IP-toegangslijst; firewall; IP/poort (44818) |
| Tag onvindbaar | onjuiste naam (hoofdletters); zie de tagtabel |
| Schrijfactie zonder effect | tag alleen-lezen |
| Inconsistente waarden | EtherNet/IP is **little-endian** (REAL = `f32` LE) |

## 5. Docker (headless)

Headless image via `scripts/build-prod.sh` (entry
`mock_bin_ru_ethernetip:ru_eip:44818`, `EXPOSE 44818`). Koppel een volume aan de
werkmap om de `mock_ru_ethernetip.toml` aan te leveren.

## 6. De tagtabel uitbreiden

De tagtabel en de mapping van de schrijfacties zijn de **bron van waarheid** in
[`eip_server.rs`](../../src/eip_server.rs) (`read_tag` + `write_tag`). Om een
tag toe te voegen: voeg hem toe aan `read_tag` (lezen) en, indien aanstuurbaar, aan `write_tag` (schrijven →
`Command`), reflecteer dit vervolgens hier en in
[`reference_ethernetip.md`](reference_ethernetip.md). Voeg een test toe in de module.

## 7. Cross / Windows

Zoals de andere instrumenten (zie `Cross.toml`). Geen bijzondere native
afhankelijkheid: de EtherNet/IP-laag is 100% Rust over standaard-TCP.
