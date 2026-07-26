# Manuale utente — Regolatore Sparkplug B (ORSE)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · **IT** · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

---

## 1. A cosa serve lo strumento

**ORSE** simula un'**unità di regolazione** di processo (PID + processo termico del
primo ordine) e pubblica il suo stato in **MQTT Sparkplug B**, come un **edge node**
che si connette a un **broker** ed espone metriche a uno SCADA. Serve a testare una
catena di acquisizione Sparkplug B (Ignition, Chariot, EMQX, Node-RED…) senza
hardware reale.

## 2. Prerequisito: un broker MQTT

Essendo ORSE un **client**, occorre un broker MQTT raggiungibile. In locale:

```bash
docker run -it --rm -p 1883:1883 eclipse-mosquitto
```

## 3. Avvio rapido

```bash
cargo run -p mock_bin_ru_sparkplugb        # IHM + edge node Sparkplug B
```

All'avvio, la IHM tenta di connettersi al broker (`localhost:1883` per impostazione
predefinita). L'intestazione indica lo stato: **Connesso** (verde) una volta pubblicato
il `NBIRTH`, oppure **Disconnesso** (rosso) con il motivo. Un banner arancione
**⚠ MQTT in chiaro** ricorda l'assenza di TLS.

## 4. Interfaccia

- **Intestazione**: titolo, pulsanti *Parametri* / *Salva*, stato marcia/arresto, stato
  di connessione Sparkplug B, banner TLS/chiaro.
- **Pannello sinistro (Comandi)**: *Marcia/Arresto*, *Modalità automatica (PID)*,
  *Setpoint*, *Uscita manuale* (modalità manuale), regolazioni **PID** (Kp/Ki/Kd).
- **Pannello centrale**: schede *Misura / Setpoint / Uscita* + **curva** in tempo reale.
- **Modale *Parametri***: lingua, verifica degli aggiornamenti, **Broker MQTT /
  Sparkplug B** (host, porta, client_id, group_id, edge_node_id, keepalive, TLS,
  utente/password, pubblicazione su cambiamento/periodica), **processo** (K, τ, ritardo,
  ambiente), **limiti del setpoint**. *Applica* riavvia la connessione e salva il TOML.

## 5. Pilotare da uno SCADA

Lo SCADA si sottoscrive a `spBv1.0/<group_id>/#` e riceve `NBIRTH` poi `NDATA`. Per
**comandare** il regolatore, pubblica un `NCMD` su
`spBv1.0/<group_id>/NCMD/<edge_node_id>` con le metriche pilotabili (`Setpoint`,
`Run`, `Auto`, `ManualOutput`) oppure `Node Control/Rebirth = true` per forzare una
rinascita. Dettagli: [`reference_sparkplugb.md`](reference_sparkplugb.md).

## 6. FAQ

- **«Disconnesso» in permanenza** → broker irraggiungibile: verificare `broker_host`/
  `broker_port`, il firewall, e che il broker sia in esecuzione.
- **Lo SCADA non vede nulla** → verificare il `group_id`/`edge_node_id` e la
  sottoscrizione `spBv1.0/<group>/#`; i payload sono **protobuf** (richiesto un
  decodificatore Sparkplug).
- **Le mie scritture NCMD vengono ignorate** → metrica non pilotabile o tipo errato
  (cfr. tabella delle metriche). Solo `Setpoint`/`Run`/`Auto`/`ManualOutput` e
  `Rebirth` sono accettate.
- **Dov'è il file di configurazione?** → `mock_ru_sparkplugb.toml` (directory corrente;
  sovrascrivibile con `MOCK_CONFIG`).
