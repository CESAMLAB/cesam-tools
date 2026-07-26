# Referencja MQTT Sparkplug B — metryki i cykl życia (RU/Sparkplug B)

*🌍 [FR](../fr/reference_sparkplugb.md) · [EN](../en/reference_sparkplugb.md) · [DE](../de/reference_sparkplugb.md) · [ES](../es/reference_sparkplugb.md) · [IT](../it/reference_sparkplugb.md) · [PT](../pt/reference_sparkplugb.md) · [NL](../nl/reference_sparkplugb.md) · **PL***

> Źródło prawdy: [`sparkplug_node.rs`](../../src/sparkplug_node.rs) (topiki, tabela
> metryk, ładunki, mapowanie NCMD). Każda zmiana następuje **w tym pliku**
> i jest tutaj odzwierciedlana.

---

## 1. Rola i połączenie

Instrument jest **węzłem brzegowym Sparkplug B**: **nie nasłuchuje na żadnym porcie**,
**łączy się wychodząco** z **zewnętrznym brokerem MQTT** (mosquitto, EMQX, HiveMQ…) i
publikuje stan regulatora. Ustawienia w sekcji `[network]` pliku TOML / modalu
*Ustawienia*:

| Klucz | Domyślnie | Rola |
|---|---|---|
| `broker_host` | `localhost` | host brokera MQTT |
| `broker_port` | `1883` | port (`8883` w TLS) |
| `client_id` | `ru_spb` | identyfikator klienta MQTT |
| `group_id` | `CESAM` | grupa Sparkplug (`spBv1.0/<group_id>/…`) |
| `edge_node_id` | `RU1` | węzeł brzegowy (`…/<edge_node_id>`) |
| `username` / `password` | *(puste)* | uwierzytelnianie MQTT (hasło **w postaci jawnej**, wyłącznie symulator) |
| `tls` | `false` | szyfrowanie TLS (rustls) do brokera |
| `keepalive_secs` | `30` | keepalive MQTT |
| `publish_on_change` | `true` | `true`: `NDATA` gdy tylko zmieni się metryka (częstotliwość = krok symulacji, 0,5 s); `false`: okresowo |
| `publish_period_secs` | `5` | częstotliwość okresowa, gdy `publish_on_change = false` |

> ⚠️ **MQTT w postaci jawnej domyślnie**: bez TLS ruch nie jest ani szyfrowany, ani
> uwierzytelniany sieciowo. Używać wyłącznie w **sieci zaufanej**. GUI wyświetla
> baner ostrzegawczy dopóki `tls` jest wyłączone.

---

## 2. Przestrzeń nazw (topiki)

Przestrzeń nazw `spBv1.0`. Topiki węzła:

```
spBv1.0/<group_id>/NBIRTH/<edge_node_id>
spBv1.0/<group_id>/NDATA/<edge_node_id>
spBv1.0/<group_id>/NDEATH/<edge_node_id>
spBv1.0/<group_id>/NCMD/<edge_node_id>
```

Z wartościami domyślnymi: `spBv1.0/CESAM/NBIRTH/RU1`, itd.

---

## 3. Tabela metryk

Wszystkie metryki danych znajdują się pod **węzłem brzegowym** (brak *device* w
tej wersji). Typ Sparkplug (Eclipse Tahu): `Float` (9), `Boolean` (11),
`UInt64` (8).

| Metryka | Typ | Odczyt/Zapis | Pole migawki (odczyt) | NCMD → polecenie (zapis) |
|---|---|:--:|---|---|
| `Setpoint` | Float | R/W | `setpoint` | `SetSetpoint` |
| `ProcessValue` | Float | R | `pv` | — |
| `Output` | Float | R | `output` | — |
| `ManualOutput` | Float | R/W | `manual_output` | `SetManualOutput` |
| `Run` | Boolean | R/W | `run` | `SetRun` |
| `Auto` | Boolean | R/W | `auto` | `SetAuto` |
| `SetpointMin` | Float | R | `sp_min` | *(ustawiane przez GUI/TOML)* |
| `SetpointMax` | Float | R | `sp_max` | *(ustawiane przez GUI/TOML)* |
| `PID/Kp` | Float | R | `pid.kp` | *(ustawiane przez GUI/TOML)* |
| `PID/Ki` | Float | R | `pid.ki` | *(ustawiane przez GUI/TOML)* |
| `PID/Kd` | Float | R | `pid.kd` | *(ustawiane przez GUI/TOML)* |
| `bdSeq` | UInt64 | R | *(licznik sesji)* | — |
| `Node Control/Rebirth` | Boolean | W | — | ponownie publikuje `NBIRTH` |

**Powierzchnia sterowalna przez `NCMD`**: `Setpoint`, `ManualOutput`, `Run`, `Auto`, plus
`Node Control/Rebirth` (parytet z zapisami OPC UA instrumentu ORUE).
Granice wartości zadanej i wzmocnienia PID są **publikowane** (obserwowalne przez SCADA), ale
ustawia się je przez GUI/TOML. Nieznana metryka lub o **błędnym typie** w
`NCMD` jest **ignorowana** (nigdy błędu, nigdy aberracyjnej wartości: symulacja
sanityzuje każdy zapis).

---

## 4. Cykl życia

- **`NBIRTH`** — publikowany przy każdym połączeniu (ConnAck). Zawiera **wszystkie**
  metryki (z wartościami), `bdSeq` oraz `Node Control/Rebirth`. `seq = 0`.
- **`NDATA`** — wyłącznie **zmienione** metryki, `seq` zawijający **0–255**.
- **`NDEATH`** — zawiera **sam** `bdSeq`, **bez** `seq`. Zdeponowany jako **Last Will
  MQTT** przy połączeniu: **broker** publikuje go automatycznie przy utracie łącza
  (zatrzymanie, rekonfiguracja, awaria). Brak jawnego `NDEATH` po stronie węzła.
- **`NCMD`** — subskrypcja `spBv1.0/<group>/NCMD/<node>` (QoS 1) zasubskrybowana zaraz po
  `NBIRTH`. Dekodowany → polecenia stosowane do symulacji.
- **`bdSeq`** — inkrementowany przy każdym (ponownym) uruchomieniu klienta; `NDEATH` (Last Will)
  i `NBIRTH` tej **samej sesji** niosą **tę samą** wartość (niezmiennik
  Sparkplug). Wyświetlany w GUI (diagnostyka).
- **`seq`** — zerowany przy każdym `NBIRTH`, inkrementowany (zawijający) przy każdym `NDATA`.
- **Odrodzenie** (`Node Control/Rebirth = true` przez `NCMD`) → ponowna publikacja
  `NBIRTH` (resynchronizacja SCADA).

---

## 5. Przykład klienta (SCADA)

Subskrypcja całej grupy, następnie wysłanie wartości zadanej:

```bash
# Obserwowanie komunikatów węzła
mosquitto_sub -h localhost -t 'spBv1.0/CESAM/#' -v

# (ładunki to protobuf Sparkplug B — użyj dekodera Tahu, aby je odczytać)
```

`NCMD` opublikowany na `spBv1.0/CESAM/NCMD/RU1` z metrykami `Run=true` i
`Setpoint=80.0` uruchamia regulację i ustawia wartość zadaną; późniejszy `NDATA`
odzwierciedla zmianę.
