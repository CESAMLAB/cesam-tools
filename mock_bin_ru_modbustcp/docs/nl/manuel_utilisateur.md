# Gebruikershandleiding — ORME (gesimuleerde Modbus-regelaar)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · **NL** · [PL](../pl/manuel_utilisateur.md)*

> **ORME** — *Open Regulator Modbus Emulator* · binair `mock_bin_ru_modbustcp` ·
> MIT-licentie · Uitgever: **CESAM-Lab** · Modbus-apparaatidentificatie: **CESAM-Lab**
>
> *« Open de bus. »* Een veldregelaar die alleen bestaat op uw Modbus-bus
> (TCP/RTU) — om SCADA, PLC's en HMI te testen zonder echte hardware.

Deze handleiding is bedoeld voor de **gebruiker** van de gesimuleerde regelaar: hoe
hem te starten, te besturen vanuit de interface, te parametreren en aan te sluiten
via Modbus TCP. Geen programmeerkennis vereist.

---

## 1. Waarvoor dient deze software?

Hij simuleert een **industriële regelaar** (type oven of thermostaatbad):

- een realistisch **fysiek proces** (de « meting » stijgt/daalt naargelang de
  besturing);
- een **regeling**, automatisch of handmatig, in **warm** en/of **koud**;
- een **Modbus TCP-server** om hem te besturen/superviseren vanuit een andere
  software (PLC, SCADA, gateway…);
- een **grafische interface** voor besturing en visualisatie.

Het is een **testtool**: hij maakt het mogelijk een supervisor of een PLC te
ontwikkelen en te demonstreren **zonder echte hardware**.

---

## 2. De software starten

Start het uitvoerbare bestand dat overeenkomt met uw systeem:

| Systeem | Bestand |
|---------|---------|
| Windows | `orme-windows-x86_64.exe` (dubbelklik) |
| Linux-pc | `./orme-linux-x86_64` |
| Raspberry Pi (scherm) | `./orme-rpi-arm64` |

Het venster opent en de **Modbus-server start automatisch** (poort `5502`
standaard). De koptekst geeft de toestand aan:

- **● IN BEDRIJF / ● GESTOPT**: toestand van het apparaat;
- **Modbus ● 0.0.0.0:5502** (groen): server luistert; **✖ …** (rood) bij een
  netwerkprobleem.

> Zonder scherm (alleen server), zie **§ 9 (Gebruik zonder scherm)**.

---

## 3. De interface in één oogopslag

Het venster bevat vier zones:

```
┌───────────────────────────── Koptekst: titel, ⚙ Parameters, 💾 Opslaan, toestanden ─────────────────────────────┐
├──────────────────┬─────────────────────────────────────────────────┬───────────────────────────────────────────┤
│  BESTURING        │   SUPERVISIE                                    │   MODBUS-ADRESTABEL                       │
│  (links)          │   - momentane waarden (Meting / Setpoint /      │   (rechts)                                │
│  Aan/Uit          │     Uitgang)                                    │   live lijst: benaming, tabel,            │
│  Auto/Handmatig   │   - real-time TREND-grafiek                     │   adres, waarde, toegang                  │
│  Modi, setpoints  │                                                 │                                           │
│  PID-instellingen │                                                 │                                           │
└──────────────────┴─────────────────────────────────────────────────┴───────────────────────────────────────────┘
```

---

## 4. De regelaar besturen (linkerpaneel)

### 4.1 Aan / Uit
Knop **Aan / Uit**. Bij uit is de uitgang nul en keert de meting langzaam terug
naar de omgevingswaarde.

### 4.2 Auto / Handmatig
- **Handmatig**: *u* legt de uitgang op via het **handmatige setpoint** (in %).
- **Auto**: de regelaar berekent de uitgang om het **auto-setpoint** te bereiken.

### 4.3 De setpoints
Elk setpoint beschikt over een **numeriek veld** (nauwkeurige invoer met het
toetsenbord) en een **schuifregelaar**. Beide zijn altijd aanpasbaar; het
**actieve** setpoint (volgens de modus) wordt vet weergegeven.

| Setpoint | Eenheid | Rol |
|----------|---------|-----|
| **SP auto** | meeteenheid (bv. °C) | te bereiken doel in de Auto-modus |
| **SP handmatig** | % uitgang, van −100 tot +100 | opgelegde uitgang in de Handmatige modus (**+** warm / **−** koud) |

### 4.4 Regelmodi — richting 1 (warm) en richting 2 (koud)
Elke richting wordt onafhankelijk ingesteld:

- **Uitgeschakeld** — de richting werkt niet;
- **PID** — continue regeling (uitgang 0…100 %), nauwkeurig en zacht;
- **Aan-uit (TOR)** — relais met hysterese: uitgang 0 % of 100 %, eenvoudig maar
  oscillerend rond het setpoint;
- **Cyclusrelais (PWM)** — een PID berekent een cyclusverhouding, *gehakt* over een
  vaste periode: de fysieke uitgang blijft aan-uit (0/100 %), maar het
  **gemiddelde** volgt de PID. Het beste compromis om een orgaan fijn te besturen
  dat alleen kan openen of sluiten (relais, aan-uit-klep).

> 👉 **Belangrijk — zie §6 (De regeling begrijpen)**: PID/TOR/PWM kiezen voor
> koud *wapent* het koud, maar dit **levert alleen wanneer de meting het setpoint
> overschrijdt**.

### 4.5 PID-instellingen (Kp, Ki, Kd)
Voor elke richting drie live aanpasbare gains:

- **Kp** (proportioneel): hoe groter, hoe levendiger de reactie (oscillatierisico);
- **Ki** (integraal): heft de restafwijking in de tijd op (te sterk → overschrijding);
- **Kd** (afgeleide): dempt/anticipeert (te sterk → gevoelig voor ruis).

### 4.6 TOR / PWM-instellingen
- **TOR-hysterese** — breedte van de **dode zone** van de aan-uit-modus, gecentreerd
  op het setpoint (`[SP − h/2, SP + h/2]`): voorkomt dat de uitgang voortdurend
  klapt. Hoe breder, hoe groter de rimpel maar hoe meer gespreid de omschakelingen.
- **TOR min. cyclus (s)** — minimale duur waarin het relais in een toestand blijft
  voordat het opnieuw kan omschakelen (**anti-kortsluitcyclus**). Beschermt een
  echte aandrijver (relais, compressor) en effent het gedrag. `0` = uitgeschakeld.
- **PWM-periode (s)** — duur van een cyclus van het **cyclusrelais**. Kort →
  getrouwer gemiddelde maar frequente omschakelingen; lang → minder slijtage maar
  meer uitgesproken rimpel. Te kiezen veel kleiner dan de tijdconstante van het
  proces.

---

## 5. De trendgrafiek lezen

De grafiek (in het midden) tekent drie grootheden in real time. De **legende,
linksboven**, herinnert aan de kleur **en de laatste waarde** van elke reeks:

| Kleur | Reeks | Betekenis |
|-------|-------|-----------|
| 🔵 blauw | **Setpoint (SP)** | doel (in Auto) |
| 🔴 rood | **Meting (PV)** | proceswaarde |
| 🟢 groen | **Uitgang (%)** | toegepaste besturing (**+** warm / **−** koud) |

Boven de grafiek tonen drie kaarten de momentane waarden (Meting, Actief setpoint,
Uitgang). Men kan de grafiek in-/uitzoomen en verschuiven met de muis.

---

## 6. De regeling begrijpen (warm / koud)

De regelaar werkt in **slechts één richting tegelijk**, gekozen volgens de
afwijking `Setpoint − Meting`:

| Situatie | Richting die werkt | Uitgang | Indicator |
|----------|--------------------|---------|-----------|
| Meting **<** Setpoint (er moet verwarmd worden) | **Richting 1 (warm)** | **positief** (0…+100 %) | **Warm actief = 1** |
| Meting **>** Setpoint (er moet gekoeld worden) | **Richting 2 (koud)** | **negatief** (−100…0 %) | **Koud actief = 1** |

Praktische gevolgen:

- **PID/TOR voor koud** selecteren volstaat niet om « Koud actief » aan te zetten:
  de **meting moet boven het setpoint** liggen. Zolang de meting eronder ligt, is
  het de **warme** richting die werkt.
- Om het koud te zien leveren: in **Auto**, richting 2 in PID/TOR, **verlaag het
  setpoint onder de huidige meting** (of wacht op een overschrijding). De uitgang
  wordt negatief en **Koud actief** gaat naar 1.
- In **TOR** kantelt het relais op de **halve hysterese** aan weerszijden van het
  setpoint (symmetrische dode zone) en respecteert het de **minimale cyclus** tussen
  twee omschakelingen. In **PWM** hakt de uitgang op 0/100 % maar volgt het
  gemiddelde de PID.

---

## 7. Parameters (knop ⚙)

De knop **⚙ Parameters** opent een venster om te configureren:

### Modbus-transport
Keuze van de communicatiebus — **slechts één tegelijk actief**:

**TCP (Ethernet)**
- **Luister-IP** (`0.0.0.0` = alle interfaces) en **Poort** (standaard 5502);
- **Toegestane IP's**: één per regel, jokers `*` aanvaard (bv. `192.168.1.*`).
  **Lege lijst = alle IP's toegestaan.** De andere worden geweigerd.

**RTU (RS485)** — vereist een binair gecompileerd met de feature `rtu`
- **Seriële poort**: `/dev/ttyUSB0`, `/dev/ttyAMA0` (Raspberry Pi), `COM3` (Windows)…;
- **Baud** (standaard 19200), **Pariteit** (standaard Even), **Databits** (8),
  **Stopbits** (1) — af te stemmen op de master;
- **Slave-adres** (1–247).

> ⚠️ **Slechts één externe master tegelijk.** In TCP **verbreekt** de verbinding van
> een nieuwe master automatisch de vorige. De lokale GUI is **geen** master: zij
> blijft altijd actief. In RTU, geef de voorkeur aan een **punt-tot-punt**-verbinding
> (het apparaat antwoordt ongeacht het gevraagde adres).

### Overdrachtsfunctie (proces)
Gesimuleerd fysiek gedrag `G(s) = K·e^(−L·s) / (1 + T·s)`:
- **Gain K**: variatie van de meting per % uitgang;
- **Constante T** (s): traagheid/snelheid;
- **Dode tijd L** (s): dode tijd vóór reactie;
- **Ambient**: rustwaarde.

### Setpointgrenzen
Min/max-limieten van het auto-setpoint.

Knoppen: **Toepassen** (treedt onmiddellijk in werking **en** slaat op),
**Standaardwaarden herstellen**, **Sluiten**.

### Instellingen opslaan
De instellingen worden **opgeslagen** in een bestand `mock_ru_modbustcp.toml`
(naast de software) en **herladen bij de volgende start**. De knop **💾
Instellingen opslaan** in de koptekst slaat ook de PID-gains, de hysterese, de
minimale TOR-cyclus en de PWM-periode op die vanuit het linkerpaneel zijn gewijzigd.

---

## 8. Een Modbus-client aansluiten

De software is een **Modbus-slave** (TCP poort 5502 standaard, of RTU serieel
naargelang het transport gekozen in § 7). Een client (PLC, SCADA, `mbpoll`…) kan de
toestand **lezen** en de setpoints/modi **schrijven**. Herinnering: **slechts één
externe master tegelijk** (in TCP verbreekt een nieuwkomer de vorige).

Belangrijkste referenties (adressen **base 0**):

| Gegeven | Tabel | Adres | Type | Toegang |
|---------|-------|-------|------|---------|
| Aan/Uit | Coil | 0 | bit | L/S |
| Auto/Handmatig | Coil | 1 | bit | L/S |
| Modus richting 1 / richting 2 | Holding | 0 / 1 | 0=Off,1=PID,2=TOR,3=PWM | L/S |
| Auto-setpoint | Holding | 2–3 | float | L/S |
| Handmatig setpoint | Holding | 4–5 | float | L/S |
| TOR min. cyclus (s) | Holding | 20–21 | float | L/S |
| PWM-periode (s) | Holding | 22–23 | float | L/S |
| Meting (PV) | Input | 0–1 | float | L |
| Uitgang (%) | Input | 2–3 | float | L |
| Identificatie « CESAM-Lab » | Holding | 42–46 | ASCII-tekst | L |

> De **volledige tabel** (PID-gains, hysterese, codering van de floats,
> functiecodes, `mbpoll`-voorbeelden) staat in **[table_modbus.md](table_modbus.md)**.
> Dezelfde tabel is ook **live** zichtbaar in het rechterpaneel van de GUI.

---

## 9. Gebruik zonder scherm (« headless » / Docker)

Voor een achtergrondimplementatie (Raspberry Pi zonder scherm, server) bestaat er
een versie **zonder interface**: zij laat de simulatie en de Modbus-server draaien,
**uitsluitend bestuurbaar via Modbus**.

```bash
# Docker-image (overal inzetbaar):
docker run --rm -p 5502:5502 -v "$PWD/conf:/data" orme:headless
```

De map gemount op `/data` maakt het mogelijk `mock_ru_modbustcp.toml` te
leveren/behouden.

---

## 10. Veelgestelde vragen

| Vraag / symptoom | Antwoord |
|------------------|----------|
| **« Koud actief » gaat niet naar 1 terwijl ik PID/TOR heb ingesteld.** | Normaal: het koud levert alleen als **de meting het setpoint overschrijdt**. Verlaag het setpoint onder de meting (Auto-modus). Zie **§ 6 (De regeling begrijpen)**. |
| De meting beweegt niet. | Controleer of het apparaat **In bedrijf** is, en setpoint/uitgang niet nul zijn. |
| In handmatig doet het wijzigen van de modi richting 1/2 niets. | Normaal: de modi gelden alleen in **Auto**. |
| De koptekst toont **Modbus ✖**. | Poort al in gebruik of < 1024 zonder rechten: wijzig de **poort** in ⚙ Parameters. |
| Mijn Modbus-client wordt geweigerd. | Zijn IP staat niet in de **witte lijst**: maak de lijst leeg of voeg een patroon toe (`192.168.1.*`). |
| De gelezen floats zijn incoherent. | Probleem met de **woordvolgorde** aan clientzijde (hoogwaardig woord eerst). Zie table_modbus.md. |
| Een via Modbus geschreven setpoint wordt genegeerd. | Een float beslaat **2 registers**: schrijf ze **samen**. |
| Mijn instellingen worden niet bewaard. | Klik op **Toepassen** / **💾 Opslaan**. Het bestand `mock_ru_modbustcp.toml` moet schrijfbaar zijn. |

---

*Bijbehorende technische documentatie: [conception.md](conception.md) ·
[table_modbus.md](table_modbus.md) · [maintenance.md](maintenance.md).*
