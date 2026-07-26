# Conceção — Regulador S7 (ORSS)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · **PT** · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

---

## 1. Visão geral

O ORSS reutiliza a arquitetura dos outros instrumentos CESAM-Lab: **modelo de negócio
síncrono e testável** (PID + processo), **atores `ractor`** sobre Tokio, **IHM
`egui`** que lê um instantâneo partilhado. Apenas a **camada de transporte** muda: um
**servidor S7comm** (ISO-on-TCP / RFC1006) em vez de Modbus/OPC UA.

```
        Command (cast)                      refresh a cada passo
IHM   ───────────────────►  SimulationActor ───────────────────►  SharedSnapshot (IHM)
S7 Write Var ────────────►  (Regulator)      ──────────────────►  SharedSnapshot
S7 Read Var  ◄────────────────────────────────  SharedSnapshot (imagem DB1)
```

## 2. Atores

- **`SimulationActor`** — possui o único [`Regulator`]. Ciclo de passo fixo;
  aplica os `Command` (IHM ou escritas S7); publica o instantâneo após cada
  mutação.
- **`S7ServerActor`** — possui o **ciclo de escuta TCP**. Uma tarefa tokio dedicada
  liga o socket e aceita os clientes; cada sessão é suportada por um `JoinSet`
  **interno** (portanto encerrada com o ciclo — nenhuma tarefa destacada). `Reconfigure`
  relança a escuta se o IP/porta mudar e atualiza a **lista branca** partilhada.

## 3. Camada de protocolo

[`s7_server.rs`](../../src/s7_server.rs) é **puro e síncrono** (sem qualquer dependência
de rede): framing TPKT, COTP (CR→CC, DT) e S7comm (Setup, Read Var, Write Var) sobre
uma **imagem de bytes DB1**. A análise é **limitada** (acesso por `get`/slices
verificados): uma trama malformada vinda da rede **nunca** provoca pânico,
apenas uma ausência de resposta. É o equivalente S7 do `opcua_server.rs`, isolado
para ser **testável sem socket**.

### Porquê um servidor feito à mão

Não existe biblioteca **servidor** S7 em Rust (as crates `s7`/`s7-comm` são
orientadas para **cliente**). O subconjunto necessário (COTP classe 0 + S7 Read/
Write Var sobre um DB) é compacto e bem especificado: implementá-lo à mão proporciona
controlo total e uma superfície testável, coerente com os outros instrumentos.

## 4. Política de sessões

Vários clientes S7 **simultâneos** são aceites (comportamento de autómato), ao
contrário do mono-mestre do ORME (despejo) e do ponto-a-ponto do OSNE (ocupação).
Cada sessão lê a imagem DB1 atual e encaminha as suas escritas para a simulação;
o "último a escrever ganha", como um autómato real.

## 5. Postura de segurança

- **Nem autenticação nem cifragem** (S7 "classic"): apenas a **lista branca
  de IP** e a topologia de rede protegem o acesso. `0.0.0.0` + lista vazia = exposto →
  faixa de aviso na IHM ([`NetworkConfig::is_exposed`](../../src/config.rs)).
- **Saneamento TOML** ([`AppConfig::sanitized`](../../src/config.rs)): processo/
  PID/limites finitos e ordenados. Qualquer escrita S7 é **limitada/saneada** por
  `Regulator::apply`: a superfície de rede não pode produzir nem `NaN`/`Inf` nem valor
  aberrante.
- **Análise de rede limitada**: nenhuma trama pode provocar pânico (cf. §3).
