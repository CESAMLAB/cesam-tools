# Conceção — Regulador Sparkplug B (ORSE)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · **PT** · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

---

## 1. Visão geral

O ORSE reutiliza a arquitetura dos outros instrumentos CESAM-Lab: um **modelo de negócio
síncrono e testável** (regulador PID + processo), comandado por **atores
`ractor`** sobre Tokio, e uma **IHM `egui`** que lê um instantâneo partilhado. Apenas a
**camada de transporte** muda: aqui, um **edge node MQTT Sparkplug B** (cliente de saída)
em vez de um servidor Modbus/OPC UA.

```
        Command (cast)                      refresh a cada passo
IHM   ───────────────────►  SimulationActor ───────────────────►  SharedSnapshot (IHM)
NCMD (broker) ───────────►  (Regulator)      ──────────────────►  SharedSnapshot (publicação)
NBIRTH/NDATA (broker) ◄──────────────────────  SharedSnapshot
```

## 2. Atores

- **`SimulationActor`** — possui o único [`Regulator`]. Laço de passo fixo (`Tick`
  a cada 0,5 s); aplica os `Command` (IHM ou NCMD); publica o instantâneo
  após cada mutação. Idêntico aos outros instrumentos.
- **`SparkplugActor`** — possui o **cliente MQTT** (`rumqttc`) e executa o **ciclo
  de vida Sparkplug B** numa tarefa tokio dedicada (cujo `JoinHandle` é abatido ao
  parar). Uma mensagem `Reconfigure` relança o cliente se o broker/as credenciais/
  TLS mudarem.

## 3. Camada de protocolo

[`sparkplug_node.rs`](../../src/sparkplug_node.rs) é **puro e síncrono** (sem nenhuma
dependência de tokio/rumqttc): construção dos **topics**, tabela de **métricas**,
fabrico das **payloads** (`NBIRTH`/`NDATA`/`NDEATH`), (des)serialização protobuf,
mapeamento **`NCMD` → comandos** e o contador `seq`. É o equivalente do
`opcua_server.rs` do ORUE, isolado para ser **testável sem broker**.

### Escolha das bibliotecas

- **`rumqttc`** — cliente MQTT async Tokio (Last Will, reconexão automática, TLS
  via rustls — já na árvore via OPC UA, **sem OpenSSL**).
- **`sparkplug-rs`** — structs protobuf Eclipse Tahu (`Payload`/`Metric`/`Value`),
  gerados em **100 % Rust** (rust-protobuf, **sem `protoc`** → cross limpo). A
  crate reexporta `protobuf` (runtime), usado para `write_to_bytes`/`parse_from_bytes`.
- **Alternativa descartada: `srad`** — framework de alto nível de edge node Sparkplug que gere
  ele próprio `bdSeq`/`seq`/rebirth. Descartado deliberadamente: **possuímos** a máquina
  de estados no ator de rede para a tornar explícita e testável (coerência com
  os outros instrumentos).

## 4. Ciclo de vida e invariantes

- **`bdSeq`** incrementado a cada (re)arranque do cliente; **mesmo** valor no
  Last Will `NDEATH` e no `NBIRTH` de uma sessão.
- **`seq`** rolante 0–255, reposto a 0 a cada `NBIRTH`.
- **`NDEATH`** suportado pelo **Last Will MQTT**: robusto a qualquer perda de ligação.
- **Publicação `NDATA`** por **diff** de instantâneo (cadência = passo de simulação em
  modo *por alteração*, ou periódica). O bloqueio do snapshot **nunca** é mantido
  ao longo de um `.await`.

## 5. Postura de segurança

- **Sem lista branca de IP** (o instrumento é um cliente, não um servidor): desvio
  de paridade **assumido** face a ORME/OSNE.
- **MQTT em claro por defeito** (porta 1883) — não cifrado, não autenticado em rede.
  Faixa de aviso na IHM. Ativar **TLS** + credenciais para sair de uma
  rede de confiança.
- **Palavra-passe em claro** no TOML — **simulador apenas**.
- **Saneamento TOML** ([`AppConfig::sanitized`](../../src/config.rs)): processo/
  PID/limites finitos e ordenados, identificadores Sparkplug não vazios, temporizações
  limitadas. Toda escrita NCMD é **limitada/saneada** por `Regulator::apply`.
