# ORME — simulierter Modbus-Regler

*🌍 [English](README.md) · [Français](README.fr.md) · **Deutsch** · [Español](README.es.md) · [Italiano](README.it.md) · [Português](README.pt.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md)*

> *Open Regulator Modbus Emulator* · Paket `mock_bin_ru_modbustcp` · Binary `orme`

**Simulierter** Industrieregler, **Modbus-TCP/RTU**-Slave, mit grafischer
Oberfläche. Teil des Workspaces [`cesam-tools`](../README.de.md).

## Funktionen

- Prozess erster Ordnung + reine Totzeit (FOPDT-Übertragungsfunktion).
- Bidirektionale Regelung (heiß / kalt), jede Richtung als **PID** oder
  **Zweipunkt**.
- Modi Start/Stopp und auto/manuell; Sollwerte auto (physikalisch) und manuell (%).
- Modbus-TCP-Server, der den gesamten Zustand bereitstellt.
- IHM `egui` mit Echtzeit-Trendkurve und Einstellung der PID-Verstärkungen.
- **Mehrsprachige Oberfläche**: Französisch, Englisch, Deutsch, Spanisch,
  Italienisch, Portugiesisch, Niederländisch, Polnisch (Wahl im Modal
  *Parameter*, persistiert).

## Starten

```bash
cargo run -p mock_bin_ru_modbustcp
# Alternative Konfigurationsdatei:
MOCK_CONFIG=./ma_config.toml cargo run -p mock_bin_ru_modbustcp
```

Lauscht standardmäßig auf `0.0.0.0:5502`. Der Port, die Lausch-IP und die
IP-Whitelist werden im Modal **⚙ Parameter** eingestellt und in TOML persistiert.

## Modbus-Adresstabelle

Kodierung der Gleitkommawerte: 2 Register, big-endian, höchstwertiges Wort zuerst.

### Spulen (FC 1/5/15)

| Adr | Rolle |
|-----|-------|
| 0 | Start (1) / Stopp (0) |
| 1 | Auto (1) / Manuell (0) |

### Diskrete Eingänge (FC 2, nur Lesen)

| Adr | Rolle |
|-----|-------|
| 0 | In Betrieb |
| 1 | Richtung 1 (heiß) aktiv |
| 2 | Richtung 2 (kalt) aktiv |

### Halteregister (FC 3/6/16)

| Adr | Typ | Rolle |
|-----|-----|-------|
| 0 | u16 | Modus Richtung 1 (0=Off, 1=PID, 2=TOR) |
| 1 | u16 | Modus Richtung 2 (0=Off, 1=PID, 2=TOR) |
| 2–3 | f32 | Automatischer Sollwert (SP) |
| 4–5 | f32 | Manueller Sollwert (% Ausgang, vorzeichenbehaftet) |
| 6–7 | f32 | Kp Richtung 1 |
| 8–9 | f32 | Ki Richtung 1 |
| 10–11 | f32 | Kd Richtung 1 |
| 12–13 | f32 | Kp Richtung 2 |
| 14–15 | f32 | Ki Richtung 2 |
| 16–17 | f32 | Kd Richtung 2 |
| 18–19 | f32 | TOR-Hysterese |

### Eingangsregister (FC 4, nur Lesen)

| Adr | Typ | Rolle |
|-----|-----|-------|
| 0–1 | f32 | Messwert (PV) |
| 2–3 | f32 | Angewandter Ausgang (% vorzeichenbehaftet: + heiß / − kalt) |

Die Quelle der Wahrheit ist der Kopf von [`src/map.rs`](src/map.rs).

## Dokumentation

Anwendungsspezifische Dokumentation (Ordner [`docs/de/`](docs/de/)):

- [**Benutzerhandbuch**](docs/de/manuel_utilisateur.md) — Einstieg, Steuerung, Parameter, FAQ.
- [Entwurfsdokument](docs/de/conception.md) — Architektur, technische Entscheidungen, Regelungstheorie.
- [Modbus-Adresstabelle](docs/de/table_modbus.md) — vollständiger Adressplan, Kodierung, Beispiele.
- [Softwarewartung](docs/de/maintenance.md) — Build, Konfiguration, Erweiterung, Fehlerbehebung.
