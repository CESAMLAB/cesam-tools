# ORME — symulowany regulator Modbus

*🌍 [English](README.md) · [Français](README.fr.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Italiano](README.it.md) · [Português](README.pt.md) · [Nederlands](README.nl.md) · **Polski***

> *Open Regulator Modbus Emulator* · pakiet `mock_bin_ru_modbustcp` · plik binarny `orme`

**Symulowany** regulator przemysłowy, slave **Modbus TCP/RTU**, z interfejsem
graficznym. Część workspace [`cesam-tools`](../README.pl.md).

## Funkcjonalności

- Proces pierwszego rzędu + czyste opóźnienie (funkcja przejścia FOPDT).
- Regulacja dwukierunkowa (grzanie / chłodzenie), każdy kierunek jako **PID** lub
  **dwustawny**.
- Tryby start/stop oraz auto/ręczny; nastawy auto (fizyczna) i ręczna (%).
- Serwer Modbus TCP udostępniający całość stanu.
- GUI `egui` z wykresem trendu w czasie rzeczywistym i regulacją wzmocnień PID.
- **Interfejs wielojęzyczny**: francuski, angielski, niemiecki, hiszpański, włoski,
  portugalski, niderlandzki, polski (wybór w modalu *Parametry*, utrwalany).

## Uruchamianie

```bash
cargo run -p mock_bin_ru_modbustcp
# Alternatywny plik konfiguracyjny:
MOCK_CONFIG=./ma_config.toml cargo run -p mock_bin_ru_modbustcp
```

Nasłuch domyślnie na `0.0.0.0:5502`. Port, IP nasłuchu i biała lista
IP ustawiane są w modalu **⚙ Parametry** i utrwalane w TOML.

## Tablica adresów Modbus

Kodowanie zmiennoprzecinkowych: 2 rejestry, big-endian, słowo bardziej znaczące najpierw.

### Cewki (FC 1/5/15)

| Adr | Rola |
|----|------|
| 0 | Start (1) / Stop (0) |
| 1 | Auto (1) / Ręczny (0) |

### Wejścia dwustanowe (FC 2, tylko odczyt)

| Adr | Rola |
|----|------|
| 0 | W ruchu |
| 1 | Kierunek 1 (grzanie) aktywny |
| 2 | Kierunek 2 (chłodzenie) aktywny |

### Rejestry przechowujące (FC 3/6/16)

| Adr | Typ | Rola |
|-----|------|------|
| 0 | u16 | Tryb kierunek 1 (0=Off, 1=PID, 2=TOR) |
| 1 | u16 | Tryb kierunek 2 (0=Off, 1=PID, 2=TOR) |
| 2–3 | f32 | Nastawa automatyczna (SP) |
| 4–5 | f32 | Nastawa ręczna (% wyjścia, ze znakiem) |
| 6–7 | f32 | Kp kierunek 1 |
| 8–9 | f32 | Ki kierunek 1 |
| 10–11 | f32 | Kd kierunek 1 |
| 12–13 | f32 | Kp kierunek 2 |
| 14–15 | f32 | Ki kierunek 2 |
| 16–17 | f32 | Kd kierunek 2 |
| 18–19 | f32 | Histereza TOR |

### Rejestry wejściowe (FC 4, tylko odczyt)

| Adr | Typ | Rola |
|-----|------|------|
| 0–1 | f32 | Pomiar (PV) |
| 2–3 | f32 | Zastosowane wyjście (% ze znakiem: + grzanie / − chłodzenie) |

Źródłem prawdy jest nagłówek [`src/map.rs`](src/map.rs).

## Dokumentacja

Dokumentacja właściwa dla tej aplikacji (katalog [`docs/pl/`](docs/pl/)):

- [**Podręcznik użytkownika**](docs/pl/manuel_utilisateur.md) — wprowadzenie, sterowanie, parametry, FAQ.
- [Dokument projektowy](docs/pl/conception.md) — architektura, decyzje techniczne, teoria regulacji.
- [Tablica adresów Modbus](docs/pl/table_modbus.md) — pełny plan adresowania, kodowanie, przykłady.
- [Utrzymanie oprogramowania](docs/pl/maintenance.md) — build, konfiguracja, rozszerzanie, rozwiązywanie problemów.
