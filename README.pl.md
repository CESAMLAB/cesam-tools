<p align="center">
  <img src="pic/Logo-CESAM-Couleur-vect.png" alt="CESAM-Lab" height="84">
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

## Pobieranie

Gotowe pliki binarne są dostępne na stronie [**Releases**](https://github.com/CESAMLAB/cesam-tools/releases/latest) — **bez potrzeby instalowania narzędzi Rust**.

| Platforma | GUI | Headless (tylko TCP, bez GUI) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`orme-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-linux-x86_64) | [`orme-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-linux-x86_64-headless) |
| Windows x86_64 | [`orme-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`orme-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-rpi-arm64) | [`orme-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-rpi-arm64-headless) |

```bash
chmod +x orme-linux-x86_64        # Linux / Raspberry Pi
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

Zobacz [CLAUDE.md](CLAUDE.md) dla konwencji i szczegółowej architektury.

## Dokumentacja

Każdy przyrząd ma własną dokumentację w swoim podkatalogu `docs/`,
dostępną w ośmiu językach (`docs/<język>/`). Dla regulatora (wersja
polska):

- [**Podręcznik użytkownika**](mock_bin_ru_modbustcp/docs/pl/manuel_utilisateur.md) — wprowadzenie, GUI, parametry, FAQ.
- [Dokument projektowy](mock_bin_ru_modbustcp/docs/pl/conception.md) — architektura i decyzje techniczne.
- [Tablica adresów Modbus](mock_bin_ru_modbustcp/docs/pl/table_modbus.md) — pełny plan adresowania.
- [Utrzymanie oprogramowania](mock_bin_ru_modbustcp/docs/pl/maintenance.md) — build, konfiguracja, rozszerzanie, rozwiązywanie problemów.

## Marka i logo

Logo znajdują się w [`pic/`](pic/):

- [`orme-icon.svg`](pic/orme-icon.svg) / `orme-icon.png` — ikona ORME (tarcza),
  również osadzona jako ikona okna aplikacji.
- [`orme-logo.svg`](pic/orme-logo.svg) — pełne logo ORME (ikona + tekst).
- [`Logo-CESAM-Couleur-vect.png`](pic/Logo-CESAM-Couleur-vect.png) — logo CESAM-Lab.

Ikona ORME jest **generowana** z [`pic/orme-logo.gen.py`](pic/orme-logo.gen.py)
(`python3 pic/orme-logo.gen.py` produkuje pliki `.svg`, które należy następnie zrasteryzować).

## Licencja

[MIT](LICENSE) © 2026 CESAM-Lab
