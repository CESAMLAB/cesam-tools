# Modbus-adrestabel — Gesimuleerde regelaar

*🌍 [FR](../fr/table_modbus.md) · [EN](../en/table_modbus.md) · [DE](../de/table_modbus.md) · [ES](../es/table_modbus.md) · [IT](../it/table_modbus.md) · [PT](../pt/table_modbus.md) · **NL** · [PL](../pl/table_modbus.md)*

> Crate: `mock_bin_ru_modbustcp` · Protocol: **Modbus TCP** (slave / server)

Dit document is de functionele referentie van het adresseringsplan. De
**technische bron van waarheid** blijft het kopcommentaar van
[`src/map.rs`](../../src/map.rs): elke afwijking moet bij voorrang in de code worden
gecorrigeerd.

---

## 1. Algemeenheden

| Element | Waarde |
|---------|--------|
| Transport | Modbus **TCP** of **RTU serieel / RS485** (slechts één tegelijk actief) |
| Rol | **Slave** (server) |
| Standaardpoort | TCP `5502` (configureerbaar, modaal *Parameters*) |
| Serieel (RTU) | poort + baud + pariteit + bits, configureerbaar (feature `rtu`) |
| Unit ID / adres | TCP: irrelevant. RTU: `slave_id` configureerbaar maar **niet gefilterd** (zie noot) |
| Masters | **slechts één externe master tegelijk**; in TCP verbreekt een nieuwkomer de vorige (de lokale GUI is geen master) |
| Adressering | **base 0** (adres `0` = 1e element van de tabel) |
| Filtering | optionele IP-witte lijst (jokers `*`, alleen TCP) |

> **Noot RTU / slave-adres**: de RTU-server antwoordt **ongeacht het gevraagde
> adres** (het adres wordt niet doorgegeven aan de applicatieservice). Gebruik een
> **punt-tot-punt**-verbinding. De `slave_id` wordt bewaard/weergegeven maar voert
> geen filtering uit.

### Adressering base 0 vs base 1

De onderstaande adressen zijn de **protocoladressen (base 0)**, zoals verzonden in
het frame. Veel tools tonen een « conventionele » base 1-nummering (`4xxxx` voor de
holdings, `3xxxx` voor de inputs…). Zo komt het holding-register met adres `2`
overeen met de conventionele referentie `40003`.

---

## 2. Codering van de drijvende-kommagetallen (`f32`)

De analoge grootheden zijn **`f32` IEEE-754 op 2 opeenvolgende registers**:

- **woordvolgorde**: **hoogwaardig woord eerst** (big-endian, genaamd *ABCD*);
- **bytevolgorde** binnen elk register: big-endian (Modbus-standaard).

Voorbeeld: `80.0` → bytes `42 A0 00 00` → register `n` = `0x42A0`,
register `n+1` = `0x0000`.

> Als uw client afwijkende waarden leest, is dat bijna altijd een probleem met de
> woordvolgorde (probeer *word swap* / *CDAB*).

---

## 3. Bobines — *Coils* (lezen/schrijven)

Functiecodes: `0x01` (lezen), `0x05` (enkel schrijven), `0x0F` (meervoudig schrijven).

| Adres | Benaming | Waarden | Effect |
|-------|----------|---------|--------|
| `0` | Aan / Uit | `0` = uit, `1` = aan | Activeert de regeling |
| `1` | Auto / Handmatig | `0` = handmatig, `1` = auto | Keuze van de modus |

---

## 4. Discrete ingangen — *Discrete Inputs* (alleen-lezen)

Functiecode: `0x02`.

| Adres | Benaming | Betekenis |
|-------|----------|-----------|
| `0` | In bedrijf | Het apparaat is in bedrijf |
| `1` | Richting 1 (warm) actief | Uitgang > 0 |
| `2` | Richting 2 (koud) actief | Uitgang < 0 |

---

## 5. Holding-registers — *Holding Registers* (lezen/schrijven)

Functiecodes: `0x03` (lezen), `0x06` (enkel schrijven), `0x10` (meervoudig schrijven).

| Adres | Benaming | Type | Eenheid / waarden |
|-------|----------|------|-------------------|
| `0` | Regelmodus richting 1 (warm) | `u16` | `0`=Off, `1`=PID, `2`=TOR, `3`=PWM |
| `1` | Regelmodus richting 2 (koud) | `u16` | `0`=Off, `1`=PID, `2`=TOR, `3`=PWM |
| `2`–`3` | Automatisch setpoint (SP) | `f32` | meeteenheid |
| `4`–`5` | Handmatig setpoint | `f32` | % uitgang, met teken (−100…+100) |
| `6`–`7` | `Kp` richting 1 | `f32` | proportionele gain |
| `8`–`9` | `Ki` richting 1 | `f32` | integrale gain (s⁻¹) |
| `10`–`11` | `Kd` richting 1 | `f32` | afgeleide gain (s) |
| `12`–`13` | `Kp` richting 2 | `f32` | proportionele gain |
| `14`–`15` | `Ki` richting 2 | `f32` | integrale gain (s⁻¹) |
| `16`–`17` | `Kd` richting 2 | `f32` | afgeleide gain (s) |
| `18`–`19` | TOR-hysterese | `f32` | meeteenheid |
| `20`–`21` | Minimale TOR-cyclustijd | `f32` | seconden (anti-kortsluitcyclus, `0` = uitgeschakeld) |
| `22`–`23` | PWM-cyclusperiode | `f32` | seconden (> 0) |
| `42`–`46` | Apparaatidentificatie | `ASCII` | « CESAM-Lab » (alleen-lezen, 2 tek./register, hoogwaardig eerst) |

> Registers `24`–`41` gereserveerd (gelezen als `0`).

> **Gedeeltelijke schrijfactie van een `f32`**: men moet **beide registers** van
> een float schrijven opdat hij in aanmerking wordt genomen. Een schrijfactie van
> één enkel register van een `f32`-paar wordt genegeerd (en geeft de uitzondering
> *Illegal Data Address* terug als ze geen enkel geldig veld dekt).
>
> De geschreven gains worden begrensd tot eindige waarden ≥ 0 (robuustheid).

---

## 6. Ingangsregisters — *Input Registers* (alleen-lezen)

Functiecode: `0x04`.

| Adres | Benaming | Type | Eenheid |
|-------|----------|------|---------|
| `0`–`1` | Meting (PV — *process value*) | `f32` | meeteenheid |
| `2`–`3` | Toegepaste uitgang | `f32` | % met teken (+ warm / − koud) |
| `4`–`5` | Teruglezing automatisch setpoint (alleen-lezen) | `f32` | meeteenheid |
| `6`–`7` | Teruglezing handmatig setpoint (alleen-lezen) | `f32` | % uitgang, met teken (−100…+100) |

> **Teruglezingen van de setpoints**: registers `4`–`7` stellen de huidige waarde
> van de automatische/handmatige setpoints **alleen-lezen** beschikbaar (spiegels
> van de holdings `2`–`5`). Handig voor een supervisor die alleen **bewaakt**
> zonder te schrijven.

---

## 7. Modbus-uitzonderingen

| Code | Naam | Oorzaak in dit apparaat |
|------|------|-------------------------|
| `0x01` | Illegal Function | Niet-beheerde functiecode (bv. mask, FIFO) |
| `0x02` | Illegal Data Address | Adres / hoeveelheid buiten de tabel, of schrijfactie die geen enkel veld treft |
| `0x04` | Server Device Failure | Intern slot onbeschikbaar (abnormaal geval) |

---

## 8. Voorbeelden met `mbpoll`

`mbpoll` adresseert in **base 1**; men voegt dus `1` toe aan de base 0-adressen.

```bash
# Inschakelen (coil base0 0 -> -t 0 -r 1) dan naar auto schakelen (coil 1)
mbpoll -m tcp -p 5502 -a 1 -t 0 -r 1 127.0.0.1 1     # On/Off = 1
mbpoll -m tcp -p 5502 -a 1 -t 0 -r 2 127.0.0.1 1     # Auto/Manuel = 1 (auto)

# Het auto-setpoint schrijven (HR base0 2-3 -> -t 4:float -r 3) op 80.0
mbpoll -m tcp -p 5502 -a 1 -t 4:float -r 3 127.0.0.1 80.0

# De meting PV lezen (IR base0 0-1 -> -t 3:float -r 1)
mbpoll -m tcp -p 5502 -a 1 -t 3:float -r 1 127.0.0.1

# De uitgang lezen (IR base0 2-3 -> -t 3:float -r 3)
mbpoll -m tcp -p 5502 -a 1 -t 3:float -r 3 127.0.0.1
```

> Afhankelijk van de versies van `mbpoll` kan de woordvolgorde van de floats de
> permutatie-optie vereisen. Bij een incoherente waarde, controleer de woordvolgorde.

---

## 9. Beknopte geheugenkaart

```
Coils (RW)            DiscreteInputs (RO)     Holding (RW)              Input (RO)
0  On/Off             0  In bedrijf           0  Modus richt1 (u16)     0-1 PV (f32)
1  Auto/Manuel        1  Warm actief          1  Modus richt2 (u16)     2-3 Uitgang (f32)
                      2  Koud actief          2-3  SP auto (f32)         4-5 SP auto (teruglezing, RO)
                                              4-5  SP handmatig (f32)     6-7 SP handmatig (teruglezing, RO)
                                              6-7  Kp1  8-9  Ki1  10-11 Kd1
                                              12-13 Kp2 14-15 Ki2 16-17 Kd2
                                              18-19 Hysterese (f32)
                                              20-21 Min. cyclus TOR (f32, s)
                                              22-23 PWM-periode (f32, s)
                                              42-46 ASCII-identificatie "CESAM-Lab"
```

> **ASCII-identificatie** (`HR 42-46`): « CESAM-Lab » gecodeerd 2 tekens per
> register, hoogwaardig teken eerst (`42`=`'CE'`, `43`=`'SA'`, `44`=`'M-'`,
> `45`=`'La'`, `46`=`'b\0'`). Alleen-lezen. Voorbeeld:
> `mbpoll -m tcp -p 5502 -a 1 -t 4 -r 43 -c 5 127.0.0.1` (registers base 1 43..47).
