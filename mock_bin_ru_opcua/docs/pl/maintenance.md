# Dokumentacja konserwacji — RU/OPC UA (workspace `cesam-tools`)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · **PL***

> Crate: `mock_bin_ru_opcua` · Plik wykonywalny: **ru_opcua**

---

## 1. Wymagania wstępne

- **Rust** w nowej wersji. ⚠️ MSRV właściwy dla tego crate'a: **1.91**
  (`async-opcua` nie deklaruje żadnego `rust-version` i ciągnie świeże zależności;
  reszta workspace'u jest na 1.85).
- Dla GUI: zależności systemowe `eframe`/`egui` (te same co ORME/OSNE).
- Dla buildu *headless*: brak zależności graficznych.

---

## 2. Najczęstsze polecenia

```bash
cargo run -p mock_bin_ru_opcua                       # GUI + serwer OPC UA
cargo run -p mock_bin_ru_opcua --no-default-features # headless (bez GUI)
cargo test -p mock_bin_ru_opcua                      # testy jednostkowe
cargo clippy -p mock_bin_ru_opcua --all-targets      # lint
MOCK_CONFIG=./ma_config.toml cargo run -p mock_bin_ru_opcua  # alternatywna konfiguracja
```

### Features Cargo

- **`gui`** (domyślnie): interfejs graficzny `egui` + sprawdzanie aktualizacji.
- `--no-default-features`: binarka **headless** (serwer OPC UA + symulacja, bez
  GUI ani sieci aktualizacji).

Serwer `async-opcua` jest **zawsze** obecny (feature `server` z `async-opcua`),
ponieważ to racja bytu tego instrumentu.

---

## 3. Organizacja kodu

```
mock_bin_ru_opcua/src/
├── main.rs            # Składa środowisko Tokio + aktorzy + GUI/headless
├── regulator.rs       # Synchroniczny model biznesowy (PID + proces), komendy, krok
├── config.rs          # AppConfig (TOML), sanitized(), ServerStatus
├── i18n.rs            # Katalog i18n (8 języków), Lang + Msg + tr()
├── opcua_server.rs    # Serwer OPC UA: build + przestrzeń adresowa + wywołania zwrotne
├── gui.rs             # GUI egui (feature gui)
├── branding.rs        # Wbudowane logo (feature gui)
└── actors/
    ├── simulation.rs  #   pętla regulacji (tick 0,5 s)
    └── network.rs     #   serwer OPC UA (re)konfigurowalny na gorąco
```

---

## 4. Konfiguracja

`AppConfig` (język / sieć / proces / regulacja / `check_updates`) jest
serializowany do **TOML** (`mock_ru_opcua.toml`, nadpisywalny przez `MOCK_CONFIG`),
wczytywany przy starcie (domyślne, jeśli nieobecny), zapisywany z GUI. Każda
wartość jest **odkażana** przy wczytywaniu (`AppConfig::sanitized`: granice
uporządkowane, `τ ≥ 1e-3`, `dead_time ≥ 0`, skończone liczby zmiennoprzecinkowe).

**Niezmiennik**: nigdy nie wywoływać `f32::clamp` z niezweryfikowanymi granicami
(panika, jeśli `min > max` lub `NaN`). Zapisy sieciowe również przechodzą przez
`Regulator::apply`, który odkaża.

### Sprawdzanie aktualizacji

Tylko feature `gui`: przy starcie GUI odpytuje ostatnie wydanie GitHub przez
współdzieloną bibliotekę `mock_lib_update` (wątek ograniczony timeoutem) i
wyświetla baner, jeśli istnieje nowsza wersja. Regulowane przez `check_updates`.

---

## 5. Zależności i pułapki wersji

- **`async-opcua` 0.18** (serwer). Kryptografia **w 100 % w Rust** (RustCrypto):
  **żadnej zależności od OpenSSL** → czysta kompilacja skrośna. Licencja
  **MPL-2.0** (zob. `NOTICE`).
- ⚠️ `async-opcua` nie deklaruje **żadnego MSRV**: zweryfikuj na docelowym
  toolchainie przed podbiciem wersji.
- ⚠️ Certyfikat instancji (`create_sample_keypair(true)` + `pki/`) jest generowany
  **tylko w trybie szyfrowanym** (`security.encryption`). W trybie None (domyślnie)
  żaden certyfikat (natychmiastowy start). ⚠️ Generowanie RSA w czystym Rust jest
  wolne w trybie *debug*: licz się z kilkoma sekundami przy pierwszym przejściu
  w tryb szyfrowany.
- `egui_plot` pozostaje **o jedną wersję minor wyprzedzony** względem `egui` (zob.
  ORME/OSNE).

---

## 6. Rozszerzanie projektu

### 6.1 Dodanie węzła OPC UA

W [`opcua_server.rs`](../../src/opcua_server.rs): zadeklaruj węzeł (`add_var`),
podepnij wywołanie zwrotne odczytu (`on_read_*`) oraz, jeśli zapisywalny,
wywołanie zwrotne zapisu (`on_write_*`), które wysyła `Command`. Odzwierciedl
tablicę w [`reference_opcua.md`](reference_opcua.md).

### 6.2 Dodanie komendy biznesowej

Rozszerz enum `Command` ([`regulator.rs`](../../src/regulator.rs)), obsłuż
przypadek w `Regulator::apply` (z odkażaniem), dodaj test.

### 6.3 Dodanie ciągu interfejsu (i18n)

Dodaj wariant do `Msg` ([`i18n.rs`](../../src/i18n.rs)) oraz **8 tłumaczeń**
(tablica o stałym rozmiarze weryfikowana przy kompilacji).

### 6.4 Bezpieczeństwo (`SecurityConfig`)

Bezpieczeństwo jest zaimplementowane w
[`opcua_server.rs`](../../src/opcua_server.rs): `security.encryption` dodaje
endpoint `Basic256Sha256`/`SignAndEncrypt` z automatycznie generowanym
certyfikatem oraz tokenami anonimowym i/lub użytkownik/hasło
(`ServerUserToken::user_pass`). Filtr logów
`opcua_crypto::certificate_store=off` ([`main.rs`](../../src/main.rs)) dotyczy
wyłącznie trybu None (brak certyfikatu); w trybie szyfrowanym pozostaje bez
skutku. Kierunki: polityki `Aes256Sha256RsaPss`, jawna lista zaufania PKI zamiast
`trust_client_certs`, tokeny X.509.

---

## 7. Strategia testów

Rdzeń biznesowy (`regulator.rs`) i konfiguracja (`config.rs`) są **czyste i
testowane**: zbieżność PID, clamp wartości zadanej, relaksacja po zatrzymaniu,
zmiana procesu bez skoku PV, odkażanie TOML, podróż tam i z powrotem TOML. i18n
weryfikuje niepustość i podróż tam i z powrotem języka. Logika async (aktorzy,
serwer) pozostaje cienka i opiera się na tych przetestowanych elementach.

---

## 8. Rozwiązywanie problemów

| Objaw | Prawdopodobna przyczyna | Środek zaradczy |
|---|---|---|
| `failed to bind` przy starcie | port już zajęty / < 1024 bez uprawnień | zmień port (*Ustawienia*) lub uruchom jako root |
| Klient nie widzi węzłów | zły endpoint / bezpieczeństwo | `opc.tcp://…:4840/`, None, Anonymous; *Browse* pod `Objects` |
| Zapis `Bad_TypeMismatch` | niepoprawny typ | `Double` dla wielkości, `Boolean` dla `Run`/`Auto` |
| WARN „encrypted endpoints disabled” | brak certyfikatu (Faza 1b) | normalne; endpoint None działa |

---

## 9. Build „prod” — kompilacja skrośna z Linuksa

Instrument jest zintegrowany z [`scripts/build-prod.sh`](../../../scripts/build-prod.sh)
(tablica `INSTRUMENTS`): pliki wykonywalne **z GUI** dla Linux x86_64, Windows
x86_64 i Raspberry Pi arm64 (przez `cross`) oraz obraz Docker headless.

⚠️ **Cross Windows i `GetHostNameW`**: stos OPC UA ciągnie `gethostname`, który
odwołuje się do symbolu winsock `GetHostNameW`. Biblioteka importu mingw-w64 z
obrazu `cross` **domyślnego** (`:0.2.5`) jest zbyt stara, by go dostarczyć →
niepowodzenie konsolidacji. Repozytorium ustala więc, w
[`Cross.toml`](../../../Cross.toml), obraz Windows GNU na **`:main`** (świeży
mingw). Zweryfikowane: buildy headless **oraz** GUI produkują prawidłowy `.exe`;
ORME/OSNE wciąż się kompilują (obraz nadzbiór).

---

## 10. Konwencje

- Kod i komentarze po **francusku**; logi/błędy po **angielsku**.
- Ciągi GUI przez `i18n` (8 języków); nigdy zakodowane na sztywno.
- Logika biznesowa **synchroniczna i testowalna**; asynchroniczność ograniczona do
  aktorów i IO. `cargo clippy --workspace` bez ostrzeżeń.
- Niezmienniki `ractor`: brak guard `Mutex` przez `.await`; brak odłączonego
  timera/`spawn` bez `JoinHandle` porzucanego przy zatrzymaniu.
