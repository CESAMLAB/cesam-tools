# Concepção — Regulador de processo simulado (RU/OPC UA)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · **PT** · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

> Crate: `mock_bin_ru_opcua` · Executável: **ru_opcua** (*Regulation Unit over OPC UA*)

Documento de arquitetura e de modelação. Modelado a partir do regulador **ORME**
(`mock_bin_ru_modbustcp`): mesma divisão **modelo de negócio síncrono / atores
ractor / camada de protocolo / IHM egui**, mesmos invariantes. Apenas o
**transporte** muda: **OPC UA** em vez de Modbus.

---

## 1. Objetivo

Simular um **regulador de processo** (malha PID sobre um processo térmico de
primeira ordem) e expô-lo via **OPC UA**, o padrão de supervisão industrial
(Indústria 4.0). Ao contrário de ORME (Modbus) e OSNE (NAMUR) — protocolos
**de campo sem segurança** — o OPC UA suporta nativamente a autenticação, a
assinatura e a cifragem (previstas na Fase 2).

---

## 2. Modelo físico ([`regulator.rs`](../../src/regulator.rs))

O **processo** reutiliza [`mock_lib_control::FirstOrderProcess`] (partilhado com
o ORME): função de transferência de primeira ordem com atraso puro

```text
PV(s) / U(s) = K · e^(−L·s) / (1 + τ·s)
```

- `PV`: medição (unidade de processo, p. ex. °C);
- `U`: comando / saída (0-100 %);
- `K`: ganho estático; `τ`: constante de tempo; `L`: atraso puro;
- `ambient`: valor em repouso (saída nula).

Um **PID** ([`mock_lib_control::Pid`], também reutilizado do ORME) conduz a
medição para a **referência** comandando a saída, limitada a `[0, 100]`. Dois modos:
**automático** (o PID calcula a saída) e **manual** (saída imposta). O passo
de simulação é de **0,5 s** (processo térmico lento).

Todas as escritas (rede ou IHM) são **higienizadas** em `Regulator::apply`:
flutuantes não finitos ignorados, referência limitada, limites reordenados (`min ≤ max`),
ganhos PID limitados. **Invariante: nunca `f32::clamp` com limites não
validados** (panic se `min > max` ou `NaN`).

---

## 3. Arquitetura (atores)

```
IHM (egui) ───Command(cast)──►  SimulationActor ──refresh──► SharedSnapshot ──► IHM
Servidor OPC UA ─Command(cast)─►   (Regulator)    ──refresh──► SharedSnapshot ──► leituras OPC UA
```

- **`SimulationActor`** ([`actors/simulation.rs`](../../src/actors/simulation.rs)):
  proprietário **único** do `Regulator`; avança a simulação num temporizador
  one-shot rearmado (sem temporizador desanexado) e publica um `SharedSnapshot` a cada
  passo.
- **`OpcuaServerActor`** ([`actors/network.rs`](../../src/actors/network.rs)):
  possui o servidor OPC UA (tarefa tokio `server.run()`); reiniciável a quente
  (`Reconfigure`: rebind se o IP/porta mudar); conserva o `JoinHandle` (abandono
  ao parar) e o `ServerHandle` (cancelamento limpo das sessões); publica o seu
  estado de escuta para a IHM.
- **Servidor OPC UA** ([`opcua_server.rs`](../../src/opcua_server.rs)): constrói o
  servidor [`async-opcua`](https://crates.io/crates/async-opcua), declara o espaço
  de endereçamento e liga os callbacks. As **leituras** retiram do
  `SharedSnapshot`; as **escritas** emitem um `Command` para o
  `SimulationActor` por `cast` não bloqueante.

Como o NAMUR (OSNE) e ao contrário do Modbus do ORME, **não há tabela
de memória separada**: os nós OPC UA leem diretamente o instantâneo partilhado.

---

## 4. Pilha OPC UA — escolhas técnicas

- **`async-opcua`** (servidor, feature `server`): implementação **tokio-native**
  (uma tarefa por conexão), que se encaixa na stack ractor/tokio. Cripto
  **100 % Rust** (RustCrypto: `rsa`, `aes`, `sha2`, `x509-cert`) — **nenhuma
  dependência OpenSSL**, o que preserva a compilação cruzada (Linux/Windows/RPi).
- **Espaço de endereçamento**: um `SimpleNodeManager` em memória; nós `Variable`
  organizados sob `Objects` (cf. [`reference_opcua.md`](reference_opcua.md)).
- **Callbacks**: `add_read_callback` (valor vivo, amostrado para as
  subscrições) e `add_write_callback` (encaminha para a simulação).
- **Licença**: `async-opcua` está sob **MPL-2.0** (toda a linhagem OPC UA em Rust
  o está). Copyleft **por ficheiro**: uso não modificado → o código CESAM-Lab permanece
  MIT (cf. ficheiro `NOTICE` na raiz).

---

## 5. Segurança

A segurança é **regulável** (`SecurityConfig`) e constitui o diferenciador
do OPC UA face aos protocolos de campo (Modbus/NAMUR, sem segurança).

- **Modo não cifrado (predefinição)**: um endpoint `SecurityPolicy::None`, token
  **anónimo** — rede de confiança apenas, arranque instantâneo, nenhum
  certificado. A IHM exibe um **aviso laranja**.
- **Modo cifrado (Fase 2)**: endpoint `Basic256Sha256` / `SignAndEncrypt`. Um
  **certificado de instância** auto-assinado é gerado no primeiro arranque (`pki/`);
  o servidor confia nos certificados de cliente. **Autenticação** por
  utilizador/palavra-passe (`ServerUserToken::user_pass`) e/ou anónimo. A IHM
  exibe um **aviso verde** 🔒.

O modo regula-se no modal *Parâmetros*; uma alteração **reinicia** o servidor
a quente (`OpcuaServerActor`).

---

## 6. Configuração & persistência

`AppConfig` (idioma / rede / processo / regulação / verif. atualização) serializada em
**TOML** ([`config.rs`](../../src/config.rs)), **higienizada no carregamento**
(`AppConfig::sanitized`: limites ordenados, `τ ≥ 1e-3`, `dead_time ≥ 0`, flutuantes
finitos). Ficheiro: `mock_ru_opcua.toml` (substituível por `MOCK_CONFIG`).

---

## 7. Pistas de evolução

- **Fase 2**: segurança OPC UA (certificados, cifragem, autenticação).
- Métodos OPC UA (`Reset`, `Autotune`) além das variáveis.
- Modelo de informação tipado (ObjectType regulador) em vez de variáveis planas.
- Historização / `HistoryRead` sobre a medição.
- Promoção do modelo regulador do ORME numa `mock_lib_*` partilhada (está
  hoje duplicado entre ORME e este instrumento).
