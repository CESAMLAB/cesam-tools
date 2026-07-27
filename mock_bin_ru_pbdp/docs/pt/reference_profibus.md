# Referência PROFIBUS DP-V0 — Regulador simulado (ORPD)

*🌍 [FR](../fr/reference_profibus.md) · [EN](../en/reference_profibus.md) · [DE](../de/reference_profibus.md) · [ES](../es/reference_profibus.md) · [IT](../it/reference_profibus.md) · **PT** · [NL](../nl/reference_profibus.md) · [PL](../pl/reference_profibus.md)*

> Crate: `mock_bin_ru_pbdp` · Executável: **ru_pbdp** · Protocolo: **PROFIBUS DP-V0** (escravo série)

Este documento é a referência funcional do subconjunto PROFIBUS DP-V0
simulado. A **fonte de verdade técnica** continua a ser o cabeçalho de
[`src/profibus.rs`](../../src/profibus.rs) (codec + máquina de estados) e
de [`src/map.rs`](../../src/map.rs) (blocos de E/S): qualquer divergência
deve ser corrigida no código em primeiro lugar.

---

## ⚠️ 0. Âmbito e limites — ler antes de qualquer utilização

O `ru_pbdp` implementa um **subconjunto pedagógico** de DP-V0, **sem
qualquer pretensão de conformidade binária estrita** com as tabelas
normativas (IEC 61158 / EN 50170) para além dos elementos mais
universalmente documentados:

- **conformes**: delimitadores de trama (`SD1`/`SD2`/`SD3`/`SD4`/`SC`/`ED`),
  FCS (soma módulo 256), números SAP dos serviços de parametrização
  (`Slave_Diag` = 61, `Set_Prm` = 62, `Chk_Cfg` = 63).
- **convenções próprias deste simulador, não um perfil GSD real
  registado junto do PNO** (PROFIBUS & PROFINET International): codificação
  exata dos bits do campo `FC`, disposição precisa dos bytes de
  diagnóstico, disposição dos blocos de entrada/saída (§3), o
  identificador `Ident_Number` (§4).
- **nenhum temporizador de barramento real**: nem uma janela de resposta
  (*slot time*, `Tsdr` mín/máx), nem um testemunho entre mestres, nem uma
  arbitragem multi-mestre. Só um ASIC dedicado (SPC3/VPC3) ou uma placa
  mestra de hardware (Hilscher/Softing/Siemens CP) conseguem cumprir
  estas restrições ao nível do bit.

**Consequência direta: este simulador nunca será reconhecido por um
verdadeiro mestre PROFIBUS DP** (autómato + placa mestra). Serve para
compreender a estrutura do protocolo e testar um desenvolvimento de
software (codec, máquina de estados, ferramentas), não para pilotar
equipamento de campo — ver [`manuel_utilisateur.md`](manuel_utilisateur.md).

---

## 1. Tramas — delimitadores e FCS

| Delimitador | Valor | Utilização |
|---|:--:|---|
| `SD1` | `0x10` | Pedido fixo sem dados (6 bytes: `SD1 DA SA FC FCS ED`) |
| `SD2` | `0x68` | Trama de comprimento variável com dados (`SD2 LE LEr SD2 DA SA FC [dados…] FCS ED`) |
| `SD3` | `0xA2` | Trama de dados fixos, 8 bytes (14 bytes no total) — **não utilizada** por este simulador (ver §0), fornecida para completude do codec e dos seus testes |
| `SD4` | `0xDC` | Trama de testemunho, 3 bytes, sem FCS nem ED — fora de âmbito para um escravo mono-mestre simulado, fornecida para completude do codec |
| `SC` | `0xE5` | Confirmação curta, 1 byte |
| `ED` | `0x16` | Delimitador de fim |

- **`FCS`**: soma módulo 256 dos bytes úteis da trama (ver
  `profibus::checksum`). Uma trama recebida com um FCS incorreto é
  rejeitada (`FrameError::BadChecksum`) sem resposta — o mestre deve
  retransmitir.
- **`DA`/`SA`**: endereço destino / origem. Bit 7 de `DA` = **extensão de
  endereço (DAE)**: presença de um byte de SAP logo após `DA` na carga
  útil. Ausente = troca de dados por defeito (`Data_Exchange`). O
  endereço de estação ocupa os restantes 7 bits (`0`-`125`; `126`/`127`
  reservados pela norma, não utilizados aqui).
- **Este simulador privilegia sistematicamente `SD2`** para todas as
  trocas `Data_Exchange`, mesmo quando `SD3` (8 bytes fixos) bastaria
  num perfil real — escolha que simplifica o codec sem perder cobertura
  dos conceitos do protocolo (ver [`conception.md`](conception.md) §4).
- **Trama malformada / delimitador desconhecido (ruído de linha)**:
  rejeitada silenciosamente (`log::debug!`), a sessão continua — permite
  ressincronizar o fluxo de bytes sem derrubar a ligação.

---

## 2. Sequenciação — serviços e máquina de estados

O escravo simulado (`SlaveFsm`, [`profibus.rs`](../../src/profibus.rs))
percorre quatro estados:

```
PowerOn ──Slave_Diag──► WaitPrm ──Set_Prm (id OK)──► WaitCfg ──Chk_Cfg (tamanhos OK)──► DataExchange
```

| Estado | Significado | Resposta típica |
|---|---|---|
| `Power_On` | Logo após o arranque, antes da primeira consulta de diagnóstico | — |
| `Wait_Prm` | À espera de um `Set_Prm` válido | `Diag` com `Stat_1 = STAT1_PRM_REQ` |
| `Wait_Cfg` | Parametrizado, à espera de um `Chk_Cfg` válido | `Diag` com `Stat_1 = STAT1_CFG_FAULT` |
| `Data_Exchange` | Parametrizado e configurado: troca cíclica ativa | bloco de entrada (§3) |

### `Slave_Diag` (SAP 61)

Pedido sem dados (ou trama `SD1`, sempre interpretada como `Slave_Diag`
por convenção deste simulador — nenhuma extensão de endereço possível em
`SD1`, por falta de byte disponível para transportar um SAP). Resposta
`Diag` (6 bytes):

| Byte | Símbolo | Conteúdo |
|:--:|---|---|
| `0` | `Stat_1` | `0x01` (`STAT1_PRM_REQ`, enquanto não parametrizado) ou `0x02` (`STAT1_CFG_FAULT`, enquanto não configurado) ou `0x00` (`Data_Exchange`) |
| `1` | `Stat_2` | sempre `0x00` (não simulado) |
| `2` | `Stat_3` | sempre `0x00` (não simulado) |
| `3` | `Master_Add` | `0xFF` (nenhum mestre conhecido — não rastreado por este simulador) |
| `4-5` | `Ident_Number` | identificador fixo do escravo, big-endian (§4) |

O primeiro `Slave_Diag` recebido faz passar `Power_On` → `Wait_Prm`; os
seguintes não alteram o estado (apenas uma leitura de diagnóstico).

### `Set_Prm` (SAP 62)

Pedido (formato DP-V0 padrão, **corresponde** ao que um mestre real emite —
p. ex. `profirust` — não é uma convenção própria deste simulador, ao
contrário da disposição dos blocos de E/S do §3):

```
SAP(62) Station_Status(1) WD_Fact_1(1) WD_Fact_2(1) Min_Tsdr(1) Ident_Number(2, BE) Groups(1) [User_Prm_Data...]
```

`Station_Status` (bits Lock_Req/Sync_Req/Freeze_Req/WD_On), `Min_Tsdr`,
`Groups` e `User_Prm_Data` **não são utilizados** por este simulador (sem
bloqueio, sem modo Sync/Freeze nem grupos modelados); só são lidos
`WD_Fact_1`/`WD_Fact_2` e `Ident_Number`. O watchdog anunciado, se presente,
calcula-se como
`watchdog_ms = WD_Fact_1 × WD_Fact_2 × 10` (unidade 10 ms, convenção
padrão DP); `WD_Fact_1 = 0` **ou** `WD_Fact_2 = 0` significa «sem
watchdog». Resposta: `ShortAck` (`SC`) em todos os casos.

- Se `Ident_Number` **corresponder** ao perfil fixo do escravo (§4):
  estado → `Wait_Cfg`, e um eventual watchdog é transmitido à sessão
  (armado apenas se a definição local `watchdog_enabled` o permitir —
  ver [`manuel_utilisateur.md`](manuel_utilisateur.md) §4).
- Se o identificador **não corresponder**: a parametrização é rejeitada
  silenciosamente (`ShortAck` devolvido de qualquer forma, como prescrito
  pelo DP-V0 para este serviço, mas sem efeito sobre o estado interno) —
  o escravo permanece em `Wait_Prm`.

### `Chk_Cfg` (SAP 63)

Pedido: `SAP(63) Out_Len(1) In_Len(1)`. Resposta: `ShortAck`. O estado
passa a `Data_Exchange` **apenas se** `Out_Len == 45` e `In_Len == 17`
(tamanhos fixos do perfil simulado, §3) **e** o escravo estava em
`Wait_Cfg`; caso contrário o estado não muda (o mestre deve retransmitir
um `Chk_Cfg` correto).

### `Data_Exchange` (sem SAP — endereço por defeito, bit DAE ausente)

Pedido: o bloco de saída em bruto (45 bytes, §3). Resposta: o bloco de
entrada (17 bytes, §3), recalculado em tempo real a partir da imagem
partilhada no momento da resposta (sem tabela de memória persistente, ao
contrário do Modbus/ORME).

Se o mestre enviar um `Data_Exchange` **antes** de atingir o estado
`Data_Exchange` (sequenciação não respeitada), o escravo responde com o
diagnóstico atual (`Diag`) em vez de falhar ou ignorar a trama.

---

## 3. Blocos de E/S — disposição dos bytes

Copiado do cabeçalho de [`map.rs`](../../src/map.rs), única fonte de
verdade em caso de divergência. Todos os valores de vírgula flutuante
(`f32`) ocupam **4 bytes consecutivos, big-endian**.

### Bloco de saída — *Output* (mestre → escravo, `OUTPUT_LEN` = 45 bytes)

| Byte(s) | Símbolo | Tipo | Descrição |
|---|---|:--:|---|
| `0` | `OUT_MODE` | bits | bit0 = marcha, bit1 = auto, [3:2] = modo sentido 1, [5:4] = modo sentido 2 |
| `1-4` | `OUT_SP_AUTO` | f32 | Referência automática |
| `5-8` | `OUT_SP_MANUAL` | f32 | Referência manual (% saída, com sinal) |
| `9-12` | `OUT_KP1` | f32 | Ganho proporcional Kp sentido 1 |
| `13-16` | `OUT_KI1` | f32 | Ganho integral Ki sentido 1 |
| `17-20` | `OUT_KD1` | f32 | Ganho derivativo Kd sentido 1 |
| `21-24` | `OUT_KP2` | f32 | Ganho proporcional Kp sentido 2 |
| `25-28` | `OUT_KI2` | f32 | Ganho integral Ki sentido 2 |
| `29-32` | `OUT_KD2` | f32 | Ganho derivativo Kd sentido 2 |
| `33-36` | `OUT_HYSTERESIS` | f32 | Histerese dos reguladores tudo-ou-nada |
| `37-40` | `OUT_TOR_MIN_CYCLE` | f32 | Tempo de ciclo mínimo tudo-ou-nada (s) |
| `41-44` | `OUT_PWM_PERIOD` | f32 | Período do ciclo de modulação PWM (s) |

Os códigos de modo (`[3:2]`/`[5:4]`) seguem `ControllerKind`: `0` = Off,
`1` = PID, `2` = Tudo-ou-nada, `3` = PWM (ver `mock_lib_control`).

### Bloco de entrada — *Input* (escravo → mestre, `INPUT_LEN` = 17 bytes)

| Byte(s) | Símbolo | Tipo | Descrição |
|---|---|:--:|---|
| `0` | `IN_STATUS` | bits | bit0 = em marcha, bit1 = sentido 1 ativo (saída > 0), bit2 = sentido 2 ativo (saída < 0) |
| `1-4` | `IN_PV` | f32 | Medida / *process value* |
| `5-8` | `IN_OUTPUT` | f32 | Saída aplicada (% com sinal) |
| `9-12` | `IN_SP_AUTO` | f32 | Réplica (apenas leitura) da referência automática |
| `13-16` | `IN_SP_MANUAL` | f32 | Réplica (apenas leitura) da referência manual |

Um bloco de saída **demasiado curto** (< 45 bytes) é ignorado sem falhar:
nenhum `Command` é produzido, o regulador conserva o seu último estado
válido.

---

## 4. Perfil fixo do escravo

| Parâmetro | Valor | Observação |
|---|---|---|
| `Ident_Number` | `0xEE01` | **Fictício**, não registado junto do PNO — não representa nenhum dispositivo de catálogo real |
| `Out_Len` | `45` | Esperado em `Chk_Cfg.out_len` |
| `In_Len` | `17` | Esperado em `Chk_Cfg.in_len` |
| Endereço de estação | `0`-`125`, configurável | Definição local (modal *Definições*), ver [`manuel_utilisateur.md`](manuel_utilisateur.md) §4 |
| Formato de trama série | `8E1` (8 bits, paridade par, 1 bit de paragem) | **Fixado pela norma PROFIBUS DP**, não ajustável |
| Velocidades normalizadas | `9600` a `12 000 000` bit/s | Não verificado na abertura: um valor não padrão é transmitido tal e qual à porta série |

---

## 5. Watchdog de protocolo

Ao contrário do watchdog NAMUR do OSNE (acrescento caseiro), este é uma
**parte real do protocolo DP**: é **anunciado pelo mestre** no `Set_Prm`
(fatores `WD_Fact_1`/`WD_Fact_2`, §2) e só é **armado do lado do
escravo** se a definição local `watchdog_enabled` o permitir (caso
contrário o pedido do mestre é ignorado, nunca armado). No vencimento,
sem ter recebido uma nova trama para a estação, o escravo força o estado
seguro (`Command::SetOnOff(false)`) — simplificação documentada: um
verdadeiro perfil DP-V0 poderia exigir um retorno completo via
`Set_Prm`/`Chk_Cfg` antes de retomar a troca, o que este simulador não
exige explicitamente (basta retomar o envio de tramas `Data_Exchange`,
uma vez que o estado `Data_Exchange` não é abandonado com o vencimento do
watchdog).

---

## 6. Não-interoperabilidade — porquê

| Requisito do PROFIBUS DP real | Este simulador |
|---|---|
| Janela de resposta ao nível do bit (*slot time*, `Tsdr` mín/máx) | Ausente — responde assim que a trama é descodificada, sem restrição de tempo |
| Circuito dedicado (ASIC SPC3/VPC3) para o temporizador | Ausente — software Tokio comum |
| Testemunho entre mestres, arbitragem multi-mestre | Ausente — escravo mono-mestre, ligação ponto a ponto |
| Perfil GSD registado junto do PNO | Ausente — perfil de E/S próprio deste simulador (§3) |
| Codificação bit a bit exata dos campos FC/diagnóstico | Convenção de simulação, não garantida conforme |

**Um autómato real (um Siemens S7 com placa mestra, por exemplo) nunca
reconhecerá este simulador como escravo válido num verdadeiro barramento
PROFIBUS DP RS-485.** Duas instâncias deste simulador (ou um script que
reproduza a sequência abaixo), por outro lado, podem dialogar entre si
para ilustrar o protocolo — ver
[`manuel_utilisateur.md`](manuel_utilisateur.md) §5.

---

## 7. Exemplo de sequência (hexadecimal)

Sequência completa para estação `5`, mestre `3`, até à troca cíclica
(valores ilustrativos, `FCS` calculado sobre os bytes úteis):

```text
# 1. Slave_Diag (SD2, DAE=1, SAP=61)
→ TX  68 03 03 68 85 03 C0 3D FC 16
← RX  68 06 06 68 03 85 00 01 00 00 FF EE 01 F5 16   (Diag: Stat_1=0x01, Ident=0xEE01)

# 2. Set_Prm (SD2, DAE=1, SAP=62, formato DP-V0 padrão: Station_Status
#    Lock_Req+WD_On=0x88, WD_Fact_1=1, WD_Fact_2=30 (300ms), Min_Tsdr=0,
#    Ident=0xEE01, Groups=0)
→ TX  68 0B 0B 68 85 03 C0 3E 88 01 1E 00 EE 01 00 … 16
← RX  E5                                              (ShortAck)

# 3. Chk_Cfg (SD2, DAE=1, SAP=63, out_len=45, in_len=17)
→ TX  68 05 05 68 85 03 C0 3F 2D 11 … 16
← RX  E5                                              (ShortAck)

# 4. Data_Exchange (SD2, sem SAP, bloco de saída de 45 bytes)
→ TX  68 30 30 68 05 03 C0 [45 bytes] … 16
← RX  68 14 14 68 03 85 00 [17 bytes]  … 16          (bloco de entrada)
```

Os bytes exatos de FCS/comprimento dependem dos valores de carga útil;
este esquema ilustra a **ordem dos serviços**, não uma trama a reproduzir
literalmente. Ver os testes em [`profibus.rs`](../../src/profibus.rs) e
[`profibus_server.rs`](../../src/profibus_server.rs) para sequências
verificadas bit a bit.
