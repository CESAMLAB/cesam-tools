# Documento de conceção — Regulador simulado Modbus TCP

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · **PT** · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

> Produto: **ORME** · Crate: `mock_bin_ru_modbustcp` · Workspace: `cesam-tools` · Licença: MIT

Este documento descreve a arquitetura, as opções técnicas e os princípios de
funcionamento do regulador industrial simulado. Destina-se aos programadores
que mantêm ou estendem o projeto.

---

## 1. Objetivo e âmbito

Fornecer um **instrumento industrial virtual**: um regulador de processo que se
comporta de forma realista e comunica em **Modbus TCP** (escravo), a fim de
desenvolver e testar supervisores / autómatos / gateways **sem hardware**.

O simulador abrange:

- um **processo físico** modelado por uma função de transferência;
- uma **regulação** bidirecional (aquecimento / arrefecimento): PID, tudo-ou-nada (TOR) ou
  relé de ciclo (PWM);
- uma **interface Modbus TCP** que expõe o estado completo;
- uma **IHM** de comando, visualização e parametrização;
- a **persistência** dos parâmetros.

Fora do âmbito atual: Modbus RTU, redundância, historização de longo prazo,
autenticação forte (apenas é fornecida uma lista branca de IP).

---

## 2. Vista de conjunto

```
┌──────────────────────────────────────────────────────────────────────┐
│                       Processo (thread principal)                      │
│                                                                        │
│   ┌─────────────────────────┐         lê (Mutex)                       │
│   │   IHM  egui / eframe     │◄──────────────── SharedSnapshot         │
│   │   (gui.rs)               │◄──────────────── SharedStatus           │
│   └───────────┬─────────────┘                                          │
│               │ cast (não bloqueante)                                  │
└───────────────┼────────────────────────────────────────────────────────┘
                │
   ┌────────────┼──────────── Runtime Tokio (threads de fundo) ──────────┐
   │            ▼                                                         │
   │   ┌──────────────────┐  refresh  ┌──────────────┐                   │
   │   │ SimulationActor   ├──────────►│ SharedSnapshot│ (IHM)            │
   │   │  (ractor)         ├──────────►│ SharedMap     │ (Modbus)         │
   │   │  possui o          │           └──────┬───────┘                  │
   │   │  Regulator         │◄── Command ──┐    │ lê                      │
   │   └──────────────────┘              │    ▼                          │
   │          ▲ Command (cast)            │  ┌──────────────────────┐     │
   │          │                           └──┤ RegulatorService      │     │
   │   ┌──────┴───────────┐  gere/rebind     │ (trait Service)       │     │
   │   │ ModbusServerActor ├─────────────────►  servidor Modbus TCP  │◄──── clientes
   │   │  (ractor)         │  filtro IP ──────► (tokio-modbus)        │     │
   │   └──────────────────┘   (SharedAllowlist)└──────────────────────┘     │
   └────────────────────────────────────────────────────────────────────┘
```

Princípio orientador: **um único proprietário do estado de negócio**. O `Regulator`
nunca é partilhado; vive em `SimulationActor`. Todas as escritas
(IHM ou Modbus) são **mensagens** `Command`. As leituras fazem-se sobre
**cópias** atualizadas a cada passo (`SharedSnapshot`, `SharedMap`), o que elimina
os bloqueios sobre a lógica e as condições de corrida.

---

## 3. Opções técnicas

| Necessidade | Escolha | Justificação |
|--------|-------|---------------|
| Concorrência | **`ractor`** (atores) sobre **Tokio** | Isola o estado mutável num ator; mutações serializadas por mensagens, sem bloqueio aplicacional. Preferência do projeto. |
| Modbus TCP escravo | **`tokio-modbus`** (`tcp-server`) | Implementação async madura; o trait `Service` mapeia de forma limpa pedido→resposta. |
| IHM | **`egui` / `eframe`** + `egui_plot` | Modo imediato, multiplataforma, sem estado de UI complexo a sincronizar. |
| Processo | **FOPDT** (1ª ordem + atraso) | Modelo padrão e suficiente para um processo térmico; poucos parâmetros, intuitivo. |
| Persistência | **`serde` + `toml`** | Formato legível/editável à mão, ideal para parâmetros de aparelho. |

### Porquê separar a lógica síncrona da assíncrona

`mock_lib_control` e `regulator.rs` são **puramente síncronos** (nenhuma IO,
nenhum async). Vantagens: testáveis unitariamente de forma determinista,
reutilizáveis por outros instrumentos e razoáveis de reler. O assíncrono
fica confinado aos **atores** e à **camada de rede**.

---

## 4. Modelo de dados

### Estado de negócio (`regulator.rs`)

- `Regulator` — agregado proprietário: modos, consignas, reguladores (`Pid`,
  `OnOff`) e processo (`FirstOrderProcess`). Não é `Clone`, não é partilhado.
- `RegulatorConfig` — configuração estática (processo, ganhos, limites, `dt`).
  **Fonte única** dos valores predefinidos (a config TOML deriva dela).
- `RegulatorSnapshot` — **cópia imutável** (`Copy`) do estado observável, publicada
  a cada passo. É o contrato de leitura para a IHM e a tabela Modbus.
- `Command` — enumeração das mutações possíveis (marcha, modo, consignas,
  regulações, processo, limites).

### Estruturas partilhadas (`actors/mod.rs`, `config.rs`)

| Tipo | Conteúdo | Escrito por | Lido por |
|------|---------|-----------|--------|
| `SharedSnapshot` | `RegulatorSnapshot` tipado | SimulationActor | IHM |
| `SharedMap` | `MemoryMap` (imagens das 4 tabelas Modbus) | SimulationActor | RegulatorService |
| `SharedAllowlist` | `IpFilter` | ModbusServerActor | aceitação de ligações |
| `SharedStatus` | `ServerStatus` (escuta / erro) | ModbusServerActor | IHM |

Todos são `Arc<Mutex<…>>`: secções críticas **curtas** (cópia / refresh),
nunca mantidas durante um cálculo ou uma IO.

---

## 5. Componentes

### 5.1 `mock_lib_control` (biblioteca)

- `Pid` — PID em tempo discreto, derivada sobre o erro, **anti-saturação** por
  limitação do termo integral. API: `step(sp, pv, dt)` ou `step_with_error(err, dt)`
  (reutilizado para o sentido de arrefecimento).
- `OnOff` — tudo-ou-nada com **histerese simétrica** (zona morta) **e
  anti-ciclo-curto**: um tempo de ciclo mínimo (`min_cycle`, s) impede qualquer
  comutação enquanto o relé não tiver permanecido tempo suficiente no seu estado,
  modelando a proteção de um atuador real. O relé **mantém** o seu estado:
  cabe ao chamador passar-lhe o erro com sinal sem o reinicializar na
  mudança de sinal (ver § 5.2).
- `Pwm` — modulador de largura de impulso (**relé de ciclo** /
  *time-proportioning*): num período fixo `T_c`, a saída tudo-ou-nada fica
  ativa a fração `duty` do ciclo (`duty` **amostrado uma vez por ciclo**
  para evitar um enviesamento em regime estabelecido). Permite regular finamente um órgão TOR.
- `FirstOrderProcess` — função de transferência `K·e^(-L·s)/(1+T·s)`, integração
  de Euler + linha de atraso. `reconfigure(...)` altera os parâmetros sem salto.
- `ControllerKind` — `Off` / `Pid` / `OnOff` / `Pwm`, com codificação Modbus
  (`to_code`/`from_code`).

### 5.2 `regulator.rs`

Orquestração da regulação a cada passo (`step`):

1. se **parado** → saída 0, reguladores reinicializados;
2. se **manual** → saída = consigna manual (% com sinal);
3. se **auto** → calcula-se **separadamente** a contribuição do sentido de aquecimento (sentido 1,
   erro `SP − PV`) e do sentido de arrefecimento (sentido 2, erro `PV − SP`), cada uma ≥ 0,
   depois `saída = aquecimento − arrefecimento`:
   - **PID**: saída limitada a `[0, 100]` (`out_min = 0`) — o sentido inativo (erro
     negativo) sai 0 e a sua integral **purga-se naturalmente** por limitação. Não
     a repomos a zero à força: com a forte ondulação do PWM, apagá-la
     a cada ultrapassagem da consigna introduziria um erro estático;
   - **TOR**: o relé é avaliado sobre o erro com sinal e conserva o seu estado na
     travessia da consigna, o que restaura uma banda de histerese **simétrica**
     `[SP − h/2, SP + h/2]` (as bandas de aquecimento/arrefecimento permanecem disjuntas, pelo que os
     dois relés são mutuamente exclusivos);
   - **PWM**: um PID calcula a razão cíclica, modulada pelo relé de ciclo;
     a saída física é estritamente 0 % ou 100 %, mas a sua média segue o PID.
4. a saída comanda o processo que produz a nova medida (PV).

> **Histórico**: antes desta revisão, o encaminhamento aquecimento/arrefecimento fazia-se pelo
> sinal do erro e **reinicializava** o relé TOR na travessia da
> consigna — o que truncava a histerese em `[SP − h/2, SP]` (metade da banda,
> assimétrica) e tornava a regulação TOR medíocre. O cálculo por sentido separado
> corrige este defeito.

### 5.3 `actors/simulation.rs`

`SimulationActor` (ractor). `pre_start` arma um `send_interval(dt)` que emite
`Tick`. `handle` trata `Tick` (avança a simulação) e `Command` (aplica uma
mutação), depois **publica** o estado em `SharedSnapshot` e `SharedMap`.

### 5.4 `actors/network.rs`

`ModbusServerActor` possui o servidor Modbus. `Reconfigure(NetworkConfig)`:
- atualiza a **lista branca** partilhada (efeito imediato, sem reinício);
- se o **transporte** (TCP/RTU), a **porta / IP** ou os **parâmetros série**
  mudarem, **para** a tarefa do servidor e **reinicia-a** (`start_tcp` ou
  `start_rtu`); publica o estado em `SharedStatus` (sucesso ou erro).

Um **único transporte** está ativo de cada vez (`Transport::Tcp` ou `Rtu`). O RTU está
atrás da **feature `rtu`**; sem ela, selecionar RTU publica um erro de
estado explícito.

### 5.5 `modbus_server.rs`

`RegulatorService` implementa `tokio_modbus::server::Service` de forma
**síncrona** (`future::Ready`): leituras = recorte de `SharedMap`; escritas =
descodificação em `Command` (via `map.rs`) e depois `cast` para `SimulationActor`.

**Política mono-mestre.** `serve` (TCP) só autoriza **um mestre remoto de cada
vez**: a cada nova ligação (IP autorizada pela lista branca), a
anterior é fechada. Mecanismo: o `TcpStream` é envolvido num
`CancellableStream` que, ao receber um sinal `oneshot`, devolve **EOF na
leitura** — o laço de tratamento do `tokio-modbus` termina então e fecha o
socket. `serve_rtu` (feature `rtu`) serve o barramento série via
`rtu::Server::serve_forever`: o barramento RS485 *é* o único mestre (nada a expulsar).

> ⚠️ A IHM não percorre este caminho: envia os seus `Command` diretamente ao
> ator, pelo que nunca é contabilizada como mestre.
>
> ⚠️ O servidor RTU do `tokio-modbus` 0.17 não transmite o endereço de escravo ao
> serviço: o aparelho responde, portanto, independentemente do endereço solicitado. Uma ligação
> **ponto-a-ponto** é recomendada. `slave_id` é persistido e apresentado, mas não
> utilizado para filtrar (limitação a montante).

### 5.6 `map.rs`

**Fonte de verdade** do plano de endereçamento Modbus. Constantes de endereços,
`MemoryMap` (imagens das tabelas), `refresh_from(snapshot)` (estado→registos) e
`*_to_command(s)` (escritas→comandos). Codificação dos `f32` em 2 registos,
big-endian, palavra de maior peso em primeiro.

### 5.7 `config.rs`

`AppConfig` (rede / processo / regulação) ⇄ TOML. `IpFilter` (caracteres curinga `*` por
octeto IPv4). `ServerStatus`. `to_regulator_config()` faz a ponte para o domínio.

### 5.8 `gui.rs`

IHM de **página única**: cabeçalho (estados + botões), painel de comandos (esquerda),
supervisão + curva (centro), tabela Modbus ao vivo (direita), modal Parâmetros.
Lê os `Shared*`, envia `Command` por `cast` não bloqueante.

---

## 6. Cenários (sequências)

**Leitura Modbus (PV)**: cliente → `RegulatorService::call(ReadInputRegisters)` →
leitura de `SharedMap` → `Response`. Nenhuma interação com o ator (latência mínima).

**Escrita Modbus (consigna)**: cliente → `call(WriteMultipleRegisters)` →
`map::holdings_to_commands` → `cast(Command::SetSpAuto)` → o ator aplica no
passo seguinte → republica `SharedMap`/`SharedSnapshot`.

**Comando IHM**: interação → `cast(Command)` → idem.

**Reconfiguração de rede**: modal *Aplicar* → `cast(Reconfigure)` →
ModbusServerActor re-vincula se necessário → `SharedStatus` atualizado → o cabeçalho
da IHM reflete o estado.

**Tick**: temporizador → `Tick` → `Regulator::step` → publicação.

---

## 7. Teoria da regulação

**Processo (FOPDT)**: `v[k+1] = v[k] + (dt/T)·(alvo − v[k])`, com
`alvo = ambiente + K·u` e `u` atrasado de `L` segundos (linha de atraso).

**PID**: `u = Kp·e + Ki·∫e + Kd·de/dt`, integral limitada a `[out_min, out_max]`
(anti-windup). Derivada sobre o erro (compromisso simplicidade/simetria aquecimento-arrefecimento).

**TOR**: ativo se `e > +H/2`, inativo se `e < −H/2`, caso contrário estado conservado.

**Bidirecional**: um único sentido atua de cada vez, selecionado pelo sinal do
erro; a saída global tem sinal (+ aquecimento / − arrefecimento).

---

## 8. Decisões e compromissos

- **Dupla publicação (`Snapshot` + `Map`)** em vez de uma única estrutura:
  a IHM manipula tipos de negócio, o Modbus registos brutos; ambos
  permanecem simples e desacoplados, ao preço de um ligeiro sobrecusto de cópia negligenciável.
- **Leituras Modbus sem passar pelo ator**: lê-se `SharedMap` diretamente
  para minimizar a latência; o ator permanece o único **escritor**, logo sem corrida.
- **Serviço Modbus síncrono** (`future::Ready`): todo o trabalho é não bloqueante
  (lock curto + cast), inútil empacotar um futuro.
- **Re-vínculo na mudança de porta**: um socket não muda de porta; aceita-se
  uma curta interrupção de serviço na reconfiguração.
- **Derivada sobre o erro** (e não sobre a medida): ligeiro «coice» na
  mudança de consigna, aceite para manter o algoritmo simétrico e simples.

---

## 9. Evoluções possíveis

- Modbus RTU / série (reutilizar `RegulatorService`, mudar o transporte).
- Rampa de consigna, auto-tuning PID, falhas simuladas (sensor avariado, saturação).
- Historização / exportação CSV da tendência.
- Migração da IHM para **separadores** se a página única ficar demasiado densa.
- Novos instrumentos: criar `mock_bin_<nome>` e fatorizar o comum em
  `mock_lib_*` (ver [maintenance.md](maintenance.md)).
