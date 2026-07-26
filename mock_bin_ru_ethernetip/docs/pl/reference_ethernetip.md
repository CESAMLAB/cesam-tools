# Referencja EtherNet/IP — tagi i protokół (RU/EtherNet/IP)

*🌍 [FR](../fr/reference_ethernetip.md) · [EN](../en/reference_ethernetip.md) · [DE](../de/reference_ethernetip.md) · [ES](../es/reference_ethernetip.md) · [IT](../it/reference_ethernetip.md) · [PT](../pt/reference_ethernetip.md) · [NL](../nl/reference_ethernetip.md) · **PL***

> Źródło prawdy: [`eip_server.rs`](../../src/eip_server.rs) (enkapsulacja, dyspozytor
> CIP, tabela tagów). Każda zmiana następuje **w tym pliku** i przenosi się tutaj.

---

## 1. Endpoint

Adapter **EtherNet/IP** (jawna komunikacja **CIP** nienawiązana) na TCP. Domyślnie
nasłuchuje na `0.0.0.0:44818` (standardowy port EtherNet/IP, > 1024 → brak wymaganych
uprawnień). Ustawienia w sekcji `[network]` pliku TOML / w modalu *Ustawienia*:

| Klucz | Domyślnie | Rola |
|---|---|---|
| `bind_ip` | `0.0.0.0` | IP nasłuchu |
| `port` | `44818` | port TCP (standard EtherNet/IP) |
| `allowlist` | *(puste)* | biała lista IP (wzorce `*` na oktet; puste = wszystko dozwolone) |

> ⚠️ **Brak uwierzytelniania i szyfrowania** (EtherNet/IP „classic"). Jedyną kontrolą
> dostępu jest **biała lista IP** + topologia sieci. `0.0.0.0` + pusta lista =
> **wystawione**: GUI wyświetla baner ostrzegawczy.

⚠️ EtherNet/IP / CIP jest **little-endian** (w przeciwieństwie do Modbus/S7). `REAL` to
`f32` IEEE-754 little-endian.

## 2. Sesje

Akceptowanych jest wielu **jednoczesnych** klientów. Każda sesja: `RegisterSession`
(serwer przydziela niezerowy *session handle*) → `SendRRData` niosący żądania CIP →
`UnRegisterSession` (lub rozłączenie TCP).

## 3. Zaimplementowany podzbiór protokołu

- **Enkapsulacja**: `RegisterSession` (0x0065), `UnRegisterSession` (0x0066),
  `SendRRData` (0x006F, jawna komunikacja nienawiązana, CPF).
- **CIP**: `Read Tag` (usługa 0x4C) i `Write Tag` (usługa 0x4D) na **nazwanych tagach**
  (segment symboliczny ANSI `0x91`).

## 4. Tabela tagów

| Tag | Typ CIP | Dostęp | Wielkość | Zapis → polecenie |
|---|---|:--:|---|---|
| `Setpoint` | REAL (0x00CA) | R/W | wartość zadana | `SetSetpoint` |
| `ProcessValue` | REAL | R | pomiar | — |
| `Output` | REAL | R | wyjście (%) | — |
| `ManualOutput` | REAL | R/W | wyjście ręczne (%) | `SetManualOutput` |
| `Run` | BOOL (0x00C1) | R/W | praca | `SetRun` |
| `Auto` | BOOL | R/W | tryb auto | `SetAuto` |
| `SetpointMin` | REAL | R | min. wartość zadana | — |
| `SetpointMax` | REAL | R | maks. wartość zadana | — |
| `Kp` / `Ki` / `Kd` | REAL | R | wzmocnienia PID | — |

Znany tag **tylko do odczytu**, gdy jest zapisywany, zostaje **zaakceptowany** (status
CIP sukces), lecz bez efektu; **nieznany tag** zwraca status CIP `0x05` (*path
destination unknown*). Każdy zapis sterowalny jest **ograniczany/sanityzowany** przez
symulację.

## 5. Przykład klienta

Za pomocą klienta EtherNet/IP (np. `pycomm3`, `rseip`, `rust-ethernet-ip`)
wskazującego na IP/port serwera tagi odczytuje się/zapisuje po nazwie:

```python
from pycomm3 import CIPDriver  # lub LogixDriver w zależności od narzędzia
# Odczytaj pomiar, zapisz wartość zadaną i uruchom regulację:
#   read  Tag "ProcessValue" (REAL)
#   write Tag "Setpoint" = 80.0 (REAL)
#   write Tag "Run" = True (BOOL)
```

Serwer odpowiada na ogólne usługi Read/Write Tag adresowane przez segment symboliczny
ANSI; nie udostępnia drzewa obiektów CIP poza powyższymi tagami.
