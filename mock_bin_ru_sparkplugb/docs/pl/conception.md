# Projekt — Regulator Sparkplug B (ORSE)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · **PL***

---

## 1. Przegląd ogólny

ORSE ponownie wykorzystuje architekturę pozostałych instrumentów CESAM-Lab: **synchroniczny
i testowalny model biznesowy** (regulator PID + proces), sterowany przez **aktory
`ractor`** na Tokio, oraz **GUI `egui`**, które odczytuje współdzieloną migawkę. Zmienia się
jedynie **warstwa transportowa**: tutaj jest to **węzeł brzegowy MQTT Sparkplug B** (klient wychodzący)
zamiast serwera Modbus/OPC UA.

```
        Command (cast)                      odświeżanie co krok
GUI   ───────────────────►  SimulationActor ───────────────────►  SharedSnapshot (GUI)
NCMD (broker) ───────────►  (Regulator)      ──────────────────►  SharedSnapshot (publikacja)
NBIRTH/NDATA (broker) ◄──────────────────────  SharedSnapshot
```

## 2. Aktory

- **`SimulationActor`** — posiada jedyny [`Regulator`]. Pętla o stałym kroku (`Tick`
  co 0,5 s); stosuje `Command` (GUI lub NCMD); publikuje migawkę
  po każdej mutacji. Identyczny jak w pozostałych instrumentach.
- **`SparkplugActor`** — posiada **klienta MQTT** (`rumqttc`) i wykonuje **cykl
  życia Sparkplug B** w dedykowanym zadaniu tokio (którego `JoinHandle` jest przerywany przy
  zatrzymaniu). Komunikat `Reconfigure` ponownie uruchamia klienta, jeśli zmienią się broker/poświadczenia/
  TLS.

## 3. Warstwa protokołu

[`sparkplug_node.rs`](../../src/sparkplug_node.rs) jest **czysty i synchroniczny** (bez żadnej
zależności od tokio/rumqttc): budowa **topiców**, tabela **metryk**,
wytwarzanie **ładunków** (`NBIRTH`/`NDATA`/`NDEATH`), (de)serializacja protobuf,
mapowanie **`NCMD` → polecenia** oraz licznik `seq`. Jest to odpowiednik
`opcua_server.rs` z ORUE, wyizolowany tak, by był **testowalny bez brokera**.

### Wybór bibliotek

- **`rumqttc`** — asynchroniczny klient MQTT Tokio (Last Will, automatyczne ponowne połączenie, TLS
  przez rustls — już obecny w drzewie zależności dzięki OPC UA, **bez OpenSSL**).
- **`sparkplug-rs`** — struktury protobuf Eclipse Tahu (`Payload`/`Metric`/`Value`),
  generowane w **100% w Rust** (rust-protobuf, **bez `protoc`** → czysta kompilacja skrośna). Skrzynka
  re-eksportuje `protobuf` (runtime), używany do `write_to_bytes`/`parse_from_bytes`.
- **Odrzucona alternatywa: `srad`** — wysokopoziomowy framework węzła brzegowego Sparkplug, który sam
  zarządza `bdSeq`/`seq`/rebirth. Odrzucony celowo: to my **posiadamy** maszynę
  stanów w aktorze sieciowym, aby uczynić ją jawną i testowalną (spójność z
  pozostałymi instrumentami).

## 4. Cykl życia i niezmienniki

- **`bdSeq`** inkrementowany przy każdym (ponownym) uruchomieniu klienta; **ta sama** wartość w
  Last Will `NDEATH` i w `NBIRTH` danej sesji.
- **`seq`** zawijający 0–255, zerowany przy każdym `NBIRTH`.
- **`NDEATH`** przenoszony przez **Last Will MQTT**: odporny na każdą utratę łącza.
- **Publikacja `NDATA`** poprzez **różnicę** migawek (częstotliwość = krok symulacji w
  trybie *przy zmianie*, lub okresowa). Blokada migawki **nigdy** nie jest utrzymywana
  przez `.await`.

## 5. Postawa bezpieczeństwa

- **Brak białej listy IP** (instrument jest klientem, a nie serwerem): odstępstwo
  od parytetu **świadomie przyjęte** względem ORME/OSNE.
- **MQTT w postaci jawnej domyślnie** (port 1883) — nieszyfrowany, bez uwierzytelniania
  sieciowego. Baner ostrzegawczy w GUI. Aby wyjść poza
  sieć zaufaną, należy włączyć **TLS** + poświadczenia.
- **Hasło w postaci jawnej** w pliku TOML — **wyłącznie symulator**.
- **Sanityzacja TOML** ([`AppConfig::sanitized`](../../src/config.rs)): proces/
  PID/granice skończone i uporządkowane, niepuste identyfikatory Sparkplug, ograniczone
  opóźnienia czasowe. Każdy zapis NCMD jest **przycinany/sanityzowany** przez `Regulator::apply`.
