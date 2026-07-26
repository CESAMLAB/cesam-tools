# Referência MQTT Sparkplug B — métricas e ciclo de vida (RU/Sparkplug B)

*🌍 [FR](../fr/reference_sparkplugb.md) · [EN](../en/reference_sparkplugb.md) · [DE](../de/reference_sparkplugb.md) · [ES](../es/reference_sparkplugb.md) · [IT](../it/reference_sparkplugb.md) · **PT** · [NL](../nl/reference_sparkplugb.md) · [PL](../pl/reference_sparkplugb.md)*

> Fonte de verdade: [`sparkplug_node.rs`](../../src/sparkplug_node.rs) (topics, tabela
> de métricas, payloads, mapeamento NCMD). Toda evolução se faz **neste ficheiro**
> e repercute-se aqui.

---

## 1. Papel e ligação

O instrumento é um **edge node Sparkplug B**: não **escuta nenhuma porta**, ele
**liga-se em saída** a um **broker MQTT externo** (mosquitto, EMQX, HiveMQ…) e
publica o estado do regulador. Ajustes na secção `[network]` do TOML / no modal
*Parâmetros*:

| Chave | Defeito | Papel |
|---|---|---|
| `broker_host` | `localhost` | host do broker MQTT |
| `broker_port` | `1883` | porta (`8883` em TLS) |
| `client_id` | `ru_spb` | identificador de cliente MQTT |
| `group_id` | `CESAM` | grupo Sparkplug (`spBv1.0/<group_id>/…`) |
| `edge_node_id` | `RU1` | nó edge (`…/<edge_node_id>`) |
| `username` / `password` | *(vazio)* | auth MQTT (palavra-passe **em claro**, simulador apenas) |
| `tls` | `false` | cifra TLS (rustls) para o broker |
| `keepalive_secs` | `30` | keepalive MQTT |
| `publish_on_change` | `true` | `true`: `NDATA` assim que uma métrica muda (cadência = passo de simulação, 0,5 s); `false`: periódica |
| `publish_period_secs` | `5` | cadência periódica quando `publish_on_change = false` |

> ⚠️ **MQTT em claro por defeito**: sem TLS, o tráfego não é cifrado nem
> autenticado em rede. A usar apenas numa **rede de confiança**. A IHM exibe
> uma faixa de aviso enquanto `tls` estiver desativado.

---

## 2. Espaço de nomes (topics)

Namespace `spBv1.0`. Topics do nó:

```
spBv1.0/<group_id>/NBIRTH/<edge_node_id>
spBv1.0/<group_id>/NDATA/<edge_node_id>
spBv1.0/<group_id>/NDEATH/<edge_node_id>
spBv1.0/<group_id>/NCMD/<edge_node_id>
```

Com os valores por defeito: `spBv1.0/CESAM/NBIRTH/RU1`, etc.

---

## 3. Tabela de métricas

Todas as métricas de dados vivem sob o **nó edge** (sem *device* nesta
versão). Tipo Sparkplug (Eclipse Tahu): `Float` (9), `Boolean` (11),
`UInt64` (8).

| Métrica | Tipo | Leitura/Escrita | Campo instantâneo (leitura) | NCMD → comando (escrita) |
|---|---|:--:|---|---|
| `Setpoint` | Float | R/W | `setpoint` | `SetSetpoint` |
| `ProcessValue` | Float | R | `pv` | — |
| `Output` | Float | R | `output` | — |
| `ManualOutput` | Float | R/W | `manual_output` | `SetManualOutput` |
| `Run` | Boolean | R/W | `run` | `SetRun` |
| `Auto` | Boolean | R/W | `auto` | `SetAuto` |
| `SetpointMin` | Float | R | `sp_min` | *(ajustado via IHM/TOML)* |
| `SetpointMax` | Float | R | `sp_max` | *(ajustado via IHM/TOML)* |
| `PID/Kp` | Float | R | `pid.kp` | *(ajustado via IHM/TOML)* |
| `PID/Ki` | Float | R | `pid.ki` | *(ajustado via IHM/TOML)* |
| `PID/Kd` | Float | R | `pid.kd` | *(ajustado via IHM/TOML)* |
| `bdSeq` | UInt64 | R | *(contador de sessão)* | — |
| `Node Control/Rebirth` | Boolean | W | — | republica um `NBIRTH` |

**Superfície comandável por `NCMD`**: `Setpoint`, `ManualOutput`, `Run`, `Auto`, mais
`Node Control/Rebirth` (paridade com as escritas OPC UA do instrumento ORUE). Os
limites de consigna e os ganhos PID são **publicados** (observáveis por um SCADA) mas
ajustam-se via IHM/TOML. Uma métrica desconhecida ou de **tipo errado** num
`NCMD` é **ignorada** (nunca um erro, nunca um valor aberrante: a simulação
saneia qualquer escrita).

---

## 4. Ciclo de vida

- **`NBIRTH`** — publicado a cada ligação (ConnAck). Contém **todas** as
  métricas (com valores), `bdSeq`, e `Node Control/Rebirth`. `seq = 0`.
- **`NDATA`** — métricas **alteradas** apenas, `seq` rolante **0–255**.
- **`NDEATH`** — contém `bdSeq` **só**, **sem** `seq`. Depositado como **Last Will
  MQTT** na ligação: o **broker** publica-o automaticamente na perda do link
  (paragem, reconfiguração, falha). Sem `NDEATH` explícito do lado do nó.
- **`NCMD`** — subscrição `spBv1.0/<group>/NCMD/<node>` (QoS 1) subscrita logo após
  o `NBIRTH`. Descodificado → comandos aplicados à simulação.
- **`bdSeq`** — incrementado a cada (re)arranque do cliente; o `NDEATH` (Last Will)
  e o `NBIRTH` de uma **mesma sessão** portam o **mesmo** valor (invariante
  Sparkplug). Exibido na IHM (diagnóstico).
- **`seq`** — reposto a 0 a cada `NBIRTH`, incrementado (rolante) a cada `NDATA`.
- **Renascimento** (`Node Control/Rebirth = true` via `NCMD`) → republicação de um
  `NBIRTH` (ressincronização SCADA).

---

## 5. Exemplo de cliente (SCADA)

Subscrição a todo o grupo, depois envio de uma consigna:

```bash
# Observar as mensagens do nó
mosquitto_sub -h localhost -t 'spBv1.0/CESAM/#' -v

# (as payloads são protobuf Sparkplug B — usar um descodificador Tahu para as ler)
```

Um `NCMD` publicado em `spBv1.0/CESAM/NCMD/RU1` com as métricas `Run=true` e
`Setpoint=80.0` arranca a regulação e fixa a consigna; um `NDATA` posterior
reflete a alteração.
