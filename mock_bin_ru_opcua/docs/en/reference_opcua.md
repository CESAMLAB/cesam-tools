# OPC UA reference — address space (RU/OPC UA)

*🌍 [FR](../fr/reference_opcua.md) · **EN** · [DE](../de/reference_opcua.md) · [ES](../es/reference_opcua.md) · [IT](../it/reference_opcua.md) · [PT](../pt/reference_opcua.md) · [NL](../nl/reference_opcua.md) · [PL](../pl/reference_opcua.md)*

> Source of truth: [`opcua_server.rs`](../../src/opcua_server.rs) (node
> declaration + callbacks). Any change to the table is made **in this file** and
> mirrored here.

---

## 1. Endpoint & security

The URL is `opc.tcp://<bind_ip>:<port>/` (default `opc.tcp://0.0.0.0:4840/`),
binary OPC UA TCP transport. **Security** is configurable (the `[security]`
section of the TOML / *Settings* modal) and determines the exposed endpoint:

| Mode | `encryption` | Policy | Security mode | Tokens |
|---|:--:|---|---|---|
| **Unencrypted** (default) | `false` | `None` | `None` | `Anonymous` |
| **Encrypted** | `true` | `Basic256Sha256` | `SignAndEncrypt` | `Anonymous` (if `allow_anonymous`) and/or username/password |

- **Unencrypted**: neither authentication nor encryption. Expose only on a
  **trusted network**. Instant startup (no certificate).
- **Encrypted**: a **self-signed instance certificate** is generated on first
  launch (in `pki/`). The server trusts client certificates
  (`trust_client_certs`, convenient for a simulator). **Username/password**
  authentication if `username` is set; otherwise (or in addition) an **anonymous**
  token if `allow_anonymous`. ⚠️ RSA generation can take a few seconds on first
  launch (debug).

Settings (`[security]`): `encryption` (bool), `allow_anonymous` (bool), `username`
(empty = no password auth), `password` (cleartext — **simulator only**).

---

## 2. Namespace

| Index | URI |
|---|---|
| `0` | `http://opcfoundation.org/UA/` (OPC UA core namespace) |
| `ns` | `urn:cesam-lab:ru-opcua` (application namespace) |

The application namespace `ns` index is assigned dynamically at startup; a client
resolves it via `IN GetNamespaceArray` / the *Browse* service. The business nodes
below live there.

---

## 3. Nodes (under the `Objects` folder)

Each node is a `Variable`; its `NodeId` has the form `ns=<ns>;s=<name>`.

| BrowseName | NodeId (`s=`) | Type | Access | Quantity |
|---|---|---|:--:|---|
| `Setpoint` | `Setpoint` | `Double` | R/W | Setpoint (process unit) |
| `ProcessValue` | `ProcessValue` | `Double` | R | Measurement (PV) |
| `Output` | `Output` | `Double` | R | Command output (%) |
| `ManualOutput` | `ManualOutput` | `Double` | R/W | Forced output in manual mode (%) |
| `Run` | `Run` | `Boolean` | R/W | Regulation start / stop |
| `Auto` | `Auto` | `Boolean` | R/W | Automatic mode (PID) vs manual |

- **Reads**: served by a callback that reads the **shared snapshot**; they are
  therefore "live" and **samplable** by subscriptions (*Subscription* /
  *MonitoredItem*).
- **Writes**: routed to the simulation actor. Values are **sanitized** (non-finite
  rejected, setpoint bounded, manual output bounded to `[0, 100]`).

---

## 4. Mapping to the business state

| Node | Effect of a write | Source of a read |
|---|---|---|
| `Setpoint` | `Command::SetSetpoint` (bounded `[sp_min, sp_max]`) | `snapshot.setpoint` |
| `ManualOutput` | `Command::SetManualOutput` (bounded `[0, 100]`) | `snapshot.manual_output` |
| `Run` | `Command::SetRun` | `snapshot.run` |
| `Auto` | `Command::SetAuto` | `snapshot.auto` |
| `ProcessValue` | — (read-only) | `snapshot.pv` |
| `Output` | — (read-only) | `snapshot.output` |

A write of an unexpected type returns `Bad_TypeMismatch`; a write without a value,
`Bad_NothingToDo`. `Float` is accepted in addition to `Double` for the numeric
nodes.

---

## 5. Examples (OPC UA client)

With a generic client (UaExpert, `opcua` CLI, etc.), connect to
`opc.tcp://127.0.0.1:4840/`, security **None**, user **Anonymous**, then:

```text
# Read the measurement and the setpoint
Read  ns=<ns>;s=ProcessValue   → 60.0
Read  ns=<ns>;s=Setpoint       → 60.0

# Start + new setpoint
Write ns=<ns>;s=Run        = true
Write ns=<ns>;s=Setpoint   = 80.0

# Switch to manual and force output to 40 %
Write ns=<ns>;s=Auto         = false
Write ns=<ns>;s=ManualOutput = 40.0
```

Subscribing (*Subscribe* / *MonitoredItem*) to `ProcessValue` and `Output` lets
you follow the process dynamics in real time.
