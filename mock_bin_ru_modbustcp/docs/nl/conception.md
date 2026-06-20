# Ontwerpdocument — Gesimuleerde Modbus TCP-regelaar

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · **NL** · [PL](../pl/conception.md)*

> Product: **ORME** · Crate: `mock_bin_ru_modbustcp` · Workspace: `cesam-tools` · Licentie: MIT

Dit document beschrijft de architectuur, de technische keuzes en de
werkingsprincipes van de gesimuleerde industriële regelaar. Het is bedoeld voor
ontwikkelaars die het project onderhouden of uitbreiden.

---

## 1. Doel en reikwijdte

Het leveren van een **virtueel industrieel instrument**: een procesregelaar die
zich realistisch gedraagt en communiceert via **Modbus TCP** (slave), om
supervisors / PLC's / gateways te ontwikkelen en te testen **zonder hardware**.

De simulator omvat:

- een **fysiek proces** gemodelleerd door een overdrachtsfunctie;
- een **tweerichtingsregeling** (warm / koud): PID, aan-uit (TOR) of
  cyclusrelais (PWM);
- een **Modbus TCP-interface** die de volledige toestand blootstelt;
- een **GUI** voor besturing, visualisatie en parametrering;
- de **persistentie** van de parameters.

Buiten de huidige reikwijdte: Modbus RTU, redundantie, langetermijnhistoriek,
sterke authenticatie (alleen een IP-witte lijst wordt geleverd).

---

## 2. Overzicht

```
┌──────────────────────────────────────────────────────────────────────┐
│                       Proces (hoofdthread)                             │
│                                                                        │
│   ┌─────────────────────────┐         leest (Mutex)                    │
│   │   GUI  egui / eframe      │◄──────────────── SharedSnapshot         │
│   │   (gui.rs)               │◄──────────────── SharedStatus           │
│   └───────────┬─────────────┘                                          │
│               │ cast (niet-blokkerend)                                 │
└───────────────┼────────────────────────────────────────────────────────┘
                │
   ┌────────────┼──────────── Tokio-runtime (achtergrondthreads) ────────┐
   │            ▼                                                         │
   │   ┌──────────────────┐  refresh  ┌──────────────┐                   │
   │   │ SimulationActor   ├──────────►│ SharedSnapshot│ (GUI)            │
   │   │  (ractor)         ├──────────►│ SharedMap     │ (Modbus)         │
   │   │  bezit de          │           └──────┬───────┘                  │
   │   │  Regulator         │◄── Command ──┐    │ leest                   │
   │   └──────────────────┘              │    ▼                          │
   │          ▲ Command (cast)            │  ┌──────────────────────┐     │
   │          │                           └──┤ RegulatorService      │     │
   │   ┌──────┴───────────┐  beheert/rebind  │ (trait Service)       │     │
   │   │ ModbusServerActor ├─────────────────►  Modbus TCP-server    │◄──── clients
   │   │  (ractor)         │  IP-filter ──────► (tokio-modbus)        │     │
   │   └──────────────────┘   (SharedAllowlist)└──────────────────────┘     │
   └────────────────────────────────────────────────────────────────────┘
```

Leidend principe: **één enkele eigenaar van de bedrijfstoestand**. De `Regulator`
wordt nooit gedeeld; hij leeft in `SimulationActor`. Alle schrijfacties (GUI of
Modbus) zijn `Command`-**berichten**. Leesacties gebeuren op **kopieën** die bij
elke stap worden ververst (`SharedSnapshot`, `SharedMap`), wat sloten op de logica
en race-condities elimineert.

---

## 3. Technische keuzes

| Behoefte | Keuze | Verantwoording |
|----------|-------|----------------|
| Gelijktijdigheid | **`ractor`** (actoren) op **Tokio** | Isoleert de muteerbare toestand in een actor; mutaties geserialiseerd per bericht, zonder applicatieslot. Projectvoorkeur. |
| Modbus TCP-slave | **`tokio-modbus`** (`tcp-server`) | Volwassen async-implementatie; de trait `Service` mapt verzoek→antwoord netjes. |
| GUI | **`egui` / `eframe`** + `egui_plot` | Immediate mode, platformonafhankelijk, zonder complexe UI-toestand om te synchroniseren. |
| Proces | **FOPDT** (1e orde + dode tijd) | Standaardmodel, volstaat voor een thermisch proces; weinig parameters, intuïtief. |
| Persistentie | **`serde` + `toml`** | Leesbaar/handmatig bewerkbaar formaat, ideaal voor apparaatparameters. |

### Waarom synchrone en asynchrone logica scheiden

`mock_lib_control` en `regulator.rs` zijn **zuiver synchroon** (geen IO, geen
async). Voordelen: deterministisch unit-testbaar, herbruikbaar door andere
instrumenten en goed te begrijpen bij het lezen. Het asynchrone deel blijft
beperkt tot de **actoren** en de **netwerklaag**.

---

## 4. Datamodel

### Bedrijfstoestand (`regulator.rs`)

- `Regulator` — bezittende aggregaat: modi, setpoints, regelaars (`Pid`,
  `OnOff`) en proces (`FirstOrderProcess`). Niet `Clone`, niet gedeeld.
- `RegulatorConfig` — statische configuratie (proces, gains, grenzen, `dt`).
  **Enige bron** van de standaardwaarden (de TOML-config is hiervan afgeleid).
- `RegulatorSnapshot` — **onveranderlijke kopie** (`Copy`) van de waarneembare
  toestand, bij elke stap gepubliceerd. Dit is het leescontract voor de GUI en de
  Modbus-tabel.
- `Command` — opsomming van de mogelijke mutaties (aan/uit, modus, setpoints,
  instellingen, proces, grenzen).

### Gedeelde structuren (`actors/mod.rs`, `config.rs`)

| Type | Inhoud | Geschreven door | Gelezen door |
|------|--------|-----------------|--------------|
| `SharedSnapshot` | getypeerde `RegulatorSnapshot` | SimulationActor | GUI |
| `SharedMap` | `MemoryMap` (beelden van de 4 Modbus-tabellen) | SimulationActor | RegulatorService |
| `SharedAllowlist` | `IpFilter` | ModbusServerActor | acceptatie verbindingen |
| `SharedStatus` | `ServerStatus` (luisteren / fout) | ModbusServerActor | GUI |

Allemaal `Arc<Mutex<…>>`: **korte** kritieke secties (kopie / refresh), nooit
vastgehouden tijdens een berekening of een IO.

---

## 5. Componenten

### 5.1 `mock_lib_control` (bibliotheek)

- `Pid` — PID met discrete tijd, afgeleide op de fout, **anti-windup** door
  begrenzing van de integrale term. API: `step(sp, pv, dt)` of
  `step_with_error(err, dt)` (hergebruikt voor de koude richting).
- `OnOff` — aan-uit met **symmetrische hysterese** (dode zone) **en
  anti-kortsluitcyclus**: een minimale cyclustijd (`min_cycle`, s) verbiedt elke
  omschakeling zolang het relais niet lang genoeg in zijn toestand is gebleven, wat
  de bescherming van een echte aandrijver modelleert. Het relais **vergrendelt**
  zijn toestand: het is aan de aanroeper om het de getekende fout door te geven
  zonder het te resetten bij tekenwisseling (zie § 5.2).
- `Pwm` — pulsbreedtemodulator (**cyclusrelais** / *time-proportioning*): over een
  vaste periode `T_c` is de aan-uit-uitgang actief gedurende de fractie `duty` van
  de cyclus (`duty` **één keer per cyclus bemonsterd** om een afwijking in
  ingeschakelde toestand te voorkomen). Maakt het mogelijk een aan-uit-orgaan fijn
  te regelen.
- `FirstOrderProcess` — overdrachtsfunctie `K·e^(-L·s)/(1+T·s)`, Euler-integratie
  + vertragingslijn. `reconfigure(...)` wijzigt de parameters zonder sprong.
- `ControllerKind` — `Off` / `Pid` / `OnOff` / `Pwm`, met Modbus-codering
  (`to_code`/`from_code`).

### 5.2 `regulator.rs`

Orkestratie van de regeling bij elke stap (`step`):

1. indien **gestopt** → uitgang 0, regelaars gereset;
2. indien **handmatig** → uitgang = handmatige setpoint (% met teken);
3. indien **auto** → men berekent **afzonderlijk** de bijdrage van de warme
   richting (richting 1, fout `SP − PV`) en van de koude richting (richting 2, fout
   `PV − SP`), elk ≥ 0, dan `uitgang = warm − koud`:
   - **PID**: uitgang begrensd tot `[0, 100]` (`out_min = 0`) — de inactieve
     richting (negatieve fout) levert 0 op en haar integraal **leegt zich op
     natuurlijke wijze** door begrenzing. We zetten hem **niet** geforceerd op nul:
     met de sterke rimpel van de PWM zou hem bij elke overschrijding van het
     setpoint wissen een statische fout introduceren;
   - **TOR**: het relais wordt geëvalueerd op de getekende fout en behoudt zijn
     toestand bij het passeren van het setpoint, wat een **symmetrische**
     hysteresisband `[SP − h/2, SP + h/2]` herstelt (de warme/koude banden blijven
     disjunct, dus de twee relais zijn wederzijds uitsluitend);
   - **PWM**: een PID berekent de cyclusverhouding, gemoduleerd door het
     cyclusrelais; de fysieke uitgang is strikt 0 % of 100 %, maar het gemiddelde
     volgt de PID.
4. de uitgang stuurt het proces aan dat de nieuwe meting (PV) produceert.

> **Historiek**: vóór deze herziening gebeurde de warm/koud-omschakeling op basis
> van het teken van de fout en **reset** het TOR-relais bij het passeren van het
> setpoint — wat de hysterese tot `[SP − h/2, SP]` afkapte (halve band,
> asymmetrisch) en de TOR-regeling middelmatig maakte. De berekening per
> afzonderlijke richting corrigeert dit gebrek.

### 5.3 `actors/simulation.rs`

`SimulationActor` (ractor). `pre_start` wapent een `send_interval(dt)` die `Tick`
uitzendt. `handle` verwerkt `Tick` (de simulatie vooruit) en `Command` (past een
mutatie toe), en **publiceert** dan de toestand in `SharedSnapshot` en `SharedMap`.

### 5.4 `actors/network.rs`

`ModbusServerActor` bezit de Modbus-server. `Reconfigure(NetworkConfig)`:
- werkt de gedeelde **witte lijst** bij (onmiddellijk effect, zonder herstart);
- als het **transport** (TCP/RTU), de **poort / IP** of de **seriële parameters**
  wijzigen, **stopt** de servertaak en **herstart** ze (`start_tcp` of `start_rtu`);
  publiceert de toestand in `SharedStatus` (succes of fout).

**Eén enkel transport** is tegelijk actief (`Transport::Tcp` of `Rtu`). RTU zit
achter de **feature `rtu`**; zonder deze publiceert het selecteren van RTU een
expliciete statusfout.

### 5.5 `modbus_server.rs`

`RegulatorService` implementeert `tokio_modbus::server::Service` op **synchrone**
wijze (`future::Ready`): leesacties = uitsnijden van `SharedMap`; schrijfacties =
decoderen naar `Command` (via `map.rs`) gevolgd door een `cast` naar
`SimulationActor`.

**Single-master-beleid.** `serve` (TCP) staat **slechts één externe master
tegelijk** toe: bij elke nieuwe verbinding (IP toegestaan door de witte lijst)
wordt de vorige gesloten. Mechanisme: de `TcpStream` wordt verpakt in een
`CancellableStream` die, bij ontvangst van een `oneshot`-signaal, **EOF bij het
lezen** teruggeeft — de verwerkingslus van `tokio-modbus` eindigt dan en sluit de
socket. `serve_rtu` (feature `rtu`) bedient de seriële bus via
`rtu::Server::serve_forever`: de RS485-bus *is* de enige master (niets te
verdringen).

> ⚠️ De GUI gebruikt dit pad niet: ze stuurt haar `Command`'s rechtstreeks naar de
> actor en wordt dus nooit als master geteld.
>
> ⚠️ De RTU-server van `tokio-modbus` 0.17 geeft het slave-adres niet door aan de
> service: het apparaat antwoordt dus ongeacht het gevraagde adres. Een
> **punt-tot-punt**-verbinding wordt aanbevolen. `slave_id` wordt bewaard en
> weergegeven, maar niet gebruikt om te filteren (beperking stroomopwaarts).

### 5.6 `map.rs`

**Bron van waarheid** van het Modbus-adresseringsplan. Adresconstanten,
`MemoryMap` (beelden van de tabellen), `refresh_from(snapshot)` (toestand→registers)
en `*_to_command(s)` (schrijfacties→commando's). Codering van de `f32`'s op 2
registers, big-endian, hoogwaardig woord eerst.

### 5.7 `config.rs`

`AppConfig` (netwerk / proces / regeling) ⇄ TOML. `IpFilter` (jokers `*` per
IPv4-octet). `ServerStatus`. `to_regulator_config()` vormt de brug naar het domein.

### 5.8 `gui.rs`

**Eénpagina**-GUI: koptekst (toestanden + knoppen), bedieningspaneel (links),
supervisie + grafiek (midden), live Modbus-tabel (rechts), Parameters-modaal.
Leest de `Shared*`, stuurt `Command`'s via een niet-blokkerende `cast`.

---

## 6. Scenario's (sequenties)

**Modbus-lezing (PV)**: client → `RegulatorService::call(ReadInputRegisters)` →
lezen van `SharedMap` → `Response`. Geen interactie met de actor (minimale latentie).

**Modbus-schrijving (setpoint)**: client → `call(WriteMultipleRegisters)` →
`map::holdings_to_commands` → `cast(Command::SetSpAuto)` → de actor past toe bij de
volgende stap → herpubliceert `SharedMap`/`SharedSnapshot`.

**GUI-commando**: interactie → `cast(Command)` → idem.

**Netwerk-herconfiguratie**: modaal *Toepassen* → `cast(Reconfigure)` →
ModbusServerActor rebindt indien nodig → `SharedStatus` bijgewerkt → de koptekst
van de GUI weerspiegelt de toestand.

**Tick**: timer → `Tick` → `Regulator::step` → publicatie.

---

## 7. Regeltheorie

**Proces (FOPDT)**: `v[k+1] = v[k] + (dt/T)·(doel − v[k])`, met
`doel = ambient + K·u` en `u` vertraagd met `L` seconden (vertragingslijn).

**PID**: `u = Kp·e + Ki·∫e + Kd·de/dt`, integraal begrensd tot `[out_min, out_max]`
(anti-windup). Afgeleide op de fout (compromis eenvoud/warm-koud-symmetrie).

**TOR**: actief als `e > +H/2`, inactief als `e < −H/2`, anders blijft de toestand
behouden.

**Tweerichting**: slechts één richting werkt tegelijk, geselecteerd door het teken
van de fout; de globale uitgang heeft een teken (+ warm / − koud).

---

## 8. Beslissingen en compromissen

- **Dubbele publicatie (`Snapshot` + `Map`)** in plaats van één enkele structuur:
  de GUI manipuleert bedrijfstypen, Modbus ruwe registers; beide blijven eenvoudig
  en ontkoppeld, ten koste van een lichte, verwaarloosbare kopieeroverhead.
- **Modbus-leesacties zonder de actor te passeren**: men leest `SharedMap`
  rechtstreeks om de latentie te minimaliseren; de actor blijft de enige
  **schrijver**, dus geen race.
- **Synchrone Modbus-service** (`future::Ready`): al het werk is niet-blokkerend
  (kort slot + cast), het is onnodig een future te boxen.
- **Rebind bij poortwijziging**: een socket verandert niet van poort; we
  accepteren een korte onderbreking van de dienst bij de herconfiguratie.
- **Afgeleide op de fout** (en niet op de meting): lichte « zweepslag » bij
  setpointwijziging, geaccepteerd om het algoritme symmetrisch en eenvoudig te
  houden.

---

## 9. Mogelijke uitbreidingen

- Modbus RTU / serieel (hergebruik `RegulatorService`, wijzig het transport).
- Setpointhelling, PID-autotuning, gesimuleerde fouten (sensor defect, verzadiging).
- Historiek / CSV-export van de trend.
- Omschakeling van de GUI naar **tabbladen** als de enkele pagina te dicht wordt.
- Nieuwe instrumenten: maak `mock_bin_<naam>` en factoriseer het gemeenschappelijke
  in `mock_lib_*` (zie [maintenance.md](maintenance.md)).
