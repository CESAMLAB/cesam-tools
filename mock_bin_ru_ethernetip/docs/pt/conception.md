# Conceção — Regulador EtherNet/IP (OREE)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · **PT** · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

---

## 1. Visão geral

O OREE reutiliza a arquitetura dos outros instrumentos CESAM-Lab: **modelo de negócio
síncrono e testável** (PID + processo), **atores `ractor`** sobre Tokio, **IHM
`egui`** que lê um instantâneo partilhado. Apenas a **camada de transporte** muda: um
**adaptador EtherNet/IP** (encapsulamento + CIP) em vez de Modbus/OPC UA/S7.

```
        Command (cast)                      refresh chaque pas
IHM   ───────────────────►  SimulationActor ───────────────────►  SharedSnapshot (IHM)
CIP Write Tag ───────────►  (Regulator)      ──────────────────►  SharedSnapshot
CIP Read Tag  ◄────────────────────────────────  SharedSnapshot
```

## 2. Atores

- **`SimulationActor`** — possui o único [`Regulator`]; aplica os `Command`
  (IHM ou escritas CIP); publica o instantâneo após cada mutação.
- **`EipServerActor`** — possui o **laço de escuta TCP**. Uma tarefa tokio liga o
  socket e aceita os clientes; cada sessão (com o seu *session handle*) é
  suportada por um `JoinSet` **interno** (abatido com o laço — nenhuma tarefa
  destacada). `Reconfigure` reinicia a escuta se o IP/porta mudar e atualiza a
  **lista branca** partilhada.

## 3. Camada de protocolo

[`eip_server.rs`](../../src/eip_server.rs) é **puro e síncrono**: encapsulamento
EtherNet/IP (`RegisterSession`, `SendRRData`/CPF) e CIP (`Read Tag`/`Write Tag` por
segmento simbólico). Tudo é **little-endian**. O parsing é **limitado** (slices
verificados): um pacote malformado vindo da rede **nunca** provoca pânico,
apenas uma ausência de resposta. É o equivalente do `opcua_server.rs`, isolado para
ser **testável sem socket**.

### Porquê um adaptador feito à mão

Não existe biblioteca **servidor/adaptador** EtherNet/IP em Rust (as
crates `rseip`, `rust-ethernet-ip`, `cip` são orientadas a **cliente/scanner**). O
subconjunto necessário (encapsulamento + CIP Read/Write Tag sobre tags nomeados) é
compacto: implementá-lo à mão dá um controlo total e uma superfície testável,
coerente com os outros instrumentos.

## 4. Política de sessões

Vários clientes **simultâneos** são aceites (comportamento de um adaptador), ao
contrário do mono-mestre do ORME. Cada sessão recebe um *session handle* e lê
o instantâneo atual; «o último que escreve ganha».

## 5. Postura de segurança

- **Nem autenticação nem cifragem** (EtherNet/IP «classic»): apenas a **lista
  branca de IP** e a topologia de rede protegem o acesso. `0.0.0.0` + lista vazia =
  exposto → faixa de aviso ([`NetworkConfig::is_exposed`](../../src/config.rs)).
- **Saneamento TOML** ([`AppConfig::sanitized`](../../src/config.rs)): processo/
  PID/limites finitos e ordenados. Toda a escrita CIP é **limitada/saneada** por
  `Regulator::apply`: a superfície de rede não pode produzir nem `NaN`/`Inf` nem valor
  aberrante.
- **Parsing de rede limitado**: nenhum pacote pode provocar pânico (cf. §3).
