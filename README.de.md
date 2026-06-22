<p align="center">
  <img src="pic/Logo-CESAM-Couleur-vect-card.png" alt="CESAM-Lab" height="84">
</p>

# cesam-tools — CESAM-Lab-Werkzeugkasten

*🌍 [English](README.md) · [Français](README.fr.md) · **Deutsch** · [Español](README.es.md) · [Italiano](README.it.md) · [Português](README.pt.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md)*

<p align="center">
  <a href="https://github.com/CESAMLAB/cesam-tools/releases/latest"><img src="https://img.shields.io/github/v/release/CESAMLAB/cesam-tools?label=release" alt="Latest release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue" alt="License: MIT"></a>
</p>

Rust-Workspace, der die **Werkzeuge von CESAM-Lab** zusammenfasst, beginnend mit
**Simulatoren für Industrieinstrumente**: virtuelle Geräte, die ein realistisches
physikalisches Verhalten nachbilden und über Feldprotokolle kommunizieren.
Nützlich zum Entwickeln, Testen und Vorführen von Leitsystemen, SPSen oder
Gateways **ohne reale Hardware**.

> Kostenlos unter der [MIT](LICENSE)-Lizenz verteilt.

## Verfügbare Instrumente

| Crate | Produkt | Beschreibung | Protokoll | IHM |
|-------|---------|--------------|-----------|-----|
| [`mock_bin_ru_modbustcp`](mock_bin_ru_modbustcp) | **ORME** | Regler (PID / TOR / PWM) auf Übertragungsfunktion | Modbus TCP & RTU (Slave) | egui |
| [`mock_bin_su_namur`](mock_bin_su_namur) | **OSNE** | Labor-Überkopfrührer: Motor-Übertragungsfunktion, schnelle Drehzahlregelung, einstellbare viskose Last | NAMUR über TCP & seriell RS-232 (Slave) | egui |

Geteilte Bibliothek:

| Crate | Beschreibung |
|-------|--------------|
| [`mock_lib_control`](mock_lib_control) | Wiederverwendbare Regelungsbausteine: PID mit Anti-Windup, Zweipunkt mit Hysterese, Prozess 1. Ordnung + reine Totzeit (FOPDT). |

## ORME — der simulierte Regler

<p align="center">
  <img src="pic/orme-logo.svg" alt="ORME — Open Regulator Modbus Emulator" height="120">
</p>

> **ORME** — *Open Regulator Modbus Emulator*. **„Öffnen Sie den Bus."**
> Ein Feldregler, der nur auf Ihrem Modbus-Bus existiert.

Ein vollständiger virtueller Industrieregler:

- **Prozess**, modelliert durch eine Übertragungsfunktion erster Ordnung mit
  reiner Totzeit `K·e^(-Ls) / (1 + T·s)` (typisch für einen Ofen oder ein
  Thermostatbad).
- **Bidirektionale Regelung**: Richtung 1 (heiß) und Richtung 2 (kalt),
  jede konfigurierbar als **PID**, **Zweipunkt (TOR)** oder **Taktrelais (PWM)**.
- **Modi** Start/Stopp und automatisch/manuell.
- **Modbus-Server** in **TCP** oder **seriell RTU / RS485** (Feature `rtu`), nach
  Wahl. Adresstabelle (Sollwert, Messwert, Ausgang, Modi…), im laufenden Betrieb
  konfigurierbare **IP-Whitelist** (Joker `*`) und **Single-Master-Politik** (nur
  ein entfernter Master gleichzeitig; in TCP trennt ein Neuankömmling den vorherigen).
- **Grafische Oberfläche** auf einer Seite: Steuerung, **Trendkurve** in Echtzeit,
  **Live-Modbus-Adresstabelle** und ein **Parameter-Modal** (Transport TCP/RTU,
  Port, erlaubte IPs, serielle Parameter, Übertragungsfunktion, Sollwertgrenzen).
- **Persistierte Konfiguration** im TOML-Format (`mock_ru_modbustcp.toml`),
  beim Start neu geladen, mit Schaltfläche zum Zurücksetzen auf die Standardwerte.

### Asynchrone Architektur

```
        Command (nicht blockierender cast)     geteilter Momentanzustand
  IHM (egui) ──────────────────────►  SimulationActor  ──────────►  IHM (Lesen)
  Modbus Schreiben ────────────────►   (ractor)         ──────────►  Modbus-Abbild
  Modbus Lesen    ◄──────────────────────────────────────  Modbus-Abbild
```

- **`ractor`**: Ein einzelner Aktor besitzt den Reglerzustand; alle Mutationen
  laufen über Nachrichten (keine Sperre auf der Geschäftslogik).
- **`tokio-modbus`**: Modbus-Server TCP und seriell RTU (Trait `Service`).
- **`eframe`/`egui`**: grafische Oberfläche auf dem Haupt-Thread.

## OSNE — der simulierte Laborrührer

<p align="center">
  <img src="pic/osne-logo.svg" alt="OSNE — Open Stirrer NAMUR Emulator" height="120">
</p>

> **OSNE** — *Open Stirrer NAMUR Emulator*.
> Ein Labor-Überkopfrührer (im Stil von IKA), der nur auf Ihrer NAMUR-Verbindung existiert.

Ein vollständiger virtueller Laborrührer:

- **Motor**, modelliert durch eine Dreh-Übertragungsfunktion `J·dω/dt = T − k·η·ω −
  Reibung` (explizites Euler-Verfahren), mit einem **schnellen PID**, der das
  Drehmoment so führt, dass der Drehzahlsollwert erreicht wird.
- **Einstellbare Viskosität** `η`: erhöht das Lastmoment; bei hoher Viskosität
  sättigt der Motor und der Sollwert wird unerreichbar (**Überlast**) — wie bei
  einem echten Rührer.
- **NAMUR-Server** (ASCII-Befehlsprotokoll) über **TCP** (Test ohne Hardware) oder
  **seriell RS-232** (Feature `serial`), mit einem **Watchdog** pro Sitzung
  (`OUT_WD1@<m>`), **Single-Master**-Politik und einer **IP-Whitelist** (TCP).
- **Grafische Oberfläche** auf einer Seite: Drehzahlsollwert, Viskosität, Live-
  **Trendkurve** für Drehzahl/Drehmoment, ein eingebettetes **NAMUR-Mini-Terminal**
  (Rahmen senden/untersuchen mit Befehlsverlauf) und ein **Parameter-Modal**
  (Transport TCP/seriell, Motorparameter, Grenzen, i18n in 8 Sprachen).
- **Persistierte Konfiguration** im TOML-Format (`mock_su_namur.toml`), beim Start
  neu geladen, mit Schaltfläche zum Zurücksetzen auf die Standardwerte.

Er teilt die Architektur von ORME (synchrones Geschäftsmodell, `ractor`-Aktoren,
`egui`-Oberfläche). Starten Sie ihn mit `cargo run -p mock_bin_su_namur`; der
NAMUR-Server lauscht standardmäßig auf `0.0.0.0:4001`.

## Download

Vorkompilierte Binärdateien sind auf der [**Releases**](https://github.com/CESAMLAB/cesam-tools/releases/latest)-Seite verfügbar — **keine Rust-Toolchain erforderlich**. Jedes Instrument liefert seine eigene ausführbare Datei (`orme`, `osne`).

**ORME** (Modbus-Regler):

| Plattform | GUI | Headless (nur TCP, ohne GUI) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`orme-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-linux-x86_64) | [`orme-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-linux-x86_64-headless) |
| Windows x86_64 | [`orme-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`orme-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-rpi-arm64) | [`orme-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-rpi-arm64-headless) |

**OSNE** (NAMUR-Laborrührer):

| Plattform | GUI | Headless (nur TCP, ohne GUI) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`osne-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-linux-x86_64) | [`osne-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-linux-x86_64-headless) |
| Windows x86_64 | [`osne-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`osne-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-rpi-arm64) | [`osne-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-rpi-arm64-headless) |

```bash
chmod +x orme-linux-x86_64        # Linux / Raspberry Pi (genauso für osne-*)
./orme-linux-x86_64
```

Die Linux-/RPi-Binärdateien sind dynamisch mit glibc verknüpft und benötigen für die GUI eine Desktop-Umgebung (X11/Wayland). Installieren Sie unter **Wayland** den Desktop-Eintrag für das Taskleistensymbol: `scripts/install-desktop.sh`. Überprüfen Sie die Integrität mit den veröffentlichten Prüfsummen:

```bash
sha256sum -c SHA256SUMS
```

## Schnellstart

```bash
# Voraussetzung: Rust stable (Edition 2021, >= 1.85).
# Linux-Systemabhängigkeiten für die IHM: libxkbcommon, libwayland/xcb, openGL.

cargo run -p mock_bin_ru_modbustcp
```

Das Fenster öffnet sich und der Modbus-TCP-Server lauscht auf `0.0.0.0:5502`.
Der **Port**, die **Lausch-IP** und die **IP-Whitelist** werden im Modal
**⚙ Parameter** eingestellt (im laufenden Betrieb angewandt) und dann in
`mock_ru_modbustcp.toml` **persistiert**. Die **Sprache der Oberfläche**
(Französisch, Englisch, Deutsch, Spanisch, Italienisch, Portugiesisch,
Niederländisch, Polnisch) wird in demselben Modal gewählt und persistiert. Um eine
andere Konfigurationsdatei zu verwenden:

```bash
MOCK_CONFIG=/pfad/zu/ma_config.toml cargo run -p mock_bin_ru_modbustcp
```

### Die Modbus-Verbindung testen

Mit einem beliebigen Modbus-Client (z. B. `mbpoll`):

```bash
# Starten (Coil 0), dann den Messwert lesen (Input Registers 0-1, f32)
mbpoll -m tcp -a 1 -t 0 -p 5502 127.0.0.1 1      # Coil On/Off schreiben
mbpoll -m tcp -a 1 -t 3:float -r 1 -p 5502 127.0.0.1   # PV lesen (f32)
```

Die vollständige Adresstabelle ist in
[`mock_bin_ru_modbustcp/src/map.rs`](mock_bin_ru_modbustcp/src/map.rs) dokumentiert.

## Entwicklung

```bash
cargo test --workspace      # Unit- + Integrationstests
cargo clippy --workspace    # Lint
```

## Dokumentation

Jedes Instrument trägt seine eigene Dokumentation in seinem `docs/`-Unterordner,
verfügbar in acht Sprachen (`docs/<sprache>/`). Deutsche Versionen:

**ORME** (Modbus-Regler):

- [**Benutzerhandbuch**](mock_bin_ru_modbustcp/docs/de/manuel_utilisateur.md) — Einstieg, IHM, Parameter, FAQ.
- [Entwurfsdokument](mock_bin_ru_modbustcp/docs/de/conception.md) — Architektur und technische Entscheidungen.
- [Modbus-Adresstabelle](mock_bin_ru_modbustcp/docs/de/table_modbus.md) — vollständiger Adressplan.
- [Softwarewartung](mock_bin_ru_modbustcp/docs/de/maintenance.md) — Build, Konfiguration, Erweiterung, Fehlerbehebung.

**OSNE** (NAMUR-Laborrührer):

- [**Benutzerhandbuch**](mock_bin_su_namur/docs/de/manuel_utilisateur.md) — Einstieg, IHM, NAMUR-Mini-Terminal, Parameter, FAQ.
- [Entwurfsdokument](mock_bin_su_namur/docs/de/conception.md) — Motormodell, Regelkreis, Architektur.
- [NAMUR-Befehlssatz](mock_bin_su_namur/docs/de/commandes_namur.md) — Protokollreferenz (Kanäle, Befehle, Beispiele).
- [Softwarewartung](mock_bin_su_namur/docs/de/maintenance.md) — Build, Konfiguration, Erweiterung, Fehlerbehebung.

## Marke & Logos

Die Logos befinden sich in [`pic/`](pic/):

- [`orme-icon.svg`](pic/orme-icon.svg) / `orme-icon.png` — ORME-Symbol (Zifferblatt),
  auch als Fenstersymbol der Anwendung eingebettet.
- [`orme-logo.svg`](pic/orme-logo.svg) — vollständiges ORME-Logo (Symbol + Text).
- [`osne-icon.svg`](pic/osne-icon.svg) / `osne-icon.png` — OSNE-Symbol
  (Rührflügel), auch als OSNE-Fenstersymbol eingebettet.
- [`osne-logo.svg`](pic/osne-logo.svg) — vollständiges OSNE-Logo (Symbol + Text).
- [`Logo-CESAM-Couleur-vect.png`](pic/Logo-CESAM-Couleur-vect.png) — CESAM-Lab-Logo.

Jedes Symbol wird aus seinem `*-logo.gen.py`-Skript **generiert**
([`pic/orme-logo.gen.py`](pic/orme-logo.gen.py),
[`pic/osne-logo.gen.py`](pic/osne-logo.gen.py)). Das OSNE-Skript rastert außerdem
`osne-icon.png` direkt (via Pillow); die ORME-`.svg` wird anschließend gerastert.

Installieren Sie unter **Wayland** das Taskleistensymbol eines Instruments mit
`scripts/install-desktop.sh [orme|osne]`.

## Lizenz

[MIT](LICENSE) © 2026 CESAM-Lab
