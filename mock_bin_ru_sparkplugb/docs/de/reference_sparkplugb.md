# MQTT-Sparkplug-B-Referenz — Metriken & Lebenszyklus (RU/Sparkplug B)

*🌍 [FR](../fr/reference_sparkplugb.md) · [EN](../en/reference_sparkplugb.md) · **DE** · [ES](../es/reference_sparkplugb.md) · [IT](../it/reference_sparkplugb.md) · [PT](../pt/reference_sparkplugb.md) · [NL](../nl/reference_sparkplugb.md) · [PL](../pl/reference_sparkplugb.md)*

> Quelle der Wahrheit: [`sparkplug_node.rs`](../../src/sparkplug_node.rs) (Topics,
> Metriktabelle, Payloads, NCMD-Abbildung). Jede Weiterentwicklung erfolgt **in dieser
> Datei** und wird hierher übertragen.

---

## 1. Rolle & Verbindung

Das Instrument ist ein **Sparkplug-B-Edge-Node**: Es **lauscht auf keinem Port**, es
**verbindet sich ausgehend** mit einem **externen MQTT-Broker** (mosquitto, EMQX,
HiveMQ…) und veröffentlicht den Zustand des Reglers. Einstellungen im Abschnitt
`[network]` der TOML / im Modal *Parameter*:

| Schlüssel | Standard | Rolle |
|---|---|---|
| `broker_host` | `localhost` | Host des MQTT-Brokers |
| `broker_port` | `1883` | Port (`8883` bei TLS) |
| `client_id` | `ru_spb` | MQTT-Client-Kennung |
| `group_id` | `CESAM` | Sparkplug-Gruppe (`spBv1.0/<group_id>/…`) |
| `edge_node_id` | `RU1` | Edge-Node (`…/<edge_node_id>`) |
| `username` / `password` | *(leer)* | MQTT-Auth (Passwort **im Klartext**, nur Simulator) |
| `tls` | `false` | TLS-Verschlüsselung (rustls) zum Broker |
| `keepalive_secs` | `30` | MQTT-Keepalive |
| `publish_on_change` | `true` | `true`: `NDATA`, sobald sich eine Metrik ändert (Takt = Simulationsschritt, 0,5 s); `false`: periodisch |
| `publish_period_secs` | `5` | periodischer Takt, wenn `publish_on_change = false` |

> ⚠️ **MQTT unverschlüsselt standardmäßig**: Ohne TLS ist der Verkehr weder verschlüsselt
> noch netzseitig authentifiziert. Nur in einem **vertrauenswürdigen Netz** zu verwenden.
> Die IHM zeigt ein Warnbanner an, solange `tls` deaktiviert ist.

---

## 2. Namensraum (Topics)

Namensraum `spBv1.0`. Topics des Nodes:

```
spBv1.0/<group_id>/NBIRTH/<edge_node_id>
spBv1.0/<group_id>/NDATA/<edge_node_id>
spBv1.0/<group_id>/NDEATH/<edge_node_id>
spBv1.0/<group_id>/NCMD/<edge_node_id>
```

Mit den Standardwerten: `spBv1.0/CESAM/NBIRTH/RU1` usw.

---

## 3. Metriktabelle

Alle Datenmetriken leben unter dem **Edge-Node** (kein *Device* in dieser Version).
Sparkplug-Typ (Eclipse Tahu): `Float` (9), `Boolean` (11), `UInt64` (8).

| Metrik | Typ | Lesen/Schreiben | Schnappschuss-Feld (Lesen) | NCMD → Befehl (Schreiben) |
|---|---|:--:|---|---|
| `Setpoint` | Float | R/W | `setpoint` | `SetSetpoint` |
| `ProcessValue` | Float | R | `pv` | — |
| `Output` | Float | R | `output` | — |
| `ManualOutput` | Float | R/W | `manual_output` | `SetManualOutput` |
| `Run` | Boolean | R/W | `run` | `SetRun` |
| `Auto` | Boolean | R/W | `auto` | `SetAuto` |
| `SetpointMin` | Float | R | `sp_min` | *(über IHM/TOML eingestellt)* |
| `SetpointMax` | Float | R | `sp_max` | *(über IHM/TOML eingestellt)* |
| `PID/Kp` | Float | R | `pid.kp` | *(über IHM/TOML eingestellt)* |
| `PID/Ki` | Float | R | `pid.ki` | *(über IHM/TOML eingestellt)* |
| `PID/Kd` | Float | R | `pid.kd` | *(über IHM/TOML eingestellt)* |
| `bdSeq` | UInt64 | R | *(Sitzungszähler)* | — |
| `Node Control/Rebirth` | Boolean | W | — | veröffentlicht erneut ein `NBIRTH` |

**Über `NCMD` steuerbare Oberfläche**: `Setpoint`, `ManualOutput`, `Run`, `Auto`, plus
`Node Control/Rebirth` (Parität mit den OPC-UA-Schreibvorgängen des Instruments ORUE).
Die Sollwertgrenzen und die PID-Verstärkungen werden **veröffentlicht** (von einem SCADA
beobachtbar), aber über die IHM/TOML eingestellt. Eine unbekannte Metrik oder eine mit
**falschem Typ** in einem `NCMD` wird **ignoriert** (nie ein Fehler, nie ein abwegiger
Wert: die Simulation bereinigt jeden Schreibvorgang).

---

## 4. Lebenszyklus

- **`NBIRTH`** — veröffentlicht bei jeder Verbindung (ConnAck). Enthält **alle** Metriken
  (mit Werten), `bdSeq` und `Node Control/Rebirth`. `seq = 0`.
- **`NDATA`** — nur **geänderte** Metriken, `seq` umlaufend **0–255**.
- **`NDEATH`** — enthält `bdSeq` **allein**, **ohne** `seq`. Bei der Verbindung als
  **MQTT Last Will** hinterlegt: Der **Broker** veröffentlicht ihn automatisch beim
  Verbindungsverlust (Stopp, Rekonfiguration, Ausfall). Kein explizites `NDEATH`
  nodeseitig.
- **`NCMD`** — Abonnement `spBv1.0/<group>/NCMD/<node>` (QoS 1), das unmittelbar nach dem
  `NBIRTH` abonniert wird. Dekodiert → Befehle, die auf die Simulation angewendet werden.
- **`bdSeq`** — bei jedem (Neu-)Start des Clients inkrementiert; das `NDEATH` (Last Will)
  und das `NBIRTH` einer **gleichen Sitzung** tragen den **gleichen** Wert
  (Sparkplug-Invariante). In der IHM angezeigt (Diagnose).
- **`seq`** — bei jedem `NBIRTH` auf 0 zurückgesetzt, bei jedem `NDATA` (umlaufend)
  inkrementiert.
- **Wiedergeburt** (`Node Control/Rebirth = true` über `NCMD`) → erneute Veröffentlichung
  eines `NBIRTH` (SCADA-Resynchronisation).

---

## 5. Client-Beispiel (SCADA)

Abonnement der gesamten Gruppe, dann Senden eines Sollwerts:

```bash
# Die Nachrichten des Nodes beobachten
mosquitto_sub -h localhost -t 'spBv1.0/CESAM/#' -v

# (die Payloads sind protobuf Sparkplug B — einen Tahu-Decoder zum Lesen verwenden)
```

Ein `NCMD`, das auf `spBv1.0/CESAM/NCMD/RU1` mit den Metriken `Run=true` und
`Setpoint=80.0` veröffentlicht wird, startet die Regelung und legt den Sollwert fest;
ein nachfolgendes `NDATA` spiegelt die Änderung wider.
