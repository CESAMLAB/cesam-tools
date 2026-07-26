# Manutenção — Regulador EtherNet/IP (OREE)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · **PT** · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

---

## 1. Build e arranque

```bash
cargo run -p mock_bin_ru_ethernetip                        # IHM + adaptateur EtherNet/IP
cargo build -p mock_bin_ru_ethernetip --release            # exécutable IHM
cargo build -p mock_bin_ru_ethernetip --no-default-features # headless (sans IHM)
```

Features: `gui` (IHM `egui`, por defeito). `--no-default-features` produz um binário
**headless**: adaptador EtherNet/IP + simulação, sem IHM nem verificação de atualizações.
A porta 44818 não requer **nenhum privilégio**.

## 2. Configuração

Ficheiro TOML `mock_ru_ethernetip.toml` (diretório atual; caminho substituível por
`MOCK_CONFIG`). Secções: `language`, `[network]` (`bind_ip`, `port`, `allowlist`),
`[process]`, `[regulation]`, `check_updates`. Qualquer valor é **saneado** no
carregamento.

## 3. Testes

```bash
cargo test -p mock_bin_ru_ethernetip      # unitaires + round-trip TCP local
```

- **Camada de protocolo** (`eip_server`, sem rede): RegisterSession, Read/Write Tag,
  escrita BOOL, tag desconhecido (`0x05`), escrita de um tag só de leitura, **não-pânico**
  em pacotes malformados.
- **Ator de rede**: bind/escuta e um **round-trip TCP real** (RegisterSession,
  Write depois Read da consigna) — sem dependência de um cliente externo.

## 4. Resolução de problemas

| Sintoma | Pista |
|---|---|
| Cliente recusado | lista branca de IP; firewall; IP/porta (44818) |
| Tag não encontrado | nome inexato (maiúsculas/minúsculas); ver a tabela de tags |
| Escrita sem efeito | tag só de leitura |
| Valores incoerentes | EtherNet/IP é **little-endian** (REAL = `f32` LE) |

## 5. Docker (headless)

Imagem headless via `scripts/build-prod.sh` (entrada
`mock_bin_ru_ethernetip:ru_eip:44818`, `EXPOSE 44818`). Montar um volume no
diretório de trabalho para fornecer o `mock_ru_ethernetip.toml`.

## 6. Estender a tabela de tags

A tabela de tags e o mapeamento das escritas são a **fonte de verdade** em
[`eip_server.rs`](../../src/eip_server.rs) (`read_tag` + `write_tag`). Para adicionar um
tag: adicioná-lo a `read_tag` (leitura) e, se pilotável, a `write_tag` (escrita →
`Command`), depois refletir aqui e em
[`reference_ethernetip.md`](reference_ethernetip.md). Adicionar um teste no módulo.

## 7. Cross / Windows

Como os outros instrumentos (cf. `Cross.toml`). Nenhuma dependência nativa
particular: a camada EtherNet/IP é 100 % Rust sobre TCP padrão.
