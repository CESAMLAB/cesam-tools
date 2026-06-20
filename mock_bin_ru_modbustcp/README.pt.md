# ORME — regulador simulado Modbus

*🌍 [English](README.md) · [Français](README.fr.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Italiano](README.it.md) · **Português** · [Nederlands](README.nl.md) · [Polski](README.pl.md)*

> *Open Regulator Modbus Emulator* · pacote `mock_bin_ru_modbustcp` · binário `orme`

Regulador industrial **simulado**, escravo **Modbus TCP/RTU**, com interface
gráfica. Faz parte do workspace [`cesam-tools`](../README.pt.md).

## Funcionalidades

- Processo de primeira ordem + atraso puro (função de transferência FOPDT).
- Regulação bidirecional (aquecimento / arrefecimento), cada sentido em **PID** ou
  **tudo-ou-nada**.
- Modos marcha/paragem e auto/manual; consignas auto (física) e manual (%).
- Servidor Modbus TCP que expõe a totalidade do estado.
- IHM `egui` com curva de tendência em tempo real e regulação dos ganhos PID.
- **Interface multilingue**: francês, inglês, alemão, espanhol, italiano,
  português, neerlandês, polaco (escolha no modal *Parâmetros*, persistida).

## Lançar

```bash
cargo run -p mock_bin_ru_modbustcp
# Ficheiro de configuração alternativo:
MOCK_CONFIG=./ma_config.toml cargo run -p mock_bin_ru_modbustcp
```

Escuta por omissão em `0.0.0.0:5502`. A porta, a IP de escuta e a lista branca
de IP regulam-se no modal **⚙ Parâmetros** e são persistidas em TOML.

## Tabela de endereços Modbus

Codificação dos flutuantes: 2 registos, big-endian, palavra de maior peso primeiro.

### Bobinas (FC 1/5/15)

| End | Papel |
|----|------|
| 0 | Marcha (1) / Paragem (0) |
| 1 | Auto (1) / Manual (0) |

### Entradas discretas (FC 2, leitura apenas)

| End | Papel |
|----|------|
| 0 | Em marcha |
| 1 | Sentido 1 (aquecimento) ativo |
| 2 | Sentido 2 (arrefecimento) ativo |

### Registos de retenção (FC 3/6/16)

| End | Tipo | Papel |
|-----|------|------|
| 0 | u16 | Modo sentido 1 (0=Off, 1=PID, 2=TOR) |
| 1 | u16 | Modo sentido 2 (0=Off, 1=PID, 2=TOR) |
| 2–3 | f32 | Consigna automática (SP) |
| 4–5 | f32 | Consigna manual (% saída, com sinal) |
| 6–7 | f32 | Kp sentido 1 |
| 8–9 | f32 | Ki sentido 1 |
| 10–11 | f32 | Kd sentido 1 |
| 12–13 | f32 | Kp sentido 2 |
| 14–15 | f32 | Ki sentido 2 |
| 16–17 | f32 | Kd sentido 2 |
| 18–19 | f32 | Histerese TOR |

### Registos de entrada (FC 4, leitura apenas)

| End | Tipo | Papel |
|-----|------|------|
| 0–1 | f32 | Medida (PV) |
| 2–3 | f32 | Saída aplicada (% com sinal: + aquecimento / − arrefecimento) |

A fonte de verdade é o cabeçalho de [`src/map.rs`](src/map.rs).

## Documentação

Documentação própria desta aplicação (pasta [`docs/pt/`](docs/pt/)):

- [**Manual do utilizador**](docs/pt/manuel_utilisateur.md) — primeiros passos, comando, parâmetros, FAQ.
- [Documento de conceção](docs/pt/conception.md) — arquitetura, opções técnicas, teoria da regulação.
- [Tabela de endereços Modbus](docs/pt/table_modbus.md) — plano de endereçamento completo, codificação, exemplos.
- [Manutenção do software](docs/pt/maintenance.md) — build, configuração, extensão, resolução de problemas.
