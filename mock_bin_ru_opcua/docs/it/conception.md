# Progettazione — Regolatore di processo simulato (RU/OPC UA)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · **IT** · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

> Crate: `mock_bin_ru_opcua` · Eseguibile: **ru_opcua** (*Regulation Unit over OPC UA*)

Documento di architettura e modellazione. Ricalcato sul regolatore **ORME**
(`mock_bin_ru_modbustcp`): stessa suddivisione **modello di business sincrono /
attori ractor / livello di protocollo / IHM egui**, stessi invarianti. Cambia solo
il **trasporto**: **OPC UA** anziché Modbus.

---

## 1. Obiettivo

Simulare un **regolatore di processo** (anello PID su un processo termico del
primo ordine) ed esporlo tramite **OPC UA**, lo standard di supervisione
industriale (Industria 4.0). A differenza di ORME (Modbus) e OSNE (NAMUR) —
protocolli **di campo senza sicurezza** — OPC UA supporta nativamente
l'autenticazione, la firma e la cifratura (previste in Fase 2).

---

## 2. Modello fisico ([`regulator.rs`](../../src/regulator.rs))

Il **processo** riutilizza [`mock_lib_control::FirstOrderProcess`] (condiviso con
ORME): funzione di trasferimento del primo ordine con ritardo puro

```text
PV(s) / U(s) = K · e^(−L·s) / (1 + τ·s)
```

- `PV`: misura (unità di processo, p. es. °C);
- `U`: comando / uscita (0-100 %);
- `K`: guadagno statico; `τ`: costante di tempo; `L`: ritardo puro;
- `ambient`: valore a riposo (uscita nulla).

Un **PID** ([`mock_lib_control::Pid`], anch'esso riutilizzato da ORME) regola la
misura verso il **setpoint** pilotando l'uscita, limitata a `[0, 100]`. Due modalità:
**automatica** (il PID calcola l'uscita) e **manuale** (uscita imposta). Il passo
di simulazione è di **0,5 s** (processo termico lento).

Tutte le scritture (rete o IHM) sono **sanificate** in `Regulator::apply`:
valori in virgola mobile non finiti ignorati, setpoint limitato, limiti riordinati
(`min ≤ max`), guadagni PID limitati. **Invariante: mai `f32::clamp` con limiti non
validati** (panic se `min > max` o `NaN`).

---

## 3. Architettura (attori)

```
IHM (egui) ───Command(cast)──►  SimulationActor ──refresh──► SharedSnapshot ──► IHM
Server OPC UA ─Command(cast)─►   (Regulator)    ──refresh──► SharedSnapshot ──► letture OPC UA
```

- **`SimulationActor`** ([`actors/simulation.rs`](../../src/actors/simulation.rs)):
  proprietario **unico** del `Regulator`; avanza la simulazione su un timer
  one-shot riarmato (nessun timer scollegato) e pubblica un `SharedSnapshot` a ogni
  passo.
- **`OpcuaServerActor`** ([`actors/network.rs`](../../src/actors/network.rs)):
  possiede il server OPC UA (task tokio `server.run()`); riavviabile a caldo
  (`Reconfigure`: rebind se l'IP/porta cambia); conserva il `JoinHandle` (abbandono
  all'arresto) e il `ServerHandle` (annullamento pulito delle sessioni); pubblica il
  suo stato di ascolto per l'IHM.
- **Server OPC UA** ([`opcua_server.rs`](../../src/opcua_server.rs)): costruisce il
  server [`async-opcua`](https://crates.io/crates/async-opcua), dichiara lo spazio
  di indirizzamento e collega i callback. Le **letture** attingono dal
  `SharedSnapshot`; le **scritture** emettono una `Command` verso il
  `SimulationActor` tramite `cast` non bloccante.

Come NAMUR (OSNE) e a differenza del Modbus di ORME, non c'è **nessuna tabella
di memoria separata**: i nodi OPC UA leggono direttamente l'istantanea condivisa.

---

## 4. Stack OPC UA — scelte tecniche

- **`async-opcua`** (server, feature `server`): implementazione **tokio-native**
  (un task per connessione), che si integra nello stack ractor/tokio. Crittografia
  **100 % Rust** (RustCrypto: `rsa`, `aes`, `sha2`, `x509-cert`) — **nessuna
  dipendenza da OpenSSL**, il che preserva la cross-compilazione (Linux/Windows/RPi).
- **Spazio di indirizzamento**: un `SimpleNodeManager` in memoria; nodi `Variable`
  organizzati sotto `Objects` (cfr. [`reference_opcua.md`](reference_opcua.md)).
- **Callback**: `add_read_callback` (valore vivo, campionato per le
  sottoscrizioni) e `add_write_callback` (instrada verso la simulazione).
- **Licenza**: `async-opcua` è sotto **MPL-2.0** (tutta la linea OPC UA in Rust
  lo è). Copyleft **per file**: uso non modificato → il codice CESAM-Lab resta
  MIT (cfr. file `NOTICE` nella radice).

---

## 5. Sicurezza

La sicurezza è **regolabile** (`SecurityConfig`) e costituisce il fattore
differenziante di OPC UA rispetto ai protocolli di campo (Modbus/NAMUR, senza sicurezza).

- **Modalità non cifrata (default)**: un endpoint `SecurityPolicy::None`, token
  **anonimo** — solo rete fidata, avvio istantaneo, nessun
  certificato. L'IHM mostra un **banner arancione** di avviso.
- **Modalità cifrata (Fase 2)**: endpoint `Basic256Sha256` / `SignAndEncrypt`. Un
  **certificato di istanza** autofirmato viene generato al primo avvio (`pki/`);
  il server si fida dei certificati client. **Autenticazione** tramite
  utente/password (`ServerUserToken::user_pass`) e/o anonima. L'IHM
  mostra un **banner verde** 🔒.

La modalità si regola nel modale *Parametri*; una modifica **riavvia** il server
a caldo (`OpcuaServerActor`).

---

## 6. Configurazione e persistenza

`AppConfig` (lingua / rete / processo / regolazione / verif. aggiorn.) serializzata in
**TOML** ([`config.rs`](../../src/config.rs)), **sanificata al caricamento**
(`AppConfig::sanitized`: limiti ordinati, `τ ≥ 1e-3`, `dead_time ≥ 0`, valori in
virgola mobile finiti). File: `mock_ru_opcua.toml` (sovrascrivibile tramite `MOCK_CONFIG`).

---

## 7. Spunti di evoluzione

- **Fase 2**: sicurezza OPC UA (certificati, cifratura, auth).
- Metodi OPC UA (`Reset`, `Autotune`) oltre alle variabili.
- Modello di informazione tipizzato (ObjectType regolatore) anziché variabili piatte.
- Storicizzazione / `HistoryRead` sulla misura.
- Promozione del modello regolatore di ORME in una `mock_lib_*` condivisa (oggi è
  duplicato tra ORME e questo strumento).
