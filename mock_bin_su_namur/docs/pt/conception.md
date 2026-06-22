# Conceção — Agitador de laboratório simulado (OSNE)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · **PT** · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

> Crate: `mock_bin_su_namur` · Executável: **OSNE** (*Open Stirrer NAMUR Emulator*)

Documento de arquitetura e de modelação. Decalcado do regulador **ORME**
(`mock_bin_ru_modbustcp`): mesma divisão **modelo de negócio síncrono / atores
ractor / camada de protocolo / IHM egui**, mesmos invariantes.

---

## 1. Objetivo

Simular um **agitador de laboratório** (estilo IKA) comandado pelo protocolo série
**NAMUR**. O motor possui uma **função de transferência** (dinâmica de velocidade)
controlada por uma **regulação rápida**, e a **viscosidade** do meio é ajustável
e influencia o binário.

---

## 2. Modelo físico

### Motor ([`motor.rs`](../../src/motor.rs))

Equilíbrio dos binários, integrado por Euler explícito:

```text
J · dω/dt = T_moteur − k · η · ω − T_frottement
```

- `ω`: velocidade (tr/min);
- `T_moteur`: binário do motor (comando, N·cm, ≥ 0);
- `k · η · ω`: **binário de carga viscoso** (∝ viscosidade `η` e velocidade);
- `T_frottement`: atrito seco residual;
- `J` (`inertia`): regula a **reatividade** (pequeno ⇒ rápido).

Em regime estabelecido, `T_moteur = k·η·ω + T_frottement`: o binário necessário
para manter uma velocidade **cresce com a viscosidade**. Se esse binário ultrapassa
o **binário máximo**, a consigna deixa de ser atingível → **sobrecarga**.

### Controlo ([`stirrer.rs`](../../src/stirrer.rs))

Um **PID** ([`mock_lib_control::Pid`], reutilizado do ORME) recebe o erro de
velocidade `consigna − medição` e produz o **binário do motor**, limitado a
`[0, couple_max]`. Os ganhos por defeito são deliberadamente «agressivos»: a saída
satura no binário máximo enquanto o erro é grande (subida rápida), depois o termo
integral estabiliza. O passo de simulação é de **20 ms** (50 Hz), mais fino do que
o do ORME porque a dinâmica de um motor é rápida.

---

## 3. Arquitetura (atores)

```
IHM (egui) ──Command(cast)──►  SimulationActor ──refresh──► SharedSnapshot ──► IHM
Servidor NAMUR ──Command(cast)─►  (Stirrer)     ──refresh──► SharedSnapshot ──► leituras NAMUR
```

- **`SimulationActor`** ([`actors/simulation.rs`](../../src/actors/simulation.rs)):
  proprietário único do `Stirrer`; avança a simulação num temporizador one-shot
  rearmado (sem temporizador destacado) e publica um `SharedSnapshot`.
- **`NamurServerActor`** ([`actors/network.rs`](../../src/actors/network.rs)):
  possui o servidor NAMUR, reiniciável a quente (`Reconfigure`); lista branca de IP
  partilhada; estado de escuta publicado para a IHM.
- **Servidor NAMUR** ([`namur_server.rs`](../../src/namur_server.rs)): lê as linhas
  ASCII, interpreta-as ([`namur.rs`](../../src/namur.rs)), responde às leituras e
  encaminha as escritas/ações para o ator. **Um mestre de cada vez**
  (ponto-a-ponto). **Cão de guarda** por sessão.

As leituras NAMUR vão buscar ao `SharedSnapshot` (sem tabela de memória separada
como o Modbus do ORME: o protocolo NAMUR é orientado a «comandos», não a
«registos»).

---

## 4. Configuração e segurança

- `AppConfig` (idioma / rede-série / motor / regulação) serializada em **TOML**
  ([`config.rs`](../../src/config.rs)), **higienizada no carregamento**
  (`AppConfig::sanitized`: limites ordenados, flutuantes finitos) — invariante
  partilhado com o ORME (nunca fazer `clamp` com limites não validados).
- NAMUR não tem **nem autenticação nem cifragem**: rede de confiança + lista
  branca de IP (TCP). Por defeito `0.0.0.0` + lista vazia ⇒ exposto: a IHM mostra
  um **aviso em faixa**.

---

## 5. Pistas de evolução

- Sentido de rotação (CW/CCW) e rampa de aceleração.
- Sensor de temperatura (`IN_PV_2/3`) se for adicionado um modelo térmico.
- Binário de carga não linear (regime turbulento ∝ ω²).
- Promoção do modelo de motor para `mock_lib_control` se servir um segundo instrumento.
