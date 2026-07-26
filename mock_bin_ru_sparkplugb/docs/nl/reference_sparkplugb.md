# MQTT Sparkplug B-referentie — metrics & levenscyclus (RU/Sparkplug B)

*🌍 [FR](../fr/reference_sparkplugb.md) · [EN](../en/reference_sparkplugb.md) · [DE](../de/reference_sparkplugb.md) · [ES](../es/reference_sparkplugb.md) · [IT](../it/reference_sparkplugb.md) · [PT](../pt/reference_sparkplugb.md) · **NL** · [PL](../pl/reference_sparkplugb.md)*

> Bron van waarheid: [`sparkplug_node.rs`](../../src/sparkplug_node.rs) (topics, metric-
> tabel, payloads, NCMD-mapping). Elke evolutie gebeurt **in dat bestand**
> en wordt hier weerspiegeld.

---

## 1. Rol & verbinding

Het instrument is een **Sparkplug B edge node**: het **luistert naar geen enkele poort**, het
**maakt uitgaande verbinding** met een **externe MQTT-broker** (mosquitto, EMQX, HiveMQ…) en
publiceert de toestand van de regelaar. Instellingen in de sectie `[network]` van de TOML / het
modale venster *Parameters*:

| Sleutel | Standaard | Rol |
|---|---|---|
| `broker_host` | `localhost` | host van de MQTT-broker |
| `broker_port` | `1883` | poort (`8883` in TLS) |
| `client_id` | `ru_spb` | MQTT-clientidentificatie |
| `group_id` | `CESAM` | Sparkplug-groep (`spBv1.0/<group_id>/…`) |
| `edge_node_id` | `RU1` | edge node (`…/<edge_node_id>`) |
| `username` / `password` | *(leeg)* | MQTT-auth (wachtwoord **in leesbare tekst**, alleen simulator) |
| `tls` | `false` | TLS-versleuteling (rustls) naar de broker |
| `keepalive_secs` | `30` | MQTT-keepalive |
| `publish_on_change` | `true` | `true`: `NDATA` zodra een metric verandert (cadans = simulatiestap, 0,5 s); `false`: periodiek |
| `publish_period_secs` | `5` | periodieke cadans wanneer `publish_on_change = false` |

> ⚠️ **MQTT onversleuteld standaard**: zonder TLS is het verkeer niet versleuteld noch
> netwerkgeauthenticeerd. Alleen te gebruiken op een **vertrouwd netwerk**. De GUI toont
> een waarschuwingsbanner zolang `tls` uitgeschakeld is.

---

## 2. Naamruimte (topics)

Namespace `spBv1.0`. Topics van de node:

```
spBv1.0/<group_id>/NBIRTH/<edge_node_id>
spBv1.0/<group_id>/NDATA/<edge_node_id>
spBv1.0/<group_id>/NDEATH/<edge_node_id>
spBv1.0/<group_id>/NCMD/<edge_node_id>
```

Met de standaardwaarden: `spBv1.0/CESAM/NBIRTH/RU1`, enz.

---

## 3. Metrictabel

Alle data-metrics leven onder de **edge node** (geen *device* in deze versie).
Sparkplug-type (Eclipse Tahu): `Float` (9), `Boolean` (11), `UInt64` (8).

| Metric | Type | Lezen/Schrijven | Momentopnameveld (lezen) | NCMD → commando (schrijven) |
|---|---|:--:|---|---|
| `Setpoint` | Float | R/W | `setpoint` | `SetSetpoint` |
| `ProcessValue` | Float | R | `pv` | — |
| `Output` | Float | R | `output` | — |
| `ManualOutput` | Float | R/W | `manual_output` | `SetManualOutput` |
| `Run` | Boolean | R/W | `run` | `SetRun` |
| `Auto` | Boolean | R/W | `auto` | `SetAuto` |
| `SetpointMin` | Float | R | `sp_min` | *(ingesteld via GUI/TOML)* |
| `SetpointMax` | Float | R | `sp_max` | *(ingesteld via GUI/TOML)* |
| `PID/Kp` | Float | R | `pid.kp` | *(ingesteld via GUI/TOML)* |
| `PID/Ki` | Float | R | `pid.ki` | *(ingesteld via GUI/TOML)* |
| `PID/Kd` | Float | R | `pid.kd` | *(ingesteld via GUI/TOML)* |
| `bdSeq` | UInt64 | R | *(sessieteller)* | — |
| `Node Control/Rebirth` | Boolean | W | — | herpubliceert een `NBIRTH` |

**Aanstuurbaar oppervlak via `NCMD`**: `Setpoint`, `ManualOutput`, `Run`, `Auto`, plus
`Node Control/Rebirth` (pariteit met de OPC UA-schrijfbewerkingen van het instrument ORUE). De
setpointgrenzen en de PID-gains worden **gepubliceerd** (waarneembaar door een SCADA) maar
worden ingesteld via de GUI/TOML. Een onbekende metric of een metric van **verkeerd type** in een
`NCMD` wordt **genegeerd** (nooit een fout, nooit een afwijkende waarde: de simulatie
saneert elke schrijfbewerking).

---

## 4. Levenscyclus

- **`NBIRTH`** — gepubliceerd bij elke verbinding (ConnAck). Bevat **alle**
  metrics (met waarden), `bdSeq`, en `Node Control/Rebirth`. `seq = 0`.
- **`NDATA`** — alleen **gewijzigde** metrics, `seq` rollend **0–255**.
- **`NDEATH`** — bevat `bdSeq` **alleen**, **zonder** `seq`. Geplaatst als **MQTT Last
  Will** bij de verbinding: de **broker** publiceert het automatisch bij verlies van de
  verbinding (afsluiten, herconfiguratie, storing). Geen expliciete `NDEATH` aan de node-zijde.
- **`NCMD`** — abonnement `spBv1.0/<group>/NCMD/<node>` (QoS 1) ingeschreven net na
  de `NBIRTH`. Gedecodeerd → commando's toegepast op de simulatie.
- **`bdSeq`** — verhoogd bij elke (her)start van de client; de `NDEATH` (Last Will)
  en de `NBIRTH` van een **zelfde sessie** dragen **dezelfde** waarde (Sparkplug-
  invariant). Weergegeven in de GUI (diagnostiek).
- **`seq`** — teruggezet op 0 bij elke `NBIRTH`, verhoogd (rollend) bij elke `NDATA`.
- **Wedergeboorte** (`Node Control/Rebirth = true` via `NCMD`) → herpublicatie van een
  `NBIRTH` (SCADA-hersynchronisatie).

---

## 5. Clientvoorbeeld (SCADA)

Abonnement op de hele groep, daarna verzending van een setpoint:

```bash
# De berichten van de node observeren
mosquitto_sub -h localhost -t 'spBv1.0/CESAM/#' -v

# (de payloads zijn Sparkplug B-protobuf — gebruik een Tahu-decoder om ze te lezen)
```

Een `NCMD` gepubliceerd op `spBv1.0/CESAM/NCMD/RU1` met de metrics `Run=true` en
`Setpoint=80.0` start de regeling en stelt het setpoint in; een latere `NDATA`
weerspiegelt de wijziging.
