# Documentazione di manutenzione — OSNE (workspace `cesam-tools`)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · **IT** · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

> Pubblico: sviluppatori che mantengono, correggono o estendono il progetto.
> Vedi anche: [conception.md](conception.md) · [commandes_namur.md](commandes_namur.md).

---

## 1. Prerequisiti

- **Rust stable** (edizione 2021, `rust-version` ≥ 1.85). Installazione: <https://rustup.rs>.
- **Dipendenze di sistema (Linux) per l'IHM** (`eframe`/`egui`, OpenGL/winit):
  - Debian/Ubuntu: `libxkbcommon-dev`, `libwayland-dev`, `libxcb1-dev`,
    `libgl1-mesa-dev` (o equivalenti), più un server grafico (X11/Wayland).
  - L'IHM richiede un **display**: in ambiente headless, la finestra non si apre
    (il server NAMUR, invece, non dipende dal display).
- **Collegamento seriale** (feature `serial`): accesso alla porta (`/dev/ttyUSB*`,
  gruppo `dialout` su Linux). Senza hardware, usare il trasporto **TCP**.
- Accesso di rete al registro crates.io per la prima compilazione.

---

## 2. Comandi correnti

```bash
cargo check -p mock_bin_su_namur          # Verifica rapida (senza codegen)
cargo build -p mock_bin_su_namur          # Compilazione debug
cargo build --release -p mock_bin_su_namur   # Compilazione ottimizzata (LTO thin)
cargo test  -p mock_bin_su_namur          # Test unitari + integrazione
cargo clippy --workspace --all-targets    # Lint (deve restare SENZA avviso)
cargo run   -p mock_bin_su_namur          # Lancia l'agitatore (IHM + NAMUR/TCP)

# File di configurazione alternativo:
MOCK_CONFIG=./ma_config.toml cargo run -p mock_bin_su_namur
# Registrazione dettagliata:
RUST_LOG=debug cargo run -p mock_bin_su_namur
```

Binario prodotto: `target/debug/osne` o `target/release/osne` (il pacchetto Cargo
resta `mock_bin_su_namur`, ma l'eseguibile si chiama **`osne`** — vedi `[[bin]]`
nel `Cargo.toml` del crate).

### Feature Cargo

| Feature | Predefinita | Effetto |
|---------|:---------:|-------|
| `gui` | ✅ | IHM `egui`/`eframe` (altrimenti binario headless) |
| `serial` | ✅ | Trasporto NAMUR su collegamento seriale RS-232 tramite `tokio-serial` |

```bash
cargo build -p mock_bin_su_namur --no-default-features                  # headless, NAMUR/TCP solo
cargo build -p mock_bin_su_namur --no-default-features --features serial # headless TCP + seriale
cargo build -p mock_bin_su_namur --no-default-features --features gui    # IHM, TCP solo (senza seriale)
```

> ⚠️ **`serial` = dipendenza nativa.** `tokio-serial` apre la porta tramite termios
> (Linux); l'enumerazione `libudev` è disattivata (`default-features = false`).
> In **cross-compilazione** (`build-prod.sh`, exe desktop con feature predefinite),
> l'immagine `cross` del target può comunque richiedere gli header seriali; se la
> toolchain crea problemi, rimuovere `serial` dal build interessato. Il **Docker
> headless non è impattato** (compila in `--no-default-features`).

---

## 3. Organizzazione del codice

```
mock_lib_control/        Libreria di regolazione (pura, senza IO, testabile)
  src/pid.rs             PID anti-deriva (riutilizzato per l'asservimento di velocità)
  src/lib.rs             ri-export (feature `serde` opzionale)

mock_bin_su_namur/       Binario agitatore (eseguibile `osne`)
  src/main.rs            Avvio: config, runtime Tokio, attori, IHM
  src/motor.rs           Modello fisico del motore (dinamica rotazionale, Euler)
  src/stirrer.rs         Modello di dominio sincrono (stato, Command, step) — possiede il PID
  src/config.rs          AppConfig (TOML), Transport/SerialConfig, IpFilter, ServerStatus
  src/namur.rs           Protocollo NAMUR: handle_line (FONTE DI VERITÀ del set di comandi)
  src/namur_server.rs    Servizio NAMUR (righe ASCII) + mono-master TCP + serve seriale + watchdog
  src/trace.rs           Giornale circolare delle trame (mini-terminale IHM)
  src/gui.rs             IHM egui (pagina unica + mini-terminale + modale Impostazioni)
  src/branding.rs        Logo incorporati (feature `gui`)
  src/i18n.rs            Catalogo i18n tipizzato (8 lingue), senza dipendenza
  src/actors/
    simulation.rs        Ciclo di simulazione (tick 20 ms)
    network.rs           Servizio NAMUR TCP/seriale (ri)configurabile a caldo

docs/                    Concezione, comandi NAMUR, manuale, manutenzione (multilingua)
```

**Regola d'oro**: la logica di dominio (`mock_lib_control`, `motor.rs`,
`stirrer.rs`) resta **sincrona e testata**; l'asincrono è confinato agli attori e
all'IO. Calco esatto del regolatore **ORME** (`mock_bin_ru_modbus`) — stessi
invarianti.

---

## 4. Configurazione

- File: `mock_su_namur.toml` nella directory corrente, o percorso fornito dalla
  variabile d'ambiente `MOCK_CONFIG`.
- Caricato all'avvio; **valori predefiniti** se assente o illeggibile (un
  avvertimento è registrato, l'applicazione si avvia comunque).
- **Ogni valore proveniente dal TOML è sanificato** (`AppConfig::sanitized`):
  limiti riordinati (`min ≤ max`), float forzati finiti, inerzia/coppia/viscosità
  strettamente positivi. **Invariante: non eseguire mai `f32::clamp` con limiti non
  validati** (panic se `min > max` o `NaN`).
- Salvato dall'IHM (pulsanti *Applica* / *Salva* / *Ripristina*).

Struttura (tutte le sezioni sono opzionali, completate per default):

```toml
language = "it"
check_updates = true       # verifica all'avvio se esiste una release più recente (IHM)

[network]
transport = "tcp"          # "tcp" o "serial"
bind_ip = "0.0.0.0"
port = 4001
allowlist = ["192.168.1.*", "127.0.0.1"]   # vuoto = tutti gli IP autorizzati
[network.serial]
port = "/dev/ttyUSB0"
baud = 9600 ; parity = "even" ; data_bits = 7 ; stop_bits = 1   # NAMUR 7E1

[motor]   # J·dω/dt = T − k·η·ω − attrito
inertia = 0.02      # J (reattività)
load_coeff = 0.05   # k (peso della viscosità)
friction = 2.0      # N·cm
torque_max = 100.0  # N·cm (limite massimo dell'uscita PID)

[regulation]
speed_min = 0.0 ; speed_max = 2000.0
viscosity = 1.0 ; viscosity_min = 0.1 ; viscosity_max = 20.0
[regulation.pid]
kp = ... ; ki = ... ; kd = ... ; out_min = 0.0 ; out_max = 100.0
```

> I **valori predefiniti** hanno una **fonte unica**: `StirrerConfig::default` in
> `stirrer.rs`. `MotorConfig`/`RegulationConfig` (config.rs) ne derivano. I limiti
> di uscita del PID (`out_min`/`out_max`) sono **forzati** a `[0, couple_max]` al
> momento di costruire l'agitatore (`to_stirrer_config`).

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

## 5. Dipendenze e trappole di versione

| Crate | Ruolo | Punto di attenzione |
|-------|------|-------------------|
| `tokio` | runtime async | feature condivise + **`io-util`** (BufReader / righe ASCII NAMUR) |
| `ractor` | attori | feature predefinite (async nativo, **non** `async-trait`) |
| `tokio-serial` | NAMUR seriale | opzionale (feature `serial`), `default-features = false` (nessuna enumerazione libudev) |
| `eframe`/`egui` | IHM | versioni collegate tra loro |
| `egui_plot` | curva | ⚠️ **versionato una minore in anticipo su `egui`**: per `egui` 0.33 → `egui_plot` **0.34** |
| `serde`/`toml` | persistenza | `mock_lib_control` espone una feature `serde` attivata dal binario |
| `mock_lib_update` (`ureq`/`rustls`) | verifica aggiornamenti | **solo feature `gui`**; rustls 0.23 (webpki aggiornato); assente in headless |

Le versioni condivise sono centralizzate in `[workspace.dependencies]` del
`Cargo.toml` radice. Per aggiornare `egui`/`eframe`, **verificare la versione
corrispondente di `egui_plot`** (altrimenti errore «two versions of crate egui»).

---

## 6. Estendere il progetto

### 6.1 Aggiungere un comando NAMUR

Tutto avviene in **`namur.rs`** (fonte di verità del protocollo):

1. Aggiungere il ramo in `handle_line` (lettura → `Reply`, scrittura/azione →
   `Apply(Command)` o `SetWatchdog`).
2. Se è un'**azione**, aggiungere la variante in `enum Command` (`stirrer.rs`) e il
   suo trattamento in `Stirrer::apply`.
3. Aggiornare il doc-commento di intestazione, **[commandes_namur.md](commandes_namur.md)**
   e la tabella di riferimento del mini-terminale (`gui.rs`, tabella `rows`).
4. Aggiungere un test nel modulo `tests` di `namur.rs`.

### 6.2 Aggiungere un comando / un'impostazione IHM

1. Variante in `enum Command` (`stirrer.rs`) + trattamento in `Stirrer::apply`.
2. Campo in `StirrerSnapshot` se il valore deve essere osservabile.
3. Cablaggio IHM (`gui.rs`) tramite un `cast` non bloccante.
4. Se persistente: campo in `AppConfig` (`config.rs`) + sanificazione in
   `sanitized` + riporto in `to_stirrer_config`.

### 6.3 Aggiungere una stringa di interfaccia (i18n)

Ogni stringa IHM **deve** passare per una chiave `Msg` (`i18n.rs`) con le sue **8
traduzioni** (tabella di dimensione fissa verificata alla compilazione). Gli
acronimi NAMUR, i suffissi di unità e i nomi dei comandi restano codificati in
modo fisso.

### 6.4 Aggiungere un nuovo strumento

1. Creare `mock_bin_<nom>/` e aggiungerlo ai `members` del `Cargo.toml` radice.
2. Riutilizzare `mock_lib_control`; fattorizzare tutto ciò che è comune in una
   `mock_lib_*` (es. promozione del modello `motor.rs` se serve a un secondo
   strumento).
3. Seguire la stessa suddivisione: modello sincrono, attore(i) ractor, livello
   protocollo, IHM. Convenzione di nome: `mock_bin_<type>_<protocole>`.

---

## 7. Strategia di test

- **Unitari** (`mock_lib_control`): PID (proporzionale, limitazione, anti-windup).
- **Motore** (`motor.rs`): dinamica rotazionale, convergenza a regime stabilizzato,
  effetto della viscosità sulla coppia, saturazione/sovraccarico.
- **Dominio** (`stirrer.rs`): convergenza della velocità verso il riferimento,
  decelerazione all'arresto, rilevamento di sovraccarico.
- **Protocollo** (`namur.rs`): decodifica delle letture (`IN_*`), delle scritture
  (`OUT_SP_4`), delle azioni (`START/STOP/RESET`), del watchdog e dei comandi
  sconosciuti.
- **Config / rete** (`config.rs`, `actors/network.rs`): round-trip TOML, filtro IP
  (jolly, IPv4-mapped), sanificazione senza panic, apertura seriale in errore su
  porta assente.

Lanciare: `cargo test -p mock_bin_su_namur` (o `--workspace`). I test sono
**deterministici e senza IHM**.

---

## 8. Risoluzione dei problemi

| Sintomo | Pista |
|----------|-------|
| «two versions of crate `egui`» | Disaccordo `egui_plot` / `egui`: allineare le versioni (§5). |
| L'IHM non si apre | Display assente (headless) o librerie di sistema mancanti (§1). |
| `NAMUR ✖` nell'intestazione | Porta TCP già in uso / < 1024 senza privilegi, o porta seriale non disponibile: cambiare nei *Parametri*. |
| Un client TCP è rifiutato | IP fuori dalla **lista bianca**: svuotare la lista o aggiungere un pattern (`192.168.1.*`). |
| La seriale non si apre | Feature `serial` assente, porta errata, o permessi (`dialout`). |
| Il motore si arresta da solo | **Watchdog** armato (`OUT_WD1@…`) senza traffico: inviare trame o `OUT_WD1@0`. |
| Sovraccarico permanente | Viscosità troppo elevata vs `torque_max`: regolare i parametri motore. |
| Config non ricaricata | Directory corrente errata o `MOCK_CONFIG`; verificare il giornale all'avvio. |

Aumentare la verbosità: `RUST_LOG=debug` (o `trace`).

---

## 9. Build di distribuzione

```bash
cargo build --release -p mock_bin_su_namur
# Binario autonomo:
target/release/osne
```

Il profilo `release` attiva `lto = "thin"` e `opt-level = 3` (vedi `Cargo.toml`
radice). Per distribuire: fornire il binario + un `mock_su_namur.toml` di esempio.
Licenza **MIT** (file `LICENSE`).

### Feature `gui` (build con / senza interfaccia)

```bash
cargo build --release -p mock_bin_su_namur                       # con IHM (postazione di lavoro)
cargo build --release -p mock_bin_su_namur --no-default-features  # «headless»: NAMUR + simulazione, senza IHM
```

La modalità **headless** è destinata ai deployment senza schermo e rende la
**cross-compilazione ARM banale** (nessuna dipendenza grafica da collegare).

### Integrazione nel desktop Linux (icona della barra delle applicazioni)

L'icona OSNE (`pic/osne-icon.png`, motivo agitatore, generata da
[`pic/osne-logo.gen.py`](../../../pic/osne-logo.gen.py)) è **incorporata** nel
binario (`branding.rs` → `window_icon`). Questo è sufficiente sotto **X11, Windows
e macOS**. Sotto **Wayland**, il compositore **ignora** l'icona incorporata:
associa la finestra al suo **`app_id`** («osne», definito in `main.rs` tramite
`with_app_id`) a un file `osne.desktop` dello stesso nome, e mostra l'`Icon=osne`
risolta nel tema di icone `hicolor`.

Per ottenere l'icona sotto Wayland, installare la voce di desktop per l'utente
corrente:

```bash
scripts/install-desktop.sh osne
```

Lo script copia:

| Sorgente | Destinazione |
|----------|--------------|
| `pic/osne-icon.png` | `~/.local/share/icons/hicolor/256x256/apps/osne.png` |
| `packaging/osne.desktop` | `~/.local/share/applications/osne.desktop` |

poi aggiorna le cache. Tre nomi **devono restare allineati**: l'`app_id`
(`main.rs`), il file `osne.desktop` (+ il suo `StartupWMClass`) e l'icona
`osne.png` (= `Icon=osne`). Lo stesso script installa ORME senza argomenti
(`scripts/install-desktop.sh`).

---

## 10. Build «prod» — cross-compilazione da Linux

### Procedura unica

Tutto è prodotto **da Linux** da
[`scripts/build-prod.sh`](../../../scripts/build-prod.sh), che costruisce **tutti
gli strumenti del workspace** (ORME *e* OSNE):

| Output | Target | IHM | Metodo |
|--------|-------|-----|---------|
| `dist/osne-linux-x86_64` | `x86_64-unknown-linux-gnu` | ✅ | `cross` |
| `dist/osne-windows-x86_64.exe` | `x86_64-pc-windows-gnu` | ✅ | `cross` (mingw) |
| `dist/osne-rpi-arm64` | `aarch64-unknown-linux-gnu` (Pi 3/4/5, Pi OS 64b) | ✅ | `cross` |
| Image Docker headless `osne:headless` | multi-arch `amd64` + `arm64` | ❌ | `docker buildx` |
| `dist/osne_<ver>_amd64.deb` / `_arm64.deb` | pacchetto Debian/Ubuntu | ✅ | `dpkg-deb` |
| `dist/osne-setup-x86_64.exe` | installer Windows | ✅ | NSIS (`makensis`) |

```bash
# Prerequisiti (una volta) — Docker deve essere in esecuzione:
cargo install cross

# Produrre tutto (exe ORME + OSNE + installer in dist/ + immagini Docker amd64):
scripts/build-prod.sh

# Variante: immagini Docker MULTI-ARCH inviate a un registro:
IMAGE_PREFIX=ghcr.io/<account> scripts/build-prod.sh

# Senza costruire gli installer:
INSTALLERS=0 scripts/build-prod.sh
```

### Perché `cross` per TUTTI i build (compreso Linux x86_64)

`cross` fornisce immagini Docker contenenti le toolchain di ciascun target.
⚠️ **Non mescolare `cargo` nativo e `cross` nello stesso `target/`.** Le
**proc-macro** compilate dall'uno sono rifiutate dall'altro (`can't find crate for
…_derive`). Lo script passa **sempre per `cross`**. (Se l'errore si verifica:
`rm -rf target/release` poi rilanciare.)

### IHM cross-compilata verso ARM: perché funziona

`eframe`/`egui` caricano OpenGL, X11/Wayland e xkbcommon **all'esecuzione**
(`dlopen`): il binario collega al build solo la `libc`. Nessuna lib grafica ARM è
necessaria lato cross; prevedere un ambiente desktop sul target.

### Immagine Docker headless

L'immagine ([`docker/Dockerfile.headless`](../../../docker/Dockerfile.headless))
parte da `debian:bookworm-slim` e **copia** il binario headless dell'architettura
voluta (nessuna compilazione nell'immagine → niente QEMU). Il nome del binario e la
porta esposta sono passati tramite `--build-arg` (`BIN=osne`, `PORT=4001`). Montare
un volume su `/data` per fornire/persistere `mock_su_namur.toml`.

```bash
docker run --rm -p 4001:4001 -v "$PWD/conf:/data" osne:headless
```

### Installer (`.deb` Linux/RPi + setup Windows)

Alla fine di ogni build, `build-prod.sh` chiama
[`scripts/make-installers.sh <bin>`](../../../scripts/make-installers.sh), che
trasforma gli eseguibili release di `dist/` in **installer**:

| Installer | Sorgente | Contenuto | Strumento |
|------------|--------|---------|-------|
| `osne_<ver>_amd64.deb` | `dist/osne-linux-x86_64` | binario → `/usr/bin`, voce di desktop, icona hicolor | `dpkg-deb` |
| `osne_<ver>_arm64.deb` | `dist/osne-rpi-arm64` | idem (Raspberry Pi OS 64 bit) | `dpkg-deb` |
| `osne-setup-x86_64.exe` | `dist/osne-windows-x86_64.exe` | exe + collegamenti (menu Start/desktop) + disinstaller | NSIS (`makensis`) |

- I `.deb` installano l'icona e il `.desktop`; un `postinst` aggiorna le cache
  (`update-desktop-database`, `gtk-update-icon-cache`). Dipendenze: `libc6`;
  raccomandazioni grafiche (`libgl1`, `libxkbcommon0`, `libwayland-client0`).
- L'installer Windows è generato a partire da
  [`packaging/windows/installer.nsi`](../../../packaging/windows/installer.nsi);
  i collegamenti usano un'icona `.ico` multi-risoluzione derivata da
  `pic/osne-icon.png` (tramite Pillow).
- **Prerequisiti**: `dpkg-deb` (presente su Debian/Ubuntu) per i `.deb`,
  **`makensis`** (`sudo apt install nsis`) per il setup Windows, `python3`+Pillow
  per il `.ico`. Ogni target il cui strumento o artefatto manca è **avvisato e
  saltato** (la build non si rompe). Disattivare tramite `INSTALLERS=0`. Si possono
  anche (ri)generare solo gli installer di uno strumento:
  `scripts/make-installers.sh osne`.
- La **versione** dei pacchetti proviene da `[workspace.package].version` del `Cargo.toml`
  radice.

### Note

- I binari sono **collegati dinamicamente alla glibc**; compilati tramite `cross`
  (baseline glibc vecchia) girano su distribuzioni recenti.
- `dist/` è ignorato da git (artefatti di build).

---

## 11. Convenzioni

- Codice e commenti in **francese**; log e messaggi di errore in **inglese**.
- `cargo clippy --workspace` **senza avvertimenti** prima di ogni commit.
- Ogni nuovo comportamento di dominio, di motore o di protocollo è accompagnato da
  un **test**.
- Il set di comandi NAMUR si modifica in **`namur.rs`** (fonte di verità), con
  aggiornamento congiunto della documentazione.
