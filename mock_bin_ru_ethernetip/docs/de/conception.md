# Konzeption — EtherNet/IP-Regler (OREE)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · **DE** · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

---

## 1. Überblick

OREE verwendet die Architektur der anderen CESAM-Lab-Instrumente wieder: **synchrones
und testbares Fachmodell** (PID + Prozess), **`ractor`-Aktoren** auf Tokio, **`egui`-IHM**,
die einen gemeinsam genutzten Schnappschuss liest. Nur die **Transportschicht** ändert
sich: ein **EtherNet/IP-Adapter** (Encapsulation + CIP) anstelle von Modbus/OPC UA/S7.

```
        Command (cast)                      refresh bei jedem Schritt
IHM   ───────────────────►  SimulationActor ───────────────────►  SharedSnapshot (IHM)
CIP Write Tag ───────────►  (Regulator)      ──────────────────►  SharedSnapshot
CIP Read Tag  ◄────────────────────────────────  SharedSnapshot
```

## 2. Aktoren

- **`SimulationActor`** — besitzt den einzigen [`Regulator`]; wendet die `Command`
  (IHM oder CIP-Schreibvorgänge) an; veröffentlicht den Schnappschuss nach jeder
  Mutation.
- **`EipServerActor`** — besitzt die **TCP-Lauschschleife**. Eine Tokio-Task bindet den
  Socket und nimmt die Clients an; jede Sitzung (mit ihrem *session handle*) wird von
  einem **internen** `JoinSet` getragen (mit der Schleife abgebrochen — keine
  abgekoppelte Task). `Reconfigure` startet das Lauschen neu, wenn sich IP/Port ändert,
  und aktualisiert die gemeinsam genutzte **Whitelist**.

## 3. Protokollschicht

[`eip_server.rs`](../../src/eip_server.rs) ist **rein und synchron**: EtherNet/IP-Encapsulation
(`RegisterSession`, `SendRRData`/CPF) und CIP (`Read Tag`/`Write Tag` per symbolischem
Segment). Alles ist **little-endian**. Das Parsing ist **begrenzt** (geprüfte Slices):
ein fehlerhaftes Paket aus dem Netzwerk verursacht **niemals** eine Panik, sondern nur
eine ausbleibende Antwort. Es ist das Äquivalent zu `opcua_server.rs`, isoliert, um
**ohne Socket testbar** zu sein.

### Warum ein selbst gebauter Adapter

Es existiert keine **Server-/Adapter**-Bibliothek für EtherNet/IP in Rust (die Crates
`rseip`, `rust-ethernet-ip`, `cip` sind **Client-/Scanner**-orientiert). Die benötigte
Teilmenge (Encapsulation + CIP Read/Write Tag auf benannten Tags) ist kompakt: sie
von Hand zu implementieren gibt volle Kontrolle und eine testbare Oberfläche, im
Einklang mit den anderen Instrumenten.

## 4. Sitzungsrichtlinie

Mehrere **gleichzeitige** Clients werden angenommen (Verhalten eines Adapters), im
Gegensatz zum Single-Master von ORME. Jede Sitzung erhält ein *session handle* und
liest den aktuellen Schnappschuss; „wer zuletzt schreibt, gewinnt“.

## 5. Sicherheitslage

- **Weder Authentifizierung noch Verschlüsselung** (EtherNet/IP „classic“): nur die
  **IP-Whitelist** und die Netztopologie schützen den Zugang. `0.0.0.0` + leere Liste =
  exponiert → Warnbanner ([`NetworkConfig::is_exposed`](../../src/config.rs)).
- **TOML-Bereinigung** ([`AppConfig::sanitized`](../../src/config.rs)): Prozess-/
  PID-/Grenzwerte endlich und geordnet. Jeder CIP-Schreibvorgang wird durch
  `Regulator::apply` **geklemmt/bereinigt**: die Netzwerkoberfläche kann weder `NaN`/`Inf`
  noch einen abweichenden Wert erzeugen.
- **Begrenztes Netzwerk-Parsing**: kein Paket kann eine Panik verursachen (vgl. §3).
