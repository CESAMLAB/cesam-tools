# PROFIBUS-DP-V0-referentie — Gesimuleerde regelaar (ORPD)

*🌍 [FR](../fr/reference_profibus.md) · [EN](../en/reference_profibus.md) · [DE](../de/reference_profibus.md) · [ES](../es/reference_profibus.md) · [IT](../it/reference_profibus.md) · [PT](../pt/reference_profibus.md) · **NL** · [PL](../pl/reference_profibus.md)*

> Crate: `mock_bin_ru_pbdp` · Uitvoerbaar bestand: **ru_pbdp** · Protocol: **PROFIBUS DP-V0** (seriële slave)

Dit document is de functionele referentie van de gesimuleerde PROFIBUS-
DP-V0-subset. De **technische bron van waarheid** blijft de kop van
[`src/profibus.rs`](../../src/profibus.rs) (codec + toestandsmachine) en
van [`src/map.rs`](../../src/map.rs) (I/O-blokken): elke afwijking moet
eerst in de code worden gecorrigeerd.

---

## ⚠️ 0. Reikwijdte en beperkingen — lees dit vóór elk gebruik

`ru_pbdp` implementeert een **educatieve subset** van DP-V0, **zonder
enige pretentie van strikte binaire conformiteit** met de normatieve
tabellen (IEC 61158 / EN 50170) verder dan de meest universeel
gedocumenteerde elementen:

- **conform**: framebegrenzers (`SD1`/`SD2`/`SD3`/`SD4`/`SC`/`ED`), FCS
  (som modulo 256), SAP-nummers van de parametreringsdiensten
  (`Slave_Diag` = 61, `Set_Prm` = 62, `Chk_Cfg` = 63).
- **conventies eigen aan deze simulator, geen echt bij de PNO
  geregistreerd GSD-profiel** (PROFIBUS & PROFINET International): exacte
  codering van de `FC`-veldbits, precieze indeling van de diagnosebytes,
  indeling van de invoer-/uitvoerblokken (§3), de identificatie
  `Ident_Number` (§4).
- **geen enkele echte bus-timing**: noch een antwoordvenster (*slot
  time*, `Tsdr` min/max), noch een token tussen masters, noch multi-
  master-arbitrage. Alleen een dedicated ASIC (SPC3/VPC3) of een
  hardware-masterkaart (Hilscher/Softing/Siemens CP) kunnen deze
  beperkingen op bitniveau naleven.

**Direct gevolg: deze simulator zal nooit worden herkend door een echte
PROFIBUS-DP-master** (PLC + masterkaart). Hij dient om de structuur van
het protocol te begrijpen en een software-ontwikkeling te testen (codec,
toestandsmachine, tooling), niet om veldapparatuur aan te sturen — zie
[`manuel_utilisateur.md`](manuel_utilisateur.md).

---

## 1. Frames — begrenzers en FCS

| Begrenzer | Waarde | Gebruik |
|---|:--:|---|
| `SD1` | `0x10` | Vast verzoek zonder data (6 bytes: `SD1 DA SA FC FCS ED`) |
| `SD2` | `0x68` | Frame met variabele lengte met data (`SD2 LE LEr SD2 DA SA FC [data…] FCS ED`) |
| `SD3` | `0xA2` | Frame met vaste data, 8 bytes (14 bytes in totaal) — **niet gebruikt** door deze simulator (zie §0), geleverd voor volledigheid van de codec en de tests ervan |
| `SD4` | `0xDC` | Tokenframe, 3 bytes, zonder FCS of ED — buiten bereik voor een gesimuleerde single-mastr-slave, geleverd voor volledigheid van de codec |
| `SC` | `0xE5` | Korte bevestiging, 1 byte |
| `ED` | `0x16` | Eindbegrenzer |

- **`FCS`**: som modulo 256 van de nuttige bytes van het frame (zie
  `profibus::checksum`). Een frame met een onjuiste FCS wordt zonder
  antwoord verworpen (`FrameError::BadChecksum`) — de master moet
  opnieuw verzenden.
- **`DA`/`SA`**: bestemmings-/bronadres. Bit 7 van `DA` = **adresextensie
  (DAE)**: aanwezigheid van een SAP-byte direct na `DA` in de payload.
  Afwezig = standaard gegevensuitwisseling (`Data_Exchange`). Het
  stationsadres neemt de resterende 7 bits in beslag (`0`-`125`;
  `126`/`127` gereserveerd door de norm, hier ongebruikt).
- **Deze simulator geeft systematisch de voorkeur aan `SD2`** voor alle
  `Data_Exchange`-uitwisselingen, zelfs wanneer `SD3` (8 vaste bytes) zou
  volstaan in een echt profiel — een keuze die de codec vereenvoudigt
  zonder dekking van de protocolconcepten te verliezen (zie
  [`conception.md`](conception.md) §4).
- **Misvormd frame / onbekende begrenzer (lijnruis)**: stilzwijgend
  verworpen (`log::debug!`), de sessie gaat door — maakt het mogelijk de
  bytestroom opnieuw te synchroniseren zonder de verbinding te laten
  crashen.

---

## 2. Sequencing — diensten en toestandsmachine

De gesimuleerde slave (`SlaveFsm`, [`profibus.rs`](../../src/profibus.rs))
doorloopt vier toestanden:

```
PowerOn ──Slave_Diag──► WaitPrm ──Set_Prm (id OK)──► WaitCfg ──Chk_Cfg (lengtes OK)──► DataExchange
```

| Toestand | Betekenis | Typisch antwoord |
|---|---|---|
| `Power_On` | Direct na het opstarten, vóór de eerste diagnosepoll | — |
| `Wait_Prm` | Wacht op een geldige `Set_Prm` | `Diag` met `Stat_1 = STAT1_PRM_REQ` |
| `Wait_Cfg` | Geparametreerd, wacht op een geldige `Chk_Cfg` | `Diag` met `Stat_1 = STAT1_CFG_FAULT` |
| `Data_Exchange` | Geparametreerd en geconfigureerd: cyclische uitwisseling actief | invoerblok (§3) |

### `Slave_Diag` (SAP 61)

Verzoek zonder data (of een `SD1`-frame, volgens deze simulator altijd
geïnterpreteerd als `Slave_Diag` — geen adresextensie mogelijk op `SD1`,
bij gebrek aan een beschikbare byte om een SAP te dragen). `Diag`-
antwoord (6 bytes):

| Byte | Symbool | Inhoud |
|:--:|---|---|
| `0` | `Stat_1` | `0x01` (`STAT1_PRM_REQ`, zolang niet geparametreerd) of `0x02` (`STAT1_CFG_FAULT`, zolang niet geconfigureerd) of `0x00` (`Data_Exchange`) |
| `1` | `Stat_2` | altijd `0x00` (niet gesimuleerd) |
| `2` | `Stat_3` | altijd `0x00` (niet gesimuleerd) |
| `3` | `Master_Add` | `0xFF` (geen bekende master — niet bijgehouden door deze simulator) |
| `4-5` | `Ident_Number` | vaste identificatie van de slave, big-endian (§4) |

De eerste ontvangen `Slave_Diag` doet `Power_On` → `Wait_Prm` overgaan; de
volgende wijzigen de toestand niet (slechts een diagnoselezing).

### `Set_Prm` (SAP 62)

Verzoek (standaard DP-V0-formaat, **komt overeen** met wat een echte master
verstuurt — bijv. `profirust` — geen conventie die specifiek is voor deze
simulator, in tegenstelling tot de I/O-blokindeling in §3):

```
SAP(62) Station_Status(1) WD_Fact_1(1) WD_Fact_2(1) Min_Tsdr(1) Ident_Number(2, BE) Groups(1) [User_Prm_Data...]
```

`Station_Status` (bits Lock_Req/Sync_Req/Freeze_Req/WD_On), `Min_Tsdr`,
`Groups` en `User_Prm_Data` worden door deze simulator **niet gebruikt**
(geen vergrendeling, geen Sync-/Freeze-modus, geen groepen gemodelleerd);
alleen `WD_Fact_1`/`WD_Fact_2` en `Ident_Number` worden gelezen. De
aangekondigde watchdog, indien aanwezig, wordt berekend als
`watchdog_ms = WD_Fact_1 × WD_Fact_2 × 10` (eenheid 10 ms, standaard-DP-
conventie); `WD_Fact_1 = 0` **of** `WD_Fact_2 = 0` betekent «geen
watchdog». Antwoord: in alle gevallen `ShortAck` (`SC`).

- Als `Ident_Number` **overeenkomt** met het vaste profiel van de slave
  (§4): toestand → `Wait_Cfg`, en een eventuele watchdog wordt
  doorgegeven aan de sessie (alleen bewapend als de lokale instelling
  `watchdog_enabled` dit toestaat — zie
  [`manuel_utilisateur.md`](manuel_utilisateur.md) §4).
- Als de identificatie **niet overeenkomt**: de parametrering wordt
  stilzwijgend afgewezen (`ShortAck` toch teruggestuurd, zoals DP-V0
  voorschrijft voor deze dienst, maar zonder effect op de interne
  toestand) — de slave blijft in `Wait_Prm`.

### `Chk_Cfg` (SAP 63)

Verzoek: `SAP(63) Out_Len(1) In_Len(1)`. Antwoord: `ShortAck`. De
toestand gaat **alleen** over naar `Data_Exchange` als `Out_Len == 45` en
`In_Len == 17` (vaste groottes van het gesimuleerde profiel, §3) **en**
de slave in `Wait_Cfg` was; anders verandert de toestand niet (de master
moet een correcte `Chk_Cfg` opnieuw verzenden).

### `Data_Exchange` (geen SAP — standaardadres, DAE-bit afwezig)

Verzoek: het ruwe uitvoerblok (45 bytes, §3). Antwoord: het invoerblok
(17 bytes, §3), ter plekke herberekend vanuit de gedeelde snapshot op het
moment van antwoorden (geen persistente geheugentabel, in tegenstelling
tot Modbus/ORME).

Als de master een `Data_Exchange` stuurt **voordat** de toestand
`Data_Exchange` is bereikt (sequencing niet nageleefd), antwoordt de
slave met de huidige diagnose (`Diag`) in plaats van te crashen of het
frame te negeren.

---

## 3. I/O-blokken — byte-indeling

Overgenomen uit de kop van [`map.rs`](../../src/map.rs), de enige bron
van waarheid bij afwijking. Alle zwevendekommawaarden (`f32`) nemen
**4 opeenvolgende bytes, big-endian** in beslag.

### Uitvoerblok — *Output* (master → slave, `OUTPUT_LEN` = 45 bytes)

| Byte(s) | Symbool | Type | Beschrijving |
|---|---|:--:|---|
| `0` | `OUT_MODE` | bits | bit0 = werking, bit1 = auto, [3:2] = modus richting 1, [5:4] = modus richting 2 |
| `1-4` | `OUT_SP_AUTO` | f32 | Automatisch setpoint |
| `5-8` | `OUT_SP_MANUAL` | f32 | Handmatig setpoint (% uitgang, met teken) |
| `9-12` | `OUT_KP1` | f32 | Proportionele versterking Kp richting 1 |
| `13-16` | `OUT_KI1` | f32 | Integrale versterking Ki richting 1 |
| `17-20` | `OUT_KD1` | f32 | Differentiële versterking Kd richting 1 |
| `21-24` | `OUT_KP2` | f32 | Proportionele versterking Kp richting 2 |
| `25-28` | `OUT_KI2` | f32 | Integrale versterking Ki richting 2 |
| `29-32` | `OUT_KD2` | f32 | Differentiële versterking Kd richting 2 |
| `33-36` | `OUT_HYSTERESIS` | f32 | Hysterese van de aan-uit-regelaars |
| `37-40` | `OUT_TOR_MIN_CYCLE` | f32 | Minimale aan-uit-cyclustijd (s) |
| `41-44` | `OUT_PWM_PERIOD` | f32 | Periode van de PWM-modulatiecyclus (s) |

De moduscodes (`[3:2]`/`[5:4]`) volgen `ControllerKind`: `0` = Uit,
`1` = PID, `2` = Aan-uit, `3` = PWM (zie `mock_lib_control`).

### Invoerblok — *Input* (slave → master, `INPUT_LEN` = 17 bytes)

| Byte(s) | Symbool | Type | Beschrijving |
|---|---|:--:|---|
| `0` | `IN_STATUS` | bits | bit0 = in werking, bit1 = richting 1 actief (uitgang > 0), bit2 = richting 2 actief (uitgang < 0) |
| `1-4` | `IN_PV` | f32 | Meetwaarde / *process value* |
| `5-8` | `IN_OUTPUT` | f32 | Toegepaste uitgang (% met teken) |
| `9-12` | `IN_SP_AUTO` | f32 | Terugmelding (alleen-lezen) van het automatisch setpoint |
| `13-16` | `IN_SP_MANUAL` | f32 | Terugmelding (alleen-lezen) van het handmatig setpoint |

Een **te kort** uitvoerblok (< 45 bytes) wordt genegeerd zonder te
crashen: er wordt geen `Command` geproduceerd, de regelaar behoudt zijn
laatst geldige toestand.

---

## 4. Vast profiel van de slave

| Parameter | Waarde | Opmerking |
|---|---|---|
| `Ident_Number` | `0xEE01` | **Fictief**, niet geregistreerd bij de PNO — vertegenwoordigt geen enkel echt catalogusapparaat |
| `Out_Len` | `45` | Verwacht in `Chk_Cfg.out_len` |
| `In_Len` | `17` | Verwacht in `Chk_Cfg.in_len` |
| Stationsadres | `0`-`125`, configureerbaar | Lokale instelling (modaal *Instellingen*), zie [`manuel_utilisateur.md`](manuel_utilisateur.md) §4 |
| Seriële frameformaat | `8E1` (8 bits, even pariteit, 1 stopbit) | **Vastgelegd door de PROFIBUS-DP-norm**, niet instelbaar |
| Genormaliseerde baudrates | `9600` tot `12.000.000` bit/s | Niet gecontroleerd bij het openen: een niet-standaardwaarde wordt zonder meer doorgegeven aan de seriële poort |

---

## 5. Protocolwatchdog

In tegenstelling tot de NAMUR-watchdog van OSNE (een zelfgemaakte
toevoeging) is dit een **echt onderdeel van het DP-protocol**: hij wordt
**door de master aangekondigd** in `Set_Prm` (factoren
`WD_Fact_1`/`WD_Fact_2`, §2) en wordt alleen **aan de slave-zijde
bewapend** als de lokale instelling `watchdog_enabled` dit toestaat
(anders wordt het verzoek van de master genegeerd, nooit bewapend). Bij
verlopen, zonder een nieuw frame voor het station te hebben ontvangen,
dwingt de slave de veilige toestand af
(`Command::SetOnOff(false)`) — een gedocumenteerde vereenvoudiging: een
echt DP-V0-profiel zou een volledige terugkeer via `Set_Prm`/`Chk_Cfg`
kunnen vereisen voordat de uitwisseling wordt hervat, wat deze simulator
niet expliciet vereist (het volstaat om het verzenden van
`Data_Exchange`-frames te hervatten, aangezien de toestand
`Data_Exchange` niet wordt verlaten bij het verlopen van de watchdog).

---

## 6. Niet-interoperabiliteit — waarom

| Vereiste van echte PROFIBUS DP | Deze simulator |
|---|---|
| Antwoordvenster op bitniveau (*slot time*, `Tsdr` min/max) | Afwezig — antwoordt zodra het frame is gedecodeerd, zonder tijdbeperking |
| Dedicated circuit (SPC3/VPC3-ASIC) voor de timing | Afwezig — gewone Tokio-software |
| Token tussen masters, multi-master-arbitrage | Afwezig — single-master-slave, punt-naar-puntverbinding |
| Bij de PNO geregistreerd GSD-profiel | Afwezig — I/O-profiel eigen aan deze simulator (§3) |
| Bit-exacte codering van de FC-/diagnosevelden | Simulatieconventie, niet gegarandeerd conform |

**Een echte PLC (bijvoorbeeld een Siemens S7 met masterkaart) zal deze
simulator nooit als geldige slave herkennen op een echte PROFIBUS-DP-
RS-485-bus.** Twee instanties van deze simulator (of een script dat de
onderstaande sequentie afspeelt) kunnen daarentegen met elkaar
communiceren om het protocol te illustreren — zie
[`manuel_utilisateur.md`](manuel_utilisateur.md) §5.

---

## 7. Voorbeeldsequentie (hexadecimaal)

Volledige sequentie voor station `5`, master `3`, tot aan de cyclische
uitwisseling (illustratieve waarden, `FCS` berekend over de nuttige
bytes):

```text
# 1. Slave_Diag (SD2, DAE=1, SAP=61)
→ TX  68 03 03 68 85 03 C0 3D FC 16
← RX  68 06 06 68 03 85 00 01 00 00 FF EE 01 F5 16   (Diag: Stat_1=0x01, Ident=0xEE01)

# 2. Set_Prm (SD2, DAE=1, SAP=62, standaard DP-V0-formaat: Station_Status
#    Lock_Req+WD_On=0x88, WD_Fact_1=1, WD_Fact_2=30 (300ms), Min_Tsdr=0,
#    Ident=0xEE01, Groups=0)
→ TX  68 0B 0B 68 85 03 C0 3E 88 01 1E 00 EE 01 00 … 16
← RX  E5                                              (ShortAck)

# 3. Chk_Cfg (SD2, DAE=1, SAP=63, out_len=45, in_len=17)
→ TX  68 05 05 68 85 03 C0 3F 2D 11 … 16
← RX  E5                                              (ShortAck)

# 4. Data_Exchange (SD2, geen SAP, uitvoerblok van 45 bytes)
→ TX  68 30 30 68 05 03 C0 [45 bytes] … 16
← RX  68 14 14 68 03 85 00 [17 bytes]  … 16          (invoerblok)
```

De exacte FCS-/lengtebytes hangen af van de payloadwaarden; dit schema
illustreert de **volgorde van de diensten**, geen letterlijk te herhalen
frame. Zie de tests in [`profibus.rs`](../../src/profibus.rs) en
[`profibus_server.rs`](../../src/profibus_server.rs) voor bit-exact
geverifieerde sequenties.
