# Gebruikershandleiding — Gesimuleerde PROFIBUS-DP-regelaar (ORPD)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · **NL** · [PL](../pl/manuel_utilisateur.md)*

> Crate: `mock_bin_ru_pbdp` · Uitvoerbaar bestand: **ru_pbdp** · Merk: **ORPD**

---

## ⚠️ Voordat u begint: wat deze simulator NIET is

`ru_pbdp` **is geen** hardwareconforme PROFIBUS-DP-slave. PROFIBUS DP is
een tokenbus waarvan de naleving van de tijdvensters (*slot time*, `Tsdr`,
watchdog) een dedicated circuit vereist (SPC3/VPC3-ASIC,
Hilscher/Softing/Siemens-CP-masterkaart). Een gewoon Tokio-programma, zelfs
aangesloten op een echte RS-485-poort, **kan deze beperkingen niet
naleven**: een echte PLC (bijvoorbeeld een Siemens S7 met masterkaart) zal
deze simulator **nooit** als geldige slave op een echte bus herkennen.

Wat `ru_pbdp` daadwerkelijk doet: het implementeert, **in software en
zonder realtime-beperkingen**, de framestructuur en de toestandsmachine
van een DP-V0-slave (parametrering, configuratie, diagnose, cyclische
uitwisseling). Het is een hulpmiddel om **het protocol te begrijpen** en
**een software-ontwikkeling te testen** (codec, toestandsmachine,
tooling) — niet om veldapparatuur aan te sturen. Zie
[reference_profibus.md](reference_profibus.md) §6 voor het detail van de
beperkingen.

---

## 1. Waarvoor dient deze simulator

`ru_pbdp` simuleert een **procesregelaar** (PID-lus op een thermisch
proces, model identiek aan ORME/Modbus) en stelt deze bloot via een
gesimuleerde set PROFIBUS-DP-V0-frames, over een seriële verbinding
(RS-485/RS-232). De grafische interface maakt het mogelijk de simulatie
te **besturen** en de dynamiek ervan te **visualiseren**; het framelog
toont het uitgewisselde verkeer in hexadecimaal.

---

## 2. Aan de slag

```bash
cargo run -p mock_bin_ru_pbdp          # GUI + seriële PROFIBUS-DP-verbinding
```

Bij het opstarten probeert de simulator de geconfigureerde seriële poort
te openen (standaard `/dev/ttyUSB0` of `COM3`, 500 kbit/s, stationsadres
3). Als de poort niet bestaat (vaak het geval zonder seriële hardware),
toont de GUI de openingsfout in de kop — de regelaarsimulatie blijft
draaien, alleen de verbinding is niet beschikbaar. Stel de **seriële
poort** in *Instellingen* in om te wijzen naar een beschikbare
pseudo-terminal of USB-seriële adapter.

---

## 3. De interface

### Kop

- **Titel** en knoppen **⚙ Instellingen** / **💾 Instellingen opslaan**.
- Rechts: **apparaatstatus** (IN WERKING / GESTOPT), **verbindingsstatus**
  (`PROFIBUS ● <poort> [<status>]` groen indien geopend — de getoonde
  status is die van de DP-V0-toestandsmachine:
  `Power_On`/`Wait_Prm`/`Wait_Cfg`/`Data_Exchange`), en het **CESAM-Lab-
  logo**.
- Een **permanente oranje balk** herinnert aan de niet-interoperabiliteit
  met echte hardware (zie de waarschuwing hierboven).

### Mini-terminal (onderaan het venster)

Alleen-lezen log van **ontvangen** (← RX) en **verzonden** (→ TX) frames,
met tijdstempel en hexadecimale weergave. Knop **Wissen** om het log te
legen.

### Bedieningspaneel (links)

Identiek aan ORME: **Start/Stop**, **Auto/Handmatig**, regelmodi voor
**richting 1 (verwarmen)** / **richting 2 (koelen)**
(Uit/PID/Aan-uit/PWM), **setpoints** (automatisch en handmatig), **PID-
instellingen** van beide richtingen, **hysterese**, **minimale aan-uit-
cyclus**, **PWM-periode**.

### Rechterpaneel: PROFIBUS-I/O-blokken

Livetabel van de *Output*-blokken (master→slave) en *Input*-blokken
(slave→master), met de door deze simulator gebruikte byte-indeling — zie
[reference_profibus.md](reference_profibus.md) §3.

### Middengebied

Kaarten **Meetwaarde**, **Actief setpoint**, **Uitgang**, en een
trendcurve.

---

## 4. Instellingen (⚙-modaal)

- Interface**taal** (8 talen), bewaard.
- **Bij opstarten controleren op updates** + knop **Nu controleren**.
- **Seriële poort**, **baudrate** (gebruik een genormaliseerde PROFIBUS-
  DP-waarde: 9600, 19200, 45450, 93750, 187500, 500000, 1500000, 3000000,
  6000000 of 12000000), **stationsadres** (0-125).
- **Protocolwatchdog (toegestaan)**: selectievakje — indien uitgeschakeld
  wordt de door de master via `Set_Prm` aangevraagde watchdog **genegeerd**
  (nooit bewapend).
- **Overdrachtsfunctie van het proces**: versterking `K`, tijdconstante
  `τ`, zuivere dode tijd, omgevingswaarde.
- **Setpointgrenzen**: min / max (automatisch herordend indien
  omgekeerd).
- **Toepassen** / **Standaard herstellen** / **Sluiten**.

Een wijziging van poort/baudrate/adres **sluit en heropent** de seriële
verbinding. De instellingen worden opgeslagen in `mock_ru_pbdp.toml`
(huidige map; overschrijfbaar via de omgevingsvariabele `MOCK_CONFIG`).

**Het frameformaat (8E1) is vastgelegd door de PROFIBUS-DP-norm** en is
hier niet instelbaar, in tegenstelling tot Modbus RTU of seriële NAMUR.

---

## 5. De mini-terminal als educatief hulpmiddel

Zonder echte PROFIBUS-hardware is de beste manier om het protocol te
observeren om **twee instanties** van dit hulpmiddel met elkaar te laten
communiceren — of een klein script te schrijven dat een sequentie
`Slave_Diag` → `Set_Prm` → `Chk_Cfg` → `Data_Exchange` afspeelt over een
pseudo-terminal (`socat -d -d pty,raw,echo=0 pty,raw,echo=0`) — en de
mini-terminal te lezen om de uitgewisselde frames in hexadecimaal te
zien, met hun decodering in
[reference_profibus.md](reference_profibus.md).

---

## 6. Veelgestelde vragen

**Kan ik deze simulator aansluiten op een echte PROFIBUS-DP-PLC?** Nee —
zie de waarschuwing bovenaan dit document en §6 van
[reference_profibus.md](reference_profibus.md).

**De seriële poort opent niet.** Het opgegeven bestand/apparaat bestaat
niet of de rechten zijn onvoldoende (`dialout`-groep onder Linux). De
exacte fout wordt getoond in de kop van de GUI.

**De verbinding blijft in `Wait_Prm`.** De master heeft nog geen
`Set_Prm` met de verwachte identificatie gestuurd (`0xEE01`, een
**fictieve** identificatie, niet geregistreerd bij de PNO). Zie
[reference_profibus.md](reference_profibus.md) §2.

**De verbinding blijft in `Wait_Cfg`.** De ontvangen `Chk_Cfg` kondigt
niet de verwachte I/O-lengtes aan (45 uitvoerbytes, 17 invoerbytes voor
deze simulator).

**Het apparaat stopt vanzelf.** De protocolwatchdog (bewapend door de
master via `Set_Prm`) is verlopen bij gebrek aan tijdig ontvangen
cyclische uitwisseling — dit is de verwachte veilige toestand, geen bug.

**Starten zonder grafische interface?** Compileer *headless*:
`cargo run -p mock_bin_ru_pbdp --no-default-features` — de seriële
verbinding en de simulatie draaien zonder GUI.
