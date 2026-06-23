# Referência OPC UA — espaço de endereçamento (RU/OPC UA)

*🌍 [FR](../fr/reference_opcua.md) · [EN](../en/reference_opcua.md) · [DE](../de/reference_opcua.md) · [ES](../es/reference_opcua.md) · [IT](../it/reference_opcua.md) · **PT** · [NL](../nl/reference_opcua.md) · [PL](../pl/reference_opcua.md)*

> Fonte de verdade: [`opcua_server.rs`](../../src/opcua_server.rs) (declaração dos
> nós + callbacks). Toda a evolução da tabela faz-se **neste ficheiro** e
> repercute-se aqui.

---

## 1. Endpoint & segurança

O URL é `opc.tcp://<bind_ip>:<port>/` (predefinição `opc.tcp://0.0.0.0:4840/`), transporte
OPC UA TCP binário. A **segurança** é regulável (secção `[security]` do TOML / modal
*Parâmetros*) e determina o endpoint exposto:

| Modo | `encryption` | Política | Modo de segurança | Tokens |
|---|:--:|---|---|---|
| **Não cifrado** (predefinição) | `false` | `None` | `None` | `Anonymous` |
| **Cifrado** | `true` | `Basic256Sha256` | `SignAndEncrypt` | `Anonymous` (se `allow_anonymous`) e/ou utilizador/palavra-passe |

- **Não cifrado**: nem autenticação nem cifragem. A expor apenas numa **rede de
  confiança**. Arranque instantâneo (nenhum certificado).
- **Cifrado**: um **certificado de instância auto-assinado** é gerado no primeiro
  arranque (em `pki/`). O servidor confia nos certificados de cliente
  (`trust_client_certs`, prático para um simulador). Autenticação por
  **utilizador/palavra-passe** se `username` estiver preenchido; senão (ou além disso)
  token **anónimo** se `allow_anonymous`. ⚠️ A geração RSA pode levar alguns
  segundos no primeiro arranque (debug).

Definições (`[security]`): `encryption` (bool), `allow_anonymous` (bool), `username`
(vazio = sem auth por palavra-passe), `password` (em claro — **simulador apenas**).

---

## 2. Namespace

| Índice | URI |
|---|---|
| `0` | `http://opcfoundation.org/UA/` (namespace núcleo OPC UA) |
| `ns` | `urn:cesam-lab:ru-opcua` (namespace aplicacional) |

O índice `ns` do namespace aplicacional é atribuído dinamicamente no arranque;
um cliente resolve-o via `IN GetNamespaceArray` / o serviço *Browse*. Os nós
de negócio abaixo aí residem.

---

## 3. Nós (sob a pasta `Objects`)

Cada nó é uma `Variable`; o seu `NodeId` tem a forma `ns=<ns>;s=<nome>`.

| BrowseName | NodeId (`s=`) | Tipo | Acesso | Grandeza |
|---|---|---|:--:|---|
| `Setpoint` | `Setpoint` | `Double` | R/W | Referência (unidade de processo) |
| `ProcessValue` | `ProcessValue` | `Double` | R | Medição (PV) |
| `Output` | `Output` | `Double` | R | Saída de comando (%) |
| `ManualOutput` | `ManualOutput` | `Double` | R/W | Saída imposta em modo manual (%) |
| `Run` | `Run` | `Boolean` | R/W | Marcha / paragem da regulação |
| `Auto` | `Auto` | `Boolean` | R/W | Modo automático (PID) vs manual |

- **Leituras**: servidas por um callback que lê o **instantâneo partilhado**; são
  portanto «vivas» e **amostráveis** pelas subscrições (*Subscription*
  / *MonitoredItem*).
- **Escritas**: encaminhadas para o ator de simulação. Os valores são **higienizados**
  (não finitos rejeitados, referência limitada, saída manual limitada a `[0, 100]`).

---

## 4. Mapeamento para o estado de negócio

| Nó | Efeito de uma escrita | Fonte de uma leitura |
|---|---|---|
| `Setpoint` | `Command::SetSetpoint` (limitada `[sp_min, sp_max]`) | `snapshot.setpoint` |
| `ManualOutput` | `Command::SetManualOutput` (limitada `[0, 100]`) | `snapshot.manual_output` |
| `Run` | `Command::SetRun` | `snapshot.run` |
| `Auto` | `Command::SetAuto` | `snapshot.auto` |
| `ProcessValue` | — (só leitura) | `snapshot.pv` |
| `Output` | — (só leitura) | `snapshot.output` |

Uma escrita de um tipo inesperado devolve `Bad_TypeMismatch`; uma escrita sem
valor, `Bad_NothingToDo`. O `Float` é aceite além do `Double` para os
nós numéricos.

---

## 5. Exemplos (cliente OPC UA)

Com um cliente genérico (UaExpert, `opcua` CLI, etc.), ligar-se a
`opc.tcp://127.0.0.1:4840/`, segurança **None**, utilizador **Anonymous**, depois:

```text
# Leitura da medição e da referência
Read  ns=<ns>;s=ProcessValue   → 60.0
Read  ns=<ns>;s=Setpoint       → 60.0

# Arranque + nova referência
Write ns=<ns>;s=Run        = true
Write ns=<ns>;s=Setpoint   = 80.0

# Mudança para manual e saída imposta a 40 %
Write ns=<ns>;s=Auto         = false
Write ns=<ns>;s=ManualOutput = 40.0
```

Subscrever (*Subscribe* / *MonitoredItem*) a `ProcessValue` e `Output` permite
seguir a dinâmica do processo em tempo real.
