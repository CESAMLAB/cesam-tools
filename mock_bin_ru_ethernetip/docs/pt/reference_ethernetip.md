# Referência EtherNet/IP — tags e protocolo (RU/EtherNet/IP)

*🌍 [FR](../fr/reference_ethernetip.md) · [EN](../en/reference_ethernetip.md) · [DE](../de/reference_ethernetip.md) · [ES](../es/reference_ethernetip.md) · [IT](../it/reference_ethernetip.md) · **PT** · [NL](../nl/reference_ethernetip.md) · [PL](../pl/reference_ethernetip.md)*

> Fonte de verdade: [`eip_server.rs`](../../src/eip_server.rs) (encapsulamento,
> dispatch CIP, tabela de tags). Toda a evolução é feita **neste ficheiro** e
> repercute-se aqui.

---

## 1. Endpoint

Adaptador **EtherNet/IP** (mensagens explícitas **CIP** não conectadas) sobre TCP.
Escuta por defeito em `0.0.0.0:44818` (porta padrão EtherNet/IP, > 1024 → nenhum
privilégio necessário). Ajustes na secção `[network]` do TOML / no modal
*Parâmetros*:

| Chave | Defeito | Função |
|---|---|---|
| `bind_ip` | `0.0.0.0` | IP de escuta |
| `port` | `44818` | porta TCP (EtherNet/IP padrão) |
| `allowlist` | *(vazia)* | lista branca de IP (padrões `*` por octeto; vazia = tudo autorizado) |

> ⚠️ **Nenhuma autenticação nem cifragem** (EtherNet/IP «classic»). O único
> controlo de acesso é a **lista branca de IP** + a topologia de rede. `0.0.0.0` +
> lista vazia = **exposto**: a IHM exibe uma faixa de aviso.

⚠️ EtherNet/IP / CIP é **little-endian** (ao contrário de Modbus/S7). Os `REAL`
são `f32` IEEE-754 little-endian.

## 2. Sessões

Vários clientes **simultâneos** são aceites. Cada sessão: `RegisterSession`
(o servidor atribui um *session handle* não nulo) → `SendRRData` que transporta os pedidos
CIP → `UnRegisterSession` (ou desligamento TCP).

## 3. Subconjunto do protocolo implementado

- **Encapsulamento**: `RegisterSession` (0x0065), `UnRegisterSession` (0x0066),
  `SendRRData` (0x006F, mensagens explícitas não conectadas, CPF).
- **CIP**: `Read Tag` (serviço 0x4C) e `Write Tag` (serviço 0x4D) sobre **tags
  nomeados** (segmento simbólico ANSI `0x91`).

## 4. Tabela de tags

| Tag | Tipo CIP | Acesso | Grandeza | Escrita → comando |
|---|---|:--:|---|---|
| `Setpoint` | REAL (0x00CA) | R/W | consigna | `SetSetpoint` |
| `ProcessValue` | REAL | R | medida | — |
| `Output` | REAL | R | saída (%) | — |
| `ManualOutput` | REAL | R/W | saída manual (%) | `SetManualOutput` |
| `Run` | BOOL (0x00C1) | R/W | funcionamento | `SetRun` |
| `Auto` | BOOL | R/W | modo auto | `SetAuto` |
| `SetpointMin` | REAL | R | consigna mín. | — |
| `SetpointMax` | REAL | R | consigna máx. | — |
| `Kp` / `Ki` / `Kd` | REAL | R | ganhos PID | — |

Um tag conhecido em **só leitura** quando escrito é **aceite** (estado CIP sucesso) mas sem
efeito; um **tag desconhecido** devolve o estado CIP `0x05` (*path destination unknown*).
Toda a escrita pilotável é **limitada/saneada** pela simulação.

## 5. Exemplo de cliente

Com um cliente EtherNet/IP (p. ex. `pycomm3`, `rseip`, `rust-ethernet-ip`) apontando
para o IP/porta do servidor, os tags leem-se/escrevem-se pelo seu nome:

```python
from pycomm3 import CIPDriver  # ou LogixDriver consoante a ferramenta
# Ler a medida, escrever a consigna e iniciar a regulação:
#   read  Tag "ProcessValue" (REAL)
#   write Tag "Setpoint" = 80.0 (REAL)
#   write Tag "Run" = True (BOOL)
```

O servidor responde aos serviços genéricos Read/Write Tag endereçados por segmento
simbólico ANSI; não expõe nenhuma árvore de objetos CIP para além dos tags
acima.
