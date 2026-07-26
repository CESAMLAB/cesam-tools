# Utrzymanie — Regulator Sparkplug B (ORSE)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · **PL***

---

## 1. Budowanie i uruchamianie

```bash
cargo run -p mock_bin_ru_sparkplugb                       # GUI + węzeł brzegowy
cargo build -p mock_bin_ru_sparkplugb --release           # plik wykonywalny GUI
cargo build -p mock_bin_ru_sparkplugb --no-default-features # headless (bez GUI)
```

Funkcje: `gui` (GUI `egui`, domyślnie). `--no-default-features` produkuje plik binarny
**headless**: węzeł brzegowy Sparkplug B + symulacja, bez GUI i bez sprawdzania aktualizacji.

## 2. Konfiguracja

Plik TOML `mock_ru_sparkplugb.toml` (katalog bieżący; ścieżkę można nadpisać przez
`MOCK_CONFIG`). Sekcje: `language`, `[network]` (broker/Sparkplug), `[process]`,
`[regulation]`, `check_updates`. Zobacz [`reference_sparkplugb.md`](reference_sparkplugb.md)
dla kluczy `[network]`. Każda wartość jest **sanityzowana** przy ładowaniu.

## 3. Testy

```bash
cargo test -p mock_bin_ru_sparkplugb              # jednostkowe (bez brokera)
cargo test -p mock_bin_ru_sparkplugb -- --ignored # round-trip z lokalnym brokerem
```

- **Jednostkowe** (bez sieci): regulacja, sanityzacja konfiguracji, a przede wszystkim
  warstwa Sparkplug (topiki, ładunki `NBIRTH`/`NDEATH`, round-trip encode/decode,
  mapowanie `NCMD`, odrzucenie błędnego typu, zawinięcie `seq` 255→0).
- **Integracyjne `#[ignore]`**: wymaga lokalnego brokera MQTT —
  `docker run -it --rm -p 1883:1883 eclipse-mosquitto` — następnie uruchamia pełny
  round-trip (odebrany NBIRTH, zastosowany NCMD, odzwierciedlony NDATA).

## 4. Rozwiązywanie problemów

| Objaw | Trop |
|---|---|
| Trwałe „Rozłączono" | broker nieosiągalny (`broker_host`/`broker_port`, zapora, zatrzymany broker) |
| SCADA nic nie odbiera | `group_id`/`edge_node_id`; subskrypcja `spBv1.0/<group>/#`; ładunki protobuf |
| Niepowodzenie TLS | broker w trybie TLS na 8883; certyfikat główny rozpoznawany przez system |
| NCMD ignorowany | metryka niesterowalna lub błędny typ (zob. tabela metryk) |

## 5. Docker (headless)

Obraz headless buduje się przez `scripts/build-prod.sh` (wpis
`mock_bin_ru_sparkplugb:ru_spb:0`). Ponieważ ORSE jest **klientem**, **nie udostępnia żadnego
portu** (`PORT=0`, `EXPOSE 0` = bezczynne metadane) i **żaden `HEALTHCHECK`** TCP
nie jest istotny: liveness stwierdza się po stronie brokera poprzez **Last Will/NDEATH**.
Zamontuj wolumin na katalogu roboczym, aby dostarczyć `mock_ru_sparkplugb.toml`.

## 6. Rozszerzanie

Tabela metryk i mapowanie `NCMD` są **źródłem prawdy** w
[`sparkplug_node.rs`](../../src/sparkplug_node.rs). Aby dodać metrykę:
dodaj ją do `data_metrics`/`changed_metrics` (odczyt) oraz, jeśli sterowalna, do
`ncmd_to_actions` (zapis → `Command`), następnie odzwierciedl to tutaj oraz w
[`reference_sparkplugb.md`](reference_sparkplugb.md). Dodaj test w module.

## 7. Istotne zależności

- `rumqttc` (klient MQTT, rustls), `sparkplug-rs` (protobuf Tahu, generowanie kodu w czystym Rust).
- MSRV: do zweryfikowania po pełnym buildzie `cross` (może przekroczyć dolny próg 1.85
  workspace'u w zależności od zależności rustls).
