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
| [`mock_bin_ru_modbus`](mock_bin_ru_modbus) | **ORME** | Regulador (PID / TOR / PWM) sobre função de transferência | Modbus TCP & RTU (escravo) | egui |
| [`mock_bin_su_namur`](mock_bin_su_namur) | **OSNE** | Agitador de laboratório suspenso: função de transferência do motor, controlo rápido de velocidade, carga viscosa ajustável | NAMUR sobre TCP & série RS-232 (escravo) | egui |
| [`mock_bin_ru_opcua`](mock_bin_ru_opcua) | **ORUE** | Regulador de processo (PID anti-saturação) sobre um processo de primeira ordem, com segurança OPC UA configurável | OPC UA (servidor) | egui |
| [`mock_bin_ru_sparkplugb`](mock_bin_ru_sparkplugb) | **ORSE** | Regulador de processo exposto como nó periférico MQTT Sparkplug B (saída) | Sparkplug B / MQTT (cliente) | egui |
| [`mock_bin_ru_s7`](mock_bin_ru_s7) | **ORSS** | Regulador de processo exposto como servidor S7comm sobre ISO-on-TCP (RFC1006) | S7comm (servidor) | egui |
| [`mock_bin_ru_ethernetip`](mock_bin_ru_ethernetip) | **OREE** | Regulador de processo exposto como adaptador EtherNet/IP (mensagens explícitas CIP) | EtherNet/IP (adaptador) | egui |
| [`mock_bin_ru_pbdp`](mock_bin_ru_pbdp) | **ORPD** | Regulador de processo exposto como escravo PROFIBUS DP-V0 simulado sobre ligação série | PROFIBUS DP (escravo, série) | egui |

Bibliotecas partilhadas:

| Crate | Descrição |
|-------|-------------|
| [`mock_lib_control`](mock_lib_control) | Blocos de regulação reutilizáveis: PID anti-saturação, tudo-ou-nada com histerese, processo de 1ª ordem + atraso puro (FOPDT). |
| [`mock_lib_regulator`](mock_lib_regulator) | Regulador PID pronto a usar (estado, configuração TOML, ator `ractor`), partilhado tal e qual por ORUE, ORSE, ORSS e OREE. |

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
- **Configuração persistida** no formato TOML (`mock_ru_modbus.toml`),
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

## OSNE — o agitador de laboratório simulado

<p align="center">
  <img src="pic/osne-logo.svg" alt="OSNE — Open Stirrer NAMUR Emulator" height="120">
</p>

> **OSNE** — *Open Stirrer NAMUR Emulator*.
> Um agitador de laboratório suspenso (estilo IKA) que só existe na sua ligação NAMUR.

Um agitador de laboratório virtual completo:

- **Motor** modelado por uma função de transferência rotacional `J·dω/dt = T − k·η·ω −
  atrito` (Euler explícito), com um **PID rápido** a comandar o binário para seguir
  a consigna de velocidade.
- **Viscosidade ajustável** `η`: aumenta o binário de carga; com viscosidade elevada o
  motor satura e a consigna torna-se inalcançável (**sobrecarga**) — como um
  agitador real.
- **Servidor NAMUR** (protocolo de comandos ASCII) sobre **TCP** (teste sem hardware)
  ou **série RS-232** (feature `serial`), com um **watchdog** por sessão
  (`OUT_WD1@<m>`), política **mono-mestre** e uma **lista branca de IP** (TCP).
- **Interface gráfica** numa página: consigna de velocidade, viscosidade, **curva de
  tendência** ao vivo de velocidade/binário, um **mini-terminal NAMUR** embutido
  (enviar/inspecionar tramas com histórico de comandos), e um **modal Parâmetros**
  (transporte TCP/série, parâmetros do motor, limites, i18n em 8 línguas).
- **Configuração persistida** no formato TOML (`mock_su_namur.toml`), recarregada no
  arranque, com botão de reposição dos valores predefinidos.

Partilha a arquitetura do ORME (modelo de negócio síncrono, atores `ractor`, IHM
`egui`). Execute-o com `cargo run -p mock_bin_su_namur`; o servidor NAMUR escuta em
`0.0.0.0:4001` por predefinição.

## ORUE — o regulador OPC UA simulado

<p align="center">
  <img src="pic/ru_opcua-logo.svg" alt="ORUE — Open Regulator UA Emulator" height="120">
</p>

> **ORUE** — *Open Regulator UA Emulator*. **«Unifique o processo.»**
> Um regulador de processo que só existe no seu espaço de endereços OPC UA.

Um regulador de processo virtual completo:

- **Processo** modelado por uma função de transferência de primeira ordem comandada
  por um **PID anti-saturação**, com passo a cada 0,5 s.
- **Servidor OPC UA** (`async-opcua`, nativo Tokio, criptografia 100 % Rust — sem
  OpenSSL, pilha MPL-2.0). **Segurança configurável** (`SecurityConfig`):
  `None`/anónimo por predefinição (arranque instantâneo) **ou** `Basic256Sha256` /
  SignAndEncrypt com um certificado auto-assinado (`pki/`, gerado no primeiro
  arranque cifrado), além de fichas anónimas e/ou de **utilizador/palavra-passe**.
- **Uma postura que difere do ORME/OSNE**: a segurança OPC UA assenta em
  **certificado + autenticação**, não numa lista branca de IP (não há **nenhuma**);
  o servidor aceita **várias sessões de cliente em simultâneo** (sem mono-mestre,
  ganha o último a escrever). A predefinição `None`/anónimo em `0.0.0.0:4840` é a
  mais aberta do workspace — um aviso na IHM alerta sempre que a cifragem está
  desativada.
- **Interface gráfica** numa página: comando, **curva de tendência** em tempo real,
  e um **modal Parâmetros** (rede, função de transferência do processo, ganhos PID,
  limites de consigna, segurança, i18n em 8 línguas).
- **Configuração persistida** no formato TOML (`mock_ru_opcua.toml`), recarregada
  no arranque, com botão de reposição dos valores predefinidos.

Partilha a arquitetura do ORME (modelo de negócio síncrono, atores `ractor`, IHM
`egui`). Execute-o com `cargo run -p mock_bin_ru_opcua`; o servidor OPC UA escuta em
`0.0.0.0:4840` por predefinição. O espaço de endereços está documentado em
[`mock_bin_ru_opcua/docs/pt/reference_opcua.md`](mock_bin_ru_opcua/docs/pt/reference_opcua.md).

## ORSE — o nó periférico Sparkplug B simulado

<p align="center">
  <img src="pic/ru_spb-logo.svg" alt="ORSE — Open Regulator Sparkplug Emulator" height="120">
</p>

> **ORSE** — *Open Regulator Sparkplug Emulator*.
> Um regulador de processo que só existe como nó periférico MQTT Sparkplug B.

Um regulador de processo virtual completo, mesmo modelo PID + processo de primeira ordem que o ORME:

- **Nó periférico MQTT Sparkplug B** (cliente de saída, `rumqttc` +
  `sparkplug-rs`, protobuf Eclipse Tahu, 100% Rust — sem `protoc`). Publica
  `NBIRTH`/`NDATA` e um `NDEATH` transportado pelo **testamento MQTT**
  (*Last Will*, robusto a qualquer perda de ligação); reage às escritas
  `NCMD` do broker. Contadores `bdSeq`/`seq` possuídos e testados numa
  camada de protocolo pura, não delegados a uma framework.
- **Uma postura diferente do ORME/OSNE**: sendo um cliente e não um
  servidor, **sem lista branca de IP**. **MQTT em texto simples por
  predefinição** (porta 1883, não cifrado, sem autenticação) — um aviso na
  IHM alerta enquanto TLS + credenciais não forem ativados para sair de
  uma rede de confiança.
- **Interface gráfica** numa página: comando, **curva de tendência** em
  tempo real, e um **modal Parâmetros** (endereço/credenciais/TLS do
  broker, função de transferência do processo, ganhos PID, limites de
  consigna, i18n em 8 línguas).
- **Configuração persistida** no formato TOML (`mock_ru_sparkplugb.toml`),
  recarregada no arranque, com botão de reposição dos valores
  predefinidos.

Execute-o com `cargo run -p mock_bin_ru_sparkplugb`; liga-se de saída ao
broker configurado em *Parâmetros* (`localhost:1883` por predefinição) —
nenhuma porta em escuta.

## ORSS — o regulador S7 simulado

<p align="center">
  <img src="pic/ru_s7-logo.svg" alt="ORSS — Open Regulator S7 Server" height="120">
</p>

> **ORSS** — *Open Regulator S7 Server*.
> Um regulador de processo que só existe na sua ligação S7comm.

Um regulador de processo virtual completo, mesmo modelo PID + processo de primeira ordem que o ORME:

- **Servidor S7comm feito à mão** sobre ISO-on-TCP (RFC1006), porta 102:
  tramas TPKT, COTP (CR→CC, DT) e S7comm (Setup, Read/Write Var) sobre uma
  **imagem de bytes DB1**. Não existe nenhum crate de **servidor** S7 em
  Rust (apenas orientados a cliente): o subconjunto necessário é assim
  implementado diretamente — análise limitada, sem pânico perante uma
  trama malformada.
- **Vários clientes simultâneos aceites** (comportamento de um autómato
  real), ao contrário da política mono-mestre por expulsão do ORME — o
  último a escrever ganha.
- **Sem autenticação nem cifragem** (S7 «clássico»): apenas a **lista
  branca de IP** e a topologia de rede protegem o acesso; um aviso na IHM
  alerta em caso de exposição (`0.0.0.0` + lista branca vazia).
- **Interface gráfica** numa página: comando, **curva de tendência** em
  tempo real, e um **modal Parâmetros** (rede, lista branca, função de
  transferência do processo, ganhos PID, limites de consigna, i18n em 8
  línguas).
- **Configuração persistida** no formato TOML (`mock_ru_s7.toml`),
  recarregada no arranque, com botão de reposição dos valores
  predefinidos.

Execute-o com `cargo run -p mock_bin_ru_s7`; o servidor S7comm escuta por
predefinição em `0.0.0.0:102` (porta < 1024 requer privilégios root).

## OREE — o regulador EtherNet/IP simulado

<p align="center">
  <img src="pic/ru_eip-logo.svg" alt="OREE — Open Regulator EtherNet/IP Emulator" height="120">
</p>

> **OREE** — *Open Regulator EtherNet/IP Emulator*.
> Um regulador de processo que só existe na sua ligação EtherNet/IP.

Um regulador de processo virtual completo, mesmo modelo PID + processo de primeira ordem que o ORME:

- **Adaptador EtherNet/IP feito à mão** (encapsulamento `RegisterSession`,
  `SendRRData`/CPF, e CIP `Read Tag`/`Write Tag` por segmento simbólico,
  **little-endian**), porta 44818. Não existe nenhum crate de
  **adaptador** EtherNet/IP em Rust (apenas orientados a cliente/scanner):
  o subconjunto necessário é assim implementado diretamente — análise
  limitada, sem pânico perante um pacote malformado.
- **Vários clientes simultâneos aceites** (comportamento de um
  adaptador), ao contrário da política mono-mestre por expulsão do ORME —
  cada sessão recebe um *session handle*, o último a escrever ganha.
- **Sem autenticação nem cifragem** (EtherNet/IP «clássico»): apenas a
  **lista branca de IP** e a topologia de rede protegem o acesso; um
  aviso na IHM alerta em caso de exposição.
- **Interface gráfica** numa página: comando, **curva de tendência** em
  tempo real, e um **modal Parâmetros** (rede, lista branca, função de
  transferência do processo, ganhos PID, limites de consigna, i18n em 8
  línguas).
- **Configuração persistida** no formato TOML (`mock_ru_ethernetip.toml`),
  recarregada no arranque, com botão de reposição dos valores
  predefinidos.

Execute-o com `cargo run -p mock_bin_ru_ethernetip`; o adaptador
EtherNet/IP escuta por predefinição em `0.0.0.0:44818`.

## ORPD — o regulador PROFIBUS DP simulado

<p align="center">
  <img src="pic/ru_pbdp-logo.svg" alt="ORPD — Open Regulator Profibus DP" height="120">
</p>

> **ORPD** — *Open Regulator Profibus DP*.
> Um regulador de processo que só existe na sua ligação PROFIBUS DP.

Um regulador de processo virtual completo, mesmo modelo PID + processo de primeira ordem que o ORME:

- **Simulador de software de tramas PROFIBUS DP-V0** sobre ligação série
  (RS-485/RS-232): codec de tramas (`SD1`/`SD2`/`SD3`/`SD4`/`SC`, FCS) e
  máquina de estados do escravo
  (`Power_On → Wait_Prm → Wait_Cfg → Data_Exchange`). ⚠️ **Não
  interoperável com hardware PROFIBUS DP real**: a verdadeira
  temporização de barramento (*slot time*, `Tsdr`) exige um ASIC dedicado
  que este simulador puramente software não pretende emular — ver
  [`reference_profibus.md`](mock_bin_ru_pbdp/docs/pt/reference_profibus.md) §6.
- **A ligação série é o único transporte** (sem equivalente TCP para
  PROFIBUS DP, ao contrário do ORME/OSNE onde a série é uma funcionalidade
  opcional a par de um transporte TCP sempre presente): `tokio-serial` é
  uma dependência direta, não opcional. Sem lista branca de IP
  (intrinsecamente ponto a ponto).
- **Watchdog de protocolo** — uma parte real do DP-V0 (armado pelo mestre
  via `Set_Prm`), não um acrescento caseiro; força o estado seguro ao
  vencer.
- **Interface gráfica** numa página: comando, **curva de tendência** em
  tempo real, um **mini-terminal de tramas** (registo hexadecimal do
  tráfego RX/TX), e um **modal Parâmetros** (porta série, velocidade,
  endereço de estação, função de transferência do processo, ganhos PID,
  limites de consigna, i18n em 8 línguas).
- **Configuração persistida** no formato TOML (`mock_ru_pbdp.toml`),
  recarregada no arranque, com botão de reposição dos valores
  predefinidos.

Execute-o com `cargo run -p mock_bin_ru_pbdp`; tenta abrir a porta série
configurada (por predefinição `/dev/ttyUSB0` ou `COM3`, 500 kbit/s,
endereço de estação 3).

## Transferência

Estão disponíveis binários pré-compilados na página [**Releases**](https://github.com/CESAMLAB/cesam-tools/releases/latest) — **sem necessidade de toolchain Rust**. Cada instrumento fornece o seu próprio executável (`orme`, `osne`, `ru_opcua`, `ru_spb`, `ru_s7`, `ru_eip`, `ru_pbdp`).

**ORME** (regulador Modbus):

| Plataforma | IHM | Headless (apenas TCP, sem IHM) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`orme-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-linux-x86_64) | [`orme-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-linux-x86_64-headless) |
| Windows x86_64 | [`orme-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`orme-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-rpi-arm64) | [`orme-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/orme-rpi-arm64-headless) |

**OSNE** (agitador de laboratório NAMUR):

| Plataforma | IHM | Headless (apenas TCP, sem IHM) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`osne-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-linux-x86_64) | [`osne-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-linux-x86_64-headless) |
| Windows x86_64 | [`osne-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`osne-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-rpi-arm64) | [`osne-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/osne-rpi-arm64-headless) |

**ORUE** (regulador OPC UA):

| Plataforma | IHM | Headless (apenas TCP, sem IHM) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`ru_opcua-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-linux-x86_64) | [`ru_opcua-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-linux-x86_64-headless) |
| Windows x86_64 | [`ru_opcua-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`ru_opcua-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-rpi-arm64) | [`ru_opcua-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_opcua-rpi-arm64-headless) |

**ORSE** (nó periférico Sparkplug B):

| Plataforma | IHM | Headless (apenas cliente, sem IHM) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`ru_spb-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_spb-linux-x86_64) | [`ru_spb-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_spb-linux-x86_64-headless) |
| Windows x86_64 | [`ru_spb-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_spb-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`ru_spb-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_spb-rpi-arm64) | [`ru_spb-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_spb-rpi-arm64-headless) |

**ORSS** (regulador S7comm):

| Plataforma | IHM | Headless (apenas TCP, sem IHM) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`ru_s7-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_s7-linux-x86_64) | [`ru_s7-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_s7-linux-x86_64-headless) |
| Windows x86_64 | [`ru_s7-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_s7-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`ru_s7-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_s7-rpi-arm64) | [`ru_s7-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_s7-rpi-arm64-headless) |

**OREE** (adaptador EtherNet/IP):

| Plataforma | IHM | Headless (apenas TCP, sem IHM) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`ru_eip-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_eip-linux-x86_64) | [`ru_eip-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_eip-linux-x86_64-headless) |
| Windows x86_64 | [`ru_eip-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_eip-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`ru_eip-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_eip-rpi-arm64) | [`ru_eip-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_eip-rpi-arm64-headless) |

**ORPD** (regulador PROFIBUS DP):

| Plataforma | IHM | Headless (ligação série, sem IHM) |
|----------|-----|-----------------------------|
| Linux x86_64 | [`ru_pbdp-linux-x86_64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_pbdp-linux-x86_64) | [`ru_pbdp-linux-x86_64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_pbdp-linux-x86_64-headless) |
| Windows x86_64 | [`ru_pbdp-windows-x86_64.exe`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_pbdp-windows-x86_64.exe) | — |
| Raspberry Pi arm64 (Pi OS 64-bit) | [`ru_pbdp-rpi-arm64`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_pbdp-rpi-arm64) | [`ru_pbdp-rpi-arm64-headless`](https://github.com/CESAMLAB/cesam-tools/releases/latest/download/ru_pbdp-rpi-arm64-headless) |

```bash
chmod +x orme-linux-x86_64        # Linux / Raspberry Pi (o mesmo para os outros instrumentos)
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

cargo run -p mock_bin_ru_modbus
```

A janela abre-se e o servidor Modbus TCP escuta em `0.0.0.0:5502`.
A **porta**, a **IP de escuta** e a **lista branca de IP** regulam-se no
modal **⚙ Parâmetros** (aplicado a quente) e são depois **persistidos** em
`mock_ru_modbus.toml`. A **língua da interface** (francês, inglês,
alemão, espanhol, italiano, português, neerlandês, polaco) escolhe-se neste
mesmo modal e é persistida. Para utilizar outro ficheiro de configuração:

```bash
MOCK_CONFIG=/caminho/para/ma_config.toml cargo run -p mock_bin_ru_modbus
```

### Testar a ligação Modbus

Com qualquer cliente Modbus (ex. `mbpoll`):

```bash
# Pôr em marcha (bobina 0) e depois ler a medida (input registers 0-1, f32)
mbpoll -m tcp -a 1 -t 0 -p 5502 127.0.0.1 1      # escrever a bobina On/Off
mbpoll -m tcp -a 1 -t 3:float -r 1 -p 5502 127.0.0.1   # ler PV (f32)
```

A tabela de endereços completa está documentada em
[`mock_bin_ru_modbus/src/map.rs`](mock_bin_ru_modbus/src/map.rs).

## Desenvolvimento

```bash
cargo test --workspace      # testes unitários + integração
cargo clippy --workspace    # lint
```

## Documentação

Cada instrumento possui a sua própria documentação na sua subpasta `docs/`,
disponível em oito línguas (`docs/<língua>/`). Versões portuguesas:

**ORME** (regulador Modbus):

- [**Manual do utilizador**](mock_bin_ru_modbus/docs/pt/manuel_utilisateur.md) — primeiros passos, IHM, parâmetros, FAQ.
- [Documento de conceção](mock_bin_ru_modbus/docs/pt/conception.md) — arquitetura e opções técnicas.
- [Tabela de endereços Modbus](mock_bin_ru_modbus/docs/pt/table_modbus.md) — plano de endereçamento completo.
- [Manutenção do software](mock_bin_ru_modbus/docs/pt/maintenance.md) — build, configuração, extensão, resolução de problemas.

**OSNE** (agitador de laboratório NAMUR):

- [**Manual do utilizador**](mock_bin_su_namur/docs/pt/manuel_utilisateur.md) — primeiros passos, IHM, mini-terminal NAMUR, parâmetros, FAQ.
- [Documento de conceção](mock_bin_su_namur/docs/pt/conception.md) — modelo do motor, laço de regulação, arquitetura.
- [Conjunto de comandos NAMUR](mock_bin_su_namur/docs/pt/commandes_namur.md) — referência do protocolo (canais, comandos, exemplos).
- [Manutenção do software](mock_bin_su_namur/docs/pt/maintenance.md) — build, configuração, extensão, resolução de problemas.

**ORUE** (regulador OPC UA):

- [**Manual do utilizador**](mock_bin_ru_opcua/docs/pt/manuel_utilisateur.md) — primeiros passos, IHM, ligação de um cliente OPC UA, FAQ.
- [Documento de conceção](mock_bin_ru_opcua/docs/pt/conception.md) — modelo PID + processo, arquitetura de atores, pilha `async-opcua`, segurança.
- [Referência OPC UA](mock_bin_ru_opcua/docs/pt/reference_opcua.md) — endpoint, namespace, nós (leituras/escritas, exemplos).
- [Manutenção do software](mock_bin_ru_opcua/docs/pt/maintenance.md) — build, configuração, extensão, resolução de problemas.

**ORSE** (nó periférico Sparkplug B):

- [**Manual do utilizador**](mock_bin_ru_sparkplugb/docs/pt/manuel_utilisateur.md) — primeiros passos, IHM, ligação ao broker, FAQ.
- [Documento de conceção](mock_bin_ru_sparkplugb/docs/pt/conception.md) — arquitetura de atores, camada de protocolo, escolha de bibliotecas.
- [Referência Sparkplug B](mock_bin_ru_sparkplugb/docs/pt/reference_sparkplugb.md) — topics, métricas, NBIRTH/NDATA/NDEATH, mapeamento NCMD.
- [Manutenção do software](mock_bin_ru_sparkplugb/docs/pt/maintenance.md) — build, configuração, extensão, resolução de problemas.

**ORSS** (regulador S7comm):

- [**Manual do utilizador**](mock_bin_ru_s7/docs/pt/manuel_utilisateur.md) — primeiros passos, IHM, ligação de um cliente S7, FAQ.
- [Documento de conceção](mock_bin_ru_s7/docs/pt/conception.md) — arquitetura de atores, camada de protocolo, política de sessão.
- [Referência S7comm](mock_bin_ru_s7/docs/pt/reference_s7.md) — tramas TPKT/COTP/S7comm, imagem DB1, exemplos.
- [Manutenção do software](mock_bin_ru_s7/docs/pt/maintenance.md) — build, configuração, extensão, resolução de problemas.

**OREE** (adaptador EtherNet/IP):

- [**Manual do utilizador**](mock_bin_ru_ethernetip/docs/pt/manuel_utilisateur.md) — primeiros passos, IHM, ligação de um cliente CIP, FAQ.
- [Documento de conceção](mock_bin_ru_ethernetip/docs/pt/conception.md) — arquitetura de atores, camada de protocolo, política de sessão.
- [Referência EtherNet/IP](mock_bin_ru_ethernetip/docs/pt/reference_ethernetip.md) — encapsulamento, CIP Read/Write Tag, exemplos.
- [Manutenção do software](mock_bin_ru_ethernetip/docs/pt/maintenance.md) — build, configuração, extensão, resolução de problemas.

**ORPD** (regulador PROFIBUS DP):

- [**Manual do utilizador**](mock_bin_ru_pbdp/docs/pt/manuel_utilisateur.md) — primeiros passos, IHM, aviso de não-interoperabilidade, FAQ.
- [Documento de conceção](mock_bin_ru_pbdp/docs/pt/conception.md) — arquitetura de atores, camada de protocolo, decisões de codec.
- [Referência PROFIBUS DP-V0](mock_bin_ru_pbdp/docs/pt/reference_profibus.md) — tramas, sequenciação, blocos de E/S, watchdog, exemplo de sequência.
- [Manutenção do software](mock_bin_ru_pbdp/docs/pt/maintenance.md) — build, configuração, extensão, resolução de problemas.

## Marca & logótipos

Os logótipos estão em [`pic/`](pic/):

- [`orme-icon.svg`](pic/orme-icon.svg) / `orme-icon.png` — ícone ORME (mostrador),
  também embutido como ícone de janela da aplicação.
- [`orme-logo.svg`](pic/orme-logo.svg) — logótipo ORME completo (ícone + texto).
- [`osne-icon.svg`](pic/osne-icon.svg) / `osne-icon.png` — ícone OSNE (hélice de
  agitador), também embutido como ícone de janela do OSNE.
- [`osne-logo.svg`](pic/osne-logo.svg) — logótipo OSNE completo (ícone + texto).
- [`ru_opcua-icon.svg`](pic/ru_opcua-icon.svg) / `ru_opcua-icon.png` — ícone ORUE
  (mostrador de regulador envolto num anel de nó OPC UA), também embutido como
  ícone de janela do ORUE.
- [`ru_opcua-logo.svg`](pic/ru_opcua-logo.svg) — logótipo ORUE completo (ícone + texto).
- [`ru_spb-icon.svg`](pic/ru_spb-icon.svg) / `ru_spb-icon.png` — ícone ORSE
  (mostrador de regulador + relâmpago Sparkplug com nós pub/sub não ligados),
  também embutido como ícone de janela do ORSE.
- [`ru_spb-logo.svg`](pic/ru_spb-logo.svg) — logótipo ORSE completo (ícone + texto).
- [`ru_s7-icon.svg`](pic/ru_s7-icon.svg) / `ru_s7-icon.png` — ícone ORSS (mostrador
  de regulador + rack aberto de módulos quadrados, backplane S7), também embutido
  como ícone de janela do ORSS.
- [`ru_s7-logo.svg`](pic/ru_s7-logo.svg) — logótipo ORSS completo (ícone + texto).
- [`ru_eip-icon.svg`](pic/ru_eip-icon.svg) / `ru_eip-icon.png` — ícone OREE
  (mostrador de regulador + anel fechado de losangos, DLR EtherNet/IP), também
  embutido como ícone de janela do OREE.
- [`ru_eip-logo.svg`](pic/ru_eip-logo.svg) — logótipo OREE completo (ícone + texto).
- [`ru_pbdp-icon.svg`](pic/ru_pbdp-icon.svg) / `ru_pbdp-icon.png` — ícone ORPD
  (mostrador de regulador com motivo PROFIBUS DP), também embutido como ícone de
  janela do ORPD.
- [`ru_pbdp-logo.svg`](pic/ru_pbdp-logo.svg) — logótipo ORPD completo (ícone + texto).
- [`Logo-CESAM-Couleur-vect.png`](pic/Logo-CESAM-Couleur-vect.png) — logótipo CESAM-Lab.

Cada ícone é **gerado** a partir do seu script `*-logo.gen.py`
([`pic/orme-logo.gen.py`](pic/orme-logo.gen.py),
[`pic/osne-logo.gen.py`](pic/osne-logo.gen.py),
[`pic/ru_opcua-logo.gen.py`](pic/ru_opcua-logo.gen.py),
[`pic/ru_spb-logo.gen.py`](pic/ru_spb-logo.gen.py),
[`pic/ru_s7-logo.gen.py`](pic/ru_s7-logo.gen.py),
[`pic/ru_eip-logo.gen.py`](pic/ru_eip-logo.gen.py),
[`pic/ru_pbdp-logo.gen.py`](pic/ru_pbdp-logo.gen.py)). Todos os scripts exceto o
do ORME também rasterizam diretamente o respetivo `-icon.png` (via Pillow); o
`.svg` do ORME é rasterizado em seguida.

No **Wayland**, instale o ícone da barra de tarefas de um instrumento com
`scripts/install-desktop.sh [orme|osne|ru_opcua|ru_spb|ru_s7|ru_eip|ru_pbdp]`.

## Licença

[MIT](LICENSE) © 2026 CESAM-Lab

Os componentes de terceiros integrados em alguns instrumentos são distribuídos sob as suas próprias licenças (nomeadamente a pilha OPC UA sob MPL-2.0 utilizada por `mock_bin_ru_opcua`); consulte [NOTICE](NOTICE). Não alteram a licença MIT do código do cesam-tools.
