# Zestaw komend NAMUR — Symulowane mieszadło (OSNE)

*🌍 [FR](../fr/commandes_namur.md) · [EN](../en/commandes_namur.md) · [DE](../de/commandes_namur.md) · [ES](../es/commandes_namur.md) · [IT](../it/commandes_namur.md) · [PT](../pt/commandes_namur.md) · [NL](../nl/commandes_namur.md) · **PL***

> Crate: `mock_bin_su_namur` · Plik wykonywalny: **OSNE** · Protokół: **NAMUR** (ASCII, slave)

Referencja funkcjonalna protokołu. **Techniczne źródło prawdy** to nagłówek pliku
[`src/namur.rs`](../../src/namur.rs).

---

## 1. Informacje ogólne

| Element | Wartość |
|---------|--------|
| Transport | **TCP** (port `4001` domyślnie) lub **szeregowy RS-232** (feature `serial`) |
| Rola | **Slave** (odpowiada na żądania mastera) |
| Ramka | jedna **linia ASCII** na żądanie, zakończona `CR LF` |
| Odczyty | `IN_*` → zwracają `wartość kanał` (np. `1200.0 4`) |
| Zapisy / akcje | `OUT_*`, `START_*`, `STOP_*`, `RESET` → **ciche** (brak odpowiedzi) |
| Masterzy | **tylko jeden naraz** (punkt-punkt); w TCP nowy master czeka aż poprzedni się rozłączy |
| Filtrowanie | opcjonalna biała lista IP (TCP) |

> Typowe ustawienia szeregowe NAMUR: **9600 bodów, 7 bitów, parzystość parzysta, 1 stop (7E1)**.

### Kanały

| Kanał | Wielkość | Jednostka |
|-------|----------|-------|
| `4` | Prędkość | tr/min |
| `5` | Moment obrotowy | N·cm |

---

## 2. Komendy

| Komenda | Typ | Efekt | Odpowiedź |
|----------|------|-------|---------|
| `IN_NAME` | odczyt | Nazwa urządzenia | `CESAM-STIRRER` |
| `IN_TYPE` | odczyt | Typ urządzenia | `OSNE` |
| `IN_SW_VERSION` | odczyt | Wersja symulowanego firmware'u | np. `0.1.0` |
| `IN_PV_4` | odczyt | Prędkość **zmierzona** | `<v> 4` |
| `IN_PV_5` | odczyt | Moment **zmierzony** | `<c> 5` |
| `IN_SP_4` | odczyt | Wartość zadana prędkości | `<v> 4` |
| `OUT_SP_4 <v>` | zapis | **Ustawienie** wartości zadanej prędkości (tr/min) | — |
| `START_4` | akcja | Uruchomienie silnika | — |
| `STOP_4` | akcja | Zatrzymanie silnika | — |
| `RESET` | akcja | Zatrzymanie + powrót do sterowania lokalnego | — |
| `OUT_WD1@<m>` | zapis | **Watchdog**: bezpieczne zatrzymanie, jeśli brak komend przez `<m>` s | — |
| `OUT_WD2@<m>` | zapis | Watchdog (jak v1: bezpieczne zatrzymanie) | — |

> Każda nieznana komenda lub nieprawidłowy argument jest **ignorowana** (brak
> odpowiedzi) i zapisywana w `debug`.

### Watchdog

Po `OUT_WD1@30`, jeśli **żadna linia** nie nadejdzie przez 30 s, silnik zostaje
automatycznie **zatrzymany** (`STOP`) — zabezpieczenie na wypadek utraty
komunikacji z systemem nadzoru. `OUT_WD1@0` rozbraja watchdog. Licznik jest
**zerowany przy każdej odebranej komendzie**.

---

## 3. Przykłady (`nc` / netcat)

```text
$ nc 127.0.0.1 4001
IN_NAME
CESAM-STIRRER
OUT_SP_4 1200          (cicho)
START_4                (cicho)
IN_PV_4
1200.0 4
IN_PV_5
62.0 5
STOP_4                 (cicho)
```

> Odczytany **moment obrotowy** rośnie wraz z ustawioną **lepkością** (po stronie
> GUI) i prędkością: `moment ≈ coeff_charge · lepkość · prędkość + tarcie`. Przy
> dużej lepkości moment nasyca się przy maksimum silnika: zadana prędkość nie jest
> już osiągana (**przeciążenie**), zachowanie odwzorowujące rzeczywiste mieszadło.
