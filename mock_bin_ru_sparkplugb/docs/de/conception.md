# Konzeption — Sparkplug-B-Regler (ORSE)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · **DE** · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

---

## 1. Überblick

ORSE übernimmt die Architektur der anderen CESAM-Lab-Instrumente: ein **synchrones,
testbares Geschäftsmodell** (PID-Regler + Prozess), gesteuert durch **`ractor`-Aktoren**
auf Tokio, sowie eine **`egui`-IHM**, die einen geteilten Schnappschuss liest. Lediglich
die **Transportschicht** ändert sich: hier ein **MQTT-Edge-Node nach Sparkplug B**
(ausgehender Client) statt eines Modbus-/OPC-UA-Servers.

```
        Command (cast)                      refresh bei jedem Schritt
IHM   ───────────────────►  SimulationActor ───────────────────►  SharedSnapshot (IHM)
NCMD (Broker) ───────────►  (Regulator)      ──────────────────►  SharedSnapshot (Veröffentlichung)
NBIRTH/NDATA (Broker) ◄──────────────────────  SharedSnapshot
```

## 2. Aktoren

- **`SimulationActor`** — besitzt den einzigen [`Regulator`]. Schleife mit festem
  Schritt (`Tick` alle 0,5 s); wendet die `Command` (IHM oder NCMD) an; veröffentlicht
  den Schnappschuss nach jeder Mutation. Identisch zu den anderen Instrumenten.
- **`SparkplugActor`** — besitzt den **MQTT-Client** (`rumqttc`) und führt den
  **Lebenszyklus von Sparkplug B** in einer dedizierten Tokio-Task aus (deren
  `JoinHandle` beim Stopp abgebrochen wird). Eine `Reconfigure`-Nachricht startet den
  Client neu, falls sich Broker/Anmeldedaten/TLS ändern.

## 3. Protokollschicht

[`sparkplug_node.rs`](../../src/sparkplug_node.rs) ist **rein und synchron** (keinerlei
Abhängigkeit zu tokio/rumqttc): Aufbau der **Topics**, Tabelle der **Metriken**,
Erzeugung der **Payloads** (`NBIRTH`/`NDATA`/`NDEATH`), (De-)Serialisierung in protobuf,
Abbildung **`NCMD` → Befehle** sowie der `seq`-Zähler. Es ist das Pendant zur
`opcua_server.rs` von ORUE, isoliert, um **ohne Broker testbar** zu sein.

### Auswahl der Bibliotheken

- **`rumqttc`** — asynchroner MQTT-Client auf Tokio (Last Will, automatische
  Wiederverbindung, TLS über rustls — bereits im Baum durch OPC UA, **ohne OpenSSL**).
- **`sparkplug-rs`** — protobuf-Structs von Eclipse Tahu (`Payload`/`Metric`/`Value`),
  in **100 % Rust** generiert (rust-protobuf, **kein `protoc`** → sauberes Cross). Die
  Crate re-exportiert `protobuf` (Runtime), genutzt für `write_to_bytes`/`parse_from_bytes`.
- **Verworfene Alternative: `srad`** — High-Level-Framework für Sparkplug-Edge-Nodes,
  das `bdSeq`/`seq`/rebirth selbst verwaltet. Bewusst verworfen: Wir **besitzen** die
  Zustandsmaschine im Netzwerkaktor, um sie explizit und testbar zu machen (Konsistenz
  mit den anderen Instrumenten).

## 4. Lebenszyklus & Invarianten

- **`bdSeq`** wird bei jedem (Neu-)Start des Clients inkrementiert; **derselbe** Wert
  im Last Will `NDEATH` und im `NBIRTH` einer Sitzung.
- **`seq`** umlaufend 0–255, bei jedem `NBIRTH` auf 0 zurückgesetzt.
- **`NDEATH`** getragen vom **MQTT Last Will**: robust gegen jeglichen Verbindungsverlust.
- **`NDATA`-Veröffentlichung** per **Differenz** des Schnappschusses (Takt = Simulationsschritt
  im Modus *bei Änderung* oder periodisch). Die Sperre des Schnappschusses wird **niemals**
  über ein `.await` hinweg gehalten.

## 5. Sicherheitslage

- **Keine IP-Whitelist** (das Instrument ist ein Client, kein Server): **bewusst**
  hingenommene Abweichung in der Parität gegenüber ORME/OSNE.
- **MQTT unverschlüsselt standardmäßig** (Port 1883) — nicht verschlüsselt, netzseitig
  nicht authentifiziert. Warnbanner in der IHM. **TLS** + Anmeldedaten aktivieren, um
  ein vertrauenswürdiges Netz zu verlassen.
- **Passwort im Klartext** in der TOML — **nur Simulator**.
- **TOML-Bereinigung** ([`AppConfig::sanitized`](../../src/config.rs)): Prozess/PID/Grenzen
  endlich und geordnet, Sparkplug-Kennungen nicht leer, Zeitgeber begrenzt. Jeder
  NCMD-Schreibvorgang wird von `Regulator::apply` **geklemmt/bereinigt**.
