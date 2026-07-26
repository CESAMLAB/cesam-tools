# Gebruikershandleiding — S7-regelaar (ORSS)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · **NL** · [PL](../pl/manuel_utilisateur.md)*

---

## 1. Waarvoor dient het instrument

**ORSS** simuleert een proces-**regeleenheid** (PID + thermisch proces van de eerste
orde) en stelt deze beschikbaar als een **Siemens S7-PLC** (S7comm-server op
ISO-on-TCP). Het dient om een supervisie of een S7-client (Snap7, TIA Portal in
leesmodus, nodes7…) te testen zonder echte PLC.

## 2. Snel aan de slag

```bash
cargo run -p mock_bin_ru_s7        # GUI + S7-server
```

De server luistert standaard op `0.0.0.0:102`. ⚠️ **Poort 102 vereist root-rechten**;
stel anders een hoge poort in (bijv. 1102) in het modaal *Parameters*.

De koptekst toont de status: **S7 ●** (groen) met het luisteradres, of een
foutmelding (rood) als de bind mislukt. Een oranje banner waarschuwt als de server
**blootgesteld** is (alle interfaces + lege toelatingslijst).

## 3. Interface

- **Koptekst**: titel, knoppen *Parameters* / *Opslaan*, start-/stopstatus,
  S7-luisterstatus, blootstellingsbanner van het netwerk.
- **Linkerpaneel (Commando's)**: *Start/Stop*, *Automatische modus (PID)*, *Setpoint*,
  *Handmatige uitgang* (handmatige modus), **PID**-instellingen (Kp/Ki/Kd).
- **Centraal paneel**: kaarten *Meting / Setpoint / Uitgang* + realtime **grafiek**.
- **Modaal *Parameters***: taal, updatecontrole, **S7-netwerk** (luister-IP, poort,
  **toelatingslijst** van IP's — één patroon per regel, `*` = jokerteken), **proces**
  (K, τ, vertraging, omgeving), **setpointgrenzen**. *Toepassen* herstart het luisteren
  als het IP/de poort verandert en slaat het TOML op.

## 4. Een S7-client verbinden

De client verbindt met het IP/de poort van de server. De gebruikelijke **rack/slot**-waarden
(0/1 of 0/2) werken: de server legt geen TSAP op. De grootheden bevinden zich in
**DB1** (zie [`reference_s7.md`](reference_s7.md)): setpoint in `DB1.DBD0`, meting in
`DB1.DBD4`, start in `DB1.DBX16.0`, enz.

## 5. FAQ

- **"Permission denied" bij het opstarten** → poort 102 vereist root-rechten; gebruik
  een hoge poort of start met de juiste privileges.
- **De client verbindt niet** → controleer IP/poort, de **toelatingslijst**, de
  firewall. Test rack/slot 0/1 en daarna 0/2.
- **Mijn schrijfacties hebben geen effect** → alleen de stuurbare offsets werken
  (setpoint, handmatige uitgang, start, auto); de overige zijn alleen-lezen.
- **Waar is het configuratiebestand?** → `mock_ru_s7.toml` (huidige map;
  overschrijfbaar via `MOCK_CONFIG`).
