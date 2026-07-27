<p align="center">
  <img src="pic/Logo-CESAM-Couleur-vect-card.png" alt="CESAM-Lab" height="84">
</p>

# cesam-tools — Zestaw narzędzi CESAM-Lab

*🌍 [English](README.md) · [Français](README.fr.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Italiano](README.it.md) · [Português](README.pt.md) · [Nederlands](README.nl.md) · **Polski***

<p align="center">
  <a href="https://github.com/CESAMLAB/cesam-tools/releases/latest"><img src="https://img.shields.io/github/v/release/CESAMLAB/cesam-tools?label=release" alt="Latest release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue" alt="License: MIT"></a>
</p>

Workspace Rust zbierający **narzędzia CESAM-Lab**, zaczynając od **symulatorów
przyrządów przemysłowych**: wirtualnych urządzeń, które odtwarzają realistyczne
zachowanie fizyczne i komunikują się przez protokoły obiektowe. Przydatne do
tworzenia, testowania i demonstrowania systemów nadzoru, sterowników lub bramek
**bez rzeczywistego sprzętu**.

> Rozpowszechniane bezpłatnie na licencji [MIT](LICENSE).

## Dostępne przyrządy

| Crate | Produkt | Opis | Protokół | GUI |
|-------|---------|-------------|-----------|-----|
| [`mock_bin_ru_modbus`](mock_bin_ru_modbus) | **ORME** | Regulator (PID / TOR / PWM) na funkcji przejścia | Modbus TCP & RTU (slave) | egui |
| [`mock_bin_su_namur`](mock_bin_su_namur) | **OSNE** | Mieszadło laboratoryjne nadstawne: funkcja przejścia silnika, szybka regulacja prędkości, regulowane obciążenie lepkościowe | NAMUR po TCP i szeregowo RS-232 (slave) | egui |
| [`mock_bin_ru_opcua`](mock_bin_ru_opcua) | **ORUE** | Regulator obiektowy (PID antynasyceniowy) na procesie pierwszego rzędu, z konfigurowalnym zabezpieczeniem OPC UA | OPC UA (serwer) | egui |
| [`mock_bin_ru_sparkplugb`](mock_bin_ru_sparkplugb) | **ORSE** | Regulator obiektowy udostępniony jako wychodzący węzeł brzegowy MQTT Sparkplug B | Sparkplug B / MQTT (klient) | egui |
| [`mock_bin_ru_s7`](mock_bin_ru_s7) | **ORSS** | Regulator obiektowy udostępniony jako serwer S7comm przez ISO-on-TCP (RFC1006) | S7comm (serwer) | egui |
| [`mock_bin_ru_ethernetip`](mock_bin_ru_ethernetip) | **OREE** | Regulator obiektowy udostępniony jako adapter EtherNet/IP (jawna komunikacja CIP) | EtherNet/IP (adapter) | egui |
| [`mock_bin_ru_pbdp`](mock_bin_ru_pbdp) | **ORPD** | Regulator obiektowy udostępniony jako symulowany slave PROFIBUS DP-V0 na łączu szeregowym | PROFIBUS DP (slave, szeregowy) | egui |

Biblioteki współdzielone:

| Crate | Opis |
|-------|-------------|
| [`mock_lib_control`](mock_lib_control) | Wielokrotnego użytku elementy regulacji: PID antynasyceniowy, dwustawny z histerezą, proces 1. rzędu + czyste opóźnienie (FOPDT). |
| [`mock_lib_regulator`](mock_lib_regulator) | Gotowy do użycia regulator PID (stan, konfiguracja TOML, aktor `ractor`), współdzielony bez zmian przez ORUE, ORSE, ORSS i OREE. |

## ORME — symulowany regulator

<p align="center">
  <img src="pic/orme-logo.svg" alt="ORME — Open Regulator Modbus Emulator" height="120">
</p>

> **ORME** — *Open Regulator Modbus Emulator*. **„Otwórz magistralę.”**
> Regulator obiektowy, który istnieje tylko na Twojej magistrali Modbus.

Kompletny wirtualny regulator przemysłowy:

- **Proces** modelowany funkcją przejścia pierwszego rzędu z czystym opóźnieniem
  `K·e^(-Ls) / (1 + T·s)` (typowy dla pieca lub łaźni termostatycznej).
- **Regulacja** dwukierunkowa: kierunek 1 (grzanie) i kierunek 2 (chłodzenie),
  każdy konfigurowalny jako **PID**, **dwustawny (TOR)** lub **przekaźnik cyklowy (PWM)**.
- **Tryby** start/stop oraz automatyczny/ręczny.
- **Serwer Modbus** w **TCP** lub **RTU szeregowym / RS485** (feature `rtu`), do wyboru.
  Tablica adresów (nastawa, pomiar, wyjście, tryby…), **biała lista IP**
  (znaki wieloznaczne `*`) konfigurowalna na gorąco oraz **polityka jednego mastera**
  (tylko jeden zdalny master naraz; w TCP nowo przybyły odłącza poprzedniego).
- **Interfejs graficzny** na jednej stronie: sterowanie, **wykres trendu**
  w czasie rzeczywistym, **tablica adresów Modbus na żywo** oraz **modal Parametry**
  (transport TCP/RTU, port, dozwolone IP, parametry szeregowe, funkcja
  przejścia, granice nastawy).
- **Konfiguracja utrwalana** w formacie TOML (`mock_ru_modbus.toml`),
  przeładowywana przy starcie, z przyciskiem przywracania wartości domyślnych.

### Architektura asynchroniczna

```
        Command (cast nieblokujący)            współdzielony chwilowy
  GUI (egui) ──────────────────────►  SimulationActor  ──────────►  GUI (odczyt)
  Modbus zapis ────────────────────►   (ractor)         ──────────►  obraz Modbus
  Modbus odczyt ◄──────────────────────────────────────  obraz Modbus
```

- **`ractor`**: jeden jedyny aktor posiada stan regulatora; wszystkie mutacje
  przechodzą przez komunikaty (brak blokady na logice biznesowej).
- **`tokio-modbus`**: serwer Modbus TCP i RTU szeregowy (trait `Service`).
- **`eframe`/`egui`**: interfejs graficzny na wątku głównym.

## OSNE — symulowane mieszadło laboratoryjne

<p align="center">
  <img src="pic/osne-logo.svg" alt="OSNE — Open Stirrer NAMUR Emulator" height="120">
</p>

> **OSNE** — *Open Stirrer NAMUR Emulator*.
> Laboratoryjne mieszadło nadstawne (w stylu IKA), które istnieje tylko na Twoim
> łączu NAMUR.

Kompletne wirtualne mieszadło laboratoryjne:

- **Silnik** modelowany obrotową funkcją przejścia `J·dω/dt = T − k·η·ω −
  tarcie` (jawny Euler), z **szybkim PID** sterującym momentem, aby śledzić
  nastawę prędkości.
- **Regulowana lepkość** `η`: zwiększa moment obciążenia; przy wysokiej lepkości
  silnik się nasyca, a nastawa staje się nieosiągalna (**przeciążenie**) — jak
  w prawdziwym mieszadle.
- **Serwer NAMUR** (protokół poleceń ASCII) po **TCP** (test bez sprzętu) lub
  **szeregowo RS-232** (feature `serial`), z **watchdogiem** na sesję
  (`OUT_WD1@<m>`), **polityką jednego mastera** oraz **białą listą IP** (TCP).
- **Interfejs graficzny** na jednej stronie: nastawa prędkości, lepkość,
  **wykres trendu** prędkości/momentu na żywo, wbudowany **mini-terminal NAMUR**
  (wysyłanie/inspekcja ramek z historią poleceń) oraz **modal Parametry**
  (transport TCP/szeregowy, parametry silnika, granice, i18n w 8 językach).
- **Konfiguracja utrwalana** w formacie TOML (`mock_su_namur.toml`),
  przeładowywana przy starcie, z przyciskiem przywracania wartości domyślnych.

Współdzieli architekturę ORME (synchroniczny model biznesowy, aktory `ractor`,
GUI `egui`). Uruchom go poleceniem `cargo run -p mock_bin_su_namur`; serwer NAMUR
domyślnie nasłuchuje na `0.0.0.0:4001`.

## ORUE — symulowany regulator OPC UA

<p align="center">
  <img src="pic/ru_opcua-logo.svg" alt="ORUE — Open Regulator UA Emulator" height="120">
</p>

> **ORUE** — *Open Regulator UA Emulator*. **„Zjednocz proces.”**
> Regulator obiektowy, który istnieje tylko w Twojej przestrzeni adresowej OPC UA.

Kompletny wirtualny regulator obiektowy:

- **Proces** modelowany funkcją przejścia pierwszego rzędu sterowaną
  **PID antynasyceniowym**, taktowany co 0,5 s.
- **Serwer OPC UA** (`async-opcua`, natywny dla Tokio, kryptografia w 100% w Rust
  — bez OpenSSL, stos na licencji MPL-2.0). **Konfigurowalne zabezpieczenie**
  (`SecurityConfig`): `None`/anonimowe domyślnie (natychmiastowy start) **lub**
  `Basic256Sha256` / SignAndEncrypt z certyfikatem samopodpisanym (`pki/`,
  generowanym przy pierwszym uruchomieniu szyfrowanym), wraz z tokenami
  anonimowymi i/lub **użytkownik/hasło**.
- **Postawa odmienna od ORME/OSNE**: zabezpieczenie OPC UA opiera się na
  **certyfikacie + uwierzytelnianiu**, a nie na białej liście IP (której **nie ma**);
  serwer akceptuje **kilka równoczesnych sesji klientów** (brak jednego mastera,
  wygrywa ostatni zapisujący). Domyślne `None`/anonimowe na `0.0.0.0:4840` jest
  najbardziej otwarte w całym workspace — baner GUI ostrzega, gdy szyfrowanie jest
  wyłączone.
- **Interfejs graficzny** na jednej stronie: sterowanie, **wykres trendu**
  w czasie rzeczywistym oraz **modal Parametry** (sieć, funkcja przejścia procesu,
  wzmocnienia PID, granice nastawy, zabezpieczenie, i18n w 8 językach).
- **Konfiguracja utrwalana** w formacie TOML (`mock_ru_opcua.toml`),
  przeładowywana przy starcie, z przyciskiem przywracania wartości domyślnych.

Współdzieli architekturę ORME (synchroniczny model biznesowy, aktory `ractor`,
GUI `egui`). Uruchom go poleceniem `cargo run -p mock_bin_ru_opcua`; serwer OPC UA
domyślnie nasłuchuje na `0.0.0.0:4840`. Przestrzeń adresowa jest udokumentowana w
[`mock_bin_ru_opcua/docs/pl/reference_opcua.md`](mock_bin_ru_opcua/docs/pl/reference_opcua.md).

## ORSE — symulowany węzeł brzegowy Sparkplug B

<p align="center">
  <img src="pic/ru_spb-logo.svg" alt="ORSE — Open Regulator Sparkplug Emulator" height="120">
</p>

> **ORSE** — *Open Regulator Sparkplug Emulator*.
> Regulator obiektowy, który istnieje tylko jako węzeł brzegowy MQTT Sparkplug B.

Kompletny wirtualny regulator obiektowy, ten sam model PID + proces pierwszego rzędu co ORME:

- **Węzeł brzegowy MQTT Sparkplug B** (klient wychodzący, `rumqttc` +
  `sparkplug-rs`, protobuf Eclipse Tahu, 100% Rust — bez `protoc`).
  Publikuje `NBIRTH`/`NDATA` oraz `NDEATH` niesiony przez **testament MQTT**
  (*Last Will*, odporny na dowolną utratę łącza); reaguje na zapisy `NCMD`
  z brokera. Liczniki `bdSeq`/`seq` posiadane i testowane w czystej warstwie
  protokołu, nie delegowane do frameworka.
- **Postawa odmienna od ORME/OSNE**: będąc klientem, a nie serwerem, **brak
  białej listy IP**. **MQTT w postaci jawnej domyślnie** (port 1883, bez
  szyfrowania, bez uwierzytelniania) — baner GUI ostrzega, dopóki TLS +
  poświadczenia nie zostaną włączone, aby opuścić zaufaną sieć.
- **Interfejs graficzny** na jednej stronie: sterowanie, **wykres trendu**
  w czasie rzeczywistym, oraz **modal Parametry** (adres/poświadczenia/TLS
  brokera, funkcja przejścia procesu, wzmocnienia PID, granice nastawy,
  i18n w 8 językach).
- **Konfiguracja utrwalana** w formacie TOML (`mock_ru_sparkplugb.toml`),
  przeładowywana przy starcie, z przyciskiem przywracania wartości
  domyślnych.

Uruchom go poleceniem `cargo run -p mock_bin_ru_sparkplugb`; łączy się
wychodząco z brokerem skonfigurowanym w *Parametrach*
(`localhost:1883` domyślnie) — bez portu nasłuchu.

## ORSS — symulowany regulator S7

<p align="center">
  <img src="pic/ru_s7-logo.svg" alt="ORSS — Open Regulator S7 Server" height="120">
</p>

> **ORSS** — *Open Regulator S7 Server*.
> Regulator obiektowy, który istnieje tylko na Twoim łączu S7comm.

Kompletny wirtualny regulator obiektowy, ten sam model PID + proces pierwszego rzędu co ORME:

- **Ręcznie napisany serwer S7comm** przez ISO-on-TCP (RFC1006), port 102:
  ramkowanie TPKT, COTP (CR→CC, DT) i S7comm (Setup, Read/Write Var) na
  **obrazie bajtów DB1**. W Rust nie istnieje żaden crate **serwera** S7
  (tylko zorientowane na klienta): wymagany podzbiór jest więc
  zaimplementowany bezpośrednio — ograniczone parsowanie, brak paniki przy
  zniekształconej ramce.
- **Akceptowanych jest wielu równoczesnych klientów** (zachowanie
  prawdziwego sterownika PLC), w przeciwieństwie do polityki jednego
  mastera z wyparciem w ORME — wygrywa ostatni zapisujący.
- **Bez uwierzytelniania ani szyfrowania** (S7 „klasyczny”): tylko **biała
  lista IP** i topologia sieci chronią dostęp; baner GUI ostrzega w
  przypadku ekspozycji (`0.0.0.0` + pusta biała lista).
- **Interfejs graficzny** na jednej stronie: sterowanie, **wykres trendu**
  w czasie rzeczywistym, oraz **modal Parametry** (sieć, biała lista,
  funkcja przejścia procesu, wzmocnienia PID, granice nastawy, i18n w 8
  językach).
- **Konfiguracja utrwalana** w formacie TOML (`mock_ru_s7.toml`),
  przeładowywana przy starcie, z przyciskiem przywracania wartości
  domyślnych.

Uruchom go poleceniem `cargo run -p mock_bin_ru_s7`; serwer S7comm
domyślnie nasłuchuje na `0.0.0.0:102` (port < 1024 wymaga uprawnień
roota).

## OREE — symulowany regulator EtherNet/IP

<p align="center">
  <img src="pic/ru_eip-logo.svg" alt="OREE — Open Regulator EtherNet/IP Emulator" height="120">
</p>

> **OREE** — *Open Regulator EtherNet/IP Emulator*.
> Regulator obiektowy, który istnieje tylko na Twoim łączu EtherNet/IP.

Kompletny wirtualny regulator obiektowy, ten sam model PID + proces pierwszego rzędu co ORME:

- **Ręcznie napisany adapter EtherNet/IP** (enkapsulacja
  `RegisterSession`, `SendRRData`/CPF, oraz CIP `Read Tag`/`Write Tag`
  według segmentu symbolicznego, **little-endian**), port 44818. W Rust
  nie istnieje żaden crate **adaptera** EtherNet/IP (tylko zorientowane na
  klienta/skaner): wymagany podzbiór jest więc zaimplementowany
  bezpośrednio — ograniczone parsowanie, brak paniki przy zniekształconym
  pakiecie.
- **Akceptowanych jest wielu równoczesnych klientów** (zachowanie
  adaptera), w przeciwieństwie do polityki jednego mastera z wyparciem w
  ORME — każda sesja otrzymuje *session handle*, wygrywa ostatni
  zapisujący.
- **Bez uwierzytelniania ani szyfrowania** (EtherNet/IP „klasyczny”):
  tylko **biała lista IP** i topologia sieci chronią dostęp; baner GUI
  ostrzega w przypadku ekspozycji.
- **Interfejs graficzny** na jednej stronie: sterowanie, **wykres trendu**
  w czasie rzeczywistym, oraz **modal Parametry** (sieć, biała lista,
  funkcja przejścia procesu, wzmocnienia PID, granice nastawy, i18n w 8
  językach).
- **Konfiguracja utrwalana** w formacie TOML
  (`mock_ru_ethernetip.toml`), przeładowywana przy starcie, z przyciskiem
  przywracania wartości domyślnych.

Uruchom go poleceniem `cargo run -p mock_bin_ru_ethernetip`; adapter
EtherNet/IP domyślnie nasłuchuje na `0.0.0.0:44818`.

## ORPD — symulowany regulator PROFIBUS DP

<p align="center">
  <img src="pic/ru_pbdp-logo.svg" alt="ORPD — Open Regulator Profibus DP" height="120">
</p>

> **ORPD** — *Open Regulator Profibus DP*.
> Regulator obiektowy, który istnieje tylko na Twoim łączu PROFIBUS DP.

Kompletny wirtualny regulator obiektowy, ten sam model PID + proces pierwszego rzędu co ORME:

- **Symulator programowy ramek PROFIBUS DP-V0** na łączu szeregowym
  (RS-485/RS-232): kodek ramek (`SD1`/`SD2`/`SD3`/`SD4`/`SC`, FCS) oraz
  maszyna stanów slave'a
  (`Power_On → Wait_Prm → Wait_Cfg → Data_Exchange`). ⚠️ **Niekompatybilny
  z rzeczywistym sprzętem PROFIBUS DP**: rzeczywiste czasowanie magistrali
  (slot time, `Tsdr`) wymaga dedykowanego ASIC, którego ten czysto
  programowy symulator nie próbuje emulować — zob.
  [`reference_profibus.md`](mock_bin_ru_pbdp/docs/pl/reference_profibus.md) §6.
- **Łącze szeregowe jest jedynym transportem** (brak odpowiednika TCP dla
  PROFIBUS DP, w przeciwieństwie do ORME/OSNE, gdzie łącze szeregowe jest
  opcjonalną funkcją obok zawsze obecnego transportu TCP): `tokio-serial`
  jest bezpośrednią, nieopcjonalną zależnością. Brak białej listy IP
  (z natury punkt-punkt).
- **Watchdog protokołu** — prawdziwa część DP-V0 (uzbrajana przez mastera
  przez `Set_Prm`), a nie domowy dodatek; wymusza stan bezpieczny po
  wygaśnięciu.
- **Interfejs graficzny** na jednej stronie: sterowanie, **wykres trendu**
  w czasie rzeczywistym, **mini-terminal ramek** (dziennik szesnastkowy
  ruchu RX/TX), oraz **modal Parametry** (port szeregowy, prędkość
  transmisji, adres stacji, funkcja przejścia procesu, wzmocnienia PID,
  granice nastawy, i18n w 8 językach).
- **Konfiguracja utrwalana** w formacie TOML (`mock_ru_pbdp.toml`),
  przeładowywana przy starcie, z przyciskiem przywracania wartości
  domyślnych.

Uruchom go poleceniem `cargo run -p mock_bin_ru_pbdp`; próbuje otworzyć
skonfigurowany port szeregowy (domyślnie `/dev/ttyUSB0` lub `COM3`,
500 kbit/s, adres stacji 3).

## Pobieranie

Gotowe pliki binarne są dostępne na stronie [**Releases**](https://github.com/CESAMLAB/cesam-tools/releases/latest) — **bez potrzeby instalowania narzędzi Rust**. Każdy przyrząd dostarcza własny plik wykonywalny (`orme`, `osne`, `ru_opcua`, `ru_spb`, `ru_s7`, `ru_eip`, `ru_pbdp`).

**ORME** (regulator Modbus):

| Platforma | GUI | Headless (tylko TCP, bez GUI) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`orme-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-linux-x86_64) | [`orme-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-linux-x86_64-headless) |
| Windows x86_64 | [`orme-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`orme-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-rpi-arm64) | [`orme-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-rpi-arm64-headless) |

**OSNE** (mieszadło laboratoryjne NAMUR):

| Platforma | GUI | Headless (tylko TCP, bez GUI) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`osne-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-linux-x86_64) | [`osne-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-linux-x86_64-headless) |
| Windows x86_64 | [`osne-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`osne-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-rpi-arm64) | [`osne-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-rpi-arm64-headless) |

**ORUE** (regulator OPC UA):

| Platforma | GUI | Headless (tylko TCP, bez GUI) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`ru_opcua-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-linux-x86_64) | [`ru_opcua-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-linux-x86_64-headless) |
| Windows x86_64 | [`ru_opcua-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`ru_opcua-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-rpi-arm64) | [`ru_opcua-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-rpi-arm64-headless) |

**ORSE** (węzeł brzegowy Sparkplug B):

| Platforma | GUI | Headless (tylko klient, bez GUI) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`ru_spb-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_spb-linux-x86_64) | [`ru_spb-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_spb-linux-x86_64-headless) |
| Windows x86_64 | [`ru_spb-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_spb-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`ru_spb-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_spb-rpi-arm64) | [`ru_spb-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_spb-rpi-arm64-headless) |

**ORSS** (regulator S7comm):

| Platforma | GUI | Headless (tylko TCP, bez GUI) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`ru_s7-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_s7-linux-x86_64) | [`ru_s7-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_s7-linux-x86_64-headless) |
| Windows x86_64 | [`ru_s7-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_s7-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`ru_s7-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_s7-rpi-arm64) | [`ru_s7-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_s7-rpi-arm64-headless) |

**OREE** (adapter EtherNet/IP):

| Platforma | GUI | Headless (tylko TCP, bez GUI) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`ru_eip-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_eip-linux-x86_64) | [`ru_eip-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_eip-linux-x86_64-headless) |
| Windows x86_64 | [`ru_eip-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_eip-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`ru_eip-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_eip-rpi-arm64) | [`ru_eip-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_eip-rpi-arm64-headless) |

**ORPD** (regulator PROFIBUS DP):

| Platforma | GUI | Headless (łącze szeregowe, bez GUI) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`ru_pbdp-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_pbdp-linux-x86_64) | [`ru_pbdp-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_pbdp-linux-x86_64-headless) |
| Windows x86_64 | [`ru_pbdp-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_pbdp-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`ru_pbdp-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_pbdp-rpi-arm64) | [`ru_pbdp-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_pbdp-rpi-arm64-headless) |

```bash
chmod +x orme-linux-x86_64        # Linux / Raspberry Pi (to samo dla pozostałych przyrządów)
./orme-linux-x86_64
```

Pliki binarne dla Linux/RPi są dynamicznie linkowane z glibc i wymagają środowiska graficznego (X11/Wayland) dla GUI. W systemie **Wayland** zainstaluj wpis pulpitu, aby uzyskać ikonę na pasku zadań: `scripts/install-desktop.sh`. Sprawdź integralność za pomocą opublikowanych sum kontrolnych:

```bash
sha256sum -c SHA256SUMS
```

## Szybki start

```bash
# Wymagania: Rust stable (edycja 2021, >= 1.85).
# Zależności systemowe Linux dla GUI: libxkbcommon, libwayland/xcb, openGL.

cargo run -p mock_bin_ru_modbus
```

Okno otwiera się, a serwer Modbus TCP nasłuchuje na `0.0.0.0:5502`.
**Port**, **IP nasłuchu** oraz **biała lista IP** ustawia się w modalu
**⚙ Parametry** (stosowane na gorąco), a następnie są **utrwalane** w
`mock_ru_modbus.toml`. **Język interfejsu** (francuski, angielski,
niemiecki, hiszpański, włoski, portugalski, niderlandzki, polski) wybiera się w tym
samym modalu i jest utrwalany. Aby użyć innego pliku konfiguracyjnego:

```bash
MOCK_CONFIG=/sciezka/do/ma_config.toml cargo run -p mock_bin_ru_modbus
```

### Testowanie połączenia Modbus

Dowolnym klientem Modbus (np. `mbpoll`):

```bash
# Uruchomić (coil 0), następnie odczytać pomiar (input registers 0-1, f32)
mbpoll -m tcp -a 1 -t 0 -p 5502 127.0.0.1 1      # zapisać coil On/Off
mbpoll -m tcp -a 1 -t 3:float -r 1 -p 5502 127.0.0.1   # odczytać PV (f32)
```

Pełna tablica adresów jest udokumentowana w
[`mock_bin_ru_modbus/src/map.rs`](mock_bin_ru_modbus/src/map.rs).

## Rozwój

```bash
cargo test --workspace      # testy jednostkowe + integracyjne
cargo clippy --workspace    # lint
```

## Dokumentacja

Każdy przyrząd ma własną dokumentację w swoim podkatalogu `docs/`,
dostępną w ośmiu językach (`docs/<język>/`). Wersje polskie:

**ORME** (regulator Modbus):

- [**Podręcznik użytkownika**](mock_bin_ru_modbus/docs/pl/manuel_utilisateur.md) — wprowadzenie, GUI, parametry, FAQ.
- [Dokument projektowy](mock_bin_ru_modbus/docs/pl/conception.md) — architektura i decyzje techniczne.
- [Tablica adresów Modbus](mock_bin_ru_modbus/docs/pl/table_modbus.md) — pełny plan adresowania.
- [Utrzymanie oprogramowania](mock_bin_ru_modbus/docs/pl/maintenance.md) — build, konfiguracja, rozszerzanie, rozwiązywanie problemów.

**OSNE** (mieszadło laboratoryjne NAMUR):

- [**Podręcznik użytkownika**](mock_bin_su_namur/docs/pl/manuel_utilisateur.md) — wprowadzenie, GUI, mini-terminal NAMUR, parametry, FAQ.
- [Dokument projektowy](mock_bin_su_namur/docs/pl/conception.md) — model silnika, pętla regulacji, architektura.
- [Zestaw poleceń NAMUR](mock_bin_su_namur/docs/pl/commandes_namur.md) — opis protokołu (kanały, polecenia, przykłady).
- [Utrzymanie oprogramowania](mock_bin_su_namur/docs/pl/maintenance.md) — build, konfiguracja, rozszerzanie, rozwiązywanie problemów.

**ORUE** (regulator OPC UA):

- [**Podręcznik użytkownika**](mock_bin_ru_opcua/docs/pl/manuel_utilisateur.md) — wprowadzenie, GUI, podłączanie klienta OPC UA, FAQ.
- [Dokument projektowy](mock_bin_ru_opcua/docs/pl/conception.md) — model PID + procesu, architektura aktorów, stos `async-opcua`, zabezpieczenie.
- [Referencja OPC UA](mock_bin_ru_opcua/docs/pl/reference_opcua.md) — endpoint, przestrzeń nazw, węzły (odczyty/zapisy, przykłady).
- [Utrzymanie oprogramowania](mock_bin_ru_opcua/docs/pl/maintenance.md) — build, konfiguracja, rozszerzanie, rozwiązywanie problemów.

**ORSE** (węzeł brzegowy Sparkplug B):

- [**Podręcznik użytkownika**](mock_bin_ru_sparkplugb/docs/pl/manuel_utilisateur.md) — wprowadzenie, GUI, połączenie z brokerem, FAQ.
- [Dokument projektowy](mock_bin_ru_sparkplugb/docs/pl/conception.md) — architektura aktorów, warstwa protokołu, wybór bibliotek.
- [Referencja Sparkplug B](mock_bin_ru_sparkplugb/docs/pl/reference_sparkplugb.md) — topics, metryki, NBIRTH/NDATA/NDEATH, mapowanie NCMD.
- [Utrzymanie oprogramowania](mock_bin_ru_sparkplugb/docs/pl/maintenance.md) — build, konfiguracja, rozszerzanie, rozwiązywanie problemów.

**ORSS** (regulator S7comm):

- [**Podręcznik użytkownika**](mock_bin_ru_s7/docs/pl/manuel_utilisateur.md) — wprowadzenie, GUI, podłączanie klienta S7, FAQ.
- [Dokument projektowy](mock_bin_ru_s7/docs/pl/conception.md) — architektura aktorów, warstwa protokołu, polityka sesji.
- [Referencja S7comm](mock_bin_ru_s7/docs/pl/reference_s7.md) — ramkowanie TPKT/COTP/S7comm, obraz DB1, przykłady.
- [Utrzymanie oprogramowania](mock_bin_ru_s7/docs/pl/maintenance.md) — build, konfiguracja, rozszerzanie, rozwiązywanie problemów.

**OREE** (adapter EtherNet/IP):

- [**Podręcznik użytkownika**](mock_bin_ru_ethernetip/docs/pl/manuel_utilisateur.md) — wprowadzenie, GUI, podłączanie klienta CIP, FAQ.
- [Dokument projektowy](mock_bin_ru_ethernetip/docs/pl/conception.md) — architektura aktorów, warstwa protokołu, polityka sesji.
- [Referencja EtherNet/IP](mock_bin_ru_ethernetip/docs/pl/reference_ethernetip.md) — enkapsulacja, CIP Read/Write Tag, przykłady.
- [Utrzymanie oprogramowania](mock_bin_ru_ethernetip/docs/pl/maintenance.md) — build, konfiguracja, rozszerzanie, rozwiązywanie problemów.

**ORPD** (regulator PROFIBUS DP):

- [**Podręcznik użytkownika**](mock_bin_ru_pbdp/docs/pl/manuel_utilisateur.md) — wprowadzenie, GUI, ostrzeżenie o braku interoperacyjności, FAQ.
- [Dokument projektowy](mock_bin_ru_pbdp/docs/pl/conception.md) — architektura aktorów, warstwa protokołu, decyzje dotyczące kodeka.
- [Referencja PROFIBUS DP-V0](mock_bin_ru_pbdp/docs/pl/reference_profibus.md) — ramki, sekwencjonowanie, bloki I/O, watchdog, przykładowa sekwencja.
- [Utrzymanie oprogramowania](mock_bin_ru_pbdp/docs/pl/maintenance.md) — build, konfiguracja, rozszerzanie, rozwiązywanie problemów.

## Marka i logo

Logo znajdują się w [`pic/`](pic/):

- [`orme-icon.svg`](pic/orme-icon.svg) / `orme-icon.png` — ikona ORME (tarcza),
  również osadzona jako ikona okna aplikacji.
- [`orme-logo.svg`](pic/orme-logo.svg) — pełne logo ORME (ikona + tekst).
- [`osne-icon.svg`](pic/osne-icon.svg) / `osne-icon.png` — ikona OSNE (wirnik
  mieszadła), również osadzona jako ikona okna OSNE.
- [`osne-logo.svg`](pic/osne-logo.svg) — pełne logo OSNE (ikona + tekst).
- [`ru_opcua-icon.svg`](pic/ru_opcua-icon.svg) / `ru_opcua-icon.png` — ikona ORUE
  (tarcza regulatora otoczona pierścieniem węzła OPC UA), również osadzona jako
  ikona okna ORUE.
- [`ru_opcua-logo.svg`](pic/ru_opcua-logo.svg) — pełne logo ORUE (ikona + tekst).
- [`ru_spb-icon.svg`](pic/ru_spb-icon.svg) / `ru_spb-icon.png` — ikona ORSE
  (tarcza regulatora + błyskawica Sparkplug z niepołączonymi węzłami pub/sub),
  również osadzona jako ikona okna ORSE.
- [`ru_spb-logo.svg`](pic/ru_spb-logo.svg) — pełne logo ORSE (ikona + tekst).
- [`ru_s7-icon.svg`](pic/ru_s7-icon.svg) / `ru_s7-icon.png` — ikona ORSS (tarcza
  regulatora + otwarta obudowa modułów kwadratowych, backplane S7), również
  osadzona jako ikona okna ORSS.
- [`ru_s7-logo.svg`](pic/ru_s7-logo.svg) — pełne logo ORSS (ikona + tekst).
- [`ru_eip-icon.svg`](pic/ru_eip-icon.svg) / `ru_eip-icon.png` — ikona OREE
  (tarcza regulatora + zamknięty pierścień rombów, DLR EtherNet/IP), również
  osadzona jako ikona okna OREE.
- [`ru_eip-logo.svg`](pic/ru_eip-logo.svg) — pełne logo OREE (ikona + tekst).
- [`ru_pbdp-icon.svg`](pic/ru_pbdp-icon.svg) / `ru_pbdp-icon.png` — ikona ORPD
  (tarcza regulatora z motywem PROFIBUS DP), również osadzona jako ikona okna
  ORPD.
- [`ru_pbdp-logo.svg`](pic/ru_pbdp-logo.svg) — pełne logo ORPD (ikona + tekst).
- [`Logo-CESAM-Couleur-vect.png`](pic/Logo-CESAM-Couleur-vect.png) — logo CESAM-Lab.

Każda ikona jest **generowana** ze swojego skryptu `*-logo.gen.py`
([`pic/orme-logo.gen.py`](pic/orme-logo.gen.py),
[`pic/osne-logo.gen.py`](pic/osne-logo.gen.py),
[`pic/ru_opcua-logo.gen.py`](pic/ru_opcua-logo.gen.py),
[`pic/ru_spb-logo.gen.py`](pic/ru_spb-logo.gen.py),
[`pic/ru_s7-logo.gen.py`](pic/ru_s7-logo.gen.py),
[`pic/ru_eip-logo.gen.py`](pic/ru_eip-logo.gen.py),
[`pic/ru_pbdp-logo.gen.py`](pic/ru_pbdp-logo.gen.py)). Wszystkie skrypty poza
ORME rasteryzują też swoje `-icon.png` bezpośrednio (przez Pillow); plik `.svg`
ORME jest rasteryzowany później.

W systemie **Wayland** zainstaluj ikonę na pasku zadań danego przyrządu
poleceniem `scripts/install-desktop.sh [orme|osne|ru_opcua|ru_spb|ru_s7|ru_eip|ru_pbdp]`.

## Licencja

[MIT](LICENSE) © 2026 CESAM-Lab

Komponenty innych firm dołączone do niektórych instrumentów są rozpowszechniane na własnych licencjach (w szczególności stos OPC UA na licencji MPL-2.0 używany przez `mock_bin_ru_opcua`); zobacz [NOTICE](NOTICE). Nie zmieniają one licencji MIT kodu cesam-tools.
