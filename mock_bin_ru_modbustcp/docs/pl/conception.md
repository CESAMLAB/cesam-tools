# Dokument projektowy — Symulowany regulator Modbus TCP

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · **PL***

> Produkt: **ORME** · Crate: `mock_bin_ru_modbustcp` · Workspace: `cesam-tools` · Licencja: MIT

Niniejszy dokument opisuje architekturę, decyzje techniczne oraz zasady działania
symulowanego regulatora przemysłowego. Jest przeznaczony dla programistów, którzy
utrzymują lub rozszerzają projekt.

---

## 1. Cel i zakres

Dostarczenie **wirtualnego przyrządu przemysłowego**: regulatora procesu, który
zachowuje się realistycznie i komunikuje się przez **Modbus TCP** (slave), aby
umożliwić tworzenie i testowanie systemów nadzoru / sterowników / bramek **bez
sprzętu**.

Symulator obejmuje:

- **proces fizyczny** modelowany funkcją przejścia;
- **regulację** dwukierunkową (grzanie / chłodzenie): PID, dwustawną (TOR) lub
  przekaźnik cyklowy (PWM);
- **interfejs Modbus TCP** udostępniający pełny stan;
- **interfejs GUI** do sterowania, wizualizacji i konfiguracji;
- **trwałość** parametrów.

Poza obecnym zakresem: Modbus RTU, redundancja, długoterminowe archiwizowanie,
silne uwierzytelnianie (dostępna jest jedynie biała lista adresów IP).

---

## 2. Widok ogólny

```
┌──────────────────────────────────────────────────────────────────────┐
│                       Proces (wątek główny)                            │
│                                                                        │
│   ┌─────────────────────────┐         czyta (Mutex)                    │
│   │   GUI  egui / eframe      │◄──────────────── SharedSnapshot         │
│   │   (gui.rs)               │◄──────────────── SharedStatus           │
│   └───────────┬─────────────┘                                          │
│               │ cast (nieblokujący)                                    │
└───────────────┼────────────────────────────────────────────────────────┘
                │
   ┌────────────┼──────────── Runtime Tokio (wątki w tle) ───────────────┐
   │            ▼                                                         │
   │   ┌──────────────────┐  refresh  ┌──────────────┐                   │
   │   │ SimulationActor   ├──────────►│ SharedSnapshot│ (GUI)            │
   │   │  (ractor)         ├──────────►│ SharedMap     │ (Modbus)         │
   │   │  posiada           │           └──────┬───────┘                  │
   │   │  Regulator         │◄── Command ──┐    │ czyta                   │
   │   └──────────────────┘              │    ▼                          │
   │          ▲ Command (cast)            │  ┌──────────────────────┐     │
   │          │                           └──┤ RegulatorService      │     │
   │   ┌──────┴───────────┐  zarządza/rebind │ (trait Service)       │     │
   │   │ ModbusServerActor ├─────────────────►  serwer Modbus TCP    │◄──── klienci
   │   │  (ractor)         │  filtr IP ───────► (tokio-modbus)        │     │
   │   └──────────────────┘   (SharedAllowlist)└──────────────────────┘     │
   └────────────────────────────────────────────────────────────────────┘
```

Zasada przewodnia: **jeden jedyny właściciel stanu biznesowego**. `Regulator`
nigdy nie jest współdzielony; żyje w `SimulationActor`. Wszystkie zapisy
(GUI lub Modbus) to **komunikaty** `Command`. Odczyty odbywają się na
**kopiach** odświeżanych w każdym kroku (`SharedSnapshot`, `SharedMap`), co
eliminuje blokady na logice oraz wyścigi.

---

## 3. Decyzje techniczne

| Potrzeba | Wybór | Uzasadnienie |
|--------|-------|---------------|
| Współbieżność | **`ractor`** (aktory) na **Tokio** | Izoluje stan mutowalny w aktorze; mutacje serializowane przez komunikaty, bez blokad aplikacyjnych. Preferencja projektu. |
| Modbus TCP slave | **`tokio-modbus`** (`tcp-server`) | Dojrzała implementacja async; trait `Service` czysto mapuje żądanie→odpowiedź. |
| GUI | **`egui` / `eframe`** + `egui_plot` | Tryb natychmiastowy, wieloplatformowy, bez złożonego stanu UI do synchronizacji. |
| Proces | **FOPDT** (1. rzędu + opóźnienie) | Standardowy i wystarczający model procesu termicznego; mało parametrów, intuicyjny. |
| Trwałość | **`serde` + `toml`** | Format czytelny/edytowalny ręcznie, idealny dla parametrów urządzenia. |

### Dlaczego oddzielenie logiki synchronicznej i asynchronicznej

`mock_lib_control` i `regulator.rs` są **czysto synchroniczne** (żadnego IO,
żadnego async). Zalety: deterministycznie testowalne jednostkowo,
wielokrotnego użytku przez inne przyrządy oraz łatwe do przeglądu. Asynchroniczność
ograniczona jest do **aktorów** i do **warstwy sieciowej**.

---

## 4. Model danych

### Stan biznesowy (`regulator.rs`)

- `Regulator` — agregat-właściciel: tryby, nastawy, regulatory (`Pid`,
  `OnOff`) i proces (`FirstOrderProcess`). Nie `Clone`, nie współdzielony.
- `RegulatorConfig` — konfiguracja statyczna (proces, wzmocnienia, granice, `dt`).
  **Jedyne źródło** wartości domyślnych (konfiguracja TOML z niego wywodzi).
- `RegulatorSnapshot` — **niemutowalna kopia** (`Copy`) stanu obserwowalnego,
  publikowana w każdym kroku. To kontrakt odczytu dla GUI i tablicy Modbus.
- `Command` — wyliczenie możliwych mutacji (uruchomienie, tryb, nastawy,
  ustawienia, proces, granice).

### Struktury współdzielone (`actors/mod.rs`, `config.rs`)

| Typ | Zawartość | Zapisywane przez | Odczytywane przez |
|------|---------|-----------|--------|
| `SharedSnapshot` | typowany `RegulatorSnapshot` | SimulationActor | GUI |
| `SharedMap` | `MemoryMap` (obrazy 4 tablic Modbus) | SimulationActor | RegulatorService |
| `SharedAllowlist` | `IpFilter` | ModbusServerActor | akceptacja połączeń |
| `SharedStatus` | `ServerStatus` (nasłuch / błąd) | ModbusServerActor | GUI |

Wszystkie są typu `Arc<Mutex<…>>`: sekcje krytyczne **krótkie** (kopia / refresh),
nigdy nie trzymane podczas obliczeń ani IO.

---

## 5. Komponenty

### 5.1 `mock_lib_control` (biblioteka)

- `Pid` — PID czasu dyskretnego, pochodna po uchybie, **antynasycenie** przez
  ograniczanie członu całkującego. API: `step(sp, pv, dt)` lub `step_with_error(err, dt)`
  (używane ponownie dla kierunku chłodzenia).
- `OnOff` — dwustawny z **symetryczną histerezą** (strefa martwa) **oraz
  zabezpieczeniem przed krótkim cyklem**: minimalny czas cyklu (`min_cycle`, s)
  zabrania jakiegokolwiek przełączenia, dopóki przekaźnik nie pozostanie dostatecznie
  długo w swoim stanie, modelując ochronę rzeczywistego elementu wykonawczego.
  Przekaźnik **zatrzaskuje** swój stan: to wołający musi przekazać mu uchyb ze
  znakiem, nie resetując go przy zmianie znaku (zob. § 5.2).
- `Pwm` — modulator szerokości impulsu (**przekaźnik cyklowy** /
  *time-proportioning*): w stałym okresie `T_c` wyjście dwustawne jest aktywne
  przez ułamek `duty` cyklu (`duty` **próbkowany raz na cykl**, aby uniknąć
  obciążenia w stanie ustalonym). Pozwala precyzyjnie sterować elementem dwustawnym.
- `FirstOrderProcess` — funkcja przejścia `K·e^(-L·s)/(1+T·s)`, całkowanie metodą
  Eulera + linia opóźniająca. `reconfigure(...)` zmienia parametry bez skoku.
- `ControllerKind` — `Off` / `Pid` / `OnOff` / `Pwm`, z kodowaniem Modbus
  (`to_code`/`from_code`).

### 5.2 `regulator.rs`

Sterowanie regulacją w każdym kroku (`step`):

1. jeśli **zatrzymany** → wyjście 0, regulatory zresetowane;
2. jeśli **ręczny** → wyjście = nastawa ręczna (% ze znakiem);
3. jeśli **auto** → obliczamy **osobno** udział kierunku grzania (kierunek 1,
   uchyb `SP − PV`) i kierunku chłodzenia (kierunek 2, uchyb `PV − SP`), każdy ≥ 0,
   następnie `wyjście = grzanie − chłodzenie`:
   - **PID**: wyjście ograniczone do `[0, 100]` (`out_min = 0`) — nieaktywny kierunek
     (uchyb ujemny) daje wyjście 0, a jego całka **opróżnia się naturalnie** przez
     ograniczanie. Nie zerujemy jej **na siłę**: przy silnym tętnieniu PWM kasowanie
     jej przy każdym przekroczeniu nastawy wprowadzałoby uchyb statyczny;
   - **TOR**: przekaźnik jest oceniany na uchybie ze znakiem i zachowuje swój stan
     przy przekroczeniu nastawy, co przywraca **symetryczne** pasmo histerezy
     `[SP − h/2, SP + h/2]` (pasma grzanie/chłodzenie pozostają rozłączne, więc oba
     przekaźniki są wzajemnie wykluczające się);
   - **PWM**: PID oblicza współczynnik wypełnienia, modulowany przez przekaźnik cyklowy;
     wyjście fizyczne wynosi ściśle 0 % lub 100 %, ale jego średnia podąża za PID.
4. wyjście steruje procesem, który wytwarza nowy pomiar (PV).

> **Historia**: przed tą rewizją przełączanie grzanie/chłodzenie odbywało się
> według znaku uchybu i **resetowało** przekaźnik TOR przy przekroczeniu nastawy
> — co obcinało histerezę do `[SP − h/2, SP]` (połowa pasma, asymetryczna) i czyniło
> regulację TOR przeciętną. Obliczanie z podziałem na kierunki koryguje tę wadę.

### 5.3 `actors/simulation.rs`

`SimulationActor` (ractor). `pre_start` uzbraja `send_interval(dt)`, który emituje
`Tick`. `handle` przetwarza `Tick` (przesuwa symulację) i `Command` (stosuje
mutację), następnie **publikuje** stan w `SharedSnapshot` i `SharedMap`.

### 5.4 `actors/network.rs`

`ModbusServerActor` posiada serwer Modbus. `Reconfigure(NetworkConfig)`:
- aktualizuje współdzieloną **białą listę** (efekt natychmiastowy, bez restartu);
- jeśli **transport** (TCP/RTU), **port / IP** lub **parametry szeregowe**
  ulegają zmianie, **zatrzymuje** zadanie serwera i **restartuje** je (`start_tcp`
  lub `start_rtu`); publikuje stan w `SharedStatus` (sukces lub błąd).

**Jeden** transport jest aktywny naraz (`Transport::Tcp` lub `Rtu`). RTU jest
za **feature `rtu`**; bez niej wybranie RTU publikuje jawny błąd statusu.

### 5.5 `modbus_server.rs`

`RegulatorService` implementuje `tokio_modbus::server::Service` w sposób
**synchroniczny** (`future::Ready`): odczyty = wycinek `SharedMap`; zapisy =
dekodowanie do `Command` (przez `map.rs`), następnie `cast` do `SimulationActor`.

**Polityka jednego mastera.** `serve` (TCP) zezwala **tylko na jednego zdalnego
mastera naraz**: przy każdym nowym połączeniu (IP dopuszczone przez białą listę)
poprzednie jest zamykane. Mechanizm: `TcpStream` jest opakowany w
`CancellableStream`, który po odebraniu sygnału `oneshot` zwraca **EOF przy
odczycie** — pętla obsługi `tokio-modbus` kończy się wtedy i zamyka gniazdo.
`serve_rtu` (feature `rtu`) obsługuje magistralę szeregową przez
`rtu::Server::serve_forever`: magistrala RS485 *jest* jedynym masterem (nikogo nie
trzeba odłączać).

> ⚠️ GUI nie korzysta z tej ścieżki: wysyła swoje `Command` bezpośrednio do
> aktora, więc nigdy nie jest liczone jako master.
>
> ⚠️ Serwer RTU `tokio-modbus` 0.17 nie przekazuje adresu slave do usługi:
> urządzenie odpowiada więc niezależnie od żądanego adresu. Zalecane jest
> połączenie **punkt-punkt**. `slave_id` jest utrwalany i wyświetlany, ale nie
> używany do filtrowania (ograniczenie pochodzące z biblioteki).

### 5.6 `map.rs`

**Źródło prawdy** planu adresowania Modbus. Stałe adresów,
`MemoryMap` (obrazy tablic), `refresh_from(snapshot)` (stan→rejestry) oraz
`*_to_command(s)` (zapisy→komendy). Kodowanie `f32` na 2 rejestrach,
big-endian, słowo bardziej znaczące na początku.

### 5.7 `config.rs`

`AppConfig` (sieć / proces / regulacja) ⇄ TOML. `IpFilter` (znaki wieloznaczne `*`
na oktet IPv4). `ServerStatus`. `to_regulator_config()` tworzy pomost do domeny.

### 5.8 `gui.rs`

GUI **jednostronicowe**: nagłówek (stany + przyciski), panel komend (po lewej),
nadzór + wykres (środek), tablica Modbus na żywo (prawa), modal Parametry.
Czyta `Shared*`, wysyła `Command` przez nieblokujący `cast`.

---

## 6. Scenariusze (sekwencje)

**Odczyt Modbus (PV)**: klient → `RegulatorService::call(ReadInputRegisters)` →
odczyt `SharedMap` → `Response`. Brak interakcji z aktorem (minimalne opóźnienie).

**Zapis Modbus (nastawa)**: klient → `call(WriteMultipleRegisters)` →
`map::holdings_to_commands` → `cast(Command::SetSpAuto)` → aktor stosuje w
kolejnym kroku → ponownie publikuje `SharedMap`/`SharedSnapshot`.

**Komenda GUI**: interakcja → `cast(Command)` → analogicznie.

**Rekonfiguracja sieci**: modal *Zastosuj* → `cast(Reconfigure)` →
ModbusServerActor wykonuje rebind w razie potrzeby → `SharedStatus` zaktualizowany
→ nagłówek GUI odzwierciedla stan.

**Tick**: timer → `Tick` → `Regulator::step` → publikacja.

---

## 7. Teoria regulacji

**Proces (FOPDT)**: `v[k+1] = v[k] + (dt/T)·(cel − v[k])`, gdzie
`cel = ambient + K·u`, a `u` opóźnione o `L` sekund (linia opóźniająca).

**PID**: `u = Kp·e + Ki·∫e + Kd·de/dt`, całka ograniczona do `[out_min, out_max]`
(anti-windup). Pochodna po uchybie (kompromis prostota/symetria grzanie-chłodzenie).

**TOR**: aktywny gdy `e > +H/2`, nieaktywny gdy `e < −H/2`, w przeciwnym razie
stan zachowany.

**Dwukierunkowy**: tylko jeden kierunek działa naraz, wybierany według znaku
uchybu; wyjście globalne jest ze znakiem (+ grzanie / − chłodzenie).

---

## 8. Decyzje i kompromisy

- **Podwójna publikacja (`Snapshot` + `Map`)** zamiast jednej struktury:
  GUI operuje typami biznesowymi, Modbus surowymi rejestrami; oba pozostają proste
  i rozdzielone, kosztem niewielkiego, pomijalnego narzutu kopiowania.
- **Odczyty Modbus bez przechodzenia przez aktora**: czytamy `SharedMap`
  bezpośrednio, aby zminimalizować opóźnienie; aktor pozostaje jedynym **piszącym**,
  więc brak wyścigu.
- **Synchroniczna usługa Modbus** (`future::Ready`): cała praca jest nieblokująca
  (krótka blokada + cast), nie ma potrzeby pakowania future.
- **Rebind przy zmianie portu**: gniazdo nie zmienia portu; akceptujemy krótką
  przerwę w działaniu usługi przy rekonfiguracji.
- **Pochodna po uchybie** (a nie po pomiarze): lekki „bicz” przy zmianie nastawy,
  zaakceptowany dla zachowania symetrycznego i prostego algorytmu.

---

## 9. Możliwe rozszerzenia

- Modbus RTU / szeregowy (użycie ponowne `RegulatorService`, zmiana transportu).
- Rampa nastawy, autostrojenie PID, symulowane usterki (uszkodzony czujnik,
  nasycenie).
- Archiwizowanie / eksport CSV trendu.
- Przejście GUI na **karty**, jeśli pojedyncza strona stanie się zbyt gęsta.
- Nowe przyrządy: utworzyć `mock_bin_<nazwa>` i wynieść wspólne elementy do
  `mock_lib_*` (zob. [maintenance.md](maintenance.md)).
