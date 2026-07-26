# Riferimento MQTT Sparkplug B — metriche e ciclo di vita (RU/Sparkplug B)

*🌍 [FR](../fr/reference_sparkplugb.md) · [EN](../en/reference_sparkplugb.md) · [DE](../de/reference_sparkplugb.md) · [ES](../es/reference_sparkplugb.md) · **IT** · [PT](../pt/reference_sparkplugb.md) · [NL](../nl/reference_sparkplugb.md) · [PL](../pl/reference_sparkplugb.md)*

> Fonte di verità: [`sparkplug_node.rs`](../../src/sparkplug_node.rs) (topic, tabella
> delle metriche, payload, mapping NCMD). Ogni evoluzione si fa **in questo file** e si
> ripercuote qui.

---

## 1. Ruolo e connessione

Lo strumento è un **edge node Sparkplug B**: **non ascolta alcuna porta**, si
**connette in uscita** a un **broker MQTT esterno** (mosquitto, EMQX, HiveMQ…) e
pubblica lo stato del regolatore. Impostazioni nella sezione `[network]` del TOML / nel
modale *Parametri*:

| Chiave | Predefinito | Ruolo |
|---|---|---|
| `broker_host` | `localhost` | host del broker MQTT |
| `broker_port` | `1883` | porta (`8883` in TLS) |
| `client_id` | `ru_spb` | identificatore del client MQTT |
| `group_id` | `CESAM` | gruppo Sparkplug (`spBv1.0/<group_id>/…`) |
| `edge_node_id` | `RU1` | edge node (`…/<edge_node_id>`) |
| `username` / `password` | *(vuoto)* | auth MQTT (password **in chiaro**, solo simulatore) |
| `tls` | `false` | cifratura TLS (rustls) verso il broker |
| `keepalive_secs` | `30` | keepalive MQTT |
| `publish_on_change` | `true` | `true`: `NDATA` non appena una metrica cambia (cadenza = passo di simulazione, 0,5 s); `false`: periodico |
| `publish_period_secs` | `5` | cadenza periodica quando `publish_on_change = false` |

> ⚠️ **MQTT in chiaro per impostazione predefinita**: senza TLS, il traffico non è né
> cifrato né autenticato a livello di rete. Da usare solo su una **rete fidata**. La IHM
> mostra un banner di avviso finché `tls` è disabilitato.

---

## 2. Spazio dei nomi (topic)

Namespace `spBv1.0`. Topic del nodo:

```
spBv1.0/<group_id>/NBIRTH/<edge_node_id>
spBv1.0/<group_id>/NDATA/<edge_node_id>
spBv1.0/<group_id>/NDEATH/<edge_node_id>
spBv1.0/<group_id>/NCMD/<edge_node_id>
```

Con i valori predefiniti: `spBv1.0/CESAM/NBIRTH/RU1`, ecc.

---

## 3. Tabella delle metriche

Tutte le metriche di dati vivono sotto l'**edge node** (nessun *device* in questa
versione). Tipo Sparkplug (Eclipse Tahu): `Float` (9), `Boolean` (11), `UInt64` (8).

| Metrica | Tipo | Lettura/Scrittura | Campo istantanea (lettura) | NCMD → comando (scrittura) |
|---|---|:--:|---|---|
| `Setpoint` | Float | R/W | `setpoint` | `SetSetpoint` |
| `ProcessValue` | Float | R | `pv` | — |
| `Output` | Float | R | `output` | — |
| `ManualOutput` | Float | R/W | `manual_output` | `SetManualOutput` |
| `Run` | Boolean | R/W | `run` | `SetRun` |
| `Auto` | Boolean | R/W | `auto` | `SetAuto` |
| `SetpointMin` | Float | R | `sp_min` | *(regolato via IHM/TOML)* |
| `SetpointMax` | Float | R | `sp_max` | *(regolato via IHM/TOML)* |
| `PID/Kp` | Float | R | `pid.kp` | *(regolato via IHM/TOML)* |
| `PID/Ki` | Float | R | `pid.ki` | *(regolato via IHM/TOML)* |
| `PID/Kd` | Float | R | `pid.kd` | *(regolato via IHM/TOML)* |
| `bdSeq` | UInt64 | R | *(contatore di sessione)* | — |
| `Node Control/Rebirth` | Boolean | W | — | ripubblica un `NBIRTH` |

**Superficie pilotabile via `NCMD`**: `Setpoint`, `ManualOutput`, `Run`, `Auto`, più
`Node Control/Rebirth` (parità con le scritture OPC UA dello strumento ORUE). I limiti
del setpoint e i guadagni PID sono **pubblicati** (osservabili da uno SCADA) ma si
regolano via IHM/TOML. Una metrica sconosciuta o di **tipo errato** in un `NCMD` viene
**ignorata** (mai un errore, mai un valore aberrante: la simulazione sanifica ogni
scrittura).

---

## 4. Ciclo di vita

- **`NBIRTH`** — pubblicato a ogni connessione (ConnAck). Contiene **tutte** le
  metriche (con valori), `bdSeq`, e `Node Control/Rebirth`. `seq = 0`.
- **`NDATA`** — solo le metriche **modificate**, `seq` rotante **0–255**.
- **`NDEATH`** — contiene **solo** `bdSeq`, **senza** `seq`. Depositato come **Last Will
  MQTT** alla connessione: il **broker** lo pubblica automaticamente alla perdita del
  collegamento (arresto, riconfigurazione, guasto). Nessun `NDEATH` esplicito lato nodo.
- **`NCMD`** — sottoscrizione `spBv1.0/<group>/NCMD/<node>` (QoS 1) sottoscritta subito
  dopo il `NBIRTH`. Decodificato → comandi applicati alla simulazione.
- **`bdSeq`** — incrementato a ogni (ri)avvio del client; il `NDEATH` (Last Will) e il
  `NBIRTH` di una **stessa sessione** portano lo **stesso** valore (invariante
  Sparkplug). Mostrato nella IHM (diagnostica).
- **`seq`** — riportato a 0 a ogni `NBIRTH`, incrementato (rotante) a ogni `NDATA`.
- **Rinascita** (`Node Control/Rebirth = true` via `NCMD`) → ripubblicazione di un
  `NBIRTH` (risincronizzazione SCADA).

---

## 5. Esempio client (SCADA)

Sottoscrizione a tutto il gruppo, poi invio di un setpoint:

```bash
# Osservare i messaggi del nodo
mosquitto_sub -h localhost -t 'spBv1.0/CESAM/#' -v

# (i payload sono protobuf Sparkplug B — usare un decodificatore Tahu per leggerli)
```

Un `NCMD` pubblicato su `spBv1.0/CESAM/NCMD/RU1` con le metriche `Run=true` e
`Setpoint=80.0` avvia la regolazione e fissa il setpoint; un `NDATA` successivo riflette
il cambiamento.
