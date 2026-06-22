# OPC UA-referentie — adresruimte (RU/OPC UA)

*🌍 [FR](../fr/reference_opcua.md) · [EN](../en/reference_opcua.md) · [DE](../de/reference_opcua.md) · [ES](../es/reference_opcua.md) · [IT](../it/reference_opcua.md) · [PT](../pt/reference_opcua.md) · **NL** · [PL](../pl/reference_opcua.md)*

> Bron van waarheid: [`opcua_server.rs`](../../src/opcua_server.rs) (declaratie van de
> nodes + callbacks). Elke wijziging van de tabel gebeurt **in dit bestand** en wordt
> hier weergegeven.

---

## 1. Endpoint

| Element | Waarde |
|---|---|
| URL | `opc.tcp://<bind_ip>:<port>/` (standaard `opc.tcp://0.0.0.0:4840/`) |
| Transport | OPC UA TCP binair |
| Beveiligingsbeleid | `None` |
| Beveiligingsmodus | `None` |
| Gebruikerstoken | `Anonymous` |

⚠️ **Beveiliging None**: noch authenticatie noch versleuteling (Fase 1b). Alleen
bloot te stellen op een **vertrouwd netwerk**. Echte beveiliging (`Basic256Sha256`,
certificaten, auth) voorzien in **Fase 2**.

---

## 2. Namespace

| Index | URI |
|---|---|
| `0` | `http://opcfoundation.org/UA/` (OPC UA-kernnamespace) |
| `ns` | `urn:cesam-lab:ru-opcua` (toepassingsnamespace) |

De index `ns` van de toepassingsnamespace wordt dynamisch toegekend bij het
opstarten; een client lost deze op via `IN GetNamespaceArray` / de *Browse*-service.
De onderstaande bedrijfsnodes leven daarin.

---

## 3. Nodes (onder de map `Objects`)

Elke node is een `Variable`; zijn `NodeId` heeft de vorm `ns=<ns>;s=<naam>`.

| BrowseName | NodeId (`s=`) | Type | Toegang | Grootheid |
|---|---|---|:--:|---|
| `Setpoint` | `Setpoint` | `Double` | R/W | Setpoint (proceseenheid) |
| `ProcessValue` | `ProcessValue` | `Double` | R | Meting (PV) |
| `Output` | `Output` | `Double` | R | Stuuruitgang (%) |
| `ManualOutput` | `ManualOutput` | `Double` | R/W | Opgelegde uitgang in handmatige modus (%) |
| `Run` | `Run` | `Boolean` | R/W | Aan / uit van de regeling |
| `Auto` | `Auto` | `Boolean` | R/W | Automatische modus (PID) vs handmatig |

- **Lezingen**: bediend door een callback die de **gedeelde momentopname** leest; ze
  zijn dus "levend" en **bemonsterbaar** door de abonnementen (*Subscription*
  / *MonitoredItem*).
- **Schrijfbewerkingen**: gerouteerd naar de simulatie-actor. De waarden worden
  **gesaneerd** (niet-eindige verworpen, setpoint begrensd, handmatige uitgang
  begrensd tot `[0, 100]`).

---

## 4. Mapping naar de bedrijfstoestand

| Node | Effect van een schrijfbewerking | Bron van een lezing |
|---|---|---|
| `Setpoint` | `Command::SetSetpoint` (begrensd `[sp_min, sp_max]`) | `snapshot.setpoint` |
| `ManualOutput` | `Command::SetManualOutput` (begrensd `[0, 100]`) | `snapshot.manual_output` |
| `Run` | `Command::SetRun` | `snapshot.run` |
| `Auto` | `Command::SetAuto` | `snapshot.auto` |
| `ProcessValue` | — (alleen lezen) | `snapshot.pv` |
| `Output` | — (alleen lezen) | `snapshot.output` |

Een schrijfbewerking van een onverwacht type retourneert `Bad_TypeMismatch`; een
schrijfbewerking zonder waarde, `Bad_NothingToDo`. De `Float` wordt naast de `Double`
geaccepteerd voor de numerieke nodes.

---

## 5. Voorbeelden (OPC UA-client)

Maak met een generieke client (UaExpert, `opcua` CLI, enz.) verbinding met
`opc.tcp://127.0.0.1:4840/`, beveiliging **None**, gebruiker **Anonymous**, en vervolgens:

```text
# Lezing van de meting en het setpoint
Read  ns=<ns>;s=ProcessValue   → 60.0
Read  ns=<ns>;s=Setpoint       → 60.0

# Start + nieuw setpoint
Write ns=<ns>;s=Run        = true
Write ns=<ns>;s=Setpoint   = 80.0

# Omschakelen naar handmatig en uitgang opgelegd op 40 %
Write ns=<ns>;s=Auto         = false
Write ns=<ns>;s=ManualOutput = 40.0
```

Abonneren (*Subscribe* / *MonitoredItem*) op `ProcessValue` en `Output` maakt het
mogelijk de procesdynamiek in realtime te volgen.
