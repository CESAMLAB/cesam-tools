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
| [`mock_bin_ru_modbus`](mock_bin_ru_modbus) | **ORME** | Regelaar (PID / TOR / PWM) op overdrachtsfunctie | Modbus TCP & RTU (slave) | egui |
| [`mock_bin_su_namur`](mock_bin_su_namur) | **OSNE** | Bovenroerder voor laboratorium: motoroverdrachtsfunctie, snelle toerenregeling, instelbare viskeuze belasting | NAMUR over TCP & serieel RS-232 (slave) | egui |
| [`mock_bin_ru_opcua`](mock_bin_ru_opcua) | **ORUE** | Procesregelaar (anti-windup-PID) op een eerste-orde-proces, met instelbare OPC UA-beveiliging | OPC UA (server) | egui |
| [`mock_bin_ru_sparkplugb`](mock_bin_ru_sparkplugb) | **ORSE** | Procesregelaar, blootgesteld als uitgaande MQTT Sparkplug B-edge-node | Sparkplug B / MQTT (client) | egui |
| [`mock_bin_ru_s7`](mock_bin_ru_s7) | **ORSS** | Procesregelaar, blootgesteld als S7comm-server over ISO-on-TCP (RFC1006) | S7comm (server) | egui |
| [`mock_bin_ru_ethernetip`](mock_bin_ru_ethernetip) | **OREE** | Procesregelaar, blootgesteld als EtherNet/IP-adapter (expliciete CIP-berichten) | EtherNet/IP (adapter) | egui |
| [`mock_bin_ru_pbdp`](mock_bin_ru_pbdp) | **ORPD** | Procesregelaar, blootgesteld als gesimuleerde PROFIBUS-DP-V0-slave over seriële verbinding | PROFIBUS DP (slave, serieel) | egui |

Gedeelde bibliotheken:

| Crate | Beschrijving |
|-------|--------------|
| [`mock_lib_control`](mock_lib_control) | Herbruikbare regelbouwstenen: PID met anti-windup, aan-uit met hysterese, eerste-orde-proces + zuivere dode tijd (FOPDT). |
| [`mock_lib_regulator`](mock_lib_regulator) | Kant-en-klare PID-regelaar (status, TOML-configuratie, `ractor`-actor), ongewijzigd gedeeld door ORUE, ORSE, ORSS en OREE. |

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
- **Persistente configuratie** in TOML-formaat (`mock_ru_modbus.toml`), herladen
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

<p align="center">
  <img src="pic/osne-logo.svg" alt="OSNE — Open Stirrer NAMUR Emulator" height="120">
</p>

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

## ORUE — de gesimuleerde OPC UA-regelaar

<p align="center">
  <img src="pic/ru_opcua-logo.svg" alt="ORUE — Open Regulator UA Emulator" height="120">
</p>

> **ORUE** — *Open Regulator UA Emulator*. **« Verenig het proces. »**
> Een procesregelaar die alleen bestaat op uw OPC UA-adresruimte.

Een volledige virtuele procesregelaar:

- **Proces** gemodelleerd door een eerste-orde-overdrachtsfunctie aangedreven door
  een **anti-windup-PID**, met een stap elke 0,5 s.
- **OPC UA-server** (`async-opcua`, Tokio-native, 100% Rust-crypto — geen OpenSSL,
  MPL-2.0-stack). **Instelbare beveiliging** (`SecurityConfig`): standaard
  `None`/anoniem (onmiddellijke start) **of** `Basic256Sha256` / SignAndEncrypt met
  een zelfondertekend certificaat (`pki/`, gegenereerd bij de eerste versleutelde
  uitvoering), plus anonieme en/of **gebruikersnaam-/wachtwoord**-tokens.
- **Een houding die verschilt van ORME/OSNE**: de OPC UA-beveiliging steunt op
  **certificaat + authenticatie**, niet op een IP-witte lijst (die is er **niet**);
  de server aanvaardt **meerdere gelijktijdige cliëntsessies** (geen single-master,
  de laatste schrijver wint). De standaard `None`/anoniem op `0.0.0.0:4840` is de
  meest open van de workspace — een GUI-banner waarschuwt zodra de versleuteling
  uitstaat.
- **Grafische interface** op één pagina: besturing, real-time **trendgrafiek**, en
  een **Parameters-modaal** (netwerk, procesoverdrachtsfunctie, PID-versterkingen,
  setpointgrenzen, beveiliging, i18n in 8 talen).
- **Persistente configuratie** in TOML-formaat (`mock_ru_opcua.toml`), herladen bij
  opstart, met knop om terug te zetten naar de standaardwaarden.

Het deelt de architectuur van ORME (synchroon bedrijfsmodel, `ractor`-actoren,
`egui`-GUI). Start het met `cargo run -p mock_bin_ru_opcua`; de OPC UA-server luistert
standaard op `0.0.0.0:4840`. De adresruimte is gedocumenteerd in
[`mock_bin_ru_opcua/docs/nl/reference_opcua.md`](mock_bin_ru_opcua/docs/nl/reference_opcua.md).

## ORSE — de gesimuleerde Sparkplug B-edge-node

<p align="center">
  <img src="pic/ru_spb-logo.svg" alt="ORSE — Open Regulator Sparkplug Emulator" height="120">
</p>

> **ORSE** — *Open Regulator Sparkplug Emulator*.
> Een procesregelaar die alleen bestaat als MQTT Sparkplug B-edge-node.

Een volledige virtuele procesregelaar, hetzelfde PID- + eerste-orde-procesmodel als ORME:

- **MQTT Sparkplug B-edge-node** (uitgaande client, `rumqttc` +
  `sparkplug-rs`, Eclipse Tahu-protobuf, 100% Rust — zonder `protoc`).
  Publiceert `NBIRTH`/`NDATA` en een `NDEATH` gedragen door het MQTT-
  **testament** (*Last Will*, robuust tegen elk verbindingsverlies);
  reageert op `NCMD`-schrijfbewerkingen van de broker. `bdSeq`/`seq`-
  tellers in eigen beheer en getest in een pure protocollaag, niet
  gedelegeerd aan een framework.
- **Een houding die verschilt van ORME/OSNE**: als client in plaats van
  server is er **geen IP-witte lijst**. **Standaard MQTT in platte tekst**
  (poort 1883, onversleuteld, zonder authenticatie) — een GUI-banner
  waarschuwt zolang TLS + inloggegevens niet zijn geactiveerd om een
  vertrouwd netwerk te verlaten.
- **Grafische interface** op één pagina: besturing, real-time
  **trendgrafiek**, en een **Parameters-modaal** (broker-adres/
  inloggegevens/TLS, procesoverdrachtsfunctie, PID-versterkingen,
  setpointgrenzen, i18n in 8 talen).
- **Persistente configuratie** in TOML-formaat (`mock_ru_sparkplugb.toml`),
  herladen bij opstart, met knop om terug te zetten naar de
  standaardwaarden.

Start het met `cargo run -p mock_bin_ru_sparkplugb`; het verbindt uitgaand
met de in *Parameters* geconfigureerde broker (standaard
`localhost:1883`) — geen luisterende poort.

## ORSS — de gesimuleerde S7-regelaar

<p align="center">
  <img src="pic/ru_s7-logo.svg" alt="ORSS — Open Regulator S7 Server" height="120">
</p>

> **ORSS** — *Open Regulator S7 Server*.
> Een procesregelaar die alleen bestaat op uw S7comm-verbinding.

Een volledige virtuele procesregelaar, hetzelfde PID- + eerste-orde-procesmodel als ORME:

- **Handgeschreven S7comm-server** over ISO-on-TCP (RFC1006), poort 102:
  TPKT-framing, COTP (CR→CC, DT) en S7comm (Setup, Read/Write Var) op een
  **DB1-bytebeeld**. Er bestaat geen S7-**server**-crate in Rust (alleen
  client-georiënteerde): de vereiste subset wordt daarom rechtstreeks
  geïmplementeerd — begrensde parsing, geen paniek bij een misvormd
  frame.
- **Meerdere gelijktijdige clients aanvaard** (gedrag van een echte PLC),
  in tegenstelling tot het single-master-verdringingsbeleid van ORME — de
  laatste schrijver wint.
- **Zonder authenticatie of versleuteling** («klassieke» S7): alleen de
  **IP-witte lijst** en de netwerktopologie beschermen de toegang; een
  GUI-banner waarschuwt bij blootstelling (`0.0.0.0` + lege witte lijst).
- **Grafische interface** op één pagina: besturing, real-time
  **trendgrafiek**, en een **Parameters-modaal** (netwerk, witte lijst,
  procesoverdrachtsfunctie, PID-versterkingen, setpointgrenzen, i18n in 8
  talen).
- **Persistente configuratie** in TOML-formaat (`mock_ru_s7.toml`),
  herladen bij opstart, met knop om terug te zetten naar de
  standaardwaarden.

Start het met `cargo run -p mock_bin_ru_s7`; de S7comm-server luistert
standaard op `0.0.0.0:102` (poort < 1024 vereist rootrechten).

## OREE — de gesimuleerde EtherNet/IP-regelaar

<p align="center">
  <img src="pic/ru_eip-logo.svg" alt="OREE — Open Regulator EtherNet/IP Emulator" height="120">
</p>

> **OREE** — *Open Regulator EtherNet/IP Emulator*.
> Een procesregelaar die alleen bestaat op uw EtherNet/IP-verbinding.

Een volledige virtuele procesregelaar, hetzelfde PID- + eerste-orde-procesmodel als ORME:

- **Handgeschreven EtherNet/IP-adapter** (encapsulatie `RegisterSession`,
  `SendRRData`/CPF, en CIP `Read Tag`/`Write Tag` per symbolisch segment,
  **little-endian**), poort 44818. Er bestaat geen EtherNet/IP-
  **adapter**-crate in Rust (alleen client-/scanner-georiënteerde): de
  vereiste subset wordt daarom rechtstreeks geïmplementeerd — begrensde
  parsing, geen paniek bij een misvormd pakket.
- **Meerdere gelijktijdige clients aanvaard** (adaptergedrag), in
  tegenstelling tot het single-master-verdringingsbeleid van ORME — elke
  sessie krijgt een *session handle*, de laatste schrijver wint.
- **Zonder authenticatie of versleuteling** («klassieke» EtherNet/IP):
  alleen de **IP-witte lijst** en de netwerktopologie beschermen de
  toegang; een GUI-banner waarschuwt bij blootstelling.
- **Grafische interface** op één pagina: besturing, real-time
  **trendgrafiek**, en een **Parameters-modaal** (netwerk, witte lijst,
  procesoverdrachtsfunctie, PID-versterkingen, setpointgrenzen, i18n in 8
  talen).
- **Persistente configuratie** in TOML-formaat
  (`mock_ru_ethernetip.toml`), herladen bij opstart, met knop om terug te
  zetten naar de standaardwaarden.

Start het met `cargo run -p mock_bin_ru_ethernetip`; de EtherNet/IP-adapter
luistert standaard op `0.0.0.0:44818`.

## ORPD — de gesimuleerde PROFIBUS-DP-regelaar

<p align="center">
  <img src="pic/ru_pbdp-logo.svg" alt="ORPD — Open Regulator Profibus DP" height="120">
</p>

> **ORPD** — *Open Regulator Profibus DP*.
> Een procesregelaar die alleen bestaat op uw PROFIBUS-DP-verbinding.

Een volledige virtuele procesregelaar, hetzelfde PID- + eerste-orde-procesmodel als ORME:

- **Software-simulator van PROFIBUS-DP-V0-frames** over een seriële
  verbinding (RS-485/RS-232): framecodec (`SD1`/`SD2`/`SD3`/`SD4`/`SC`,
  FCS) en de toestandsmachine van de slave
  (`Power_On → Wait_Prm → Wait_Cfg → Data_Exchange`). ⚠️ **Niet
  interoperabel met echte PROFIBUS-DP-hardware**: de echte bus-timing
  (slot time, `Tsdr`) vereist een dedicated ASIC die deze zuiver
  software-simulator niet probeert te emuleren — zie
  [`reference_profibus.md`](mock_bin_ru_pbdp/docs/nl/reference_profibus.md) §6.
- **De seriële verbinding is het enige transport** (geen TCP-equivalent
  voor PROFIBUS DP, in tegenstelling tot ORME/OSNE waar serieel een
  optionele feature is naast een steeds aanwezig TCP-transport):
  `tokio-serial` is een directe, niet-optionele afhankelijkheid. Geen
  IP-witte lijst (inherent punt-naar-punt).
- **Protocolwatchdog** — een echt onderdeel van DP-V0 (bewapend door de
  master via `Set_Prm`), geen zelfgemaakte toevoeging; dwingt de veilige
  toestand af bij verlopen.
- **Grafische interface** op één pagina: besturing, real-time
  **trendgrafiek**, een **frame-mini-terminal** (hex-log van RX/TX-
  verkeer), en een **Parameters-modaal** (seriële poort, baudrate,
  stationsadres, procesoverdrachtsfunctie, PID-versterkingen,
  setpointgrenzen, i18n in 8 talen).
- **Persistente configuratie** in TOML-formaat (`mock_ru_pbdp.toml`),
  herladen bij opstart, met knop om terug te zetten naar de
  standaardwaarden.

Start het met `cargo run -p mock_bin_ru_pbdp`; het probeert de
geconfigureerde seriële poort te openen (standaard `/dev/ttyUSB0` of
`COM3`, 500 kbit/s, stationsadres 3).

## Downloaden

Voorgecompileerde binaries zijn beschikbaar op de pagina [**Releases**](https://github.com/CESAMLAB/cesam-tools/releases/latest) — **geen Rust-toolchain vereist**. Elk instrument levert zijn eigen uitvoerbaar bestand (`orme`, `osne`, `ru_opcua`, `ru_spb`, `ru_s7`, `ru_eip`, `ru_pbdp`).

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

**ORUE** (OPC UA-regelaar):

| Platform | GUI | Headless (alleen TCP, geen GUI) |
|----------|-----|---------------------------------|
| Linux x86_64 | [`ru_opcua-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-linux-x86_64) | [`ru_opcua-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-linux-x86_64-headless) |
| Windows x86_64 | [`ru_opcua-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`ru_opcua-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-rpi-arm64) | [`ru_opcua-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-rpi-arm64-headless) |

**ORSE** (Sparkplug B-edge-node):

| Platform | GUI | Headless (alleen client, geen GUI) |
|----------|-----|---------------------------------|
| Linux x86_64 | [`ru_spb-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_spb-linux-x86_64) | [`ru_spb-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_spb-linux-x86_64-headless) |
| Windows x86_64 | [`ru_spb-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_spb-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`ru_spb-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_spb-rpi-arm64) | [`ru_spb-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_spb-rpi-arm64-headless) |

**ORSS** (S7comm-regelaar):

| Platform | GUI | Headless (alleen TCP, geen GUI) |
|----------|-----|---------------------------------|
| Linux x86_64 | [`ru_s7-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_s7-linux-x86_64) | [`ru_s7-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_s7-linux-x86_64-headless) |
| Windows x86_64 | [`ru_s7-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_s7-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`ru_s7-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_s7-rpi-arm64) | [`ru_s7-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_s7-rpi-arm64-headless) |

**OREE** (EtherNet/IP-adapter):

| Platform | GUI | Headless (alleen TCP, geen GUI) |
|----------|-----|---------------------------------|
| Linux x86_64 | [`ru_eip-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_eip-linux-x86_64) | [`ru_eip-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_eip-linux-x86_64-headless) |
| Windows x86_64 | [`ru_eip-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_eip-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`ru_eip-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_eip-rpi-arm64) | [`ru_eip-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_eip-rpi-arm64-headless) |

**ORPD** (PROFIBUS-DP-regelaar):

| Platform | GUI | Headless (seriële verbinding, geen GUI) |
|----------|-----|---------------------------------|
| Linux x86_64 | [`ru_pbdp-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_pbdp-linux-x86_64) | [`ru_pbdp-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_pbdp-linux-x86_64-headless) |
| Windows x86_64 | [`ru_pbdp-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_pbdp-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`ru_pbdp-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_pbdp-rpi-arm64) | [`ru_pbdp-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_pbdp-rpi-arm64-headless) |

```bash
chmod +x orme-linux-x86_64        # Linux / Raspberry Pi (idem voor de andere instrumenten)
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

cargo run -p mock_bin_ru_modbus
```

Het venster opent en de Modbus TCP-server luistert op `0.0.0.0:5502`.
De **poort**, het **luister-IP** en de **IP-witte lijst** worden ingesteld in het
**⚙ Parameters**-modaal (tijdens werking toegepast) en vervolgens **persistent
opgeslagen** in `mock_ru_modbus.toml`. De **taal van de interface** (Frans,
Engels, Duits, Spaans, Italiaans, Portugees, Nederlands, Pools) wordt in datzelfde
modaal gekozen en is persistent. Om een ander configuratiebestand te gebruiken:

```bash
MOCK_CONFIG=/pad/naar/ma_config.toml cargo run -p mock_bin_ru_modbus
```

### De Modbus-verbinding testen

Met om het even welke Modbus-client (bv. `mbpoll`):

```bash
# Inschakelen (coil 0) dan de meting lezen (input registers 0-1, f32)
mbpoll -m tcp -a 1 -t 0 -p 5502 127.0.0.1 1      # de On/Off-coil schrijven
mbpoll -m tcp -a 1 -t 3:float -r 1 -p 5502 127.0.0.1   # PV lezen (f32)
```

De volledige adrestabel is gedocumenteerd in
[`mock_bin_ru_modbus/src/map.rs`](mock_bin_ru_modbus/src/map.rs).

## Ontwikkeling

```bash
cargo test --workspace      # unit- + integratietests
cargo clippy --workspace    # lint
```

## Documentatie

Elk instrument draagt zijn eigen documentatie in zijn submap `docs/`, beschikbaar in
acht talen (`docs/<taal>/`). Nederlandse versies:

**ORME** (Modbus-regelaar):

- [**Gebruikershandleiding**](mock_bin_ru_modbus/docs/nl/manuel_utilisateur.md) — ingebruikname, GUI, parameters, FAQ.
- [Ontwerpdocument](mock_bin_ru_modbus/docs/nl/conception.md) — architectuur en technische keuzes.
- [Modbus-adrestabel](mock_bin_ru_modbus/docs/nl/table_modbus.md) — volledig adresseringsplan.
- [Software-onderhoud](mock_bin_ru_modbus/docs/nl/maintenance.md) — build, configuratie, uitbreiding, probleemoplossing.

**OSNE** (NAMUR-laboratoriumroerder):

- [**Gebruikershandleiding**](mock_bin_su_namur/docs/nl/manuel_utilisateur.md) — ingebruikname, GUI, NAMUR-miniterminal, parameters, FAQ.
- [Ontwerpdocument](mock_bin_su_namur/docs/nl/conception.md) — motormodel, regellus, architectuur.
- [NAMUR-commandoset](mock_bin_su_namur/docs/nl/commandes_namur.md) — protocolreferentie (kanalen, commando's, voorbeelden).
- [Software-onderhoud](mock_bin_su_namur/docs/nl/maintenance.md) — build, configuratie, uitbreiding, probleemoplossing.

**ORUE** (OPC UA-regelaar):

- [**Gebruikershandleiding**](mock_bin_ru_opcua/docs/nl/manuel_utilisateur.md) — ingebruikname, GUI, een OPC UA-cliënt verbinden, FAQ.
- [Ontwerpdocument](mock_bin_ru_opcua/docs/nl/conception.md) — PID + procesmodel, architectuur met actoren, `async-opcua`-stack, beveiliging.
- [OPC UA-referentie](mock_bin_ru_opcua/docs/nl/reference_opcua.md) — endpoint, namespace, nodes (lezen/schrijven, voorbeelden).
- [Software-onderhoud](mock_bin_ru_opcua/docs/nl/maintenance.md) — build, configuratie, uitbreiding, probleemoplossing.

**ORSE** (Sparkplug B-edge-node):

- [**Gebruikershandleiding**](mock_bin_ru_sparkplugb/docs/nl/manuel_utilisateur.md) — ingebruikname, GUI, broker-verbinding, FAQ.
- [Ontwerpdocument](mock_bin_ru_sparkplugb/docs/nl/conception.md) — actorarchitectuur, protocollaag, bibliotheekkeuzes.
- [Sparkplug B-referentie](mock_bin_ru_sparkplugb/docs/nl/reference_sparkplugb.md) — topics, metrieken, NBIRTH/NDATA/NDEATH, NCMD-mapping.
- [Software-onderhoud](mock_bin_ru_sparkplugb/docs/nl/maintenance.md) — build, configuratie, uitbreiding, probleemoplossing.

**ORSS** (S7comm-regelaar):

- [**Gebruikershandleiding**](mock_bin_ru_s7/docs/nl/manuel_utilisateur.md) — ingebruikname, GUI, een S7-cliënt verbinden, FAQ.
- [Ontwerpdocument](mock_bin_ru_s7/docs/nl/conception.md) — actorarchitectuur, protocollaag, sessiebeleid.
- [S7comm-referentie](mock_bin_ru_s7/docs/nl/reference_s7.md) — TPKT/COTP/S7comm-framing, DB1-beeld, voorbeelden.
- [Software-onderhoud](mock_bin_ru_s7/docs/nl/maintenance.md) — build, configuratie, uitbreiding, probleemoplossing.

**OREE** (EtherNet/IP-adapter):

- [**Gebruikershandleiding**](mock_bin_ru_ethernetip/docs/nl/manuel_utilisateur.md) — ingebruikname, GUI, een CIP-cliënt verbinden, FAQ.
- [Ontwerpdocument](mock_bin_ru_ethernetip/docs/nl/conception.md) — actorarchitectuur, protocollaag, sessiebeleid.
- [EtherNet/IP-referentie](mock_bin_ru_ethernetip/docs/nl/reference_ethernetip.md) — encapsulatie, CIP Read/Write Tag, voorbeelden.
- [Software-onderhoud](mock_bin_ru_ethernetip/docs/nl/maintenance.md) — build, configuratie, uitbreiding, probleemoplossing.

**ORPD** (PROFIBUS-DP-regelaar):

- [**Gebruikershandleiding**](mock_bin_ru_pbdp/docs/nl/manuel_utilisateur.md) — ingebruikname, GUI, waarschuwing niet-interoperabiliteit, FAQ.
- [Ontwerpdocument](mock_bin_ru_pbdp/docs/nl/conception.md) — actorarchitectuur, protocollaag, codeckeuzes.
- [PROFIBUS-DP-V0-referentie](mock_bin_ru_pbdp/docs/nl/reference_profibus.md) — frames, sequencing, I/O-blokken, watchdog, voorbeeldsequentie.
- [Software-onderhoud](mock_bin_ru_pbdp/docs/nl/maintenance.md) — build, configuratie, uitbreiding, probleemoplossing.

## Merk & logo's

De logo's staan in [`pic/`](pic/):

- [`orme-icon.svg`](pic/orme-icon.svg) / `orme-icon.png` — ORME-pictogram (wijzerplaat),
  ook ingebed als vensterpictogram van de applicatie.
- [`orme-logo.svg`](pic/orme-logo.svg) — volledig ORME-logo (pictogram + tekst).
- [`osne-icon.svg`](pic/osne-icon.svg) / `osne-icon.png` — OSNE-pictogram
  (roerderschoep), ook ingebed als OSNE-vensterpictogram.
- [`osne-logo.svg`](pic/osne-logo.svg) — volledig OSNE-logo (pictogram + tekst).
- [`ru_opcua-icon.svg`](pic/ru_opcua-icon.svg) / `ru_opcua-icon.png` — ORUE-pictogram
  (regelaarwijzerplaat omsloten door een OPC UA-nodering), ook ingebed als
  ORUE-vensterpictogram.
- [`ru_opcua-logo.svg`](pic/ru_opcua-logo.svg) — volledig ORUE-logo (pictogram + tekst).
- [`ru_spb-icon.svg`](pic/ru_spb-icon.svg) / `ru_spb-icon.png` — ORSE-pictogram
  (regelaarwijzerplaat + Sparkplug-bliksemschicht met niet-verbonden pub/sub-
  nodes), ook ingebed als ORSE-vensterpictogram.
- [`ru_spb-logo.svg`](pic/ru_spb-logo.svg) — volledig ORSE-logo (pictogram + tekst).
- [`ru_s7-icon.svg`](pic/ru_s7-icon.svg) / `ru_s7-icon.png` — ORSS-pictogram
  (regelaarwijzerplaat + open rek van vierkante modules, S7-backplane), ook
  ingebed als ORSS-vensterpictogram.
- [`ru_s7-logo.svg`](pic/ru_s7-logo.svg) — volledig ORSS-logo (pictogram + tekst).
- [`ru_eip-icon.svg`](pic/ru_eip-icon.svg) / `ru_eip-icon.png` — OREE-pictogram
  (regelaarwijzerplaat + gesloten ring van ruiten, DLR EtherNet/IP), ook ingebed
  als OREE-vensterpictogram.
- [`ru_eip-logo.svg`](pic/ru_eip-logo.svg) — volledig OREE-logo (pictogram + tekst).
- [`ru_pbdp-icon.svg`](pic/ru_pbdp-icon.svg) / `ru_pbdp-icon.png` — ORPD-pictogram
  (regelaarwijzerplaat met PROFIBUS-DP-motief), ook ingebed als
  ORPD-vensterpictogram.
- [`ru_pbdp-logo.svg`](pic/ru_pbdp-logo.svg) — volledig ORPD-logo (pictogram + tekst).
- [`Logo-CESAM-Couleur-vect.png`](pic/Logo-CESAM-Couleur-vect.png) — CESAM-Lab-logo.

Elk pictogram wordt **gegenereerd** vanuit zijn `*-logo.gen.py`-script
([`pic/orme-logo.gen.py`](pic/orme-logo.gen.py),
[`pic/osne-logo.gen.py`](pic/osne-logo.gen.py),
[`pic/ru_opcua-logo.gen.py`](pic/ru_opcua-logo.gen.py),
[`pic/ru_spb-logo.gen.py`](pic/ru_spb-logo.gen.py),
[`pic/ru_s7-logo.gen.py`](pic/ru_s7-logo.gen.py),
[`pic/ru_eip-logo.gen.py`](pic/ru_eip-logo.gen.py),
[`pic/ru_pbdp-logo.gen.py`](pic/ru_pbdp-logo.gen.py)). Alle scripts behalve dat
van ORME rasteren ook hun `-icon.png` rechtstreeks (via Pillow); de ORME-`.svg`
wordt daarna gerasterd.

Op **Wayland** installeer je het taakbalkpictogram van een instrument met
`scripts/install-desktop.sh [orme|osne|ru_opcua|ru_spb|ru_s7|ru_eip|ru_pbdp]`.

## Licentie

[MIT](LICENSE) © 2026 CESAM-Lab

Componenten van derden die in sommige instrumenten zijn gebundeld, worden onder hun eigen licenties gedistribueerd (met name de OPC UA-stack onder MPL-2.0 die door `mock_bin_ru_opcua` wordt gebruikt); zie [NOTICE](NOTICE). Ze wijzigen de MIT-licentie van de cesam-tools-code niet.
