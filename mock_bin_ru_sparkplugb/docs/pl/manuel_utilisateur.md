# Podręcznik użytkownika — Regulator Sparkplug B (ORSE)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · **PL***

---

## 1. Do czego służy instrument

**ORSE** symuluje **jednostkę regulacji** procesu (PID + proces termiczny pierwszego
rzędu) i publikuje swój stan w **MQTT Sparkplug B**, jak **węzeł brzegowy**,
który łączy się z **brokerem** i udostępnia metryki systemowi SCADA. Służy do testowania
łańcucha akwizycji Sparkplug B (Ignition, Chariot, EMQX, Node-RED…) bez
rzeczywistego sprzętu.

## 2. Wymaganie wstępne: broker MQTT

Ponieważ ORSE jest **klientem**, potrzebny jest osiągalny broker MQTT. Lokalnie:

```bash
docker run -it --rm -p 1883:1883 eclipse-mosquitto
```

## 3. Pierwsze kroki

```bash
cargo run -p mock_bin_ru_sparkplugb        # GUI + węzeł brzegowy Sparkplug B
```

Przy uruchomieniu GUI próbuje połączyć się z brokerem (`localhost:1883` domyślnie).
Nagłówek wskazuje stan: **Połączono** (zielony) po opublikowaniu `NBIRTH`, lub
**Rozłączono** (czerwony) wraz z przyczyną. Pomarańczowy baner **⚠ MQTT w postaci jawnej** przypomina
o braku TLS.

## 4. Interfejs

- **Nagłówek**: tytuł, przyciski *Ustawienia* / *Zapisz*, stan praca/zatrzymanie, stan
  połączenia Sparkplug B, baner TLS/jawny.
- **Panel lewy (Polecenia)**: *Praca/Zatrzymanie*, *Tryb automatyczny (PID)*,
  *Wartość zadana*, *Wyjście ręczne* (tryb ręczny), nastawy **PID** (Kp/Ki/Kd).
- **Panel centralny**: karty *Pomiar / Wartość zadana / Wyjście* + **wykres** czasu rzeczywistego.
- **Modal *Ustawienia***: język, sprawdzanie aktualizacji, **Broker MQTT / Sparkplug B**
  (host, port, client_id, group_id, edge_node_id, keepalive, TLS, użytkownik/hasło,
  publikacja przy zmianie/okresowa), **proces** (K, τ, opóźnienie, otoczenie),
  **granice wartości zadanej**. *Zastosuj* ponownie uruchamia połączenie i zapisuje TOML.

## 5. Sterowanie z systemu SCADA

SCADA subskrybuje `spBv1.0/<group_id>/#` i odbiera `NBIRTH`, a następnie `NDATA`. Aby
**sterować** regulatorem, publikuje `NCMD` na
`spBv1.0/<group_id>/NCMD/<edge_node_id>` z metrykami sterowalnymi (`Setpoint`,
`Run`, `Auto`, `ManualOutput`) lub `Node Control/Rebirth = true`, aby wymusić
odrodzenie. Szczegóły: [`reference_sparkplugb.md`](reference_sparkplugb.md).

## 6. FAQ

- **Trwałe „Rozłączono"** → broker nieosiągalny: sprawdź `broker_host`/
  `broker_port`, zaporę oraz to, czy broker działa.
- **SCADA nic nie widzi** → sprawdź `group_id`/`edge_node_id` oraz subskrypcję
  `spBv1.0/<group>/#`; ładunki to **protobuf** (wymagany dekoder Sparkplug).
- **Moje zapisy NCMD są ignorowane** → metryka niesterowalna lub błędny typ (zob.
  tabela metryk). Akceptowane są wyłącznie `Setpoint`/`Run`/`Auto`/`ManualOutput` oraz `Rebirth`.
- **Gdzie jest plik konfiguracyjny?** → `mock_ru_sparkplugb.toml` (katalog bieżący;
  nadpisywalny przez `MOCK_CONFIG`).
