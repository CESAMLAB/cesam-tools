# Manuale utente — Regolatore PROFIBUS DP simulato (ORPD)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · **IT** · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

> Crate: `mock_bin_ru_pbdp` · Eseguibile: **ru_pbdp** · Marchio: **ORPD**

---

## ⚠️ Prima di iniziare: cosa questo simulatore NON è

`ru_pbdp` **non è** uno slave PROFIBUS DP conforme all'hardware reale. Il
PROFIBUS DP è un bus a token il cui rispetto delle finestre temporali
(*slot time*, `Tsdr`, watchdog) richiede un circuito dedicato (ASIC
SPC3/VPC3, scheda master Hilscher/Softing/Siemens CP). Un normale
programma Tokio, anche collegato a una vera porta RS-485, **non può
rispettare questi vincoli**: un vero PLC (ad esempio un Siemens S7 con
scheda master) **non riconoscerà mai** questo simulatore come slave valido
su un bus reale.

Cosa fa realmente `ru_pbdp`: implementa, **in software e senza vincoli di
tempo reale**, la struttura delle trame e la macchina a stati di uno slave
DP-V0 (parametrizzazione, configurazione, diagnostica, scambio ciclico). È
uno strumento per **comprendere il protocollo** e **testare uno sviluppo
software** (codec, macchina a stati, strumentazione) — non per pilotare
apparecchiature di campo. Vedere
[reference_profibus.md](reference_profibus.md) §6 per il dettaglio dei
limiti.

---

## 1. A cosa serve questo simulatore

`ru_pbdp` simula un **regolatore di processo** (anello PID su un processo
termico, modello identico a ORME/Modbus) e lo espone tramite un insieme
simulato di trame PROFIBUS DP-V0, su un collegamento seriale
(RS-485/RS-232). L'interfaccia grafica permette di **pilotare** la
simulazione e **visualizzarne** la dinamica; il registro delle trame mostra
il traffico scambiato in esadecimale.

---

## 2. Primi passi

```bash
cargo run -p mock_bin_ru_pbdp          # GUI + collegamento seriale PROFIBUS DP
```

All'avvio, il simulatore tenta di aprire la porta seriale configurata (per
default `/dev/ttyUSB0` o `COM3`, 500 kbit/s, indirizzo di stazione 3). Se
la porta non esiste (caso frequente in assenza di hardware seriale), la
GUI mostra l'errore di apertura nell'intestazione — la simulazione del
regolatore continua a funzionare, solo il collegamento non è disponibile.
Impostare la **porta seriale** in *Impostazioni* per puntare a uno
pseudo-terminale o a un adattatore USB-seriale disponibile.

---

## 3. L'interfaccia

### Intestazione

- **Titolo** e pulsanti **⚙ Impostazioni** / **💾 Salva impostazioni**.
- A destra: **stato dell'apparecchio** (IN MARCIA / FERMO), **stato del
  collegamento** (`PROFIBUS ● <porta> [<stato>]` in verde se aperto — lo
  stato mostrato è quello della macchina a stati DP-V0:
  `Power_On`/`Wait_Prm`/`Wait_Cfg`/`Data_Exchange`), e il **logo
  CESAM-Lab**.
- Un **banner arancione permanente** ricorda la non interoperabilità con
  hardware reale (vedere l'avvertenza precedente).

### Mini-terminale (parte inferiore della finestra)

Registro in sola lettura delle trame **ricevute** (← RX) ed **emesse**
(→ TX), con marca temporale e visualizzazione esadecimale. Pulsante
**Cancella** per svuotare il registro.

### Pannello comandi (sinistra)

Identico a ORME: **Marcia/Arresto**, **Auto/Manuale**, modi di
regolazione **direzione 1 (caldo)** / **direzione 2 (freddo)**
(Off/PID/Tutto-o-niente/PWM), **setpoint** (automatico e manuale),
**impostazioni PID** di entrambe le direzioni, **isteresi**, **ciclo
minimo tutto-o-niente**, **periodo PWM**.

### Pannello destro: blocchi I/O PROFIBUS

Tabella in tempo reale dei blocchi *Output* (master→slave) e *Input*
(slave→master), con la disposizione dei byte usata da questo simulatore —
vedere [reference_profibus.md](reference_profibus.md) §3.

### Area centrale

Schede **Misura**, **Setpoint attivo**, **Uscita**, e curva di tendenza.

---

## 4. Impostazioni (modale ⚙)

- **Lingua** dell'interfaccia (8 lingue), persistita.
- **Verificare gli aggiornamenti all'avvio** + pulsante **Verifica ora**.
- **Porta seriale**, **velocità** (baud — usare un valore normalizzato
  PROFIBUS DP: 9600, 19200, 45450, 93750, 187500, 500000, 1500000,
  3000000, 6000000 o 12000000), **indirizzo di stazione** (0-125).
- **Watchdog di protocollo (consentito)**: casella — se deselezionata, il
  watchdog richiesto dal master tramite `Set_Prm` viene **ignorato** (mai
  armato).
- **Funzione di trasferimento del processo**: guadagno `K`, costante di
  tempo `τ`, ritardo puro, valore ambiente.
- **Limiti di setpoint**: min / max (riordinati automaticamente se
  invertiti).
- **Applica** / **Ripristina predefiniti** / **Chiudi**.

Una modifica di porta/velocità/indirizzo **richiude e riapre** il
collegamento seriale. Le impostazioni vengono salvate in
`mock_ru_pbdp.toml` (directory corrente; sovrascrivibile tramite la
variabile d'ambiente `MOCK_CONFIG`).

**Il formato di trama (8E1) è fissato dalla norma PROFIBUS DP** e non è
regolabile qui, a differenza di Modbus RTU o NAMUR seriale.

---

## 5. Il mini-terminale come strumento didattico

Senza hardware PROFIBUS reale, il modo migliore per osservare il
protocollo è far dialogare **due istanze** di questo strumento tra loro —
o scrivere un piccolo script che riproduca una sequenza `Slave_Diag` →
`Set_Prm` → `Chk_Cfg` → `Data_Exchange` su uno pseudo-terminale (`socat -d
-d pty,raw,echo=0 pty,raw,echo=0`) — e leggere il mini-terminale per
vedere le trame scambiate in esadecimale, con la loro decodifica in
[reference_profibus.md](reference_profibus.md).

---

## 6. Domande frequenti

**Posso collegare questo simulatore a un vero PLC PROFIBUS DP?** No —
vedere l'avvertenza in cima a questo documento e il §6 di
[reference_profibus.md](reference_profibus.md).

**La porta seriale non si apre.** Il file/dispositivo indicato non esiste
o i permessi sono insufficienti (gruppo `dialout` su Linux). L'errore
esatto viene mostrato nell'intestazione della GUI.

**Il collegamento resta in `Wait_Prm`.** Il master non ha ancora inviato
un `Set_Prm` con l'identificativo atteso (`0xEE01`, identificativo
**fittizio**, non registrato presso il PNO). Vedere
[reference_profibus.md](reference_profibus.md) §2.

**Il collegamento resta in `Wait_Cfg`.** Il `Chk_Cfg` ricevuto non
annuncia le lunghezze I/O attese (45 byte di uscita, 17 di ingresso per
questo simulatore).

**L'apparecchio si ferma da solo.** Il watchdog di protocollo (armato dal
master tramite `Set_Prm`) è scaduto per mancanza di scambio ciclico
ricevuto in tempo — è lo stato sicuro atteso, non un bug.

**Avviare senza interfaccia grafica?** Compilare in modalità *headless*:
`cargo run -p mock_bin_ru_pbdp --no-default-features` — il collegamento
seriale e la simulazione funzionano senza GUI.
