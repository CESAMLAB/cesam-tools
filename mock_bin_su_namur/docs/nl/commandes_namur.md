# NAMUR-commandoset — Gesimuleerde roerder (OSNE)

*🌍 [FR](../fr/commandes_namur.md) · [EN](../en/commandes_namur.md) · [DE](../de/commandes_namur.md) · [ES](../es/commandes_namur.md) · [IT](../it/commandes_namur.md) · [PT](../pt/commandes_namur.md) · **NL** · [PL](../pl/commandes_namur.md)*

> Crate: `mock_bin_su_namur` · Uitvoerbaar bestand: **OSNE** · Protocol: **NAMUR** (ASCII, slave)

Functionele referentie van het protocol. De **technische bron van waarheid** is
de header van [`src/namur.rs`](../../src/namur.rs).

---

## 1. Algemeen

| Element | Waarde |
|---------|--------|
| Transport | **TCP** (poort `4001` standaard) of **serieel RS-232** (feature `serial`) |
| Rol | **Slave** (beantwoordt de verzoeken van de master) |
| Frame | één **ASCII-regel** per verzoek, afgesloten met `CR LF` |
| Leesopdrachten | `IN_*` → geven `waarde kanaal` terug (bv. `1200.0 4`) |
| Schrijfopdrachten / acties | `OUT_*`, `START_*`, `STOP_*`, `RESET` → **stil** (geen antwoord) |
| Masters | **slechts één tegelijk** (punt-tot-punt); in TCP wacht een nieuwe master tot de vorige is losgekoppeld |
| Filtering | optionele IP-witlijst (TCP) |

> Typische seriële NAMUR-instelling: **9600 baud, 7 bits, even pariteit, 1 stop (7E1)**.

### Kanalen

| Kanaal | Grootheid | Eenheid |
|--------|-----------|---------|
| `4` | Snelheid | tr/min |
| `5` | Koppel | N·cm |

---

## 2. Commando's

| Commando | Type | Effect | Antwoord |
|----------|------|--------|----------|
| `IN_NAME` | lezen | Naam van het apparaat | `CESAM-STIRRER` |
| `IN_TYPE` | lezen | Type apparaat | `OSNE` |
| `IN_SW_VERSION` | lezen | Versie van de gesimuleerde firmware (alias: `IN_VERSION`) | bv. `0.2.0` |
| `IN_PV_4` | lezen | **Gemeten** snelheid | `<v> 4` |
| `IN_PV_5` | lezen | **Gemeten** koppel | `<c> 5` |
| `IN_SP_4` | lezen | Snelheidssetpoint | `<v> 4` |
| `OUT_SP_4 <v>` | schrijven | Snelheidssetpoint **instellen** (tr/min) | — |
| `START_4` | actie | Motor starten | — |
| `STOP_4` | actie | Motor stoppen | — |
| `RESET` | actie | Stop + terugkeer naar lokale besturing | — |
| `OUT_WD1@<m>` | schrijven | **Waakhond**: veilige stop als er gedurende `<m>` s geen commando komt | — |
| `OUT_WD2@<m>` | schrijven | Waakhond (idem v1: veilige stop) | — |

> Elk onbekend commando of ongeldig argument wordt **genegeerd** (geen antwoord) en
> geregistreerd in `debug`.

### Waakhond

Na `OUT_WD1@30`, als er gedurende 30 s **geen enkele regel** binnenkomt, wordt de
motor automatisch **gestopt** (`STOP`) — bescherming bij verlies van communicatie
met de supervisor. `OUT_WD1@0` (of een negatieve vertraging) schakelt de waakhond
uit. De teller wordt **bij elk ontvangen commando opnieuw ingesteld**. Eenmaal
**geactiveerd** schakelt de waakhond zichzelf **uit** (de motor staat al stil):
herstel hem met een nieuwe `OUT_WD1@<m>` om de bewaking te hervatten.

---

## 3. Voorbeelden (`nc` / netcat)

```text
$ nc 127.0.0.1 4001
IN_NAME
CESAM-STIRRER
OUT_SP_4 1200          (stil)
START_4                (stil)
IN_PV_4
1200.0 4
IN_PV_5
62.0 5
STOP_4                 (stil)
```

> Het afgelezen **koppel** stijgt met de ingestelde **viscositeit** (GUI-zijde) en
> de snelheid: `koppel ≈ belastingscoeff · viscositeit · snelheid + wrijving`. Bij hoge
> viscositeit verzadigt het koppel bij het motormaximum: het snelheidssetpoint wordt
> niet meer bereikt (**overbelasting**), een gedrag dat een echte roerder reproduceert.
