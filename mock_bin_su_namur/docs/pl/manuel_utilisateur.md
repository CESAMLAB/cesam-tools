# Podręcznik użytkownika — OSNE (symulowane mieszadło laboratoryjne NAMUR)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · **PL***

> **OSNE** — *Open Stirrer NAMUR Emulator* · plik binarny `mock_bin_su_namur`
> (plik wykonywalny `osne`) · Licencja MIT · Wydawca: **CESAM-Lab** · Identyfikator
> NAMUR: nazwa `CESAM-STIRRER`, typ `OSNE`.
>
> *Mieszadło laboratoryjne (typu IKA), które istnieje tylko na Twojej magistrali
> NAMUR — do testowania systemów nadzoru, skryptów i bramek bez rzeczywistego
> sprzętu.*

Ten podręcznik jest przeznaczony dla **użytkownika** symulowanego mieszadła: jak
je uruchomić, sterować nim z interfejsu, skonfigurować i podłączyć przez **NAMUR**
(TCP lub szeregowy RS-232). Nie jest wymagana żadna wiedza programistyczna.

---

## 1. Do czego służy to oprogramowanie?

Symuluje **mieszadło laboratoryjne** (mieszadło stołowe ze śmigłem, typu IKA):

- realistyczny **silnik fizyczny**: prędkość rośnie/maleje zależnie od
  przyłożonego momentu, z **szybką regulacją prędkości**;
- regulowane **obciążenie lepkościowe**: im bardziej lepki ośrodek, tym większy
  potrzebny moment — aż do **przeciążenia** (nieosiągalna wartość zadana);
- **serwer NAMUR** (szeregowy protokół ASCII urządzeń laboratoryjnych) do
  sterowania/nadzoru z innego oprogramowania lub skryptu;
- **interfejs graficzny** do prowadzenia, wizualizacji i **testowania protokołu**
  (wbudowany mini-terminal NAMUR).

To narzędzie **testowe**: pozwala dopracować i zademonstrować system nadzoru,
skrypt akwizycji lub bramkę **bez rzeczywistego sprzętu**.

---

## 2. Uruchamianie oprogramowania

Uruchom plik wykonywalny odpowiedni dla Twojego systemu:

| System | Plik |
|---------|---------|
| Windows | `osne-windows-x86_64.exe` (dwuklik) |
| Linux PC | `./osne-linux-x86_64` |
| Raspberry Pi (z ekranem) | `./osne-rpi-arm64` |

Okno otwiera się, a **serwer NAMUR startuje automatycznie** (port `4001`
domyślnie). Nagłówek pokazuje stan:

- **● W RUCHU / ● ZATRZYMANY**: stan silnika;
- **NAMUR ● 0.0.0.0:4001** (zielony): serwer nasłuchuje; **✖ …** (czerwony) w
  razie problemu (zajęty port, niedostępny port szeregowy…);
- **wskaźnik połączenia**: w TCP pokazuje podłączonego mastera (lub „brak
  mastera”), w trybie szeregowym zwykłą kropkę. Staje się **zielony**, gdy ramka
  została niedawno odebrana (łącze aktywne), szary w przeciwnym razie.

> Bez ekranu (sam serwer), zob. **§ 9 (Użytkowanie bez ekranu)**.

---

## 3. Interfejs na pierwszy rzut oka

```
┌──────────────── Nagłówek: tytuł OSNE, ⚙ Parametry, 💾 Zapisz, stany i wskaźniki ────────────────┐
├──────────────────┬──────────────────────────────────────────────────────────────────────────────────┤
│  KOMENDY          │   NADZÓR                                                                           │
│  (lewa strona)    │   - karty wartości (Prędkość / Moment / Lepkość / Przeciążenie)                   │
│  Start/Stop       │   - WYKRES trendu w czasie rzeczywistym (Wartość zadana / Prędkość / Moment)      │
│  Wart. zadana     │                                                                                   │
│  Lepkość          │                                                                                   │
│  Nastawy PID      │                                                                                   │
├──────────────────┴──────────────────────────────────────────────────────────────────────────────────┤
│  ⇄ RAMKI NAMUR: mini-terminal (RX/TX) + wiersz poleceń + referencja protokołu (po prawej)             │
└───────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 4. Sterowanie mieszadłem (lewy panel)

### 4.1 Start / Stop
Przycisk **Start / Stop**. Po zatrzymaniu silnik swobodnie zwalnia aż do
unieruchomienia (tarcie + obciążenie), moment napędowy zerowy.

### 4.2 Wartość zadana prędkości
Suwak **Wartość zadana prędkości** (w `tr/min`), ograniczony przez wartości
min/max ustawione w *Parametrach*. To ta sama wielkość co komenda NAMUR
`OUT_SP_4` (kanał 4). Podczas pracy regulacja prowadzi zmierzoną prędkość do tej
wartości zadanej.

### 4.3 Lepkość ośrodka
Suwak **Lepkość** (skala logarytmiczna). Reprezentuje **obciążenie** mieszanego
ośrodka:

- **niska** lepkość → mały moment, wartość zadana osiągana szybko;
- **wysoka** lepkość → duży moment obciążenia; jeśli potrzebny moment przekroczy
  **maksymalny moment silnika**, zadana prędkość **nie jest już osiągana** →
  zapala się wskaźnik **Przeciążenie ⚠** (zachowanie rzeczywistego mieszadła wobec
  zbyt gęstego ośrodka).

### 4.4 Nastawy PID (Kp, Ki, Kd)
Trzy nastawy regulacji prędkości, regulowane na bieżąco:

- **Kp** (proporcjonalna): im większa, tym żywszy rozruch prędkości (ryzyko
  przeregulowania/oscylacji);
- **Ki** (całkująca): likwiduje w czasie szczątkowy uchyb prędkości;
- **Kd** (różniczkująca): tłumi/antycypuje (zbyt duża → wrażliwa na szum).

> Domyślne nastawy są celowo „ostre”: wyjście nasyca się przy maksymalnym
> momencie, dopóki uchyb jest duży (szybki rozruch), po czym człon całkujący
> stabilizuje. Wyjście PID **jest** momentem napędowym, ograniczonym do
> `[0, couple_max]`.

---

## 5. Odczyt wykresu trendu

Wykres (w centrum) kreśli trzy wielkości w czasie rzeczywistym. **Legenda, w
lewym górnym rogu**, przypomina kolor **i ostatnią wartość** każdej serii:

| Kolor | Seria | Znaczenie |
|---------|-------|---------------|
| 🔵 niebieski | **Wartość zadana** | zadana prędkość (podczas pracy) |
| 🔴 czerwony | **Prędkość** | zmierzona prędkość (`tr/min`, oś lewa) |
| 🟢 zielony | **Moment** | zmierzony moment (`N·cm`, **oś prawa**) |

> Wykres ma **dwie osie pionowe**: **prędkość** (`tr/min`) po lewej, **moment**
> (`N·cm`) po prawej. Moment jest skalowany, aby dzielić wykres, ale oś prawa
> faktycznie pokazuje `N·cm`.

Nad wykresem **karty** wyświetlają wartości chwilowe: **Prędkość**, **Moment**,
**Lepkość** oraz **Przeciążenie ⚠**, gdy silnik się nasyca. Wykres można
powiększać/przesuwać myszą.

---

## 6. Mini-terminal NAMUR (dół okna)

Panel **⇄ Ramki NAMUR** pozwala **testować protokół** bezpośrednio z GUI, bez
zewnętrznego klienta:

- **dziennik** wyświetla ramki **odebrane** (`← RX`, niebieski) i **wysłane**
  (`→ TX`, zielony), ze znacznikami czasu;
- **wiersz poleceń** wysyła ramkę NAMUR do symulatora (klawisz **Enter** lub
  przycisk **▶ Wyślij**). Strzałki **↑/↓** przywołują poprzednie komendy
  (historia);
- **referencja protokołu** (panel po prawej) wymienia komendy: **kliknięcie**
  wstawia komendę do wiersza wprowadzania;
- przycisk **🗑 Wyczyść** opróżnia dziennik.

> Ramki wpisane tutaj są interpretowane dokładnie tak samo jak te od mastera
> sieciowego: `OUT_SP_4 500` ustawia wartość zadaną, `START_4`/`STOP_4`
> startują/zatrzymują, `IN_PV_4` odczytuje prędkość itd. **Watchdog**
> (`OUT_WD1@…`) ma jednak efekt tylko w ramach prawdziwej sesji sieciowej
> (zob. § 8).

---

## 7. Parametry (przycisk ⚙)

Przycisk **⚙ Parametry** otwiera okno do konfiguracji:

### Język interfejsu
Selektor u góry: **Français, English, Deutsch, Español, Italiano, Português,
Nederlands, Polski** (8 języków). Język jest zachowywany.

### Transport NAMUR
Wybór łącza — **tylko jedno aktywne naraz**:

**TCP (Ethernet)**
- **IP nasłuchu** (`0.0.0.0` = wszystkie interfejsy) i **Port** (domyślnie 4001);
- **Dozwolone IP**: jedno na linię, dozwolone jokery `*` (np. `192.168.1.*`).
  **Pusta lista = wszystkie IP dozwolone.** Pozostałe są odrzucane.

**Szeregowy (RS-232)** — wymaga binarki skompilowanej z feature `serial`
- **Port szeregowy**: `/dev/ttyUSB0` (Linux), `COM3` (Windows)…;
- **Baud** (domyślnie 9600), **Parzystość** (domyślnie Parzysta), **Bity danych**
  (7), **Bity stopu** (1) — typowe ustawienie NAMUR laboratoryjne: **9600 7E1**.

> ⚠️ **Tylko jeden master naraz.** W TCP nowy master **czeka** aż poprzedni się
> rozłączy (łącze punkt-punkt). Lokalne GUI **nie jest** masterem. W trybie
> szeregowym magistrala *jest* jedynym masterem; zalecane **łącze punkt-punkt**
> (serwer odpowiada niezależnie od żądanego adresu).

### Parametry silnika
Symulowane zachowanie fizyczne `J·dω/dt = T − k·η·ω − tarcie`:
- **Bezwładność** (`J`): reaktywność silnika (mała ⇒ szybka);
- **Współczynnik obciążenia** (`k`): waga lepkości na momencie;
- **Tarcie** (`N·cm`): szczątkowe tarcie suche;
- **Moment maks.** (`N·cm`): maksymalny moment silnika (pułap wyjścia PID).

### Granice prędkości
Limity min/maks wartości zadanej prędkości (`tr/min`).

### Granice lepkości
Limity min/maks suwaka lepkości.

Przyciski: **Zastosuj** (działa natychmiast **i** zapisuje), **Przywróć
domyślne**, **Zamknij**.

### Zapisywanie nastaw
Nastawy są **zapisywane** w pliku `mock_su_namur.toml` (obok oprogramowania) i
**wczytywane ponownie przy następnym uruchomieniu**. Przycisk **💾 Zapisz** w
nagłówku zapisuje również nastawy PID i lepkość zmienione z lewego panelu.

---

## 8. Podłączanie klienta NAMUR

Oprogramowanie jest **slave'em NAMUR** (TCP port 4001 domyślnie lub szeregowy
zależnie od transportu wybranego w § 7). Klient (skrypt, terminal, bramka)
**wysyła jedną linię ASCII na żądanie**, zakończoną `CR LF`. **Odczyty** (`IN_*`)
zwracają wartość; **zapisy/akcje** (`OUT_*`, `START_*`, `STOP_*`, `RESET`) są
**ciche** (brak odpowiedzi), zgodnie z praktyką NAMUR.

Główne punkty odniesienia:

| Komenda | Efekt |
|----------|-------|
| `IN_NAME` / `IN_TYPE` | tożsamość (`CESAM-STIRRER` / `OSNE`) |
| `IN_PV_4` / `IN_PV_5` | odczyt prędkości (`tr/min`) / momentu (`N·cm`) |
| `IN_SP_4` | odczyt wartości zadanej prędkości |
| `OUT_SP_4 <v>` | **ustawienie** wartości zadanej prędkości |
| `START_4` / `STOP_4` / `RESET` | start / stop / reset |
| `OUT_WD1@<m>` | **watchdog**: bezpieczne zatrzymanie przy ciszy przez `<m>` s |

Przykład z `nc` (netcat):

```text
$ nc 127.0.0.1 4001
IN_NAME
CESAM-STIRRER
OUT_SP_4 1200          (cicho)
START_4                (cicho)
IN_PV_4
1200.0 4
STOP_4                 (cicho)
```

> **Watchdog** `OUT_WD1@30` automatycznie zatrzymuje silnik, jeśli **żadna linia**
> nie nadejdzie przez 30 s (zabezpieczenie na wypadek utraty komunikacji).
> `OUT_WD1@0` go rozbraja. Licznik jest zerowany przy każdej odebranej komendzie.

> **Pełna referencja protokołu** (kanały, kodowanie, przykłady) znajduje się w
> **[commandes_namur.md](commandes_namur.md)**. Ta sama lista jest przypominana
> **na żywo** w panelu po prawej stronie mini-terminala.

---

## 9. Użytkowanie bez ekranu („headless” / Docker)

Do wdrożenia w tle (Raspberry Pi bez ekranu, serwer) istnieje wersja **bez
interfejsu**: uruchamia symulację i serwer NAMUR, sterowalne **wyłącznie przez
NAMUR**.

```bash
# Obraz Docker (wdrażalny gdziekolwiek):
docker run --rm -p 4001:4001 -v "$PWD/conf:/data" osne:headless
```

Katalog zamontowany na `/data` pozwala dostarczyć/zachować `mock_su_namur.toml`.

---

## 10. Najczęstsze pytania

| Pytanie / objaw | Odpowiedź |
|---------------------|---------|
| **Przeciążenie ⚠** zapala się, a prędkość nie osiąga wartości zadanej. | Normalne: **lepkość** wymaga większego momentu, niż dostarcza silnik. Zmniejsz lepkość lub wartość zadaną, albo zwiększ **Moment maks.** (Parametry). |
| Prędkość nie zmienia się. | Sprawdź, czy mieszadło jest **W ruchu** i czy wartość zadana jest niezerowa. |
| Nagłówek pokazuje **NAMUR ✖**. | Port już zajęty lub < 1024 bez uprawnień (TCP), albo niedostępny port szeregowy: zmień ustawienie w ⚙ Parametry. |
| Mój klient NAMUR/TCP jest odrzucany. | Jego IP nie jest na **białej liście**: opróżnij listę lub dodaj wzorzec (`192.168.1.*`). |
| `OUT_SP_4 …` niczego nie zwraca. | Normalne: zapisy/akcje NAMUR są **ciche**. Odczytuj przez `IN_SP_4` / `IN_PV_4`. |
| Silnik zatrzymuje się sam. | Uzbrojony jest **watchdog** (`OUT_WD1@…`) i żadna komenda nie nadeszła na czas. Rozbrój go (`OUT_WD1@0`) lub regularnie wysyłaj ramki. |
| Łącze szeregowe się nie otwiera. | Binarka skompilowana **bez** feature `serial`, albo nieprawidłowy port/uprawnienia (grupa `dialout` pod Linux). |
| Moje nastawy nie są zachowywane. | Kliknij **Zastosuj** / **💾 Zapisz**. Plik `mock_su_namur.toml` musi być dostępny do zapisu. |

---

*Powiązana dokumentacja techniczna: [conception.md](conception.md) ·
[commandes_namur.md](commandes_namur.md) · [maintenance.md](maintenance.md).*
