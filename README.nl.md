<p align="center">
  <img src="pic/Logo-CESAM-Couleur-vect-card.png" alt="CESAM-Lab" height="84">
</p>

# cesam-tools — CESAM-Lab-toolkit

*🌍 [English](README.md) · [Français](README.fr.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Italiano](README.it.md) · [Português](README.pt.md) · **Nederlands** · [Polski](README.pl.md)*

<p align="center">
  <a href="https://github.com/CESAMLAB/cesam-tools/releases/latest"><img src="https://img.shields.io/github/v/release/CESAMLAB/cesam-tools?label=release" alt="Latest release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue" alt="License: MIT"></a>
</p>

Rust-workspace die de **tools van CESAM-Lab** verzamelt, te beginnen met
**simulatoren van industriële instrumenten**: virtuele apparaten die een
realistisch fysiek gedrag reproduceren en communiceren via veldprotocollen.
Nuttig om supervisors, PLC's of gateways te ontwikkelen, te testen en te
demonstreren **zonder echte hardware**.

> Gratis gedistribueerd onder [MIT](LICENSE)-licentie.

## Beschikbare instrumenten

| Crate | Product | Beschrijving | Protocol | GUI |
|-------|---------|--------------|----------|-----|
| [`mock_bin_ru_modbustcp`](mock_bin_ru_modbustcp) | **ORME** | Regelaar (PID / TOR / PWM) op overdrachtsfunctie | Modbus TCP & RTU (slave) | egui |
| [`mock_bin_su_namur`](mock_bin_su_namur) | **OSNE** | Bovenroerder voor laboratorium: motoroverdrachtsfunctie, snelle toerenregeling, instelbare viskeuze belasting | NAMUR over TCP & serieel RS-232 (slave) | egui |

Gedeelde bibliotheek:

| Crate | Beschrijving |
|-------|--------------|
| [`mock_lib_control`](mock_lib_control) | Herbruikbare regelbouwstenen: PID met anti-windup, aan-uit met hysterese, eerste-orde-proces + zuivere dode tijd (FOPDT). |

## ORME — de gesimuleerde regelaar

<p align="center">
  <img src="pic/orme-logo.svg" alt="ORME — Open Regulator Modbus Emulator" height="120">
</p>

> **ORME** — *Open Regulator Modbus Emulator*. **« Open de bus. »**
> Een veldregelaar die alleen bestaat op uw Modbus-bus.

Een volledige virtuele industriële regelaar:

- **Proces** gemodelleerd door een eerste-orde-overdrachtsfunctie met zuivere
  dode tijd `K·e^(-Ls) / (1 + T·s)` (typisch voor een oven of thermostaatbad).
- **Tweerichtingsregeling**: richting 1 (warm) en richting 2 (koud), elk
  configureerbaar in **PID**, **aan-uit (TOR)** of **cyclusrelais (PWM)**.
- **Modi** aan/uit en automatisch/handmatig.
- **Modbus-server** in **TCP** of **RTU serieel / RS485** (feature `rtu`), naar
  keuze. Adrestabel (setpoint, meting, uitgang, modi…), **IP-witte lijst** (jokers
  `*`) tijdens werking configureerbaar, en **single-master-beleid** (slechts één
  externe master tegelijk; in TCP verbreekt een nieuwkomer de vorige).
- **Grafische interface** op één pagina: besturing, real-time **trendgrafiek**,
  **live Modbus-adrestabel**, en een **Parameters-modaal** (transport TCP/RTU,
  poort, toegestane IP's, seriële parameters, overdrachtsfunctie, setpointgrenzen).
- **Persistente configuratie** in TOML-formaat (`mock_ru_modbustcp.toml`), herladen
  bij opstart, met knop om terug te zetten naar de standaardwaarden.

### Asynchrone architectuur

```
        Command (niet-blokkerende cast)        gedeelde momentopname
  GUI (egui) ──────────────────────►  SimulationActor  ──────────►  GUI (lezen)
  Modbus schrijven ────────────────►   (ractor)         ──────────►  Modbus-beeld
  Modbus lezen    ◄──────────────────────────────────────  Modbus-beeld
```

- **`ractor`**: één enkele actor bezit de toestand van de regelaar; alle mutaties
  verlopen via berichten (geen slot op de bedrijfslogica).
- **`tokio-modbus`**: Modbus TCP- en RTU-serieel-server (trait `Service`).
- **`eframe`/`egui`**: grafische interface op de hoofdthread.

## OSNE — de gesimuleerde laboratoriumroerder

> **OSNE** — *Open Stirrer NAMUR Emulator*.
> Een bovenroerder voor het laboratorium (IKA-stijl) die alleen bestaat op uw
> NAMUR-verbinding.

Een volledige virtuele laboratoriumroerder:

- **Motor** gemodelleerd door een rotatie-overdrachtsfunctie `J·dω/dt = T − k·η·ω −
  wrijving` (expliciete Euler), met een **snelle PID** die het koppel stuurt om het
  toerental-setpoint te volgen.
- **Instelbare viscositeit** `η`: verhoogt het belastingskoppel; bij hoge
  viscositeit verzadigt de motor en wordt het setpoint onbereikbaar
  (**overbelasting**) — net als een echte roerder.
- **NAMUR-server** (ASCII-commandoprotocol) over **TCP** (testen zonder hardware) of
  **serieel RS-232** (feature `serial`), met een **watchdog** per sessie
  (`OUT_WD1@<m>`), **single-master**-beleid en een **IP-witte lijst** (TCP).
- **Grafische interface** op één pagina: toerental-setpoint, viscositeit, live
  **trendgrafiek** van toerental/koppel, een ingebedde **NAMUR-miniterminal**
  (frames verzenden/inspecteren met commandogeschiedenis), en een
  **Parameters-modaal** (transport TCP/serieel, motorparameters, grenzen, i18n in 8
  talen).
- **Persistente configuratie** in TOML-formaat (`mock_su_namur.toml`), herladen bij
  opstart, met knop om terug te zetten naar de standaardwaarden.

Het deelt de architectuur van ORME (synchroon bedrijfsmodel, `ractor`-actoren,
`egui`-GUI). Start het met `cargo run -p mock_bin_su_namur`; de NAMUR-server luistert
standaard op `0.0.0.0:4001`.

## Downloaden

Voorgecompileerde binaries zijn beschikbaar op de pagina [**Releases**](https://github.com/CESAMLAB/cesam-tools/releases/latest) — **geen Rust-toolchain vereist**. Elk instrument levert zijn eigen uitvoerbaar bestand (`orme`, `osne`).

**ORME** (Modbus-regelaar):

| Platform | GUI | Headless (alleen TCP, geen GUI) |
|----------|-----|---------------------------------|
| Linux x86_64 | [`orme-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-linux-x86_64) | [`orme-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-linux-x86_64-headless) |
| Windows x86_64 | [`orme-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`orme-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-rpi-arm64) | [`orme-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-rpi-arm64-headless) |

**OSNE** (NAMUR-laboratoriumroerder):

| Platform | GUI | Headless (alleen TCP, geen GUI) |
|----------|-----|---------------------------------|
| Linux x86_64 | [`osne-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-linux-x86_64) | [`osne-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-linux-x86_64-headless) |
| Windows x86_64 | [`osne-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`osne-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-rpi-arm64) | [`osne-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-rpi-arm64-headless) |

```bash
chmod +x orme-linux-x86_64        # Linux / Raspberry Pi (idem voor osne-*)
./orme-linux-x86_64
```

Linux-/RPi-binaries zijn dynamisch gelinkt aan glibc en hebben een desktopomgeving (X11/Wayland) nodig voor de GUI. Op **Wayland** installeer je het desktopitem voor het pictogram in de taakbalk: `scripts/install-desktop.sh`. Controleer de integriteit met de gepubliceerde checksums:

```bash
sha256sum -c SHA256SUMS
```

## Snel starten

```bash
# Vereisten: Rust stable (editie 2021, >= 1.85).
# Linux-systeemafhankelijkheden voor de GUI: libxkbcommon, libwayland/xcb, openGL.

cargo run -p mock_bin_ru_modbustcp
```

Het venster opent en de Modbus TCP-server luistert op `0.0.0.0:5502`.
De **poort**, het **luister-IP** en de **IP-witte lijst** worden ingesteld in het
**⚙ Parameters**-modaal (tijdens werking toegepast) en vervolgens **persistent
opgeslagen** in `mock_ru_modbustcp.toml`. De **taal van de interface** (Frans,
Engels, Duits, Spaans, Italiaans, Portugees, Nederlands, Pools) wordt in datzelfde
modaal gekozen en is persistent. Om een ander configuratiebestand te gebruiken:

```bash
MOCK_CONFIG=/pad/naar/ma_config.toml cargo run -p mock_bin_ru_modbustcp
```

### De Modbus-verbinding testen

Met om het even welke Modbus-client (bv. `mbpoll`):

```bash
# Inschakelen (coil 0) dan de meting lezen (input registers 0-1, f32)
mbpoll -m tcp -a 1 -t 0 -p 5502 127.0.0.1 1      # de On/Off-coil schrijven
mbpoll -m tcp -a 1 -t 3:float -r 1 -p 5502 127.0.0.1   # PV lezen (f32)
```

De volledige adrestabel is gedocumenteerd in
[`mock_bin_ru_modbustcp/src/map.rs`](mock_bin_ru_modbustcp/src/map.rs).

## Ontwikkeling

```bash
cargo test --workspace      # unit- + integratietests
cargo clippy --workspace    # lint
```

## Documentatie

Elk instrument draagt zijn eigen documentatie in zijn submap `docs/`, beschikbaar in
acht talen (`docs/<taal>/`). Nederlandse versies:

**ORME** (Modbus-regelaar):

- [**Gebruikershandleiding**](mock_bin_ru_modbustcp/docs/nl/manuel_utilisateur.md) — ingebruikname, GUI, parameters, FAQ.
- [Ontwerpdocument](mock_bin_ru_modbustcp/docs/nl/conception.md) — architectuur en technische keuzes.
- [Modbus-adrestabel](mock_bin_ru_modbustcp/docs/nl/table_modbus.md) — volledig adresseringsplan.
- [Software-onderhoud](mock_bin_ru_modbustcp/docs/nl/maintenance.md) — build, configuratie, uitbreiding, probleemoplossing.

**OSNE** (NAMUR-laboratoriumroerder):

- [**Gebruikershandleiding**](mock_bin_su_namur/docs/nl/manuel_utilisateur.md) — ingebruikname, GUI, NAMUR-miniterminal, parameters, FAQ.
- [Ontwerpdocument](mock_bin_su_namur/docs/nl/conception.md) — motormodel, regellus, architectuur.
- [NAMUR-commandoset](mock_bin_su_namur/docs/nl/commandes_namur.md) — protocolreferentie (kanalen, commando's, voorbeelden).
- [Software-onderhoud](mock_bin_su_namur/docs/nl/maintenance.md) — build, configuratie, uitbreiding, probleemoplossing.

## Merk & logo's

De logo's staan in [`pic/`](pic/):

- [`orme-icon.svg`](pic/orme-icon.svg) / `orme-icon.png` — ORME-pictogram (wijzerplaat),
  ook ingebed als vensterpictogram van de applicatie.
- [`orme-logo.svg`](pic/orme-logo.svg) — volledig ORME-logo (pictogram + tekst).
- [`osne-icon.svg`](pic/osne-icon.svg) / `osne-icon.png` — OSNE-pictogram
  (roerderschoep), ook ingebed als OSNE-vensterpictogram.
- [`osne-logo.svg`](pic/osne-logo.svg) — volledig OSNE-logo (pictogram + tekst).
- [`Logo-CESAM-Couleur-vect.png`](pic/Logo-CESAM-Couleur-vect.png) — CESAM-Lab-logo.

Elk pictogram wordt **gegenereerd** vanuit zijn `*-logo.gen.py`-script
([`pic/orme-logo.gen.py`](pic/orme-logo.gen.py),
[`pic/osne-logo.gen.py`](pic/osne-logo.gen.py)). Het OSNE-script rastert ook
`osne-icon.png` rechtstreeks (via Pillow); de ORME-`.svg` wordt daarna gerasterd.

Op **Wayland** installeer je het taakbalkpictogram van een instrument met
`scripts/install-desktop.sh [orme|osne]`.

## Licentie

[MIT](LICENSE) © 2026 CESAM-Lab
