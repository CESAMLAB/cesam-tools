# Onderhoud — S7-regelaar (ORSS)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · **NL** · [PL](../pl/maintenance.md)*

---

## 1. Build & start

```bash
cargo run -p mock_bin_ru_s7                        # GUI + S7-server
cargo build -p mock_bin_ru_s7 --release            # uitvoerbaar bestand met GUI
cargo build -p mock_bin_ru_s7 --no-default-features # headless (zonder GUI)
```

Features: `gui` (`egui`-GUI, standaard). `--no-default-features` produceert een
**headless** binary: S7-server + simulatie, zonder GUI noch updatecontrole.

⚠️ Poort **102** (S7-standaard) heeft de voorkeur (< 1024): voer uit met de juiste
rechten of kies een hoge poort in de configuratie.

## 2. Configuratie

TOML-bestand `mock_ru_s7.toml` (huidige map; pad overschrijfbaar via `MOCK_CONFIG`).
Secties: `language`, `[network]` (`bind_ip`, `port`, `allowlist`), `[process]`,
`[regulation]`, `check_updates`. Elke waarde wordt **gesaneerd** bij het laden.

## 3. Tests

```bash
cargo test -p mock_bin_ru_s7      # unit + lokale TCP-round-trip
```

- **Protocollaag** (`s7_server`, zonder netwerk): CR→CC, Setup, Read/Write Var,
  bit-schrijfactie, retourcode buiten zone, **niet-panic** bij misvormde frames,
  round-trip van de DB-image.
- **Netwerkactor**: bind/luisteren, en een **echte TCP-round-trip** (COTP-verbinding,
  schrijven en daarna teruglezen van de setpoint via ruwe S7-frames) — zonder
  afhankelijkheid van een externe client.

## 4. Probleemoplossing

| Symptoom | Aanwijzing |
|---|---|
| Bind mislukt (`permission denied`) | poort 102 < 1024 → root-rechten of hoge poort |
| Client geweigerd | IP-toelatingslijst; firewall; IP/poort |
| Geen antwoord | rack/slot (test 0/1, 0/2); frames buiten deelverzameling genegeerd |
| Schrijfactie zonder effect | offset alleen-lezen (zie adresseringsplan) |

## 5. Docker (headless)

Headless-image via `scripts/build-prod.sh` (entry `mock_bin_ru_s7:ru_s7:102`,
`EXPOSE 102`). Koppel een volume aan de werkmap om het `mock_ru_s7.toml` te leveren.
De container publiceert poort 102; map naar een hoge poort aan de hostzijde indien
nodig.

## 6. Het adresseringsplan uitbreiden

Het DB1-plan en de mapping van schrijfacties zijn de **bron van waarheid** in
[`s7_server.rs`](../../src/s7_server.rs) (`db_image` + `handle_write`). Om een grootheid
toe te voegen: schrijf deze in `db_image` (lezen) en, indien stuurbaar, voeg ze toe aan
de `match` van `handle_write` (schrijven → `Command`), en weerspiegel dit hier en in
[`reference_s7.md`](reference_s7.md). Voeg een test toe in de module.

## 7. Cross / Windows

Zoals de andere instrumenten (zie `Cross.toml`). Geen bijzondere native
afhankelijkheid: de S7-laag is 100% Rust over standaard TCP.
