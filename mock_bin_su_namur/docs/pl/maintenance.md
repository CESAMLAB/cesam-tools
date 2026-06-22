# Dokumentacja utrzymaniowa — OSNE (workspace `cesam-tools`)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · **PL***

> Odbiorcy: programiści, którzy utrzymują, naprawiają lub rozszerzają projekt.
> Zobacz także: [conception.md](conception.md) · [commandes_namur.md](commandes_namur.md).

---

## 1. Wymagania wstępne

- **Rust stable** (edycja 2021, `rust-version` ≥ 1.85). Instalacja: <https://rustup.rs>.
- **Zależności systemowe (Linux) dla GUI** (`eframe`/`egui`, OpenGL/winit):
  - Debian/Ubuntu: `libxkbcommon-dev`, `libwayland-dev`, `libxcb1-dev`,
    `libgl1-mesa-dev` (lub odpowiedniki), plus serwer graficzny (X11/Wayland).
  - GUI wymaga **wyświetlacza**: w środowisku headless okno się nie otwiera
    (serwer NAMUR natomiast nie zależy od wyświetlacza).
- **Łącze szeregowe** (feature `serial`): dostęp do portu (`/dev/ttyUSB*`, grupa
  `dialout` pod Linux). Bez sprzętu używaj transportu **TCP**.
- Dostęp sieciowy do rejestru crates.io przy pierwszej kompilacji.

---

## 2. Częste komendy

```bash
cargo check -p mock_bin_su_namur          # Szybka weryfikacja (bez codegen)
cargo build -p mock_bin_su_namur          # Kompilacja debug
cargo build --release -p mock_bin_su_namur   # Kompilacja zoptymalizowana (LTO thin)
cargo test  -p mock_bin_su_namur          # Testy jednostkowe + integracyjne
cargo clippy --workspace --all-targets    # Lint (musi pozostać BEZ ostrzeżeń)
cargo run   -p mock_bin_su_namur          # Uruchamia mieszadło (GUI + NAMUR/TCP)

# Alternatywny plik konfiguracyjny:
MOCK_CONFIG=./ma_config.toml cargo run -p mock_bin_su_namur
# Szczegółowe logowanie:
RUST_LOG=debug cargo run -p mock_bin_su_namur
```

Wytworzona binarka: `target/debug/osne` lub `target/release/osne` (pakiet Cargo
pozostaje `mock_bin_su_namur`, ale plik wykonywalny nazywa się **`osne`** — zob.
`[[bin]]` w `Cargo.toml` crate'a).

### Features Cargo

| Feature | Domyślnie | Efekt |
|---------|:---------:|-------|
| `gui` | ✅ | GUI `egui`/`eframe` (w przeciwnym razie binarka headless) |
| `serial` | ✅ | Transport NAMUR po łączu szeregowym RS-232 przez `tokio-serial` |

```bash
cargo build -p mock_bin_su_namur --no-default-features                  # headless, tylko NAMUR/TCP
cargo build -p mock_bin_su_namur --no-default-features --features serial # headless TCP + szereg
cargo build -p mock_bin_su_namur --no-default-features --features gui    # GUI, tylko TCP (bez szeregu)
```

> ⚠️ **`serial` = zależność natywna.** `tokio-serial` otwiera port przez termios
> (Linux); enumeracja `libudev` jest wyłączona (`default-features = false`). W
> **kompilacji skrośnej** (`build-prod.sh`, exe desktopowe z domyślnymi features)
> obraz `cross` celu może mimo to żądać nagłówków szeregowych; jeśli łańcuch
> sprawia problem, usuń `serial` z danego buildu. **Docker headless nie jest tym
> dotknięty** (buduje się w `--no-default-features`).

---

## 3. Organizacja kodu

```
mock_lib_control/        Biblioteka regulacji (czysta, bez IO, testowalna)
  src/pid.rs             PID z zabezpieczeniem przed nasyceniem (użyty do regulacji prędkości)
  src/lib.rs             re-eksporty (opcjonalna feature `serde`)

mock_bin_su_namur/       Binarka mieszadła (plik wykonywalny `osne`)
  src/main.rs            Start: config, runtime Tokio, aktorzy, GUI
  src/motor.rs           Model fizyczny silnika (dynamika obrotowa, Euler)
  src/stirrer.rs         Synchroniczny model biznesowy (stan, Command, step) — posiada PID
  src/config.rs          AppConfig (TOML), Transport/SerialConfig, IpFilter, ServerStatus
  src/namur.rs           Protokół NAMUR: handle_line (ŹRÓDŁO PRAWDY zestawu komend)
  src/namur_server.rs    Serwis NAMUR (linie ASCII) + mono-master TCP + serwowanie szeregowe + watchdog
  src/trace.rs           Bufor cykliczny ramek (mini-terminal GUI)
  src/gui.rs             GUI egui (pojedyncza strona + mini-terminal + modal Parametry)
  src/branding.rs        Wbudowane logo (feature `gui`)
  src/i18n.rs            Typowany katalog i18n (8 języków), bez zależności
  src/actors/
    simulation.rs        Pętla symulacji (tick 20 ms)
    network.rs           Serwer NAMUR TCP/szereg (re)konfigurowalny na gorąco

docs/                    Projekt, komendy NAMUR, podręcznik, utrzymanie (wielojęzyczne)
```

**Złota zasada**: logika biznesowa (`mock_lib_control`, `motor.rs`, `stirrer.rs`)
pozostaje **synchroniczna i przetestowana**; asynchroniczność jest ograniczona do
aktorów i IO. Dokładna kopia regulatora **ORME** (`mock_bin_ru_modbustcp`) — te
same niezmienniki.

---

## 4. Konfiguracja

- Plik: `mock_su_namur.toml` w bieżącym katalogu lub ścieżka podana przez zmienną
  środowiskową `MOCK_CONFIG`.
- Wczytywany przy starcie; **wartości domyślne**, jeśli brak lub nieczytelny
  (logowane jest ostrzeżenie, aplikacja i tak startuje).
- **Każda wartość pochodząca z TOML jest sanityzowana** (`AppConfig::sanitized`):
  uporządkowane granice (`min ≤ max`), liczby zmiennoprzecinkowe wymuszone na
  skończone, bezwładność/moment/lepkość ściśle dodatnie. **Niezmiennik: nigdy nie
  `f32::clamp` z niezweryfikowanymi granicami** (panika przy `min > max` lub
  `NaN`).
- Zapisywany z GUI (przyciski *Zastosuj* / *Zapisz* / *Przywróć*).

Struktura (wszystkie sekcje są opcjonalne, uzupełniane domyślnie):

```toml
language = "fr"

[network]
transport = "tcp"          # "tcp" lub "serial"
bind_ip = "0.0.0.0"
port = 4001
allowlist = ["192.168.1.*", "127.0.0.1"]   # puste = wszystkie IP dozwolone
[network.serial]
port = "/dev/ttyUSB0"
baud = 9600 ; parity = "even" ; data_bits = 7 ; stop_bits = 1   # NAMUR 7E1

[motor]   # J·dω/dt = T − k·η·ω − tarcie
inertia = 0.02      # J (reaktywność)
load_coeff = 0.05   # k (waga lepkości)
friction = 2.0      # N·cm
torque_max = 100.0  # N·cm (pułap wyjścia PID)

[regulation]
speed_min = 0.0 ; speed_max = 2000.0
viscosity = 1.0 ; viscosity_min = 0.1 ; viscosity_max = 20.0
[regulation.pid]
kp = ... ; ki = ... ; kd = ... ; out_min = 0.0 ; out_max = 100.0
```

> **Wartości domyślne** mają **jedno źródło**: `StirrerConfig::default` w
> `stirrer.rs`. `MotorConfig`/`RegulationConfig` (config.rs) z nich dziedziczą.
> Granice wyjścia PID (`out_min`/`out_max`) są **wymuszane** na `[0, couple_max]`
> w momencie budowania mieszadła (`to_stirrer_config`).

---

## 5. Zależności i pułapki wersji

| Crate | Rola | Punkt uwagi |
|-------|------|-------------------|
| `tokio` | runtime async | współdzielone features + **`io-util`** (BufReader / linie ASCII NAMUR) |
| `ractor` | aktorzy | domyślne features (natywny async, **nie** `async-trait`) |
| `tokio-serial` | NAMUR szeregowy | opcjonalny (feature `serial`), `default-features = false` (brak enumeracji libudev) |
| `eframe`/`egui` | GUI | wersje powiązane ze sobą |
| `egui_plot` | wykres | ⚠️ **wersjonowany o jedną mniejszą przed `egui`**: dla `egui` 0.33 → `egui_plot` **0.34** |
| `serde`/`toml` | trwałość | `mock_lib_control` udostępnia feature `serde` aktywowaną przez binarkę |

Współdzielone wersje są scentralizowane w `[workspace.dependencies]` głównego
`Cargo.toml`. Aby podnieść `egui`/`eframe`, **sprawdź odpowiadającą wersję
`egui_plot`** (inaczej błąd „two versions of crate egui”).

---

## 6. Rozszerzanie projektu

### 6.1 Dodanie komendy NAMUR

Wszystko dzieje się w **`namur.rs`** (źródło prawdy protokołu):

1. Dodaj gałąź w `handle_line` (odczyt → `Reply`, zapis/akcja →
   `Apply(Command)` lub `SetWatchdog`).
2. Jeśli to **akcja**, dodaj wariant w `enum Command` (`stirrer.rs`) i jego
   obsługę w `Stirrer::apply`.
3. Zaktualizuj nagłówkowy komentarz dokumentacyjny,
   **[commandes_namur.md](commandes_namur.md)** i tablicę referencyjną
   mini-terminala (`gui.rs`, tablica `rows`).
4. Dodaj test w module `tests` pliku `namur.rs`.

### 6.2 Dodanie komendy / nastawy GUI

1. Wariant w `enum Command` (`stirrer.rs`) + obsługa w `Stirrer::apply`.
2. Pole w `StirrerSnapshot`, jeśli wartość ma być obserwowalna.
3. Połączenie GUI (`gui.rs`) przez nieblokujący `cast`.
4. Jeśli trwałe: pole w `AppConfig` (`config.rs`) + sanityzacja w `sanitized` +
   przeniesienie do `to_stirrer_config`.

### 6.3 Dodanie napisu interfejsu (i18n)

Każdy napis GUI **musi** przechodzić przez klucz `Msg` (`i18n.rs`) ze swoimi **8
tłumaczeniami** (tablica o stałym rozmiarze weryfikowana przy kompilacji).
Akronimy NAMUR, sufiksy jednostek i nazwy komend pozostają zakodowane na stałe.

### 6.4 Dodanie nowego przyrządu

1. Utwórz `mock_bin_<nom>/` i dodaj go do `members` głównego `Cargo.toml`.
2. Wykorzystaj ponownie `mock_lib_control`; wyodrębnij wszystko wspólne do
   `mock_lib_*` (np. promocja modelu `motor.rs`, jeśli posłuży drugiemu
   przyrządowi).
3. Zachowaj ten sam podział: model synchroniczny, aktor(zy) ractor, warstwa
   protokołu, GUI. Konwencja nazewnictwa: `mock_bin_<type>_<protocole>`.

---

## 7. Strategia testów

- **Jednostkowe** (`mock_lib_control`): PID (proporcjonalny, ograniczanie,
  anti-windup).
- **Silnik** (`motor.rs`): dynamika obrotowa, zbieżność stanu ustalonego, wpływ
  lepkości na moment, nasycenie/przeciążenie.
- **Domena** (`stirrer.rs`): zbieżność prędkości do wartości zadanej, zwalnianie
  przy zatrzymaniu, wykrywanie przeciążenia.
- **Protokół** (`namur.rs`): dekodowanie odczytów (`IN_*`), zapisów (`OUT_SP_4`),
  akcji (`START/STOP/RESET`), watchdoga i nieznanych komend.
- **Config / sieć** (`config.rs`, `actors/network.rs`): round-trip TOML, filtr IP
  (jokery, IPv4-mapped), sanityzacja bez paniki, otwarcie szeregowe z błędem przy
  nieobecnym porcie.

Uruchomienie: `cargo test -p mock_bin_su_namur` (lub `--workspace`). Testy są
**deterministyczne i bez GUI**.

---

## 8. Rozwiązywanie problemów

| Objaw | Trop |
|----------|-------|
| „two versions of crate `egui`” | Niezgodność `egui_plot` / `egui`: wyrównaj wersje (§5). |
| GUI się nie otwiera | Brak wyświetlacza (headless) lub brakujące biblioteki systemowe (§1). |
| `NAMUR ✖` w nagłówku | Port TCP już zajęty / < 1024 bez uprawnień, albo niedostępny port szeregowy: zmień w *Parametry*. |
| Klient TCP jest odrzucany | IP poza **białą listą**: opróżnij listę lub dodaj wzorzec (`192.168.1.*`). |
| Szereg się nie otwiera | Brak feature `serial`, zły port lub uprawnienia (`dialout`). |
| Silnik zatrzymuje się sam | **Watchdog** uzbrojony (`OUT_WD1@…`) bez ruchu: wysyłaj ramki lub `OUT_WD1@0`. |
| Stałe przeciążenie | Lepkość zbyt wysoka względem `torque_max`: dostosuj parametry silnika. |
| Config nie wczytany ponownie | Zły bieżący katalog lub `MOCK_CONFIG`; sprawdź dziennik przy starcie. |

Zwiększenie szczegółowości: `RUST_LOG=debug` (lub `trace`).

---

## 9. Build dystrybucyjny

```bash
cargo build --release -p mock_bin_su_namur
# Samodzielna binarka:
target/release/osne
```

Profil `release` aktywuje `lto = "thin"` i `opt-level = 3` (zob. główny
`Cargo.toml`). Aby dystrybuować: dostarcz binarkę + przykładowy
`mock_su_namur.toml`. Licencja **MIT** (plik `LICENSE`).

### Feature `gui` (build z / bez interfejsu)

```bash
cargo build --release -p mock_bin_su_namur                       # z GUI (stacja robocza)
cargo build --release -p mock_bin_su_namur --no-default-features  # „headless”: NAMUR + symulacja, bez GUI
```

Tryb **headless** jest przeznaczony do wdrożeń bez ekranu i czyni
**kompilację skrośną ARM trywialną** (brak zależności graficznych do linkowania).

### Integracja z pulpitem Linux (ikona w pasku zadań)

Ikona OSNE (`pic/osne-icon.png`, motyw mieszadła, generowana przez
[`pic/osne-logo.gen.py`](../../../pic/osne-logo.gen.py)) jest **wbudowana** w
binarkę (`branding.rs` → `window_icon`). To wystarcza pod **X11, Windows i
macOS**. Pod **Wayland** kompozytor **ignoruje** wbudowaną ikonę: kojarzy okno z
jego **`app_id`** („osne”, zdefiniowanym w `main.rs` przez `with_app_id`) z
plikiem `osne.desktop` o tej samej nazwie i wyświetla `Icon=osne` rozwiązaną w
motywie ikon `hicolor`.

Aby uzyskać ikonę pod Wayland, zainstaluj wpis pulpitu dla bieżącego
użytkownika:

```bash
scripts/install-desktop.sh osne
```

Skrypt kopiuje:

| Źródło | Cel |
|--------|-------------|
| `pic/osne-icon.png` | `~/.local/share/icons/hicolor/256x256/apps/osne.png` |
| `packaging/osne.desktop` | `~/.local/share/applications/osne.desktop` |

a następnie odświeża cache. Trzy nazwy **muszą pozostać wyrównane**: `app_id`
(`main.rs`), plik `osne.desktop` (+ jego `StartupWMClass`) oraz ikona `osne.png`
(= `Icon=osne`). Ten sam skrypt instaluje ORME bez argumentu
(`scripts/install-desktop.sh`).

---

## 10. Build „prod” — kompilacja skrośna z Linux

### Jedna procedura

Wszystko jest produkowane **z Linux** przez
[`scripts/build-prod.sh`](../../../scripts/build-prod.sh), który buduje **wszystkie
przyrządy workspace'a** (ORME *i* OSNE):

| Wyjście | Cel | GUI | Metoda |
|--------|-------|-----|---------|
| `dist/osne-linux-x86_64` | `x86_64-unknown-linux-gnu` | ✅ | `cross` |
| `dist/osne-windows-x86_64.exe` | `x86_64-pc-windows-gnu` | ✅ | `cross` (mingw) |
| `dist/osne-rpi-arm64` | `aarch64-unknown-linux-gnu` (Pi 3/4/5, Pi OS 64b) | ✅ | `cross` |
| Obraz Docker headless `osne:headless` | multi-arch `amd64` + `arm64` | ❌ | `docker buildx` |
| `dist/osne_<ver>_amd64.deb` / `_arm64.deb` | pakiet Debian/Ubuntu | ✅ | `dpkg-deb` |
| `dist/osne-setup-x86_64.exe` | instalator Windows | ✅ | NSIS (`makensis`) |

```bash
# Wymagania wstępne (raz) — Docker musi działać:
cargo install cross

# Wyprodukuj wszystko (exe ORME + OSNE + instalatory w dist/ + obrazy Docker amd64):
scripts/build-prod.sh

# Wariant: obrazy Docker MULTI-ARCH wypchnięte do rejestru:
IMAGE_PREFIX=ghcr.io/<konto> scripts/build-prod.sh

# Bez budowania instalatorów:
INSTALLERS=0 scripts/build-prod.sh
```

### Dlaczego `cross` dla WSZYSTKICH buildów (w tym Linux x86_64)

`cross` dostarcza obrazy Docker zawierające toolchainy każdego celu.
⚠️ **Nie mieszaj natywnego `cargo` i `cross` w tym samym `target/`.**
**Proc-makra** skompilowane przez jeden są odrzucane przez drugi (`can't find
crate for …_derive`). Skrypt zawsze przechodzi **przez `cross`**. (Jeśli wystąpi
błąd: `rm -rf target/release`, a następnie uruchom ponownie.)

### GUI kompilowane skrośnie na ARM: dlaczego to działa

`eframe`/`egui` ładują OpenGL, X11/Wayland i xkbcommon **w czasie wykonania**
(`dlopen`): binarka linkuje przy buildzie jedynie `libc`. Żadna biblioteka
graficzna ARM nie jest potrzebna po stronie cross; zapewnij środowisko pulpitu na
celu.

### Obraz Docker headless

Obraz ([`docker/Dockerfile.headless`](../../../docker/Dockerfile.headless)) startuje
z `debian:bookworm-slim` i **kopiuje** binarkę headless żądanej architektury (brak
kompilacji w obrazie → brak QEMU). Nazwa binarki i wystawiony port są przekazywane
przez `--build-arg` (`BIN=osne`, `PORT=4001`). Zamontuj wolumen na `/data`, aby
dostarczyć/utrwalić `mock_su_namur.toml`.

```bash
docker run --rm -p 4001:4001 -v "$PWD/conf:/data" osne:headless
```

### Instalatory (`.deb` Linux/RPi + setup Windows)

Na końcu każdego buildu `build-prod.sh` wywołuje
[`scripts/make-installers.sh <bin>`](../../../scripts/make-installers.sh), który
przekształca pliki wykonywalne release z `dist/` w **instalatory**:

| Instalator | Źródło | Zawartość | Narzędzie |
|------------|--------|-----------|-----------|
| `osne_<ver>_amd64.deb` | `dist/osne-linux-x86_64` | binarka → `/usr/bin`, wpis pulpitu, ikona hicolor | `dpkg-deb` |
| `osne_<ver>_arm64.deb` | `dist/osne-rpi-arm64` | to samo (Raspberry Pi OS 64-bitowy) | `dpkg-deb` |
| `osne-setup-x86_64.exe` | `dist/osne-windows-x86_64.exe` | exe + skróty (menu Start/pulpit) + deinstalator | NSIS (`makensis`) |

- Pliki `.deb` instalują ikonę i `.desktop`; `postinst` odświeża pamięci podręczne
  (`update-desktop-database`, `gtk-update-icon-cache`). Zależności: `libc6`;
  zalecenia graficzne (`libgl1`, `libxkbcommon0`, `libwayland-client0`).
- Instalator Windows jest generowany z
  [`packaging/windows/installer.nsi`](../../../packaging/windows/installer.nsi);
  skróty używają ikony `.ico` o wielu rozdzielczościach wyprowadzonej z
  `pic/osne-icon.png` (przez Pillow).
- **Wymagania**: `dpkg-deb` (obecny w Debian/Ubuntu) dla `.deb`, **`makensis`**
  (`sudo apt install nsis`) dla setupu Windows, `python3`+Pillow dla `.ico`. Każdy
  cel, którego narzędzie lub artefakt brakuje, jest **ostrzegany i pomijany** (build
  się nie psuje). Wyłączyć przez `INSTALLERS=0`. Można też (ponownie) wygenerować
  same instalatory jednego przyrządu: `scripts/make-installers.sh osne`.
- **Wersja** pakietów pochodzi z `[workspace.package].version` w głównym `Cargo.toml`.

### Uwagi

- Binarki są **dynamicznie linkowane z glibc**; skompilowane przez `cross`
  (stara baseline glibc) działają na nowoczesnych dystrybucjach.
- `dist/` jest ignorowany przez git (artefakty buildu).

---

## 11. Konwencje

- Kod i komentarze po **francusku**; logi i komunikaty błędów po **angielsku**.
- `cargo clippy --workspace` **bez ostrzeżeń** przed każdym commitem.
- Każde nowe zachowanie biznesowe, silnika lub protokołu jest opatrzone
  **testem**.
- Zestaw komend NAMUR modyfikuje się w **`namur.rs`** (źródło prawdy), z
  równoczesną aktualizacją dokumentacji.
