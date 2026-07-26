# Podręcznik użytkownika — Regulator S7 (ORSS)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · **PL***

---

## 1. Do czego służy instrument

**ORSS** symuluje **jednostkę regulacji** procesu (PID + proces cieplny pierwszego
rzędu) i udostępnia ją jako **sterownik Siemens S7** (serwer S7comm na
ISO-on-TCP). Służy do testowania systemu nadzoru lub klienta S7 (Snap7, TIA Portal w
trybie odczytu, nodes7…) bez rzeczywistego sterownika.

## 2. Pierwsze kroki

```bash
cargo run -p mock_bin_ru_s7        # GUI + serwer S7
```

Serwer nasłuchuje domyślnie na `0.0.0.0:102`. ⚠️ **Port 102 wymaga uprawnień
root**; w przeciwnym razie ustaw port wysoki (np. 1102) w oknie modalnym *Ustawienia*.

Nagłówek wskazuje stan: **S7 ●** (zielony) z adresem nasłuchu lub komunikat
o błędzie (czerwony), jeśli bind się nie powiedzie. Pomarańczowy baner ostrzega, jeśli serwer jest
**wystawiony** (wszystkie interfejsy + pusta lista dozwolonych).

## 3. Interfejs

- **Nagłówek**: tytuł, przyciski *Ustawienia* / *Zapisz*, stan praca/zatrzymanie, stan
  nasłuchu S7, baner wystawienia sieciowego.
- **Panel lewy (Polecenia)**: *Praca/Zatrzymanie*, *Tryb automatyczny (PID)*,
  *Nastawa*, *Wyjście ręczne* (tryb ręczny), ustawienia **PID** (Kp/Ki/Kd).
- **Panel środkowy**: karty *Pomiar / Nastawa / Wyjście* + **wykres** w czasie rzeczywistym.
- **Okno modalne *Ustawienia***: język, sprawdzanie aktualizacji, **sieć S7** (IP nasłuchu,
  port, **lista dozwolonych** adresów IP — jeden wzorzec na wiersz, `*` = symbol wieloznaczny), **proces**
  (K, τ, opóźnienie, otoczenie), **granice nastawy**. *Zastosuj* ponownie uruchamia nasłuch, jeśli
  zmieni się IP/port, i zapisuje TOML.

## 4. Podłączenie klienta S7

Klient łączy się z IP/portem serwera. Zwyczajowe wartości **rack/slot**
(0/1 lub 0/2) działają: serwer nie narzuca TSAP. Wielkości znajdują się w
**DB1** (zob. [`reference_s7.md`](reference_s7.md)): nastawa w `DB1.DBD0`, pomiar
w `DB1.DBD4`, praca w `DB1.DBX16.0` itd.

## 5. FAQ

- **„Permission denied" przy uruchomieniu** → port 102 wymaga uprawnień root;
  użyj portu wysokiego lub uruchom z odpowiednimi przywilejami.
- **Klient się nie łączy** → sprawdź IP/port, **listę dozwolonych**,
  zaporę. Przetestuj rack/slot 0/1, a następnie 0/2.
- **Moje zapisy nie mają efektu** → działają tylko sterowalne offsety
  (nastawa, wyjście ręczne, praca, auto); pozostałe są tylko do odczytu.
- **Gdzie jest plik konfiguracyjny?** → `mock_ru_s7.toml` (katalog bieżący;
  można nadpisać przez `MOCK_CONFIG`).
