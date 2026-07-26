# Referência S7 — plano de endereçamento e protocolo (RU/S7)

*🌍 [FR](../fr/reference_s7.md) · [EN](../en/reference_s7.md) · [DE](../de/reference_s7.md) · [ES](../es/reference_s7.md) · [IT](../it/reference_s7.md) · **PT** · [NL](../nl/reference_s7.md) · [PL](../pl/reference_s7.md)*

> Fonte de verdade: [`s7_server.rs`](../../src/s7_server.rs) (análise das tramas,
> plano de endereçamento DB1, mapeamento das escritas). Qualquer evolução faz-se **neste
> ficheiro** e repercute-se aqui.

---

## 1. Endpoint

Servidor **S7comm** sobre **ISO-on-TCP / RFC1006**. Escuta por defeito em
`0.0.0.0:102` (porta padrão S7; **< 1024 → direitos root** necessários, caso contrário escolher uma
porta alta). Ajustes na secção `[network]` do TOML / no modal *Parâmetros*:

| Chave | Defeito | Função |
|---|---|---|
| `bind_ip` | `0.0.0.0` | IP de escuta |
| `port` | `102` | porta TCP (S7 padrão) |
| `allowlist` | *(vazio)* | lista branca de IP (padrões `*` por byte; vazio = tudo autorizado) |

> ⚠️ **Nenhuma autenticação nem cifragem** (S7 "classic"). O único controlo
> de acesso é a **lista branca de IP** + a topologia de rede. `0.0.0.0` + lista vazia
> = **exposto a toda a rede**: a IHM exibe uma faixa de aviso.

## 2. Sessões

Ao contrário do ORME (mono-mestre), o servidor S7 aceita **várias sessões
clientes simultâneas** (comportamento habitual de um autómato). Cada sessão negocia
COTP (Connection Request → Confirm) e depois S7 *Setup Communication*, antes das
trocas *Read Var* / *Write Var*.

## 3. Subconjunto de protocolo implementado

- **COTP**: Connection Request (CR) → Connection Confirm (CC); Data (DT).
- **S7comm**: *Setup Communication*, *Read Var* (função `0x04`), *Write Var*
  (função `0x05`) sobre o bloco de dados **DB1**.

O servidor expõe uma **imagem de bytes do DB1** (40 bytes). As leituras servem
uma fatia desta imagem; as escritas nos offsets controláveis produzem comandos
saneados para a simulação.

## 4. Plano de endereçamento DB1

REAL = `f32` big-endian (IEEE-754). Endereçamento por byte (`DBDx`) ou por bit
(`DBXx.y`).

| Endereço | Tipo | Acesso | Grandeza | Escrita → comando |
|---|---|:--:|---|---|
| `DB1.DBD0`  | REAL | R/W | Setpoint (Setpoint) | `SetSetpoint` |
| `DB1.DBD4`  | REAL | R   | Medida (ProcessValue) | — |
| `DB1.DBD8`  | REAL | R   | Saída (Output, %) | — |
| `DB1.DBD12` | REAL | R/W | Saída manual (ManualOutput, %) | `SetManualOutput` |
| `DB1.DBX16.0` | BOOL | R/W | Funcionamento (Run) | `SetRun` |
| `DB1.DBX16.1` | BOOL | R/W | Modo auto (Auto) | `SetAuto` |
| `DB1.DBD20` | REAL | R | Setpoint mín | — |
| `DB1.DBD24` | REAL | R | Setpoint máx | — |
| `DB1.DBD28` | REAL | R | PID Kp | — |
| `DB1.DBD32` | REAL | R | PID Ki | — |
| `DB1.DBD36` | REAL | R | PID Kd | — |

Escrita de `DB1.DBB16` (byte) aceite: bit 0 = Run, bit 1 = Auto. Qualquer escrita
num offset apenas de leitura é **aceite mas ignorada** (código de retorno sucesso).
Uma leitura/escrita fora do DB1 devolve o código de retorno S7 `0x0A` (objeto inexistente).

## 5. Exemplo de cliente

Com um cliente S7 (Snap7, `python-snap7`, nodes7…) configurado no IP/porta do
servidor, **rack 0 / slot 1** (valores habituais; o servidor não impõe o TSAP):

```python
import snap7, struct
c = snap7.client.Client()
c.connect("127.0.0.1", 0, 1, 102)
c.db_write(1, 0, struct.pack(">f", 80.0))   # Setpoint = 80.0
c.db_write(1, 16, bytes([0x01]))            # Run = true (bit 0)
pv = struct.unpack(">f", c.db_read(1, 4, 4))[0]  # Medida
```
