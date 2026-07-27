# Projekt — Symulowany regulator PROFIBUS DP (ORPD)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · **PL***

> Crate: `mock_bin_ru_pbdp` · Plik wykonywalny: **ru_pbdp** (*Regulation Unit over PROFIBUS DP*)

Dokument architektury i modelowania. Wzorowany na regulatorze **ORME**
(`mock_bin_ru_modbus`) w zakresie modelu biznesowego i aktorów oraz na
**OSNE** (`mock_bin_su_namur`) w zakresie łącza szeregowego. Zmienia się
tylko **warstwa protokołu**: **symulator programowy ramek PROFIBUS
DP-V0**, opracowany od podstaw (do tej pory w ekosystemie Rust nie
istnieje żadna opublikowana biblioteka `profibus`/`profibus-dp`).

---

## 1. Cel

Symulacja **regulatora procesu** (pętla PID na procesie termicznym
pierwszego rzędu, model **identyczny** z ORME) i udostępnienie go poprzez
**strukturę ramek PROFIBUS DP-V0** na łączu szeregowym (RS-485/RS-232).

**Ten dokument zakłada, że czytelnik zapoznał się z ostrzeżeniem o braku
interoperacyjności** (zob. [`manuel_utilisateur.md`](manuel_utilisateur.md)
i [`reference_profibus.md`](reference_profibus.md) §6): prawdziwy
PROFIBUS DP wymaga zachowania czasowania magistrali na poziomie bitów
(*slot time*, `Tsdr` min/maks, watchdog rzędu dziesiątek milisekund),
które może zagwarantować tylko dedykowany układ ASIC (SPC3/VPC3). Ten
symulator nie rości sobie takiego prawa — jest narzędziem edukacyjnym i
do testowania oprogramowania, a nie sterownikiem magistrali.

---

## 2. Model fizyczny ([`regulator.rs`](../../src/regulator.rs))

Przejęty bez zmian z regulatora ORME:
[`mock_lib_control::FirstOrderProcess`] (transmitancja pierwszego rzędu z
czystym opóźnieniem) i [`mock_lib_control::Pid`] (PID anti-windup), z
tymi samymi trybami (Wył./PID/Dwustawny/PWM) w obu kierunkach
(grzanie/chłodzenie). Krok symulacji: **50 ms**. Wszystkie zapisy są
**sanityzowane** w `Regulator::apply` (granice porządkowane, niekończone
wartości zmiennoprzecinkowe ignorowane, wzmocnienia PID ograniczane) —
ten sam niezmiennik co wszędzie indziej w workspace: nigdy nie wywoływać
`f32::clamp` z niezwalidowanymi granicami.

---

## 3. Architektura (aktorzy)

```
GUI (egui) ──Command(cast)──►  SimulationActor  ──refresh──► SharedSnapshot ──► GUI
Symulowany master PROFIBUS ──►  (Regulator)      ──refresh──► SharedSnapshot ──► odpowiedzi Data_Exchange
```

- **`SimulationActor`** ([`actors/simulation.rs`](../../src/actors/simulation.rs)):
  identyczny w formie do tych z ORME/OSNE — jedyny właściciel
  `Regulator`, ponownie uzbrajany jednorazowy timer, publikuje
  `SharedSnapshot` przy każdym kroku.
- **`ProfibusServerActor`** ([`actors/network.rs`](../../src/actors/network.rs)):
  posiada łącze szeregowe; `Reconfigure` zamyka/otwiera ponownie
  transport, jeśli zmieni się port/prędkość transmisji/adres stacji;
  zachowuje `JoinHandle` sesji (przerywany przy zatrzymaniu); publikuje
  stan łącza (`ServerStatus`, w tym bieżący stan maszyny stanów DP-V0) dla
  GUI.
- **[`profibus.rs`](../../src/profibus.rs)** — **źródło prawdy** protokołu:
  kodek ramek (`SD1`/`SD2`/`SD3`/`SD4`/`SC`, FCS), dekodowanie usług
  (`Slave_Diag`/`Set_Prm`/`Chk_Cfg`/`Data_Exchange`) oraz maszyna stanów
  urządzenia podrzędnego `SlaveFsm`
  (`PowerOn → WaitPrm → WaitCfg → DataExchange`).
- **[`map.rs`](../../src/map.rs)** — konwersja bloków bajtów I/O
  `Data_Exchange` do/z `Command` regulatora (zob.
  [`reference_profibus.md`](reference_profibus.md) §3).
- **[`profibus_server.rs`](../../src/profibus_server.rs)** — pętla sesji
  na dowolnym strumieniu `AsyncRead + AsyncWrite` (port szeregowy w
  produkcji, `tokio::io::duplex` w testach): odczytuje ramkę, dekoduje
  ją, wywołuje `SlaveFsm::handle`, stosuje wynikowe `Command`, koduje
  odpowiedź i odsyła ją. Obsługuje również **watchdog protokołu**
  (`tokio::select!` między odczytem ramki a opóźnieniem, jak watchdog
  NAMUR w OSNE — ale tutaj jest to **prawdziwa część protokołu DP**,
  uzbrajana przez `Set_Prm`, a nie domowy dodatek).

W przeciwieństwie do Modbus (ORME, osobna tabela pamięci regenerowana
przy każdym takcie) i podobnie jak w OPC UA/NAMUR, **nie istnieje trwała
tabela pamięci**: blok wejściowy `Data_Exchange` jest przeliczany na
bieżąco z `SharedSnapshot` w momencie odpowiedzi.

**Brak polityki wielu masterów do zarządzania**: łącze szeregowe *jest*
jedynym masterem (jak Modbus RTU czy szeregowy port NAMUR), w
przeciwieństwie do Modbus TCP w ORME (wyparcie) czy nawet NAMUR TCP w
OSNE (punkt-punkt bez wyparcia).

---

## 4. Kodek PROFIBUS DP-V0 — decyzje i zaakceptowane ograniczenia

- **Ograniczniki ramek** (`SD1=0x10`, `SD2=0x68`, `SD3=0xA2`,
  `SD4=0xDC`, `SC=0xE5`, `ED=0x16`) i **FCS** (suma modulo 256): zgodne z
  normą, dobrze udokumentowane publicznie.
- **Numery SAP usług parametryzacji** (`Slave_Diag=61`, `Set_Prm=62`,
  `Chk_Cfg=63`): zgodne.
- **Dokładne kodowanie bitów pola FC**, **precyzyjne rozmieszczenie
  bajtów diagnostycznych** oraz **rozmieszczenie bloków wejścia/wyjścia**
  (`map.rs`): są to **konwencje właściwe temu symulatorowi**, a nie
  prawdziwy profil GSD zarejestrowany w PNO. Symulator systematycznie
  używa ramek **SD2** (zmienna długość) dla wszystkich wymian
  `Data_Exchange`, nawet gdy `SD3` (8 stałych bajtów) wystarczyłoby w
  prawdziwym profilu — wybór, który upraszcza kodek bez utraty pokrycia
  koncepcji protokołu.
- **Identyfikator PROFIBUS** (`Ident_Number = 0xEE01`): **fikcyjny**, nie
  zarejestrowany w PNO (PROFIBUS & PROFINET International) — nie
  reprezentuje żadnego rzeczywistego urządzenia katalogowego.
- **Brak jakiegokolwiek czasowania magistrali**: nie zaimplementowano ani
  okna odpowiedzi (`Tsdr`), ani żetonu, ani arbitrażu wielu masterów —
  zob. §1.

Pełny szczegół w [`reference_profibus.md`](reference_profibus.md).

---

## 5. Konfiguracja i trwałość

`AppConfig` (język / łącze szeregowe / proces / regulacja / sprawdzanie
aktualizacji) serializowany w formacie **TOML**
([`config.rs`](../../src/config.rs)), **sanityzowany przy wczytywaniu**
(`AppConfig::sanitized`: granice uporządkowane, `τ ≥ 1e-3`,
`dead_time ≥ 0`, skończone wartości zmiennoprzecinkowe, adres stacji
ograniczony do `[0, 125]`). Plik: `mock_ru_pbdp.toml` (nadpisywalny przez
`MOCK_CONFIG`). W przeciwieństwie do ORME/OSNE, **brak białej listy IP**
(łącze szeregowe jest z natury punkt-punkt, bez pojęcia adresu
sieciowego).

---

## 6. Kierunki rozwoju

- Prawdziwe narzędzie **symulowanego mastera PROFIBUS DP** (osobny plik
  wykonywalny), wykorzystujące te same funkcje kodowania/dekodowania
  udostępnione do testów w `profibus.rs`, do sterowania tym symulatorem
  lub dowolnym innym programowym urządzeniem podrzędnym bez zależności
  od doraźnego skryptu.
- Generowanie ilustracyjnego pliku **GSD** (niefunkcjonalnego po stronie
  symulatora) dokumentującego symulowany profil I/O, w celach
  edukacyjnych.
- Wsparcie dla **DP-V1** (dostęp acykliczny, alarmy), gdyby pojawiła się
  potrzeba edukacyjna — początkowo poza zakresem (tylko DP-V0).
- Przeniesienie modelu regulatora do wspólnej `mock_lib_*` (dziś
  zduplikowany między ORME a tym instrumentem, jak w przypadku ORUE).
