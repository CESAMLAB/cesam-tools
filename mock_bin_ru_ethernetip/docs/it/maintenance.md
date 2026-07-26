# Manutenzione — Regolatore EtherNet/IP (OREE)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · **IT** · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

---

## 1. Build & avvio

```bash
cargo run -p mock_bin_ru_ethernetip                        # IHM + adattatore EtherNet/IP
cargo build -p mock_bin_ru_ethernetip --release            # eseguibile IHM
cargo build -p mock_bin_ru_ethernetip --no-default-features # headless (senza IHM)
```

Feature: `gui` (IHM `egui`, di default). `--no-default-features` produce un binario
**headless**: adattatore EtherNet/IP + simulazione, senza IHM né verifica degli
aggiornamenti. La porta 44818 non richiede **alcun privilegio**.

## 2. Configurazione

File TOML `mock_ru_ethernetip.toml` (directory corrente; percorso sovrascrivibile con
`MOCK_CONFIG`). Sezioni: `language`, `[network]` (`bind_ip`, `port`, `allowlist`),
`[process]`, `[regulation]`, `check_updates`. Ogni valore è **sanificato** al
caricamento.

## 3. Test

```bash
cargo test -p mock_bin_ru_ethernetip      # unitari + round-trip TCP locale
```

- **Livello protocollo** (`eip_server`, senza rete): RegisterSession, Read/Write Tag,
  scrittura BOOL, tag sconosciuto (`0x05`), scrittura di un tag in sola lettura, **non-panic**
  su pacchetti malformati.
- **Attore di rete**: bind/ascolto e un **round-trip TCP reale** (RegisterSession,
  Write poi Read del setpoint) — senza dipendenza da un client esterno.

## 4. Risoluzione dei problemi

| Sintomo | Indizio |
|---|---|
| Client rifiutato | lista bianca di IP; firewall; IP/porta (44818) |
| Tag introvabile | nome inesatto (maiuscole/minuscole); vedere la tabella dei tag |
| Scrittura senza effetto | tag in sola lettura |
| Valori incoerenti | EtherNet/IP è **little-endian** (REAL = `f32` LE) |

## 5. Docker (headless)

Immagine headless tramite `scripts/build-prod.sh` (voce
`mock_bin_ru_ethernetip:ru_eip:44818`, `EXPOSE 44818`). Montare un volume sulla
directory di lavoro per fornire il `mock_ru_ethernetip.toml`.

## 6. Estendere la tabella dei tag

La tabella dei tag e il mapping delle scritture sono la **fonte di verità** in
[`eip_server.rs`](../../src/eip_server.rs) (`read_tag` + `write_tag`). Per aggiungere un
tag: aggiungerlo a `read_tag` (lettura) e, se pilotabile, a `write_tag` (scrittura →
`Command`), poi rifletterlo qui e in
[`reference_ethernetip.md`](reference_ethernetip.md). Aggiungere un test nel modulo.

## 7. Cross / Windows

Come gli altri strumenti (cfr. `Cross.toml`). Nessuna dipendenza nativa
particolare: il livello EtherNet/IP è 100 % Rust su TCP standard.
