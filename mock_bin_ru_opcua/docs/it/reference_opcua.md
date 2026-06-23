# Riferimento OPC UA — spazio di indirizzamento (RU/OPC UA)

*🌍 [FR](../fr/reference_opcua.md) · [EN](../en/reference_opcua.md) · [DE](../de/reference_opcua.md) · [ES](../es/reference_opcua.md) · **IT** · [PT](../pt/reference_opcua.md) · [NL](../nl/reference_opcua.md) · [PL](../pl/reference_opcua.md)*

> Fonte di verità: [`opcua_server.rs`](../../src/opcua_server.rs) (dichiarazione dei
> nodi + callback). Ogni evoluzione della tabella avviene **in questo file** e si
> ripercuote qui.

---

## 1. Endpoint e sicurezza

L'URL è `opc.tcp://<bind_ip>:<port>/` (default `opc.tcp://0.0.0.0:4840/`), trasporto
OPC UA TCP binario. La **sicurezza** è regolabile (sezione `[security]` del TOML / modale
*Parametri*) e determina l'endpoint esposto:

| Modalità | `encryption` | Politica | Modalità di sicurezza | Token |
|---|:--:|---|---|---|
| **Non cifrato** (default) | `false` | `None` | `None` | `Anonymous` |
| **Cifrato** | `true` | `Basic256Sha256` | `SignAndEncrypt` | `Anonymous` (se `allow_anonymous`) e/o utente/password |

- **Non cifrato**: né autenticazione né cifratura. Da esporre solo su una
  **rete fidata**. Avvio istantaneo (nessun certificato).
- **Cifrato**: un **certificato di istanza** autofirmato viene generato al primo
  avvio (in `pki/`). La fiducia nei certificati client è **regolabile**
  (`trust_client_certs`): **automatica** (default, pratica per un simulatore) o
  **stretta** — il client deve allora essere pre-approvato in `pki/trusted/` (altrimenti
  viene depositato in `pki/rejected/` e rifiutato). Autenticazione tramite
  **utente/password** se `username` è valorizzato; altrimenti (o in aggiunta)
  token **anonimo** se `allow_anonymous`. ⚠️ La generazione RSA può richiedere alcuni
  secondi al primo avvio (debug).

Impostazioni (`[security]`): `encryption` (bool), `allow_anonymous` (bool),
`trust_client_certs` (bool, default `true`), `username` (vuoto = nessuna auth
password), `password` (in chiaro — **solo simulatore**).

---

## 2. Namespace

| Indice | URI |
|---|---|
| `0` | `http://opcfoundation.org/UA/` (namespace core OPC UA) |
| `ns` | `urn:cesam-lab:ru-opcua` (namespace applicativo) |

L'indice `ns` del namespace applicativo è assegnato dinamicamente all'avvio;
un client lo risolve tramite `IN GetNamespaceArray` / il servizio *Browse*. I nodi
di business qui sotto vivono lì.

---

## 3. Nodi (sotto la cartella `Objects`)

Ogni nodo è una `Variable`; il suo `NodeId` è della forma `ns=<ns>;s=<nome>`.

| BrowseName | NodeId (`s=`) | Tipo | Accesso | Grandezza |
|---|---|---|:--:|---|
| `Setpoint` | `Setpoint` | `Double` | R/W | Setpoint (unità di processo) |
| `ProcessValue` | `ProcessValue` | `Double` | R | Misura (PV) |
| `Output` | `Output` | `Double` | R | Uscita di comando (%) |
| `ManualOutput` | `ManualOutput` | `Double` | R/W | Uscita imposta in modalità manuale (%) |
| `Run` | `Run` | `Boolean` | R/W | Marcia / arresto della regolazione |
| `Auto` | `Auto` | `Boolean` | R/W | Modalità automatica (PID) vs manuale |

- **Letture**: servite da un callback che legge l'**istantanea condivisa**; sono
  quindi «vive» e **campionabili** dalle sottoscrizioni (*Subscription*
  / *MonitoredItem*).
- **Scritture**: instradate verso l'attore di simulazione. I valori sono **sanificati**
  (non finiti rifiutati, setpoint limitato, uscita manuale limitata a `[0, 100]`).

---

## 4. Mapping verso lo stato di business

| Nodo | Effetto di una scrittura | Fonte di una lettura |
|---|---|---|
| `Setpoint` | `Command::SetSetpoint` (limitato `[sp_min, sp_max]`) | `snapshot.setpoint` |
| `ManualOutput` | `Command::SetManualOutput` (limitato `[0, 100]`) | `snapshot.manual_output` |
| `Run` | `Command::SetRun` | `snapshot.run` |
| `Auto` | `Command::SetAuto` | `snapshot.auto` |
| `ProcessValue` | — (sola lettura) | `snapshot.pv` |
| `Output` | — (sola lettura) | `snapshot.output` |

Una scrittura di un tipo inatteso restituisce `Bad_TypeMismatch`; una scrittura senza
valore, `Bad_NothingToDo`. Il `Float` è accettato oltre al `Double` per i
nodi numerici.

---

## 5. Esempi (client OPC UA)

Con un client generico (UaExpert, `opcua` CLI, ecc.), connettersi a
`opc.tcp://127.0.0.1:4840/`, sicurezza **None**, utente **Anonymous**, poi:

```text
# Lettura della misura e del setpoint
Read  ns=<ns>;s=ProcessValue   → 60.0
Read  ns=<ns>;s=Setpoint       → 60.0

# Avvio + nuovo setpoint
Write ns=<ns>;s=Run        = true
Write ns=<ns>;s=Setpoint   = 80.0

# Passaggio a manuale e uscita imposta al 40 %
Write ns=<ns>;s=Auto         = false
Write ns=<ns>;s=ManualOutput = 40.0
```

Sottoscrivere (*Subscribe* / *MonitoredItem*) a `ProcessValue` e `Output` permette di
seguire la dinamica del processo in tempo reale.
