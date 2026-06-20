<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="pic/Logo-CESAM-Couleur-vect-dark.png">
    <img src="pic/Logo-CESAM-Couleur-vect.png" alt="CESAM-Lab" height="84">
  </picture>
</p>

# cesam-tools — Cassetta degli attrezzi CESAM-Lab

*🌍 [English](README.md) · [Français](README.fr.md) · [Deutsch](README.de.md) · [Español](README.es.md) · **Italiano** · [Português](README.pt.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md)*

<p align="center">
  <a href="https://github.com/CESAMLAB/cesam-tools/releases/latest"><img src="https://img.shields.io/github/v/release/CESAMLAB/cesam-tools?label=release" alt="Latest release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue" alt="License: MIT"></a>
</p>

Workspace Rust che riunisce gli **strumenti di CESAM-Lab**, a cominciare da
**simulatori di strumenti industriali**: apparecchi virtuali che
riproducono un comportamento fisico realistico e comunicano tramite protocolli
di campo. Utile per sviluppare, testare e dimostrare supervisori, PLC
o gateway **senza hardware reale**.

> Distribuito gratuitamente sotto licenza [MIT](LICENSE).

## Strumenti disponibili

| Crate | Prodotto | Descrizione | Protocollo | IHM |
|-------|---------|-------------|-----------|-----|
| [`mock_bin_ru_modbustcp`](mock_bin_ru_modbustcp) | **ORME** | Regolatore (PID / TOR / PWM) su funzione di trasferimento | Modbus TCP & RTU (slave) | egui |

Libreria condivisa:

| Crate | Descrizione |
|-------|-------------|
| [`mock_lib_control`](mock_lib_control) | Blocchi di regolazione riutilizzabili: PID anti-windup, tutto-o-niente a isteresi, processo del 1° ordine + ritardo puro (FOPDT). |

## ORME — il regolatore simulato

<p align="center">
  <img src="pic/orme-logo.svg" alt="ORME — Open Regulator Modbus Emulator" height="120">
</p>

> **ORME** — *Open Regulator Modbus Emulator*. **«Aprite il bus.»**
> Un regolatore di campo che esiste solo sul vostro bus Modbus.

Un regolatore industriale virtuale completo:

- **Processo** modellato da una funzione di trasferimento del primo ordine con
  ritardo puro `K·e^(-Ls) / (1 + T·s)` (tipico di un forno o bagno termostatato).
- **Regolazione** bidirezionale: verso 1 (caldo) e verso 2 (freddo),
  ciascuno configurabile in **PID**, **tutto-o-niente (TOR)** o **relè a ciclo (PWM)**.
- **Modalità** marcia/arresto e automatico/manuale.
- **Server Modbus** in **TCP** o **RTU seriale / RS485** (feature `rtu`), a scelta.
  Tabella di indirizzi (setpoint, misura, uscita, modalità…), **lista bianca di IP**
  (jolly `*`) configurabile a caldo, e **politica mono-master** (un solo master
  remoto alla volta; in TCP un nuovo arrivato disconnette il precedente).
- **Interfaccia grafica** su una pagina: pilotaggio, **curva di andamento**
  in tempo reale, **tabella di indirizzi Modbus live**, e un **modale Parametri**
  (trasporto TCP/RTU, porta, IP autorizzate, parametri seriali, funzione di
  trasferimento, limiti di setpoint).
- **Configurazione persistita** in formato TOML (`mock_ru_modbustcp.toml`),
  ricaricata all'avvio, con pulsante di ripristino ai valori predefiniti.

### Architettura asincrona

```
        Command (cast non bloccante)           istantanea condivisa
  IHM (egui) ──────────────────────►  SimulationActor  ──────────►  IHM (lettura)
  Modbus scrittura ────────────────►   (ractor)         ──────────►  immagine Modbus
  Modbus lettura  ◄──────────────────────────────────────  immagine Modbus
```

- **`ractor`**: un attore unico possiede lo stato del regolatore; tutte le
  mutazioni passano per messaggi (nessun lock sulla logica di business).
- **`tokio-modbus`**: server Modbus TCP e RTU seriale (trait `Service`).
- **`eframe`/`egui`**: interfaccia grafica sul thread principale.

## Download

I binari precompilati sono disponibili nella pagina [**Releases**](https://github.com/CESAMLAB/cesam-tools/releases/latest) — **nessuna toolchain Rust necessaria**.

| Piattaforma | GUI | Headless (solo TCP, senza GUI) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`orme-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-linux-x86_64) | [`orme-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-linux-x86_64-headless) |
| Windows x86_64 | [`orme-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`orme-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-rpi-arm64) | [`orme-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-rpi-arm64-headless) |

```bash
chmod +x orme-linux-x86_64        # Linux / Raspberry Pi
./orme-linux-x86_64
```

I binari Linux/RPi sono collegati dinamicamente a glibc e richiedono un ambiente desktop (X11/Wayland) per la GUI. Su **Wayland**, installa la voce desktop per l'icona nella barra delle applicazioni: `scripts/install-desktop.sh`. Verifica l'integrità con i checksum pubblicati:

```bash
sha256sum -c SHA256SUMS
```

## Avvio rapido

```bash
# Prerequisiti : Rust stable (edizione 2021, >= 1.85).
# Dipendenze di sistema Linux per l'IHM : libxkbcommon, libwayland/xcb, openGL.

cargo run -p mock_bin_ru_modbustcp
```

La finestra si apre e il server Modbus TCP ascolta su `0.0.0.0:5502`.
La **porta**, l'**IP di ascolto** e la **lista bianca di IP** si regolano nel
modale **⚙ Parametri** (applicato a caldo) poi sono **persistiti** in
`mock_ru_modbustcp.toml`. La **lingua dell'interfaccia** (francese, inglese,
tedesco, spagnolo, italiano, portoghese, olandese, polacco) si sceglie in questo
stesso modale ed è persistita. Per usare un altro file di configurazione:

```bash
MOCK_CONFIG=/percorso/verso/ma_config.toml cargo run -p mock_bin_ru_modbustcp
```

### Testare la connessione Modbus

Con qualsiasi client Modbus (es. `mbpoll`):

```bash
# Mettere in marcia (bobina 0) poi leggere la misura (input registers 0-1, f32)
mbpoll -m tcp -a 1 -t 0 -p 5502 127.0.0.1 1      # scrivere la bobina On/Off
mbpoll -m tcp -a 1 -t 3:float -r 1 -p 5502 127.0.0.1   # leggere PV (f32)
```

La tabella di indirizzi completa è documentata in
[`mock_bin_ru_modbustcp/src/map.rs`](mock_bin_ru_modbustcp/src/map.rs).

## Sviluppo

```bash
cargo test --workspace      # test unitari + integrazione
cargo clippy --workspace    # lint
```

Vedi [CLAUDE.md](CLAUDE.md) per le convenzioni e l'architettura dettagliata.

## Documentazione

Ogni strumento ha la propria documentazione nella sua sottocartella `docs/`,
disponibile in otto lingue (`docs/<lingua>/`). Per il regolatore (versione
italiana):

- [**Manuale utente**](mock_bin_ru_modbustcp/docs/it/manuel_utilisateur.md) — guida introduttiva, IHM, parametri, FAQ.
- [Documento di progettazione](mock_bin_ru_modbustcp/docs/it/conception.md) — architettura e scelte tecniche.
- [Tabella di indirizzi Modbus](mock_bin_ru_modbustcp/docs/it/table_modbus.md) — piano di indirizzamento completo.
- [Manutenzione software](mock_bin_ru_modbustcp/docs/it/maintenance.md) — build, configurazione, estensione, risoluzione dei problemi.

## Marchio & loghi

I loghi sono in [`pic/`](pic/):

- [`orme-icon.svg`](pic/orme-icon.svg) / `orme-icon.png` — icona ORME (quadrante),
  anch'essa incorporata come icona di finestra dell'applicazione.
- [`orme-logo.svg`](pic/orme-logo.svg) — logo ORME completo (icona + testo).
- [`Logo-CESAM-Couleur-vect.png`](pic/Logo-CESAM-Couleur-vect.png) — logo CESAM-Lab.

L'icona ORME è **generata** da [`pic/orme-logo.gen.py`](pic/orme-logo.gen.py)
(`python3 pic/orme-logo.gen.py` produce gli `.svg`, da rasterizzare in seguito).

## Licenza

[MIT](LICENSE) © 2026 CESAM-Lab
