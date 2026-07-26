# Ontwerp — Sparkplug B-regelaar (ORSE)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · **NL** · [PL](../pl/conception.md)*

---

## 1. Overzicht

ORSE hergebruikt de architectuur van de andere CESAM-Lab-instrumenten: een **synchroon
en testbaar businessmodel** (PID-regelaar + proces), aangestuurd door **`ractor`-actoren**
op Tokio, en een **`egui`-GUI** die een gedeelde momentopname leest. Alleen de
**transportlaag** verandert: hier een **Sparkplug B MQTT edge node** (uitgaande client)
in plaats van een Modbus/OPC UA-server.

```
        Command (cast)                      refresh elke stap
GUI   ───────────────────►  SimulationActor ───────────────────►  SharedSnapshot (GUI)
NCMD (broker) ───────────►  (Regulator)      ──────────────────►  SharedSnapshot (publicatie)
NBIRTH/NDATA (broker) ◄──────────────────────  SharedSnapshot
```

## 2. Actoren

- **`SimulationActor`** — bezit de enige [`Regulator`]. Lus met vaste stap (`Tick`
  elke 0,5 s); past de `Command` toe (GUI of NCMD); publiceert de momentopname
  na elke mutatie. Identiek aan de andere instrumenten.
- **`SparkplugActor`** — bezit de **MQTT-client** (`rumqttc`) en voert de
  **Sparkplug B-levenscyclus** uit in een toegewijde tokio-taak (waarvan de `JoinHandle`
  bij het afsluiten wordt afgebroken). Een `Reconfigure`-bericht herstart de client als
  de broker/de identificatiegegevens/TLS veranderen.

## 3. Protocollaag

[`sparkplug_node.rs`](../../src/sparkplug_node.rs) is **puur en synchroon** (geen
tokio/rumqttc-afhankelijkheid): opbouw van de **topics**, tabel van **metrics**,
fabricage van de **payloads** (`NBIRTH`/`NDATA`/`NDEATH`), protobuf-(de)serialisatie,
mapping **`NCMD` → commando's**, en de `seq`-teller. Dit is het equivalent van de
`opcua_server.rs` van ORUE, geïsoleerd om **zonder broker testbaar** te zijn.

### Keuze van de bibliotheken

- **`rumqttc`** — async MQTT-client op Tokio (Last Will, automatische herverbinding, TLS
  via rustls — al in de boom aanwezig via OPC UA, **zonder OpenSSL**).
- **`sparkplug-rs`** — protobuf-structs van Eclipse Tahu (`Payload`/`Metric`/`Value`),
  gegenereerd in **100 % Rust** (rust-protobuf, **geen `protoc`** → schone cross). De
  crate her-exporteert `protobuf` (runtime), gebruikt voor `write_to_bytes`/`parse_from_bytes`.
- **Verworpen alternatief: `srad`** — high-level Sparkplug edge node-framework dat
  `bdSeq`/`seq`/rebirth zelf beheert. Bewust verworpen: we **bezitten** de toestandsmachine
  in de netwerkactor om hem expliciet en testbaar te maken (consistentie met de andere
  instrumenten).

## 4. Levenscyclus & invarianten

- **`bdSeq`** wordt verhoogd bij elke (her)start van de client; **dezelfde** waarde in de
  Last Will `NDEATH` en de `NBIRTH` van een sessie.
- **`seq`** rollend 0–255, teruggezet op 0 bij elke `NBIRTH`.
- **`NDEATH`** gedragen door de **MQTT Last Will**: robuust tegen elk verlies van de verbinding.
- **`NDATA`-publicatie** via **diff** van de momentopname (cadans = simulatiestap in
  *bij-wijziging*-modus, of periodiek). Het slot van de momentopname wordt **nooit**
  vastgehouden over een `.await`.

## 5. Beveiligingshouding

- **Geen IP-witte lijst** (het instrument is een client, geen server): pariteitsverschil
  **bewust aanvaard** met ORME/OSNE.
- **MQTT onversleuteld standaard** (poort 1883) — niet versleuteld, niet netwerkgeauthenticeerd.
  Waarschuwingsbanner in de GUI. Schakel **TLS** + identificatiegegevens in om buiten een
  vertrouwd netwerk te treden.
- **Wachtwoord in leesbare tekst** in de TOML — **alleen simulator**.
- **TOML-sanering** ([`AppConfig::sanitized`](../../src/config.rs)): proces/PID/grenzen
  eindig en geordend, Sparkplug-identificatiegegevens niet leeg, time-outs begrensd.
  Elke NCMD-schrijfbewerking wordt **geklemd/gesaneerd** door `Regulator::apply`.
