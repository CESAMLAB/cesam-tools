# Manuale utente — OSNE (agitatore da laboratorio simulato NAMUR)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · **IT** · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

> **OSNE** — *Open Stirrer NAMUR Emulator* · binario `mock_bin_su_namur`
> (eseguibile `osne`) · Licenza MIT · Editore: **CESAM-Lab** · Identità NAMUR:
> nome `CESAM-STIRRER`, tipo `OSNE`.
>
> *Un agitatore da laboratorio (tipo IKA) che esiste solo sulla vostra linea
> NAMUR — per testare supervisori, script e gateway senza hardware reale.*

Questo manuale è destinato all'**utente** dell'agitatore simulato: come avviarlo,
pilotarlo dall'interfaccia, parametrizzarlo e collegarlo in **NAMUR** (TCP o
seriale RS-232). Nessuna conoscenza di programmazione è necessaria.

---

## 1. A cosa serve questo software?

Simula un **agitatore da laboratorio** (agitatore da banco a elica, tipo IKA):

- un **motore fisico** realistico: la velocità sale/scende secondo la coppia
  applicata, con una **regolazione di velocità rapida**;
- un **carico viscoso regolabile**: più il mezzo è viscoso, più la coppia
  necessaria è elevata — fino al **sovraccarico** (riferimento irraggiungibile);
- un **server NAMUR** (protocollo seriale ASCII degli apparecchi da laboratorio)
  per pilotarlo/supervisionarlo da un altro software o uno script;
- un'**interfaccia grafica** di conduzione, visualizzazione e **test del
  protocollo** (mini-terminale NAMUR integrato).

È uno strumento di **test**: permette di mettere a punto e dimostrare un
supervisore, uno script di acquisizione o un gateway **senza hardware reale**.

---

## 2. Avviare il software

Lanciare l'eseguibile corrispondente al vostro sistema:

| Sistema | File |
|---------|---------|
| Windows | `osne-windows-x86_64.exe` (doppio clic) |
| Linux PC | `./osne-linux-x86_64` |
| Raspberry Pi (schermo) | `./osne-rpi-arm64` |

La finestra si apre e il **server NAMUR si avvia automaticamente** (porta `4001`
predefinita). L'intestazione indica lo stato:

- **● IN MARCIA / ● FERMO**: stato del motore;
- **NAMUR ● 0.0.0.0:4001** (verde): server in ascolto; **✖ …** (rosso) in caso di
  problema (porta occupata, seriale non disponibile…);
- un **indicatore di connessione**: in TCP mostra il master connesso (o «nessun
  master»), in seriale un semplice punto. Diventa **verde** quando una trama è
  stata ricevuta di recente (collegamento attivo), grigio altrimenti.

> Senza schermo (solo server), vedi il **§ 9 (Utilizzo senza schermo)**.

---

## 3. L'interfaccia a colpo d'occhio

```
┌──────────────── Intestazione: titolo OSNE, ⚙ Impostazioni, 💾 Salva, stati & indicatori ────────────┐
├──────────────────┬──────────────────────────────────────────────────────────────────────────────────┤
│  COMANDI          │   SUPERVISIONE                                                                     │
│  (sinistra)       │   - carte di valori (Velocità / Coppia / Viscosità / Sovraccarico)                │
│  Marcia/Arresto   │   - CURVA di tendenza in tempo reale (Riferimento / Velocità / Coppia)            │
│  Riferimento vel. │                                                                                   │
│  Viscosità        │                                                                                   │
│  Regolazioni PID  │                                                                                   │
├──────────────────┴──────────────────────────────────────────────────────────────────────────────────┤
│  ⇄ TRAME NAMUR: mini-terminale (RX/TX) + riga di comando + riferimento del protocollo (a destra)      │
└───────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 4. Pilotare l'agitatore (pannello di sinistra)

### 4.1 Marcia / Arresto
Pulsante **Marcia / Arresto**. All'arresto, il motore decelera liberamente fino
all'immobilizzazione (attrito + carico), coppia motore nulla.

### 4.2 Riferimento di velocità
Cursore **Riferimento di velocità** (in `tr/min`), limitato dai limiti min/max
impostati nei *Parametri*. È la stessa grandezza del comando NAMUR `OUT_SP_4`
(canale 4). In marcia, l'asservimento porta la velocità misurata verso questo
riferimento.

### 4.3 Viscosità del mezzo
Cursore **Viscosità** (scala logaritmica). Rappresenta il **carico** del mezzo
agitato:

- viscosità **bassa** → coppia bassa, il riferimento è raggiunto rapidamente;
- viscosità **elevata** → coppia di carico importante; se la coppia necessaria
  supera la **coppia motore massima**, la velocità di riferimento **non viene più
  raggiunta** → l'indicatore **Sovraccarico ⚠** si accende (comportamento di un
  agitatore reale di fronte a un mezzo troppo denso).

### 4.4 Regolazioni PID (Kp, Ki, Kd)
I tre guadagni dell'asservimento di velocità, regolabili in diretta:

- **Kp** (proporzionale): più è grande, più la salita in velocità è vivace
  (rischio di sovraelongazione/oscillazione);
- **Ki** (integrale): annulla lo scarto residuo di velocità nel tempo;
- **Kd** (derivativo): smorza/anticipa (troppo forte → sensibile al rumore).

> I guadagni predefiniti sono volutamente «duri»: l'uscita satura alla coppia
> massima finché l'errore è grande (salita rapida), poi il termine integrale
> stabilizza. L'uscita del PID **è** la coppia motore, limitata a `[0, couple_max]`.

---

## 5. Leggere la curva di tendenza

La curva (al centro) traccia tre grandezze in tempo reale. La **legenda, in alto a
sinistra**, ricorda il colore **e l'ultimo valore** di ogni serie:

| Colore | Serie | Significato |
|---------|-------|---------------|
| 🔵 blu | **Riferimento** | riferimento di velocità (in marcia) |
| 🔴 rosso | **Velocità** | velocità misurata (`tr/min`, asse di sinistra) |
| 🟢 verde | **Coppia** | coppia misurata (`N·cm`, **asse di destra**) |

> La curva ha **due assi verticali**: la **velocità** (`tr/min`) a sinistra, la
> **coppia** (`N·cm`) a destra. La coppia è messa in scala per condividere il
> grafico, ma l'asse di destra mostra effettivamente dei `N·cm`.

Sopra la curva, delle **carte** mostrano i valori istantanei: **Velocità**,
**Coppia**, **Viscosità**, e **Sovraccarico ⚠** quando il motore satura. Si può
zoomare/spostare la curva con il mouse.

---

## 6. Il mini-terminale NAMUR (parte bassa della finestra)

Il pannello **⇄ Trame NAMUR** permette di **testare il protocollo** direttamente
dall'IHM, senza client esterno:

- il **giornale** mostra le trame **ricevute** (`← RX`, blu) ed **emesse**
  (`→ TX`, verde), con marca temporale;
- la **riga di comando** invia una trama NAMUR al simulatore (tasto **Invio** o
  pulsante **▶ Invia**). Le frecce **↑/↓** richiamano i comandi precedenti
  (cronologia);
- il **riferimento del protocollo** (pannello di destra) elenca i comandi: un
  **clic** inserisce il comando nella riga di immissione;
- il pulsante **🗑 Cancella** svuota il giornale.

> Le trame digitate qui sono interpretate esattamente come quelle di un master di
> rete: `OUT_SP_4 500` imposta il riferimento, `START_4`/`STOP_4`
> avviano/arrestano, `IN_PV_4` legge la velocità, ecc. Il **watchdog**
> (`OUT_WD1@…`) ha tuttavia effetto solo all'interno di una vera sessione di rete
> (cfr. § 8).

---

## 7. Parametri (pulsante ⚙)

Il pulsante **⚙ Parametri** apre una finestra per configurare:

### Lingua dell'interfaccia
Selettore in alto: **Français, English, Deutsch, Español, Italiano, Português,
Nederlands, Polski** (8 lingue). La lingua è persistita.

### Trasporto NAMUR
Scelta del collegamento — **uno solo attivo alla volta**:

**TCP (Ethernet)**
- **IP di ascolto** (`0.0.0.0` = tutte le interfacce) e **Porta** (predefinita 4001);
- **IP autorizzate**: una per riga, jolly `*` accettati (es. `192.168.1.*`).
  **Lista vuota = tutte le IP autorizzate.** Le altre sono rifiutate.

**Seriale (RS-232)** — richiede un binario compilato con la feature `serial`
- **Porta seriale**: `/dev/ttyUSB0` (Linux), `COM3` (Windows)…;
- **Baud** (predefinito 9600), **Parità** (predefinita Pari), **Bit di dati** (7),
  **Bit di stop** (1) — impostazione NAMUR da laboratorio tipica: **9600 7E1**.

> ⚠️ **Un solo master alla volta.** In TCP, un nuovo master **attende** fino alla
> disconnessione del precedente (collegamento punto-punto). L'IHM locale **non** è
> un master. In seriale, il bus *è* l'unico master; privilegiare un **collegamento
> punto-punto** (il server risponde qualunque sia l'indirizzo richiesto).

### Parametri motore
Comportamento fisico simulato `J·dω/dt = T − k·η·ω − attrito`:
- **Inerzia** (`J`): reattività del motore (piccolo ⇒ rapido);
- **Coefficiente di carico** (`k`): peso della viscosità sulla coppia;
- **Attrito** (`N·cm`): attrito secco residuo;
- **Coppia max** (`N·cm`): coppia motore massima (limite dell'uscita PID).

### Limiti di velocità
Limiti min/max del riferimento di velocità (`tr/min`).

### Limiti di viscosità
Limiti min/max del cursore di viscosità.

Pulsanti: **Applica** (ha effetto immediatamente **e** registra), **Ripristina ai
valori predefiniti**, **Chiudi**.

### Registrazione delle impostazioni
Le impostazioni sono **salvate** in un file `mock_su_namur.toml` (accanto al
software) e **ricaricate al prossimo avvio**. Il pulsante **💾 Salva**
dell'intestazione registra anche i guadagni PID e la viscosità modificati dal
pannello di sinistra.

---

## 8. Collegare un client NAMUR

Il software è uno **slave NAMUR** (TCP porta 4001 predefinita, o seriale secondo il
trasporto scelto al § 7). Un client (script, terminale, gateway) **invia una riga
ASCII per richiesta**, terminata da `CR LF`. Le **letture** (`IN_*`) restituiscono
un valore; le **scritture/azioni** (`OUT_*`, `START_*`, `STOP_*`, `RESET`) sono
**silenziose** (nessuna risposta), conformemente all'uso NAMUR.

Riferimenti principali:

| Comando | Effetto |
|----------|-------|
| `IN_NAME` / `IN_TYPE` | identità (`CESAM-STIRRER` / `OSNE`) |
| `IN_PV_4` / `IN_PV_5` | leggere la velocità (`tr/min`) / la coppia (`N·cm`) |
| `IN_SP_4` | leggere il riferimento di velocità |
| `OUT_SP_4 <v>` | **impostare** il riferimento di velocità |
| `START_4` / `STOP_4` / `RESET` | avviare / arrestare / reinizializzare |
| `OUT_WD1@<m>` | **watchdog**: arresto sicuro se silenzio per `<m>` s |

Esempio con `nc` (netcat):

```text
$ nc 127.0.0.1 4001
IN_NAME
CESAM-STIRRER
OUT_SP_4 1200          (silenzioso)
START_4                (silenzioso)
IN_PV_4
1200.0 4
STOP_4                 (silenzioso)
```

> Il **watchdog** `OUT_WD1@30` arresta automaticamente il motore se **nessuna
> riga** arriva per 30 s (protezione in caso di perdita di comunicazione).
> `OUT_WD1@0` lo disarma. Il contatore è ri-armato a ogni comando ricevuto.

> Il **riferimento completo del protocollo** (canali, codifica, esempi) è in
> **[commandes_namur.md](commandes_namur.md)**. La stessa lista è ricordata **in
> diretta** nel pannello di destra del mini-terminale.

---

## 9. Utilizzo senza schermo («headless» / Docker)

Per un deployment in background (Raspberry Pi senza schermo, server), esiste una
versione **senza interfaccia**: fa girare la simulazione e il server NAMUR,
pilotabili **unicamente tramite NAMUR**.

```bash
# Immagine Docker (distribuibile ovunque):
docker run --rm -p 4001:4001 -v "$PWD/conf:/data" osne:headless
```

La cartella montata su `/data` permette di fornire/conservare `mock_su_namur.toml`.

---

## 10. Domande frequenti

| Domanda / sintomo | Risposta |
|---------------------|---------|
| **Sovraccarico ⚠** si accende e la velocità non raggiunge il riferimento. | Normale: la **viscosità** richiede più coppia di quella che il motore fornisce. Abbassate la viscosità o il riferimento, o aumentate la **coppia max** (Parametri). |
| La velocità non si muove. | Verificate che l'agitatore sia **In marcia** e il riferimento non nullo. |
| L'intestazione mostra **NAMUR ✖**. | Porta già in uso o < 1024 senza privilegi (TCP), o porta seriale non disponibile: cambiate l'impostazione in ⚙ Parametri. |
| Il mio client NAMUR/TCP è rifiutato. | La sua IP non è nella **lista bianca**: svuotate la lista o aggiungete un pattern (`192.168.1.*`). |
| `OUT_SP_4 …` non restituisce nulla. | Normale: le scritture/azioni NAMUR sono **silenziose**. Leggete con `IN_SP_4` / `IN_PV_4`. |
| Il motore si arresta da solo. | Un **watchdog** è armato (`OUT_WD1@…`) e nessun comando è arrivato in tempo. Disarmatelo (`OUT_WD1@0`) o inviate trame regolarmente. |
| Il collegamento seriale non si apre. | Binario compilato **senza** la feature `serial`, o porta/permessi errati (gruppo `dialout` su Linux). |
| Le mie impostazioni non vengono conservate. | Cliccate **Applica** / **💾 Salva**. Il file `mock_su_namur.toml` deve essere accessibile in scrittura. |

---

*Documentazione tecnica associata: [conception.md](conception.md) ·
[commandes_namur.md](commandes_namur.md) · [maintenance.md](maintenance.md).*
