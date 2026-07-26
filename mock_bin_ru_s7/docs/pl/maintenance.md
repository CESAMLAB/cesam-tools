# Utrzymanie — Regulator S7 (ORSS)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · **PL***

---

## 1. Build i uruchomienie

```bash
cargo run -p mock_bin_ru_s7                        # GUI + serwer S7
cargo build -p mock_bin_ru_s7 --release            # plik wykonywalny GUI
cargo build -p mock_bin_ru_s7 --no-default-features # headless (bez GUI)
```

Features: `gui` (GUI `egui`, domyślnie). `--no-default-features` wytwarza binarkę
**headless**: serwer S7 + symulacja, bez GUI i bez sprawdzania aktualizacji.

⚠️ Port **102** (standard S7) jest uprzywilejowany (< 1024): uruchamiaj z odpowiednimi
uprawnieniami lub wybierz port wysoki w konfiguracji.

## 2. Konfiguracja

Plik TOML `mock_ru_s7.toml` (katalog bieżący; ścieżkę można nadpisać przez
`MOCK_CONFIG`). Sekcje: `language`, `[network]` (`bind_ip`, `port`, `allowlist`),
`[process]`, `[regulation]`, `check_updates`. Każda wartość jest **sanityzowana** podczas
ładowania.

## 3. Testy

```bash
cargo test -p mock_bin_ru_s7      # jednostkowe + lokalny round-trip TCP
```

- **Warstwa protokołu** (`s7_server`, bez sieci): CR→CC, Setup, Read/Write Var,
  zapis bitu, kod powrotu poza strefą, **brak paniki** przy zniekształconych ramkach,
  round-trip obrazu DB.
- **Aktor sieciowy**: bind/nasłuch oraz **rzeczywisty round-trip TCP** (połączenie COTP,
  zapis, a następnie ponowny odczyt nastawy poprzez surowe ramki S7) — bez zależności od
  zewnętrznego klienta.

## 4. Rozwiązywanie problemów

| Objaw | Trop |
|---|---|
| Bind nie powiódł się (`permission denied`) | port 102 < 1024 → uprawnienia root lub port wysoki |
| Klient odrzucony | lista dozwolonych adresów IP; zapora; IP/port |
| Brak odpowiedzi | rack/slot (przetestuj 0/1, 0/2); ramki poza podzbiorem ignorowane |
| Zapis bez efektu | offset tylko do odczytu (zob. plan adresowania) |

## 5. Docker (headless)

Obraz headless przez `scripts/build-prod.sh` (wpis `mock_bin_ru_s7:ru_s7:102`,
`EXPOSE 102`). Zamontuj wolumin na katalogu roboczym, aby dostarczyć
`mock_ru_s7.toml`. Kontener publikuje port 102; w razie potrzeby zmapuj go na port wysoki po
stronie hosta.

## 6. Rozszerzanie planu adresowania

Plan DB1 i mapowanie zapisów są **źródłem prawdy** w
[`s7_server.rs`](../../src/s7_server.rs) (`db_image` + `handle_write`). Aby dodać
wielkość: zapisz ją w `db_image` (odczyt) oraz, jeśli ma być sterowalna, dodaj ją do
`match` w `handle_write` (zapis → `Command`), a następnie odzwierciedl tutaj i w
[`reference_s7.md`](reference_s7.md). Dodaj test w module.

## 7. Cross / Windows

Tak jak pozostałe instrumenty (zob. `Cross.toml`). Brak szczególnej zależności
natywnej: warstwa S7 jest w 100 % w Rust na standardowym TCP.
