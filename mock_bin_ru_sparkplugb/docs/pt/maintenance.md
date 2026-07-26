# Manutenção — Regulador Sparkplug B (ORSE)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · **PT** · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

---

## 1. Build e arranque

```bash
cargo run -p mock_bin_ru_sparkplugb                       # IHM + edge node
cargo build -p mock_bin_ru_sparkplugb --release           # executável IHM
cargo build -p mock_bin_ru_sparkplugb --no-default-features # headless (sem IHM)
```

Features: `gui` (IHM `egui`, por defeito). `--no-default-features` produz um binário
**headless**: edge node Sparkplug B + simulação, sem IHM nem verificação de atualizações.

## 2. Configuração

Ficheiro TOML `mock_ru_sparkplugb.toml` (diretório atual; caminho substituível por
`MOCK_CONFIG`). Secções: `language`, `[network]` (broker/Sparkplug), `[process]`,
`[regulation]`, `check_updates`. Ver [`reference_sparkplugb.md`](reference_sparkplugb.md)
para as chaves `[network]`. Qualquer valor é **saneado** no carregamento.

## 3. Testes

```bash
cargo test -p mock_bin_ru_sparkplugb              # unitários (sem broker)
cargo test -p mock_bin_ru_sparkplugb -- --ignored # round-trip com broker local
```

- **Unitários** (sem rede): regulação, saneamento de config e, sobretudo, a
  camada Sparkplug (topics, payloads `NBIRTH`/`NDEATH`, round-trip encode/decode,
  mapeamento `NCMD`, rejeição de tipo errado, retorno do `seq` 255→0).
- **Integração `#[ignore]`**: requer um broker MQTT local —
  `docker run -it --rm -p 1883:1883 eclipse-mosquitto` — depois lança o round-trip
  completo (NBIRTH recebido, NCMD aplicado, NDATA refletido).

## 4. Resolução de problemas

| Sintoma | Pista |
|---|---|
| "Desligado" permanente | broker inacessível (`broker_host`/`broker_port`, firewall, broker parado) |
| O SCADA não recebe nada | `group_id`/`edge_node_id`; subscrição `spBv1.0/<group>/#`; payloads protobuf |
| Falha de TLS | broker em TLS na 8883; certificado raiz reconhecido pelo sistema |
| NCMD ignorado | métrica não comandável ou tipo errado (cf. tabela das métricas) |

## 5. Docker (headless)

A imagem headless constrói-se via `scripts/build-prod.sh` (entrada
`mock_bin_ru_sparkplugb:ru_spb:0`). Sendo o ORSE um **cliente**, **não expõe nenhuma
porta** (`PORT=0`, `EXPOSE 0` = metadado inerte) e **nenhum `HEALTHCHECK`** TCP
é pertinente: a liveness constata-se do lado do broker via o **Last Will/NDEATH**.
Montar um volume no diretório de trabalho para fornecer o `mock_ru_sparkplugb.toml`.

## 6. Estender

A tabela de métricas e o mapeamento `NCMD` são a **fonte de verdade** em
[`sparkplug_node.rs`](../../src/sparkplug_node.rs). Para adicionar uma métrica:
adicioná-la a `data_metrics`/`changed_metrics` (leitura) e, se comandável, a
`ncmd_to_actions` (escrita → `Command`), depois refletir aqui e em
[`reference_sparkplugb.md`](reference_sparkplugb.md). Adicionar um teste no módulo.

## 7. Dependências notáveis

- `rumqttc` (cliente MQTT, rustls), `sparkplug-rs` (protobuf Tahu, codegen Rust puro).
- MSRV: a verificar após um build `cross` completo (pode ultrapassar o piso 1.85 do
  workspace conforme as dependências rustls).
