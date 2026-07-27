# Progettazione — Regolatore PROFIBUS DP simulato (ORPD)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · **IT** · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

> Crate: `mock_bin_ru_pbdp` · Eseguibile: **ru_pbdp** (*Regulation Unit over PROFIBUS DP*)

Documento di architettura e modellazione. Ricalcato sul regolatore **ORME**
(`mock_bin_ru_modbus`) per il modello di dominio e gli attori, e su
**OSNE** (`mock_bin_su_namur`) per il collegamento seriale. Cambia solo lo
**strato di protocollo**: un **simulatore software di trame PROFIBUS
DP-V0**, sviluppato da zero (nessun crate `profibus`/`profibus-dp`
pubblicato esiste ad oggi nell'ecosistema Rust).

---

## 1. Scopo

Simulare un **regolatore di processo** (anello PID su un processo termico
del primo ordine, modello **identico** a ORME) ed esporlo tramite una
**struttura di trame PROFIBUS DP-V0** su un collegamento seriale
(RS-485/RS-232).

**Questo documento presuppone che il lettore abbia letto l'avvertenza di
non interoperabilità** (vedere [`manuel_utilisateur.md`](manuel_utilisateur.md)
e [`reference_profibus.md`](reference_profibus.md) §6): il vero PROFIBUS DP
richiede il rispetto del temporizzazione del bus a livello di bit (*slot
time*, `Tsdr` min/max, un watchdog nell'ordine delle decine di
millisecondi) che solo un ASIC dedicato (SPC3/VPC3) può garantire. Questo
simulatore non pretende tale conformità — è uno strumento didattico e di
test software, non un driver di bus.

---

## 2. Modello fisico ([`regulator.rs`](../../src/regulator.rs))

Ripreso identico dal regolatore ORME:
[`mock_lib_control::FirstOrderProcess`] (funzione di trasferimento del
primo ordine con ritardo puro) e [`mock_lib_control::Pid`]
(PID anti-windup), con gli stessi modi (Off/PID/Tutto-o-niente/PWM) in
entrambe le direzioni (caldo/freddo). Passo di simulazione: **50 ms**. Tutte
le scritture sono **sanificate** in `Regulator::apply` (limiti riordinati,
valori in virgola mobile non finiti ignorati, guadagni PID limitati) —
stessa invariante di tutto il resto del workspace: mai chiamare
`f32::clamp` con limiti non validati.

---

## 3. Architettura (attori)

```
GUI (egui) ──Command(cast)──►  SimulationActor  ──refresh──► SharedSnapshot ──► GUI
Master PROFIBUS (simulato) ──►  (Regulator)      ──refresh──► SharedSnapshot ──► risposte Data_Exchange
```

- **`SimulationActor`** ([`actors/simulation.rs`](../../src/actors/simulation.rs)):
  identico nella forma a quelli di ORME/OSNE — unico proprietario del
  `Regulator`, timer one-shot riarmato, pubblica lo `SharedSnapshot` ad
  ogni passo.
- **`ProfibusServerActor`** ([`actors/network.rs`](../../src/actors/network.rs)):
  possiede il collegamento seriale; `Reconfigure` richiude/riapre il
  trasporto se cambiano porta/baud/indirizzo di stazione; conserva il
  `JoinHandle` della sessione (abortito all'arresto); pubblica lo stato del
  collegamento (`ServerStatus`, incluso lo stato corrente della macchina a
  stati DP-V0) per la GUI.
- **[`profibus.rs`](../../src/profibus.rs)** — **fonte di verità** del
  protocollo: codec delle trame (`SD1`/`SD2`/`SD3`/`SD4`/`SC`, FCS),
  decodifica dei servizi
  (`Slave_Diag`/`Set_Prm`/`Chk_Cfg`/`Data_Exchange`) e macchina a stati
  dello slave `SlaveFsm` (`PowerOn → WaitPrm → WaitCfg → DataExchange`).
- **[`map.rs`](../../src/map.rs)** — conversione dei blocchi di byte I/O
  `Data_Exchange` verso/da i `Command` del regolatore (vedere
  [`reference_profibus.md`](reference_profibus.md) §3).
- **[`profibus_server.rs`](../../src/profibus_server.rs)** — ciclo di
  sessione su un flusso qualsiasi `AsyncRead + AsyncWrite` (la porta
  seriale in produzione, un `tokio::io::duplex` nei test): legge una
  trama, la decodifica, chiama `SlaveFsm::handle`, applica i `Command`
  risultanti, codifica la risposta e la rinvia. Gestisce anche il
  **watchdog di protocollo** (`tokio::select!` tra lettura della trama e
  un ritardo, come il watchdog NAMUR di OSNE — ma qui è una **parte reale
  del protocollo DP**, armata da `Set_Prm`, non un'aggiunta artigianale).

A differenza di Modbus (ORME, tabella di memoria separata rigenerata ad
ogni tick) e come per OPC UA/NAMUR, **non esiste una tabella di memoria
persistente**: il blocco di ingresso `Data_Exchange` viene ricalcolato al
volo dallo `SharedSnapshot` al momento della risposta.

**Nessuna politica multi-master da gestire**: il collegamento seriale *è*
l'unico master (come il Modbus RTU o la porta seriale NAMUR), a differenza
del Modbus TCP di ORME (espulsione) o persino del NAMUR TCP di OSNE
(punto-punto senza espulsione).

---

## 4. Codec PROFIBUS DP-V0 — scelte e limiti accettati

- **Delimitatori di trama** (`SD1=0x10`, `SD2=0x68`, `SD3=0xA2`,
  `SD4=0xDC`, `SC=0xE5`, `ED=0x16`) e **FCS** (somma modulo 256): conformi
  alla norma, ben documentati pubblicamente.
- **Numeri SAP dei servizi di parametrizzazione** (`Slave_Diag=61`,
  `Set_Prm=62`, `Chk_Cfg=63`): conformi.
- **Codifica esatta dei bit del campo FC**, **disposizione precisa dei
  byte di diagnostica**, e **disposizione dei blocchi di ingresso/uscita**
  (`map.rs`): sono **convenzioni proprie di questo simulatore**, non un
  profilo GSD reale registrato presso il PNO. Il simulatore utilizza
  sistematicamente trame **SD2** (lunghezza variabile) per tutti gli
  scambi `Data_Exchange`, anche quando `SD3` (8 byte fissi)
  basterebbe in un profilo reale — scelta che semplifica il codec senza
  perdere copertura dei concetti del protocollo.
- **Identificativo PROFIBUS** (`Ident_Number = 0xEE01`): **fittizio**, non
  registrato presso il PNO (PROFIBUS & PROFINET International) — non
  rappresenta alcun dispositivo di catalogo reale.
- **Nessuna temporizzazione di bus**: né una finestra di risposta
  (`Tsdr`), né un token, né un arbitraggio multi-master sono implementati
  — vedere §1.

Dettaglio completo in [`reference_profibus.md`](reference_profibus.md).

---

## 5. Configurazione e persistenza

`AppConfig` (lingua / collegamento seriale / processo / regolazione /
verifica aggiornamenti) serializzato in **TOML**
([`config.rs`](../../src/config.rs)), **sanificato al caricamento**
(`AppConfig::sanitized`: limiti ordinati, `τ ≥ 1e-3`, `dead_time ≥ 0`,
valori in virgola mobile finiti, indirizzo di stazione limitato a
`[0, 125]`). File: `mock_ru_pbdp.toml` (sovrascrivibile tramite
`MOCK_CONFIG`). A differenza di ORME/OSNE, **nessuna lista bianca IP** (il
collegamento seriale è intrinsecamente punto-punto, nessuna nozione di
indirizzo di rete).

---

## 6. Prospettive di evoluzione

- Un vero strumento di **master PROFIBUS DP simulato** (eseguibile
  separato), che utilizzi le stesse funzioni di codifica/decodifica
  esposte per i test in `profibus.rs`, per pilotare questo simulatore o
  qualsiasi altro slave software senza dipendere da uno script ad hoc.
- Generazione di un file **GSD** illustrativo (non funzionale lato
  simulatore) che documenti il profilo I/O simulato, a scopo didattico.
- Supporto di **DP-V1** (accesso aciclico, allarmi) qualora emerga
  l'esigenza didattica — fuori ambito inizialmente (solo DP-V0).
- Promozione del modello del regolatore in una `mock_lib_*` condivisa
  (oggi duplicato tra ORME e questo strumento, come per ORUE).
