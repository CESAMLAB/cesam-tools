# Podręcznik użytkownika — Regulator EtherNet/IP (OREE)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · **PL***

---

## 1. Do czego służy instrument

**OREE** symuluje **jednostkę regulacji** procesu (PID + proces termiczny pierwszego
rzędu) i udostępnia ją jako **adapter EtherNet/IP** (jawna komunikacja CIP). Służy do
testowania systemu nadzoru lub klienta EtherNet/IP (pycomm3, RSLinx do odczytu,
rseip…) bez rzeczywistego sprzętu.

## 2. Pierwsze kroki

```bash
cargo run -p mock_bin_ru_ethernetip        # GUI + adapter EtherNet/IP
```

Serwer domyślnie nasłuchuje na `0.0.0.0:44818` (nie są wymagane żadne uprawnienia).
Nagłówek wskazuje stan: **EtherNet/IP ●** (zielony) z adresem nasłuchu lub komunikat
błędu (czerwony). Pomarańczowy baner ostrzega, jeśli serwer jest **wystawiony**
(wszystkie interfejsy + pusta biała lista).

## 3. Interfejs

- **Nagłówek**: tytuł, przyciski *Ustawienia* / *Zapisz*, stan praca/zatrzymanie, stan
  nasłuchu EtherNet/IP, baner ekspozycji sieciowej.
- **Lewy panel (Polecenia)**: *Praca/Zatrzymanie*, *Tryb automatyczny (PID)*, *Wartość
  zadana*, *Wyjście ręczne* (tryb ręczny), ustawienia **PID** (Kp/Ki/Kd).
- **Panel centralny**: karty *Pomiar / Wartość zadana / Wyjście* + **wykres** czasu
  rzeczywistego.
- **Modal *Ustawienia***: język, sprawdzanie aktualizacji, **sieć EtherNet/IP** (IP
  nasłuchu, port, **biała lista** IP — jeden wzorzec na wiersz, `*` = symbol
  wieloznaczny), **proces** (K, τ, opóźnienie, otoczenie), **granice wartości
  zadanej**. *Zastosuj* ponownie uruchamia nasłuch, jeśli zmieni się IP/port, i
  zapisuje TOML.

## 4. Podłączenie klienta EtherNet/IP

Klient łączy się z IP/portem serwera (automatyczny `RegisterSession`), a następnie
odczytuje/zapisuje **nazwane tagi** poprzez jawną komunikację: `Setpoint`,
`ProcessValue`, `Output`, `ManualOutput`, `Run`, `Auto` itd. (zob.
[`reference_ethernetip.md`](reference_ethernetip.md)). ⚠️ Wartości są w
**little-endian** (REAL = `f32` LE).

## 5. FAQ

- **Klient się nie łączy** → sprawdź IP/port (44818), **białą listę**, zaporę.
- **Tag nieodnaleziony** → istnieją tylko udokumentowane tagi; nazwy rozróżniają
  wielkość liter.
- **Moje zapisy nie mają efektu** → działają tylko tagi sterowalne (`Setpoint`,
  `ManualOutput`, `Run`, `Auto`); pozostałe są tylko do odczytu.
- **Gdzie jest plik konfiguracji?** → `mock_ru_ethernetip.toml` (bieżący katalog;
  można nadpisać przez `MOCK_CONFIG`).
