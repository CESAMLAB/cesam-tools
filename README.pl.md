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
| [`mock_bin_ru_modbustcp`](mock_bin_ru_modbustcp) | **ORME** | Regulator (PID / TOR / PWM) na funkcji przejścia | Modbus TCP & RTU (slave) | egui |
| [`mock_bin_su_namur`](mock_bin_su_namur) | **OSNE** | Mieszadło laboratoryjne nadstawne: funkcja przejścia silnika, szybka regulacja prędkości, regulowane obciążenie lepkościowe | NAMUR po TCP i szeregowo RS-232 (slave) | egui |
| [`mock_bin_ru_opcua`](mock_bin_ru_opcua) | **ORUE** | Regulator obiektowy (PID antynasyceniowy) na procesie pierwszego rzędu, z konfigurowalnym zabezpieczeniem OPC UA | OPC UA (serwer) | egui |

Biblioteka współdzielona:

| Crate | Opis |
|-------|-------------|
| [`mock_lib_control`](mock_lib_control) | Wielokrotnego użytku elementy regulacji: PID antynasyceniowy, dwustawny z histerezą, proces 1. rzędu + czyste opóźnienie (FOPDT). |

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
- **Konfiguracja utrwalana** w formacie TOML (`mock_ru_modbustcp.toml`),
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

## Pobieranie

Gotowe pliki binarne są dostępne na stronie [**Releases**](https://github.com/CESAMLAB/cesam-tools/releases/latest) — **bez potrzeby instalowania narzędzi Rust**. Każdy przyrząd dostarcza własny plik wykonywalny (`orme`, `osne`, `ru_opcua`).

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

```bash
chmod +x orme-linux-x86_64        # Linux / Raspberry Pi (to samo dla osne-*, ru_opcua-*)
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

cargo run -p mock_bin_ru_modbustcp
```

Okno otwiera się, a serwer Modbus TCP nasłuchuje na `0.0.0.0:5502`.
**Port**, **IP nasłuchu** oraz **biała lista IP** ustawia się w modalu
**⚙ Parametry** (stosowane na gorąco), a następnie są **utrwalane** w
`mock_ru_modbustcp.toml`. **Język interfejsu** (francuski, angielski,
niemiecki, hiszpański, włoski, portugalski, niderlandzki, polski) wybiera się w tym
samym modalu i jest utrwalany. Aby użyć innego pliku konfiguracyjnego:

```bash
MOCK_CONFIG=/sciezka/do/ma_config.toml cargo run -p mock_bin_ru_modbustcp
```

### Testowanie połączenia Modbus

Dowolnym klientem Modbus (np. `mbpoll`):

```bash
# Uruchomić (coil 0), następnie odczytać pomiar (input registers 0-1, f32)
mbpoll -m tcp -a 1 -t 0 -p 5502 127.0.0.1 1      # zapisać coil On/Off
mbpoll -m tcp -a 1 -t 3:float -r 1 -p 5502 127.0.0.1   # odczytać PV (f32)
```

Pełna tablica adresów jest udokumentowana w
[`mock_bin_ru_modbustcp/src/map.rs`](mock_bin_ru_modbustcp/src/map.rs).

## Rozwój

```bash
cargo test --workspace      # testy jednostkowe + integracyjne
cargo clippy --workspace    # lint
```

## Dokumentacja

Każdy przyrząd ma własną dokumentację w swoim podkatalogu `docs/`,
dostępną w ośmiu językach (`docs/<język>/`). Wersje polskie:

**ORME** (regulator Modbus):

- [**Podręcznik użytkownika**](mock_bin_ru_modbustcp/docs/pl/manuel_utilisateur.md) — wprowadzenie, GUI, parametry, FAQ.
- [Dokument projektowy](mock_bin_ru_modbustcp/docs/pl/conception.md) — architektura i decyzje techniczne.
- [Tablica adresów Modbus](mock_bin_ru_modbustcp/docs/pl/table_modbus.md) — pełny plan adresowania.
- [Utrzymanie oprogramowania](mock_bin_ru_modbustcp/docs/pl/maintenance.md) — build, konfiguracja, rozszerzanie, rozwiązywanie problemów.

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
- [`Logo-CESAM-Couleur-vect.png`](pic/Logo-CESAM-Couleur-vect.png) — logo CESAM-Lab.

Każda ikona jest **generowana** ze swojego skryptu `*-logo.gen.py`
([`pic/orme-logo.gen.py`](pic/orme-logo.gen.py),
[`pic/osne-logo.gen.py`](pic/osne-logo.gen.py),
[`pic/ru_opcua-logo.gen.py`](pic/ru_opcua-logo.gen.py)). Skrypty OSNE i ORUE
rasteryzują też swoje `-icon.png` bezpośrednio (przez Pillow); plik `.svg` ORME
jest rasteryzowany później.

W systemie **Wayland** zainstaluj ikonę na pasku zadań danego przyrządu
poleceniem `scripts/install-desktop.sh [orme|osne|ru_opcua]`.

## Licencja

[MIT](LICENSE) © 2026 CESAM-Lab

Komponenty innych firm dołączone do niektórych instrumentów są rozpowszechniane na własnych licencjach (w szczególności stos OPC UA na licencji MPL-2.0 używany przez `mock_bin_ru_opcua`); zobacz [NOTICE](NOTICE). Nie zmieniają one licencji MIT kodu cesam-tools.
