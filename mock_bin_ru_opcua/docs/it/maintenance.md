# Documentazione di manutenzione — RU/OPC UA (workspace `cesam-tools`)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · **IT** · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

> Crate: `mock_bin_ru_opcua` · Eseguibile: **ru_opcua**

---

## 1. Prerequisiti

- **Rust** recente. ⚠️ MSRV propria di questo crate: **1.91** (`async-opcua` non dichiara
  alcun `rust-version` e tira dipendenze recenti; il resto del workspace
  è a 1.85).
- Per l'IHM: le dipendenze di sistema di `eframe`/`egui` (le stesse di ORME/OSNE).
- Per il build *headless*: nessuna dipendenza grafica.

---

## 2. Comandi comuni

```bash
cargo run -p mock_bin_ru_opcua                       # IHM + server OPC UA
cargo run -p mock_bin_ru_opcua --no-default-features # headless (senza IHM)
cargo test -p mock_bin_ru_opcua                      # test unitari
cargo clippy -p mock_bin_ru_opcua --all-targets      # lint
MOCK_CONFIG=./ma_config.toml cargo run -p mock_bin_ru_opcua  # config alternativa
```

### Feature Cargo

- **`gui`** (default): interfaccia grafica `egui` + verifica degli aggiornamenti.
- `--no-default-features`: binario **headless** (server OPC UA + simulazione,
  senza IHM né rete di aggiornamento).

Il server `async-opcua` è **sempre** presente (la feature `server` di
`async-opcua`), perché è la ragion d'essere dello strumento.

---

## 3. Organizzazione del codice

```
mock_bin_ru_opcua/src/
├── main.rs            # Assembla runtime Tokio + attori + IHM/headless
├── regulator.rs       # Modello di business sincrono (PID + processo), comandi, passo
├── config.rs          # AppConfig (TOML), sanitized(), ServerStatus
├── i18n.rs            # Catalogo i18n (8 lingue), Lang + Msg + tr()
├── opcua_server.rs    # Server OPC UA: build + spazio di indirizzamento + callback
├── gui.rs             # IHM egui (feature gui)
├── branding.rs        # Logo incorporati (feature gui)
└── actors/
    ├── simulation.rs  #   anello di regolazione (tick 0,5 s)
    └── network.rs     #   server OPC UA (ri)configurabile a caldo
```

---

## 4. Configurazione

`AppConfig` (lingua / rete / processo / regolazione / `check_updates`) è
serializzata in **TOML** (`mock_ru_opcua.toml`, sovrascrivibile tramite `MOCK_CONFIG`),
caricata all'avvio (default se assente), salvata dall'IHM. Ogni valore
è **sanificato** al caricamento (`AppConfig::sanitized`: limiti ordinati,
`τ ≥ 1e-3`, `dead_time ≥ 0`, valori in virgola mobile finiti).

**Invariante**: mai chiamare `f32::clamp` con limiti non validati (panic
se `min > max` o `NaN`). Le scritture di rete passano anch'esse per
`Regulator::apply`, che sanifica.

### Verifica degli aggiornamenti

Solo feature `gui`: all'avvio, l'IHM interroga l'ultima release
GitHub tramite la lib condivisa `mock_lib_update` (thread limitato da timeout) e mostra
un banner se esiste una versione più recente. Regolabile tramite `check_updates`.

---

## 5. Dipendenze e trappole di versione

- **`async-opcua` 0.18** (server). Crittografia **100 % Rust** (RustCrypto): **nessuna
  dipendenza da OpenSSL** → cross-compilazione pulita. Licenza **MPL-2.0** (cfr. `NOTICE`).
- ⚠️ `async-opcua` non dichiara **alcun MSRV**: validare sulla toolchain target prima
  di alzare la versione.
- ⚠️ La generazione di certificato (`create_sample_keypair(true)`) è **volontariamente
  disabilitata**: la generazione RSA in Rust puro è molto lenta in *debug* e scriverebbe
  in `pki/`. In Fase 1b (endpoint None), nessun certificato è richiesto.
- `egui_plot` resta **in anticipo di una minore** su `egui` (cfr. ORME/OSNE).

---

## 6. Estendere il progetto

### 6.1 Aggiungere un nodo OPC UA

In [`opcua_server.rs`](../../src/opcua_server.rs): dichiarare il nodo
(`add_var`), collegare un callback di lettura (`on_read_*`) e, se scrivibile, un
callback di scrittura (`on_write_*`) che emette una `Command`. Riflettere la tabella in
[`reference_opcua.md`](reference_opcua.md).

### 6.2 Aggiungere un comando di business

Estendere l'enum `Command` ([`regulator.rs`](../../src/regulator.rs)), gestire il caso
in `Regulator::apply` (con sanificazione), aggiungere un test.

### 6.3 Aggiungere una stringa di interfaccia (i18n)

Aggiungere una variante a `Msg` ([`i18n.rs`](../../src/i18n.rs)) e **le 8
traduzioni** (tabella di dimensione fissa verificata alla compilazione).

### 6.4 Fase 2 — sicurezza

Attivare un endpoint cifrato (`Basic256Sha256`), provvedere un certificato
di istanza, aggiungere l'autenticazione utente. Rimuovere allora il filtro di log
`opcua_crypto::certificate_store=off` posto in [`main.rs`](../../src/main.rs).

---

## 7. Strategia di test

Il cuore di business (`regulator.rs`) e la configurazione (`config.rs`) sono **puri e
testati**: convergenza PID, clamp del setpoint, rilassamento all'arresto, cambio di
processo senza salto di PV, sanificazione TOML, andata-ritorno TOML. L'i18n verifica la
non-vacuità e l'andata-ritorno di lingua. La logica async (attori, server) resta
sottile e si appoggia su questi mattoni testati.

---

## 8. Risoluzione dei problemi

| Sintomo | Causa probabile | Rimedio |
|---|---|---|
| `failed to bind` all'avvio | porta già occupata / < 1024 senza diritti | cambiare la porta (*Parametri*) o avviare in root |
| Il client non vede i nodi | endpoint / sicurezza errati | `opc.tcp://…:4840/`, None, Anonymous; *Browse* sotto `Objects` |
| Scrittura `Bad_TypeMismatch` | tipo errato | `Double` per le grandezze, `Boolean` per `Run`/`Auto` |
| WARN «encrypted endpoints disabled» | nessun certificato (Fase 1b) | normale; l'endpoint None funziona |

---

## 9. Build «prod» — cross-compilazione da Linux

Lo strumento è integrato in [`scripts/build-prod.sh`](../../../scripts/build-prod.sh)
(tabella `INSTRUMENTS`): eseguibili **con IHM** per Linux x86_64, Windows x86_64 e
Raspberry Pi arm64 (tramite `cross`), più un'immagine Docker headless.

⚠️ **Cross Windows e `GetHostNameW`**: lo stack OPC UA tira `gethostname`, che fa
riferimento al simbolo winsock `GetHostNameW`. La libreria di import mingw-w64 dell'immagine
`cross` **di default** (`:0.2.5`) è troppo vecchia per fornirlo →
fallimento al link. Il repository fissa quindi, in [`Cross.toml`](../../../Cross.toml),
l'immagine Windows GNU su **`:main`** (mingw recente). Validato: i build headless **e**
IHM producono un `.exe` valido; ORME/OSNE compilano sempre (immagine sovra-insieme).

---

## 10. Convenzioni

- Codice e commenti in **francese**; log/errori in **inglese**.
- Stringhe IHM tramite `i18n` (8 lingue); mai codificate in modo fisso.
- Logica di business **sincrona e testabile**; l'asincrono è confinato agli attori
  e all'IO. `cargo clippy --workspace` senza avvisi.
- Invarianti `ractor`: nessuna guardia `Mutex` attraverso un `.await`; nessun
  timer/`spawn` scollegato senza `JoinHandle` abbandonato all'arresto.
