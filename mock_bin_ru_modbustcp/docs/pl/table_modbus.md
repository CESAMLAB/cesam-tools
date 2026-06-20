# Tablica adresów Modbus — Symulowany regulator

*🌍 [FR](../fr/table_modbus.md) · [EN](../en/table_modbus.md) · [DE](../de/table_modbus.md) · [ES](../es/table_modbus.md) · [IT](../it/table_modbus.md) · [PT](../pt/table_modbus.md) · [NL](../nl/table_modbus.md) · **PL***

> Crate: `mock_bin_ru_modbustcp` · Protokół: **Modbus TCP** (slave / serwer)

Niniejszy dokument jest funkcjonalnym odniesieniem planu adresowania.
**Technicznym źródłem prawdy** pozostaje nagłówek [`src/map.rs`](../../src/map.rs):
każdą rozbieżność należy w pierwszej kolejności poprawić w kodzie.

---

## 1. Informacje ogólne

| Element | Wartość |
|---------|--------|
| Transport | Modbus **TCP** lub **RTU szeregowy / RS485** (tylko jeden aktywny naraz) |
| Rola | **Slave** (serwer) |
| Port domyślny | TCP `5502` (konfigurowalny, modal *Parametry*) |
| Szeregowy (RTU) | port + baud + parzystość + bity, konfigurowalne (feature `rtu`) |
| Unit ID / adres | TCP: dowolny. RTU: `slave_id` konfigurowalny, ale **niefiltrowany** (zob. uwaga) |
| Mastery | **tylko jeden zdalny master naraz**; w TCP nowo przybyły odłącza poprzedniego (lokalne GUI nie jest masterem) |
| Adresowanie | **baza 0** (adres `0` = 1. element tablicy) |
| Filtrowanie | opcjonalna biała lista IP (znaki wieloznaczne `*`, tylko TCP) |

> **Uwaga RTU / adres slave**: serwer RTU odpowiada **niezależnie od żądanego
> adresu** (adres nie jest przekazywany do usługi aplikacyjnej). Użyć
> **połączenia punkt-punkt**. `slave_id` jest zachowywany/wyświetlany, ale nie
> realizuje filtrowania.

### Adresowanie baza 0 vs baza 1

Poniższe adresy to **adresy protokolarne (baza 0)**, takie jak wysyłane w ramce.
Wiele narzędzi wyświetla „konwencjonalną” numerację baza 1 (`4xxxx` dla holdingów,
`3xxxx` dla inputów…). Tak więc rejestr holding o adresie `2` odpowiada
konwencjonalnemu oznaczeniu `40003`.

---

## 2. Kodowanie liczb zmiennoprzecinkowych (`f32`)

Wielkości analogowe to **`f32` IEEE-754 na 2 kolejnych rejestrach**:

- **kolejność słów**: słowo **bardziej znaczące najpierw** (big-endian, tzw. *ABCD*);
- **kolejność bajtów** w każdym rejestrze: big-endian (standard Modbus).

Przykład: `80.0` → bajty `42 A0 00 00` → rejestr `n` = `0x42A0`,
rejestr `n+1` = `0x0000`.

> Jeśli Twój klient odczytuje nieprawidłowe wartości, to prawie zawsze problem
> kolejności słów (spróbuj *word swap* / *CDAB*).

---

## 3. Cewki — *Coils* (odczyt/zapis)

Kody funkcji: `0x01` (odczyt), `0x05` (zapis pojedynczy), `0x0F` (zapis wielokrotny).

| Adres | Oznaczenie | Wartości | Efekt |
|---------|-------------|---------|-------|
| `0` | Start / Stop | `0` = stop, `1` = start | Aktywuje regulację |
| `1` | Auto / Ręczny | `0` = ręczny, `1` = auto | Wybór trybu |

---

## 4. Wejścia dwustanowe — *Discrete Inputs* (tylko odczyt)

Kod funkcji: `0x02`.

| Adres | Oznaczenie | Znaczenie |
|---------|-------------|---------------|
| `0` | W ruchu | Urządzenie jest w ruchu |
| `1` | Kierunek 1 (grzanie) aktywny | Wyjście > 0 |
| `2` | Kierunek 2 (chłodzenie) aktywny | Wyjście < 0 |

---

## 5. Rejestry przechowujące — *Holding Registers* (odczyt/zapis)

Kody funkcji: `0x03` (odczyt), `0x06` (zapis pojedynczy), `0x10` (zapis wielokrotny).

| Adres | Oznaczenie | Typ | Jednostka / wartości |
|---------|-------------|------|-----------------|
| `0` | Tryb regulacji kierunek 1 (grzanie) | `u16` | `0`=Off, `1`=PID, `2`=TOR, `3`=PWM |
| `1` | Tryb regulacji kierunek 2 (chłodzenie) | `u16` | `0`=Off, `1`=PID, `2`=TOR, `3`=PWM |
| `2`–`3` | Nastawa automatyczna (SP) | `f32` | jednostka pomiaru |
| `4`–`5` | Nastawa ręczna | `f32` | % wyjścia, ze znakiem (−100…+100) |
| `6`–`7` | `Kp` kierunek 1 | `f32` | wzmocnienie proporcjonalne |
| `8`–`9` | `Ki` kierunek 1 | `f32` | wzmocnienie całkujące (s⁻¹) |
| `10`–`11` | `Kd` kierunek 1 | `f32` | wzmocnienie różniczkujące (s) |
| `12`–`13` | `Kp` kierunek 2 | `f32` | wzmocnienie proporcjonalne |
| `14`–`15` | `Ki` kierunek 2 | `f32` | wzmocnienie całkujące (s⁻¹) |
| `16`–`17` | `Kd` kierunek 2 | `f32` | wzmocnienie różniczkujące (s) |
| `18`–`19` | Histereza TOR | `f32` | jednostka pomiaru |
| `20`–`21` | Minimalny czas cyklu TOR | `f32` | sekundy (zabezpieczenie przed krótkim cyklem, `0` = wyłączone) |
| `22`–`23` | Okres cyklu PWM | `f32` | sekundy (> 0) |
| `42`–`46` | Identyfikator urządzenia | `ASCII` | „CESAM-Lab” (tylko odczyt, 2 znaki/rejestr, bardziej znaczący najpierw) |

> Rejestry `24`–`41` zarezerwowane (odczytywane jako `0`).

> **Zapis częściowy `f32`**: trzeba zapisać **oba rejestry** wartości
> zmiennoprzecinkowej, aby została uwzględniona. Zapis pojedynczego rejestru pary
> `f32` jest ignorowany (i zwraca wyjątek *Illegal Data Address*, jeśli nie obejmuje
> żadnego prawidłowego pola).
>
> Zapisane wzmocnienia są ograniczane do skończonych wartości ≥ 0 (odporność).

---

## 6. Rejestry wejściowe — *Input Registers* (tylko odczyt)

Kod funkcji: `0x04`.

| Adres | Oznaczenie | Typ | Jednostka |
|---------|-------------|------|-------|
| `0`–`1` | Pomiar (PV — *process value*) | `f32` | jednostka pomiaru |
| `2`–`3` | Zastosowane wyjście | `f32` | % ze znakiem (+ grzanie / − chłodzenie) |

---

## 7. Wyjątki Modbus

| Kod | Nazwa | Przyczyna w tym urządzeniu |
|------|-----|--------------------------|
| `0x01` | Illegal Function | Nieobsługiwany kod funkcji (np. maska, FIFO) |
| `0x02` | Illegal Data Address | Adres / ilość poza tablicą lub zapis niecelujący w żadne pole |
| `0x04` | Server Device Failure | Wewnętrzna blokada niedostępna (przypadek nietypowy) |

---

## 8. Przykłady z `mbpoll`

`mbpoll` adresuje w **bazie 1**; dodajemy więc `1` do adresów baza 0.

```bash
# Uruchomić (coil base0 0 -> -t 0 -r 1), następnie przełączyć na auto (coil 1)
mbpoll -m tcp -p 5502 -a 1 -t 0 -r 1 127.0.0.1 1     # On/Off = 1
mbpoll -m tcp -p 5502 -a 1 -t 0 -r 2 127.0.0.1 1     # Auto/Manuel = 1 (auto)

# Zapisać nastawę auto (HR base0 2-3 -> -t 4:float -r 3) na 80.0
mbpoll -m tcp -p 5502 -a 1 -t 4:float -r 3 127.0.0.1 80.0

# Odczytać pomiar PV (IR base0 0-1 -> -t 3:float -r 1)
mbpoll -m tcp -p 5502 -a 1 -t 3:float -r 1 127.0.0.1

# Odczytać wyjście (IR base0 2-3 -> -t 3:float -r 3)
mbpoll -m tcp -p 5502 -a 1 -t 3:float -r 3 127.0.0.1
```

> W zależności od wersji `mbpoll` kolejność słów zmiennoprzecinkowych może wymagać
> opcji permutacji. W razie niespójnej wartości sprawdź kolejność słów.

---

## 9. Skrócona mapa pamięci

```
Coils (RW)            DiscreteInputs (RO)     Holding (RW)              Input (RO)
0  On/Off             0  W ruchu              0  Tryb kier.1 (u16)      0-1 PV (f32)
1  Auto/Ręczny        1  Grzanie aktywne      1  Tryb kier.2 (u16)      2-3 Wyjście (f32)
                      2  Chłodzenie aktywne   2-3  SP auto (f32)
                                              4-5  SP ręczna (f32)
                                              6-7  Kp1  8-9  Ki1  10-11 Kd1
                                              12-13 Kp2 14-15 Ki2 16-17 Kd2
                                              18-19 Histereza (f32)
                                              20-21 Min. cykl TOR (f32, s)
                                              22-23 Okres PWM (f32, s)
                                              42-46 Identyfikator ASCII "CESAM-Lab"
```

> **Identyfikator ASCII** (`HR 42-46`): „CESAM-Lab” zakodowany po 2 znaki na
> rejestr, znak bardziej znaczący najpierw (`42`=`'CE'`, `43`=`'SA'`, `44`=`'M-'`,
> `45`=`'La'`, `46`=`'b\0'`). Tylko odczyt. Przykład:
> `mbpoll -m tcp -p 5502 -a 1 -t 4 -r 43 -c 5 127.0.0.1` (rejestry baza 1 43..47).
