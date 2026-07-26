# Ontwerp — EtherNet/IP-regelaar (OREE)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · **NL** · [PL](../pl/conception.md)*

---

## 1. Overzicht

OREE hergebruikt de architectuur van de andere CESAM-Lab-instrumenten: **synchroon
en testbaar bedrijfsmodel** (PID + proces), **`ractor`-actoren** op Tokio, **GUI
`egui`** die een gedeelde momentopname leest. Alleen de **transportlaag** verandert: een
**EtherNet/IP-adapter** (encapsulatie + CIP) in plaats van Modbus/OPC UA/S7.

```
        Command (cast)                      refresh elke stap
GUI   ───────────────────►  SimulationActor ───────────────────►  SharedSnapshot (GUI)
CIP Write Tag ───────────►  (Regulator)      ──────────────────►  SharedSnapshot
CIP Read Tag  ◄────────────────────────────────  SharedSnapshot
```

## 2. Actoren

- **`SimulationActor`** — bezit de unieke [`Regulator`]; past de `Command`'s toe
  (GUI of CIP-schrijfacties); publiceert de momentopname na elke mutatie.
- **`EipServerActor`** — bezit de **TCP-luisterlus**. Een tokio-taak bindt de
  socket en accepteert de clients; elke sessie (met zijn *session handle*) wordt
  gedragen door een **interne** `JoinSet` (afgebroken samen met de lus — geen losgekoppelde
  taak). `Reconfigure` herstart het luisteren als het IP/de poort verandert en werkt de
  gedeelde **toegangslijst** bij.

## 3. Protocollaag

[`eip_server.rs`](../../src/eip_server.rs) is **puur en synchroon**: encapsulatie
EtherNet/IP (`RegisterSession`, `SendRRData`/CPF) en CIP (`Read Tag`/`Write Tag` per
symbolisch segment). Alles is **little-endian**. Het parsen is **begrensd** (gecontroleerde
slices): een misvormd pakket van het netwerk veroorzaakt **nooit** een panic,
slechts het uitblijven van een antwoord. Dit is het equivalent van `opcua_server.rs`, geïsoleerd om
**testbaar zonder socket** te zijn.

### Waarom een zelfgemaakte adapter

Er bestaat geen **server-/adapterbibliotheek** EtherNet/IP in Rust (de
crates `rseip`, `rust-ethernet-ip`, `cip` zijn gericht op **client/scanner**). De
benodigde subset (encapsulatie + CIP Read/Write Tag op benoemde tags) is
compact: hem met de hand implementeren geeft volledige controle en een testbaar oppervlak,
coherent met de andere instrumenten.

## 4. Sessiebeleid

Meerdere **gelijktijdige** clients worden geaccepteerd (gedrag van een adapter), in
tegenstelling tot het mono-master van ORME. Elke sessie ontvangt een *session handle* en leest
de huidige momentopname; "de laatste die schrijft, wint".

## 5. Beveiligingshouding

- **Geen authenticatie noch versleuteling** (EtherNet/IP "classic"): alleen de
  **IP-toegangslijst** en de netwerktopologie beschermen de toegang. `0.0.0.0` + lege lijst =
  blootgesteld → waarschuwingsbanner ([`NetworkConfig::is_exposed`](../../src/config.rs)).
- **TOML-ontsmetting** ([`AppConfig::sanitized`](../../src/config.rs)): proces/
  PID/grenzen eindig en geordend. Elke CIP-schrijfactie wordt **geklemd/ontsmet** door
  `Regulator::apply`: het netwerkoppervlak kan noch `NaN`/`Inf` noch een afwijkende
  waarde produceren.
- **Begrensd netwerkparsen**: geen enkel pakket kan een panic veroorzaken (zie §3).
