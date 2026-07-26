# Referencja S7 — plan adresowania i protokół (RU/S7)

*🌍 [FR](../fr/reference_s7.md) · [EN](../en/reference_s7.md) · [DE](../de/reference_s7.md) · [ES](../es/reference_s7.md) · [IT](../it/reference_s7.md) · [PT](../pt/reference_s7.md) · [NL](../nl/reference_s7.md) · **PL***

> Źródło prawdy: [`s7_server.rs`](../../src/s7_server.rs) (analiza ramek,
> plan adresowania DB1, mapowanie zapisów). Każda zmiana następuje **w tym
> pliku** i jest tu odzwierciedlana.

---

## 1. Endpoint

Serwer **S7comm** na **ISO-on-TCP / RFC1006**. Nasłuchuje domyślnie na
`0.0.0.0:102` (port standardowy S7; **< 1024 → uprawnienia root** wymagane, w przeciwnym razie wybierz
port wysoki). Ustawienia w sekcji `[network]` w TOML / oknie modalnym *Ustawienia*:

| Klucz | Domyślnie | Rola |
|---|---|---|
| `bind_ip` | `0.0.0.0` | IP nasłuchu |
| `port` | `102` | port TCP (standard S7) |
| `allowlist` | *(puste)* | lista dozwolonych adresów IP (wzorce `*` na bajt; puste = wszystko dozwolone) |

> ⚠️ **Brak uwierzytelniania i szyfrowania** (S7 „classic"). Jedyną kontrolą
> dostępu jest **lista dozwolonych adresów IP** + topologia sieci. `0.0.0.0` + pusta lista
> = **wystawiony na całą sieć**: GUI wyświetla baner ostrzegawczy.

## 2. Sesje

W przeciwieństwie do ORME (jeden master) serwer S7 przyjmuje **wiele jednoczesnych
sesji klientów** (zwyczajowe zachowanie sterownika). Każda sesja negocjuje
COTP (Connection Request → Confirm), następnie S7 *Setup Communication*, przed
wymianami *Read Var* / *Write Var*.

## 3. Zaimplementowany podzbiór protokołu

- **COTP**: Connection Request (CR) → Connection Confirm (CC); Data (DT).
- **S7comm**: *Setup Communication*, *Read Var* (funkcja `0x04`), *Write Var*
  (funkcja `0x05`) na bloku danych **DB1**.

Serwer udostępnia **obraz bajtów DB1** (40 bajtów). Odczyty zwracają
wycinek tego obrazu; zapisy na sterowalnych offsetach wytwarzają
zsanityzowane polecenia dla symulacji.

## 4. Plan adresowania DB1

REAL = `f32` big-endian (IEEE-754). Adresowanie po bajcie (`DBDx`) lub po bicie
(`DBXx.y`).

| Adres | Typ | Dostęp | Wielkość | Zapis → polecenie |
|---|---|:--:|---|---|
| `DB1.DBD0`  | REAL | R/W | Nastawa (Setpoint) | `SetSetpoint` |
| `DB1.DBD4`  | REAL | R   | Pomiar (ProcessValue) | — |
| `DB1.DBD8`  | REAL | R   | Wyjście (Output, %) | — |
| `DB1.DBD12` | REAL | R/W | Wyjście ręczne (ManualOutput, %) | `SetManualOutput` |
| `DB1.DBX16.0` | BOOL | R/W | Praca (Run) | `SetRun` |
| `DB1.DBX16.1` | BOOL | R/W | Tryb auto (Auto) | `SetAuto` |
| `DB1.DBD20` | REAL | R | Nastawa min | — |
| `DB1.DBD24` | REAL | R | Nastawa max | — |
| `DB1.DBD28` | REAL | R | PID Kp | — |
| `DB1.DBD32` | REAL | R | PID Ki | — |
| `DB1.DBD36` | REAL | R | PID Kd | — |

Zapis `DB1.DBB16` (bajt) jest akceptowany: bit 0 = Run, bit 1 = Auto. Każdy zapis
na offsecie tylko do odczytu jest **akceptowany, ale ignorowany** (kod powrotu sukcesu).
Odczyt/zapis poza DB1 zwraca kod powrotu S7 `0x0A` (obiekt nieistniejący).

## 5. Przykład klienta

Z klientem S7 (Snap7, `python-snap7`, nodes7…) skonfigurowanym na IP/port
serwera, **rack 0 / slot 1** (wartości zwyczajowe; serwer nie narzuca TSAP):

```python
import snap7, struct
c = snap7.client.Client()
c.connect("127.0.0.1", 0, 1, 102)
c.db_write(1, 0, struct.pack(">f", 80.0))   # Nastawa = 80.0
c.db_write(1, 16, bytes([0x01]))            # Run = true (bit 0)
pv = struct.unpack(">f", c.db_read(1, 4, 4))[0]  # Pomiar
```
