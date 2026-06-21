# Tabella degli indirizzi Modbus — Regolatore simulato

*🌍 [FR](../fr/table_modbus.md) · [EN](../en/table_modbus.md) · [DE](../de/table_modbus.md) · [ES](../es/table_modbus.md) · **IT** · [PT](../pt/table_modbus.md) · [NL](../nl/table_modbus.md) · [PL](../pl/table_modbus.md)*

> Crate: `mock_bin_ru_modbustcp` · Protocollo: **Modbus TCP** (slave / server)

Questo documento è il riferimento funzionale del piano di indirizzamento. La **fonte di
verità tecnica** resta l'intestazione di [`src/map.rs`](../../src/map.rs): ogni
divergenza deve essere corretta nel codice in via prioritaria.

---

## 1. Generalità

| Elemento | Valore |
|---------|--------|
| Trasporto | Modbus **TCP** o **RTU seriale / RS485** (uno solo attivo alla volta) |
| Ruolo | **Slave** (server) |
| Porta predefinita | TCP `5502` (configurabile, modale *Parametri*) |
| Seriale (RTU) | porta + baud + parità + bit, configurabili (feature `rtu`) |
| Unit ID / indirizzo | TCP: indifferente. RTU: `slave_id` configurabile ma **non filtrato** (vedi nota) |
| Master | **un solo master remoto alla volta**; in TCP un nuovo arrivato disconnette il precedente (l'IHM locale non è un master) |
| Indirizzamento | **base 0** (l'indirizzo `0` = 1° elemento della tabella) |
| Filtraggio | lista bianca di IP opzionale (jolly `*`, TCP solamente) |

> **Nota RTU / indirizzo slave**: il server RTU risponde **qualunque sia
> l'indirizzo** richiesto (l'indirizzo non è trasmesso al servizio applicativo).
> Utilizzare una **connessione punto-punto**. Lo `slave_id` è conservato/mostrato ma
> non effettua alcun filtraggio.

### Indirizzamento base 0 vs base 1

Gli indirizzi qui sotto sono gli **indirizzi protocollari (base 0)**, così
come inviati nella trama. Molti strumenti mostrano una numerazione base 1
«convenzionale» (`4xxxx` per gli holding, `3xxxx` per gli input…). Così
il registro di mantenimento all'indirizzo `2` corrisponde al riferimento convenzionale `40003`.

---

## 2. Codifica dei numeri in virgola mobile (`f32`)

Le grandezze analogiche sono **`f32` IEE-754 su 2 registri consecutivi**:

- **ordine delle parole**: parola di **peso maggiore per prima** (big-endian, detto *ABCD*);
- **ordine dei byte** in ogni registro: big-endian (standard Modbus).

Esempio: `80.0` → byte `42 A0 00 00` → registro `n` = `0x42A0`,
registro `n+1` = `0x0000`.

> Se il vostro client legge valori aberranti, è quasi sempre un problema
> di ordine delle parole (provare *word swap* / *CDAB*).

---

## 3. Bobine — *Coils* (lettura/scrittura)

Codici funzione: `0x01` (lettura), `0x05` (scrittura singola), `0x0F` (scrittura multipla).

| Indirizzo | Designazione | Valori | Effetto |
|---------|-------------|---------|-------|
| `0` | Marcia / Arresto | `0` = arresto, `1` = marcia | Attiva la regolazione |
| `1` | Auto / Manuale | `0` = manuale, `1` = auto | Scelta della modalità |

---

## 4. Ingressi discreti — *Discrete Inputs* (sola lettura)

Codice funzione: `0x02`.

| Indirizzo | Designazione | Significato |
|---------|-------------|---------------|
| `0` | In marcia | L'apparecchio è in marcia |
| `1` | Verso 1 (caldo) attivo | Uscita > 0 |
| `2` | Verso 2 (freddo) attivo | Uscita < 0 |

---

## 5. Registri di mantenimento — *Holding Registers* (lettura/scrittura)

Codici funzione: `0x03` (lettura), `0x06` (scrittura singola), `0x10` (scrittura multipla).

| Indirizzo | Designazione | Tipo | Unità / valori |
|---------|-------------|------|-----------------|
| `0` | Modalità di regolazione verso 1 (caldo) | `u16` | `0`=Off, `1`=PID, `2`=TOR, `3`=PWM |
| `1` | Modalità di regolazione verso 2 (freddo) | `u16` | `0`=Off, `1`=PID, `2`=TOR, `3`=PWM |
| `2`–`3` | Setpoint automatico (SP) | `f32` | unità di misura |
| `4`–`5` | Setpoint manuale | `f32` | % di uscita, con segno (−100…+100) |
| `6`–`7` | `Kp` verso 1 | `f32` | guadagno proporzionale |
| `8`–`9` | `Ki` verso 1 | `f32` | guadagno integrale (s⁻¹) |
| `10`–`11` | `Kd` verso 1 | `f32` | guadagno derivativo (s) |
| `12`–`13` | `Kp` verso 2 | `f32` | guadagno proporzionale |
| `14`–`15` | `Ki` verso 2 | `f32` | guadagno integrale (s⁻¹) |
| `16`–`17` | `Kd` verso 2 | `f32` | guadagno derivativo (s) |
| `18`–`19` | Isteresi TOR | `f32` | unità di misura |
| `20`–`21` | Tempo di ciclo minimo TOR | `f32` | secondi (anti-corto-ciclo, `0` = disattivato) |
| `22`–`23` | Periodo del ciclo PWM | `f32` | secondi (> 0) |
| `42`–`46` | Identificativo apparecchio | `ASCII` | «CESAM-Lab» (sola lettura, 2 car./registro, peso maggiore per primo) |

> Registri `24`–`41` riservati (letti a `0`).

> **Scrittura parziale di un `f32`**: occorre scrivere **entrambi i registri** di un
> virgola mobile perché venga preso in considerazione. Una scrittura di un solo registro di una
> coppia `f32` è ignorata (e restituisce l'eccezione *Illegal Data Address* se essa
> non copre alcun campo valido).
>
> I guadagni scritti sono limitati a valori finiti ≥ 0 (robustezza).

---

## 6. Registri di ingresso — *Input Registers* (sola lettura)

Codice funzione: `0x04`.

| Indirizzo | Designazione | Tipo | Unità |
|---------|-------------|------|-------|
| `0`–`1` | Misura (PV — *process value*) | `f32` | unità di misura |
| `2`–`3` | Uscita applicata | `f32` | % con segno (+ caldo / − freddo) |
| `4`–`5` | Rilettura setpoint auto (sola lettura) | `f32` | unità di misura |
| `6`–`7` | Rilettura setpoint manuale (sola lettura) | `f32` | % di uscita, con segno (−100…+100) |

> **Riletture dei setpoint**: i registri `4`–`7` espongono in **sola lettura** il
> valore corrente dei setpoint auto/manuale (specchi degli holding `2`–`5`).
> Comodo per un supervisore che si limita a **monitorare** senza scrivere.

---

## 7. Eccezioni Modbus

| Codice | Nome | Causa in questo apparecchio |
|------|-----|--------------------------|
| `0x01` | Illegal Function | Codice funzione non gestito (es. maschera, FIFO) |
| `0x02` | Illegal Data Address | Indirizzo / quantità fuori tabella, o scrittura che non punta ad alcun campo |
| `0x04` | Server Device Failure | Lock interno non disponibile (caso anomalo) |

---

## 8. Esempi con `mbpoll`

`mbpoll` indirizza in **base 1**; si aggiunge quindi `1` agli indirizzi base 0.

```bash
# Mettere in marcia (bobina base0 0 -> -t 0 -r 1) poi passare in auto (bobina 1)
mbpoll -m tcp -p 5502 -a 1 -t 0 -r 1 127.0.0.1 1     # On/Off = 1
mbpoll -m tcp -p 5502 -a 1 -t 0 -r 2 127.0.0.1 1     # Auto/Manuale = 1 (auto)

# Scrivere il setpoint auto (HR base0 2-3 -> -t 4:float -r 3) a 80.0
mbpoll -m tcp -p 5502 -a 1 -t 4:float -r 3 127.0.0.1 80.0

# Leggere la misura PV (IR base0 0-1 -> -t 3:float -r 1)
mbpoll -m tcp -p 5502 -a 1 -t 3:float -r 1 127.0.0.1

# Leggere l'uscita (IR base0 2-3 -> -t 3:float -r 3)
mbpoll -m tcp -p 5502 -a 1 -t 3:float -r 3 127.0.0.1
```

> A seconda delle versioni di `mbpoll`, l'ordine delle parole in virgola mobile può richiedere
> l'opzione di permutazione. In caso di valore incoerente, verificare l'ordine delle parole.

---

## 9. Mappa di memoria condensata

```
Coils (RW)            DiscreteInputs (RO)     Holding (RW)              Input (RO)
0  On/Off             0  In marcia            0  Modo verso1 (u16)      0-1 PV (f32)
1  Auto/Manuale       1  Caldo attivo         1  Modo verso2 (u16)      2-3 Uscita (f32)
                      2  Freddo attivo        2-3  SP auto (f32)         4-5 SP auto (rilettura, RO)
                                              4-5  SP manuale (f32)       6-7 SP manuale (rilettura, RO)
                                              6-7  Kp1  8-9  Ki1  10-11 Kd1
                                              12-13 Kp2 14-15 Ki2 16-17 Kd2
                                              18-19 Isteresi (f32)
                                              20-21 Ciclo min. TOR (f32, s)
                                              22-23 Periodo PWM (f32, s)
                                              42-46 Identificativo ASCII "CESAM-Lab"
```

> **Identificativo ASCII** (`HR 42-46`): «CESAM-Lab» codificato 2 caratteri per
> registro, carattere di peso maggiore per primo (`42`=`'CE'`, `43`=`'SA'`, `44`=`'M-'`,
> `45`=`'La'`, `46`=`'b\0'`). Sola lettura. Esempio:
> `mbpoll -m tcp -p 5502 -a 1 -t 4 -r 43 -c 5 127.0.0.1` (registri base 1 43..47).
