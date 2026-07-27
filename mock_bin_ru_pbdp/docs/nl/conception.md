# Ontwerp — Gesimuleerde PROFIBUS-DP-regelaar (ORPD)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · **NL** · [PL](../pl/conception.md)*

> Crate: `mock_bin_ru_pbdp` · Uitvoerbaar bestand: **ru_pbdp** (*Regulation Unit over PROFIBUS DP*)

Architectuur- en modelleringsdocument. Naar het voorbeeld van de **ORME**-
regelaar (`mock_bin_ru_modbus`) voor het bedrijfsmodel en de actoren, en van
**OSNE** (`mock_bin_su_namur`) voor de seriële verbinding. Alleen de
**protocollaag** verandert: een vanaf nul ontwikkelde **software-simulator
van PROFIBUS-DP-V0-frames** (er bestaat tot op heden geen gepubliceerde
`profibus`/`profibus-dp`-crate in het Rust-ecosysteem).

---

## 1. Doel

Een **procesregelaar** simuleren (PID-lus op een thermisch proces van de
eerste orde, model **identiek** aan ORME) en deze blootstellen via een
**PROFIBUS-DP-V0-framestructuur** over een seriële verbinding
(RS-485/RS-232).

**Dit document veronderstelt dat de lezer de waarschuwing over
niet-interoperabiliteit heeft gelezen** (zie
[`manuel_utilisateur.md`](manuel_utilisateur.md) en
[`reference_profibus.md`](reference_profibus.md) §6): echte PROFIBUS DP
vereist naleving van de bus-timing op bitniveau (*slot time*, `Tsdr`
min/max, een watchdog in de orde van tientallen milliseconden) die alleen
een dedicated ASIC (SPC3/VPC3) kan garanderen. Deze simulator claimt dit
niet — het is een educatief en software-testinstrument, geen busdriver.

---

## 2. Fysiek model ([`regulator.rs`](../../src/regulator.rs))

Ongewijzigd overgenomen van de ORME-regelaar:
[`mock_lib_control::FirstOrderProcess`] (overdrachtsfunctie van de eerste
orde met zuivere dode tijd) en [`mock_lib_control::Pid`] (anti-windup
PID), met dezelfde modi (Uit/PID/Aan-uit/PWM) in beide richtingen
(verwarmen/koelen). Simulatiestap: **50 ms**. Alle schrijfbewerkingen
worden **gesaneerd** in `Regulator::apply` (grenzen herordend, niet-
eindige zwevendekommawaarden genegeerd, PID-versterkingen begrensd) —
dezelfde invariant als overal elders in de workspace: nooit `f32::clamp`
aanroepen met ongevalideerde grenzen.

---

## 3. Architectuur (actoren)

```
GUI (egui) ──Command(cast)──►  SimulationActor  ──refresh──► SharedSnapshot ──► GUI
Gesimuleerde PROFIBUS-master ►  (Regulator)      ──refresh──► SharedSnapshot ──► Data_Exchange-antwoorden
```

- **`SimulationActor`** ([`actors/simulation.rs`](../../src/actors/simulation.rs)):
  qua vorm identiek aan die van ORME/OSNE — enige eigenaar van de
  `Regulator`, opnieuw bewapende eenmalige timer, publiceert de
  `SharedSnapshot` bij elke stap.
- **`ProfibusServerActor`** ([`actors/network.rs`](../../src/actors/network.rs)):
  bezit de seriële verbinding; `Reconfigure` sluit/heropent het transport
  als poort/baudrate/stationsadres wijzigt; behoudt de `JoinHandle` van de
  sessie (afgebroken bij het stoppen); publiceert de verbindingsstatus
  (`ServerStatus`, inclusief de huidige toestand van de DP-V0-
  toestandsmachine) voor de GUI.
- **[`profibus.rs`](../../src/profibus.rs)** — **bron van waarheid** van
  het protocol: framecodec (`SD1`/`SD2`/`SD3`/`SD4`/`SC`, FCS), decodering
  van de diensten
  (`Slave_Diag`/`Set_Prm`/`Chk_Cfg`/`Data_Exchange`) en de
  toestandsmachine van de slave `SlaveFsm`
  (`PowerOn → WaitPrm → WaitCfg → DataExchange`).
- **[`map.rs`](../../src/map.rs)** — omzetting van de `Data_Exchange`-I/O-
  bytenblokken naar/van de `Command`s van de regelaar (zie
  [`reference_profibus.md`](reference_profibus.md) §3).
- **[`profibus_server.rs`](../../src/profibus_server.rs)** — sessielus over
  eender welke `AsyncRead + AsyncWrite`-stream (de seriële poort in
  productie, een `tokio::io::duplex` in tests): leest een frame, decodeert
  het, roept `SlaveFsm::handle` aan, past de resulterende `Command`s toe,
  codeert het antwoord en stuurt het terug. Behandelt ook de
  **protocolwatchdog** (`tokio::select!` tussen frame lezen en een
  vertraging, zoals de NAMUR-watchdog van OSNE — maar hier is het een
  **echt onderdeel van het DP-protocol**, bewapend door `Set_Prm`, geen
  zelfgemaakte toevoeging).

In tegenstelling tot Modbus (ORME, een aparte, bij elke tick opnieuw
opgebouwde geheugentabel) en zoals bij OPC UA/NAMUR is er **geen
persistente geheugentabel**: het `Data_Exchange`-invoerblok wordt op het
moment van antwoorden ter plekke herberekend vanuit de `SharedSnapshot`.

**Geen multi-master-beleid te beheren**: de seriële verbinding *is* de
enige master (zoals Modbus RTU of de seriële NAMUR-poort), in tegenstelling
tot Modbus TCP van ORME (verdringing) of zelfs NAMUR TCP van OSNE
(punt-naar-punt zonder verdringing).

---

## 4. PROFIBUS-DP-V0-codec — keuzes en aanvaarde beperkingen

- **Framebegrenzers** (`SD1=0x10`, `SD2=0x68`, `SD3=0xA2`, `SD4=0xDC`,
  `SC=0xE5`, `ED=0x16`) en **FCS** (som modulo 256): conform de norm,
  goed publiek gedocumenteerd.
- **SAP-nummers van de parametreringsdiensten** (`Slave_Diag=61`,
  `Set_Prm=62`, `Chk_Cfg=63`): conform.
- **Exacte codering van de FC-veldbits**, **precieze indeling van de
  diagnosebytes**, en **indeling van de invoer-/uitvoerblokken**
  (`map.rs`): dit zijn **conventies eigen aan deze simulator**, geen
  echt bij de PNO geregistreerd GSD-profiel. De simulator gebruikt
  systematisch **SD2**-frames (variabele lengte) voor alle
  `Data_Exchange`-uitwisselingen, zelfs wanneer `SD3` (8 vaste bytes) zou
  volstaan in een echt profiel — een keuze die de codec vereenvoudigt
  zonder dekking van de protocolconcepten te verliezen.
- **PROFIBUS-identificatie** (`Ident_Number = 0xEE01`): **fictief**, niet
  geregistreerd bij de PNO (PROFIBUS & PROFINET International) —
  vertegenwoordigt geen enkel echt catalogusapparaat.
- **Geen enkele bus-timing**: noch een antwoordvenster (`Tsdr`), noch een
  token, noch multi-master-arbitrage zijn geïmplementeerd — zie §1.

Volledig detail in [`reference_profibus.md`](reference_profibus.md).

---

## 5. Configuratie en persistentie

`AppConfig` (taal / seriële verbinding / proces / regeling / update-
controle) geserialiseerd als **TOML**
([`config.rs`](../../src/config.rs)), **gesaneerd bij het laden**
(`AppConfig::sanitized`: grenzen geordend, `τ ≥ 1e-3`, `dead_time ≥ 0`,
eindige zwevendekommawaarden, stationsadres begrensd tot `[0, 125]`).
Bestand: `mock_ru_pbdp.toml` (overschrijfbaar via `MOCK_CONFIG`). In
tegenstelling tot ORME/OSNE, **geen IP-whitelist** (de seriële verbinding
is inherent punt-naar-punt, geen begrip van een netwerkadres).

---

## 6. Ontwikkelingsmogelijkheden

- Een echt hulpmiddel voor een **gesimuleerde PROFIBUS-DP-master**
  (afzonderlijk binair bestand), dat dezelfde codeer-/decodeerfuncties
  gebruikt die voor tests worden blootgesteld in `profibus.rs`, om deze
  simulator of elke andere software-slave aan te sturen zonder
  afhankelijk te zijn van een ad-hocscript.
- Generatie van een illustratief **GSD**-bestand (niet-functioneel aan de
  simulatorzijde) dat het gesimuleerde I/O-profiel documenteert, voor
  educatieve doeleinden.
- Ondersteuning van **DP-V1** (acyclische toegang, alarmen) mocht de
  educatieve behoefte ontstaan — aanvankelijk buiten bereik (alleen
  DP-V0).
- Promotie van het regelaarmodel naar een gedeelde `mock_lib_*` (vandaag
  gedupliceerd tussen ORME en dit instrument, zoals bij ORUE).
