# ORME — regolatore simulato Modbus

*🌍 [English](README.md) · [Français](README.fr.md) · [Deutsch](README.de.md) · [Español](README.es.md) · **Italiano** · [Português](README.pt.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md)*

> *Open Regulator Modbus Emulator* · pacchetto `mock_bin_ru_modbustcp` · binario `orme`

Regolatore industriale **simulato**, slave **Modbus TCP/RTU**, con interfaccia
grafica. Fa parte del workspace [`cesam-tools`](../README.it.md).

## Funzionalità

- Processo del primo ordine + ritardo puro (funzione di trasferimento FOPDT).
- Regolazione bidirezionale (caldo / freddo), ogni verso in **PID** o
  **tutto-o-niente**.
- Modalità marcia/arresto e auto/manuale; setpoint auto (fisico) e manuale (%).
- Server Modbus TCP che espone l'intero stato.
- IHM `egui` con curva di andamento in tempo reale e regolazione dei guadagni PID.
- **Interfaccia multilingue**: francese, inglese, tedesco, spagnolo, italiano,
  portoghese, olandese, polacco (scelta nel modale *Parametri*, persistita).

## Lanciare

```bash
cargo run -p mock_bin_ru_modbustcp
# File di configurazione alternativo :
MOCK_CONFIG=./ma_config.toml cargo run -p mock_bin_ru_modbustcp
```

Ascolta per impostazione predefinita su `0.0.0.0:5502`. La porta, l'IP di ascolto e la lista bianca
di IP si regolano nel modale **⚙ Parametri** e sono persistiti in TOML.

## Tabella degli indirizzi Modbus

Codifica dei virgola mobile: 2 registri, big-endian, parola di peso maggiore per prima.

### Bobine (FC 1/5/15)

| Ind | Ruolo |
|----|------|
| 0 | Marcia (1) / Arresto (0) |
| 1 | Auto (1) / Manuale (0) |

### Ingressi discreti (FC 2, sola lettura)

| Ind | Ruolo |
|----|------|
| 0 | In marcia |
| 1 | Verso 1 (caldo) attivo |
| 2 | Verso 2 (freddo) attivo |

### Registri di mantenimento (FC 3/6/16)

| Ind | Tipo | Ruolo |
|-----|------|------|
| 0 | u16 | Modalità verso 1 (0=Off, 1=PID, 2=TOR) |
| 1 | u16 | Modalità verso 2 (0=Off, 1=PID, 2=TOR) |
| 2–3 | f32 | Setpoint automatico (SP) |
| 4–5 | f32 | Setpoint manuale (% uscita, con segno) |
| 6–7 | f32 | Kp verso 1 |
| 8–9 | f32 | Ki verso 1 |
| 10–11 | f32 | Kd verso 1 |
| 12–13 | f32 | Kp verso 2 |
| 14–15 | f32 | Ki verso 2 |
| 16–17 | f32 | Kd verso 2 |
| 18–19 | f32 | Isteresi TOR |

### Registri di ingresso (FC 4, sola lettura)

| Ind | Tipo | Ruolo |
|-----|------|------|
| 0–1 | f32 | Misura (PV) |
| 2–3 | f32 | Uscita applicata (% con segno: + caldo / − freddo) |

La fonte di verità è l'intestazione di [`src/map.rs`](src/map.rs).

## Documentazione

Documentazione propria di questa applicazione (cartella [`docs/it/`](docs/it/)):

- [**Manuale utente**](docs/it/manuel_utilisateur.md) — guida introduttiva, pilotaggio, parametri, FAQ.
- [Documento di progettazione](docs/it/conception.md) — architettura, scelte tecniche, teoria della regolazione.
- [Tabella di indirizzi Modbus](docs/it/table_modbus.md) — piano di indirizzamento completo, codifica, esempi.
- [Manutenzione software](docs/it/maintenance.md) — build, configurazione, estensione, risoluzione dei problemi.
