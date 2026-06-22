# Dokument projektowy — Symulowany mieszadło laboratoryjne (OSNE)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · **PL***

> Crate: `mock_bin_su_namur` · Plik wykonywalny: **OSNE** (*Open Stirrer NAMUR Emulator*)

Dokument architektury i modelowania. Wzorowany na regulatorze **ORME**
(`mock_bin_ru_modbustcp`): ten sam podział na **synchroniczną logikę biznesową /
aktorów ractor / warstwę protokołu / GUI egui**, te same niezmienniki.

---

## 1. Cel

Symulacja **mieszadła laboratoryjnego** (typu IKA) sterowanego szeregowym
protokołem **NAMUR**. Silnik posiada **funkcję przejścia** (dynamikę prędkości)
nadzorowaną przez **szybką regulację**, a **lepkość** ośrodka jest regulowana
i wpływa na moment obrotowy.

---

## 2. Model fizyczny

### Silnik ([`motor.rs`](../../src/motor.rs))

Bilans momentów obrotowych, całkowany jawną metodą Eulera:

```text
J · dω/dt = T_moteur − k · η · ω − T_frottement
```

- `ω`: prędkość (tr/min);
- `T_moteur`: moment napędowy (sterowanie, N·cm, ≥ 0);
- `k · η · ω`: **lepki moment obciążenia** (∝ lepkość `η` i prędkość);
- `T_frottement`: szczątkowe tarcie suche;
- `J` (`inertia`): określa **reaktywność** (mała ⇒ szybka).

W stanie ustalonym `T_moteur = k·η·ω + T_frottement`: moment potrzebny do
utrzymania prędkości **rośnie wraz z lepkością**. Jeśli ten moment przekroczy
**maksymalny moment obrotowy**, wartość zadana staje się nieosiągalna →
**przeciążenie**.

### Regulacja ([`stirrer.rs`](../../src/stirrer.rs))

**Regulator PID** ([`mock_lib_control::Pid`], ponownie użyty z ORME) pobiera
uchyb prędkości `wartość zadana − pomiar` i wytwarza **moment napędowy**,
ograniczony do `[0, couple_max]`. Domyślne nastawy są celowo „ostre”: wyjście
nasyca się przy maksymalnym momencie, dopóki uchyb jest duży (szybki rozruch),
po czym człon całkujący stabilizuje. Krok symulacji wynosi **20 ms** (50 Hz),
drobniejszy niż w ORME, ponieważ dynamika silnika jest szybka.

---

## 3. Architektura (aktorzy)

```
GUI (egui) ──Command(cast)──►  SimulationActor ──refresh──► SharedSnapshot ──► GUI
Serwer NAMUR ──Command(cast)─►   (Stirrer)     ──refresh──► SharedSnapshot ──► odczyty NAMUR
```

- **`SimulationActor`** ([`actors/simulation.rs`](../../src/actors/simulation.rs)):
  jedyny właściciel `Stirrer`; posuwa symulację na ponownie uzbrajanym timerze
  jednorazowym (brak odłączonego timera) i publikuje `SharedSnapshot`.
- **`NamurServerActor`** ([`actors/network.rs`](../../src/actors/network.rs)):
  posiada serwer NAMUR, przeładowywalny na gorąco (`Reconfigure`); współdzielona
  biała lista IP; status nasłuchu publikowany dla GUI.
- **Serwer NAMUR** ([`namur_server.rs`](../../src/namur_server.rs)): czyta linie
  ASCII, interpretuje je ([`namur.rs`](../../src/namur.rs)), odpowiada na odczyty
  i przekazuje zapisy/akcje do aktora. **Jeden master naraz** (punkt-punkt).
  **Watchdog** na sesję.

Odczyty NAMUR czerpią z `SharedSnapshot` (brak osobnej tablicy pamięci jak
Modbus w ORME: protokół NAMUR jest zorientowany na „komendy”, a nie na
„rejestry”).

---

## 4. Konfiguracja i bezpieczeństwo

- `AppConfig` (język / sieć-szereg / silnik / regulacja) serializowany do **TOML**
  ([`config.rs`](../../src/config.rs)), **sanityzowany przy wczytaniu**
  (`AppConfig::sanitized`: uporządkowane granice, skończone liczby
  zmiennoprzecinkowe) — niezmiennik współdzielony z ORME (nigdy nie `clamp` z
  niezweryfikowanymi granicami).
- NAMUR **nie ma ani uwierzytelniania, ani szyfrowania**: zaufana sieć + biała
  lista IP (TCP). Domyślnie `0.0.0.0` + pusta lista ⇒ wystawiony: GUI wyświetla
  **baner ostrzegawczy**.

---

## 5. Kierunki rozwoju

- Kierunek obrotu (CW/CCW) i rampa przyspieszenia.
- Czujnik temperatury (`IN_PV_2/3`), jeśli zostanie dodany model termiczny.
- Nieliniowy moment obciążenia (reżim turbulentny ∝ ω²).
- Promocja modelu silnika do `mock_lib_control`, jeśli posłuży drugiemu
  przyrządowi.
