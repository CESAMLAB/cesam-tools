# Manuale utente — Regolatore S7 (ORSS)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · **IT** · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

---

## 1. A cosa serve lo strumento

**ORSS** simula un'**unità di regolazione** di processo (PID + processo termico del
primo ordine) e la espone come un **PLC Siemens S7** (server S7comm su
ISO-on-TCP). Serve a testare una supervisione o un client S7 (Snap7, TIA Portal in
lettura, nodes7…) senza un PLC reale.

## 2. Primi passi

```bash
cargo run -p mock_bin_ru_s7        # IHM + server S7
```

Il server ascolta in modo predefinito su `0.0.0.0:102`. ⚠️ La **porta 102 richiede i
permessi root**; in caso contrario, impostare una porta alta (es. 1102) nel modale *Parametri*.

L'intestazione indica lo stato: **S7 ●** (verde) con l'indirizzo di ascolto, oppure un messaggio
di errore (rosso) se il bind fallisce. Un banner arancione avverte se il server è
**esposto** (tutte le interfacce + lista bianca vuota).

## 3. Interfaccia

- **Intestazione**: titolo, pulsanti *Parametri* / *Salva*, stato marcia/arresto, stato
  di ascolto S7, banner di esposizione di rete.
- **Pannello sinistro (Comandi)**: *Marcia/Arresto*, *Modalità automatica (PID)*,
  *Setpoint*, *Uscita manuale* (modalità manuale), regolazioni **PID** (Kp/Ki/Kd).
- **Pannello centrale**: schede *Misura / Setpoint / Uscita* + **curva** in tempo reale.
- **Modale *Parametri***: lingua, verifica degli aggiornamenti, **rete S7** (IP di ascolto,
  porta, **lista bianca** di IP — un modello per riga, `*` = jolly), **processo**
  (K, τ, ritardo, ambiente), **limiti del setpoint**. *Applica* riavvia l'ascolto se
  l'IP/porta cambia e salva il TOML.

## 4. Connettere un client S7

Il client si connette all'IP/porta del server. I valori **rack/slot** usuali
(0/1 o 0/2) funzionano: il server non impone alcun TSAP. Le grandezze sono in
**DB1** (vedi [`reference_s7.md`](reference_s7.md)): setpoint in `DB1.DBD0`, misura
in `DB1.DBD4`, marcia in `DB1.DBX16.0`, ecc.

## 5. FAQ

- **«Permission denied» all'avvio** → la porta 102 richiede i permessi root;
  usare una porta alta o avviare con i privilegi adeguati.
- **Il client non si connette** → verificare IP/porta, la **lista bianca**, il
  firewall. Provare rack/slot 0/1 poi 0/2.
- **Le mie scritture non hanno effetto** → solo gli offset pilotabili agiscono
  (setpoint, uscita manuale, marcia, auto); gli altri sono in sola lettura.
- **Dov'è il file di configurazione?** → `mock_ru_s7.toml` (directory corrente;
  sovrascrivibile con `MOCK_CONFIG`).
