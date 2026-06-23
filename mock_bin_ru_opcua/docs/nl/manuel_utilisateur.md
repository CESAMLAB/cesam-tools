# Gebruikershandleiding — Gesimuleerde procesregelaar (RU/OPC UA)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · **NL** · [PL](../pl/manuel_utilisateur.md)*

> Crate: `mock_bin_ru_opcua` · Uitvoerbaar bestand: **ru_opcua**

---

## 1. Waarvoor dient deze simulator

`ru_opcua` simuleert een **procesregelaar** (PID-lus op een thermisch proces) en
stelt deze beschikbaar in **OPC UA**, de standaard voor industriële supervisie.
Hij dient om **een OPC UA-client / een SCADA te testen** (lezen van metingen,
schrijven van setpoints, abonnementen) zonder echte hardware.

De grafische interface maakt het mogelijk de simulatie te **besturen** en de
dynamiek te **visualiseren**; de OPC UA-server stelt dezelfde grootheden aan het
netwerk beschikbaar.

---

## 2. Aan de slag

```bash
cargo run -p mock_bin_ru_opcua          # GUI + OPC UA-server
```

Bij het opstarten luistert de server standaard op `opc.tcp://0.0.0.0:4840/`
(beveiliging None). Het venster toont de huidige toestand en start de trendcurve.

Verbind een OPC UA-client (UaExpert, enz.) met `opc.tcp://127.0.0.1:4840/`,
beveiliging **None**, gebruiker **Anonymous**. De nodes worden beschreven in de
[OPC UA-referentie](reference_opcua.md).

---

## 3. De interface

### Koptekst

- **Titel** en knoppen **⚙ Parameters** / **💾 Instellingen opslaan**.
- Rechts: **toestand van het apparaat** (IN BEDRIJF / GESTOPT), **toestand van de
  server** (`OPC UA ● opc.tcp://…` in het groen bij luisteren, ✖ + bericht bij een
  fout), en het **CESAM-Lab-logo**.
- Een **oranje banner** herinnert er permanent aan dat de endpoint **anoniem
  (beveiliging None)** is: alleen bloot te stellen op een vertrouwd netwerk.
- Als er een update beschikbaar is, biedt een **banner** de download aan.

### Bedieningspaneel (links)

- **Aan / Uit**: start of stopt de regeling. Bij stilstand ontspant het proces
  naar de omgevingswaarde.
- **Automatische modus (PID)**: ingeschakeld = de PID berekent de uitgang;
  uitgeschakeld = **handmatige modus** (de uitgang wordt opgelegd).
- **Setpoint**: schuifregelaar, begrensd door de setpointgrenzen (instelbaar in
  *Parameters*).
- **Handmatige uitgang (%)**: schuifregelaar alleen actief in **handmatige modus**.
- **PID-instellingen**: versterkingen `Kp`, `Ki`, `Kd` warm bewerkbaar.

### Centrale zone

- **Kaarten**: Meting, Setpoint, Uitgang.
- **Trendcurve**: Meting (PV) en Setpoint op de linkeras (proceseenheid),
  Uitgang (%) op de rechteras.

---

## 4. Parameters (modaal venster ⚙)

- **Taal** van de interface (8 talen), persistent.
- **Updates controleren bij het opstarten** + knop **Nu controleren**.
- **Endpoint**: **luister-IP** en **poort** van de OPC UA-server. Een wijziging
  **herstart** de server warm (de lopende sessies worden netjes gesloten).
- **OPC UA-beveiliging**: **Versleuteling** (`Basic256Sha256`), **Anoniem toestaan**,
  **Gebruiker** / **Wachtwoord** (velden actief wanneer versleuteling is aangevinkt).
  Versleuteling inschakelen genereert bij de eerste start een certificaat (enkele
  seconden) en herstart de server.
- **Proces (overdrachtsfunctie)**: versterking `K`, tijdconstante `τ`, zuivere
  dode tijd, omgevingswaarde.
- **Setpointgrenzen**: min / max (automatisch herordend indien omgekeerd).
- **Toepassen** / **Standaardwaarden herstellen** / **Sluiten**.

De instellingen worden opgeslagen in `mock_ru_opcua.toml` (huidige map; te
overschrijven via de omgevingsvariabele `MOCK_CONFIG`).

---

## 5. Beveiliging

De OPC UA-beveiliging is **instelbaar** in *Parameters*:

- **Zonder versleuteling** (standaard): endpoint **beveiliging None**, **anonieme**
  toegang — geen enkele bescherming. **Niet blootstellen op een open netwerk.** Een
  **oranje** banner herinnert daaraan.
- **Met versleuteling**: endpoint **`Basic256Sha256`** (ondertekend + versleuteld).
  De server genereert zijn certificaat bij de eerste start en aanvaardt de
  clientcertificaten. Men kan een **gebruiker / wachtwoord** vereisen en/of het
  anonieme token toestaan. Een **groene banner 🔒** bevestigt de versleuteling. Om
  verbinding te maken moet de client dan het beleid `Basic256Sha256` gebruiken en
  het servercertificaat vertrouwen (eerste uitwisseling).

Het wachtwoord wordt **in klare tekst** opgeslagen in het TOML-bestand: het is een
**simulator**, te gebruiken op een vertrouwd netwerk.

---

## 6. FAQ

**Is poort 4840 verplicht?** Nee: hij wordt ingesteld in *Parameters* (of via het
TOML-bestand). Een poort < 1024 vereist root-rechten.

**Mijn client ziet de nodes niet.** Controleer de verbinding met `opc.tcp://…:4840/`,
beveiliging **None**, gebruiker **Anonymous**, en vervolgens *Browse* onder de map
`Objects` (namespace `urn:cesam-lab:ru-opcua`).

**Een schrijfbewerking wordt geweigerd.** Het type moet overeenkomen (`Double` voor
de grootheden, `Boolean` voor `Run`/`Auto`); anders retourneert de server
`Bad_TypeMismatch`.

**Starten zonder grafische interface?** Compileer in *headless*:
`cargo run -p mock_bin_ru_opcua --no-default-features` — de OPC UA-server en de
simulatie draaien zonder GUI.

**Een bericht "encrypted endpoints disabled" verschijnt.** Dat is normaal in
Fase 1b: er wordt geen instantiecertificaat geleverd (versleutelde endpoints niet
beschikbaar). De None-endpoint zelf werkt wel.
