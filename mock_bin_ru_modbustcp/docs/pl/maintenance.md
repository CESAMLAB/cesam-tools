# Dokumentacja utrzymaniowa — ORME (workspace `cesam-tools`)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · **PL***

> Odbiorcy: programiści utrzymujący, poprawiający lub rozszerzający projekt.
> Zobacz też: [conception.md](conception.md) · [table_modbus.md](table_modbus.md).

---

## 1. Wymagania wstępne

- **Rust stable** (edycja 2021, `rust-version` ≥ 1.85). Instalacja: <https://rustup.rs>.
- **Zależności systemowe (Linux) dla GUI** (`eframe`/`egui`, OpenGL/winit):
  - Debian/Ubuntu: `libxkbcommon-dev`, `libwayland-dev`, `libxcb1-dev`,
    `libgl1-mesa-dev` (lub odpowiedniki), plus serwer graficzny (X11/Wayland).
  - GUI wymaga **wyświetlacza**: w środowisku headless okno się nie otwiera
    (sam serwer Modbus nie zależy od wyświetlacza).
- Dostęp sieciowy do rejestru crates.io przy pierwszej kompilacji.

---

## 2. Najczęstsze polecenia

```bash
cargo check --workspace          # Szybka weryfikacja (bez codegen)
cargo build --workspace          # Kompilacja debug
cargo build --release            # Kompilacja zoptymalizowana (LTO thin)
cargo test  --workspace          # Testy jednostkowe + integracyjne
cargo clippy --workspace --all-targets   # Lint (musi pozostać BEZ ostrzeżeń)
cargo run -p mock_bin_ru_modbustcp       # Uruchamia regulator

# Alternatywny plik konfiguracyjny:
MOCK_CONFIG=./ma_config.toml cargo run -p mock_bin_ru_modbustcp
# Szczegółowe logowanie:
RUST_LOG=debug cargo run -p mock_bin_ru_modbustcp
```

Wytworzony plik binarny: `target/debug/orme` lub `target/release/orme` (pakiet Cargo
pozostaje `mock_bin_ru_modbustcp`, ale plik wykonywalny nazywa się **`orme`** — zob.
`[[bin]]` w `Cargo.toml` crate'a).

### Feature Cargo

| Feature | Domyślnie | Efekt |
|---------|:---------:|-------|
| `gui` | ✅ | GUI `egui`/`eframe` (inaczej plik headless) |
| `rtu` | ✅ | Transport Modbus RTU szeregowy (RS485) przez `tokio-serial` |

```bash
cargo build --no-default-features                 # headless, tylko Modbus TCP
cargo build --no-default-features --features rtu  # headless TCP + RTU szeregowy
cargo build --no-default-features --features gui  # GUI, tylko TCP (bez szeregowego)
```

> ⚠️ **`rtu` = zależność natywna.** `tokio-serial` otwiera port przez termios
> (Linux); enumeracja `libudev` jest wyłączona (`default-features = false`).
> W **kompilacji skrośnej** (`build-prod.sh`, pliki desktop z domyślnymi feature)
> obraz `cross` danego targetu może mimo to wymagać nagłówków szeregowych systemu;
> jeśli łańcuch sprawia problemy, usuń `rtu` z danego buildu. **Docker headless nie
> jest dotknięty** (buduje się z `--no-default-features`).

---

## 3. Organizacja kodu

```
mock_lib_control/        Biblioteka regulacji (czysta, bez IO, testowalna)
  src/pid.rs             PID antynasyceniowy
  src/onoff.rs           Dwustawny z symetryczną histerezą + zabezpieczenie przed krótkim cyklem
  src/pwm.rs             Przekaźnik cyklowy (PWM / time-proportioning)
  src/process.rs         Funkcja przejścia FOPDT
  src/lib.rs             ControllerKind + reeksporty (opcjonalna feature `serde`)

mock_bin_ru_modbustcp/   Plik binarny regulatora
  src/main.rs            Start: konfiguracja, runtime Tokio, aktory, GUI
  src/regulator.rs       Synchroniczny model biznesowy (stan, Command, step)
  src/config.rs          AppConfig (TOML), Transport/SerialConfig, IpFilter, ServerStatus
  src/map.rs             Plan adresowania Modbus (ŹRÓDŁO PRAWDY)
  src/modbus_server.rs   RegulatorService (trait Service) + jeden master TCP + serve_rtu
  src/gui.rs             GUI egui (jedna strona + modal Parametry)
  src/actors/
    simulation.rs        Pętla regulacji (tick)
    network.rs           Serwer Modbus TCP/RTU (re)konfigurowalny na gorąco

docs/                    Projekt, tablica Modbus, utrzymanie
```

**Złota zasada**: logika biznesowa (`mock_lib_control`, `regulator.rs`) pozostaje
**synchroniczna i testowana**; asynchroniczność jest ograniczona do aktorów i IO.

---

## 4. Konfiguracja

- Plik: `mock_ru_modbustcp.toml` w bieżącym katalogu lub ścieżka podana przez
  zmienną środowiskową `MOCK_CONFIG`.
- Ładowany przy starcie; **wartości domyślne**, jeśli brak lub nieczytelny
  (ostrzeżenie jest logowane, aplikacja i tak startuje).
- Zapisywany z GUI (przyciski *Zastosuj* / *Zapisz ustawienia* /
  *Przywróć domyślne*).

Struktura (wszystkie sekcje są opcjonalne, uzupełniane domyślnymi):

```toml
[network]
bind_ip = "0.0.0.0"
port = 5502
allowlist = ["192.168.1.*", "127.0.0.1"]   # pusta = wszystkie IP dozwolone

[process]   # funkcja przejścia G(s) = K·e^(-L·s)/(1+T·s)
gain = 1.6        # K (jednostka/%)
tau = 30.0        # T (s)
dead_time = 2.0   # L (s)
ambient = 20.0

[regulation]
sp_min = 0.0
sp_max = 250.0
hysteresis = 2.0
[regulation.pid_heat]   # kierunek 1 (grzanie)
kp = 4.0 ; ki = 0.25 ; kd = 1.0 ; out_min = 0.0 ; out_max = 100.0
[regulation.pid_cool]   # kierunek 2 (chłodzenie)
kp = 4.0 ; ki = 0.25 ; kd = 1.0 ; out_min = 0.0 ; out_max = 100.0
```

> **Wartości domyślne** mają **jedno źródło**: `RegulatorConfig::default`
> w `regulator.rs`. `ProcessConfig`/`RegulationConfig` (config.rs) z niego wywodzą.
> Aby zmienić domyślną wartość, zmodyfikuj wyłącznie `RegulatorConfig::default`.

---

## 5. Zależności i pułapki wersji

| Crate | Rola | Na co zwrócić uwagę |
|-------|------|-------------------|
| `tokio` | runtime async | feature: `rt-multi-thread, macros, net, time, sync` |
| `ractor` | aktory | feature domyślne (async natywny, **nie** `async-trait`) |
| `tokio-serial` | Modbus RTU szeregowy | opcjonalny (feature `rtu`), `default-features = false` (bez enumeracji libudev) |
| `tokio-modbus` | Modbus TCP | `default-features = false`, feature **`tcp-server`** |
| `eframe`/`egui` | GUI | wersje powiązane ze sobą |
| `egui_plot` | wykres | ⚠️ **wersjonowany o jedną minor wyżej niż `egui`**: dla `egui` 0.33 → `egui_plot` **0.34** |
| `serde`/`toml` | trwałość | `mock_lib_control` udostępnia feature `serde` włączaną przez plik binarny |

Współdzielone wersje są scentralizowane w `[workspace.dependencies]` w głównym
`Cargo.toml`. Aby podnieść `egui`/`eframe`, **sprawdź odpowiednią wersję
`egui_plot`** (inaczej błąd „two versions of crate egui”).

---

## 6. Rozszerzanie projektu

### 6.1 Dodawanie punktu Modbus

Wszystko dzieje się w **`map.rs`** (potem snapshot/Command, jeśli potrzeba):

1. Zadeklaruj stałą adresu i dostosuj `*_COUNT` danej tablicy.
2. Wpisz wartość w `MemoryMap::refresh_from` (stan → rejestr).
3. Jeśli punkt jest zapisywalny, zdekoduj go w `coil_to_command` /
   `holdings_to_commands` (rejestr → `Command`).
4. Zaktualizuj komentarz nagłówkowy **oraz** [table_modbus.md](table_modbus.md).
5. Dodaj wiersz w tablicy live GUI (`gui.rs::modbus_rows`).

### 6.2 Dodawanie komendy / ustawienia

1. Wariant w `enum Command` (`regulator.rs`) + obsługa w `Regulator::apply`.
2. Pole w `RegulatorSnapshot`, jeśli wartość ma być obserwowalna.
3. Podłączenie GUI (`gui.rs`) i/lub dekodowanie Modbus (`map.rs`).
4. Jeśli trwałe: pole w `AppConfig` (`config.rs`) + `to_regulator_config`.

### 6.3 Dodawanie nowego przyrządu

1. Utwórz `mock_bin_<nazwa>/` i dodaj go do `members` głównego `Cargo.toml`.
2. Wykorzystaj ponownie `mock_lib_control`; wynieś wspólne elementy do `mock_lib_*`.
3. Zastosuj ten sam podział: model synchroniczny, aktor(y) ractor, warstwa
   protokołu, GUI. Konwencja nazw: `mock_bin_<typ>_<protokół>`.

---

## 7. Strategia testowania

- **Jednostkowe** (`mock_lib_control`): PID (proporcjonalny, ograniczanie,
  anti-windup), TOR (strefa martwa), proces (zbieżność w stanie ustalonym).
- **Domena** (`regulator.rs`): zbieżność PID w auto, wyjście w trybie ręcznym,
  powrót do otoczenia po zatrzymaniu.
- **Mapowanie** (`map.rs`): round-trip `f32`↔rejestry, dekodowanie zapisu,
  odrzucenie częściowego zapisu `f32`.
- **Konfiguracja / sieć** (`config.rs`, `actors/network.rs`): round-trip TOML,
  filtr IP (znaki wieloznaczne), faktyczny start serwera (bind na efemerycznym porcie).

Uruchomienie: `cargo test --workspace`. Testy są **deterministyczne i bez GUI**.

---

## 8. Rozwiązywanie problemów

| Objaw | Trop |
|----------|-------|
| „two versions of crate `egui`” | Niezgodność `egui_plot` / `egui`: ujednolić wersje (§5). |
| GUI się nie otwiera | Brak wyświetlacza (headless) lub brakujące biblioteki systemowe (§1). |
| `Modbus ✖ błąd nasłuchu` w nagłówku | Port już zajęty lub < 1024 bez uprawnień: zmień port w *Parametry*. |
| Klient jest odrzucany | IP poza **białą listą**: opróżnij listę lub dodaj wzorzec (`192.168.1.*`). |
| Nieprawidłowe wartości `f32` po stronie klienta | Kolejność słów (słowo bardziej znaczące na początku): zob. [table_modbus.md](table_modbus.md). |
| Zapis nastawy `f32` jest ignorowany | Zapisz **oba** rejestry pary w jednym żądaniu. |
| Konfiguracja nie przeładowana | Zły bieżący katalog lub `MOCK_CONFIG`; sprawdź log przy starcie. |
| Brak ikony na pasku zadań (Linux) | Sesja **Wayland**: wbudowana ikona jest ignorowana. Zainstaluj wpis pulpitu: `scripts/install-desktop.sh` (§9, *Integracja z pulpitem*). |

Zwiększenie szczegółowości: `RUST_LOG=debug` (lub `trace`).

---

## 9. Build dystrybucyjny

```bash
cargo build --release
# Samodzielny plik binarny:
target/release/orme
```

Profil `release` aktywuje `lto = "thin"` i `opt-level = 3` (zob. główny
`Cargo.toml`). Aby dystrybuować: dostarcz plik binarny + przykładowy
`mock_ru_modbustcp.toml`. Licencja **MIT** (plik `LICENSE`).

### Feature `gui` (build z / bez interfejsu)

GUI jest za feature Cargo **`gui`**, aktywną domyślnie:

```bash
cargo build --release                       # z GUI (stacja robocza)
cargo build --release --no-default-features  # „headless”: Modbus + symulacja, bez GUI
```

Tryb **headless** jest przeznaczony do wdrożeń bez ekranu (Raspberry Pi jako usługa)
i czyni **kompilację skrośną ARM trywialną** (brak zależności graficznych do linkowania).

### Integracja z pulpitem Linux (ikona na pasku zadań)

Ikona ORME jest wbudowana w plik binarny (`branding.rs` → `with_icon`). To
wystarcza pod **X11, Windowsem i macOS**. Ale pod **Waylandem** kompozytor
**ignoruje** wbudowaną ikonę: kojarzy okno przez jego **`app_id`** („orme”,
zdefiniowany w `main.rs` przez `ViewportBuilder::with_app_id`) z plikiem
`orme.desktop` o tej samej nazwie i wyświetla `Icon=` z tego pliku (rozwiązywany
w motywie ikon `hicolor`).

Aby uzyskać ikonę pod Waylandem, zainstaluj wpis pulpitu dla bieżącego
użytkownika:

```bash
scripts/install-desktop.sh
```

Skrypt kopiuje:

| Źródło | Cel |
|--------|-----|
| `pic/orme-icon.png` | `~/.local/share/icons/hicolor/256x256/apps/orme.png` |
| `packaging/orme.desktop` | `~/.local/share/applications/orme.desktop` |

następnie odświeża pamięci podręczne (`gtk-update-icon-cache`,
`update-desktop-database`). Ikona pojawia się przy następnym uruchomieniu ORME
(i niezawodnie po ponownym zalogowaniu do sesji Wayland).

> ⚠️ Trzy nazwy **muszą pozostać spójne**: `app_id` (`main.rs`), nazwa pliku
> `orme.desktop` i jego `StartupWMClass`, oraz nazwa ikony `orme.png`
> (= `Icon=orme`). `packaging/orme.desktop` zakłada plik wykonywalny `orme`
> w `PATH` (pole `Exec=`); w trybie deweloperskim (`cargo run`) to pole nie ma
> wpływu na wyświetlanie ikony.

---

## 10. Build „prod” — kompilacja skrośna z Linuksa

### Jedna procedura

Wszystko wytwarzane jest **z Linuksa** przez
[`scripts/build-prod.sh`](../../../scripts/build-prod.sh), który buduje **wszystkie
przyrządy workspace** (ORME *oraz* OSNE) w jednym przebiegu. Dla każdego przyrządu
(`<bin>` = `orme`, `osne`):

| Wynik | Cel | GUI | Metoda |
|--------|-------|-----|---------|
| `dist/<bin>-linux-x86_64` | `x86_64-unknown-linux-gnu` | ✅ | `cross` |
| `dist/<bin>-windows-x86_64.exe` | `x86_64-pc-windows-gnu` | ✅ | `cross` (mingw) |
| `dist/<bin>-rpi-arm64` | `aarch64-unknown-linux-gnu` (Pi 3/4/5, Pi OS 64b) | ✅ | `cross` |
| Obraz Docker headless `<bin>:headless` | wieloarchitekturowy `linux/amd64` + `linux/arm64` | ❌ | `docker buildx` |
| `dist/<bin>_<ver>_amd64.deb` / `_arm64.deb` | pakiet Debian/Ubuntu | ✅ | `dpkg-deb` |
| `dist/<bin>-setup-x86_64.exe` | instalator Windows | ✅ | NSIS (`makensis`) |

```bash
# Wymagania (jednorazowo) — Docker musi działać:
cargo install cross

# Wytworzyć wszystko (pliki ORME + OSNE w dist/ + lokalne obrazy Docker amd64 załadowane):
scripts/build-prod.sh

# Wariant: obrazy Docker WIELOARCHITEKTUROWE wypchnięte do rejestru (<prefix>/<bin>:latest):
IMAGE_PREFIX=ghcr.io/<konto> scripts/build-prod.sh

# Zbudować tylko jeden przyrząd:
ONLY=orme scripts/build-prod.sh
```

### Dlaczego `cross` dla WSZYSTKICH buildów (włącznie z Linux x86_64)

`cross` dostarcza obrazy Docker zawierające łańcuchy narzędzi każdego celu: ani
`mingw-w64`, ani łańcucha ARM, ani *sysroot* do instalacji.

⚠️ **Nie mieszaj natywnego `cargo` i `cross` w tym samym `target/`.** Oba używają
różnych wersji `rustc` (host vs kontener); **proc-makra** skompilowane przez jeden
są odrzucane przez drugi, stąd błędy `can't find crate for …_derive`
(np. `zerofrom_derive`, `tracing_attributes`). Skrypt zawsze przechodzi więc
**przez `cross`**, nawet dla Linux x86_64 — jeden łańcuch narzędzi, powtarzalne
buildy. (Jeśli błąd mimo to wystąpi po wcześniejszym buildzie natywnym:
`rm -rf target/release`, następnie ponów.)

### GUI kompilowane skrośnie na ARM: dlaczego to działa

`eframe`/`egui` ładują OpenGL, X11/Wayland i xkbcommon **w czasie wykonania**
(`dlopen`): plik binarny linkuje przy buildzie tylko `libc`. Żadna graficzna
biblioteka ARM nie jest więc potrzebna po stronie cross. Na Raspberry Pi przewidzieć
środowisko graficzne (mesa/X11 lub Wayland) — obecne w Raspberry Pi OS *Desktop*.

> Dla **Raspbiana 32-bitowego** celować w `armv7-unknown-linux-gnueabihf` (dostosować
> cele w skrypcie).

### Obraz Docker headless „gdziekolwiek”

Obraz ([`docker/Dockerfile.headless`](../../../docker/Dockerfile.headless)) wychodzi z
`debian:bookworm-slim` i **kopiuje** plik binarny headless żądanej architektury
(brak kompilacji w obrazie → brak QEMU). `docker buildx` składa wieloarchitekturowy
`amd64`+`arm64`. Serwer nasłuchuje na `5502`. Zamontuj wolumin na
`/data`, aby dostarczyć/utrwalić `mock_ru_modbustcp.toml`.

```bash
# Bez rejestru: lokalny obraz amd64 załadowany, gotowy do testów od razu
docker run --rm -p 5502:5502 -v "$PWD/conf:/data" orme:headless
```

### Instalatory (`.deb` Linux/RPi + setup Windows)

Na końcu buildu `build-prod.sh` wywołuje
[`scripts/make-installers.sh <bin>`](../../../scripts/make-installers.sh), który
przekształca pliki wykonywalne release z `dist/` w **instalatory**:

| Instalator | Źródło | Zawartość | Narzędzie |
|------------|--------|-----------|-----------|
| `<bin>_<ver>_amd64.deb` | `dist/<bin>-linux-x86_64` | binarka → `/usr/bin`, wpis pulpitu, ikona hicolor | `dpkg-deb` |
| `<bin>_<ver>_arm64.deb` | `dist/<bin>-rpi-arm64` | to samo (Raspberry Pi OS 64-bitowy) | `dpkg-deb` |
| `<bin>-setup-x86_64.exe` | `dist/<bin>-windows-x86_64.exe` | exe + skróty (menu Start/pulpit) + deinstalator | NSIS (`makensis`) |

- Pliki `.deb` instalują ikonę i `.desktop`; `postinst` odświeża pamięci podręczne
  ikon i bazę `.desktop`. Zależności: `libc6`; zalecenia graficzne (`libgl1`,
  `libxkbcommon0`, `libwayland-client0`).
- Instalator Windows pochodzi z
  [`packaging/windows/installer.nsi`](../../../packaging/windows/installer.nsi);
  jego skróty mają ikonę `.ico` o wielu rozdzielczościach wyprowadzoną z
  `pic/<bin>-icon.png` (przez Pillow).
- **Wymagania**: `dpkg-deb` (Debian/Ubuntu) dla `.deb`, **`makensis`**
  (`sudo apt install nsis`) dla setupu Windows, `python3`+Pillow dla `.ico`. Każdy
  cel, którego narzędzie/artefakt brakuje, jest **ostrzegany i pomijany** (build się
  nie psuje). Wyłączyć przez `INSTALLERS=0` lub (ponownie) wygenerować same
  instalatory jednego przyrządu: `scripts/make-installers.sh orme`.

### Natywny build Windows (MSVC) — opcjonalny

Wytworzony powyżej `.exe` jest **GNU/mingw** (natywny plik wykonywalny Windows, z GUI).
Jeśli wymagany jest plik binarny **MSVC**, skompiluj na maszynie Windows z
[`scripts/build-windows.ps1`](../../../scripts/build-windows.ps1) (wymagania:
Rust + *Visual Studio Build Tools*, ładunek „Programowanie aplikacji desktop w C++”),
lub z Linuksa przez `cargo-xwin` (`cargo xwin build --release --target x86_64-pc-windows-msvc`).

### Uwagi

- Pliki binarne są **dynamicznie linkowane do glibc**; kompilowane przez `cross`
  (stary baseline glibc) działają na nowoczesnych dystrybucjach (oraz w
  `debian:bookworm-slim`). Dla w pełni statycznego pliku celować w `*-musl`.
- `dist/` jest ignorowany przez git (artefakty buildu).

---

## 11. Konwencje

- Kod i komentarze po **francusku**.
- `cargo clippy --workspace` **bez ostrzeżeń** przed każdym commitem.
- Każde nowe zachowanie biznesowe lub mapowania jest opatrzone **testem**.
- Plan adresowania modyfikuje się w **`map.rs`** (źródło prawdy), wraz ze
  wspólną aktualizacją dokumentacji.
