# Podręcznik użytkownika — ORME (symulowany regulator Modbus)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · **PL***

> **ORME** — *Open Regulator Modbus Emulator* · plik binarny `mock_bin_ru_modbustcp` ·
> Licencja MIT · Wydawca: **CESAM-Lab** · Identyfikator urządzenia Modbus: **CESAM-Lab**
>
> *„Otwórz magistralę.”* Regulator obiektowy, który istnieje tylko na Twojej
> magistrali Modbus (TCP/RTU) — do testowania SCADA, sterowników i HMI bez
> rzeczywistego sprzętu.

Ten podręcznik jest przeznaczony dla **użytkownika** symulowanego regulatora: jak
go uruchomić, sterować nim z interfejsu, skonfigurować i podłączyć przez Modbus TCP.
Nie jest wymagana żadna wiedza programistyczna.

---

## 1. Do czego służy to oprogramowanie?

Symuluje **regulator przemysłowy** (typu piec lub łaźnia termostatyczna):

- realistyczny **proces fizyczny** („pomiar” rośnie/maleje zależnie od sterowania);
- **regulację** automatyczną lub ręczną, w trybie **grzania** i/lub **chłodzenia**;
- **serwer Modbus TCP** do sterowania/nadzoru z innego oprogramowania
  (sterownik, SCADA, bramka…);
- **interfejs graficzny** do prowadzenia i wizualizacji.

To narzędzie **testowe**: pozwala dopracować i zademonstrować system nadzoru lub
sterownik **bez rzeczywistego sprzętu**.

---

## 2. Uruchamianie oprogramowania

Uruchom plik wykonywalny odpowiedni dla Twojego systemu:

| System | Plik |
|---------|---------|
| Windows | `orme-windows-x86_64.exe` (dwuklik) |
| Linux PC | `./orme-linux-x86_64` |
| Raspberry Pi (z ekranem) | `./orme-rpi-arm64` |

Okno otwiera się, a **serwer Modbus startuje automatycznie** (port `5502`
domyślnie). Nagłówek pokazuje stan:

- **● W RUCHU / ● ZATRZYMANY**: stan urządzenia;
- **Modbus ● 0.0.0.0:5502** (zielony): serwer nasłuchuje; **✖ …** (czerwony) w
  razie problemu sieciowego.

> Bez ekranu (sam serwer), zob. **§ 9 (Użytkowanie bez ekranu)**.

---

## 3. Interfejs na pierwszy rzut oka

Okno zawiera cztery strefy:

```
┌───────────────────────────── Nagłówek: tytuł, ⚙ Parametry, 💾 Zapisz, stany ─────────────────────────────┐
├──────────────────┬─────────────────────────────────────────────────┬───────────────────────────────────────────┤
│  KOMENDY          │   NADZÓR                                        │   TABLICA ADRESÓW MODBUS                  │
│  (lewa)           │   - wartości chwilowe (Pomiar / Nastawa /       │   (prawa)                                 │
│  Start/Stop       │     Wyjście)                                    │   lista live: oznaczenie, tablica,        │
│  Auto/Ręczny      │   - WYKRES trendu w czasie rzeczywistym         │   adres, wartość, dostęp                  │
│  Tryby, nastawy   │                                                 │                                           │
│  ustawienia PID…  │                                                 │                                           │
└──────────────────┴─────────────────────────────────────────────────┴───────────────────────────────────────────┘
```

---

## 4. Sterowanie regulatorem (lewy panel)

### 4.1 Start / Stop
Przycisk **Start / Stop**. Po zatrzymaniu wyjście jest zerowe, a pomiar powoli
wraca do wartości otoczenia.

### 4.2 Auto / Ręczny
- **Ręczny**: *Ty* narzucasz wyjście przez **nastawę ręczną** (w %).
- **Auto**: regulator oblicza wyjście, aby osiągnąć **nastawę auto**.

### 4.3 Nastawy
Każda nastawa posiada **pole liczbowe** (precyzyjne wprowadzanie z klawiatury)
oraz **suwak**. Oba są zawsze edytowalne; nastawa **aktywna** (zależnie od trybu)
jest wyświetlana pogrubieniem.

| Nastawa | Jednostka | Rola |
|----------|-------|------|
| **SP auto** | jednostka pomiaru (np. °C) | cel do osiągnięcia w trybie Auto |
| **SP ręczna** | % wyjścia, od −100 do +100 | wyjście narzucone w trybie Ręcznym (**+** grzanie / **−** chłodzenie) |

### 4.4 Tryby regulacji — kierunek 1 (grzanie) i kierunek 2 (chłodzenie)
Każdy kierunek ustawia się niezależnie:

- **Wyłączony** — kierunek nie działa;
- **PID** — regulacja ciągła (wyjście 0…100 %), precyzyjna i łagodna;
- **Dwustawny (TOR)** — przekaźnik z histerezą: wyjście 0 % lub 100 %, proste, ale
  oscylujące wokół nastawy;
- **Przekaźnik cyklowy (PWM)** — PID oblicza współczynnik wypełnienia, *siekany*
  na stałym okresie: wyjście fizyczne pozostaje dwustawne (0/100 %), ale jego
  **średnia** podąża za PID. To najlepszy kompromis do precyzyjnego sterowania
  elementem, który potrafi się tylko otwierać lub zamykać (przekaźnik, zawór dwustawny).

> 👉 **Ważne — zob. **§ 6 (Zrozumieć regulację)****: wybór
> PID/TOR/PWM dla chłodzenia *uzbraja* chłodzenie, ale ono **wytwarza tylko wtedy,
> gdy pomiar przekroczy nastawę**.

### 4.5 Ustawienia PID (Kp, Ki, Kd)
Dla każdego kierunku trzy wzmocnienia regulowane na żywo:

- **Kp** (proporcjonalne): im większe, tym żywsza reakcja (ryzyko oscylacji);
- **Ki** (całkujące): likwiduje resztkowy uchyb w czasie (zbyt duże → przeregulowanie);
- **Kd** (różniczkujące): tłumi/wyprzedza (zbyt duże → wrażliwość na szum).

### 4.6 Ustawienia TOR / PWM
- **Histereza TOR** — szerokość **strefy martwej** trybu dwustawnego, wyśrodkowana
  na nastawie (`[SP − h/2, SP + h/2]`): zapobiega ciągłemu przełączaniu wyjścia.
  Im szersza, tym większe tętnienie, ale rzadsze przełączenia.
- **Min. cykl TOR (s)** — minimalny czas, przez który przekaźnik pozostaje w
  jednym stanie, zanim będzie mógł przełączyć ponownie (**zabezpieczenie przed
  krótkim cyklem**). Chroni rzeczywisty element wykonawczy (przekaźnik, sprężarkę)
  i wygładza zachowanie. `0` = wyłączone.
- **Okres PWM (s)** — czas trwania jednego cyklu **przekaźnika cyklowego**. Krótki
  → wierniejsza średnia, ale częste przełączenia; długi → mniejsze zużycie, ale
  wyraźniejsze tętnienie. Dobierać znacznie mniejszy niż stała czasowa procesu.

---

## 5. Odczyt wykresu trendu

Wykres (na środku) kreśli w czasie rzeczywistym trzy wielkości. **Legenda, w
lewym górnym rogu**, przypomina kolor **i ostatnią wartość** każdej serii:

| Kolor | Seria | Znaczenie |
|---------|-------|---------------|
| 🔵 niebieski | **Nastawa (SP)** | cel (w trybie Auto) |
| 🔴 czerwony | **Pomiar (PV)** | wartość procesu |
| 🟢 zielony | **Wyjście (%)** | zastosowane sterowanie (**+** grzanie / **−** chłodzenie) |

Nad wykresem trzy karty pokazują wartości chwilowe (Pomiar, Nastawa aktywna,
Wyjście). Wykres można powiększać/przesuwać myszą.

---

## 6. Zrozumieć regulację (grzanie / chłodzenie)

Regulator działa w **jednym kierunku naraz**, wybranym według uchybu
`Nastawa − Pomiar`:

| Sytuacja | Działający kierunek | Wyjście | Sygnalizacja |
|-----------|---------------|--------|--------|
| Pomiar **<** Nastawa (trzeba grzać) | **Kierunek 1 (grzanie)** | **dodatnie** (0…+100 %) | **Grzanie aktywne = 1** |
| Pomiar **>** Nastawa (trzeba chłodzić) | **Kierunek 2 (chłodzenie)** | **ujemne** (−100…0 %) | **Chłodzenie aktywne = 1** |

Praktyczne konsekwencje:

- Wybranie **PID/TOR dla chłodzenia** nie wystarczy, aby zapaliło się „Chłodzenie
  aktywne”: trzeba, aby **pomiar był powyżej nastawy**. Dopóki pomiar jest poniżej,
  pracuje **grzanie**.
- Aby zobaczyć wytwarzanie chłodzenia: w trybie **Auto**, kierunek 2 w PID/TOR,
  **obniż nastawę poniżej bieżącego pomiaru** (lub poczekaj na przeregulowanie).
  Wyjście stanie się ujemne, a **Chłodzenie aktywne** przejdzie na 1.
- W **TOR** przekaźnik przełącza się na **półhisterezie** po obu stronach nastawy
  (symetryczna strefa martwa) i respektuje **minimalny cykl** między dwoma
  przełączeniami. W **PWM** wyjście siecze na 0/100 %, ale jego średnia podąża za PID.

---

## 7. Parametry (przycisk ⚙)

Przycisk **⚙ Parametry** otwiera okno do konfiguracji:

### Transport Modbus
Wybór magistrali komunikacyjnej — **tylko jedna aktywna naraz**:

**TCP (Ethernet)**
- **IP nasłuchu** (`0.0.0.0` = wszystkie interfejsy) oraz **Port** (domyślnie 5502);
- **Dozwolone IP**: jedno na linię, znaki wieloznaczne `*` akceptowane (np. `192.168.1.*`).
  **Pusta lista = wszystkie IP dozwolone.** Pozostałe są odrzucane.

**RTU (RS485)** — wymaga pliku binarnego skompilowanego z feature `rtu`
- **Port szeregowy**: `/dev/ttyUSB0`, `/dev/ttyAMA0` (Raspberry Pi), `COM3` (Windows)…;
- **Baud** (domyślnie 19200), **Parzystość** (domyślnie Parzysta), **Bity danych** (8),
  **Bity stopu** (1) — uzgodnić z masterem;
- **Adres slave** (1–247).

> ⚠️ **Tylko jeden zdalny master naraz.** W TCP połączenie nowego mastera
> **automatycznie odłącza** poprzedniego. Lokalne GUI **nie** jest masterem:
> pozostaje zawsze aktywne. W RTU preferować **połączenie punkt-punkt**
> (urządzenie odpowiada niezależnie od żądanego adresu).

### Funkcja przejścia (proces)
Symulowane zachowanie fizyczne `G(s) = K·e^(−L·s) / (1 + T·s)`:
- **Wzmocnienie K**: zmiana pomiaru na % wyjścia;
- **Stała T** (s): bezwładność/szybkość;
- **Opóźnienie L** (s): czas martwy przed reakcją;
- **Otoczenie**: wartość spoczynkowa.

### Granice nastawy
Limity min/maks nastawy auto.

Przyciski: **Zastosuj** (działa natychmiast **i** zapisuje),
**Przywróć domyślne**, **Zamknij**.

### Zapis ustawień
Ustawienia są **zapisywane** w pliku `mock_ru_modbustcp.toml` (obok
oprogramowania) i **przeładowywane przy następnym uruchomieniu**. Przycisk
**💾 Zapisz ustawienia** w nagłówku zapisuje także wzmocnienia PID, histerezę,
minimalny cykl TOR i okres PWM zmienione w lewym panelu.

---

## 8. Podłączanie klienta Modbus

Oprogramowanie jest **slave'em Modbus** (TCP port 5502 domyślnie lub RTU szeregowy
zależnie od transportu wybranego w § 7). Klient (sterownik, SCADA, `mbpoll`…) może
**odczytywać** stan i **zapisywać** nastawy/tryby. Przypomnienie: **tylko jeden
zdalny master naraz** (w TCP nowo przybyły odłącza poprzedniego).

Główne punkty (adresy **baza 0**):

| Dane | Tablica | Adres | Typ | Dostęp |
|--------|-------|---------|------|-------|
| Start/Stop | Coil | 0 | bit | O/Z |
| Auto/Ręczny | Coil | 1 | bit | O/Z |
| Tryb kierunek 1 / kierunek 2 | Holding | 0 / 1 | 0=Off,1=PID,2=TOR,3=PWM | O/Z |
| Nastawa auto | Holding | 2–3 | zmiennoprzecinkowy | O/Z |
| Nastawa ręczna | Holding | 4–5 | zmiennoprzecinkowy | O/Z |
| Min. cykl TOR (s) | Holding | 20–21 | zmiennoprzecinkowy | O/Z |
| Okres PWM (s) | Holding | 22–23 | zmiennoprzecinkowy | O/Z |
| Pomiar (PV) | Input | 0–1 | zmiennoprzecinkowy | O |
| Sterowanie (%) | Input | 2–3 | zmiennoprzecinkowy | O |
| Identyfikator „CESAM-Lab” | Holding | 42–46 | tekst ASCII | O |

> **Pełna tablica** (wzmocnienia PID, histereza, kodowanie zmiennoprzecinkowych,
> kody funkcji, przykłady `mbpoll`) znajduje się w **[table_modbus.md](table_modbus.md)**.
> Ta sama tablica jest też widoczna **na żywo** w prawym panelu GUI.

---

## 9. Użytkowanie bez ekranu („headless” / Docker)

Do wdrożenia w tle (Raspberry Pi bez ekranu, serwer) istnieje wersja **bez
interfejsu**: uruchamia symulację i serwer Modbus, sterowalne **wyłącznie przez
Modbus**.

```bash
# Obraz Docker (wdrażalny gdziekolwiek):
docker run --rm -p 5502:5502 -v "$PWD/conf:/data" orme:headless
```

Katalog zamontowany na `/data` pozwala dostarczyć/zachować `mock_ru_modbustcp.toml`.

---

## 10. Najczęstsze pytania

| Pytanie / objaw | Odpowiedź |
|---------------------|---------|
| **„Chłodzenie aktywne” nie przechodzi na 1, choć ustawiłem PID/TOR.** | Normalne: chłodzenie wytwarza tylko, gdy **pomiar przekracza nastawę**. Obniż nastawę poniżej pomiaru (tryb Auto). Zob. **§ 6 (Zrozumieć regulację)**. |
| Pomiar się nie zmienia. | Sprawdź, czy urządzenie jest **W ruchu**, a nastawa/wyjście niezerowe. |
| W trybie ręcznym zmiana trybów kierunku 1/2 nic nie daje. | Normalne: tryby stosują się tylko w **Auto**. |
| Nagłówek pokazuje **Modbus ✖**. | Port już zajęty lub < 1024 bez uprawnień: zmień **port** w ⚙ Parametry. |
| Mój klient Modbus jest odrzucany. | Jego IP nie jest na **białej liście**: opróżnij listę lub dodaj wzorzec (`192.168.1.*`). |
| Odczytywane wartości zmiennoprzecinkowe są niespójne. | Problem **kolejności słów** po stronie klienta (słowo bardziej znaczące najpierw). Zob. table_modbus.md. |
| Nastawa zapisana przez Modbus jest ignorowana. | Wartość zmiennoprzecinkowa zajmuje **2 rejestry**: zapisz je **razem**. |
| Moje ustawienia nie są zachowywane. | Kliknij **Zastosuj** / **💾 Zapisz**. Plik `mock_ru_modbustcp.toml` musi być dostępny do zapisu. |

---

*Powiązana dokumentacja techniczna: [conception.md](conception.md) ·
[table_modbus.md](table_modbus.md) · [maintenance.md](maintenance.md).*
