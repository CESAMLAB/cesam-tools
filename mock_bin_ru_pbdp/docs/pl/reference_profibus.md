# Dokumentacja referencyjna PROFIBUS DP-V0 — Symulowany regulator (ORPD)

*🌍 [FR](../fr/reference_profibus.md) · [EN](../en/reference_profibus.md) · [DE](../de/reference_profibus.md) · [ES](../es/reference_profibus.md) · [IT](../it/reference_profibus.md) · [PT](../pt/reference_profibus.md) · [NL](../nl/reference_profibus.md) · **PL***

> Crate: `mock_bin_ru_pbdp` · Plik wykonywalny: **ru_pbdp** · Protokół: **PROFIBUS DP-V0** (urządzenie podrzędne szeregowe)

Ten dokument jest referencją funkcjonalną symulowanego podzbioru
PROFIBUS DP-V0. **Techniczne źródło prawdy** pozostaje nagłówek
[`src/profibus.rs`](../../src/profibus.rs) (kodek + maszyna stanów) oraz
[`src/map.rs`](../../src/map.rs) (bloki I/O): każda rozbieżność musi
najpierw zostać poprawiona w kodzie.

---

## ⚠️ 0. Zakres i ograniczenia — przeczytaj przed jakimkolwiek użyciem

`ru_pbdp` implementuje **edukacyjny podzbiór** DP-V0, **bez jakichkolwiek
roszczeń do ścisłej zgodności binarnej** z tabelami normatywnymi
(IEC 61158 / EN 50170), poza najpowszechniej udokumentowanymi
elementami:

- **zgodne**: ograniczniki ramek (`SD1`/`SD2`/`SD3`/`SD4`/`SC`/`ED`), FCS
  (suma modulo 256), numery SAP usług parametryzacji
  (`Slave_Diag` = 61, `Set_Prm` = 62, `Chk_Cfg` = 63).
- **konwencje właściwe temu symulatorowi, a nie prawdziwy profil GSD
  zarejestrowany w PNO** (PROFIBUS & PROFINET International): dokładne
  kodowanie bitów pola `FC`, precyzyjne rozmieszczenie bajtów
  diagnostycznych, rozmieszczenie bloków wejścia/wyjścia (§3),
  identyfikator `Ident_Number` (§4).
- **brak jakiegokolwiek rzeczywistego czasowania magistrali**: ani okno
  odpowiedzi (*slot time*, `Tsdr` min/maks), ani żeton między masterami,
  ani arbitraż wielu masterów. Tylko dedykowany układ ASIC (SPC3/VPC3)
  lub sprzętowa karta mastera (Hilscher/Softing/Siemens CP) mogą
  spełnić te ograniczenia na poziomie bitów.

**Bezpośrednia konsekwencja: ten symulator nigdy nie zostanie rozpoznany
przez prawdziwy master PROFIBUS DP** (sterownik PLC + karta mastera).
Służy do zrozumienia struktury protokołu i testowania rozwoju
oprogramowania (kodek, maszyna stanów, narzędzia), a nie do sterowania
urządzeniami terenowymi — zob.
[`manuel_utilisateur.md`](manuel_utilisateur.md).

---

## 1. Ramki — ograniczniki i FCS

| Ogranicznik | Wartość | Zastosowanie |
|---|:--:|---|
| `SD1` | `0x10` | Stałe żądanie bez danych (6 bajtów: `SD1 DA SA FC FCS ED`) |
| `SD2` | `0x68` | Ramka o zmiennej długości z danymi (`SD2 LE LEr SD2 DA SA FC [dane…] FCS ED`) |
| `SD3` | `0xA2` | Ramka o stałych danych, 8 bajtów (14 bajtów łącznie) — **nieużywana** przez ten symulator (zob. §0), dostarczona dla kompletności kodeka i jego testów |
| `SD4` | `0xDC` | Ramka żetonu, 3 bajty, bez FCS ani ED — poza zakresem dla symulowanego urządzenia podrzędnego z jednym masterem, dostarczona dla kompletności kodeka |
| `SC` | `0xE5` | Krótkie potwierdzenie, 1 bajt |
| `ED` | `0x16` | Ogranicznik końca |

- **`FCS`**: suma modulo 256 użytecznych bajtów ramki (zob.
  `profibus::checksum`). Ramka odebrana z nieprawidłowym FCS jest
  odrzucana (`FrameError::BadChecksum`) bez odpowiedzi — master musi
  wysłać ją ponownie.
- **`DA`/`SA`**: adres docelowy / źródłowy. Bit 7 `DA` = **rozszerzenie
  adresu (DAE)**: obecność bajtu SAP zaraz po `DA` w danych użytecznych.
  Brak = domyślna wymiana danych (`Data_Exchange`). Adres stacji zajmuje
  pozostałe 7 bitów (`0`-`125`; `126`/`127` zarezerwowane przez normę,
  nieużywane tutaj).
- **Ten symulator systematycznie preferuje `SD2`** dla wszystkich wymian
  `Data_Exchange`, nawet gdy `SD3` (8 stałych bajtów) wystarczyłoby w
  prawdziwym profilu — wybór, który upraszcza kodek bez utraty pokrycia
  koncepcji protokołu (zob. [`conception.md`](conception.md) §4).
- **Zniekształcona ramka / nieznany ogranicznik (szum linii)**:
  odrzucana po cichu (`log::debug!`), sesja jest kontynuowana — pozwala
  na ponowną synchronizację strumienia bajtów bez awarii łącza.

---

## 2. Sekwencjonowanie — usługi i maszyna stanów

Symulowane urządzenie podrzędne (`SlaveFsm`,
[`profibus.rs`](../../src/profibus.rs)) przechodzi przez cztery stany:

```
PowerOn ──Slave_Diag──► WaitPrm ──Set_Prm (id OK)──► WaitCfg ──Chk_Cfg (długości OK)──► DataExchange
```

| Stan | Znaczenie | Typowa odpowiedź |
|---|---|---|
| `Power_On` | Zaraz po uruchomieniu, przed pierwszym zapytaniem diagnostycznym | — |
| `Wait_Prm` | Oczekiwanie na prawidłowe `Set_Prm` | `Diag` z `Stat_1 = STAT1_PRM_REQ` |
| `Wait_Cfg` | Sparametryzowane, oczekiwanie na prawidłowe `Chk_Cfg` | `Diag` z `Stat_1 = STAT1_CFG_FAULT` |
| `Data_Exchange` | Sparametryzowane i skonfigurowane: cykliczna wymiana aktywna | blok wejściowy (§3) |

### `Slave_Diag` (SAP 61)

Żądanie bez danych (lub ramka `SD1`, zgodnie z konwencją tego symulatora
zawsze interpretowana jako `Slave_Diag` — na `SD1` niemożliwe jest
żadne rozszerzenie adresu, ponieważ brak wolnego bajtu na przeniesienie
SAP). Odpowiedź `Diag` (6 bajtów):

| Bajt | Symbol | Zawartość |
|:--:|---|---|
| `0` | `Stat_1` | `0x01` (`STAT1_PRM_REQ`, dopóki nie sparametryzowane) lub `0x02` (`STAT1_CFG_FAULT`, dopóki nie skonfigurowane) lub `0x00` (`Data_Exchange`) |
| `1` | `Stat_2` | zawsze `0x00` (niesymulowane) |
| `2` | `Stat_3` | zawsze `0x00` (niesymulowane) |
| `3` | `Master_Add` | `0xFF` (brak znanego mastera — nieśledzone przez ten symulator) |
| `4-5` | `Ident_Number` | stały identyfikator urządzenia podrzędnego, big-endian (§4) |

Pierwsze odebrane `Slave_Diag` powoduje przejście `Power_On` →
`Wait_Prm`; kolejne nie zmieniają stanu (tylko odczyt diagnostyczny).

### `Set_Prm` (SAP 62)

Żądanie: `SAP(62) Ident_Number(2, BE) WD_Fact_1(1) WD_Fact_2(1)`.
Zapowiedziany watchdog, jeśli obecny, jest obliczany jako
`watchdog_ms = WD_Fact_1 × WD_Fact_2 × 10` (jednostka 10 ms, standardowa
konwencja DP); `WD_Fact_1 = 0` **lub** `WD_Fact_2 = 0` oznacza „brak
watchdoga”. Odpowiedź: w każdym przypadku `ShortAck` (`SC`).

- Jeśli `Ident_Number` **odpowiada** stałemu profilowi urządzenia
  podrzędnego (§4): stan → `Wait_Cfg`, a ewentualny watchdog jest
  przekazywany do sesji (uzbrajany tylko jeśli lokalne ustawienie
  `watchdog_enabled` na to pozwala — zob.
  [`manuel_utilisateur.md`](manuel_utilisateur.md) §4).
- Jeśli identyfikator **nie odpowiada**: parametryzacja jest odrzucana po
  cichu (`ShortAck` mimo to zwracany, zgodnie z zaleceniem DP-V0 dla tej
  usługi, ale bez wpływu na stan wewnętrzny) — urządzenie podrzędne
  pozostaje w `Wait_Prm`.

### `Chk_Cfg` (SAP 63)

Żądanie: `SAP(63) Out_Len(1) In_Len(1)`. Odpowiedź: `ShortAck`. Stan
przechodzi do `Data_Exchange` **tylko jeśli** `Out_Len == 45` i
`In_Len == 17` (stałe rozmiary symulowanego profilu, §3) **i** urządzenie
podrzędne było w stanie `Wait_Cfg`; w przeciwnym razie stan się nie
zmienia (master musi ponownie wysłać prawidłowe `Chk_Cfg`).

### `Data_Exchange` (bez SAP — domyślny adres, brak bitu DAE)

Żądanie: surowy blok wyjściowy (45 bajtów, §3). Odpowiedź: blok
wejściowy (17 bajtów, §3), przeliczany na bieżąco ze współdzielonego
zrzutu w momencie odpowiedzi (brak trwałej tabeli pamięci, w
przeciwieństwie do Modbus/ORME).

Jeśli master wyśle `Data_Exchange` **przed** osiągnięciem stanu
`Data_Exchange` (sekwencjonowanie niezachowane), urządzenie podrzędne
odpowiada bieżącą diagnostyką (`Diag`) zamiast ulec awarii lub
zignorować ramkę.

---

## 3. Bloki I/O — rozmieszczenie bajtów

Skopiowane z nagłówka [`map.rs`](../../src/map.rs), jedynego źródła
prawdy w przypadku rozbieżności. Wszystkie wartości zmiennoprzecinkowe
(`f32`) zajmują **4 kolejne bajty, big-endian**.

### Blok wyjściowy — *Output* (master → urządzenie podrzędne, `OUTPUT_LEN` = 45 bajtów)

| Bajt(y) | Symbol | Typ | Opis |
|---|---|:--:|---|
| `0` | `OUT_MODE` | bity | bit0 = praca, bit1 = auto, [3:2] = tryb kierunku 1, [5:4] = tryb kierunku 2 |
| `1-4` | `OUT_SP_AUTO` | f32 | Wartość zadana automatyczna |
| `5-8` | `OUT_SP_MANUAL` | f32 | Wartość zadana ręczna (% wyjścia, ze znakiem) |
| `9-12` | `OUT_KP1` | f32 | Wzmocnienie proporcjonalne Kp, kierunek 1 |
| `13-16` | `OUT_KI1` | f32 | Wzmocnienie całkujące Ki, kierunek 1 |
| `17-20` | `OUT_KD1` | f32 | Wzmocnienie różniczkujące Kd, kierunek 1 |
| `21-24` | `OUT_KP2` | f32 | Wzmocnienie proporcjonalne Kp, kierunek 2 |
| `25-28` | `OUT_KI2` | f32 | Wzmocnienie całkujące Ki, kierunek 2 |
| `29-32` | `OUT_KD2` | f32 | Wzmocnienie różniczkujące Kd, kierunek 2 |
| `33-36` | `OUT_HYSTERESIS` | f32 | Histereza regulatorów dwustawnych |
| `37-40` | `OUT_TOR_MIN_CYCLE` | f32 | Minimalny czas cyklu dwustawnego (s) |
| `41-44` | `OUT_PWM_PERIOD` | f32 | Okres cyklu modulacji PWM (s) |

Kody trybów (`[3:2]`/`[5:4]`) są zgodne z `ControllerKind`: `0` = Wył.,
`1` = PID, `2` = Dwustawny, `3` = PWM (zob. `mock_lib_control`).

### Blok wejściowy — *Input* (urządzenie podrzędne → master, `INPUT_LEN` = 17 bajtów)

| Bajt(y) | Symbol | Typ | Opis |
|---|---|:--:|---|
| `0` | `IN_STATUS` | bity | bit0 = w ruchu, bit1 = kierunek 1 aktywny (wyjście > 0), bit2 = kierunek 2 aktywny (wyjście < 0) |
| `1-4` | `IN_PV` | f32 | Pomiar / *process value* |
| `5-8` | `IN_OUTPUT` | f32 | Zastosowane wyjście (% ze znakiem) |
| `9-12` | `IN_SP_AUTO` | f32 | Odczyt zwrotny (tylko do odczytu) wartości zadanej automatycznej |
| `13-16` | `IN_SP_MANUAL` | f32 | Odczyt zwrotny (tylko do odczytu) wartości zadanej ręcznej |

Zbyt krótki blok wyjściowy (< 45 bajtów) jest ignorowany bez awarii: nie
jest generowany żaden `Command`, regulator zachowuje swój ostatni
prawidłowy stan.

---

## 4. Stały profil urządzenia podrzędnego

| Parametr | Wartość | Uwaga |
|---|---|---|
| `Ident_Number` | `0xEE01` | **Fikcyjny**, niezarejestrowany w PNO — nie reprezentuje żadnego rzeczywistego urządzenia katalogowego |
| `Out_Len` | `45` | Oczekiwany w `Chk_Cfg.out_len` |
| `In_Len` | `17` | Oczekiwany w `Chk_Cfg.in_len` |
| Adres stacji | `0`-`125`, konfigurowalny | Ustawienie lokalne (okno modalne *Ustawienia*), zob. [`manuel_utilisateur.md`](manuel_utilisateur.md) §4 |
| Format ramki szeregowej | `8E1` (8 bitów, parzystość parzysta, 1 bit stopu) | **Ustalony przez normę PROFIBUS DP**, nieregulowalny |
| Znormalizowane prędkości transmisji | od `9600` do `12 000 000` bit/s | Niesprawdzane przy otwarciu: niestandardowa wartość jest przekazywana bez zmian do portu szeregowego |

---

## 5. Watchdog protokołu

W przeciwieństwie do watchdoga NAMUR w OSNE (domowy dodatek), ten jest
**prawdziwą częścią protokołu DP**: jest **zapowiadany przez mastera** w
`Set_Prm` (współczynniki `WD_Fact_1`/`WD_Fact_2`, §2) i jest **uzbrajany
po stronie urządzenia podrzędnego** tylko wtedy, gdy zezwala na to
lokalne ustawienie `watchdog_enabled` (w przeciwnym razie żądanie mastera
jest ignorowane, nigdy nie uzbrajane). Po upływie czasu, bez odebrania
nowej ramki dla stacji, urządzenie podrzędne wymusza stan bezpieczny
(`Command::SetOnOff(false)`) — udokumentowane uproszczenie: prawdziwy
profil DP-V0 mógłby wymagać pełnego powrotu poprzez `Set_Prm`/`Chk_Cfg`
przed wznowieniem wymiany, czego ten symulator nie wymaga jawnie
(wystarczy wznowić wysyłanie ramek `Data_Exchange`, ponieważ stan
`Data_Exchange` nie jest opuszczany po upływie watchdoga).

---

## 6. Brak interoperacyjności — dlaczego

| Wymóg prawdziwego PROFIBUS DP | Ten symulator |
|---|---|
| Okno odpowiedzi na poziomie bitów (*slot time*, `Tsdr` min/maks) | Brak — odpowiada natychmiast po zdekodowaniu ramki, bez ograniczenia czasowego |
| Dedykowany układ (ASIC SPC3/VPC3) do czasowania | Brak — zwykłe oprogramowanie Tokio |
| Żeton między masterami, arbitraż wielu masterów | Brak — urządzenie podrzędne z jednym masterem, łącze punkt-punkt |
| Profil GSD zarejestrowany w PNO | Brak — profil I/O właściwy temu symulatorowi (§3) |
| Dokładne bitowo kodowanie pól FC/diagnostycznych | Konwencja symulacji, niegwarantowana zgodność |

**Prawdziwy sterownik PLC (np. Siemens S7 z kartą mastera) nigdy nie
rozpozna tego symulatora jako prawidłowego urządzenia podrzędnego na
rzeczywistej magistrali PROFIBUS DP RS-485.** Dwie instancje tego
symulatora (lub skrypt odtwarzający poniższą sekwencję) mogą jednak
prowadzić dialog między sobą, aby zilustrować protokół — zob.
[`manuel_utilisateur.md`](manuel_utilisateur.md) §5.

---

## 7. Przykładowa sekwencja (szesnastkowo)

Pełna sekwencja dla stacji `5`, mastera `3`, aż do wymiany cyklicznej
(wartości ilustracyjne, `FCS` obliczone na użytecznych bajtach):

```text
# 1. Slave_Diag (SD2, DAE=1, SAP=61)
→ TX  68 03 03 68 85 03 C0 3D FC 16
← RX  68 06 06 68 03 85 00 01 00 00 FF EE 01 F5 16   (Diag: Stat_1=0x01, Ident=0xEE01)

# 2. Set_Prm (SD2, DAE=1, SAP=62, Ident=0xEE01, WD=1×30×10ms=300ms)
→ TX  68 07 07 68 85 03 C0 3E EE 01 01 1E … 16
← RX  E5                                              (ShortAck)

# 3. Chk_Cfg (SD2, DAE=1, SAP=63, out_len=45, in_len=17)
→ TX  68 05 05 68 85 03 C0 3F 2D 11 … 16
← RX  E5                                              (ShortAck)

# 4. Data_Exchange (SD2, bez SAP, blok wyjściowy 45 bajtów)
→ TX  68 30 30 68 05 03 C0 [45 bajtów] … 16
← RX  68 14 14 68 03 85 00 [17 bajtów]  … 16          (blok wejściowy)
```

Dokładne bajty FCS/długości zależą od wartości danych użytecznych; ten
schemat ilustruje **kolejność usług**, a nie ramkę do odtworzenia
dosłownie. Zob. testy w [`profibus.rs`](../../src/profibus.rs) i
[`profibus_server.rs`](../../src/profibus_server.rs) dla sekwencji
zweryfikowanych bit po bicie.
