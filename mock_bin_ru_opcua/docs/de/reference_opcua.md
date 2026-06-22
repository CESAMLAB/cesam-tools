# OPC-UA-Referenz — Adressraum (RU/OPC UA)

*🌍 [FR](../fr/reference_opcua.md) · [EN](../en/reference_opcua.md) · **DE** · [ES](../es/reference_opcua.md) · [IT](../it/reference_opcua.md) · [PT](../pt/reference_opcua.md) · [NL](../nl/reference_opcua.md) · [PL](../pl/reference_opcua.md)*

> Quelle der Wahrheit: [`opcua_server.rs`](../../src/opcua_server.rs) (Deklaration
> der Knoten + Callbacks). Jede Änderung der Tabelle erfolgt **in dieser Datei** und
> wird hier nachgezogen.

---

## 1. Endpoint

| Element | Wert |
|---|---|
| URL | `opc.tcp://<bind_ip>:<port>/` (Standard `opc.tcp://0.0.0.0:4840/`) |
| Transport | OPC UA TCP binär |
| Sicherheitsrichtlinie | `None` |
| Sicherheitsmodus | `None` |
| Benutzer-Token | `Anonymous` |

⚠️ **Sicherheit None**: weder Authentifizierung noch Verschlüsselung (Phase 1b). Nur
in einem **vertrauenswürdigen Netzwerk** bereitstellen. Echte Sicherheit
(`Basic256Sha256`, Zertifikate, Auth) ist für **Phase 2** vorgesehen.

---

## 2. Namespace

| Index | URI |
|---|---|
| `0` | `http://opcfoundation.org/UA/` (OPC-UA-Kern-Namespace) |
| `ns` | `urn:cesam-lab:ru-opcua` (Anwendungs-Namespace) |

Der `ns`-Index des Anwendungs-Namespace wird beim Start dynamisch zugewiesen; ein
Client löst ihn über `IN GetNamespaceArray` / den *Browse*-Dienst auf. Die
nachfolgenden Fachknoten leben darin.

---

## 3. Knoten (unter dem Ordner `Objects`)

Jeder Knoten ist eine `Variable`; seine `NodeId` hat die Form `ns=<ns>;s=<name>`.

| BrowseName | NodeId (`s=`) | Typ | Zugriff | Größe |
|---|---|---|:--:|---|
| `Setpoint` | `Setpoint` | `Double` | R/W | Sollwert (Prozesseinheit) |
| `ProcessValue` | `ProcessValue` | `Double` | R | Messwert (PV) |
| `Output` | `Output` | `Double` | R | Stellgrößenausgang (%) |
| `ManualOutput` | `ManualOutput` | `Double` | R/W | Im Manuellmodus vorgegebener Ausgang (%) |
| `Run` | `Run` | `Boolean` | R/W | Start / Stopp der Regelung |
| `Auto` | `Auto` | `Boolean` | R/W | Automatikmodus (PID) vs. manuell |

- **Lesevorgänge**: bedient durch einen Callback, der den **geteilten Schnappschuss**
  liest; sie sind somit „lebendig“ und durch Abonnements (*Subscription* /
  *MonitoredItem*) **abtastbar**.
- **Schreibvorgänge**: zum Simulationsaktor weitergeleitet. Die Werte werden
  **bereinigt** (nicht endliche verworfen, Sollwert begrenzt, manueller Ausgang auf
  `[0, 100]` begrenzt).

---

## 4. Abbildung auf den Fachzustand

| Knoten | Wirkung eines Schreibvorgangs | Quelle eines Lesevorgangs |
|---|---|---|
| `Setpoint` | `Command::SetSetpoint` (begrenzt auf `[sp_min, sp_max]`) | `snapshot.setpoint` |
| `ManualOutput` | `Command::SetManualOutput` (begrenzt auf `[0, 100]`) | `snapshot.manual_output` |
| `Run` | `Command::SetRun` | `snapshot.run` |
| `Auto` | `Command::SetAuto` | `snapshot.auto` |
| `ProcessValue` | — (schreibgeschützt) | `snapshot.pv` |
| `Output` | — (schreibgeschützt) | `snapshot.output` |

Ein Schreibvorgang mit unerwartetem Typ liefert `Bad_TypeMismatch`; ein
Schreibvorgang ohne Wert `Bad_NothingToDo`. `Float` wird zusätzlich zu `Double` für
die numerischen Knoten akzeptiert.

---

## 5. Beispiele (OPC-UA-Client)

Mit einem generischen Client (UaExpert, `opcua` CLI usw.) eine Verbindung zu
`opc.tcp://127.0.0.1:4840/` herstellen, Sicherheit **None**, Benutzer **Anonymous**,
dann:

```text
# Lesen von Messwert und Sollwert
Read  ns=<ns>;s=ProcessValue   → 60.0
Read  ns=<ns>;s=Setpoint       → 60.0

# Start + neuer Sollwert
Write ns=<ns>;s=Run        = true
Write ns=<ns>;s=Setpoint   = 80.0

# Umschalten auf manuell und Ausgang auf 40 % vorgeben
Write ns=<ns>;s=Auto         = false
Write ns=<ns>;s=ManualOutput = 40.0
```

Das Abonnieren (*Subscribe* / *MonitoredItem*) von `ProcessValue` und `Output`
erlaubt es, die Prozessdynamik in Echtzeit zu verfolgen.
