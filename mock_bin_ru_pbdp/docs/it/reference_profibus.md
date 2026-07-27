# Riferimento PROFIBUS DP-V0 — Regolatore simulato (ORPD)

*🌍 [FR](../fr/reference_profibus.md) · [EN](../en/reference_profibus.md) · [DE](../de/reference_profibus.md) · [ES](../es/reference_profibus.md) · **IT** · [PT](../pt/reference_profibus.md) · [NL](../nl/reference_profibus.md) · [PL](../pl/reference_profibus.md)*

> Crate: `mock_bin_ru_pbdp` · Eseguibile: **ru_pbdp** · Protocollo: **PROFIBUS DP-V0** (slave seriale)

Questo documento è il riferimento funzionale del sottoinsieme PROFIBUS
DP-V0 simulato. La **fonte di verità tecnica** resta l'intestazione di
[`src/profibus.rs`](../../src/profibus.rs) (codec + macchina a stati) e di
[`src/map.rs`](../../src/map.rs) (blocchi I/O): ogni discrepanza deve
essere corretta prima nel codice.

---

## ⚠️ 0. Ambito e limiti — leggere prima di ogni uso

`ru_pbdp` implementa un **sottoinsieme didattico** di DP-V0, **senza
alcuna pretesa di conformità binaria stretta** alle tabelle normative
(IEC 61158 / EN 50170) oltre agli elementi più universalmente
documentati:

- **conformi**: delimitatori di trama (`SD1`/`SD2`/`SD3`/`SD4`/`SC`/`ED`),
  FCS (somma modulo 256), numeri SAP dei servizi di parametrizzazione
  (`Slave_Diag` = 61, `Set_Prm` = 62, `Chk_Cfg` = 63).
- **convenzioni proprie di questo simulatore, non un profilo GSD reale
  registrato presso il PNO** (PROFIBUS & PROFINET International):
  codifica esatta dei bit del campo `FC`, disposizione precisa dei byte di
  diagnostica, disposizione dei blocchi di ingresso/uscita (§3),
  l'identificativo `Ident_Number` (§4).
- **nessuna temporizzazione di bus reale**: né una finestra di risposta
  (*slot time*, `Tsdr` min/max), né un token tra master, né un
  arbitraggio multi-master. Solo un ASIC dedicato (SPC3/VPC3) o una
  scheda master hardware (Hilscher/Softing/Siemens CP) possono rispettare
  questi vincoli a livello di bit.

**Conseguenza diretta: questo simulatore non sarà mai riconosciuto da un
vero master PROFIBUS DP** (PLC + scheda master). Serve a comprendere la
struttura del protocollo e a testare uno sviluppo software (codec,
macchina a stati, strumentazione), non a pilotare apparecchiature di campo
— vedere [`manuel_utilisateur.md`](manuel_utilisateur.md).

---

## 1. Trame — delimitatori e FCS

| Delimitatore | Valore | Uso |
|---|:--:|---|
| `SD1` | `0x10` | Richiesta fissa senza dati (6 byte: `SD1 DA SA FC FCS ED`) |
| `SD2` | `0x68` | Trama a lunghezza variabile con dati (`SD2 LE LEr SD2 DA SA FC [dati…] FCS ED`) |
| `SD3` | `0xA2` | Trama a dati fissi, 8 byte (14 byte totali) — **non utilizzata** da questo simulatore (vedere §0), fornita per completezza del codec e dei suoi test |
| `SD4` | `0xDC` | Trama token, 3 byte, senza FCS né ED — fuori ambito per uno slave mono-master simulato, fornita per completezza del codec |
| `SC` | `0xE5` | Riscontro breve, 1 byte |
| `ED` | `0x16` | Delimitatore di fine |

- **`FCS`**: somma modulo 256 dei byte utili della trama (vedere
  `profibus::checksum`). Una trama ricevuta con un FCS errato viene
  respinta (`FrameError::BadChecksum`) senza risposta — il master deve
  ritrasmettere.
- **`DA`/`SA`**: indirizzo destinazione / origine. Bit 7 di `DA` =
  **estensione di indirizzo (DAE)**: presenza di un byte SAP subito dopo
  `DA` nel payload. Assente = scambio dati predefinito (`Data_Exchange`).
  L'indirizzo di stazione occupa i restanti 7 bit (`0`-`125`; `126`/`127`
  riservati dalla norma, non usati qui).
- **Questo simulatore privilegia sistematicamente `SD2`** per tutti gli
  scambi `Data_Exchange`, anche quando `SD3` (8 byte fissi) basterebbe
  in un profilo reale — scelta che semplifica il codec senza perdere
  copertura dei concetti di protocollo (vedere
  [`conception.md`](conception.md) §4).
- **Trama malformata / delimitatore sconosciuto (rumore di linea)**:
  respinta silenziosamente (`log::debug!`), la sessione continua —
  permette di risincronizzare il flusso di byte senza far crashare il
  collegamento.

---

## 2. Sequenziamento — servizi e macchina a stati

Lo slave simulato (`SlaveFsm`, [`profibus.rs`](../../src/profibus.rs))
attraversa quattro stati:

```
PowerOn ──Slave_Diag──► WaitPrm ──Set_Prm (ident OK)──► WaitCfg ──Chk_Cfg (lunghezze OK)──► DataExchange
```

| Stato | Significato | Risposta tipica |
|---|---|---|
| `Power_On` | Subito dopo l'avvio, prima del primo interrogatorio di diagnostica | — |
| `Wait_Prm` | In attesa di un `Set_Prm` valido | `Diag` con `Stat_1 = STAT1_PRM_REQ` |
| `Wait_Cfg` | Parametrizzato, in attesa di un `Chk_Cfg` valido | `Diag` con `Stat_1 = STAT1_CFG_FAULT` |
| `Data_Exchange` | Parametrizzato e configurato: scambio ciclico attivo | blocco di ingresso (§3) |

### `Slave_Diag` (SAP 61)

Richiesta senza dati (o trama `SD1`, sempre interpretata come
`Slave_Diag` per convenzione di questo simulatore — nessuna estensione di
indirizzo possibile su `SD1`, mancando un byte disponibile per portare un
SAP). Risposta `Diag` (6 byte):

| Byte | Simbolo | Contenuto |
|:--:|---|---|
| `0` | `Stat_1` | `0x01` (`STAT1_PRM_REQ`, finché non parametrizzato) o `0x02` (`STAT1_CFG_FAULT`, finché non configurato) o `0x00` (`Data_Exchange`) |
| `1` | `Stat_2` | sempre `0x00` (non simulato) |
| `2` | `Stat_3` | sempre `0x00` (non simulato) |
| `3` | `Master_Add` | `0xFF` (nessun master noto — non tracciato da questo simulatore) |
| `4-5` | `Ident_Number` | identificativo fisso dello slave, big-endian (§4) |

Il primo `Slave_Diag` ricevuto fa passare `Power_On` → `Wait_Prm`; i
successivi non cambiano lo stato (solo una lettura di diagnostica).

### `Set_Prm` (SAP 62)

Richiesta: `SAP(62) Ident_Number(2, BE) WD_Fact_1(1) WD_Fact_2(1)`. Il
watchdog annunciato, se presente, si calcola come
`watchdog_ms = WD_Fact_1 × WD_Fact_2 × 10` (unità 10 ms, convenzione
standard DP); `WD_Fact_1 = 0` **o** `WD_Fact_2 = 0` significa «nessun
watchdog». Risposta: `ShortAck` (`SC`) in tutti i casi.

- Se `Ident_Number` **corrisponde** al profilo fisso dello slave (§4):
  stato → `Wait_Cfg`, ed un eventuale watchdog viene trasmesso alla
  sessione (armato solo se l'impostazione locale `watchdog_enabled` lo
  consente — vedere [`manuel_utilisateur.md`](manuel_utilisateur.md) §4).
- Se l'identificativo **non corrisponde**: la parametrizzazione viene
  respinta silenziosamente (`ShortAck` restituito comunque, come
  prescritto da DP-V0 per questo servizio, ma senza effetto sullo stato
  interno) — lo slave resta in `Wait_Prm`.

### `Chk_Cfg` (SAP 63)

Richiesta: `SAP(63) Out_Len(1) In_Len(1)`. Risposta: `ShortAck`. Lo stato
passa a `Data_Exchange` **solo se** `Out_Len == 45` e `In_Len == 17`
(dimensioni fisse del profilo simulato, §3) **e** lo slave era in
`Wait_Cfg`; altrimenti lo stato non cambia (il master deve ritrasmettere
un `Chk_Cfg` corretto).

### `Data_Exchange` (nessun SAP — indirizzo predefinito, bit DAE assente)

Richiesta: il blocco di uscita grezzo (45 byte, §3). Risposta: il blocco
di ingresso (17 byte, §3), ricalcolato al volo dallo snapshot condiviso
al momento della risposta (nessuna tabella di memoria persistente, a
differenza di Modbus/ORME).

Se il master invia un `Data_Exchange` **prima** di raggiungere lo stato
`Data_Exchange` (sequenziamento non rispettato), lo slave risponde con la
diagnostica corrente (`Diag`) anziché crashare o ignorare la trama.

---

## 3. Blocchi I/O — disposizione dei byte

Copiato dall'intestazione di [`map.rs`](../../src/map.rs), unica fonte di
verità in caso di discrepanza. Tutti i valori in virgola mobile (`f32`)
occupano **4 byte consecutivi, big-endian**.

### Blocco di uscita — *Output* (master → slave, `OUTPUT_LEN` = 45 byte)

| Byte | Simbolo | Tipo | Descrizione |
|---|---|:--:|---|
| `0` | `OUT_MODE` | bit | bit0 = marcia, bit1 = auto, [3:2] = modo direzione 1, [5:4] = modo direzione 2 |
| `1-4` | `OUT_SP_AUTO` | f32 | Setpoint automatico |
| `5-8` | `OUT_SP_MANUAL` | f32 | Setpoint manuale (% uscita, con segno) |
| `9-12` | `OUT_KP1` | f32 | Guadagno proporzionale Kp direzione 1 |
| `13-16` | `OUT_KI1` | f32 | Guadagno integrale Ki direzione 1 |
| `17-20` | `OUT_KD1` | f32 | Guadagno derivativo Kd direzione 1 |
| `21-24` | `OUT_KP2` | f32 | Guadagno proporzionale Kp direzione 2 |
| `25-28` | `OUT_KI2` | f32 | Guadagno integrale Ki direzione 2 |
| `29-32` | `OUT_KD2` | f32 | Guadagno derivativo Kd direzione 2 |
| `33-36` | `OUT_HYSTERESIS` | f32 | Isteresi dei regolatori tutto-o-niente |
| `37-40` | `OUT_TOR_MIN_CYCLE` | f32 | Tempo di ciclo minimo tutto-o-niente (s) |
| `41-44` | `OUT_PWM_PERIOD` | f32 | Periodo del ciclo di modulazione PWM (s) |

I codici di modo (`[3:2]`/`[5:4]`) seguono `ControllerKind`: `0` = Off,
`1` = PID, `2` = Tutto-o-niente, `3` = PWM (vedere `mock_lib_control`).

### Blocco di ingresso — *Input* (slave → master, `INPUT_LEN` = 17 byte)

| Byte | Simbolo | Tipo | Descrizione |
|---|---|:--:|---|
| `0` | `IN_STATUS` | bit | bit0 = in marcia, bit1 = direzione 1 attiva (uscita > 0), bit2 = direzione 2 attiva (uscita < 0) |
| `1-4` | `IN_PV` | f32 | Misura / *process value* |
| `5-8` | `IN_OUTPUT` | f32 | Uscita applicata (% con segno) |
| `9-12` | `IN_SP_AUTO` | f32 | Riflesso (sola lettura) del setpoint automatico |
| `13-16` | `IN_SP_MANUAL` | f32 | Riflesso (sola lettura) del setpoint manuale |

Un blocco di uscita **troppo corto** (< 45 byte) viene ignorato senza
crashare: non viene prodotto alcun `Command`, il regolatore mantiene
l'ultimo stato valido.

---

## 4. Profilo fisso dello slave

| Parametro | Valore | Nota |
|---|---|---|
| `Ident_Number` | `0xEE01` | **Fittizio**, non registrato presso il PNO — non rappresenta alcun dispositivo di catalogo reale |
| `Out_Len` | `45` | Atteso in `Chk_Cfg.out_len` |
| `In_Len` | `17` | Atteso in `Chk_Cfg.in_len` |
| Indirizzo di stazione | `0`-`125`, configurabile | Impostazione locale (modale *Impostazioni*), vedere [`manuel_utilisateur.md`](manuel_utilisateur.md) §4 |
| Formato di trama seriale | `8E1` (8 bit, parità pari, 1 bit di stop) | **Fissato dalla norma PROFIBUS DP**, non regolabile |
| Velocità normalizzate | da `9600` a `12.000.000` bit/s | Non verificato all'apertura: un valore non standard viene trasmesso tale e quale alla porta seriale |

---

## 5. Watchdog di protocollo

A differenza del watchdog NAMUR di OSNE (aggiunta artigianale), questo è
una **parte reale del protocollo DP**: è **annunciato dal master** in
`Set_Prm` (fattori `WD_Fact_1`/`WD_Fact_2`, §2) ed è **armato lato slave**
solo se l'impostazione locale `watchdog_enabled` lo consente (altrimenti
la richiesta del master viene ignorata, mai armata). Alla scadenza, senza
aver ricevuto una nuova trama per la stazione, lo slave forza lo stato
sicuro (`Command::SetOnOff(false)`) — semplificazione documentata: un
vero profilo DP-V0 potrebbe richiedere un ritorno completo tramite
`Set_Prm`/`Chk_Cfg` prima di riprendere lo scambio, cosa che questo
simulatore non richiede esplicitamente (basta riprendere l'invio di
trame `Data_Exchange`, poiché lo stato `Data_Exchange` non viene lasciato
alla scadenza del watchdog).

---

## 6. Non interoperabilità — perché

| Requisito del vero PROFIBUS DP | Questo simulatore |
|---|---|
| Finestra di risposta a livello di bit (*slot time*, `Tsdr` min/max) | Assente — risponde non appena la trama è decodificata, senza vincolo di tempo |
| Circuito dedicato (ASIC SPC3/VPC3) per la temporizzazione | Assente — software Tokio ordinario |
| Token tra master, arbitraggio multi-master | Assente — slave mono-master, collegamento punto-punto |
| Profilo GSD registrato presso il PNO | Assente — profilo I/O proprio di questo simulatore (§3) |
| Codifica bit-esatta dei campi FC/diagnostica | Convenzione di simulazione, non garantita conforme |

**Un vero PLC (ad esempio un Siemens S7 con scheda master) non
riconoscerà mai questo simulatore come slave valido su un vero bus
PROFIBUS DP RS-485.** Due istanze di questo simulatore (o uno script che
riproduce la sequenza seguente), invece, possono dialogare tra loro per
illustrare il protocollo — vedere
[`manuel_utilisateur.md`](manuel_utilisateur.md) §5.

---

## 7. Esempio di sequenza (esadecimale)

Sequenza completa per stazione `5`, master `3`, fino allo scambio ciclico
(valori illustrativi, `FCS` calcolato sui byte utili):

```text
# 1. Slave_Diag (SD2, DAE=1, SAP=61)
→ TX  68 03 03 68 85 03 C0 3D FC 16
← RX  68 06 06 68 03 85 00 01 00 00 FF EE 01 F5 16   (Diag: Stat_1=0x01, Ident=0xEE01)

# 2. Set_Prm (SD2, DAE=1, SAP=62, Ident=0xEE01, WD=1×30×10ms=300ms)
→ TX  68 07 07 68 85 03 C0 3E EE 01 01 1E … 16
← RX  E5                                              (ShortAck)

# 3. Chk_Cfg (SD2, DAE=1, SAP=63, out_len=45, in_len=17)
→ TX  68 05 05 68 85 03 C0 3F 2D 11 … 16
← RX  E5                                              (ShortAck)

# 4. Data_Exchange (SD2, nessun SAP, blocco di uscita di 45 byte)
→ TX  68 30 30 68 05 03 C0 [45 byte] … 16
← RX  68 14 14 68 03 85 00 [17 byte]  … 16          (blocco di ingresso)
```

I byte esatti di FCS/lunghezza dipendono dai valori del payload; questo
schema illustra l'**ordine dei servizi**, non una trama da riprodurre
letteralmente. Vedere i test in [`profibus.rs`](../../src/profibus.rs) e
[`profibus_server.rs`](../../src/profibus_server.rs) per sequenze
verificate bit per bit.
