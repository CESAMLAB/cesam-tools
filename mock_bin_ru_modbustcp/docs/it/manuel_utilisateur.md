# Manuale utente — ORME (regolatore simulato Modbus)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · **IT** · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

> **ORME** — *Open Regulator Modbus Emulator* · binario `mock_bin_ru_modbustcp` ·
> Licenza MIT · Editore: **CESAM-Lab** · Identificativo apparecchio Modbus: **CESAM-Lab**
>
> *«Aprite il bus.»* Un regolatore di campo che esiste solo sul vostro bus
> Modbus (TCP/RTU) — per testare SCADA, PLC e IHM senza hardware reale.

Questo manuale è destinato all'**utente** del regolatore simulato: come avviarlo,
pilotarlo dall'interfaccia, parametrizzarlo e collegarlo in Modbus TCP.
Nessuna conoscenza di programmazione è necessaria.

---

## 1. A cosa serve questo software?

Simula un **regolatore industriale** (tipo forno o bagno termostatato):

- un **processo fisico** realistico (la «misura» sale/scende secondo il comando);
- una **regolazione** automatica o manuale, in **caldo** e/o in **freddo**;
- un **server Modbus TCP** per pilotarlo/supervisionarlo da un altro software
  (PLC, SCADA, gateway…);
- un'**interfaccia grafica** di conduzione e visualizzazione.

È uno strumento di **test**: permette di mettere a punto e dimostrare un
supervisore o un PLC **senza hardware reale**.

---

## 2. Avviare il software

Lanciare l'eseguibile corrispondente al vostro sistema:

| Sistema | File |
|---------|---------|
| Windows | `orme-windows-x86_64.exe` (doppio clic) |
| Linux PC | `./orme-linux-x86_64` |
| Raspberry Pi (schermo) | `./orme-rpi-arm64` |

La finestra si apre e il **server Modbus si avvia automaticamente** (porta `5502`
predefinita). L'intestazione indica lo stato:

- **● IN MARCIA / ● FERMO**: stato dell'apparecchio;
- **Modbus ● 0.0.0.0:5502** (verde): server in ascolto; **✖ …** (rosso) in caso
  di problema di rete.

> Senza schermo (solo server), vedi il **§ 9 (Utilizzo senza schermo)**.

---

## 3. L'interfaccia a colpo d'occhio

La finestra comprende quattro zone:

```
┌───────────────────────────── Intestazione: titolo, ⚙ Parametri, 💾 Salva, stati ─────────────────────────────┐
├──────────────────┬─────────────────────────────────────────────────┬───────────────────────────────────────────┤
│  COMANDI          │   SUPERVISIONE                                  │   TABELLA INDIRIZZI MODBUS                │
│  (sinistra)       │   - valori istantanei (Misura / Setpoint /      │   (destra)                                │
│  Marcia/Arresto   │     Uscita)                                     │   lista live: designazione, tabella,      │
│  Auto/Manuale     │   - CURVA di andamento in tempo reale           │   indirizzo, valore, accesso              │
│  Modalità, setpoint│                                                │                                           │
│  regolazioni PID… │                                                 │                                           │
└──────────────────┴─────────────────────────────────────────────────┴───────────────────────────────────────────┘
```

---

## 4. Pilotare il regolatore (pannello di sinistra)

### 4.1 Marcia / Arresto
Pulsante **Marcia / Arresto**. All'arresto, l'uscita è nulla e la misura ritorna
dolcemente verso il valore ambiente.

### 4.2 Auto / Manuale
- **Manuale**: *voi* imponete l'uscita tramite il **setpoint manuale** (in %).
- **Auto**: il regolatore calcola l'uscita per raggiungere il **setpoint auto**.

### 4.3 I setpoint
Ogni setpoint dispone di un **campo numerico** (immissione precisa da tastiera) e
di un **cursore**. Entrambi sono sempre modificabili; il setpoint **attivo**
(secondo la modalità) è mostrato in grassetto.

| Setpoint | Unità | Ruolo |
|----------|-------|------|
| **SP auto** | unità di misura (es. °C) | obiettivo da raggiungere in modalità Auto |
| **SP manuale** | % di uscita, da −100 a +100 | uscita imposta in modalità Manuale (**+** caldo / **−** freddo) |

### 4.4 Modalità di regolazione — verso 1 (caldo) e verso 2 (freddo)
Ogni verso si regola indipendentemente:

- **Disattivato** — il verso non agisce;
- **PID** — regolazione continua (uscita 0…100 %), precisa e dolce;
- **Tutto-o-niente (TOR)** — relè a isteresi: uscita 0 % o 100 %, semplice ma
  oscillante attorno al setpoint;
- **Relè a ciclo (PWM)** — un PID calcola un rapporto ciclico, *spezzettato* su un
  periodo fisso: l'uscita fisica resta tutto-o-niente (0/100 %), ma la sua
  **media** segue il PID. È il miglior compromesso per pilotare con precisione un
  organo che sa solo aprirsi o chiudersi (relè, valvola TOR).

> 👉 **Importante — vedi **§ 6 (Comprendere la regolazione)****: scegliere
> PID/TOR/PWM per il freddo *arma* il freddo, ma questo **eroga solo quando
> la misura supera il setpoint**.

### 4.5 Regolazioni PID (Kp, Ki, Kd)
Per ogni verso, tre guadagni regolabili in diretta:

- **Kp** (proporzionale): più è grande, più la reazione è vivace (rischio di oscillazione);
- **Ki** (integrale): annulla lo scarto residuo nel tempo (troppo forte → superamento);
- **Kd** (derivativo): smorza/anticipa (troppo forte → sensibile al rumore).

### 4.6 Regolazioni TOR / PWM
- **Isteresi TOR** — larghezza della **zona morta** della modalità Tutto-o-niente, centrata
  sul setpoint (`[SP − h/2, SP + h/2]`): evita che l'uscita scatti senza
  sosta. Più è larga, maggiore è l'ondulazione ma più distanziate le commutazioni.
- **Ciclo min. TOR (s)** — durata minima durante la quale il relè resta in uno
  stato prima di poter ricommutare (**anti-corto-ciclo**). Protegge un attuatore
  reale (relè, compressore) e leviga il comportamento. `0` = disattivato.
- **Periodo PWM (s)** — durata di un ciclo del **relè a ciclo**. Corto → media
  più fedele ma commutazioni frequenti; lungo → meno usura ma ondulazione
  più marcata. Da scegliere molto più piccolo della costante di tempo del processo.

---

## 5. Leggere la curva di andamento

La curva (al centro) traccia in tempo reale tre grandezze. La **legenda, in alto
a sinistra**, ricorda il colore **e l'ultimo valore** di ogni serie:

| Colore | Serie | Significato |
|---------|-------|---------------|
| 🔵 blu | **Setpoint (SP)** | obiettivo (in Auto) |
| 🔴 rosso | **Misura (PV)** | valore del processo |
| 🟢 verde | **Uscita (%)** | comando applicato (**+** caldo / **−** freddo) |

Sopra la curva, tre schede mostrano i valori istantanei
(Misura, Setpoint attivo, Uscita). Si può zoomare/spostare la curva con il mouse.

---

## 6. Comprendere la regolazione (caldo / freddo)

Il regolatore agisce in **un solo verso alla volta**, scelto secondo lo scarto
`Setpoint − Misura`:

| Situazione | Verso che agisce | Uscita | Spia |
|-----------|---------------|--------|--------|
| Misura **<** Setpoint (bisogna scaldare) | **Verso 1 (caldo)** | **positiva** (0…+100 %) | **Caldo attivo = 1** |
| Misura **>** Setpoint (bisogna raffreddare) | **Verso 2 (freddo)** | **negativa** (−100…0 %) | **Freddo attivo = 1** |

Conseguenze pratiche:

- Selezionare **PID/TOR per il freddo** non basta ad accendere «Freddo attivo»:
  occorre che **la misura sia al di sopra del setpoint**. Finché la misura è
  al di sotto, è il **caldo** che lavora.
- Per vedere il freddo erogare: in **Auto**, verso 2 in PID/TOR, **abbassare il
  setpoint sotto la misura corrente** (o attendere un superamento). L'uscita
  diventa negativa e **Freddo attivo** passa a 1.
- In **TOR**, il relè commuta sulla **mezza isteresi** da entrambi i lati del
  setpoint (zona morta simmetrica) e rispetta il **ciclo minimo** tra due
  commutazioni. In **PWM**, l'uscita spezzetta a 0/100 % ma la sua media segue il PID.

---

## 7. Parametri (pulsante ⚙)

Il pulsante **⚙ Parametri** apre una finestra per configurare:

### Trasporto Modbus
Scelta del bus di comunicazione — **uno solo attivo alla volta**:

**TCP (Ethernet)**
- **IP di ascolto** (`0.0.0.0` = tutte le interfacce) e **Porta** (predefinito 5502);
- **IP autorizzate**: una per riga, jolly `*` accettati (es. `192.168.1.*`).
  **Lista vuota = tutte le IP autorizzate.** Le altre sono rifiutate.

**RTU (RS485)** — richiede un binario compilato con la feature `rtu`
- **Porta seriale**: `/dev/ttyUSB0`, `/dev/ttyAMA0` (Raspberry Pi), `COM3` (Windows)…;
- **Baud** (predefinito 19200), **Parità** (predefinito Pari), **Bit di dati** (8),
  **Bit di stop** (1) — da concordare con il master;
- **Indirizzo slave** (1–247).

> ⚠️ **Un solo master remoto alla volta.** In TCP, la connessione di un nuovo
> master **disconnette automaticamente** il precedente. L'IHM locale **non** è
> un master: resta sempre attiva. In RTU, privilegiare una **connessione
> punto-punto** (l'apparecchio risponde qualunque sia l'indirizzo richiesto).

### Funzione di trasferimento (processo)
Comportamento fisico simulato `G(s) = K·e^(−L·s) / (1 + T·s)`:
- **Guadagno K**: variazione di misura per % di uscita;
- **Costante T** (s): inerzia/rapidità;
- **Ritardo L** (s): tempo morto prima della reazione;
- **Ambiente**: valore di riposo.

### Limiti di setpoint
Limiti minimo/massimo del setpoint auto.

Pulsanti: **Applica** (ha effetto immediato **e** registra),
**Ripristina predefiniti**, **Chiudi**.

### Registrazione delle impostazioni
Le impostazioni sono **salvate** in un file `mock_ru_modbustcp.toml` (accanto
al software) e **ricaricate al successivo avvio**. Il pulsante **💾 Salva
impostazioni** dell'intestazione registra anche i guadagni PID, l'isteresi, il ciclo
minimo TOR e il periodo PWM modificati dal pannello di sinistra.

---

## 8. Collegare un client Modbus

Il software è uno **slave Modbus** (TCP porta 5502 predefinita, o RTU seriale
secondo il trasporto scelto al § 7). Un client (PLC, SCADA, `mbpoll`…) può
**leggere** lo stato e **scrivere** i setpoint/modalità. Promemoria: **un solo master
remoto alla volta** (in TCP, un nuovo arrivato disconnette il precedente).

Riferimenti principali (indirizzi **base 0**):

| Dato | Tabella | Indirizzo | Tipo | Accesso |
|--------|-------|---------|------|-------|
| Marcia/Arresto | Coil | 0 | bit | L/S |
| Auto/Manuale | Coil | 1 | bit | L/S |
| Modalità verso 1 / verso 2 | Holding | 0 / 1 | 0=Off,1=PID,2=TOR,3=PWM | L/S |
| Setpoint auto | Holding | 2–3 | virgola mobile | L/S |
| Setpoint manuale | Holding | 4–5 | virgola mobile | L/S |
| Ciclo min. TOR (s) | Holding | 20–21 | virgola mobile | L/S |
| Periodo PWM (s) | Holding | 22–23 | virgola mobile | L/S |
| Misura (PV) | Input | 0–1 | virgola mobile | L |
| Uscita (%) | Input | 2–3 | virgola mobile | L |
| Identificativo «CESAM-Lab» | Holding | 42–46 | testo ASCII | L |

> La **tabella completa** (guadagni PID, isteresi, codifica dei virgola mobile, codici
> funzione, esempi `mbpoll`) è in **[table_modbus.md](table_modbus.md)**.
> La stessa tabella è anche visibile **in diretta** nel pannello di destra dell'IHM.

---

## 9. Utilizzo senza schermo («headless» / Docker)

Per un deployment in background (Raspberry Pi senza schermo, server), esiste una
versione **senza interfaccia**: fa girare la simulazione e il server
Modbus, pilotabili **solo tramite Modbus**.

```bash
# Immagine Docker (distribuibile ovunque) :
docker run --rm -p 5502:5502 -v "$PWD/conf:/data" orme:headless
```

La cartella montata su `/data` permette di fornire/conservare `mock_ru_modbustcp.toml`.

---

## 10. Domande frequenti

| Domanda / sintomo | Risposta |
|---------------------|---------|
| **«Freddo attivo» non passa a 1 anche se ho messo PID/TOR.** | Normale: il freddo eroga solo se **la misura supera il setpoint**. Abbassate il setpoint sotto la misura (modalità Auto). Vedi **§ 6 (Comprendere la regolazione)**. |
| La misura non si muove. | Verificate che l'apparecchio sia **In marcia**, e setpoint/uscita non nulli. |
| In manuale, cambiare le modalità verso 1/2 non fa nulla. | Normale: le modalità si applicano solo in **Auto**. |
| L'intestazione mostra **Modbus ✖**. | Porta già in uso o < 1024 senza diritti: cambiate la **porta** in ⚙ Parametri. |
| Il mio client Modbus è rifiutato. | Il suo IP non è nella **lista bianca**: svuotate la lista o aggiungete un pattern (`192.168.1.*`). |
| I virgola mobile letti sono incoerenti. | Problema di **ordine delle parole** lato client (parola di peso maggiore per prima). Vedi table_modbus.md. |
| Un setpoint scritto in Modbus è ignorato. | Un virgola mobile occupa **2 registri**: scriveteli **insieme**. |
| Le mie impostazioni non vengono conservate. | Cliccate **Applica** / **💾 Salva**. Il file `mock_ru_modbustcp.toml` deve essere accessibile in scrittura. |

---

*Documentazione tecnica associata: [conception.md](conception.md) ·
[table_modbus.md](table_modbus.md) · [maintenance.md](maintenance.md).*
