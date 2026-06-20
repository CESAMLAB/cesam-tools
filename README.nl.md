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

## Downloaden

Voorgecompileerde binaries zijn beschikbaar op de pagina [**Releases**](https://github.com/CESAMLAB/cesam-tools/releases/latest) — **geen Rust-toolchain vereist**.

| Platform | GUI | Headless (alleen TCP, geen GUI) |
|----------|-----|---------------------------------|
| Linux x86_64 | [`orme-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-linux-x86_64) | [`orme-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-linux-x86_64-headless) |
| Windows x86_64 | [`orme-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`orme-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-rpi-arm64) | [`orme-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-rpi-arm64-headless) |

```bash
chmod +x orme-linux-x86_64        # Linux / Raspberry Pi
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

Zie [CLAUDE.md](CLAUDE.md) voor de conventies en de gedetailleerde architectuur.

## Documentatie

Elk instrument draagt zijn eigen documentatie in zijn submap `docs/`, beschikbaar in
acht talen (`docs/<taal>/`). Voor de regelaar (Nederlandse versie):

- [**Gebruikershandleiding**](mock_bin_ru_modbustcp/docs/nl/manuel_utilisateur.md) — ingebruikname, GUI, parameters, FAQ.
- [Ontwerpdocument](mock_bin_ru_modbustcp/docs/nl/conception.md) — architectuur en technische keuzes.
- [Modbus-adrestabel](mock_bin_ru_modbustcp/docs/nl/table_modbus.md) — volledig adresseringsplan.
- [Software-onderhoud](mock_bin_ru_modbustcp/docs/nl/maintenance.md) — build, configuratie, uitbreiding, probleemoplossing.

## Merk & logo's

De logo's staan in [`pic/`](pic/):

- [`orme-icon.svg`](pic/orme-icon.svg) / `orme-icon.png` — ORME-pictogram (wijzerplaat),
  ook ingebed als vensterpictogram van de applicatie.
- [`orme-logo.svg`](pic/orme-logo.svg) — volledig ORME-logo (pictogram + tekst).
- [`Logo-CESAM-Couleur-vect.png`](pic/Logo-CESAM-Couleur-vect.png) — CESAM-Lab-logo.

Het ORME-pictogram wordt **gegenereerd** vanuit
[`pic/orme-logo.gen.py`](pic/orme-logo.gen.py) (`python3 pic/orme-logo.gen.py`
produceert de `.svg`'s, daarna te rasteren).

## Licentie

[MIT](LICENSE) © 2026 CESAM-Lab
