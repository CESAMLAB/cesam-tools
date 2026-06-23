# Podręcznik użytkownika — symulowany regulator procesu (RU/OPC UA)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · **PL***

> Crate: `mock_bin_ru_opcua` · Plik wykonywalny: **ru_opcua**

---

## 1. Do czego służy ten symulator

`ru_opcua` symuluje **regulator procesu** (pętla PID na procesie termicznym) i
udostępnia go przez **OPC UA**, standard przemysłowego nadzoru. Służy do
**testowania klienta OPC UA / systemu SCADA** (odczyt pomiarów, zapis wartości
zadanych, subskrypcje) bez rzeczywistego sprzętu.

Interfejs graficzny pozwala **sterować** symulacją i **wizualizować** dynamikę;
serwer OPC UA udostępnia te same wielkości w sieci.

---

## 2. Pierwsze kroki

```bash
cargo run -p mock_bin_ru_opcua          # GUI + serwer OPC UA
```

Po uruchomieniu serwer nasłuchuje domyślnie na `opc.tcp://0.0.0.0:4840/`
(bezpieczeństwo None). Okno wyświetla stan bieżący i uruchamia wykres trendu.

Podłącz klienta OPC UA (UaExpert itp.) do `opc.tcp://127.0.0.1:4840/`,
bezpieczeństwo **None**, użytkownik **Anonymous**. Węzły są opisane w
[referencji OPC UA](reference_opcua.md).

---

## 3. Interfejs

### Nagłówek

- **Tytuł** oraz przyciski **⚙ Ustawienia** / **💾 Zapisz ustawienia**.
- Po prawej: **stan urządzenia** (W RUCHU / ZATRZYMANE), **stan serwera**
  (`OPC UA ● opc.tcp://…` na zielono, gdy nasłuchuje, ✖ + komunikat w razie błędu)
  oraz **logo CESAM-Lab**.
- **Pomarańczowy baner** stale przypomina, że endpoint jest **anonimowy
  (bezpieczeństwo None)**: udostępniać wyłącznie w sieci zaufanej.
- Jeśli dostępna jest aktualizacja, **baner** proponuje pobranie.

### Panel sterowania (po lewej)

- **Praca / Stop**: uruchamia lub zatrzymuje regulację. Po zatrzymaniu proces
  relaksuje do wartości otoczenia.
- **Tryb automatyczny (PID)**: włączony = PID oblicza wyjście; wyłączony =
  **tryb ręczny** (wyjście jest narzucone).
- **Wartość zadana**: suwak, ograniczony granicami wartości zadanej (regulowanymi
  w *Ustawieniach*).
- **Wyjście ręczne (%)**: suwak aktywny tylko w **trybie ręcznym**.
- **Ustawienia PID**: wzmocnienia `Kp`, `Ki`, `Kd` edytowalne na gorąco.

### Strefa centralna

- **Karty**: Pomiar, Wartość zadana, Wyjście.
- **Wykres trendu**: Pomiar (PV) i Wartość zadana na osi lewej (jednostka
  procesu), Wyjście (%) na osi prawej.

---

## 4. Ustawienia (okno modalne ⚙)

- **Język** interfejsu (8 języków), trwały.
- **Sprawdzaj aktualizacje przy starcie** + przycisk **Sprawdź teraz**.
- **Endpoint**: **IP nasłuchiwania** i **port** serwera OPC UA. Zmiana
  **przeładowuje** serwer na gorąco (bieżące sesje są zamykane czysto).
- **Bezpieczeństwo OPC UA**: **Szyfrowanie** (`Basic256Sha256`), **Zezwalaj na
  anonimowość**, **Użytkownik** / **Hasło** (pola aktywne, gdy szyfrowanie jest
  zaznaczone). Włączenie szyfrowania generuje certyfikat przy pierwszym
  uruchomieniu (kilka sekund) i restartuje serwer.
- **Proces (funkcja przejścia)**: wzmocnienie `K`, stała czasowa `τ`, czyste
  opóźnienie, wartość otoczenia.
- **Granice wartości zadanej**: min / max (porządkowane automatycznie, jeśli
  odwrócone).
- **Zastosuj** / **Przywróć domyślne** / **Zamknij**.

Ustawienia są zapisywane w `mock_ru_opcua.toml` (katalog bieżący; nadpisywalny
przez zmienną środowiskową `MOCK_CONFIG`).

---

## 5. Bezpieczeństwo

Bezpieczeństwo OPC UA jest **konfigurowalne** w *Ustawieniach*:

- **Bez szyfrowania** (domyślnie): endpoint **bezpieczeństwo None**, dostęp
  **anonimowy** — brak jakiejkolwiek ochrony. **Nie udostępniać w sieci
  otwartej.** Przypomina o tym **pomarańczowy** baner.
- **Z szyfrowaniem**: endpoint **`Basic256Sha256`** (podpisany + szyfrowany).
  Serwer generuje swój certyfikat przy pierwszym uruchomieniu i akceptuje
  certyfikaty klientów. Można wymagać **użytkownika / hasła** i/lub zezwolić na
  anonimowość. **Zielony baner 🔒** potwierdza szyfrowanie. Aby się połączyć,
  klient musi wtedy użyć polityki `Basic256Sha256` i zaufać certyfikatowi serwera
  (pierwsza wymiana).

Hasło jest przechowywane **jawnym tekstem** w pliku TOML: to **symulator**,
do użycia w sieci zaufanej.

---

## 6. FAQ

**Czy port 4840 jest narzucony?** Nie: ustawia się go w *Ustawieniach* (lub przez
plik TOML). Port < 1024 wymaga uprawnień root.

**Mój klient nie widzi węzłów.** Sprawdź połączenie z `opc.tcp://…:4840/`,
bezpieczeństwo **None**, użytkownik **Anonymous**, następnie *Browse* pod folderem
`Objects` (namespace `urn:cesam-lab:ru-opcua`).

**Zapis jest odrzucany.** Typ musi się zgadzać (`Double` dla wielkości, `Boolean`
dla `Run`/`Auto`); w przeciwnym razie serwer zwraca `Bad_TypeMismatch`.

**Uruchomić bez interfejsu graficznego?** Skompiluj w trybie *headless*:
`cargo run -p mock_bin_ru_opcua --no-default-features` — serwer OPC UA i symulacja
działają bez GUI.

**Pojawia się komunikat „encrypted endpoints disabled”.** To normalne w Fazie 1b:
żaden certyfikat instancji nie jest zaprovisjonowany (szyfrowane endpointy
niedostępne). Endpoint None działa.
