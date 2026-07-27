# Dokumentacja utrzymaniowa — ORPD / PROFIBUS DP (workspace `cesam-tools`)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · **PL***

> Crate: `mock_bin_ru_pbdp` · Plik wykonywalny: **ru_pbdp** · Marka: **ORPD**
> Odbiorcy: programiści utrzymujący, naprawiający lub rozszerzający projekt.
> Zob. także: [conception.md](conception.md) · [reference_profibus.md](reference_profibus.md).

---

## 1. Wymagania wstępne

- **Rust stable** (edycja 2021, `rust-version` ≥ 1.85). Instalacja:
  <https://rustup.rs>.
- **Zależności systemowe (Linux) dla GUI** (`eframe`/`egui`,
  OpenGL/winit): `libxkbcommon-dev`, `libwayland-dev`, `libxcb1-dev`,
  `libgl1-mesa-dev` (lub odpowiedniki), plus serwer graficzny
  (X11/Wayland). GUI wymaga **wyświetlacza**: w środowisku headless
  okno się nie otwiera.
- **Łącze szeregowe** (dostęp do portu, `/dev/ttyUSB*`, grupa `dialout`
  w Linuksie): w przeciwieństwie do ORME/OSNE, **nie jest to tutaj
  funkcja opcjonalna** — `tokio-serial` jest bezpośrednią zależnością
  (zob. §5), ponieważ łącze szeregowe jest jedynym transportem tego
  instrumentu (nie istnieje standardowy odpowiednik „PROFIBUS przez
  TCP”). Bez sprzętu GUI i tak się uruchamia (błąd otwarcia jest
  wyświetlany w nagłówku, symulacja nadal działa) — zob.
  [`manuel_utilisateur.md`](manuel_utilisateur.md) §2.
- Dostęp sieciowy do rejestru crates.io dla pierwszej kompilacji.

---

## 2. Typowe polecenia

```bash
cargo check -p mock_bin_ru_pbdp          # Szybka weryfikacja (bez codegen)
cargo build -p mock_bin_ru_pbdp          # Kompilacja debug
cargo build --release -p mock_bin_ru_pbdp   # Kompilacja zoptymalizowana (thin LTO)
cargo test  -p mock_bin_ru_pbdp          # Testy jednostkowe + integracyjne
cargo clippy --workspace --all-targets    # Lint (musi pozostać BEZ ostrzeżeń)
cargo run   -p mock_bin_ru_pbdp          # Uruchamia GUI + szeregowe łącze PROFIBUS DP

# Alternatywny plik konfiguracyjny:
MOCK_CONFIG=./moja_config.toml cargo run -p mock_bin_ru_pbdp
# Szczegółowe logowanie:
RUST_LOG=debug cargo run -p mock_bin_ru_pbdp
```

Wygenerowany plik binarny: `target/debug/ru_pbdp` lub
`target/release/ru_pbdp` (pakiet Cargo pozostaje `mock_bin_ru_pbdp`;
plik wykonywalny i nazwa handlowa „ORPD” są wyłącznie dokumentacyjne,
zob. `[[bin]]` w `Cargo.toml` crate'a).

### Cechy (features) Cargo

| Feature | Domyślnie | Efekt |
|---------|:---------:|-------|
| `gui` | ✅ | GUI `egui`/`eframe` + sprawdzanie aktualizacji (w przeciwnym razie plik binarny headless) |

```bash
cargo build -p mock_bin_ru_pbdp --no-default-features   # headless: łącze szeregowe + symulacja, bez GUI
```

> ⚠️ **Różnica w porównaniu z ORME/OSNE**: w tych dwóch instrumentach
> łącze szeregowe (RTU/szeregowe) samo w sobie jest **funkcją
> opcjonalną** obok zawsze obecnego transportu TCP, a
> `--no-default-features` może je wykluczyć. Tutaj **nie istnieje
> wariant „bez szeregowego”**: `tokio-serial` jest bezpośrednią
> zależnością (niesterowaną przez feature), obecną w **każdej**
> kompilacji, w tym headless — to jedyny transport instrumentu.

---

## 3. Organizacja kodu

```
mock_lib_control/        Biblioteka regulacji wielokrotnego użytku (czysta, bez IO, testowalna)
  src/pid.rs             PID anti-windup
  src/lib.rs             reeksporty (opcjonalna feature `serde`)

mock_bin_ru_pbdp/        Plik binarny regulatora PROFIBUS DP (plik wykonywalny `ru_pbdp`)
  src/main.rs            Uruchomienie: konfiguracja, runtime Tokio, aktorzy, GUI/headless
  src/regulator.rs        Synchroniczny model biznesowy (PID + proces 1. rzędu), Command, krok
  src/config.rs           AppConfig (TOML), SerialConfig, ProcessConfig, RegulationConfig, ServerStatus
  src/profibus.rs         Protokół PROFIBUS DP-V0: kodek ramek + FCS + SlaveFsm (ŹRÓDŁO PRAWDY)
  src/profibus_server.rs  Pętla sesji szeregowej (odczyt ramki → SlaveFsm → odpowiedź) + watchdog
  src/map.rs              Rozmieszczenie bloków I/O Output/Input <-> Command regulatora
  src/trace.rs            Cykliczny dziennik ramek (mini-terminal GUI)
  src/gui.rs              GUI egui (pojedyncza strona + mini-terminal + okno modalne Ustawienia)
  src/branding.rs         Osadzone logotypy (feature `gui`)
  src/i18n.rs             Typowany katalog i18n (8 języków), bez zależności
  src/actors/
    simulation.rs         Pętla regulacji (krok symulacji 50 ms)
    network.rs            Aktor łącza szeregowego PROFIBUS DP, rekonfigurowalny na gorąco

docs/                     Projekt, referencja PROFIBUS, podręcznik, utrzymanie (wielojęzyczne)
```

**Złota zasada**: logika biznesowa (`mock_lib_control`, `regulator.rs`,
`profibus.rs`, `map.rs`) pozostaje **synchroniczna i testowana**;
asynchroniczność jest ograniczona do aktorów i szeregowego IO. Model
regulatora wzorowany na **ORME** (`mock_bin_ru_modbus`) — te same
niezmienniki.

---

## 4. Konfiguracja

- Plik: `mock_ru_pbdp.toml` w bieżącym katalogu, lub ścieżka podana za
  pomocą zmiennej środowiskowej `MOCK_CONFIG`.
- Wczytywany przy uruchomieniu; **wartości domyślne**, jeśli brak lub
  nieczytelny (rejestrowane jest ostrzeżenie, aplikacja i tak się
  uruchamia).
- **Każda wartość pochodząca z TOML jest sanityzowana**
  (`AppConfig::sanitized`): granice wartości zadanej/PID porządkowane,
  wartości zmiennoprzecinkowe wymuszane jako skończone, `τ ≥ 1e-3`,
  `dead_time` ograniczane, **adres stacji ograniczony do `[0, 125]`**.
  **Niezmiennik: nigdy nie wywoływać `f32::clamp` z niezwalidowanymi
  granicami** (panikuje przy `min > max` lub `NaN`).
- Zapisywana z GUI (przyciski *Zastosuj* / *Zapisz* / *Przywróć
  domyślne*).

Struktura (wszystkie sekcje są opcjonalne, uzupełniane domyślnie):

```toml
language = "pl"
check_updates = true       # sprawdzać przy uruchomieniu, czy istnieje nowsza wersja (GUI)

[network.serial]
port = "/dev/ttyUSB0"      # domyślnie "COM3" w Windows
baud = 500000              # znormalizowana wartość PROFIBUS DP (9600 .. 12000000)
station_address = 3        # adres symulowanego urządzenia podrzędnego (0-125)
watchdog_enabled = true    # zezwala na watchdog zapowiadany przez mastera (Set_Prm)

[process]
gain = 1.6 ; tau = 30.0 ; dead_time = 2.0 ; ambient = 20.0

[regulation]
sp_min = 0.0 ; sp_max = 250.0
hysteresis = 2.0 ; tor_min_cycle = 5.0 ; pwm_period = 10.0
[regulation.pid_heat]
kp = ... ; ki = ... ; kd = ... ; out_min = 0.0 ; out_max = 100.0
[regulation.pid_cool]
kp = ... ; ki = ... ; kd = ... ; out_min = 0.0 ; out_max = 100.0
```

> **Format ramki szeregowej (8E1)** jest ustalony przez normę PROFIBUS
> DP i **nie** jest polem konfiguracyjnym — zob. `SerialConfig::open` w
> [`config.rs`](../../src/config.rs). W przeciwieństwie do ORME/OSNE,
> **brak białej listy IP** (łącze szeregowe jest z natury
> punkt-punkt).

### Sprawdzanie aktualizacji

Jeśli `check_updates = true` (domyślnie) **i** plik binarny jest
skompilowany z feature `gui`, GUI odpytuje **przy uruchomieniu**
najnowsze wydanie opublikowane na GitHubie (`CESAMLAB/cesam-tools`) za
pośrednictwem współdzielonej crate **`mock_lib_update`**
(`ureq`/`rustls`, wbudowane certyfikaty root, wątek ograniczony
limitem czasu). **Nieobecne w kompilacjach headless**
(`--no-default-features`).

---

## 5. Zależności i pułapki wersji

| Crate | Rola | Na co zwrócić uwagę |
|-------|------|-------------------|
| `tokio` | runtime asynchroniczny | wspólne features + `io-util` |
| `ractor` | aktorzy | domyślne features |
| `tokio-serial` | łącze PROFIBUS DP | **bezpośrednia zależność, niesterowana przez feature** (zob. §2); `default-features = false` (brak enumeracji `libudev`) |
| `eframe`/`egui` | GUI | wersje powiązane ze sobą, feature `gui` |
| `egui_plot` | krzywa trendu | ⚠️ **wersjonowana jedną wersją pomniejszą przed `egui`**: dla `egui` 0.33 → `egui_plot` **0.34** |
| `serde`/`toml` | trwałość | `mock_lib_control` udostępnia feature `serde` aktywowaną przez plik binarny |
| `mock_lib_update` (`ureq`/`rustls`) | sprawdzanie aktualizacji | tylko feature `gui`; nieobecne headless |

Wspólne wersje są scentralizowane w `[workspace.dependencies]` w
głównym `Cargo.toml`. Przy podnoszeniu `egui`/`eframe`, **sprawdzić
odpowiednią wersję `egui_plot`** (w przeciwnym razie błąd „two versions
of crate egui”).

---

## 6. Rozszerzanie projektu

### 6.1 Dodawanie usługi PROFIBUS (SAP)

Wszystko dzieje się w **[`profibus.rs`](../../src/profibus.rs)** (źródło
prawdy protokołu):

1. Dodać stałą `SAP_*` i odpowiadający jej wariant w `enum Request`;
   podłączyć dekodowanie w `decode_request` (oraz, dla testów, w
   `encode_request`).
2. Obsłużyć nowe żądanie w `SlaveFsm::handle` (przejście stanu, jeśli
   właściwe, zwrócony `Handled`).
3. Zaktualizować komentarz dokumentacyjny modułu i
   **[reference_profibus.md](reference_profibus.md)**.
4. Dodać test w module `tests` w `profibus.rs` (a jeśli dotyczy pełnej
   sesji, także w `profibus_server.rs`).

### 6.2 Modyfikowanie bloków I/O (`Output`/`Input`)

1. Dostosować rozmieszczenie w **[`map.rs`](../../src/map.rs)**
   (`decode_output`/`encode_input`), zachowując spójność
   `OUTPUT_LEN`/`INPUT_LEN` z `SlaveProfile` (`profibus_server.rs`).
2. Zaktualizować tabelę w
   **[reference_profibus.md](reference_profibus.md)** §3 (dokumentacyjne
   źródło prawdy, skopiowane z komentarza dokumentacyjnego `map.rs`).
3. Dodać test typu round-trip w `map.rs`.

### 6.3 Dodawanie polecenia biznesowego / ustawienia GUI

1. Wariant w `enum Command` (`regulator.rs`) + obsługa w
   `Regulator::apply` (z sanityzacją).
2. Pole w `RegulatorSnapshot`, jeśli wartość ma być obserwowalna.
3. Podłączenie GUI (`gui.rs`) za pomocą nieblokującego `cast`.
4. Jeśli trwałe: pole w `AppConfig` (`config.rs`) + sanityzacja w
   `sanitized` + przekazanie w `to_regulator_config`.

### 6.4 Dodawanie ciągu interfejsu (i18n)

Każdy ciąg GUI **musi** przechodzić przez klucz `Msg` (`i18n.rs`) z jego
**8 tłumaczeniami** (tablica o stałym rozmiarze weryfikowana w czasie
kompilacji). Identyfikatory usług PROFIBUS i przyrostki jednostek
pozostają na stałe zakodowane.

### 6.5 Dodawanie nowego instrumentu

1. Utworzyć `mock_bin_<nazwa>/` i dodać go do `members` w głównym
   `Cargo.toml`.
2. Ponownie użyć `mock_lib_control`; wyodrębnić wszystko wspólne do
   `mock_lib_*`.
3. Zachować ten sam podział: model synchroniczny, aktor(y) `ractor`,
   warstwa protokołu, GUI. Konwencja nazw:
   `mock_bin_<typ>_<protokół>`.

---

## 7. Strategia testowania

- **Kodek ramek** (`profibus.rs`): round-trip
  `SD1`/`SD2`/`SD3`/`SD4`, odrzucenie nieprawidłowej sumy kontrolnej i
  długości, kodowanie/dekodowanie żądań
  (`Slave_Diag`/`Set_Prm`/`Chk_Cfg`/`Data_Exchange`) i bajtu trybu.
- **Maszyna stanów** (`profibus.rs`): pełna sekwencja
  `Power_On → Wait_Prm → Wait_Cfg → Data_Exchange`, odrzucenie
  `Set_Prm` z błędnym identyfikatorem (pozostaje w `Wait_Prm`).
- **Bloki I/O** (`map.rs`): zbyt krótki blok wyjściowy → brak polecenia;
  round-trip wartości zadanej/trybu; blok wejściowy odzwierciedla zrzut
  (bity stanu, pomiar).
- **Konfiguracja** (`config.rs`): round-trip TOML, sanityzacja
  (odwrócone granice, wartości nieskończone, adres stacji poza
  zakresem) bez paniki, czysty błąd przy otwieraniu brakującego portu
  szeregowego.
- **Sesja sieciowa** (`profibus_server.rs`, `#[tokio::test]` na
  `tokio::io::duplex`): pełny handshake aż do `Data_Exchange` z
  efektywnym zastosowaniem poleceń, ramka zaadresowana do innej stacji
  zignorowana (brak zaznaczonej aktywności), upływ watchdoga wymuszający
  stan bezpieczny.

Uruchomienie: `cargo test -p mock_bin_ru_pbdp` (lub `--workspace`) —
**36 testów**, wszystkie **deterministyczne i bez GUI**, żaden test
wolny/`#[ignore]` (w przeciwieństwie do ORUE, gdzie generowanie RSA
uzasadnia ignorowane testy).

---

## 8. Rozwiązywanie problemów

| Objaw | Wskazówka |
|----------|-------|
| „two versions of crate `egui`” | Rozbieżność `egui_plot` / `egui`: dopasować wersje (§5). |
| GUI się nie otwiera | Brak wyświetlacza (headless) lub brakujące biblioteki systemowe (§1). |
| Błąd otwarcia portu szeregowego (nagłówek GUI) | Brakujący port, błędna ścieżka lub uprawnienia (grupa `dialout` w Linuksie) — symulacja nadal działa bez łącza. |
| Łącze pozostaje w `Wait_Prm` | Master nie wysyła `Set_Prm` z oczekiwanym identyfikatorem (`0xEE01`) — zob. [reference_profibus.md](reference_profibus.md) §2. |
| Łącze pozostaje w `Wait_Cfg` | Odebrane `Chk_Cfg` nie zgłasza `out_len=45`/`in_len=17`. |
| Urządzenie zatrzymuje się samo | Watchdog protokołu uruchomiony (przedłużona cisza mastera) — oczekiwany stan bezpieczny, nie błąd. |
| Brak watchdoga mimo że master go żąda | `watchdog_enabled = false` w konfiguracji lokalnej: żądanie mastera jest celowo ignorowane. |

Zwiększyć szczegółowość: `RUST_LOG=debug` (lub `trace`).

---

## 9. Kompilacja dystrybucyjna

```bash
cargo build --release -p mock_bin_ru_pbdp
# Samodzielny plik binarny:
target/release/ru_pbdp
```

Profil `release` włącza `lto = "thin"` i `opt-level = 3` (zob. główny
`Cargo.toml`). Do dystrybucji: dostarczyć plik binarny wraz z przykładowym
`mock_ru_pbdp.toml`. Licencja: **MIT** (plik `LICENSE`).

### Feature `gui` (kompilacja z / bez interfejsu)

```bash
cargo build --release -p mock_bin_ru_pbdp                       # z GUI (stanowisko pracy)
cargo build --release -p mock_bin_ru_pbdp --no-default-features  # „headless”: łącze szeregowe + symulacja, bez GUI
```

W przeciwieństwie do OSNE, tryb **headless** nie czyni łącza
szeregowego opcjonalnym (§2): usuwa jedynie GUI. Pozostaje przydatny do
wdrożenia bez ekranu, podłączonego do prawdziwego portu
szeregowego/USB.

### Integracja z pulpitem Linux (ikona paska zadań)

Ikona ORPD (`pic/ru_pbdp-icon.png`, generowana przez
[`pic/ru_pbdp-logo.gen.py`](../../../pic/ru_pbdp-logo.gen.py)) jest
**osadzona** w pliku binarnym (`branding.rs` → `window_icon`). To
wystarcza w **X11, Windows i macOS**. W **Waylandzie** kompozytor
**ignoruje** osadzoną ikonę: kojarzy okno poprzez jego **`app_id`**
(„ru_pbdp”, ustawiony w `main.rs` za pomocą `with_app_id`) z plikiem
`ru_pbdp.desktop` o tej samej nazwie i wyświetla `Icon=ru_pbdp`
rozwiązywaną w motywie ikon `hicolor`.

Aby uzyskać ikonę w Waylandzie, zainstaluj wpis pulpitu dla bieżącego
użytkownika:

```bash
scripts/install-desktop.sh ru_pbdp
```

Skrypt kopiuje:

| Źródło | Cel |
|--------|-------------|
| `pic/ru_pbdp-icon.png` | `~/.local/share/icons/hicolor/256x256/apps/ru_pbdp.png` |
| `packaging/ru_pbdp.desktop` | `~/.local/share/applications/ru_pbdp.desktop` |

a następnie odświeża pamięci podręczne. Trzy nazwy **muszą pozostać
spójne**: `app_id` (`main.rs`), plik `ru_pbdp.desktop` (+ jego
`StartupWMClass`) oraz ikona `ru_pbdp.png` (= `Icon=ru_pbdp`).

---

## 10. Kompilacja „prod” — kompilacja krzyżowa z Linuksa

Wszystko jest produkowane **z Linuksa** przez
[`scripts/build-prod.sh`](../../../scripts/build-prod.sh), który buduje
**każdy instrument workspace** (tabela `INSTRUMENTS`, wpis
`mock_bin_ru_pbdp:ru_pbdp:0` — port `0`: łącze szeregowe, brak portu
IP):

| Wyjście | Cel | GUI | Metoda |
|--------|-------|-----|---------|
| `dist/ru_pbdp-linux-x86_64` | `x86_64-unknown-linux-gnu` | ✅ | `cross` |
| `dist/ru_pbdp-windows-x86_64.exe` | `x86_64-pc-windows-gnu` | ✅ | `cross` (mingw) |
| `dist/ru_pbdp-rpi-arm64` | `aarch64-unknown-linux-gnu` (Pi 3/4/5, Pi OS 64-bit) | ✅ | `cross` |
| Obraz Docker headless `ru_pbdp:headless` | multi-arch `amd64` + `arm64` | ❌ | `docker buildx` |
| `dist/ru_pbdp_<ver>_amd64.deb` / `_arm64.deb` | pakiet Debian/Ubuntu | ✅ | `dpkg-deb` |
| `dist/ru_pbdp-setup-x86_64.exe` | instalator Windows | ✅ | NSIS (`makensis`) |

```bash
cargo install cross          # wymaganie wstępne (jednorazowe) — Docker musi działać
scripts/build-prod.sh        # każdy instrument, w tym ru_pbdp
ONLY=ru_pbdp scripts/build-prod.sh   # tylko ten instrument
```

⚠️ **Nie mieszać natywnego `cargo` i `cross`** w tym samym `target/`
(niekompatybilne proc-makra → `can't find crate for …_derive`). Skrypt
zawsze korzysta z `cross`.

### Obraz Docker headless: ograniczona użyteczność bez przekazywania portu szeregowego

Obraz ([`docker/Dockerfile.headless`](../../../docker/Dockerfile.headless))
jest budowany tak samo jak dla innych instrumentów (`EXPOSE 0`, bezwładne
metadane), ale jest **naprawdę użyteczny tylko z zamontowanym
urządzeniem szeregowym** w kontenerze:

```bash
docker run --rm --device=/dev/ttyUSB0 -v "$PWD/conf:/data" ru_pbdp:headless
```

Bez `--device` kontener się uruchamia, ale nie może otworzyć żadnego
portu szeregowego (takie samo zachowanie jak brak sprzętu lokalnie —
zob. §8).

---

## 11. Konwencje

- Kod i komentarze w języku **francuskim** (konwencja całego projektu);
  logi i komunikaty błędów w języku **angielskim**.
- `cargo clippy --workspace` **bez ostrzeżeń** przed każdym commitem.
- Każde nowe zachowanie biznesowe lub protokołu jest opatrzone
  **testem**.
- Protokół PROFIBUS DP-V0 jest modyfikowany w **`profibus.rs`** (źródło
  prawdy), wraz z aktualizacją
  **[reference_profibus.md](reference_profibus.md)**.
