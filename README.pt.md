<p align="center">
  <img src="pic/Logo-CESAM-Couleur-vect-card.png" alt="CESAM-Lab" height="84">
</p>

# cesam-tools — Caixa de ferramentas CESAM-Lab

*🌍 [English](README.md) · [Français](README.fr.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Italiano](README.it.md) · **Português** · [Nederlands](README.nl.md) · [Polski](README.pl.md)*

<p align="center">
  <a href="https://github.com/CESAMLAB/cesam-tools/releases/latest"><img src="https://img.shields.io/github/v/release/CESAMLAB/cesam-tools?label=release" alt="Latest release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue" alt="License: MIT"></a>
</p>

Workspace Rust que reúne as **ferramentas da CESAM-Lab**, a começar por
**simuladores de instrumentos industriais**: aparelhos virtuais que
reproduzem um comportamento físico realista e comunicam via protocolos
de terreno. Útil para desenvolver, testar e demonstrar supervisores, autómatos
ou gateways **sem hardware real**.

> Distribuído gratuitamente sob licença [MIT](LICENSE).

## Instrumentos disponíveis

| Crate | Produto | Descrição | Protocolo | IHM |
|-------|---------|-------------|-----------|-----|
| [`mock_bin_ru_modbustcp`](mock_bin_ru_modbustcp) | **ORME** | Regulador (PID / TOR / PWM) sobre função de transferência | Modbus TCP & RTU (escravo) | egui |

Biblioteca partilhada:

| Crate | Descrição |
|-------|-------------|
| [`mock_lib_control`](mock_lib_control) | Blocos de regulação reutilizáveis: PID anti-saturação, tudo-ou-nada com histerese, processo de 1ª ordem + atraso puro (FOPDT). |

## ORME — o regulador simulado

<p align="center">
  <img src="pic/orme-logo.svg" alt="ORME — Open Regulator Modbus Emulator" height="120">
</p>

> **ORME** — *Open Regulator Modbus Emulator*. **«Abra o barramento.»**
> Um regulador de terreno que só existe no seu barramento Modbus.

Um regulador industrial virtual completo:

- **Processo** modelado por uma função de transferência de primeira ordem com
  atraso puro `K·e^(-Ls) / (1 + T·s)` (típica de um forno ou banho termostático).
- **Regulação** bidirecional: sentido 1 (aquecimento) e sentido 2 (arrefecimento),
  cada um configurável em **PID**, **tudo-ou-nada (TOR)** ou **relé de ciclo (PWM)**.
- **Modos** marcha/paragem e automático/manual.
- **Servidor Modbus** em **TCP** ou **RTU série / RS485** (feature `rtu`), à escolha.
  Tabela de endereços (consigna, medida, saída, modos…), **lista branca de IP**
  (curinga `*`) configurável a quente, e **política mono-mestre** (um só mestre
  remoto de cada vez; em TCP um recém-chegado desliga o anterior).
- **Interface gráfica** numa página: comando, **curva de tendência**
  em tempo real, **tabela de endereços Modbus ao vivo**, e um **modal Parâmetros**
  (transporte TCP/RTU, porta, IP autorizadas, parâmetros série, função de
  transferência, limites de consigna).
- **Configuração persistida** no formato TOML (`mock_ru_modbustcp.toml`),
  recarregada no arranque, com botão de reposição dos valores predefinidos.

### Arquitetura assíncrona

```
        Command (cast não bloqueante)          instantâneo partilhado
  IHM (egui) ──────────────────────►  SimulationActor  ──────────►  IHM (leitura)
  Modbus escrita ─────────────────►   (ractor)         ──────────►  imagem Modbus
  Modbus leitura  ◄──────────────────────────────────────  imagem Modbus
```

- **`ractor`**: um ator único possui o estado do regulador; todas as
  mutações passam por mensagens (sem bloqueio sobre a lógica de negócio).
- **`tokio-modbus`**: servidor Modbus TCP e RTU série (trait `Service`).
- **`eframe`/`egui`**: interface gráfica no thread principal.

## Transferência

Estão disponíveis binários pré-compilados na página [**Releases**](https://github.com/CESAMLAB/cesam-tools/releases/latest) — **sem necessidade de toolchain Rust**.

| Plataforma | IHM | Headless (apenas TCP, sem IHM) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`orme-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-linux-x86_64) | [`orme-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-linux-x86_64-headless) |
| Windows x86_64 | [`orme-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`orme-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-rpi-arm64) | [`orme-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-rpi-arm64-headless) |

```bash
chmod +x orme-linux-x86_64        # Linux / Raspberry Pi
./orme-linux-x86_64
```

Os binários Linux/RPi estão ligados dinamicamente à glibc e necessitam de um ambiente de trabalho (X11/Wayland) para a IHM. No **Wayland**, instale a entrada de ambiente de trabalho para o ícone da barra de tarefas: `scripts/install-desktop.sh`. Verifique a integridade com os checksums publicados:

```bash
sha256sum -c SHA256SUMS
```

## Arranque rápido

```bash
# Pré-requisitos: Rust stable (edição 2021, >= 1.85).
# Dependências de sistema Linux para a IHM: libxkbcommon, libwayland/xcb, openGL.

cargo run -p mock_bin_ru_modbustcp
```

A janela abre-se e o servidor Modbus TCP escuta em `0.0.0.0:5502`.
A **porta**, a **IP de escuta** e a **lista branca de IP** regulam-se no
modal **⚙ Parâmetros** (aplicado a quente) e são depois **persistidos** em
`mock_ru_modbustcp.toml`. A **língua da interface** (francês, inglês,
alemão, espanhol, italiano, português, neerlandês, polaco) escolhe-se neste
mesmo modal e é persistida. Para utilizar outro ficheiro de configuração:

```bash
MOCK_CONFIG=/caminho/para/ma_config.toml cargo run -p mock_bin_ru_modbustcp
```

### Testar a ligação Modbus

Com qualquer cliente Modbus (ex. `mbpoll`):

```bash
# Pôr em marcha (bobina 0) e depois ler a medida (input registers 0-1, f32)
mbpoll -m tcp -a 1 -t 0 -p 5502 127.0.0.1 1      # escrever a bobina On/Off
mbpoll -m tcp -a 1 -t 3:float -r 1 -p 5502 127.0.0.1   # ler PV (f32)
```

A tabela de endereços completa está documentada em
[`mock_bin_ru_modbustcp/src/map.rs`](mock_bin_ru_modbustcp/src/map.rs).

## Desenvolvimento

```bash
cargo test --workspace      # testes unitários + integração
cargo clippy --workspace    # lint
```

Ver [CLAUDE.md](CLAUDE.md) para as convenções e a arquitetura detalhada.

## Documentação

Cada instrumento possui a sua própria documentação na sua subpasta `docs/`,
disponível em oito línguas (`docs/<língua>/`). Para o regulador (versão
portuguesa):

- [**Manual do utilizador**](mock_bin_ru_modbustcp/docs/pt/manuel_utilisateur.md) — primeiros passos, IHM, parâmetros, FAQ.
- [Documento de conceção](mock_bin_ru_modbustcp/docs/pt/conception.md) — arquitetura e opções técnicas.
- [Tabela de endereços Modbus](mock_bin_ru_modbustcp/docs/pt/table_modbus.md) — plano de endereçamento completo.
- [Manutenção do software](mock_bin_ru_modbustcp/docs/pt/maintenance.md) — build, configuração, extensão, resolução de problemas.

## Marca & logótipos

Os logótipos estão em [`pic/`](pic/):

- [`orme-icon.svg`](pic/orme-icon.svg) / `orme-icon.png` — ícone ORME (mostrador),
  também embutido como ícone de janela da aplicação.
- [`orme-logo.svg`](pic/orme-logo.svg) — logótipo ORME completo (ícone + texto).
- [`Logo-CESAM-Couleur-vect.png`](pic/Logo-CESAM-Couleur-vect.png) — logótipo CESAM-Lab.

O ícone ORME é **gerado** a partir de [`pic/orme-logo.gen.py`](pic/orme-logo.gen.py)
(`python3 pic/orme-logo.gen.py` produz os `.svg`, a rasterizar em seguida).

## Licença

[MIT](LICENSE) © 2026 CESAM-Lab
