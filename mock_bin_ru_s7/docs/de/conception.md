# Konzeption — S7-Regler (ORSS)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · **DE** · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

---

## 1. Überblick

ORSS übernimmt die Architektur der übrigen CESAM-Lab-Instrumente: **synchrones und
testbares Geschäftsmodell** (PID + Prozess), **`ractor`-Aktoren** auf Tokio, **IHM
`egui`**, die einen geteilten Schnappschuss liest. Lediglich die **Transportschicht**
ändert sich: ein **S7comm-Server** (ISO-on-TCP / RFC1006) anstelle von Modbus/OPC UA.

```
        Command (cast)                      refresh bei jedem Schritt
IHM   ───────────────────►  SimulationActor ───────────────────►  SharedSnapshot (IHM)
S7 Write Var ────────────►  (Regulator)      ──────────────────►  SharedSnapshot
S7 Read Var  ◄────────────────────────────────  SharedSnapshot (DB1-Abbild)
```

## 2. Aktoren

- **`SimulationActor`** — besitzt den einzigen [`Regulator`]. Schleife mit festem
  Schritt; wendet die `Command` an (IHM oder S7-Schreibvorgänge); veröffentlicht den
  Schnappschuss nach jeder Mutation.
- **`S7ServerActor`** — besitzt die **TCP-Lauschschleife**. Eine dedizierte Tokio-Task
  bindet den Socket und nimmt die Clients an; jede Sitzung wird von einem **internen**
  `JoinSet` getragen (also mit der Schleife abgebrochen — keine losgelöste Task).
  `Reconfigure` startet das Lauschen neu, wenn sich IP/Port ändert, und aktualisiert die
  geteilte **Whitelist**.

## 3. Protokollschicht

[`s7_server.rs`](../../src/s7_server.rs) ist **rein und synchron** (keine
Netzabhängigkeit): TPKT-Framing, COTP (CR→CC, DT) und S7comm (Setup, Read Var, Write
Var) auf einem **DB1-Bytes-Abbild**. Das Parsing ist **begrenzt** (Zugriff über
geprüfte `get`/Slices): ein fehlerhaftes Telegramm aus dem Netz verursacht **niemals**
eine Panik, sondern nur eine ausbleibende Antwort. Es ist das S7-Pendant zu
`opcua_server.rs`, isoliert, um **ohne Socket testbar** zu sein.

### Warum ein selbst geschriebener Server

Es gibt keine S7-**Server**-Bibliothek in Rust (die Crates `s7`/`s7-comm` sind
**Client**-orientiert). Die benötigte Teilmenge (COTP Klasse 0 + S7 Read/Write Var auf
einem DB) ist kompakt und gut spezifiziert: sie von Hand zu implementieren gibt
vollständige Kontrolle und eine testbare Oberfläche, kohärent mit den übrigen
Instrumenten.

## 4. Sitzungspolitik

Mehrere **gleichzeitige** S7-Clients werden angenommen (Verhalten einer
speicherprogrammierbaren Steuerung), im Gegensatz zum Single-Master von ORME
(Verdrängung) und der Punkt-zu-Punkt-Verbindung von OSNE (Squat). Jede Sitzung liest
das aktuelle DB1-Abbild und leitet ihre Schreibvorgänge an die Simulation weiter; „der
zuletzt Schreibende gewinnt“, wie bei einer echten Steuerung.

## 5. Sicherheitslage

- **Weder Authentifizierung noch Verschlüsselung** (S7 „classic“): nur die
  **IP-Whitelist** und die Netztopologie schützen den Zugang. `0.0.0.0` + leere Liste =
  exponiert → Warnbanner in der IHM
  ([`NetworkConfig::is_exposed`](../../src/config.rs)).
- **TOML-Bereinigung** ([`AppConfig::sanitized`](../../src/config.rs)): Prozess/PID/
  Grenzen endlich und geordnet. Jeder S7-Schreibvorgang wird von `Regulator::apply`
  **geklemmt/bereinigt**: die Netzoberfläche kann weder `NaN`/`Inf` noch einen
  abwegigen Wert erzeugen.
- **Begrenztes Netz-Parsing**: kein Telegramm kann eine Panik auslösen (siehe §3).
