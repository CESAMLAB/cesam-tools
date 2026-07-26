# Ontwerp — S7-regelaar (ORSS)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · **NL** · [PL](../pl/conception.md)*

---

## 1. Overzicht

ORSS hergebruikt de architectuur van de andere CESAM-Lab-instrumenten: **synchroon
en testbaar bedrijfsmodel** (PID + proces), **`ractor`-actoren** op Tokio, **`egui`-GUI**
die een gedeelde momentopname leest. Alleen de **transportlaag** verandert: een
**S7comm-server** (ISO-on-TCP / RFC1006) in plaats van Modbus/OPC UA.

```
        Command (cast)                      refresh elke stap
GUI   ───────────────────►  SimulationActor ───────────────────►  SharedSnapshot (GUI)
S7 Write Var ────────────►  (Regulator)      ──────────────────►  SharedSnapshot
S7 Read Var  ◄────────────────────────────────  SharedSnapshot (image DB1)
```

## 2. Actoren

- **`SimulationActor`** — bezit de unieke [`Regulator`]. Lus met vaste stap; past de
  `Command`'s toe (GUI of S7-schrijfacties); publiceert de momentopname na elke
  mutatie.
- **`S7ServerActor`** — bezit de **TCP-luisterlus**. Een toegewijde tokio-taak bindt
  de socket en accepteert de clients; elke sessie wordt gedragen door een **interne**
  `JoinSet` (dus afgebroken samen met de lus — geen losgekoppelde taak). `Reconfigure`
  herstart het luisteren als het IP/de poort verandert en werkt de gedeelde
  **toelatingslijst** bij.

## 3. Protocollaag

[`s7_server.rs`](../../src/s7_server.rs) is **puur en synchroon** (geen
netwerkafhankelijkheid): framing TPKT, COTP (CR→CC, DT) en S7comm (Setup, Read Var,
Write Var) op een **byte-image van DB1**. Het parsen is **begrensd** (toegang via
gecontroleerde `get`/slices): een misvormd frame van het netwerk veroorzaakt **nooit**
een panic, slechts het uitblijven van een antwoord. Dit is het S7-equivalent van
`opcua_server.rs`, geïsoleerd om **zonder socket testbaar** te zijn.

### Waarom een zelfgemaakte server

Er bestaat geen **server**-bibliotheek voor S7 in Rust (de crates `s7`/`s7-comm` zijn
**client**-georiënteerd). De benodigde deelverzameling (COTP klasse 0 + S7 Read/Write
Var op een DB) is compact en goed gespecificeerd: deze zelf implementeren geeft
volledige controle en een testbaar oppervlak, consistent met de andere instrumenten.

## 4. Sessiebeleid

Meerdere **gelijktijdige** S7-clients worden geaccepteerd (gedrag van een PLC), in
tegenstelling tot de single-master van ORME (verdringing) en de punt-tot-punt van
OSNE (squat). Elke sessie leest de actuele image van DB1 en routeert haar
schrijfacties naar de simulatie; "wie het laatst schrijft, wint", zoals een echte PLC.

## 5. Beveiligingshouding

- **Geen authenticatie noch versleuteling** (S7 "classic"): alleen de
  **IP-toelatingslijst** en de netwerktopologie beschermen de toegang. `0.0.0.0` +
  lege lijst = blootgesteld → waarschuwingsbanner in de GUI
  ([`NetworkConfig::is_exposed`](../../src/config.rs)).
- **TOML-sanering** ([`AppConfig::sanitized`](../../src/config.rs)): proces/PID/grenzen
  eindig en geordend. Elke S7-schrijfactie wordt **geklemd/gesaneerd** door
  `Regulator::apply`: het netwerkoppervlak kan geen `NaN`/`Inf` noch afwijkende waarde
  produceren.
- **Begrensd netwerk-parsen**: geen enkel frame kan een panic veroorzaken (zie §3).
