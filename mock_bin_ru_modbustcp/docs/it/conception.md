# Documento di progettazione — Regolatore simulato Modbus TCP

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · **IT** · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

> Prodotto: **ORME** · Crate: `mock_bin_ru_modbustcp` · Workspace: `cesam-tools` · Licenza: MIT

Questo documento descrive l'architettura, le scelte tecniche e i principi di
funzionamento del regolatore industriale simulato. È destinato agli sviluppatori
che mantengono o estendono il progetto.

---

## 1. Obiettivo e ambito

Fornire uno **strumento industriale virtuale**: un regolatore di processo che si
comporta in modo realistico e comunica in **Modbus TCP** (slave), allo scopo di
sviluppare e testare supervisori / PLC / gateway **senza hardware**.

Il simulatore copre:

- un **processo fisico** modellato da una funzione di trasferimento;
- una **regolazione** bidirezionale (caldo / freddo): PID, tutto-o-niente (TOR) o
  relè a ciclo (PWM);
- un'**interfaccia Modbus TCP** che espone lo stato completo;
- un'**IHM** di pilotaggio, visualizzazione e parametrizzazione;
- la **persistenza** dei parametri.

Fuori dall'ambito attuale: Modbus RTU, ridondanza, storicizzazione a lungo
termine, autenticazione forte (è fornita solo una lista bianca di IP).

---

## 2. Visione d'insieme

```
┌──────────────────────────────────────────────────────────────────────┐
│                       Processo (thread principale)                     │
│                                                                        │
│   ┌─────────────────────────┐         legge (Mutex)                    │
│   │   IHM  egui / eframe     │◄──────────────── SharedSnapshot         │
│   │   (gui.rs)               │◄──────────────── SharedStatus           │
│   └───────────┬─────────────┘                                          │
│               │ cast (non bloccante)                                   │
└───────────────┼────────────────────────────────────────────────────────┘
                │
   ┌────────────┼──────────── Runtime Tokio (thread di fondo) ───────────┐
   │            ▼                                                         │
   │   ┌──────────────────┐  refresh  ┌──────────────┐                   │
   │   │ SimulationActor   ├──────────►│ SharedSnapshot│ (IHM)            │
   │   │  (ractor)         ├──────────►│ SharedMap     │ (Modbus)         │
   │   │  possiede il       │           └──────┬───────┘                  │
   │   │  Regulator         │◄── Command ──┐    │ legge                   │
   │   └──────────────────┘              │    ▼                          │
   │          ▲ Command (cast)            │  ┌──────────────────────┐     │
   │          │                           └──┤ RegulatorService      │     │
   │   ┌──────┴───────────┐  gestisce/rebind │ (trait Service)       │     │
   │   │ ModbusServerActor ├─────────────────►  server Modbus TCP    │◄──── client
   │   │  (ractor)         │  filtro IP ──────► (tokio-modbus)        │     │
   │   └──────────────────┘   (SharedAllowlist)└──────────────────────┘     │
   └────────────────────────────────────────────────────────────────────┘
```

Principio guida: **un solo proprietario dello stato di business**. Il `Regulator`
non è mai condiviso; vive in `SimulationActor`. Tutte le scritture
(IHM o Modbus) sono **messaggi** `Command`. Le letture avvengono su **copie**
aggiornate a ogni passo (`SharedSnapshot`, `SharedMap`), eliminando così
i lock sulla logica e le race condition.

---

## 3. Scelte tecniche

| Esigenza | Scelta | Giustificazione |
|--------|-------|---------------|
| Concorrenza | **`ractor`** (attori) su **Tokio** | Isola lo stato mutabile in un attore; mutazioni serializzate tramite messaggi, senza lock applicativo. Preferenza di progetto. |
| Modbus TCP slave | **`tokio-modbus`** (`tcp-server`) | Implementazione async matura; il trait `Service` mappa in modo pulito richiesta→risposta. |
| IHM | **`egui` / `eframe`** + `egui_plot` | Modalità immediata, multipiattaforma, senza stato UI complesso da sincronizzare. |
| Processo | **FOPDT** (1° ordine + ritardo) | Modello standard e sufficiente per un processo termico; pochi parametri, intuitivo. |
| Persistenza | **`serde` + `toml`** | Formato leggibile/modificabile a mano, ideale per parametri di apparecchio. |

### Perché separare logica sincrona e asincrona

`mock_lib_control` e `regulator.rs` sono **puramente sincroni** (nessun IO,
nessun async). Vantaggi: testabili unitariamente in modo deterministico,
riutilizzabili da altri strumenti e ragionevoli da rivedere. L'asincrono
è confinato agli **attori** e al **livello di rete**.

---

## 4. Modello dei dati

### Stato di business (`regulator.rs`)

- `Regulator` — aggregato proprietario: modalità, setpoint, regolatori (`Pid`,
  `OnOff`) e processo (`FirstOrderProcess`). Non `Clone`, non condiviso.
- `RegulatorConfig` — configurazione statica (processo, guadagni, limiti, `dt`).
  **Fonte unica** dei valori predefiniti (la config TOML ne deriva).
- `RegulatorSnapshot` — **copia immutabile** (`Copy`) dello stato osservabile,
  pubblicata a ogni passo. È il contratto di lettura per l'IHM e la tabella Modbus.
- `Command` — enumerazione delle mutazioni possibili (marcia, modalità, setpoint,
  regolazioni, processo, limiti).

### Strutture condivise (`actors/mod.rs`, `config.rs`)

| Tipo | Contenuto | Scritto da | Letto da |
|------|---------|-----------|--------|
| `SharedSnapshot` | `RegulatorSnapshot` tipizzato | SimulationActor | IHM |
| `SharedMap` | `MemoryMap` (immagini delle 4 tabelle Modbus) | SimulationActor | RegulatorService |
| `SharedAllowlist` | `IpFilter` | ModbusServerActor | accettazione connessioni |
| `SharedStatus` | `ServerStatus` (ascolto / errore) | ModbusServerActor | IHM |

Tutti sono `Arc<Mutex<…>>`: sezioni critiche **brevi** (copia / refresh),
mai mantenute durante un calcolo o un'operazione di IO.

---

## 5. Componenti

### 5.1 `mock_lib_control` (libreria)

- `Pid` — PID a tempo discreto, derivata sull'errore, **anti-windup** tramite
  limitazione del termine integrale. API: `step(sp, pv, dt)` o `step_with_error(err, dt)`
  (riutilizzato per il verso freddo).
- `OnOff` — tutto-o-niente con **isteresi simmetrica** (zona morta) **e
  anti-corto-ciclo**: un tempo di ciclo minimo (`min_cycle`, s) impedisce qualsiasi
  commutazione finché il relè non è rimasto abbastanza a lungo nel suo stato,
  modellando la protezione di un attuatore reale. Il relè **memorizza** il suo stato:
  è il chiamante che deve passargli l'errore con segno senza reinizializzarlo al
  cambio di segno (cfr. § 5.2).
- `Pwm` — modulatore di larghezza d'impulso (**relè a ciclo** /
  *time-proportioning*): su un periodo fisso `T_c`, l'uscita tutto-o-niente è
  attiva per la frazione `duty` del ciclo (`duty` **campionato una volta per ciclo**
  per evitare una distorsione a regime). Permette di regolare con precisione un
  organo TOR.
- `FirstOrderProcess` — funzione di trasferimento `K·e^(-L·s)/(1+T·s)`, integrazione
  di Eulero + linea di ritardo. `reconfigure(...)` cambia i parametri senza salto.
- `ControllerKind` — `Off` / `Pid` / `OnOff` / `Pwm`, con codifica Modbus
  (`to_code`/`from_code`).

### 5.2 `regulator.rs`

Orchestrazione della regolazione a ogni passo (`step`):

1. se **fermato** → uscita 0, regolatori reinizializzati;
2. se **manuale** → uscita = setpoint manuale (% con segno);
3. se **auto** → si calcola **separatamente** il contributo del verso caldo (verso 1,
   errore `SP − PV`) e del verso freddo (verso 2, errore `PV − SP`), ciascuno ≥ 0,
   poi `uscita = caldo − freddo`:
   - **PID**: uscita limitata a `[0, 100]` (`out_min = 0`) — il verso inattivo (errore
     negativo) restituisce 0 e il suo integrale si **scarica naturalmente** per
     limitazione. Non lo si azzera **forzatamente**: con la forte ondulazione del PWM,
     azzerarlo a ogni superamento del setpoint introdurrebbe un errore statico;
   - **TOR**: il relè è valutato sull'errore con segno e conserva il suo stato
     all'attraversamento del setpoint, ripristinando una banda di isteresi **simmetrica**
     `[SP − h/2, SP + h/2]` (le bande caldo/freddo restano disgiunte, quindi i
     due relè sono mutuamente esclusivi);
   - **PWM**: un PID calcola il rapporto ciclico, modulato dal relè a ciclo;
     l'uscita fisica è strettamente 0 % o 100 %, ma la sua media segue il PID.
4. l'uscita pilota il processo che produce la nuova misura (PV).

> **Storia**: prima di questa revisione, lo smistamento caldo/freddo si basava sul
> segno dell'errore e **reinizializzava** il relè TOR all'attraversamento del
> setpoint — il che troncava l'isteresi a `[SP − h/2, SP]` (metà banda,
> asimmetrica) e rendeva mediocre la regolazione TOR. Il calcolo per verso separato
> corregge questo difetto.

### 5.3 `actors/simulation.rs`

`SimulationActor` (ractor). `pre_start` arma un `send_interval(dt)` che emette
`Tick`. `handle` tratta `Tick` (avanza la simulazione) e `Command` (applica una
mutazione), poi **pubblica** lo stato in `SharedSnapshot` e `SharedMap`.

### 5.4 `actors/network.rs`

`ModbusServerActor` possiede il server Modbus. `Reconfigure(NetworkConfig)`:
- aggiorna la **lista bianca** condivisa (effetto immediato, senza riavvio);
- se il **trasporto** (TCP/RTU), la **porta / IP** o i **parametri seriali**
  cambiano, **arresta** il task server e lo **riavvia** (`start_tcp` o
  `start_rtu`); pubblica lo stato in `SharedStatus` (successo o errore).

Un **solo trasporto** è attivo alla volta (`Transport::Tcp` o `Rtu`). L'RTU è
dietro la **feature `rtu`**; senza di essa, selezionare RTU pubblica un errore di
stato esplicito.

### 5.5 `modbus_server.rs`

`RegulatorService` implementa `tokio_modbus::server::Service` in modo
**sincrono** (`future::Ready`): letture = ritaglio di `SharedMap`; scritture =
decodifica in `Command` (tramite `map.rs`) poi `cast` verso `SimulationActor`.

**Politica mono-master.** `serve` (TCP) autorizza **un solo master remoto alla
volta**: a ogni nuova connessione (IP autorizzata dalla lista bianca), la
precedente viene chiusa. Meccanismo: il `TcpStream` è avvolto in un
`CancellableStream` che, alla ricezione di un segnale `oneshot`, restituisce **EOF in
lettura** — il ciclo di elaborazione di `tokio-modbus` termina allora e chiude il
socket. `serve_rtu` (feature `rtu`) serve il bus seriale tramite
`rtu::Server::serve_forever`: il bus RS485 *è* l'unico master (niente da espellere).

> ⚠️ L'IHM non percorre questo cammino: invia i suoi `Command` direttamente
> all'attore, non viene quindi mai conteggiata come master.
>
> ⚠️ Il server RTU di `tokio-modbus` 0.17 non trasmette l'indirizzo slave al
> servizio: l'apparecchio risponde quindi qualunque sia l'indirizzo richiesto. Una
> connessione **punto-punto** è raccomandata. `slave_id` è persistito e mostrato, ma non
> utilizzato per filtrare (limitazione a monte).

### 5.6 `map.rs`

**Fonte di verità** del piano di indirizzamento Modbus. Costanti di indirizzo,
`MemoryMap` (immagini delle tabelle), `refresh_from(snapshot)` (stato→registri) e
`*_to_command(s)` (scritture→comandi). Codifica degli `f32` su 2 registri,
big-endian, parola di peso maggiore in testa.

### 5.7 `config.rs`

`AppConfig` (rete / processo / regolazione) ⇄ TOML. `IpFilter` (jolly `*` per
ottetto IPv4). `ServerStatus`. `to_regulator_config()` fa da ponte verso il dominio.

### 5.8 `gui.rs`

IHM a **pagina unica**: intestazione (stati + pulsanti), pannello comandi (sinistra),
supervisione + curva (centro), tabella Modbus live (destra), modale Parametri.
Legge i `Shared*`, invia `Command` tramite `cast` non bloccante.

---

## 6. Scenari (sequenze)

**Lettura Modbus (PV)**: client → `RegulatorService::call(ReadInputRegisters)` →
lettura `SharedMap` → `Response`. Nessuna interazione con l'attore (latenza minima).

**Scrittura Modbus (setpoint)**: client → `call(WriteMultipleRegisters)` →
`map::holdings_to_commands` → `cast(Command::SetSpAuto)` → l'attore applica al
passo successivo → ripubblica `SharedMap`/`SharedSnapshot`.

**Comando IHM**: interazione → `cast(Command)` → idem.

**Riconfigurazione di rete**: modale *Applica* → `cast(Reconfigure)` →
ModbusServerActor riesegue il bind se necessario → `SharedStatus` aggiornato → l'intestazione
dell'IHM riflette lo stato.

**Tick**: timer → `Tick` → `Regulator::step` → pubblicazione.

---

## 7. Teoria della regolazione

**Processo (FOPDT)**: `v[k+1] = v[k] + (dt/T)·(target − v[k])`, con
`target = ambiente + K·u` e `u` ritardata di `L` secondi (linea di ritardo).

**PID**: `u = Kp·e + Ki·∫e + Kd·de/dt`, integrale limitato a `[out_min, out_max]`
(anti-windup). Derivata sull'errore (compromesso semplicità/simmetria caldo-freddo).

**TOR**: attivo se `e > +H/2`, inattivo se `e < −H/2`, altrimenti stato conservato.

**Bidirezionale**: un solo verso agisce alla volta, selezionato dal segno
dell'errore; l'uscita globale è con segno (+ caldo / − freddo).

---

## 8. Decisioni e compromessi

- **Doppia pubblicazione (`Snapshot` + `Map`)** anziché una sola struttura:
  l'IHM manipola tipi di business, Modbus registri grezzi; entrambi
  restano semplici e disaccoppiati, al prezzo di un lieve sovraccarico di copia trascurabile.
- **Letture Modbus senza passare dall'attore**: si legge `SharedMap` direttamente
  per minimizzare la latenza; l'attore resta l'unico **scrittore**, quindi nessuna race.
- **Servizio Modbus sincrono** (`future::Ready`): tutto il lavoro è non bloccante
  (lock breve + cast), inutile incapsulare un future.
- **Rebind al cambio di porta**: un socket non cambia porta; si
  accetta una breve interruzione del servizio alla riconfigurazione.
- **Derivata sull'errore** (e non sulla misura): leggero «colpo di frusta» al
  cambio di setpoint, accettato per mantenere l'algoritmo simmetrico e semplice.

---

## 9. Evoluzioni ipotizzabili

- Modbus RTU / seriale (riutilizzare `RegulatorService`, cambiare il trasporto).
- Rampa di setpoint, auto-tuning PID, guasti simulati (sensore fuori uso, saturazione).
- Storicizzazione / esportazione CSV dell'andamento.
- Passaggio dell'IHM a **schede** se la pagina unica diventa troppo densa.
- Nuovi strumenti: creare `mock_bin_<nome>` e fattorizzare il comune in
  `mock_lib_*` (vedi [maintenance.md](maintenance.md)).
