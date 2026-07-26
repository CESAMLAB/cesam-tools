# Progettazione — Regolatore Sparkplug B (ORSE)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · **IT** · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

---

## 1. Panoramica

ORSE riutilizza l'architettura degli altri strumenti CESAM-Lab: un **modello di
dominio sincrono e testabile** (regolatore PID + processo), pilotato da **attori
`ractor`** su Tokio, e una **IHM `egui`** che legge un'istantanea condivisa. Cambia
solo lo **strato di trasporto**: qui, un **edge node MQTT Sparkplug B** (client in
uscita) invece di un server Modbus/OPC UA.

```
        Command (cast)                      refresh a ogni passo
IHM   ───────────────────►  SimulationActor ───────────────────►  SharedSnapshot (IHM)
NCMD (broker) ───────────►  (Regulator)      ──────────────────►  SharedSnapshot (pubblicazione)
NBIRTH/NDATA (broker) ◄──────────────────────  SharedSnapshot
```

## 2. Attori

- **`SimulationActor`** — possiede l'unico [`Regulator`]. Ciclo a passo fisso (`Tick`
  ogni 0,5 s); applica i `Command` (IHM o NCMD); pubblica l'istantanea dopo ogni
  mutazione. Identico agli altri strumenti.
- **`SparkplugActor`** — possiede il **client MQTT** (`rumqttc`) ed esegue il **ciclo
  di vita Sparkplug B** in un task tokio dedicato (il cui `JoinHandle` viene abbattuto
  all'arresto). Un messaggio `Reconfigure` riavvia il client se cambiano il broker/le
  credenziali/il TLS.

## 3. Strato protocollo

[`sparkplug_node.rs`](../../src/sparkplug_node.rs) è **puro e sincrono** (nessuna
dipendenza tokio/rumqttc): costruzione dei **topic**, tabella delle **metriche**,
fabbricazione dei **payload** (`NBIRTH`/`NDATA`/`NDEATH`), (de)serializzazione
protobuf, mapping **`NCMD` → comandi**, e il contatore `seq`. È l'equivalente del
`opcua_server.rs` di ORUE, isolato per essere **testabile senza broker**.

### Scelta delle librerie

- **`rumqttc`** — client MQTT async Tokio (Last Will, riconnessione automatica, TLS
  via rustls — già presente nell'albero tramite OPC UA, **senza OpenSSL**).
- **`sparkplug-rs`** — struct protobuf Eclipse Tahu (`Payload`/`Metric`/`Value`),
  generate al **100 % in Rust** (rust-protobuf, **nessun `protoc`** → cross pulito).
  La crate riesporta `protobuf` (runtime), usato per `write_to_bytes`/`parse_from_bytes`.
- **Alternativa scartata: `srad`** — framework di alto livello per edge node Sparkplug
  che gestisce esso stesso `bdSeq`/`seq`/rebirth. Scartato volontariamente: si
  **possiede** la macchina a stati nell'attore di rete per renderla esplicita e
  testabile (coerenza con gli altri strumenti).

## 4. Ciclo di vita e invarianti

- **`bdSeq`** incrementato a ogni (ri)avvio del client; **stesso** valore nel
  Last Will `NDEATH` e nel `NBIRTH` di una sessione.
- **`seq`** rotante 0–255, riportato a 0 a ogni `NBIRTH`.
- **`NDEATH`** veicolato dal **Last Will MQTT**: robusto a qualsiasi perdita di
  collegamento.
- **Pubblicazione `NDATA`** per **diff** di istantanea (cadenza = passo di simulazione
  in modalità *su cambiamento*, oppure periodica). Il lock dell'istantanea non viene
  **mai** mantenuto attraverso un `.await`.

## 5. Postura di sicurezza

- **Nessuna whitelist di IP** (lo strumento è un client, non un server): scostamento
  di parità **assunto** con ORME/OSNE.
- **MQTT in chiaro per impostazione predefinita** (porta 1883) — non cifrato, non
  autenticato a livello di rete. Banner di avviso nella IHM. Attivare **TLS** +
  credenziali per uscire da una rete fidata.
- **Password in chiaro** nel TOML — **solo simulatore**.
- **Sanificazione TOML** ([`AppConfig::sanitized`](../../src/config.rs)): processo/
  PID/limiti finiti e ordinati, identificatori Sparkplug non vuoti, temporizzazioni
  limitate. Qualsiasi scrittura NCMD è **clampata/sanificata** da `Regulator::apply`.
