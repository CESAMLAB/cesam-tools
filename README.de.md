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
| [`mock_bin_ru_modbus`](mock_bin_ru_modbus) | **ORME** | Regler (PID / TOR / PWM) auf Übertragungsfunktion | Modbus TCP & RTU (Slave) | egui |
| [`mock_bin_su_namur`](mock_bin_su_namur) | **OSNE** | Labor-Überkopfrührer: Motor-Übertragungsfunktion, schnelle Drehzahlregelung, einstellbare viskose Last | NAMUR über TCP & seriell RS-232 (Slave) | egui |
| [`mock_bin_ru_opcua`](mock_bin_ru_opcua) | **ORUE** | Prozessregler (Anti-Windup-PID) auf Prozess erster Ordnung, mit konfigurierbarer OPC-UA-Sicherheit | OPC UA (Server) | egui |
| [`mock_bin_ru_sparkplugb`](mock_bin_ru_sparkplugb) | **ORSE** | Prozessregler, exponiert als MQTT-Sparkplug-B-Edge-Node (ausgehend) | Sparkplug B / MQTT (Client) | egui |
| [`mock_bin_ru_s7`](mock_bin_ru_s7) | **ORSS** | Prozessregler, exponiert als S7comm-Server über ISO-on-TCP (RFC1006) | S7comm (Server) | egui |
| [`mock_bin_ru_ethernetip`](mock_bin_ru_ethernetip) | **OREE** | Prozessregler, exponiert als EtherNet/IP-Adapter (explizite CIP-Nachrichten) | EtherNet/IP (Adapter) | egui |
| [`mock_bin_ru_pbdp`](mock_bin_ru_pbdp) | **ORPD** | Prozessregler, exponiert als simulierter PROFIBUS-DP-V0-Slave über serielle Verbindung | PROFIBUS DP (Slave, seriell) | egui |

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
- **Persistierte Konfiguration** im TOML-Format (`mock_ru_modbus.toml`),
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

## ORUE — der simulierte OPC-UA-Regler

<p align="center">
  <img src="pic/ru_opcua-logo.svg" alt="ORUE — Open Regulator UA Emulator" height="120">
</p>

> **ORUE** — *Open Regulator UA Emulator*. **„Vereinheitlichen Sie den Prozess."**
> Ein Prozessregler, der nur auf Ihrem OPC-UA-Adressraum existiert.

Ein vollständiger virtueller Prozessregler:

- **Prozess**, modelliert durch eine Übertragungsfunktion erster Ordnung,
  geführt von einem **Anti-Windup-PID**, getaktet alle 0,5 s.
- **OPC-UA-Server** (`async-opcua`, Tokio-nativ, 100 % Rust-Krypto — ohne OpenSSL,
  MPL-2.0-Stack). **Konfigurierbare Sicherheit** (`SecurityConfig`):
  `None`/anonym standardmäßig (sofortiger Start) **oder** `Basic256Sha256` /
  SignAndEncrypt mit selbstsigniertem Zertifikat (`pki/`, beim ersten
  verschlüsselten Lauf erzeugt), dazu anonyme und/oder
  **Benutzer-/Passwort**-Token.
- **Eine Haltung, die sich von ORME/OSNE unterscheidet**: Die OPC-UA-Sicherheit
  beruht auf **Zertifikat + Authentifizierung**, nicht auf einer IP-Whitelist (es
  gibt **keine**); der Server akzeptiert **mehrere gleichzeitige Client-Sitzungen**
  (kein Single-Master, der letzte Schreiber gewinnt). Die Standardeinstellung
  `None`/anonym auf `0.0.0.0:4840` ist die offenste des Workspace — ein
  IHM-Banner warnt, sobald die Verschlüsselung aus ist.
- **Grafische Oberfläche** auf einer Seite: Steuerung, **Trendkurve** in Echtzeit
  und ein **Parameter-Modal** (Netzwerk, Prozess-Übertragungsfunktion, PID-
  Verstärkungen, Sollwertgrenzen, Sicherheit, i18n in 8 Sprachen).
- **Persistierte Konfiguration** im TOML-Format (`mock_ru_opcua.toml`), beim Start
  neu geladen, mit Schaltfläche zum Zurücksetzen auf die Standardwerte.

Er teilt die Architektur von ORME (synchrones Geschäftsmodell, `ractor`-Aktoren,
`egui`-Oberfläche). Starten Sie ihn mit `cargo run -p mock_bin_ru_opcua`; der
OPC-UA-Server lauscht standardmäßig auf `0.0.0.0:4840`. Der Adressraum ist in
[`mock_bin_ru_opcua/docs/de/reference_opcua.md`](mock_bin_ru_opcua/docs/de/reference_opcua.md)
dokumentiert.

## ORSE — der simulierte Sparkplug-B-Edge-Node

<p align="center">
  <img src="pic/ru_spb-logo.svg" alt="ORSE — Open Regulator Sparkplug Emulator" height="120">
</p>

> **ORSE** — *Open Regulator Sparkplug Emulator*.
> Ein Prozessregler, der nur als MQTT-Sparkplug-B-Edge-Node existiert.

Ein vollständiger virtueller Prozessregler, gleiches PID- + Prozessmodell erster Ordnung wie ORME:

- **MQTT-Sparkplug-B-Edge-Node** (ausgehender Client, `rumqttc` + `sparkplug-rs`,
  Eclipse-Tahu-Protobuf, 100 % Rust — ohne `protoc`). Veröffentlicht
  `NBIRTH`/`NDATA` und ein `NDEATH`, das durch das MQTT-**Testament** (*Last
  Will*, robust gegen jeden Verbindungsverlust) getragen wird; reagiert auf
  `NCMD`-Schreibvorgänge des Brokers. `bdSeq`/`seq`-Zähler werden in einer reinen
  Protokollschicht besessen und getestet, nicht an ein Framework delegiert.
- **Eine Haltung, die sich von ORME/OSNE unterscheidet**: Als Client statt als
  Server gibt es **keine IP-Whitelist**. **MQTT standardmäßig im Klartext**
  (Port 1883, unverschlüsselt, ohne Authentifizierung) — ein IHM-Banner warnt,
  solange TLS + Anmeldedaten nicht aktiviert sind, um ein vertrauenswürdiges
  Netzwerk zu verlassen.
- **Grafische Oberfläche** auf einer Seite: Steuerung, **Trendkurve** in
  Echtzeit, und ein **Parameter-Modal** (Broker-Adresse/Anmeldedaten/TLS,
  Prozess-Übertragungsfunktion, PID-Verstärkungen, Sollwertgrenzen, i18n in 8
  Sprachen).
- **Persistierte Konfiguration** im TOML-Format (`mock_ru_sparkplugb.toml`),
  beim Start neu geladen, mit Schaltfläche zum Zurücksetzen auf die
  Standardwerte.

Starten Sie ihn mit `cargo run -p mock_bin_ru_sparkplugb`; er verbindet sich
ausgehend mit dem in *Parameter* konfigurierten Broker (standardmäßig
`localhost:1883`) — kein lauschender Port.

## ORSS — der simulierte S7-Regler

<p align="center">
  <img src="pic/ru_s7-logo.svg" alt="ORSS — Open Regulator S7 Server" height="120">
</p>

> **ORSS** — *Open Regulator S7 Server*.
> Ein Prozessregler, der nur auf Ihrer S7comm-Verbindung existiert.

Ein vollständiger virtueller Prozessregler, gleiches PID- + Prozessmodell erster Ordnung wie ORME:

- **Handgeschriebener S7comm-Server** über ISO-on-TCP (RFC1006), Port 102:
  TPKT-Rahmung, COTP (CR→CC, DT) und S7comm (Setup, Read/Write Var) über ein
  **DB1-Byte-Abbild**. In Rust existiert keine S7-**Server**-Crate (nur
  clientorientierte): die erforderliche Teilmenge wird daher direkt
  implementiert — begrenztes Parsen, keine Panik bei einem fehlerhaften
  Telegramm.
- **Mehrere gleichzeitige Clients akzeptiert** (Verhalten einer echten SPS),
  anders als ORMEs Single-Master-Verdrängungsrichtlinie — der letzte
  Schreiber gewinnt.
- **Ohne Authentifizierung oder Verschlüsselung** („klassisches“ S7): nur die
  **IP-Whitelist** und die Netzwerktopologie schützen den Zugang; ein
  IHM-Banner warnt bei Exposition (`0.0.0.0` + leere Whitelist).
- **Grafische Oberfläche** auf einer Seite: Steuerung, **Trendkurve** in
  Echtzeit, und ein **Parameter-Modal** (Netzwerk, Whitelist,
  Prozess-Übertragungsfunktion, PID-Verstärkungen, Sollwertgrenzen, i18n in 8
  Sprachen).
- **Persistierte Konfiguration** im TOML-Format (`mock_ru_s7.toml`), beim
  Start neu geladen, mit Schaltfläche zum Zurücksetzen auf die Standardwerte.

Starten Sie ihn mit `cargo run -p mock_bin_ru_s7`; der S7comm-Server lauscht
standardmäßig auf `0.0.0.0:102` (Port < 1024 erfordert Root-Rechte).

## OREE — der simulierte EtherNet/IP-Regler

<p align="center">
  <img src="pic/ru_eip-logo.svg" alt="OREE — Open Regulator EtherNet/IP Emulator" height="120">
</p>

> **OREE** — *Open Regulator EtherNet/IP Emulator*.
> Ein Prozessregler, der nur auf Ihrer EtherNet/IP-Verbindung existiert.

Ein vollständiger virtueller Prozessregler, gleiches PID- + Prozessmodell erster Ordnung wie ORME:

- **Handgeschriebener EtherNet/IP-Adapter** (Kapselung `RegisterSession`,
  `SendRRData`/CPF, und CIP `Read Tag`/`Write Tag` nach symbolischem Segment,
  **Little-Endian**), Port 44818. In Rust existiert keine EtherNet/IP-
  **Adapter**-Crate (nur client-/scanner-orientierte): die erforderliche
  Teilmenge wird daher direkt implementiert — begrenztes Parsen, keine Panik
  bei einem fehlerhaften Paket.
- **Mehrere gleichzeitige Clients akzeptiert** (Adapter-Verhalten), anders als
  ORMEs Single-Master-Verdrängungsrichtlinie — jede Sitzung erhält ein
  *Session Handle*, der letzte Schreiber gewinnt.
- **Ohne Authentifizierung oder Verschlüsselung** („klassisches“
  EtherNet/IP): nur die **IP-Whitelist** und die Netzwerktopologie schützen
  den Zugang; ein IHM-Banner warnt bei Exposition.
- **Grafische Oberfläche** auf einer Seite: Steuerung, **Trendkurve** in
  Echtzeit, und ein **Parameter-Modal** (Netzwerk, Whitelist,
  Prozess-Übertragungsfunktion, PID-Verstärkungen, Sollwertgrenzen, i18n in 8
  Sprachen).
- **Persistierte Konfiguration** im TOML-Format (`mock_ru_ethernetip.toml`),
  beim Start neu geladen, mit Schaltfläche zum Zurücksetzen auf die
  Standardwerte.

Starten Sie ihn mit `cargo run -p mock_bin_ru_ethernetip`; der
EtherNet/IP-Adapter lauscht standardmäßig auf `0.0.0.0:44818`.

## ORPD — der simulierte PROFIBUS-DP-Regler

<p align="center">
  <img src="pic/ru_pbdp-logo.svg" alt="ORPD — Open Regulator Profibus DP" height="120">
</p>

> **ORPD** — *Open Regulator Profibus DP*.
> Ein Prozessregler, der nur auf Ihrer PROFIBUS-DP-Verbindung existiert.

Ein vollständiger virtueller Prozessregler, gleiches PID- + Prozessmodell erster Ordnung wie ORME:

- **Software-Simulator von PROFIBUS-DP-V0-Telegrammen** über eine serielle
  Verbindung (RS-485/RS-232): Telegramm-Codec (`SD1`/`SD2`/`SD3`/`SD4`/`SC`,
  FCS) und die Zustandsmaschine des Slaves
  (`Power_On → Wait_Prm → Wait_Cfg → Data_Exchange`). ⚠️ **Nicht
  interoperabel mit realer PROFIBUS-DP-Hardware**: das echte Bus-Timing (Slot
  Time, `Tsdr`) erfordert einen dedizierten ASIC, den dieser reine
  Software-Simulator nicht zu emulieren versucht — siehe
  [`reference_profibus.md`](mock_bin_ru_pbdp/docs/de/reference_profibus.md) §6.
- **Die serielle Verbindung ist der einzige Transport** (kein TCP-Äquivalent
  für PROFIBUS DP, anders als bei ORME/OSNE, wo seriell eine optionale
  Funktion neben einem stets vorhandenen TCP-Transport ist):
  `tokio-serial` ist eine direkte, nicht optionale Abhängigkeit. Keine
  IP-Whitelist (von Natur aus Punkt-zu-Punkt).
- **Protokoll-Watchdog** — ein echter Bestandteil von DP-V0 (durch den
  Master über `Set_Prm` scharfgeschaltet), kein selbstgemachter Zusatz;
  erzwingt bei Ablauf den sicheren Zustand.
- **Grafische Oberfläche** auf einer Seite: Steuerung, **Trendkurve** in
  Echtzeit, ein **Telegramm-Mini-Terminal** (Hex-Protokoll des RX/TX-
  Verkehrs), und ein **Parameter-Modal** (serieller Port, Baudrate,
  Stationsadresse, Prozess-Übertragungsfunktion, PID-Verstärkungen,
  Sollwertgrenzen, i18n in 8 Sprachen).
- **Persistierte Konfiguration** im TOML-Format (`mock_ru_pbdp.toml`), beim
  Start neu geladen, mit Schaltfläche zum Zurücksetzen auf die
  Standardwerte.

Starten Sie ihn mit `cargo run -p mock_bin_ru_pbdp`; er versucht, den
konfigurierten seriellen Port zu öffnen (standardmäßig `/dev/ttyUSB0` oder
`COM3`, 500 kbit/s, Stationsadresse 3).

## Download

Vorkompilierte Binärdateien sind auf der [**Releases**](https://github.com/CESAMLAB/cesam-tools/releases/latest)-Seite verfügbar — **keine Rust-Toolchain erforderlich**. Jedes Instrument liefert seine eigene ausführbare Datei (`orme`, `osne`, `ru_opcua`, `ru_spb`, `ru_s7`, `ru_eip`, `ru_pbdp`).

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

**ORUE** (OPC-UA-Regler):

| Plattform | GUI | Headless (nur TCP, ohne GUI) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`ru_opcua-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-linux-x86_64) | [`ru_opcua-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-linux-x86_64-headless) |
| Windows x86_64 | [`ru_opcua-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`ru_opcua-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-rpi-arm64) | [`ru_opcua-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-rpi-arm64-headless) |

**ORSE** (Sparkplug-B-Edge-Node):

| Plattform | GUI | Headless (nur Client, ohne GUI) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`ru_spb-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_spb-linux-x86_64) | [`ru_spb-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_spb-linux-x86_64-headless) |
| Windows x86_64 | [`ru_spb-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_spb-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`ru_spb-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_spb-rpi-arm64) | [`ru_spb-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_spb-rpi-arm64-headless) |

**ORSS** (S7comm-Regler):

| Plattform | GUI | Headless (nur TCP, ohne GUI) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`ru_s7-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_s7-linux-x86_64) | [`ru_s7-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_s7-linux-x86_64-headless) |
| Windows x86_64 | [`ru_s7-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_s7-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`ru_s7-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_s7-rpi-arm64) | [`ru_s7-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_s7-rpi-arm64-headless) |

**OREE** (EtherNet/IP-Adapter):

| Plattform | GUI | Headless (nur TCP, ohne GUI) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`ru_eip-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_eip-linux-x86_64) | [`ru_eip-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_eip-linux-x86_64-headless) |
| Windows x86_64 | [`ru_eip-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_eip-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`ru_eip-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_eip-rpi-arm64) | [`ru_eip-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_eip-rpi-arm64-headless) |

**ORPD** (PROFIBUS-DP-Regler):

| Plattform | GUI | Headless (serielle Verbindung, ohne GUI) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`ru_pbdp-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_pbdp-linux-x86_64) | [`ru_pbdp-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_pbdp-linux-x86_64-headless) |
| Windows x86_64 | [`ru_pbdp-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_pbdp-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`ru_pbdp-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_pbdp-rpi-arm64) | [`ru_pbdp-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_pbdp-rpi-arm64-headless) |

```bash
chmod +x orme-linux-x86_64        # Linux / Raspberry Pi (genauso für die anderen Instrumente)
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

cargo run -p mock_bin_ru_modbus
```

Das Fenster öffnet sich und der Modbus-TCP-Server lauscht auf `0.0.0.0:5502`.
Der **Port**, die **Lausch-IP** und die **IP-Whitelist** werden im Modal
**⚙ Parameter** eingestellt (im laufenden Betrieb angewandt) und dann in
`mock_ru_modbus.toml` **persistiert**. Die **Sprache der Oberfläche**
(Französisch, Englisch, Deutsch, Spanisch, Italienisch, Portugiesisch,
Niederländisch, Polnisch) wird in demselben Modal gewählt und persistiert. Um eine
andere Konfigurationsdatei zu verwenden:

```bash
MOCK_CONFIG=/pfad/zu/ma_config.toml cargo run -p mock_bin_ru_modbus
```

### Die Modbus-Verbindung testen

Mit einem beliebigen Modbus-Client (z. B. `mbpoll`):

```bash
# Starten (Coil 0), dann den Messwert lesen (Input Registers 0-1, f32)
mbpoll -m tcp -a 1 -t 0 -p 5502 127.0.0.1 1      # Coil On/Off schreiben
mbpoll -m tcp -a 1 -t 3:float -r 1 -p 5502 127.0.0.1   # PV lesen (f32)
```

Die vollständige Adresstabelle ist in
[`mock_bin_ru_modbus/src/map.rs`](mock_bin_ru_modbus/src/map.rs) dokumentiert.

## Entwicklung

```bash
cargo test --workspace      # Unit- + Integrationstests
cargo clippy --workspace    # Lint
```

## Dokumentation

Jedes Instrument trägt seine eigene Dokumentation in seinem `docs/`-Unterordner,
verfügbar in acht Sprachen (`docs/<sprache>/`). Deutsche Versionen:

**ORME** (Modbus-Regler):

- [**Benutzerhandbuch**](mock_bin_ru_modbus/docs/de/manuel_utilisateur.md) — Einstieg, IHM, Parameter, FAQ.
- [Entwurfsdokument](mock_bin_ru_modbus/docs/de/conception.md) — Architektur und technische Entscheidungen.
- [Modbus-Adresstabelle](mock_bin_ru_modbus/docs/de/table_modbus.md) — vollständiger Adressplan.
- [Softwarewartung](mock_bin_ru_modbus/docs/de/maintenance.md) — Build, Konfiguration, Erweiterung, Fehlerbehebung.

**OSNE** (NAMUR-Laborrührer):

- [**Benutzerhandbuch**](mock_bin_su_namur/docs/de/manuel_utilisateur.md) — Einstieg, IHM, NAMUR-Mini-Terminal, Parameter, FAQ.
- [Entwurfsdokument](mock_bin_su_namur/docs/de/conception.md) — Motormodell, Regelkreis, Architektur.
- [NAMUR-Befehlssatz](mock_bin_su_namur/docs/de/commandes_namur.md) — Protokollreferenz (Kanäle, Befehle, Beispiele).
- [Softwarewartung](mock_bin_su_namur/docs/de/maintenance.md) — Build, Konfiguration, Erweiterung, Fehlerbehebung.

**ORUE** (OPC-UA-Regler):

- [**Benutzerhandbuch**](mock_bin_ru_opcua/docs/de/manuel_utilisateur.md) — Einstieg, IHM, Verbindung eines OPC-UA-Clients, FAQ.
- [Entwurfsdokument](mock_bin_ru_opcua/docs/de/conception.md) — PID- + Prozessmodell, Aktorarchitektur, `async-opcua`-Stack, Sicherheit.
- [OPC-UA-Referenz](mock_bin_ru_opcua/docs/de/reference_opcua.md) — Endpunkt, Namespace, Knoten (Lesen/Schreiben, Beispiele).
- [Softwarewartung](mock_bin_ru_opcua/docs/de/maintenance.md) — Build, Konfiguration, Erweiterung, Fehlerbehebung.

**ORSE** (Sparkplug-B-Edge-Node):

- [**Benutzerhandbuch**](mock_bin_ru_sparkplugb/docs/de/manuel_utilisateur.md) — Einstieg, IHM, Broker-Verbindung, FAQ.
- [Entwurfsdokument](mock_bin_ru_sparkplugb/docs/de/conception.md) — Aktorarchitektur, Protokollschicht, Bibliothekswahl.
- [Sparkplug-B-Referenz](mock_bin_ru_sparkplugb/docs/de/reference_sparkplugb.md) — Topics, Metriken, NBIRTH/NDATA/NDEATH, NCMD-Zuordnung.
- [Softwarewartung](mock_bin_ru_sparkplugb/docs/de/maintenance.md) — Build, Konfiguration, Erweiterung, Fehlerbehebung.

**ORSS** (S7comm-Regler):

- [**Benutzerhandbuch**](mock_bin_ru_s7/docs/de/manuel_utilisateur.md) — Einstieg, IHM, Verbindung eines S7-Clients, FAQ.
- [Entwurfsdokument](mock_bin_ru_s7/docs/de/conception.md) — Aktorarchitektur, Protokollschicht, Sitzungsrichtlinie.
- [S7comm-Referenz](mock_bin_ru_s7/docs/de/reference_s7.md) — TPKT/COTP/S7comm-Rahmung, DB1-Abbild, Beispiele.
- [Softwarewartung](mock_bin_ru_s7/docs/de/maintenance.md) — Build, Konfiguration, Erweiterung, Fehlerbehebung.

**OREE** (EtherNet/IP-Adapter):

- [**Benutzerhandbuch**](mock_bin_ru_ethernetip/docs/de/manuel_utilisateur.md) — Einstieg, IHM, Verbindung eines CIP-Clients, FAQ.
- [Entwurfsdokument](mock_bin_ru_ethernetip/docs/de/conception.md) — Aktorarchitektur, Protokollschicht, Sitzungsrichtlinie.
- [EtherNet/IP-Referenz](mock_bin_ru_ethernetip/docs/de/reference_ethernetip.md) — Kapselung, CIP Read/Write Tag, Beispiele.
- [Softwarewartung](mock_bin_ru_ethernetip/docs/de/maintenance.md) — Build, Konfiguration, Erweiterung, Fehlerbehebung.

**ORPD** (PROFIBUS-DP-Regler):

- [**Benutzerhandbuch**](mock_bin_ru_pbdp/docs/de/manuel_utilisateur.md) — Einstieg, IHM, Hinweis zur Nicht-Interoperabilität, FAQ.
- [Entwurfsdokument](mock_bin_ru_pbdp/docs/de/conception.md) — Aktorarchitektur, Protokollschicht, Codec-Entscheidungen.
- [PROFIBUS-DP-V0-Referenz](mock_bin_ru_pbdp/docs/de/reference_profibus.md) — Telegramme, Ablauf, E/A-Blöcke, Watchdog, Beispielsequenz.
- [Softwarewartung](mock_bin_ru_pbdp/docs/de/maintenance.md) — Build, Konfiguration, Erweiterung, Fehlerbehebung.

## Marke & Logos

Die Logos befinden sich in [`pic/`](pic/):

- [`orme-icon.svg`](pic/orme-icon.svg) / `orme-icon.png` — ORME-Symbol (Zifferblatt),
  auch als Fenstersymbol der Anwendung eingebettet.
- [`orme-logo.svg`](pic/orme-logo.svg) — vollständiges ORME-Logo (Symbol + Text).
- [`osne-icon.svg`](pic/osne-icon.svg) / `osne-icon.png` — OSNE-Symbol
  (Rührflügel), auch als OSNE-Fenstersymbol eingebettet.
- [`osne-logo.svg`](pic/osne-logo.svg) — vollständiges OSNE-Logo (Symbol + Text).
- [`ru_opcua-icon.svg`](pic/ru_opcua-icon.svg) / `ru_opcua-icon.png` — ORUE-Symbol
  (Reglerzifferblatt, umschlossen von einem OPC-UA-Knotenring), auch als
  ORUE-Fenstersymbol eingebettet.
- [`ru_opcua-logo.svg`](pic/ru_opcua-logo.svg) — vollständiges ORUE-Logo (Symbol + Text).
- [`ru_spb-icon.svg`](pic/ru_spb-icon.svg) / `ru_spb-icon.png` — ORSE-Symbol
  (Reglerzifferblatt + Sparkplug-Blitz mit unverbundenen Pub/Sub-Knoten), auch als
  ORSE-Fenstersymbol eingebettet.
- [`ru_spb-logo.svg`](pic/ru_spb-logo.svg) — vollständiges ORSE-Logo (Symbol + Text).
- [`ru_s7-icon.svg`](pic/ru_s7-icon.svg) / `ru_s7-icon.png` — ORSS-Symbol
  (Reglerzifferblatt + offenes Baugruppenträger-Rack, S7-Backplane), auch als
  ORSS-Fenstersymbol eingebettet.
- [`ru_s7-logo.svg`](pic/ru_s7-logo.svg) — vollständiges ORSS-Logo (Symbol + Text).
- [`ru_eip-icon.svg`](pic/ru_eip-icon.svg) / `ru_eip-icon.png` — OREE-Symbol
  (Reglerzifferblatt + geschlossener Rautenring, DLR EtherNet/IP), auch als
  OREE-Fenstersymbol eingebettet.
- [`ru_eip-logo.svg`](pic/ru_eip-logo.svg) — vollständiges OREE-Logo (Symbol + Text).
- [`ru_pbdp-icon.svg`](pic/ru_pbdp-icon.svg) / `ru_pbdp-icon.png` — ORPD-Symbol
  (Reglerzifferblatt mit PROFIBUS-DP-Motiv), auch als ORPD-Fenstersymbol
  eingebettet.
- [`ru_pbdp-logo.svg`](pic/ru_pbdp-logo.svg) — vollständiges ORPD-Logo (Symbol + Text).
- [`Logo-CESAM-Couleur-vect.png`](pic/Logo-CESAM-Couleur-vect.png) — CESAM-Lab-Logo.

Jedes Symbol wird aus seinem `*-logo.gen.py`-Skript **generiert**
([`pic/orme-logo.gen.py`](pic/orme-logo.gen.py),
[`pic/osne-logo.gen.py`](pic/osne-logo.gen.py),
[`pic/ru_opcua-logo.gen.py`](pic/ru_opcua-logo.gen.py),
[`pic/ru_spb-logo.gen.py`](pic/ru_spb-logo.gen.py),
[`pic/ru_s7-logo.gen.py`](pic/ru_s7-logo.gen.py),
[`pic/ru_eip-logo.gen.py`](pic/ru_eip-logo.gen.py),
[`pic/ru_pbdp-logo.gen.py`](pic/ru_pbdp-logo.gen.py)). Alle Skripte außer dem
von ORME rastern außerdem ihre `-icon.png` direkt (via Pillow); die
ORME-`.svg` wird anschließend gerastert.

Installieren Sie unter **Wayland** das Taskleistensymbol eines Instruments mit
`scripts/install-desktop.sh [orme|osne|ru_opcua|ru_spb|ru_s7|ru_eip|ru_pbdp]`.

## Lizenz

[MIT](LICENSE) © 2026 CESAM-Lab

Drittanbieter-Komponenten, die in einigen Instrumenten gebündelt sind, werden unter ihren eigenen Lizenzen verteilt (insbesondere der OPC-UA-Stack unter MPL-2.0, der von `mock_bin_ru_opcua` verwendet wird); siehe [NOTICE](NOTICE). Sie ändern nichts an der MIT-Lizenz des cesam-tools-Codes.
