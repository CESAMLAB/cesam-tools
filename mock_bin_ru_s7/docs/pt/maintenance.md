# Manutenção — Regulador S7 (ORSS)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · **PT** · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

---

## 1. Build e lançamento

```bash
cargo run -p mock_bin_ru_s7                        # IHM + servidor S7
cargo build -p mock_bin_ru_s7 --release            # executável IHM
cargo build -p mock_bin_ru_s7 --no-default-features # headless (sem IHM)
```

Features: `gui` (IHM `egui`, por defeito). `--no-default-features` produz um binário
**headless**: servidor S7 + simulação, sem IHM nem verificação de atualizações.

⚠️ A porta **102** (S7 padrão) é privilegiada (< 1024): execute com os direitos
adequados ou escolha uma porta alta na configuração.

## 2. Configuração

Ficheiro TOML `mock_ru_s7.toml` (diretório atual; caminho substituível por
`MOCK_CONFIG`). Secções: `language`, `[network]` (`bind_ip`, `port`, `allowlist`),
`[process]`, `[regulation]`, `check_updates`. Qualquer valor é **saneado** ao
carregar.

## 3. Testes

```bash
cargo test -p mock_bin_ru_s7      # unitários + round-trip TCP local
```

- **Camada de protocolo** (`s7_server`, sem rede): CR→CC, Setup, Read/Write Var,
  escrita de bit, código de retorno fora de zona, **não-pânico** em tramas malformadas,
  round-trip da imagem DB.
- **Ator de rede**: bind/escuta, e um **round-trip TCP real** (ligação COTP,
  escrita e depois releitura do setpoint via tramas S7 brutas) — sem dependência de um
  cliente externo.

## 4. Resolução de problemas

| Sintoma | Pista |
|---|---|
| Bind falha (`permission denied`) | porta 102 < 1024 → direitos root ou porta alta |
| Cliente recusado | lista branca de IP; firewall; IP/porta |
| Sem resposta | rack/slot (testar 0/1, 0/2); tramas fora do subconjunto ignoradas |
| Escrita sem efeito | offset apenas de leitura (cf. plano de endereçamento) |

## 5. Docker (headless)

Imagem headless via `scripts/build-prod.sh` (entrada `mock_bin_ru_s7:ru_s7:102`,
`EXPOSE 102`). Montar um volume no diretório de trabalho para fornecer o
`mock_ru_s7.toml`. O contentor publica a porta 102; mapear para uma porta alta do
lado do anfitrião se necessário.

## 6. Estender o plano de endereçamento

O plano DB1 e o mapeamento das escritas são a **fonte de verdade** em
[`s7_server.rs`](../../src/s7_server.rs) (`db_image` + `handle_write`). Para adicionar
uma grandeza: escrevê-la em `db_image` (leitura) e, se controlável, adicioná-la ao
`match` de `handle_write` (escrita → `Command`), depois refletir aqui e em
[`reference_s7.md`](reference_s7.md). Adicionar um teste no módulo.

## 7. Cross / Windows

Como os outros instrumentos (cf. `Cross.toml`). Nenhuma dependência nativa
particular: a camada S7 é 100 % Rust sobre TCP padrão.
