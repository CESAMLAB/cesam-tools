# Progettazione — Regolatore S7 (ORSS)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · **IT** · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

---

## 1. Panoramica

ORSS riutilizza l'architettura degli altri strumenti CESAM-Lab: **modello di
dominio sincrono e testabile** (PID + processo), **attori `ractor`** su Tokio, **IHM
`egui`** che legge un'istantanea condivisa. Cambia solo il **livello di trasporto**: un
**server S7comm** (ISO-on-TCP / RFC1006) al posto di Modbus/OPC UA.

```
        Command (cast)                      refresh a ogni passo
IHM   ───────────────────►  SimulationActor ───────────────────►  SharedSnapshot (IHM)
S7 Write Var ────────────►  (Regulator)      ──────────────────►  SharedSnapshot
S7 Read Var  ◄────────────────────────────────  SharedSnapshot (immagine DB1)
```

## 2. Attori

- **`SimulationActor`** — possiede l'unico [`Regulator`]. Ciclo a passo fisso;
  applica i `Command` (IHM o scritture S7); pubblica l'istantanea dopo ogni
  mutazione.
- **`S7ServerActor`** — possiede il **ciclo di ascolto TCP**. Un task tokio dedicato
  associa il socket e accetta i client; ogni sessione è retta da un `JoinSet`
  **interno** (quindi terminata insieme al ciclo — nessun task scollegato). `Reconfigure`
  riavvia l'ascolto se l'IP/porta cambia e aggiorna la **lista bianca** condivisa.

## 3. Livello protocollo

[`s7_server.rs`](../../src/s7_server.rs) è **puro e sincrono** (nessuna dipendenza
di rete): framing TPKT, COTP (CR→CC, DT) e S7comm (Setup, Read Var, Write Var) su
un'**immagine di byte DB1**. Il parsing è **limitato** (accesso tramite `get`/slice
verificati): una trama malformata proveniente dalla rete non provoca **mai** un panic,
soltanto un'assenza di risposta. È l'equivalente S7 di `opcua_server.rs`, isolato
per essere **testabile senza socket**.

### Perché un server fatto a mano

Non esiste una libreria **server** S7 in Rust (le crate `s7`/`s7-comm` sono
orientate al **client**). Il sottoinsieme necessario (COTP classe 0 + S7 Read/
Write Var su un DB) è compatto e ben specificato: implementarlo a mano offre un
controllo totale e una superficie testabile, coerente con gli altri strumenti.

## 4. Politica delle sessioni

Più client S7 **simultanei** sono accettati (comportamento da PLC), al
contrario del mono-master di ORME (espulsione) e del punto-punto di OSNE (squat).
Ogni sessione legge l'immagine DB1 corrente e instrada le sue scritture verso la simulazione;
«l'ultimo che scrive vince», come un PLC reale.

## 5. Postura di sicurezza

- **Né autenticazione né cifratura** (S7 «classic»): solo la **lista bianca
  di IP** e la topologia di rete proteggono l'accesso. `0.0.0.0` + lista vuota = esposto →
  banner di avvertimento nell'IHM ([`NetworkConfig::is_exposed`](../../src/config.rs)).
- **Sanificazione TOML** ([`AppConfig::sanitized`](../../src/config.rs)): processo/
  PID/limiti finiti e ordinati. Ogni scrittura S7 è **limitata/sanificata** da
  `Regulator::apply`: la superficie di rete non può produrre né `NaN`/`Inf` né valori
  aberranti.
- **Parsing di rete limitato**: nessuna trama può provocare un panic (cfr. §3).
