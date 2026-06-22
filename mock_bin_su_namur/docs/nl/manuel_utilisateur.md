# Gebruikershandleiding — OSNE (gesimuleerde NAMUR-laboratoriumroerder)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · **NL** · [PL](../pl/manuel_utilisateur.md)*

> **OSNE** — *Open Stirrer NAMUR Emulator* · binair `mock_bin_su_namur`
> (uitvoerbaar bestand `osne`) · MIT-licentie · Uitgever: **CESAM-Lab** · NAMUR-
> identiteit: naam `CESAM-STIRRER`, type `OSNE`.
>
> *Een laboratoriumroerder (type IKA) die enkel bestaat op uw NAMUR-verbinding —
> om supervisors, scripts en gateways te testen zonder echte hardware.*

Deze handleiding is bedoeld voor de **gebruiker** van de gesimuleerde roerder: hoe
hem te starten, te besturen vanuit de interface, te parametreren en aan te sluiten
via **NAMUR** (TCP of serieel RS-232). Geen programmeerkennis vereist.

---

## 1. Waarvoor dient deze software?

Hij simuleert een **laboratoriumroerder** (tafelroerder met schroef, type IKA):

- een realistische **fysieke motor**: de snelheid stijgt/daalt naargelang het
  toegepaste koppel, met een **snelle snelheidsregeling**;
- een **instelbare visceuze belasting**: hoe visceuzer het medium, hoe hoger het
  benodigde koppel — tot aan de **overbelasting** (onbereikbaar setpoint);
- een **NAMUR-server** (serieel ASCII-protocol van labo-apparaten) om hem te
  besturen/superviseren vanuit een andere software of een script;
- een **grafische interface** voor besturing, visualisatie en **protocoltests**
  (geïntegreerde NAMUR-miniterminal).

Het is een **testtool**: het maakt het mogelijk een supervisor, een
acquisitiescript of een gateway te ontwikkelen en te demonstreren **zonder echte
hardware**.

---

## 2. De software starten

Start het uitvoerbare bestand dat overeenkomt met uw systeem:

| Systeem | Bestand |
|---------|---------|
| Windows | `osne-windows-x86_64.exe` (dubbelklik) |
| Linux-pc | `./osne-linux-x86_64` |
| Raspberry Pi (scherm) | `./osne-rpi-arm64` |

Het venster opent en de **NAMUR-server start automatisch** (poort `4001`
standaard). De koptekst geeft de toestand aan:

- **● IN BEDRIJF / ● GESTOPT**: toestand van de motor;
- **NAMUR ● 0.0.0.0:4001** (groen): server luistert; **✖ …** (rood) bij een
  probleem (poort bezet, serieel niet beschikbaar…);
- een **verbindingsindicator**: in TCP toont hij de verbonden master (of « geen
  master »), in serieel een eenvoudige stip. Hij wordt **groen** wanneer er
  onlangs een frame is ontvangen (link actief), anders grijs.

> Zonder scherm (alleen server), zie **§ 9 (Gebruik zonder scherm)**.

---

## 3. De interface in één oogopslag

```
┌──────────────── En-tête : titre OSNE, ⚙ Paramètres, 💾 Sauvegarder, états & voyants ────────────────┐
├──────────────────┬──────────────────────────────────────────────────────────────────────────────────┤
│  COMMANDES        │   SUPERVISION                                                                      │
│  (gauche)         │   - cartes de valeurs (Vitesse / Couple / Viscosité / Surcharge)                  │
│  Marche/Arrêt     │   - COURBE de tendance temps réel (Consigne / Vitesse / Couple)                   │
│  Consigne vitesse │                                                                                   │
│  Viscosité        │                                                                                   │
│  Réglages PID     │                                                                                   │
├──────────────────┴──────────────────────────────────────────────────────────────────────────────────┤
│  ⇄ TRAMES NAMUR : mini-terminal (RX/TX) + ligne de commande + référence du protocole (à droite)       │
└───────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 4. De roerder besturen (linkerpaneel)

### 4.1 Aan / Uit
Knop **Aan / Uit**. Bij uit vertraagt de motor vrij tot stilstand (wrijving +
belasting), motorkoppel nul.

### 4.2 Snelheidssetpoint
Schuifregelaar **Snelheidssetpoint** (in `tr/min`), begrensd door de in de
*Parameters* ingestelde min/max-grenzen. Het is dezelfde grootheid als het
NAMUR-commando `OUT_SP_4` (kanaal 4). In bedrijf brengt de regeling de gemeten
snelheid naar dit setpoint.

### 4.3 Viscositeit van het medium
Schuifregelaar **Viscositeit** (logaritmische schaal). Hij vertegenwoordigt de
**belasting** van het geroerde medium:

- **lage** viscositeit → laag koppel, het setpoint wordt snel bereikt;
- **hoge** viscositeit → belangrijk belastingskoppel; als het benodigde koppel het
  **maximale motorkoppel** overschrijdt, wordt het snelheidssetpoint **niet meer
  bereikt** → de indicator **Overbelasting ⚠** licht op (gedrag van een echte
  roerder geconfronteerd met een te dik medium).

### 4.4 PID-instellingen (Kp, Ki, Kd)
De drie versterkingen van de snelheidsregeling, rechtstreeks aanpasbaar:

- **Kp** (proportioneel): hoe groter, hoe feller de snelheidsstijging (risico op
  overshoot/oscillatie);
- **Ki** (integraal): heft de resterende snelheidsafwijking in de tijd op;
- **Kd** (afgeleide): dempt/anticipeert (te sterk → gevoelig voor ruis).

> De standaardversterkingen zijn bewust « stug »: de uitgang verzadigt bij het
> maximale koppel zolang de fout groot is (snelle stijging), waarna de integrerende
> term stabiliseert. De uitgang van de PID **is** het motorkoppel, begrensd tot
> `[0, couple_max]`.

---

## 5. De trendgrafiek lezen

De grafiek (in het midden) tekent drie grootheden in real time. De **legende,
linksboven**, herinnert aan de kleur **en de laatste waarde** van elke reeks:

| Kleur | Reeks | Betekenis |
|-------|-------|-----------|
| 🔵 blauw | **Setpoint** | snelheidssetpoint (in bedrijf) |
| 🔴 rood | **Snelheid** | gemeten snelheid (`tr/min`, linkeras) |
| 🟢 groen | **Koppel** | gemeten koppel (`N·cm`, **rechteras**) |

> De grafiek heeft **twee verticale assen**: de **snelheid** (`tr/min`) links, het
> **koppel** (`N·cm`) rechts. Het koppel wordt geschaald om de grafiek te delen,
> maar de rechteras toont wel degelijk `N·cm`.

Boven de grafiek tonen **kaarten** de momentane waarden: **Snelheid**, **Koppel**,
**Viscositeit**, en **Overbelasting ⚠** wanneer de motor verzadigt. Men kan de
grafiek zoomen/verplaatsen met de muis.

---

## 6. De NAMUR-miniterminal (onderkant van het venster)

Het paneel **⇄ NAMUR-frames** maakt het mogelijk om het **protocol te testen**
rechtstreeks vanuit de GUI, zonder externe client:

- het **journaal** toont de **ontvangen** frames (`← RX`, blauw) en de
  **verzonden** frames (`→ TX`, groen), met tijdstempel;
- de **commandoregel** stuurt een NAMUR-frame naar de simulator (toets **Enter**
  of knop **▶ Verzenden**). De pijlen **↑/↓** roepen de vorige commando's op
  (geschiedenis);
- de **protocolreferentie** (rechterpaneel) somt de commando's op: een **klik**
  voegt het commando in de invoerregel in;
- de knop **🗑 Wissen** maakt het journaal leeg.

> De hier getypte frames worden exact geïnterpreteerd zoals die van een
> netwerkmaster: `OUT_SP_4 500` stelt het setpoint in, `START_4`/`STOP_4`
> starten/stoppen, `IN_PV_4` leest de snelheid, enz. De **waakhond**
> (`OUT_WD1@…`) heeft echter alleen effect binnen een echte netwerksessie (zie § 8).

---

## 7. Parameters (knop ⚙)

De knop **⚙ Parameters** opent een venster om te configureren:

### Taal van de interface
Selector bovenaan: **Français, English, Deutsch, Español, Italiano, Português,
Nederlands, Polski** (8 talen). De taal wordt bewaard.

### NAMUR-transport
Keuze van de verbinding — **slechts één actief tegelijk**:

**TCP (Ethernet)**
- **Luister-IP** (`0.0.0.0` = alle interfaces) en **Poort** (standaard 4001);
- **Toegestane IP's**: één per regel, jokers `*` toegestaan (bv. `192.168.1.*`).
  **Lege lijst = alle IP's toegestaan.** De andere worden geweigerd.

**Serieel (RS-232)** — vereist een binair gecompileerd met de feature `serial`
- **Seriële poort**: `/dev/ttyUSB0` (Linux), `COM3` (Windows)…;
- **Baud** (standaard 9600), **Pariteit** (standaard Even), **Databits** (7),
  **Stopbits** (1) — typische labo-NAMUR-instelling: **9600 7E1**.

> ⚠️ **Slechts één master tegelijk.** In TCP **wacht** een nieuwe master tot de
> vorige is losgekoppeld (punt-tot-punt-verbinding). De lokale GUI is **geen**
> master. In serieel *is* de bus de enige master; geef de voorkeur aan een
> **punt-tot-punt-verbinding** (de server antwoordt ongeacht het gevraagde adres).

### Motorparameters
Gesimuleerd fysiek gedrag `J·dω/dt = T − k·η·ω − frottement`:
- **Inertie** (`J`): reactiviteit van de motor (klein ⇒ snel);
- **Belastingscoëfficiënt** (`k`): gewicht van de viscositeit op het koppel;
- **Wrijving** (`N·cm`): resterende droge wrijving;
- **Max. koppel** (`N·cm`): maximaal motorkoppel (plafond van de PID-uitgang).

### Snelheidsgrenzen
Min/max-grenzen van het snelheidssetpoint (`tr/min`).

### Viscositeitsgrenzen
Min/max-grenzen van de viscositeitsschuifregelaar.

Knoppen: **Toepassen** (treedt onmiddellijk in werking **en** slaat op),
**Standaardwaarden herstellen**, **Sluiten**.

### Instellingen opslaan
De instellingen worden **opgeslagen** in een bestand `mock_su_namur.toml` (naast
de software) en **opnieuw geladen bij de volgende start**. De knop **💾 Opslaan**
in de koptekst slaat ook de PID-versterkingen en de viscositeit op die vanuit het
linkerpaneel zijn gewijzigd.

---

## 8. Een NAMUR-client aansluiten

De software is een **NAMUR-slave** (TCP-poort 4001 standaard, of serieel
naargelang het in § 7 gekozen transport). Een client (script, terminal, gateway)
**stuurt één ASCII-regel per verzoek**, afgesloten met `CR LF`. De
**leesopdrachten** (`IN_*`) geven een waarde terug; de **schrijfopdrachten/acties**
(`OUT_*`, `START_*`, `STOP_*`, `RESET`) zijn **stil** (geen antwoord),
overeenkomstig het NAMUR-gebruik.

Belangrijkste referenties:

| Commando | Effect |
|----------|--------|
| `IN_NAME` / `IN_TYPE` | identiteit (`CESAM-STIRRER` / `OSNE`) |
| `IN_PV_4` / `IN_PV_5` | de snelheid (`tr/min`) / het koppel (`N·cm`) lezen |
| `IN_SP_4` | het snelheidssetpoint lezen |
| `OUT_SP_4 <v>` | het snelheidssetpoint **instellen** |
| `START_4` / `STOP_4` / `RESET` | starten / stoppen / resetten |
| `OUT_WD1@<m>` | **waakhond**: veilige stop bij stilte gedurende `<m>` s |

Voorbeeld met `nc` (netcat):

```text
$ nc 127.0.0.1 4001
IN_NAME
CESAM-STIRRER
OUT_SP_4 1200          (silencieux)
START_4                (silencieux)
IN_PV_4
1200.0 4
STOP_4                 (silencieux)
```

> De **waakhond** `OUT_WD1@30` stopt automatisch de motor als er gedurende 30 s
> **geen enkele regel** binnenkomt (bescherming bij verlies van communicatie).
> `OUT_WD1@0` schakelt hem uit. De teller wordt bij elk ontvangen commando opnieuw
> ingesteld.

> De **volledige protocolreferentie** (kanalen, codering, voorbeelden) staat in
> **[commandes_namur.md](commandes_namur.md)**. Dezelfde lijst wordt **live**
> herhaald in het rechterpaneel van de miniterminal.

---

## 9. Gebruik zonder scherm (« headless » / Docker)

Voor een implementatie op de achtergrond (Raspberry Pi zonder scherm, server)
bestaat er een versie **zonder interface**: deze laat de simulatie en de
NAMUR-server draaien, **enkel bestuurbaar via NAMUR**.

```bash
# Image Docker (déployable n'importe où) :
docker run --rm -p 4001:4001 -v "$PWD/conf:/data" osne:headless
```

De map gekoppeld aan `/data` maakt het mogelijk om `mock_su_namur.toml` te leveren/
bewaren.

---

## 10. Veelgestelde vragen

| Vraag / symptoom | Antwoord |
|------------------|----------|
| **Overbelasting ⚠** licht op en de snelheid bereikt het setpoint niet. | Normaal: de **viscositeit** vraagt meer koppel dan de motor levert. Verlaag de viscositeit of het setpoint, of verhoog het **max. koppel** (Parameters). |
| De snelheid beweegt niet. | Controleer of de roerder **In bedrijf** is en het setpoint niet nul. |
| De koptekst toont **NAMUR ✖**. | Poort al gebruikt of < 1024 zonder rechten (TCP), of seriële poort niet beschikbaar: wijzig de instelling in ⚙ Parameters. |
| Mijn NAMUR/TCP-client wordt geweigerd. | Zijn IP staat niet in de **witlijst**: maak de lijst leeg of voeg een patroon toe (`192.168.1.*`). |
| `OUT_SP_4 …` geeft niets terug. | Normaal: de NAMUR-schrijfopdrachten/acties zijn **stil**. Lees met `IN_SP_4` / `IN_PV_4`. |
| De motor stopt vanzelf. | Een **waakhond** is geactiveerd (`OUT_WD1@…`) en er is geen commando op tijd binnengekomen. Schakel hem uit (`OUT_WD1@0`) of stuur regelmatig frames. |
| De seriële verbinding gaat niet open. | Binair gecompileerd **zonder** de feature `serial`, of verkeerde poort/permissies (groep `dialout` onder Linux). |
| Mijn instellingen worden niet bewaard. | Klik **Toepassen** / **💾 Opslaan**. Het bestand `mock_su_namur.toml` moet beschrijfbaar zijn. |

---

*Bijbehorende technische documentatie: [conception.md](conception.md) ·
[commandes_namur.md](commandes_namur.md) · [maintenance.md](maintenance.md).*
