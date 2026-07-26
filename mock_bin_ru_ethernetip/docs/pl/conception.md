# Projekt — Regulator EtherNet/IP (OREE)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · **PL***

---

## 1. Przegląd

OREE wykorzystuje ponownie architekturę pozostałych instrumentów CESAM-Lab:
**synchroniczny i testowalny model biznesowy** (PID + proces), **aktorzy `ractor`**
na Tokio, **GUI `egui`** odczytujące współdzielony zrzut stanu. Zmienia się jedynie
**warstwa transportowa**: **adapter EtherNet/IP** (enkapsulacja + CIP) zamiast
Modbus/OPC UA/S7.

```
        Command (cast)                      odświeżanie przy każdym kroku
GUI   ───────────────────►  SimulationActor ───────────────────►  SharedSnapshot (GUI)
CIP Write Tag ───────────►  (Regulator)      ──────────────────►  SharedSnapshot
CIP Read Tag  ◄────────────────────────────────  SharedSnapshot
```

## 2. Aktorzy

- **`SimulationActor`** — posiada jedyny [`Regulator`]; stosuje `Command` (GUI lub
  zapisy CIP); publikuje zrzut stanu po każdej mutacji.
- **`EipServerActor`** — posiada **pętlę nasłuchu TCP**. Zadanie tokio wiąże gniazdo
  i przyjmuje klientów; każda sesja (ze swoim *session handle*) jest niesiona przez
  **wewnętrzny** `JoinSet` (zwijany razem z pętlą — żadnego odłączonego zadania).
  `Reconfigure` ponownie uruchamia nasłuch, jeśli zmieni się IP/port, i aktualizuje
  współdzieloną **białą listę**.

## 3. Warstwa protokołu

[`eip_server.rs`](../../src/eip_server.rs) jest **czysty i synchroniczny**:
enkapsulacja EtherNet/IP (`RegisterSession`, `SendRRData`/CPF) i CIP (`Read Tag`/`Write
Tag` przez segment symboliczny). Wszystko jest w **little-endian**. Parsowanie jest
**ograniczone** (weryfikowane wycinki): zniekształcony pakiet przychodzący z sieci
**nigdy** nie powoduje paniki, jedynie brak odpowiedzi. To odpowiednik
`opcua_server.rs`, wyizolowany, aby był **testowalny bez gniazda**.

### Dlaczego adapter pisany ręcznie

Nie istnieje biblioteka **serwera/adaptera** EtherNet/IP w Rust (skrzynie `rseip`,
`rust-ethernet-ip`, `cip` są zorientowane na **klienta/skaner**). Niezbędny podzbiór
(enkapsulacja + CIP Read/Write Tag na nazwanych tagach) jest zwarty: zaimplementowanie
go ręcznie daje pełną kontrolę i testowalną powierzchnię, spójną z pozostałymi
instrumentami.

## 4. Polityka sesji

Akceptowanych jest wielu **jednoczesnych** klientów (zachowanie adaptera), w
przeciwieństwie do trybu jedynego mastera w ORME. Każda sesja otrzymuje *session
handle* i odczytuje bieżący zrzut stanu; „ostatni zapisujący wygrywa".

## 5. Postawa bezpieczeństwa

- **Brak uwierzytelniania i szyfrowania** (EtherNet/IP „classic"): dostępu strzegą
  jedynie **biała lista IP** i topologia sieci. `0.0.0.0` + pusta lista = ekspozycja →
  baner ostrzegawczy ([`NetworkConfig::is_exposed`](../../src/config.rs)).
- **Sanityzacja TOML** ([`AppConfig::sanitized`](../../src/config.rs)): proces/PID/
  granice skończone i uporządkowane. Każdy zapis CIP jest **ograniczany/sanityzowany**
  przez `Regulator::apply`: powierzchnia sieciowa nie może wytworzyć ani `NaN`/`Inf`,
  ani wartości aberracyjnej.
- **Ograniczone parsowanie sieciowe**: żaden pakiet nie może wywołać paniki (zob. §3).
