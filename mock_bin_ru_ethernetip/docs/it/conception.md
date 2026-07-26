# Progettazione — Regolatore EtherNet/IP (OREE)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · **IT** · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

---

## 1. Panoramica

OREE riutilizza l'architettura degli altri strumenti CESAM-Lab: **modello di business
sincrono e testabile** (PID + processo), **attori `ractor`** su Tokio, **IHM
`egui`** che legge un'istantanea condivisa. Cambia solo il **livello di trasporto**: un
**adattatore EtherNet/IP** (incapsulamento + CIP) invece di Modbus/OPC UA/S7.

```
        Command (cast)                      refresh ad ogni passo
IHM   ───────────────────►  SimulationActor ───────────────────►  SharedSnapshot (IHM)
CIP Write Tag ───────────►  (Regulator)      ──────────────────►  SharedSnapshot
CIP Read Tag  ◄────────────────────────────────  SharedSnapshot
```

## 2. Attori

- **`SimulationActor`** — possiede l'unico [`Regulator`]; applica i `Command`
  (IHM o scritture CIP); pubblica l'istantanea dopo ogni mutazione.
- **`EipServerActor`** — possiede il **ciclo di ascolto TCP**. Un task tokio collega il
  socket e accetta i client; ogni sessione (con il proprio *session handle*) è
  gestita da un `JoinSet` **interno** (abbattuto insieme al ciclo — nessun task
  distaccato). `Reconfigure` riavvia l'ascolto se l'IP/porta cambia e aggiorna la
  **lista bianca** condivisa.

## 3. Livello protocollo

[`eip_server.rs`](../../src/eip_server.rs) è **puro e sincrono**: incapsulamento
EtherNet/IP (`RegisterSession`, `SendRRData`/CPF) e CIP (`Read Tag`/`Write Tag` per
segmento simbolico). Tutto è **little-endian**. Il parsing è **limitato** (slice
verificati): un pacchetto malformato proveniente dalla rete non provoca **mai** un panic,
solo l'assenza di risposta. È l'equivalente di `opcua_server.rs`, isolato per
essere **testabile senza socket**.

### Perché un adattatore fatto a mano

Non esiste alcuna libreria **server/adattatore** EtherNet/IP in Rust (le
crate `rseip`, `rust-ethernet-ip`, `cip` sono orientate al **client/scanner**). Il
sottoinsieme necessario (incapsulamento + CIP Read/Write Tag su tag nominati) è
compatto: implementarlo a mano offre un controllo totale e una superficie testabile,
coerente con gli altri strumenti.

## 4. Politica delle sessioni

Più client **simultanei** sono accettati (comportamento di un adattatore), al
contrario del mono-master di ORME. Ogni sessione riceve un *session handle* e legge
l'istantanea corrente; «l'ultimo che scrive vince».

## 5. Postura di sicurezza

- **Né autenticazione né cifratura** (EtherNet/IP «classic»): solo la **lista
  bianca di IP** e la topologia di rete proteggono l'accesso. `0.0.0.0` + lista vuota =
  esposto → banner di avvertimento ([`NetworkConfig::is_exposed`](../../src/config.rs)).
- **Sanificazione TOML** ([`AppConfig::sanitized`](../../src/config.rs)): processo/
  PID/limiti finiti e ordinati. Ogni scrittura CIP è **clampata/sanificata** da
  `Regulator::apply`: la superficie di rete non può produrre né `NaN`/`Inf` né valore
  aberrante.
- **Parsing di rete limitato**: nessun pacchetto può provocare un panic (cfr. §3).
