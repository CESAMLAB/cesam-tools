# ORME — gesimuleerde Modbus-regelaar

*🌍 [English](README.md) · [Français](README.fr.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Italiano](README.it.md) · [Português](README.pt.md) · **Nederlands** · [Polski](README.pl.md)*

> *Open Regulator Modbus Emulator* · pakket `mock_bin_ru_modbustcp` · binair `orme`

**Gesimuleerde** industriële regelaar, **Modbus TCP/RTU**-slave, met grafische
interface. Maakt deel uit van het workspace [`cesam-tools`](../README.nl.md).

## Functies

- Eerste-orde-proces + zuivere dode tijd (FOPDT-overdrachtsfunctie).
- Tweerichtingsregeling (warm / koud), elke richting in **PID** of **aan-uit**.
- Modi aan/uit en auto/handmatig; setpoints auto (fysiek) en handmatig (%).
- Modbus TCP-server die de volledige toestand blootstelt.
- `egui`-GUI met real-time trendgrafiek en instelling van de PID-gains.
- **Meertalige interface**: Frans, Engels, Duits, Spaans, Italiaans, Portugees,
  Nederlands, Pools (keuze in het *Parameters*-modaal, persistent).

## Starten

```bash
cargo run -p mock_bin_ru_modbustcp
# Alternatief configuratiebestand:
MOCK_CONFIG=./ma_config.toml cargo run -p mock_bin_ru_modbustcp
```

Luistert standaard op `0.0.0.0:5502`. De poort, het luister-IP en de IP-witte lijst
worden ingesteld in het **⚙ Parameters**-modaal en zijn persistent in TOML.

## Modbus-adrestabel

Codering van de floats: 2 registers, big-endian, hoogwaardig woord eerst.

### Coils (FC 1/5/15)

| Adr | Rol |
|----|------|
| 0 | Aan (1) / Uit (0) |
| 1 | Auto (1) / Handmatig (0) |

### Discrete ingangen (FC 2, alleen-lezen)

| Adr | Rol |
|----|------|
| 0 | In bedrijf |
| 1 | Richting 1 (warm) actief |
| 2 | Richting 2 (koud) actief |

### Holding-registers (FC 3/6/16)

| Adr | Type | Rol |
|-----|------|------|
| 0 | u16 | Modus richting 1 (0=Off, 1=PID, 2=TOR) |
| 1 | u16 | Modus richting 2 (0=Off, 1=PID, 2=TOR) |
| 2–3 | f32 | Automatisch setpoint (SP) |
| 4–5 | f32 | Handmatig setpoint (% uitgang, met teken) |
| 6–7 | f32 | Kp richting 1 |
| 8–9 | f32 | Ki richting 1 |
| 10–11 | f32 | Kd richting 1 |
| 12–13 | f32 | Kp richting 2 |
| 14–15 | f32 | Ki richting 2 |
| 16–17 | f32 | Kd richting 2 |
| 18–19 | f32 | TOR-hysterese |

### Ingangsregisters (FC 4, alleen-lezen)

| Adr | Type | Rol |
|-----|------|------|
| 0–1 | f32 | Meting (PV) |
| 2–3 | f32 | Toegepaste uitgang (% met teken: + warm / − koud) |

De bron van waarheid is het kopcommentaar van [`src/map.rs`](src/map.rs).

## Documentatie

Documentatie eigen aan deze applicatie (map [`docs/nl/`](docs/nl/)):

- [**Gebruikershandleiding**](docs/nl/manuel_utilisateur.md) — ingebruikname, besturing, parameters, FAQ.
- [Ontwerpdocument](docs/nl/conception.md) — architectuur, technische keuzes, regeltheorie.
- [Modbus-adrestabel](docs/nl/table_modbus.md) — volledig adresseringsplan, codering, voorbeelden.
- [Software-onderhoud](docs/nl/maintenance.md) — build, configuratie, uitbreiding, probleemoplossing.
