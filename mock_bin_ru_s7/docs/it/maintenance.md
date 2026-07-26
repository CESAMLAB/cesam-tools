# Manutenzione — Regolatore S7 (ORSS)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · **IT** · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

---

## 1. Build e avvio

```bash
cargo run -p mock_bin_ru_s7                        # IHM + server S7
cargo build -p mock_bin_ru_s7 --release            # eseguibile IHM
cargo build -p mock_bin_ru_s7 --no-default-features # headless (senza IHM)
```

Feature: `gui` (IHM `egui`, predefinita). `--no-default-features` produce un binario
**headless**: server S7 + simulazione, senza IHM né verifica degli aggiornamenti.

⚠️ La porta **102** (S7 standard) è privilegiata (< 1024): eseguire con i permessi
adeguati o scegliere una porta alta nella configurazione.

## 2. Configurazione

File TOML `mock_ru_s7.toml` (directory corrente; percorso sovrascrivibile con
`MOCK_CONFIG`). Sezioni: `language`, `[network]` (`bind_ip`, `port`, `allowlist`),
`[process]`, `[regulation]`, `check_updates`. Ogni valore è **sanificato** al
caricamento.

## 3. Test

```bash
cargo test -p mock_bin_ru_s7      # unitari + round-trip TCP locale
```

- **Livello protocollo** (`s7_server`, senza rete): CR→CC, Setup, Read/Write Var,
  scrittura di bit, codice di ritorno fuori zona, **non-panic** su trame malformate,
  round-trip dell'immagine DB.
- **Attore di rete**: bind/ascolto, e un **round-trip TCP reale** (connessione COTP,
  scrittura e successiva rilettura del setpoint tramite trame S7 grezze) — senza dipendenza da un
  client esterno.

## 4. Risoluzione dei problemi

| Sintomo | Indizio |
|---|---|
| Il bind fallisce (`permission denied`) | porta 102 < 1024 → permessi root o porta alta |
| Client rifiutato | lista bianca di IP; firewall; IP/porta |
| Nessuna risposta | rack/slot (provare 0/1, 0/2); trame fuori dal sottoinsieme ignorate |
| Scrittura senza effetto | offset in sola lettura (cfr. piano di indirizzamento) |

## 5. Docker (headless)

Immagine headless tramite `scripts/build-prod.sh` (voce `mock_bin_ru_s7:ru_s7:102`,
`EXPOSE 102`). Montare un volume sulla directory di lavoro per fornire il
`mock_ru_s7.toml`. Il container pubblica la porta 102; mappare verso una porta alta lato
host se necessario.

## 6. Estendere il piano di indirizzamento

Il piano DB1 e il mapping delle scritture sono la **fonte di verità** in
[`s7_server.rs`](../../src/s7_server.rs) (`db_image` + `handle_write`). Per aggiungere
una grandezza: scriverla in `db_image` (lettura) e, se pilotabile, aggiungerla al
`match` di `handle_write` (scrittura → `Command`), poi riflettere qui e in
[`reference_s7.md`](reference_s7.md). Aggiungere un test nel modulo.

## 7. Cross / Windows

Come gli altri strumenti (cfr. `Cross.toml`). Nessuna dipendenza nativa
particolare: il livello S7 è 100 % Rust su TCP standard.
