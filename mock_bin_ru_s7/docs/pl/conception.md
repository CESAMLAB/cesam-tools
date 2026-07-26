# Projekt — Regulator S7 (ORSS)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · **PL***

---

## 1. Przegląd

ORSS ponownie wykorzystuje architekturę pozostałych instrumentów CESAM-Lab: **synchroniczny
i testowalny model biznesowy** (PID + proces), **aktorzy `ractor`** na Tokio, **GUI
`egui`** odczytujące współdzieloną migawkę. Zmienia się jedynie **warstwa transportowa**: 
**serwer S7comm** (ISO-on-TCP / RFC1006) zamiast Modbus/OPC UA.

```
        Command (cast)                      refresh każdy krok
GUI   ───────────────────►  SimulationActor ───────────────────►  SharedSnapshot (GUI)
S7 Write Var ────────────►  (Regulator)      ──────────────────►  SharedSnapshot
S7 Read Var  ◄────────────────────────────────  SharedSnapshot (obraz DB1)
```

## 2. Aktorzy

- **`SimulationActor`** — posiada jedyny [`Regulator`]. Pętla o stałym kroku;
  stosuje `Command` (GUI lub zapisy S7); publikuje migawkę po każdej
  mutacji.
- **`S7ServerActor`** — posiada **pętlę nasłuchu TCP**. Dedykowane zadanie tokio
  wiąże gniazdo i przyjmuje klientów; każda sesja jest obsługiwana przez **wewnętrzny**
  `JoinSet` (więc kończona wraz z pętlą — żadnego odłączonego zadania). `Reconfigure`
  ponownie uruchamia nasłuch, jeśli zmieni się IP/port, i aktualizuje współdzieloną **listę dozwolonych**.

## 3. Warstwa protokołu

[`s7_server.rs`](../../src/s7_server.rs) jest **czysty i synchroniczny** (bez żadnej zależności
sieciowej): framing TPKT, COTP (CR→CC, DT) i S7comm (Setup, Read Var, Write Var) na
**obrazie bajtów DB1**. Parsowanie jest **ograniczone** (dostęp przez `get`/zweryfikowane
slice'y): zniekształcona ramka z sieci **nigdy** nie powoduje paniki,
jedynie brak odpowiedzi. To odpowiednik S7 dla `opcua_server.rs`, wyodrębniony,
aby był **testowalny bez gniazda**.

### Dlaczego serwer napisany ręcznie

W Rust nie istnieje żadna biblioteka **serwera** S7 (crate'y `s7`/`s7-comm`
są zorientowane na **klienta**). Niezbędny podzbiór (COTP klasa 0 + S7 Read/
Write Var na DB) jest zwarty i dobrze wyspecyfikowany: ręczna implementacja daje pełną
kontrolę i testowalną powierzchnię, spójną z pozostałymi instrumentami.

## 4. Polityka sesji

Akceptowanych jest wiele **jednoczesnych** klientów S7 (zachowanie sterownika), w
przeciwieństwie do trybu jednego mastera ORME (eksmisja) i punkt-punkt OSNE (squat).
Każda sesja odczytuje bieżący obraz DB1 i kieruje swoje zapisy do symulacji;
„ostatni piszący wygrywa", jak w rzeczywistym sterowniku.

## 5. Postawa bezpieczeństwa

- **Brak uwierzytelniania i szyfrowania** (S7 „classic"): dostępu chronią jedynie
  **lista dozwolonych adresów IP** i topologia sieci. `0.0.0.0` + pusta lista = wystawiony →
  baner ostrzegawczy w GUI ([`NetworkConfig::is_exposed`](../../src/config.rs)).
- **Sanityzacja TOML** ([`AppConfig::sanitized`](../../src/config.rs)): proces/
  PID/granice skończone i uporządkowane. Każdy zapis S7 jest **ograniczany/sanityzowany** przez
  `Regulator::apply`: powierzchnia sieciowa nie może wytworzyć ani `NaN`/`Inf`, ani wartości
  niedorzecznej.
- **Ograniczone parsowanie sieciowe**: żadna ramka nie może wywołać paniki (zob. §3).
