# Projektowanie — symulowany regulator procesu (RU/OPC UA)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · **PL***

> Crate: `mock_bin_ru_opcua` · Plik wykonywalny: **ru_opcua** (*Regulation Unit over OPC UA*)

Dokument architektury i modelowania. Wzorowany na regulatorze **ORME**
(`mock_bin_ru_modbustcp`): ten sam podział na **synchroniczny model biznesowy /
aktorzy ractor / warstwa protokołu / GUI egui**, te same niezmienniki. Zmienia się
jedynie **transport**: **OPC UA** zamiast Modbus.

---

## 1. Cel

Symulowanie **regulatora procesu** (pętla PID na termicznym procesie pierwszego
rzędu) i udostępnienie go przez **OPC UA**, standard przemysłowego nadzoru
(Przemysł 4.0). W przeciwieństwie do ORME (Modbus) i OSNE (NAMUR) — protokołów
**obiektowych bez zabezpieczeń** — OPC UA natywnie obsługuje uwierzytelnianie,
podpis i szyfrowanie (przewidziane w Fazie 2).

---

## 2. Model fizyczny ([`regulator.rs`](../../src/regulator.rs))

**Proces** ponownie wykorzystuje [`mock_lib_control::FirstOrderProcess`]
(współdzielony z ORME): funkcja przejścia pierwszego rzędu z czystym opóźnieniem

```text
PV(s) / U(s) = K · e^(−L·s) / (1 + τ·s)
```

- `PV`: pomiar (jednostka procesu, np. °C);
- `U`: sterowanie / wyjście (0-100 %);
- `K`: wzmocnienie statyczne; `τ`: stała czasowa; `L`: czyste opóźnienie;
- `ambient`: wartość spoczynkowa (zerowe wyjście).

**PID** ([`mock_lib_control::Pid`], również ponownie wykorzystany z ORME) prowadzi
pomiar do **wartości zadanej**, sterując wyjściem ograniczonym do `[0, 100]`. Dwa
tryby: **automatyczny** (PID oblicza wyjście) i **ręczny** (wyjście narzucone).
Krok symulacji wynosi **0,5 s** (wolny proces termiczny).

Wszystkie zapisy (sieć lub GUI) są **odkażane** w `Regulator::apply`: nieskończone
liczby zmiennoprzecinkowe ignorowane, wartość zadana ograniczona, granice
uporządkowane (`min ≤ max`), wzmocnienia PID obcięte. **Niezmiennik: nigdy
`f32::clamp` z niezweryfikowanymi granicami** (panika, jeśli `min > max` lub `NaN`).

---

## 3. Architektura (aktorzy)

```
GUI (egui) ───Command(cast)──►  SimulationActor ──refresh──► SharedSnapshot ──► GUI
Serwer OPC UA ─Command(cast)─►   (Regulator)    ──refresh──► SharedSnapshot ──► odczyty OPC UA
```

- **`SimulationActor`** ([`actors/simulation.rs`](../../src/actors/simulation.rs)):
  **wyłączny** właściciel `Regulator`; przesuwa symulację na jednorazowym,
  ponownie uzbrajanym timerze (brak odłączonego timera) i publikuje
  `SharedSnapshot` przy każdym kroku.
- **`OpcuaServerActor`** ([`actors/network.rs`](../../src/actors/network.rs)):
  posiada serwer OPC UA (zadanie tokio `server.run()`); przeładowywalny na gorąco
  (`Reconfigure`: ponowne bindowanie, jeśli zmienia się IP/port); zachowuje
  `JoinHandle` (porzucenie przy zatrzymaniu) i `ServerHandle` (czyste anulowanie
  sesji); publikuje swój status nasłuchiwania dla GUI.
- **Serwer OPC UA** ([`opcua_server.rs`](../../src/opcua_server.rs)): buduje serwer
  [`async-opcua`](https://crates.io/crates/async-opcua), deklaruje przestrzeń
  adresową i podpina wywołania zwrotne. **Odczyty** czerpią z `SharedSnapshot`;
  **zapisy** wysyłają `Command` do `SimulationActor` przez nieblokujący `cast`.

Tak jak NAMUR (OSNE) i w przeciwieństwie do Modbus ORME, **nie ma osobnej tablicy
pamięci**: węzły OPC UA odczytują bezpośrednio współdzielony zrzut.

---

## 4. Stos OPC UA — wybory techniczne

- **`async-opcua`** (serwer, feature `server`): implementacja **natywna dla tokio**
  (jedno zadanie na połączenie), która wpasowuje się w stos ractor/tokio.
  Kryptografia **w 100 % w Rust** (RustCrypto: `rsa`, `aes`, `sha2`, `x509-cert`) —
  **żadnej zależności od OpenSSL**, co zachowuje kompilację skrośną
  (Linux/Windows/RPi).
- **Przestrzeń adresowa**: `SimpleNodeManager` w pamięci; węzły `Variable`
  zorganizowane pod `Objects` (zob. [`reference_opcua.md`](reference_opcua.md)).
- **Wywołania zwrotne**: `add_read_callback` (żywa wartość, próbkowana dla
  subskrypcji) i `add_write_callback` (kieruje do symulacji).
- **Licencja**: `async-opcua` jest na licencji **MPL-2.0** (cała linia OPC UA w
  Rust taka jest). Copyleft **na poziomie pliku**: niezmodyfikowane użycie → kod
  CESAM-Lab pozostaje MIT (zob. plik `NOTICE` w katalogu głównym).

---

## 5. Bezpieczeństwo

Bezpieczeństwo jest **konfigurowalne** (`SecurityConfig`) i stanowi wyróżnik
OPC UA na tle protokołów obiektowych (Modbus/NAMUR, bez bezpieczeństwa).

- **Tryb nieszyfrowany (domyślny)**: endpoint `SecurityPolicy::None`, token
  **anonimowy** — wyłącznie sieć zaufana, natychmiastowy start, brak certyfikatu.
  GUI wyświetla **pomarańczowy baner** ostrzegawczy.
- **Tryb szyfrowany (Faza 2)**: endpoint `Basic256Sha256` / `SignAndEncrypt`.
  Samopodpisany **certyfikat instancji** jest generowany przy pierwszym
  uruchomieniu (`pki/`); serwer ufa certyfikatom klientów. **Uwierzytelnianie**
  za pomocą użytkownika/hasła (`ServerUserToken::user_pass`) i/lub anonimowe. GUI
  wyświetla **zielony baner** 🔒.

Tryb ustawia się w modalu *Ustawienia*; zmiana **restartuje** serwer na gorąco
(`OpcuaServerActor`).

---

## 6. Konfiguracja i trwałość

`AppConfig` (język / sieć / proces / regulacja / sprawdzanie aktualizacji)
serializowany do **TOML** ([`config.rs`](../../src/config.rs)), **odkażany przy
wczytywaniu** (`AppConfig::sanitized`: granice uporządkowane, `τ ≥ 1e-3`,
`dead_time ≥ 0`, skończone liczby zmiennoprzecinkowe). Plik: `mock_ru_opcua.toml`
(nadpisywalny przez `MOCK_CONFIG`).

---

## 7. Kierunki rozwoju

- **Faza 2**: bezpieczeństwo OPC UA (certyfikaty, szyfrowanie, uwierzytelnianie).
- Metody OPC UA (`Reset`, `Autotune`) oprócz zmiennych.
- Typowany model informacyjny (ObjectType regulatora) zamiast płaskich zmiennych.
- Historyzacja / `HistoryRead` na pomiarze.
- Promocja modelu regulatora ORME do współdzielonej `mock_lib_*` (jest on dziś
  duplikowany między ORME a tym instrumentem).
