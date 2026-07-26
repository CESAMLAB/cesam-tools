# Utrzymanie — Regulator EtherNet/IP (OREE)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · **PL***

---

## 1. Budowanie i uruchamianie

```bash
cargo run -p mock_bin_ru_ethernetip                        # GUI + adapter EtherNet/IP
cargo build -p mock_bin_ru_ethernetip --release            # plik wykonywalny GUI
cargo build -p mock_bin_ru_ethernetip --no-default-features # headless (bez GUI)
```

Funkcje: `gui` (GUI `egui`, domyślnie). `--no-default-features` tworzy plik binarny
**headless**: adapter EtherNet/IP + symulacja, bez GUI i bez sprawdzania
aktualizacji. Port 44818 nie wymaga **żadnych uprawnień**.

## 2. Konfiguracja

Plik TOML `mock_ru_ethernetip.toml` (bieżący katalog; ścieżkę można nadpisać przez
`MOCK_CONFIG`). Sekcje: `language`, `[network]` (`bind_ip`, `port`, `allowlist`),
`[process]`, `[regulation]`, `check_updates`. Każda wartość jest **sanityzowana**
podczas wczytywania.

## 3. Testy

```bash
cargo test -p mock_bin_ru_ethernetip      # jednostkowe + lokalny round-trip TCP
```

- **Warstwa protokołu** (`eip_server`, bez sieci): RegisterSession, Read/Write Tag,
  zapis BOOL, nieznany tag (`0x05`), zapis tagu tylko do odczytu, **brak paniki**
  przy zniekształconych pakietach.
- **Aktor sieciowy**: bind/nasłuch oraz **rzeczywisty round-trip TCP**
  (RegisterSession, Write, a następnie Read wartości zadanej) — bez zależności od
  zewnętrznego klienta.

## 4. Rozwiązywanie problemów

| Objaw | Trop |
|---|---|
| Klient odrzucony | biała lista IP; zapora; IP/port (44818) |
| Tag nieodnaleziony | nieprawidłowa nazwa (wielkość liter); zob. tabela tagów |
| Zapis bez efektu | tag tylko do odczytu |
| Niespójne wartości | EtherNet/IP jest **little-endian** (REAL = `f32` LE) |

## 5. Docker (headless)

Obraz headless przez `scripts/build-prod.sh` (wpis
`mock_bin_ru_ethernetip:ru_eip:44818`, `EXPOSE 44818`). Zamontuj wolumin na katalogu
roboczym, aby dostarczyć `mock_ru_ethernetip.toml`.

## 6. Rozszerzanie tabeli tagów

Tabela tagów i mapowanie zapisów są **źródłem prawdy** w
[`eip_server.rs`](../../src/eip_server.rs) (`read_tag` + `write_tag`). Aby dodać tag:
dodaj go do `read_tag` (odczyt) oraz, jeśli sterowalny, do `write_tag` (zapis →
`Command`), a następnie odzwierciedlij to tutaj i w
[`reference_ethernetip.md`](reference_ethernetip.md). Dodaj test w module.

## 7. Cross / Windows

Jak pozostałe instrumenty (zob. `Cross.toml`). Brak szczególnych zależności
natywnych: warstwa EtherNet/IP jest w 100 % w Rust na standardowym TCP.
