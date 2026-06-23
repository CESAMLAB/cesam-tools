# Onderhoudsdocumentatie — RU/OPC UA (workspace `cesam-tools`)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · **NL** · [PL](../pl/maintenance.md)*

> Crate: `mock_bin_ru_opcua` · Uitvoerbaar bestand: **ru_opcua**

---

## 1. Vereisten

- **Rust** recent. ⚠️ MSRV eigen aan deze crate: **1.91** (`async-opcua` declareert
  geen `rust-version` en trekt recente afhankelijkheden mee; de rest van de workspace
  zit op 1.85).
- Voor de GUI: de systeemafhankelijkheden van `eframe`/`egui` (dezelfde als ORME/OSNE).
- Voor de *headless*-build: geen grafische afhankelijkheid.

---

## 2. Gangbare commando's

```bash
cargo run -p mock_bin_ru_opcua                       # GUI + OPC UA-server
cargo run -p mock_bin_ru_opcua --no-default-features # headless (zonder GUI)
cargo test -p mock_bin_ru_opcua                      # eenheidstests
cargo clippy -p mock_bin_ru_opcua --all-targets      # lint
MOCK_CONFIG=./ma_config.toml cargo run -p mock_bin_ru_opcua  # alternatieve config
```

### Cargo-features

- **`gui`** (standaard): grafische interface `egui` + update-controle.
- `--no-default-features`: **headless**-binary (OPC UA-server + simulatie,
  zonder GUI noch update-netwerk).

De `async-opcua`-server is **altijd** aanwezig (de feature `server` van
`async-opcua`), want dat is de bestaansreden van het instrument.

---

## 3. Organisatie van de code

```
mock_bin_ru_opcua/src/
├── main.rs            # Stelt Tokio-runtime + actoren + GUI/headless samen
├── regulator.rs       # Synchroon bedrijfsmodel (PID + proces), commando's, stap
├── config.rs          # AppConfig (TOML), sanitized(), ServerStatus
├── i18n.rs            # i18n-catalogus (8 talen), Lang + Msg + tr()
├── opcua_server.rs    # OPC UA-server: build + adresruimte + callbacks
├── gui.rs             # egui-GUI (feature gui)
├── branding.rs        # Ingebedde logo's (feature gui)
└── actors/
    ├── simulation.rs  #   regellus (tick 0,5 s)
    └── network.rs     #   warm (her)configureerbare OPC UA-server
```

---

## 4. Configuratie

`AppConfig` (taal / netwerk / proces / regeling / `check_updates`) wordt
geserialiseerd in **TOML** (`mock_ru_opcua.toml`, te overschrijven via `MOCK_CONFIG`),
geladen bij het opstarten (standaardwaarden indien afwezig), opgeslagen vanuit de GUI.
Elke waarde wordt **gesaneerd** bij het laden (`AppConfig::sanitized`: geordende
grenzen, `τ ≥ 1e-3`, `dead_time ≥ 0`, eindige floats).

**Invariant**: nooit `f32::clamp` aanroepen met niet-gevalideerde grenzen (panic
als `min > max` of `NaN`). De netwerk-schrijfbewerkingen lopen eveneens via
`Regulator::apply`, dat saneert.

### Update-controle

Alleen feature `gui`: bij het opstarten bevraagt de GUI de laatste GitHub-release
via de gedeelde lib `mock_lib_update` (thread begrensd door timeout) en toont een
banner als er een recentere versie bestaat. Instelbaar via `check_updates`.

---

## 5. Afhankelijkheden en versievalkuilen

- **`async-opcua` 0.18** (server). Crypto **100 % Rust** (RustCrypto): **geen
  OpenSSL-afhankelijkheid** → schone cross-compilatie. Licentie **MPL-2.0** (zie `NOTICE`).
- ⚠️ `async-opcua` declareert **geen MSRV**: valideer op de doel-toolchain voordat
  je de versie bumpt.
- ⚠️ Het instantiecertificaat (`create_sample_keypair(true)` + `pki/`) wordt
  **alleen in versleutelde modus** gegenereerd (`security.encryption`). In de
  None-modus (standaard) geen certificaat (onmiddellijke start). ⚠️ De RSA-generatie
  in puur Rust is traag in *debug*: reken op enkele seconden bij de eerste overgang
  naar de versleutelde modus.
- `egui_plot` blijft **een minor vooruit** op `egui` (zie ORME/OSNE).

---

## 6. Het project uitbreiden

### 6.1 Een OPC UA-node toevoegen

In [`opcua_server.rs`](../../src/opcua_server.rs): declareer de node
(`add_var`), koppel een leescallback (`on_read_*`) en, indien beschrijfbaar, een
schrijfcallback (`on_write_*`) die een `Command` uitzendt. Weerspiegel de tabel in
[`reference_opcua.md`](reference_opcua.md).

### 6.2 Een bedrijfscommando toevoegen

Breid de enum `Command` uit ([`regulator.rs`](../../src/regulator.rs)), behandel het
geval in `Regulator::apply` (met sanering), voeg een test toe.

### 6.3 Een interface-string toevoegen (i18n)

Voeg een variant toe aan `Msg` ([`i18n.rs`](../../src/i18n.rs)) en **de 8
vertalingen** (array van vaste grootte gecontroleerd bij het compileren).

### 6.4 Beveiliging (`SecurityConfig`)

De beveiliging is geïmplementeerd in [`opcua_server.rs`](../../src/opcua_server.rs):
`security.encryption` voegt een endpoint `Basic256Sha256`/`SignAndEncrypt` toe met
automatisch gegenereerd certificaat en anonieme en/of gebruiker/wachtwoord-tokens
(`ServerUserToken::user_pass`). Het logfilter `opcua_crypto::certificate_store=off`
([`main.rs`](../../src/main.rs)) betreft alleen de None-modus (geen certificaat); in
versleutelde modus heeft het geen effect. Verdere mogelijkheden: beleidsregels
`Aes256Sha256RsaPss`, een expliciete PKI-vertrouwenslijst in plaats van
`trust_client_certs`, X.509-tokens.

---

## 7. Teststrategie

De bedrijfskern (`regulator.rs`) en de configuratie (`config.rs`) zijn **puur en
getest**: PID-convergentie, setpoint-clamp, relaxatie bij stilstand, proceswijziging
zonder PV-sprong, TOML-sanering, TOML-heen-en-terug. De i18n controleert de
niet-leegheid en het heen-en-terug van de taal. De async-logica (actoren, server)
blijft dun en steunt op deze geteste bouwstenen.

---

## 8. Probleemoplossing

| Symptoom | Waarschijnlijke oorzaak | Oplossing |
|---|---|---|
| `failed to bind` bij het opstarten | poort al bezet / < 1024 zonder rechten | poort wijzigen (*Parameters*) of als root starten |
| Client ziet de nodes niet | verkeerde endpoint / beveiliging | `opc.tcp://…:4840/`, None, Anonymous; *Browse* onder `Objects` |
| Schrijfbewerking `Bad_TypeMismatch` | onjuist type | `Double` voor de grootheden, `Boolean` voor `Run`/`Auto` |
| WARN "encrypted endpoints disabled" | geen certificaat (Fase 1b) | normaal; de None-endpoint werkt |

---

## 9. "Prod"-build — cross-compilatie vanaf Linux

Het instrument is geïntegreerd in [`scripts/build-prod.sh`](../../../scripts/build-prod.sh)
(array `INSTRUMENTS`): exes **met GUI** voor Linux x86_64, Windows x86_64 en
Raspberry Pi arm64 (via `cross`), plus een headless Docker-image.

⚠️ **Cross Windows en `GetHostNameW`**: de OPC UA-stack trekt `gethostname` mee, dat
verwijst naar het winsock-symbool `GetHostNameW`. De mingw-w64-importbibliotheek van
de **standaard** `cross`-image (`:0.2.5`) is te oud om dit te leveren →
mislukking bij het linken. De repo legt daarom, in [`Cross.toml`](../../../Cross.toml),
de Windows-GNU-image vast op **`:main`** (recente mingw). Gevalideerd: headless- **en**
GUI-builds produceren een geldige `.exe`; ORME/OSNE compileren nog steeds (superset-image).

---

## 10. Conventies

- Code en commentaar in het **Frans**; logs/fouten in het **Engels**.
- GUI-strings via `i18n` (8 talen); nooit hardcoded.
- Bedrijfslogica **synchroon en testbaar**; het asynchrone is beperkt tot de actoren
  en de IO. `cargo clippy --workspace` zonder waarschuwing.
- `ractor`-invarianten: geen `Mutex`-guard over een `.await`; geen losgekoppelde
  timer/`spawn` zonder `JoinHandle` afgebroken bij het stoppen.
