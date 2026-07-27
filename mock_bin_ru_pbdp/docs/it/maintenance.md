# Documentazione di manutenzione — ORPD / PROFIBUS DP (workspace `cesam-tools`)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · **IT** · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

> Crate: `mock_bin_ru_pbdp` · Eseguibile: **ru_pbdp** · Marchio: **ORPD**
> Pubblico: sviluppatori che mantengono, correggono o estendono il progetto.
> Vedere anche: [conception.md](conception.md) · [reference_profibus.md](reference_profibus.md).

---

## 1. Prerequisiti

- **Rust stable** (edizione 2021, `rust-version` ≥ 1.85). Installazione:
  <https://rustup.rs>.
- **Dipendenze di sistema (Linux) per la GUI** (`eframe`/`egui`,
  OpenGL/winit): `libxkbcommon-dev`, `libwayland-dev`, `libxcb1-dev`,
  `libgl1-mesa-dev` (o equivalenti), più un server grafico (X11/Wayland).
  La GUI necessita di un **display**: in un ambiente headless, la
  finestra non si apre.
- **Collegamento seriale** (accesso alla porta, `/dev/ttyUSB*`, gruppo
  `dialout` su Linux): a differenza di ORME/OSNE, **non è una funzione
  opzionale** qui — `tokio-serial` è una dipendenza diretta (vedere §5),
  essendo il collegamento seriale l'unico trasporto di questo strumento
  (non esiste un equivalente standard di «PROFIBUS su TCP»). Senza
  hardware, la GUI si avvia comunque (l'errore di apertura viene mostrato
  nell'intestazione, la simulazione continua a funzionare) — vedere
  [`manuel_utilisateur.md`](manuel_utilisateur.md) §2.
- Accesso di rete al registro crates.io per la prima compilazione.

---

## 2. Comandi comuni

```bash
cargo check -p mock_bin_ru_pbdp          # Verifica rapida (senza codegen)
cargo build -p mock_bin_ru_pbdp          # Compilazione debug
cargo build --release -p mock_bin_ru_pbdp   # Compilazione ottimizzata (LTO thin)
cargo test  -p mock_bin_ru_pbdp          # Test unitari + di integrazione
cargo clippy --workspace --all-targets    # Lint (deve restare SENZA avvisi)
cargo run   -p mock_bin_ru_pbdp          # Avvia la GUI + il collegamento seriale PROFIBUS DP

# File di configurazione alternativo:
MOCK_CONFIG=./mia_config.toml cargo run -p mock_bin_ru_pbdp
# Log dettagliato:
RUST_LOG=debug cargo run -p mock_bin_ru_pbdp
```

Binario prodotto: `target/debug/ru_pbdp` o `target/release/ru_pbdp` (il
pacchetto Cargo resta `mock_bin_ru_pbdp`; l'eseguibile e il nome
commerciale «ORPD» sono solo documentali, vedere `[[bin]]` nel
`Cargo.toml` del crate).

### Feature Cargo

| Feature | Predefinita | Effetto |
|---------|:---------:|-------|
| `gui` | ✅ | GUI `egui`/`eframe` + verifica aggiornamenti (altrimenti un binario headless) |

```bash
cargo build -p mock_bin_ru_pbdp --no-default-features   # headless: collegamento seriale + simulazione, senza GUI
```

> ⚠️ **Differenza con ORME/OSNE**: in questi due strumenti, il
> collegamento seriale (RTU/seriale) è esso stesso una **funzione
> opzionale** accanto a un trasporto TCP sempre presente, e
> `--no-default-features` può escluderlo. Qui **non esiste una variante
> «senza seriale»**: `tokio-serial` è una dipendenza diretta (non
> controllata da feature), presente in **ogni** compilazione, incluso
> headless — è l'unico trasporto dello strumento.

---

## 3. Organizzazione del codice

```
mock_lib_control/        Libreria di regolazione riutilizzabile (pura, senza IO, testabile)
  src/pid.rs             PID anti-windup
  src/lib.rs             ri-esportazioni (feature `serde` opzionale)

mock_bin_ru_pbdp/        Binario regolatore PROFIBUS DP (eseguibile `ru_pbdp`)
  src/main.rs            Avvio: configurazione, runtime Tokio, attori, GUI/headless
  src/regulator.rs        Modello di dominio sincrono (PID + processo di 1° ordine), Command, passo
  src/config.rs           AppConfig (TOML), SerialConfig, ProcessConfig, RegulationConfig, ServerStatus
  src/profibus.rs         Protocollo PROFIBUS DP-V0: codec trame + FCS + SlaveFsm (FONTE DI VERITÀ)
  src/profibus_server.rs  Ciclo di sessione seriale (lettura trama → SlaveFsm → risposta) + watchdog
  src/map.rs              Disposizione dei blocchi I/O Output/Input <-> Command del regolatore
  src/trace.rs            Registro circolare delle trame (mini-terminale GUI)
  src/gui.rs              GUI egui (pagina unica + mini-terminale + modale Impostazioni)
  src/branding.rs         Loghi incorporati (feature `gui`)
  src/i18n.rs             Catalogo i18n tipizzato (8 lingue), senza dipendenza
  src/actors/
    simulation.rs         Ciclo di regolazione (passo di simulazione 50 ms)
    network.rs            Attore del collegamento seriale PROFIBUS DP, riconfigurabile a caldo

docs/                     Progettazione, riferimento PROFIBUS, manuale, manutenzione (multilingue)
```

**Regola d'oro**: la logica di dominio (`mock_lib_control`, `regulator.rs`,
`profibus.rs`, `map.rs`) resta **sincrona e testata**; l'asincrono è
confinato agli attori e all'IO seriale. Modello di regolazione ricalcato
su **ORME** (`mock_bin_ru_modbus`) — stesse invarianti.

---

## 4. Configurazione

- File: `mock_ru_pbdp.toml` nella directory corrente, o un percorso
  fornito tramite la variabile d'ambiente `MOCK_CONFIG`.
- Caricato all'avvio; **valori predefiniti** se assente o illeggibile
  (viene registrato un avviso, l'applicazione si avvia comunque).
- **Ogni valore proveniente dal TOML viene sanificato**
  (`AppConfig::sanitized`): limiti di setpoint/PID riordinati, valori in
  virgola mobile forzati finiti, `τ ≥ 1e-3`, `dead_time` limitato,
  **indirizzo di stazione limitato a `[0, 125]`**. **Invariante: mai
  chiamare `f32::clamp` con limiti non validati** (va in panico se
  `min > max` o `NaN`).
- Salvato dalla GUI (pulsanti *Applica* / *Salva* / *Ripristina
  predefiniti*).

Struttura (tutte le sezioni sono opzionali, completate con valori
predefiniti):

```toml
language = "it"
check_updates = true       # verificare all'avvio se esiste una versione più recente (GUI)

[network.serial]
port = "/dev/ttyUSB0"      # "COM3" per default su Windows
baud = 500000              # valore normalizzato PROFIBUS DP (9600 .. 12000000)
station_address = 3        # indirizzo dello slave simulato (0-125)
watchdog_enabled = true    # consente il watchdog annunciato dal master (Set_Prm)

[process]
gain = 1.6 ; tau = 30.0 ; dead_time = 2.0 ; ambient = 20.0

[regulation]
sp_min = 0.0 ; sp_max = 250.0
hysteresis = 2.0 ; tor_min_cycle = 5.0 ; pwm_period = 10.0
[regulation.pid_heat]
kp = ... ; ki = ... ; kd = ... ; out_min = 0.0 ; out_max = 100.0
[regulation.pid_cool]
kp = ... ; ki = ... ; kd = ... ; out_min = 0.0 ; out_max = 100.0
```

> Il **formato di trama seriale (8E1)** è fissato dalla norma PROFIBUS DP
> e **non** è un campo di configurazione — vedere `SerialConfig::open`
> in [`config.rs`](../../src/config.rs). A differenza di ORME/OSNE,
> **nessuna lista bianca IP** (il collegamento seriale è intrinsecamente
> punto-punto).

### Verifica degli aggiornamenti

Se `check_updates = true` (predefinito) **e** il binario è compilato con
la feature `gui`, la GUI interroga **all'avvio** l'ultima release
pubblicata su GitHub (`CESAMLAB/cesam-tools`) tramite il crate condiviso
**`mock_lib_update`** (`ureq`/`rustls`, radici incorporate, thread
limitato da timeout). **Assente nelle compilazioni headless**
(`--no-default-features`).

---

## 5. Dipendenze e insidie di versione

| Crate | Ruolo | Attenzione |
|-------|------|-------------------|
| `tokio` | runtime asincrono | feature condivise + `io-util` |
| `ractor` | attori | feature predefinite |
| `tokio-serial` | collegamento PROFIBUS DP | **dipendenza diretta, non controllata da feature** (vedere §2); `default-features = false` (nessuna enumerazione `libudev`) |
| `eframe`/`egui` | GUI | versioni legate tra loro, feature `gui` |
| `egui_plot` | curva | ⚠️ **versionato una minor in anticipo su `egui`**: per `egui` 0.33 → `egui_plot` **0.34** |
| `serde`/`toml` | persistenza | `mock_lib_control` espone una feature `serde` attivata dal binario |
| `mock_lib_update` (`ureq`/`rustls`) | verifica aggiornamenti | solo feature `gui`; assente in headless |

Le versioni condivise sono centralizzate in `[workspace.dependencies]` del
`Cargo.toml` radice. Nell'aggiornare `egui`/`eframe`, **verificare la
versione corrispondente di `egui_plot`** (altrimenti errore «two versions
of crate egui»).

---

## 6. Estendere il progetto

### 6.1 Aggiungere un servizio PROFIBUS (SAP)

Tutto avviene in **[`profibus.rs`](../../src/profibus.rs)** (fonte di
verità del protocollo):

1. Aggiungere la costante `SAP_*` e la variante corrispondente in
   `enum Request`; collegare la decodifica in `decode_request` (e, per i
   test, in `encode_request`).
2. Trattare la nuova richiesta in `SlaveFsm::handle` (transizione di
   stato se pertinente, `Handled` restituito).
3. Aggiornare il commento di documentazione del modulo e
   **[reference_profibus.md](reference_profibus.md)**.
4. Aggiungere un test nel modulo `tests` di `profibus.rs` (e, se la
   sessione completa è interessata, in `profibus_server.rs`).

### 6.2 Modificare i blocchi I/O (`Output`/`Input`)

1. Regolare la disposizione in **[`map.rs`](../../src/map.rs)**
   (`decode_output`/`encode_input`), mantenendo `OUTPUT_LEN`/`INPUT_LEN`
   coerenti con `SlaveProfile` (`profibus_server.rs`).
2. Aggiornare la tabella di
   **[reference_profibus.md](reference_profibus.md)** §3 (fonte di
   verità documentale, copiata dal commento di documentazione di
   `map.rs`).
3. Aggiungere un test di andata e ritorno in `map.rs`.

### 6.3 Aggiungere un comando di dominio / un'impostazione GUI

1. Variante in `enum Command` (`regulator.rs`) + gestione in
   `Regulator::apply` (con sanificazione).
2. Campo in `RegulatorSnapshot` se il valore deve essere osservabile.
3. Collegamento GUI (`gui.rs`) tramite un `cast` non bloccante.
4. Se persistente: campo in `AppConfig` (`config.rs`) + sanificazione in
   `sanitized` + riporto in `to_regulator_config`.

### 6.4 Aggiungere una stringa di interfaccia (i18n)

Ogni stringa GUI **deve** passare per una chiave `Msg` (`i18n.rs`) con le
sue **8 traduzioni** (array a dimensione fissa verificato a tempo di
compilazione). Gli identificativi di servizio PROFIBUS e i suffissi di
unità restano codificati fissi.

### 6.5 Aggiungere un nuovo strumento

1. Creare `mock_bin_<nome>/` e aggiungerlo ai `members` del `Cargo.toml`
   radice.
2. Riutilizzare `mock_lib_control`; fattorizzare tutto ciò che è comune
   in una `mock_lib_*`.
3. Seguire la stessa suddivisione: modello sincrono, attore/i `ractor`,
   strato di protocollo, GUI. Convenzione di nome:
   `mock_bin_<tipo>_<protocollo>`.

---

## 7. Strategia di test

- **Codec di trama** (`profibus.rs`): andata e ritorno di
  `SD1`/`SD2`/`SD3`/`SD4`, rifiuto di checksum e lunghezza errati,
  codifica/decodifica delle richieste
  (`Slave_Diag`/`Set_Prm`/`Chk_Cfg`/`Data_Exchange`) e del byte di modo.
- **Macchina a stati** (`profibus.rs`): sequenza completa
  `Power_On → Wait_Prm → Wait_Cfg → Data_Exchange`, rifiuto di un
  `Set_Prm` con identificativo errato (resta in `Wait_Prm`).
- **Blocchi I/O** (`map.rs`): un blocco di uscita troppo corto → nessun
  comando; andata e ritorno di setpoint/modo; il blocco di ingresso
  riflette lo snapshot (bit di stato, misura).
- **Configurazione** (`config.rs`): andata e ritorno TOML, sanificazione
  (limiti invertiti, valori non finiti, indirizzo di stazione fuori
  intervallo) senza panico, errore pulito nell'apertura di una porta
  seriale assente.
- **Sessione di rete** (`profibus_server.rs`, `#[tokio::test]` su
  `tokio::io::duplex`): handshake completo fino a `Data_Exchange` con
  applicazione effettiva dei comandi, una trama indirizzata a un'altra
  stazione ignorata (nessuna attività segnalata), scadenza del watchdog
  che forza lo stato sicuro.

Eseguire: `cargo test -p mock_bin_ru_pbdp` (o `--workspace`) — **36
test**, tutti **deterministici e senza GUI**, nessun test
lento/`#[ignore]` (a differenza di ORUE, la cui generazione RSA
giustifica test ignorati).

---

## 8. Risoluzione dei problemi

| Sintomo | Pista |
|----------|-------|
| «two versions of crate `egui`» | Discrepanza `egui_plot` / `egui`: allineare le versioni (§5). |
| La GUI non si apre | Nessun display (headless) o librerie di sistema mancanti (§1). |
| Errore di apertura della porta seriale (intestazione GUI) | Porta assente, percorso errato, o permessi (gruppo `dialout` su Linux) — la simulazione continua a funzionare senza collegamento. |
| Il collegamento resta in `Wait_Prm` | Il master non invia `Set_Prm` con l'identificativo atteso (`0xEE01`) — vedere [reference_profibus.md](reference_profibus.md) §2. |
| Il collegamento resta in `Wait_Cfg` | Il `Chk_Cfg` ricevuto non annuncia `out_len=45`/`in_len=17`. |
| L'apparecchio si ferma da solo | Watchdog di protocollo attivato (silenzio prolungato del master) — stato sicuro atteso, non un bug. |
| Nessun watchdog anche se il master ne richiede uno | `watchdog_enabled = false` nella configurazione locale: la richiesta del master viene deliberatamente ignorata. |

Aumentare la verbosità: `RUST_LOG=debug` (o `trace`).

---

## 9. Build di distribuzione

```bash
cargo build --release -p mock_bin_ru_pbdp
# Binario autonomo:
target/release/ru_pbdp
```

Il profilo `release` attiva `lto = "thin"` e `opt-level = 3` (vedere il
`Cargo.toml` radice). Per distribuire: fornire il binario più un
`mock_ru_pbdp.toml` di esempio. Licenza **MIT** (file `LICENSE`).

### Feature `gui` (build con / senza interfaccia)

```bash
cargo build --release -p mock_bin_ru_pbdp                       # con GUI (postazione di lavoro)
cargo build --release -p mock_bin_ru_pbdp --no-default-features  # «headless»: collegamento seriale + simulazione, senza GUI
```

A differenza di OSNE, la modalità **headless** non rende opzionale il
collegamento seriale (§2): rimuove solo la GUI. Resta pertinente per un
deployment senza schermo collegato a una vera porta seriale/USB.

### Integrazione nel desktop Linux (icona nella barra delle applicazioni)

L'icona ORPD (`pic/ru_pbdp-icon.png`, generata da
[`pic/ru_pbdp-logo.gen.py`](../../../pic/ru_pbdp-logo.gen.py)) è
**incorporata** nel binario (`branding.rs` → `window_icon`). Questo basta
su **X11, Windows e macOS**. Su **Wayland**, il compositor **ignora**
l'icona incorporata: associa la finestra al proprio **`app_id`**
(«ru_pbdp», impostato in `main.rs` tramite `with_app_id`) a un file
`ru_pbdp.desktop` dello stesso nome, e mostra l'`Icon=ru_pbdp` risolta nel
tema di icone `hicolor`.

Per ottenere l'icona su Wayland, installare la voce di desktop per
l'utente corrente:

```bash
scripts/install-desktop.sh ru_pbdp
```

Lo script copia:

| Origine | Destinazione |
|--------|-------------|
| `pic/ru_pbdp-icon.png` | `~/.local/share/icons/hicolor/256x256/apps/ru_pbdp.png` |
| `packaging/ru_pbdp.desktop` | `~/.local/share/applications/ru_pbdp.desktop` |

poi aggiorna le cache. Tre nomi **devono restare allineati**: l'`app_id`
(`main.rs`), il file `ru_pbdp.desktop` (+ il suo `StartupWMClass`) e
l'icona `ru_pbdp.png` (= `Icon=ru_pbdp`).

---

## 10. Build «prod» — cross-compilazione da Linux

Tutto viene prodotto **da Linux** da
[`scripts/build-prod.sh`](../../../scripts/build-prod.sh), che compila
**ogni strumento del workspace** (tabella `INSTRUMENTS`, voce
`mock_bin_ru_pbdp:ru_pbdp:0` — porta `0`: collegamento seriale, nessuna
porta IP):

| Output | Target | GUI | Metodo |
|--------|-------|-----|---------|
| `dist/ru_pbdp-linux-x86_64` | `x86_64-unknown-linux-gnu` | ✅ | `cross` |
| `dist/ru_pbdp-windows-x86_64.exe` | `x86_64-pc-windows-gnu` | ✅ | `cross` (mingw) |
| `dist/ru_pbdp-rpi-arm64` | `aarch64-unknown-linux-gnu` (Pi 3/4/5, Pi OS 64 bit) | ✅ | `cross` |
| Immagine Docker headless `ru_pbdp:headless` | multi-arch `amd64` + `arm64` | ❌ | `docker buildx` |
| `dist/ru_pbdp_<ver>_amd64.deb` / `_arm64.deb` | pacchetto Debian/Ubuntu | ✅ | `dpkg-deb` |
| `dist/ru_pbdp-setup-x86_64.exe` | installer Windows | ✅ | NSIS (`makensis`) |

```bash
cargo install cross          # prerequisito (una tantum) — Docker deve essere in esecuzione
scripts/build-prod.sh        # ogni strumento, incluso ru_pbdp
ONLY=ru_pbdp scripts/build-prod.sh   # solo questo strumento
```

⚠️ **Non mescolare `cargo` nativo e `cross`** nella stessa `target/`
(proc-macro incompatibili → `can't find crate for …_derive`). Lo script
passa sempre per `cross`.

### Immagine Docker headless: utilità limitata senza passthrough seriale

L'immagine ([`docker/Dockerfile.headless`](../../../docker/Dockerfile.headless))
è costruita come per gli altri strumenti (`EXPOSE 0`, metadato inerte),
ma **è realmente utile solo con un dispositivo seriale montato** nel
container:

```bash
docker run --rm --device=/dev/ttyUSB0 -v "$PWD/conf:/data" ru_pbdp:headless
```

Senza `--device`, il container si avvia ma non può aprire alcuna porta
seriale (stesso comportamento dell'assenza di hardware in locale — vedere
§8).

---

## 11. Convenzioni

- Codice e commenti in **francese** (convenzione dell'intero progetto);
  log e messaggi di errore in **inglese**.
- `cargo clippy --workspace` **senza avvisi** prima di ogni commit.
- Ogni nuovo comportamento di dominio o di protocollo è accompagnato da
  un **test**.
- Il protocollo PROFIBUS DP-V0 viene modificato in **`profibus.rs`**
  (fonte di verità), insieme a un aggiornamento di
  **[reference_profibus.md](reference_profibus.md)**.
