# Manuale utente — Regolatore di processo simulato (RU/OPC UA)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · **IT** · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

> Crate: `mock_bin_ru_opcua` · Eseguibile: **ru_opcua**

---

## 1. A cosa serve questo simulatore

`ru_opcua` simula un **regolatore di processo** (anello PID su un processo
termico) e lo espone in **OPC UA**, lo standard di supervisione industriale.
Serve a **testare un client OPC UA / uno SCADA** (lettura di misure, scrittura di
setpoint, sottoscrizioni) senza hardware reale.

L'interfaccia grafica permette di **pilotare** la simulazione e di **visualizzare** la
dinamica; il server OPC UA espone le stesse grandezze alla rete.

---

## 2. Primi passi

```bash
cargo run -p mock_bin_ru_opcua          # IHM + server OPC UA
```

All'avvio, il server è in ascolto per default su `opc.tcp://0.0.0.0:4840/`
(sicurezza None). La finestra mostra lo stato corrente e avvia la curva di
tendenza.

Connettere un client OPC UA (UaExpert, ecc.) a `opc.tcp://127.0.0.1:4840/`,
sicurezza **None**, utente **Anonymous**. I nodi sono descritti nel
[riferimento OPC UA](reference_opcua.md).

---

## 3. L'interfaccia

### Intestazione

- **Titolo** e pulsanti **⚙ Parametri** / **💾 Salva le impostazioni**.
- A destra: **stato del dispositivo** (IN MARCIA / FERMO), **stato del server**
  (`OPC UA ● opc.tcp://…` in verde se in ascolto, ✖ + messaggio in caso di errore), e
  il **logo CESAM-Lab**.
- Un **banner arancione** ricorda permanentemente che l'endpoint è **anonimo
  (sicurezza None)**: da esporre solo su rete fidata.
- Se è disponibile un aggiornamento, un **banner** propone il download.

### Pannello comandi (sinistra)

- **Marcia / Arresto**: avvia o arresta la regolazione. All'arresto, il processo
  si rilassa verso il valore ambiente.
- **Modalità automatica (PID)**: attivata = il PID calcola l'uscita; disattivata =
  **modalità manuale** (l'uscita è imposta).
- **Setpoint**: cursore, limitato dai limiti di setpoint (regolabili in
  *Parametri*).
- **Uscita manuale (%)**: cursore attivo solo in **modalità manuale**.
- **Impostazioni PID**: guadagni `Kp`, `Ki`, `Kd` modificabili a caldo.

### Zona centrale

- **Schede**: Misura, Setpoint, Uscita.
- **Curva di tendenza**: Misura (PV) e Setpoint sull'asse di sinistra (unità
  di processo), Uscita (%) sull'asse di destra.

---

## 4. Parametri (modale ⚙)

- **Lingua** dell'interfaccia (8 lingue), persistita.
- **Verifica gli aggiornamenti all'avvio** + pulsante **Verifica ora**.
- **Endpoint**: **IP di ascolto** e **porta** del server OPC UA. Una modifica
  **riavvia** il server a caldo (le sessioni in corso vengono chiuse correttamente).
- **Sicurezza OPC UA**: **Cifratura** (`Basic256Sha256`), **Consenti anonimo**,
  **Fiducia auto. nei certificati client**, **Utente** / **Password**
  (campi attivi quando la cifratura è selezionata).
  Attivare la cifratura genera un certificato al primo avvio (alcuni
  secondi) e riavvia il server.
- **Processo (funzione di trasferimento)**: guadagno `K`, costante di tempo `τ`, ritardo
  puro, valore ambiente.
- **Limiti di setpoint**: min / max (riordinati automaticamente se invertiti).
- **Applica** / **Ripristina ai valori predefiniti** / **Chiudi**.

Le impostazioni sono salvate in `mock_ru_opcua.toml` (directory corrente;
sovrascrivibile tramite la variabile d'ambiente `MOCK_CONFIG`).

---

## 5. Sicurezza

La sicurezza OPC UA è **regolabile** in *Parametri*:

- **Senza cifratura** (default): endpoint **sicurezza None**, accesso **anonimo** —
  nessuna protezione. **Non esporre su una rete aperta.** Un banner **arancione**
  lo ricorda.
- **Con cifratura**: endpoint **`Basic256Sha256`** (firmato + cifrato). Il
  server genera il proprio certificato al primo avvio. La **fiducia nei
  certificati client** è regolabile (auto per default, o stretta). Si può
  richiedere un **utente / password** e/o consentire
  l'anonimo. Un banner **verde 🔒** conferma la cifratura. Per connettersi, il
  client deve allora usare la politica `Basic256Sha256` e fidarsi del
  certificato del server (primo scambio).

La password è memorizzata **in chiaro** nel file TOML: si tratta di un
**simulatore**, da usare su rete fidata.

---

## 6. FAQ

**La porta 4840 è obbligatoria?** No: si regola in *Parametri* (o tramite il
file TOML). Una porta < 1024 richiede i diritti root.

**Il mio client non vede i nodi.** Verificare la connessione a `opc.tcp://…:4840/`,
sicurezza **None**, utente **Anonymous**, poi *Browse* sotto la cartella
`Objects` (namespace `urn:cesam-lab:ru-opcua`).

**Una scrittura viene rifiutata.** Il tipo deve corrispondere (`Double` per le
grandezze, `Boolean` per `Run`/`Auto`); altrimenti il server restituisce
`Bad_TypeMismatch`.

**Avviare senza interfaccia grafica?** Compilare in *headless*:
`cargo run -p mock_bin_ru_opcua --no-default-features` — il server OPC UA e la
simulazione girano senza IHM.

**Appare un messaggio «encrypted endpoints disabled».** È normale in
Fase 1b: nessun certificato di istanza è provvisto (endpoint cifrati
non disponibili). L'endpoint None, invece, funziona.
