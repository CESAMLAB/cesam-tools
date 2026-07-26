# Gebruikershandleiding — EtherNet/IP-regelaar (OREE)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · **NL** · [PL](../pl/manuel_utilisateur.md)*

---

## 1. Waar dient het instrument voor

**OREE** simuleert een proces**regeleenheid** (PID + thermisch proces van de
eerste orde) en stelt deze beschikbaar als een **EtherNet/IP-adapter** (expliciete CIP-berichten).
Het dient om een supervisie of een EtherNet/IP-client te testen (pycomm3, RSLinx voor
lezen, rseip…) zonder echte hardware.

## 2. Aan de slag

```bash
cargo run -p mock_bin_ru_ethernetip        # GUI + EtherNet/IP-adapter
```

De server luistert standaard op `0.0.0.0:44818` (geen privilege vereist). De koptekst
geeft de status aan: **EtherNet/IP ●** (groen) met het luisteradres, of een
foutbericht (rood). Een oranje banner waarschuwt als de server **blootgesteld** is (alle
interfaces + lege toegangslijst).

## 3. Interface

- **Koptekst**: titel, knoppen *Parameters* / *Opslaan*, status aan/uit,
  luisterstatus EtherNet/IP, banner voor netwerkblootstelling.
- **Linkerpaneel (Opdrachten)**: *Aan/Uit*, *Automatische modus (PID)*,
  *Setpoint*, *Handmatige uitgang* (handmatige modus), **PID**-instellingen (Kp/Ki/Kd).
- **Centraal paneel**: kaarten *Meting / Setpoint / Uitgang* + **grafiek** in real-time.
- **Modaal *Parameters***: taal, updatecontrole, **netwerk EtherNet/IP** (IP
  voor luisteren, poort, **toegangslijst** met IP's — één patroon per regel, `*` = jokerteken),
  **proces** (K, τ, vertraging, omgeving), **setpointgrenzen**. *Toepassen* herstart het
  luisteren als het IP/de poort verandert en slaat de TOML op.

## 4. Een EtherNet/IP-client verbinden

De client maakt verbinding met het IP/de poort van de server (`RegisterSession`
automatisch), en leest/schrijft vervolgens de **benoemde tags** via expliciete berichten: `Setpoint`, `ProcessValue`,
`Output`, `ManualOutput`, `Run`, `Auto`, enz. (zie
[`reference_ethernetip.md`](reference_ethernetip.md)). ⚠️ De waarden zijn in
**little-endian** (REAL = `f32` LE).

## 5. FAQ

- **De client maakt geen verbinding** → controleer IP/poort (44818), de **toegangslijst**,
  de firewall.
- **Tag onvindbaar** → alleen de gedocumenteerde tags bestaan; de namen zijn
  hoofdlettergevoelig.
- **Mijn schrijfacties hebben geen effect** → alleen de aanstuurbare tags werken
  (`Setpoint`, `ManualOutput`, `Run`, `Auto`); de andere zijn alleen-lezen.
- **Waar staat het configuratiebestand?** → `mock_ru_ethernetip.toml` (huidige map;
  overschrijfbaar via `MOCK_CONFIG`).
