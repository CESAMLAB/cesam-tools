# Conceção — Regulador PROFIBUS DP simulado (ORPD)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · **PT** · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

> Crate: `mock_bin_ru_pbdp` · Executável: **ru_pbdp** (*Regulation Unit over PROFIBUS DP*)

Documento de arquitetura e modelação. Decalcado do regulador **ORME**
(`mock_bin_ru_modbus`) para o modelo de negócio e os atores, e do **OSNE**
(`mock_bin_su_namur`) para a ligação série. Só muda a **camada de
protocolo**: um **simulador de software de tramas PROFIBUS DP-V0**,
desenvolvido de raiz (não existe até à data nenhum crate
`profibus`/`profibus-dp` publicado no ecossistema Rust).

---

## 1. Objetivo

Simular um **regulador de processo** (ciclo PID sobre um processo térmico
de primeira ordem, modelo **idêntico** ao ORME) e expô-lo através de uma
**estrutura de tramas PROFIBUS DP-V0** sobre uma ligação série
(RS-485/RS-232).

**Este documento pressupõe que o leitor leu o aviso de
não-interoperabilidade** (ver [`manuel_utilisateur.md`](manuel_utilisateur.md)
e [`reference_profibus.md`](reference_profibus.md) §6): o PROFIBUS DP real
exige o cumprimento do temporizador do barramento ao nível do bit
(*slot time*, `Tsdr` mín/máx, um watchdog na ordem das dezenas de
milissegundos) que só um ASIC dedicado (SPC3/VPC3) pode garantir. Este
simulador não pretende tal — é uma ferramenta pedagógica e de teste de
software, não um controlador de barramento.

---

## 2. Modelo físico ([`regulator.rs`](../../src/regulator.rs))

Reutilizado tal e qual do regulador ORME:
[`mock_lib_control::FirstOrderProcess`] (função de transferência de
primeira ordem com atraso puro) e [`mock_lib_control::Pid`]
(PID anti-windup), com os mesmos modos (Off/PID/Tudo-ou-nada/PWM) em ambos
os sentidos (aquecer/arrefecer). Passo de simulação: **50 ms**. Todas as
escritas são **saneadas** em `Regulator::apply` (limites reordenados,
valores de vírgula flutuante não finitos ignorados, ganhos PID limitados)
— o mesmo invariante que em qualquer outro lugar do workspace: nunca
chamar `f32::clamp` com limites não validados.

---

## 3. Arquitetura (atores)

```
IU (egui) ──Command(cast)──►  SimulationActor  ──refresh──► SharedSnapshot ──► IU
Mestre PROFIBUS (simulado) ──►  (Regulator)      ──refresh──► SharedSnapshot ──► respostas Data_Exchange
```

- **`SimulationActor`** ([`actors/simulation.rs`](../../src/actors/simulation.rs)):
  idêntico na forma aos do ORME/OSNE — proprietário único do `Regulator`,
  temporizador de disparo único rearmado, publica o `SharedSnapshot` a
  cada passo.
- **`ProfibusServerActor`** ([`actors/network.rs`](../../src/actors/network.rs)):
  possui a ligação série; `Reconfigure` fecha/reabre o transporte se a
  porta/baud/endereço de estação mudar; conserva o `JoinHandle` da sessão
  (abortado ao parar); publica o estado da ligação (`ServerStatus`,
  incluindo o estado atual da máquina de estados DP-V0) para a IU.
- **[`profibus.rs`](../../src/profibus.rs)** — **fonte de verdade** do
  protocolo: codec de tramas (`SD1`/`SD2`/`SD3`/`SD4`/`SC`, FCS),
  descodificação dos serviços
  (`Slave_Diag`/`Set_Prm`/`Chk_Cfg`/`Data_Exchange`) e máquina de estados
  do escravo `SlaveFsm` (`PowerOn → WaitPrm → WaitCfg → DataExchange`).
- **[`map.rs`](../../src/map.rs)** — conversão dos blocos de bytes de E/S
  `Data_Exchange` de/para os `Command` do regulador (ver
  [`reference_profibus.md`](reference_profibus.md) §3).
- **[`profibus_server.rs`](../../src/profibus_server.rs)** — ciclo de
  sessão sobre qualquer fluxo `AsyncRead + AsyncWrite` (a porta série em
  produção, um `tokio::io::duplex` em testes): lê uma trama, descodifica-a,
  chama `SlaveFsm::handle`, aplica os `Command` resultantes, codifica a
  resposta e reenvia-a. Trata também o **watchdog de protocolo**
  (`tokio::select!` entre a leitura de trama e um atraso, como o watchdog
  NAMUR do OSNE — mas aqui é uma **parte real do protocolo DP**, armada
  pelo `Set_Prm`, não um acrescento caseiro).

Ao contrário do Modbus (ORME, tabela de memória separada regenerada a
cada tick) e como no OPC UA/NAMUR, **não há tabela de memória
persistente**: o bloco de entrada `Data_Exchange` é recalculado em tempo
real a partir do `SharedSnapshot` no momento da resposta.

**Sem política multi-mestre a gerir**: a ligação série *é* o único mestre
(como o Modbus RTU ou a porta série NAMUR), ao contrário do Modbus TCP do
ORME (expulsão) ou mesmo do NAMUR TCP do OSNE (ponto a ponto sem
expulsão).

---

## 4. Codec PROFIBUS DP-V0 — escolhas e limites aceites

- **Delimitadores de trama** (`SD1=0x10`, `SD2=0x68`, `SD3=0xA2`,
  `SD4=0xDC`, `SC=0xE5`, `ED=0x16`) e **FCS** (soma módulo 256): conformes
  à norma, bem documentados publicamente.
- **Números SAP dos serviços de parametrização** (`Slave_Diag=61`,
  `Set_Prm=62`, `Chk_Cfg=63`): conformes.
- **Codificação exata dos bits do campo FC**, **disposição precisa dos
  bytes de diagnóstico**, e **disposição dos blocos de entrada/saída**
  (`map.rs`): são **convenções próprias deste simulador**, não um perfil
  GSD real registado junto do PNO. O simulador utiliza sistematicamente
  tramas **SD2** (comprimento variável) para todas as trocas
  `Data_Exchange`, mesmo quando `SD3` (8 bytes fixos) bastaria num perfil
  real — escolha que simplifica o codec sem perder cobertura dos
  conceitos do protocolo.
- **Identificador PROFIBUS** (`Ident_Number = 0xEE01`): **fictício**, não
  registado junto do PNO (PROFIBUS & PROFINET International) — não
  representa nenhum dispositivo de catálogo real.
- **Nenhum temporizador de barramento**: nem uma janela de resposta
  (`Tsdr`), nem um testemunho, nem uma arbitragem multi-mestre estão
  implementados — ver §1.

Detalhe completo em [`reference_profibus.md`](reference_profibus.md).

---

## 5. Configuração e persistência

`AppConfig` (idioma / ligação série / processo / regulação / verificação
de atualizações) serializado em **TOML**
([`config.rs`](../../src/config.rs)), **saneado ao carregar**
(`AppConfig::sanitized`: limites ordenados, `τ ≥ 1e-3`, `dead_time ≥ 0`,
valores de vírgula flutuante finitos, endereço de estação limitado a
`[0, 125]`). Ficheiro: `mock_ru_pbdp.toml` (substituível via
`MOCK_CONFIG`). Ao contrário do ORME/OSNE, **sem lista branca de IP** (a
ligação série é intrinsecamente ponto a ponto, sem noção de endereço de
rede).

---

## 6. Vias de evolução

- Uma verdadeira ferramenta de **mestre PROFIBUS DP simulado** (binário
  separado), utilizando as mesmas funções de codificação/descodificação
  expostas para testes em `profibus.rs`, para pilotar este simulador ou
  qualquer outro escravo software sem depender de um script ad hoc.
- Geração de um ficheiro **GSD** ilustrativo (não funcional do lado do
  simulador) documentando o perfil de E/S simulado, com fins
  pedagógicos.
- Suporte de **DP-V1** (acesso acíclico, alarmes) caso surja a
  necessidade pedagógica — fora do âmbito inicialmente (apenas DP-V0).
- Promoção do modelo do regulador para uma `mock_lib_*` partilhada (hoje
  duplicado entre o ORME e este instrumento, como com o ORUE).
