# Manuale utente — Regolatore EtherNet/IP (OREE)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · **IT** · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

---

## 1. A cosa serve lo strumento

**OREE** simula un'**unità di regolazione** di processo (PID + processo termico del
primo ordine) e la espone come un **adattatore EtherNet/IP** (messaggistica esplicita
CIP). Serve a testare una supervisione o un client EtherNet/IP (pycomm3, RSLinx in
lettura, rseip…) senza hardware reale.

## 2. Primi passi

```bash
cargo run -p mock_bin_ru_ethernetip        # IHM + adattatore EtherNet/IP
```

Il server ascolta di default su `0.0.0.0:44818` (nessun privilegio richiesto). L'intestazione
indica lo stato: **EtherNet/IP ●** (verde) con l'indirizzo di ascolto, oppure un messaggio
di errore (rosso). Un banner arancione avverte se il server è **esposto** (tutte le
interfacce + lista bianca vuota).

## 3. Interfaccia

- **Intestazione**: titolo, pulsanti *Parametri* / *Salva*, stato avvio/arresto, stato
  di ascolto EtherNet/IP, banner di esposizione di rete.
- **Pannello sinistro (Comandi)**: *Avvio/Arresto*, *Modalità automatica (PID)*,
  *Setpoint*, *Uscita manuale* (modalità manuale), regolazioni **PID** (Kp/Ki/Kd).
- **Pannello centrale**: schede *Misura / Setpoint / Uscita* + **curva** in tempo reale.
- **Modale *Parametri***: lingua, verifica degli aggiornamenti, **rete EtherNet/IP** (IP
  di ascolto, porta, **lista bianca** di IP — un motivo per riga, `*` = jolly),
  **processo** (K, τ, ritardo, ambiente), **limiti del setpoint**. *Applica* riavvia
  l'ascolto se l'IP/porta cambia e salva il TOML.

## 4. Connettere un client EtherNet/IP

Il client si connette all'IP/porta del server (`RegisterSession` automatico), poi
legge/scrive i **tag nominati** tramite messaggistica esplicita: `Setpoint`, `ProcessValue`,
`Output`, `ManualOutput`, `Run`, `Auto`, ecc. (vedere
[`reference_ethernetip.md`](reference_ethernetip.md)). ⚠️ I valori sono in
**little-endian** (REAL = `f32` LE).

## 5. FAQ

- **Il client non si connette** → verificare IP/porta (44818), la **lista bianca**,
  il firewall.
- **Tag introvabile** → esistono solo i tag documentati; i nomi sono
  sensibili alle maiuscole/minuscole.
- **Le mie scritture non hanno effetto** → agiscono solo i tag pilotabili
  (`Setpoint`, `ManualOutput`, `Run`, `Auto`); gli altri sono in sola lettura.
- **Dov'è il file di configurazione?** → `mock_ru_ethernetip.toml` (directory corrente;
  sovrascrivibile con `MOCK_CONFIG`).
