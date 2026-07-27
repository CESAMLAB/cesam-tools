# Podręcznik użytkownika — Symulowany regulator PROFIBUS DP (ORPD)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · **PL***

> Crate: `mock_bin_ru_pbdp` · Plik wykonywalny: **ru_pbdp** · Marka: **ORPD**

---

## ⚠️ Zanim zaczniesz: czym ten symulator NIE jest

`ru_pbdp` **nie jest** urządzeniem podrzędnym PROFIBUS DP zgodnym ze
sprzętem rzeczywistym. PROFIBUS DP to magistrala żetonowa, której
zachowanie okien czasowych (*slot time*, `Tsdr`, watchdog) wymaga
dedykowanego układu (ASIC SPC3/VPC3, karta mastera Hilscher/Softing/
Siemens CP). Zwykły program Tokio, nawet podłączony do prawdziwego portu
RS-485, **nie może spełnić tych ograniczeń**: prawdziwy sterownik PLC
(np. Siemens S7 z kartą mastera) **nigdy** nie rozpozna tego symulatora
jako prawidłowego urządzenia podrzędnego na rzeczywistej magistrali.

Co `ru_pbdp` naprawdę robi: implementuje **programowo i bez ograniczeń
czasu rzeczywistego** strukturę ramek i maszynę stanów urządzenia
podrzędnego DP-V0 (parametryzacja, konfiguracja, diagnostyka, wymiana
cykliczna). To narzędzie do **zrozumienia protokołu** i **testowania
rozwoju oprogramowania** (kodek, maszyna stanów, narzędzia) — nie do
sterowania urządzeniami terenowymi. Zob.
[reference_profibus.md](reference_profibus.md) §6 dla szczegółów
ograniczeń.

---

## 1. Do czego służy ten symulator

`ru_pbdp` symuluje **regulator procesu** (pętla PID na procesie
termicznym, model identyczny z ORME/Modbus) i udostępnia go poprzez
symulowany zestaw ramek PROFIBUS DP-V0, na łączu szeregowym
(RS-485/RS-232). Interfejs graficzny umożliwia **sterowanie** symulacją i
**wizualizację** jej dynamiki; dziennik ramek pokazuje wymieniany ruch w
formacie szesnastkowym.

---

## 2. Pierwsze kroki

```bash
cargo run -p mock_bin_ru_pbdp          # GUI + szeregowe łącze PROFIBUS DP
```

Przy uruchomieniu symulator próbuje otworzyć skonfigurowany port
szeregowy (domyślnie `/dev/ttyUSB0` lub `COM3`, 500 kbit/s, adres stacji
3). Jeśli port nie istnieje (częsty przypadek bez sprzętu szeregowego),
GUI wyświetla błąd otwarcia w nagłówku — symulacja regulatora nadal
działa, tylko łącze jest niedostępne. Ustaw **port szeregowy** w
*Ustawieniach*, aby wskazywał na dostępny pseudo-terminal lub adapter
USB-szeregowy.

---

## 3. Interfejs

### Nagłówek

- **Tytuł** i przyciski **⚙ Ustawienia** / **💾 Zapisz ustawienia**.
- Po prawej: **stan urządzenia** (W RUCHU / ZATRZYMANE), **stan łącza**
  (`PROFIBUS ● <port> [<stan>]` na zielono, jeśli otwarte — pokazany stan
  to stan maszyny stanów DP-V0:
  `Power_On`/`Wait_Prm`/`Wait_Cfg`/`Data_Exchange`), oraz **logo
  CESAM-Lab**.
- **Stały pomarańczowy baner** przypomina o braku interoperacyjności z
  rzeczywistym sprzętem (zob. ostrzeżenie powyżej).

### Mini-terminal (dolna część okna)

Dziennik tylko do odczytu ramek **odebranych** (← RX) i **wysłanych**
(→ TX), z znacznikiem czasu i wyświetlaniem szesnastkowym. Przycisk
**Wyczyść**, aby opróżnić dziennik.

### Panel poleceń (lewy)

Identyczny z ORME: **Start/Stop**, **Auto/Ręczny**, tryby regulacji dla
**kierunku 1 (grzanie)** / **kierunku 2 (chłodzenie)**
(Wył./PID/Dwustawny/PWM), **wartości zadane** (automatyczna i ręczna),
**ustawienia PID** obu kierunków, **histereza**, **minimalny cykl
dwustawny**, **okres PWM**.

### Prawy panel: bloki I/O PROFIBUS

Tabela na żywo bloków *Output* (master→urządzenie podrzędne) i *Input*
(urządzenie podrzędne→master), z rozmieszczeniem bajtów używanym przez
ten symulator — zob. [reference_profibus.md](reference_profibus.md) §3.

### Obszar centralny

Karty **Pomiar**, **Aktywna wartość zadana**, **Wyjście**, oraz krzywa
trendu.

---

## 4. Ustawienia (okno modalne ⚙)

- **Język** interfejsu (8 języków), zapisywany.
- **Sprawdzać aktualizacje przy uruchomieniu** + przycisk **Sprawdź
  teraz**.
- **Port szeregowy**, **prędkość transmisji** (baud — użyj
  znormalizowanej wartości PROFIBUS DP: 9600, 19200, 45450, 93750,
  187500, 500000, 1500000, 3000000, 6000000 lub 12000000), **adres
  stacji** (0-125).
- **Watchdog protokołu (dozwolony)**: pole wyboru — jeśli odznaczone,
  watchdog żądany przez mastera za pomocą `Set_Prm` jest **ignorowany**
  (nigdy nie uzbrajany).
- **Transmitancja procesu**: wzmocnienie `K`, stała czasowa `τ`, czyste
  opóźnienie, wartość otoczenia.
- **Granice wartości zadanej**: min / maks (automatycznie porządkowane w
  przypadku odwrócenia).
- **Zastosuj** / **Przywróć domyślne** / **Zamknij**.

Zmiana portu/prędkości/adresu **zamyka i otwiera ponownie** łącze
szeregowe. Ustawienia są zapisywane w `mock_ru_pbdp.toml` (bieżący
katalog; nadpisywalny za pomocą zmiennej środowiskowej `MOCK_CONFIG`).

**Format ramki (8E1) jest ustalony przez normę PROFIBUS DP** i nie
podlega tu regulacji, w przeciwieństwie do Modbus RTU lub szeregowego
NAMUR.

---

## 5. Mini-terminal jako narzędzie edukacyjne

Bez rzeczywistego sprzętu PROFIBUS najlepszym sposobem obserwacji
protokołu jest doprowadzenie do dialogu między **dwiema instancjami**
tego narzędzia — lub napisanie małego skryptu odtwarzającego sekwencję
`Slave_Diag` → `Set_Prm` → `Chk_Cfg` → `Data_Exchange` na pseudo-
terminalu (`socat -d -d pty,raw,echo=0 pty,raw,echo=0`) — i odczytanie
mini-terminala, aby zobaczyć wymieniane ramki w formacie szesnastkowym,
wraz z ich dekodowaniem w
[reference_profibus.md](reference_profibus.md).

---

## 6. Najczęściej zadawane pytania

**Czy mogę podłączyć ten symulator do prawdziwego sterownika PLC
PROFIBUS DP?** Nie — zob. ostrzeżenie na początku tego dokumentu oraz §6
[reference_profibus.md](reference_profibus.md).

**Port szeregowy się nie otwiera.** Wskazany plik/urządzenie nie istnieje
lub uprawnienia są niewystarczające (grupa `dialout` w Linuksie).
Dokładny błąd jest wyświetlany w nagłówku GUI.

**Łącze pozostaje w stanie `Wait_Prm`.** Master nie wysłał jeszcze
`Set_Prm` z oczekiwanym identyfikatorem (`0xEE01`, identyfikator
**fikcyjny**, niezarejestrowany w PNO). Zob.
[reference_profibus.md](reference_profibus.md) §2.

**Łącze pozostaje w stanie `Wait_Cfg`.** Odebrane `Chk_Cfg` nie zgłasza
oczekiwanych długości I/O (45 bajtów wyjścia, 17 bajtów wejścia dla tego
symulatora).

**Urządzenie zatrzymuje się samo.** Watchdog protokołu (uzbrojony przez
mastera za pomocą `Set_Prm`) wygasł z powodu braku cyklicznej wymiany
odebranej na czas — to oczekiwany stan bezpieczny, a nie błąd.

**Uruchomić bez interfejsu graficznego?** Skompiluj w trybie *headless*:
`cargo run -p mock_bin_ru_pbdp --no-default-features` — łącze szeregowe
i symulacja działają bez GUI.
