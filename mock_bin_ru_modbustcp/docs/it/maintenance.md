# Documentazione di manutenzione — ORME (workspace `cesam-tools`)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · **IT** · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

> Pubblico: sviluppatori che mantengono, correggono o estendono il progetto.
> Vedi anche: [conception.md](conception.md) · [table_modbus.md](table_modbus.md).

---

## 1. Prerequisiti

- **Rust stable** (edizione 2021, `rust-version` ≥ 1.85). Installazione: <https://rustup.rs>.
- **Dipendenze di sistema (Linux) per l'IHM** (`eframe`/`egui`, OpenGL/winit):
  - Debian/Ubuntu: `libxkbcommon-dev`, `libwayland-dev`, `libxcb1-dev`,
    `libgl1-mesa-dev` (o equivalenti), più un server grafico (X11/Wayland).
  - L'IHM richiede un **display**: in ambiente headless, la finestra non
    si apre (il server Modbus, invece, non dipende dal display).
- Accesso di rete al registro crates.io per la prima compilazione.

---

## 2. Comandi comuni

```bash
cargo check --workspace          # Verifica rapida (senza codegen)
cargo build --workspace          # Compilazione debug
cargo build --release            # Compilazione ottimizzata (LTO thin)
cargo test  --workspace          # Test unitari + integrazione
cargo clippy --workspace --all-targets   # Lint (deve restare SENZA avviso)
cargo run -p mock_bin_ru_modbustcp       # Lancia il regolatore

# File di configurazione alternativo:
MOCK_CONFIG=./ma_config.toml cargo run -p mock_bin_ru_modbustcp
# Registrazione dettagliata:
RUST_LOG=debug cargo run -p mock_bin_ru_modbustcp
```

Binario prodotto: `target/debug/orme` o `target/release/orme` (il pacchetto Cargo
resta `mock_bin_ru_modbustcp`, ma l'eseguibile si chiama **`orme`** — vedi
`[[bin]]` nel `Cargo.toml` del crate).

### Feature Cargo

| Feature | Predefinita | Effetto |
|---------|:---------:|-------|
| `gui` | ✅ | IHM `egui`/`eframe` (altrimenti binario headless) |
| `rtu` | ✅ | Trasporto Modbus RTU seriale (RS485) tramite `tokio-serial` |

```bash
cargo build --no-default-features                 # headless, Modbus TCP solo
cargo build --no-default-features --features rtu  # headless TCP + RTU seriale
cargo build --no-default-features --features gui  # IHM, TCP solo (senza seriale)
```

> ⚠️ **`rtu` = dipendenza nativa.** `tokio-serial` apre la porta tramite termios
> (Linux); l'enumerazione `libudev` è disattivata (`default-features = false`).
> In **cross-compilazione** (`build-prod.sh`, exe desktop con feature
> predefinite), l'immagine `cross` del target può comunque richiedere gli header seriali
> del sistema; se la toolchain crea problemi, rimuovere `rtu` dalla build interessata. Il
> **Docker headless non è impattato** (compila in `--no-default-features`).

---

## 3. Organizzazione del codice

```
mock_lib_control/        Libreria di regolazione (pura, senza IO, testabile)
  src/pid.rs             PID anti-windup
  src/onoff.rs           Tutto-o-niente a isteresi simmetrica + anti-corto-ciclo
  src/pwm.rs             Relè a ciclo (PWM / time-proportioning)
  src/process.rs         Funzione di trasferimento FOPDT
  src/lib.rs             ControllerKind + ri-export (feature `serde` opzionale)

mock_bin_ru_modbustcp/   Binario regolatore
  src/main.rs            Avvio: config, runtime Tokio, attori, IHM
  src/regulator.rs       Modello di business sincrono (stato, Command, step)
  src/config.rs          AppConfig (TOML), Transport/SerialConfig, IpFilter, ServerStatus
  src/map.rs             Piano di indirizzamento Modbus (FONTE DI VERITÀ)
  src/modbus_server.rs   RegulatorService (trait Service) + mono-master TCP + serve_rtu
  src/gui.rs             IHM egui (pagina unica + modale Parametri)
  src/actors/
    simulation.rs        Ciclo di regolazione (tick)
    network.rs           Server Modbus TCP/RTU (ri)configurabile a caldo

docs/                    Progettazione, tabella Modbus, manutenzione
```

**Regola d'oro**: la logica di business (`mock_lib_control`, `regulator.rs`) resta
**sincrona e testata**; l'asincrono è confinato agli attori e all'IO.

---

## 4. Configurazione

- File: `mock_ru_modbustcp.toml` nella directory corrente, o percorso
  fornito dalla variabile d'ambiente `MOCK_CONFIG`.
- Caricato all'avvio; **valori predefiniti** se assente o illeggibile (un
  avviso viene registrato, l'applicazione si avvia comunque).
- Salvato dall'IHM (pulsanti *Applica* / *Salva impostazioni* /
  *Ripristina predefiniti*).

Struttura (tutte le sezioni sono opzionali, completate con i predefiniti):

```toml
language = "it"
check_updates = true       # verifica all'avvio se esiste una release più recente (IHM)

[network]
bind_ip = "0.0.0.0"
port = 5502
allowlist = ["192.168.1.*", "127.0.0.1"]   # vuoto = tutte le IP autorizzate

[process]   # funzione di trasferimento G(s) = K·e^(-L·s)/(1+T·s)
gain = 1.6        # K (unità/%)
tau = 30.0        # T (s)
dead_time = 2.0   # L (s)
ambient = 20.0

[regulation]
sp_min = 0.0
sp_max = 250.0
hysteresis = 2.0
[regulation.pid_heat]   # verso 1 (caldo)
kp = 4.0 ; ki = 0.25 ; kd = 1.0 ; out_min = 0.0 ; out_max = 100.0
[regulation.pid_cool]   # verso 2 (freddo)
kp = 4.0 ; ki = 0.25 ; kd = 1.0 ; out_min = 0.0 ; out_max = 100.0
```

> I **valori predefiniti** hanno una **fonte unica**: `RegulatorConfig::default`
> in `regulator.rs`. `ProcessConfig`/`RegulationConfig` (config.rs) ne derivano.
> Per cambiare un predefinito, modificare solo `RegulatorConfig::default`.

### Verifica degli aggiornamenti

Se `check_updates = true` (predefinito) **e** il binario è compilato con la
feature `gui`, l'IHM interroga **all'avvio** l'ultima release pubblicata su
GitHub (`CESAMLAB/cesam-tools`) e ne confronta il numero con la versione
corrente. Una versione più recente mostra un banner cliccabile «🔔 Aggiornamento
disponibile». Il pulsante *Controlla ora* (modale *Impostazioni*) riavvia la
verifica.

- La richiesta HTTPS viene eseguita in un **thread dedicato**, limitata da un
  timeout (5 s): offline o GitHub irraggiungibile non ostacola mai l'avvio.
- La logica risiede nella crate condivisa **`mock_lib_update`** (`ureq`/`rustls`,
  radici Mozilla incorporate → cross-compilazione pulita con `cross`).
- **Build headless** (`--no-default-features`): la verifica — e tutta la
  dipendenza rete/TLS — è **assente**. Su server, gestire gli aggiornamenti via
  apt/Docker. Disattivabile dall'operatore (casella di spunta della modale).

---

## 5. Dipendenze e insidie di versione

| Crate | Ruolo | Punto di attenzione |
|-------|------|-------------------|
| `tokio` | runtime async | feature: `rt-multi-thread, macros, net, time, sync` |
| `ractor` | attori | feature predefinite (async nativo, **non** `async-trait`) |
| `tokio-serial` | Modbus RTU seriale | opzionale (feature `rtu`), `default-features = false` (nessuna enumerazione libudev) |
| `tokio-modbus` | Modbus TCP | `default-features = false`, feature **`tcp-server`** |
| `eframe`/`egui` | IHM | versioni collegate tra loro |
| `egui_plot` | curva | ⚠️ **versionato una minore in anticipo su `egui`**: per `egui` 0.33 → `egui_plot` **0.34** |
| `serde`/`toml` | persistenza | `mock_lib_control` espone una feature `serde` attivata dal binario |
| `mock_lib_update` (`ureq`/`rustls`) | verifica aggiornamenti | **solo feature `gui`**; rustls 0.23 (webpki aggiornato); assente in headless |

Le versioni condivise sono centralizzate in `[workspace.dependencies]` del
`Cargo.toml` radice. Per aggiornare `egui`/`eframe`, **verificare la versione
corrispondente di `egui_plot`** (altrimenti errore «two versions of crate egui»).

---

## 6. Estendere il progetto

### 6.1 Aggiungere un punto Modbus

Tutto avviene in **`map.rs`** (poi lo snapshot/Command se necessario):

1. Dichiarare la costante di indirizzo e regolare il `*_COUNT` della tabella interessata.
2. Compilare il valore in `MemoryMap::refresh_from` (stato → registro).
3. Se il punto è scrivibile, decodificarlo in `coil_to_command` /
   `holdings_to_commands` (registro → `Command`).
4. Aggiornare il commento doc d'intestazione **e** [table_modbus.md](table_modbus.md).
5. Aggiungere la riga nella tabella live dell'IHM (`gui.rs::modbus_rows`).

### 6.2 Aggiungere un comando / una regolazione

1. Variante in `enum Command` (`regulator.rs`) + trattamento in `Regulator::apply`.
2. Campo in `RegulatorSnapshot` se il valore deve essere osservabile.
3. Cablaggio IHM (`gui.rs`) e/o decodifica Modbus (`map.rs`).
4. Se persistente: campo in `AppConfig` (`config.rs`) + `to_regulator_config`.

### 6.3 Aggiungere un nuovo strumento

1. Creare `mock_bin_<nome>/` e aggiungerlo ai `members` del `Cargo.toml` radice.
2. Riutilizzare `mock_lib_control`; fattorizzare tutto ciò che è comune in una `mock_lib_*`.
3. Seguire la stessa suddivisione: modello sincrono, attore/i ractor, livello
   protocollo, IHM. Convenzione di nome: `mock_bin_<tipo>_<protocollo>`.

---

## 7. Strategia di test

- **Unitari** (`mock_lib_control`): PID (proporzionale, limitazione, anti-windup),
  TOR (zona morta), processo (convergenza a regime).
- **Dominio** (`regulator.rs`): convergenza PID in auto, uscita in manuale,
  ritorno all'ambiente all'arresto.
- **Mapping** (`map.rs`): round-trip `f32`↔registri, decodifica di scrittura,
  rifiuto di scrittura `f32` parziale.
- **Config / rete** (`config.rs`, `actors/network.rs`): round-trip TOML, filtro
  IP (jolly), avvio effettivo del server (bind su porta effimera).

Lanciare: `cargo test --workspace`. I test sono **deterministici e senza IHM**.

---

## 8. Risoluzione dei problemi

| Sintomo | Pista |
|----------|-------|
| «two versions of crate `egui`» | Disaccordo `egui_plot` / `egui`: allineare le versioni (§5). |
| L'IHM non si apre | Display assente (headless) o librerie di sistema mancanti (§1). |
| `Modbus ✖ ascolto fallito` nell'intestazione | Porta già in uso o < 1024 senza privilegi: cambiare la porta in *Parametri*. |
| Un client è rifiutato | IP fuori dalla **lista bianca**: svuotare la lista o aggiungere un pattern (`192.168.1.*`). |
| Valori `f32` aberranti lato client | Ordine delle parole (parola maggiore in testa): vedi [table_modbus.md](table_modbus.md). |
| Una scrittura di setpoint `f32` è ignorata | Scrivere **entrambi** i registri della coppia in una sola richiesta. |
| Config non ricaricata | Directory corrente errata o `MOCK_CONFIG`; verificare il log all'avvio. |
| Nessuna icona nella barra delle applicazioni (Linux) | Sessione **Wayland**: l'icona incorporata viene ignorata. Installare la voce desktop: `scripts/install-desktop.sh` (§9). |

Aumentare la verbosità: `RUST_LOG=debug` (o `trace`).

---

## 9. Build di distribuzione

```bash
cargo build --release
# Binario autonomo:
target/release/orme
```

Il profilo `release` attiva `lto = "thin"` e `opt-level = 3` (vedi `Cargo.toml`
radice). Per distribuire: fornire il binario + un `mock_ru_modbustcp.toml`
d'esempio. Licenza **MIT** (file `LICENSE`).

### Feature `gui` (build con / senza interfaccia)

L'IHM è dietro la feature Cargo **`gui`**, attivata per impostazione predefinita:

```bash
cargo build --release                       # con IHM (postazione di lavoro)
cargo build --release --no-default-features  # «headless»: Modbus + simulazione, senza IHM
```

La modalità **headless** è destinata ai deployment senza schermo (Raspberry Pi in
servizio) e rende la **cross-compilazione ARM banale** (nessuna dipendenza
grafica da collegare).

### Integrazione nel desktop Linux (icona della barra delle applicazioni)

L'icona ORME è incorporata nel binario (`branding.rs` → `with_icon`). Questo basta
sotto **X11, Windows e macOS**. Ma sotto **Wayland**, il compositore **ignora**
l'icona incorporata: associa la finestra al suo **`app_id`** («orme», definito in
`main.rs` tramite `ViewportBuilder::with_app_id`) a un file `orme.desktop` dallo
stesso nome, e mostra l'`Icon=` di questo file (risolto nel tema di icone
`hicolor`).

Per ottenere l'icona sotto Wayland, installare la voce desktop per l'utente
corrente:

```bash
scripts/install-desktop.sh
```

Lo script copia:

| Sorgente | Destinazione |
|----------|--------------|
| `pic/orme-icon.png` | `~/.local/share/icons/hicolor/256x256/apps/orme.png` |
| `packaging/orme.desktop` | `~/.local/share/applications/orme.desktop` |

poi aggiorna le cache (`gtk-update-icon-cache`, `update-desktop-database`). L'icona
appare al successivo avvio di ORME (e in modo affidabile dopo un nuovo login della
sessione Wayland).

> ⚠️ Tre nomi **devono restare allineati**: l'`app_id` (`main.rs`), il nome del file
> `orme.desktop` e il suo `StartupWMClass`, e il nome dell'icona `orme.png`
> (= `Icon=orme`). `packaging/orme.desktop` presuppone un eseguibile `orme` nel
> `PATH` (campo `Exec=`); in sviluppo (`cargo run`) questo campo non ha alcuna
> incidenza sulla visualizzazione dell'icona.

---

## 10. Build «prod» — cross-compilazione da Linux

### Procedura unica

Tutto è prodotto **da Linux** tramite
[`scripts/build-prod.sh`](../../../scripts/build-prod.sh), che costruisce **tutti gli
strumenti del workspace** (ORME *e* OSNE) in una sola passata. Per ogni strumento
(`<bin>` = `orme`, `osne`):

| Output | Target | IHM | Metodo |
|--------|-------|-----|---------|
| `dist/<bin>-linux-x86_64` | `x86_64-unknown-linux-gnu` | ✅ | `cross` |
| `dist/<bin>-windows-x86_64.exe` | `x86_64-pc-windows-gnu` | ✅ | `cross` (mingw) |
| `dist/<bin>-rpi-arm64` | `aarch64-unknown-linux-gnu` (Pi 3/4/5, Pi OS 64b) | ✅ | `cross` |
| Immagine Docker headless `<bin>:headless` | multi-arch `linux/amd64` + `linux/arm64` | ❌ | `docker buildx` |
| `dist/<bin>_<ver>_amd64.deb` / `_arm64.deb` | pacchetto Debian/Ubuntu | ✅ | `dpkg-deb` |
| `dist/<bin>-setup-x86_64.exe` | installer Windows | ✅ | NSIS (`makensis`) |

```bash
# Prerequisiti (una volta) — Docker deve essere in esecuzione:
cargo install cross

# Produrre tutto (exe ORME + OSNE in dist/ + immagini Docker locali amd64 caricate):
scripts/build-prod.sh

# Variante: immagini Docker MULTI-ARCH inviate a un registro (<prefisso>/<bin>:latest):
IMAGE_PREFIX=ghcr.io/<account> scripts/build-prod.sh

# Costruire un solo strumento:
ONLY=orme scripts/build-prod.sh
```

### Perché `cross` per TUTTE le build (compreso Linux x86_64)

`cross` fornisce immagini Docker contenenti le toolchain di ogni target: né
`mingw-w64`, né toolchain ARM, né *sysroot* da installare.

⚠️ **Non mescolare `cargo` nativo e `cross` nello stesso `target/`.** Entrambi
usano versioni di `rustc` differenti (host vs container); le
**proc-macro** compilate dall'uno sono rifiutate dall'altro, da cui errori
`can't find crate for …_derive` (es. `zerofrom_derive`, `tracing_attributes`).
Lo script passa quindi **sempre per `cross`**, anche per Linux x86_64 — una sola
toolchain, build riproducibili. (Se l'errore si presenta comunque dopo una
build nativa precedente: `rm -rf target/release` poi rilanciare.)

### IHM cross-compilata verso ARM: perché funziona

`eframe`/`egui` caricano OpenGL, X11/Wayland e xkbcommon **all'esecuzione**
(`dlopen`): il binario collega in fase di build solo la `libc`. Nessuna libreria grafica ARM
è quindi necessaria lato cross. Sul Raspberry Pi, prevedere un ambiente
desktop (mesa/X11 o Wayland) — presente su Raspberry Pi OS *Desktop*.

> Per un **Raspbian 32 bit**, puntare a `armv7-unknown-linux-gnueabihf` (adattare
> i target nello script).

### Immagine Docker headless «ovunque»

L'immagine ([`docker/Dockerfile.headless`](../../../docker/Dockerfile.headless)) parte da
`debian:bookworm-slim` e **copia** il binario headless dell'architettura voluta
(nessuna compilazione nell'immagine → niente QEMU). `docker buildx` assembla il
multi-arch `amd64`+`arm64`. Il server ascolta su `5502`. Montare un volume su
`/data` per fornire/persistere `mock_ru_modbustcp.toml`.

```bash
# Senza registro: immagine locale amd64 caricata, testabile immediatamente
docker run --rm -p 5502:5502 -v "$PWD/conf:/data" orme:headless
```

### Installer (`.deb` Linux/RPi + setup Windows)

A fine build, `build-prod.sh` chiama
[`scripts/make-installers.sh <bin>`](../../../scripts/make-installers.sh), che
trasforma gli eseguibili release di `dist/` in **installer**:

| Installer | Sorgente | Contenuto | Strumento |
|------------|--------|---------|-------|
| `<bin>_<ver>_amd64.deb` | `dist/<bin>-linux-x86_64` | binario → `/usr/bin`, voce di desktop, icona hicolor | `dpkg-deb` |
| `<bin>_<ver>_arm64.deb` | `dist/<bin>-rpi-arm64` | idem (Raspberry Pi OS 64 bit) | `dpkg-deb` |
| `<bin>-setup-x86_64.exe` | `dist/<bin>-windows-x86_64.exe` | exe + collegamenti (menu Start/desktop) + disinstaller | NSIS (`makensis`) |

- I `.deb` installano l'icona e il `.desktop`; un `postinst` aggiorna le cache
  delle icone e il database `.desktop`. Dipendenze: `libc6`; raccomandazioni
  grafiche (`libgl1`, `libxkbcommon0`, `libwayland-client0`).
- L'installer Windows proviene da
  [`packaging/windows/installer.nsi`](../../../packaging/windows/installer.nsi);
  i suoi collegamenti portano un'icona `.ico` multi-risoluzione derivata da
  `pic/<bin>-icon.png` (tramite Pillow).
- **Prerequisiti**: `dpkg-deb` (Debian/Ubuntu) per i `.deb`, **`makensis`**
  (`sudo apt install nsis`) per il setup Windows, `python3`+Pillow per il `.ico`.
  Ogni target il cui strumento/artefatto manca è **avvisato e saltato** (la build
  non si rompe). Disattivare tramite `INSTALLERS=0`, oppure (ri)generare solo gli
  installer di uno strumento: `scripts/make-installers.sh orme`. La **versione**
  dei pacchetti proviene da `[workspace.package].version`.

### Build nativa Windows (MSVC) — opzionale

Il `.exe` prodotto sopra è **GNU/mingw** (eseguibile Windows nativo, IHM
inclusa). Se è richiesto un binario **MSVC**, compilare su una macchina Windows
con [`scripts/build-windows.ps1`](../../../scripts/build-windows.ps1) (prerequisiti:
Rust + *Visual Studio Build Tools*, carico «Sviluppo Desktop in C++»), o
da Linux tramite `cargo-xwin` (`cargo xwin build --release --target x86_64-pc-windows-msvc`).

### Note

- I binari sono **collegati dinamicamente alla glibc**; compilati tramite `cross`
  (baseline glibc datata) girano su distribuzioni recenti (e in
  `debian:bookworm-slim`). Per un binario totalmente statico, puntare a `*-musl`.
- `dist/` è ignorato da git (artefatti di build).

---

## 11. Convenzioni

- Codice e commenti in **francese**.
- `cargo clippy --workspace` **senza avviso** prima di ogni commit.
- Ogni nuovo comportamento di business o di mapping è accompagnato da un **test**.
- Il piano di indirizzamento si modifica in **`map.rs`** (fonte di verità), con aggiornamento
  congiunto della documentazione.
