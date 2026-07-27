# Documentação de manutenção — ORPD / PROFIBUS DP (workspace `cesam-tools`)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · **PT** · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

> Crate: `mock_bin_ru_pbdp` · Executável: **ru_pbdp** · Marca: **ORPD**
> Público-alvo: programadores que mantêm, corrigem ou estendem o projeto.
> Ver também: [conception.md](conception.md) · [reference_profibus.md](reference_profibus.md).

---

## 1. Pré-requisitos

- **Rust stable** (edição 2021, `rust-version` ≥ 1.85). Instalação:
  <https://rustup.rs>.
- **Dependências de sistema (Linux) para a IU** (`eframe`/`egui`,
  OpenGL/winit): `libxkbcommon-dev`, `libwayland-dev`, `libxcb1-dev`,
  `libgl1-mesa-dev` (ou equivalentes), mais um servidor gráfico
  (X11/Wayland). A IU necessita de um **ecrã**: num ambiente headless, a
  janela não abre.
- **Ligação série** (acesso à porta, `/dev/ttyUSB*`, grupo `dialout` em
  Linux): ao contrário do ORME/OSNE, **isto não é uma funcionalidade
  opcional** aqui — `tokio-serial` é uma dependência direta (ver §5),
  sendo a ligação série o único transporte deste instrumento (não existe
  equivalente padrão de «PROFIBUS sobre TCP»). Sem hardware, a IU arranca
  na mesma (o erro de abertura é mostrado no cabeçalho, a simulação
  continua a funcionar) — ver
  [`manuel_utilisateur.md`](manuel_utilisateur.md) §2.
- Acesso de rede ao registo crates.io para a primeira compilação.

---

## 2. Comandos habituais

```bash
cargo check -p mock_bin_ru_pbdp          # Verificação rápida (sem codegen)
cargo build -p mock_bin_ru_pbdp          # Compilação debug
cargo build --release -p mock_bin_ru_pbdp   # Compilação otimizada (LTO thin)
cargo test  -p mock_bin_ru_pbdp          # Testes unitários + de integração
cargo clippy --workspace --all-targets    # Lint (deve ficar SEM avisos)
cargo run   -p mock_bin_ru_pbdp          # Lança a IU + a ligação série PROFIBUS DP

# Ficheiro de configuração alternativo:
MOCK_CONFIG=./minha_config.toml cargo run -p mock_bin_ru_pbdp
# Registo detalhado:
RUST_LOG=debug cargo run -p mock_bin_ru_pbdp
```

Binário produzido: `target/debug/ru_pbdp` ou `target/release/ru_pbdp` (o
pacote Cargo mantém-se `mock_bin_ru_pbdp`; o executável e o nome
comercial «ORPD» são apenas documentais, ver `[[bin]]` no `Cargo.toml` do
crate).

### Features do Cargo

| Feature | Por defeito | Efeito |
|---------|:---------:|-------|
| `gui` | ✅ | IU `egui`/`eframe` + verificação de atualizações (caso contrário um binário headless) |

```bash
cargo build -p mock_bin_ru_pbdp --no-default-features   # headless: ligação série + simulação, sem IU
```

> ⚠️ **Diferença com o ORME/OSNE**: nestes dois instrumentos, a ligação
> série (RTU/série) é ela própria uma **funcionalidade opcional** ao
> lado de um transporte TCP sempre presente, e `--no-default-features`
> pode excluí-la. Aqui **não existe uma variante «sem série»**:
> `tokio-serial` é uma dependência direta (não controlada por feature),
> presente em **todas** as compilações, incluindo headless — é o único
> transporte do instrumento.

---

## 3. Organização do código

```
mock_lib_control/        Biblioteca de regulação reutilizável (pura, sem IO, testável)
  src/pid.rs             PID anti-windup
  src/lib.rs             reexportações (feature `serde` opcional)

mock_bin_ru_pbdp/        Binário regulador PROFIBUS DP (executável `ru_pbdp`)
  src/main.rs            Arranque: configuração, runtime Tokio, atores, IU/headless
  src/regulator.rs        Modelo de negócio síncrono (PID + processo de 1ª ordem), Command, passo
  src/config.rs           AppConfig (TOML), SerialConfig, ProcessConfig, RegulationConfig, ServerStatus
  src/profibus.rs         Protocolo PROFIBUS DP-V0: codec de tramas + FCS + SlaveFsm (FONTE DE VERDADE)
  src/profibus_server.rs  Ciclo de sessão série (ler trama → SlaveFsm → resposta) + watchdog
  src/map.rs              Disposição dos blocos de E/S Output/Input <-> Command do regulador
  src/trace.rs            Registo circular de tramas (mini-terminal da IU)
  src/gui.rs              IU egui (página única + mini-terminal + modal Definições)
  src/branding.rs         Logótipos incorporados (feature `gui`)
  src/i18n.rs             Catálogo i18n tipado (8 idiomas), sem dependência
  src/actors/
    simulation.rs         Ciclo de regulação (passo de simulação 50 ms)
    network.rs            Ator da ligação série PROFIBUS DP, reconfigurável a quente

docs/                     Conceção, referência PROFIBUS, manual, manutenção (multilingue)
```

**Regra de ouro**: a lógica de negócio (`mock_lib_control`,
`regulator.rs`, `profibus.rs`, `map.rs`) mantém-se **síncrona e testada**;
o assíncrono está confinado aos atores e à E/S série. Modelo de regulação
decalcado do **ORME** (`mock_bin_ru_modbus`) — mesmos invariantes.

---

## 4. Configuração

- Ficheiro: `mock_ru_pbdp.toml` no diretório atual, ou um caminho
  fornecido através da variável de ambiente `MOCK_CONFIG`.
- Carregado no arranque; **valores por defeito** se ausente ou ilegível
  (um aviso é registado, a aplicação arranca na mesma).
- **Todo o valor proveniente do TOML é saneado**
  (`AppConfig::sanitized`): limites de referência/PID reordenados,
  valores de vírgula flutuante forçados a finitos, `τ ≥ 1e-3`,
  `dead_time` limitado, **endereço de estação limitado a `[0, 125]`**.
  **Invariante: nunca chamar `f32::clamp` com limites não validados**
  (entra em pânico se `min > max` ou `NaN`).
- Guardado a partir da IU (botões *Aplicar* / *Guardar* / *Repor
  predefinições*).

Estrutura (todas as secções são opcionais, preenchidas por defeito):

```toml
language = "pt"
check_updates = true       # verificar no arranque se existe uma versão mais recente (IU)

[network.serial]
port = "/dev/ttyUSB0"      # "COM3" por defeito no Windows
baud = 500000              # valor normalizado PROFIBUS DP (9600 .. 12000000)
station_address = 3        # endereço do escravo simulado (0-125)
watchdog_enabled = true    # permite o watchdog anunciado pelo mestre (Set_Prm)

[process]
gain = 1.6 ; tau = 30.0 ; dead_time = 2.0 ; ambient = 20.0

[regulation]
sp_min = 0.0 ; sp_max = 250.0
hysteresis = 2.0 ; tor_min_cycle = 5.0 ; pwm_period = 10.0
[regulation.pid_heat]
kp = ... ; ki = ... ; kd = ... ; out_min = 0.0 ; out_max = 100.0
[regulation.pid_cool]
kp = ... ; ki = ... ; kd = ... ; out_min = 0.0 ; out_max = 100.0
```

> O **formato de trama série (8E1)** é fixado pela norma PROFIBUS DP e
> **não** é um campo de configuração — ver `SerialConfig::open` em
> [`config.rs`](../../src/config.rs). Ao contrário do ORME/OSNE, **sem
> lista branca de IP** (a ligação série é intrinsecamente ponto a
> ponto).

### Verificação de atualizações

Se `check_updates = true` (por defeito) **e** o binário for compilado
com a feature `gui`, a IU consulta **no arranque** a última versão
publicada no GitHub (`CESAMLAB/cesam-tools`) através do crate partilhado
**`mock_lib_update`** (`ureq`/`rustls`, raízes incorporadas, thread
limitada por tempo limite). **Ausente nas compilações headless**
(`--no-default-features`).

---

## 5. Dependências e armadilhas de versão

| Crate | Papel | Ponto de atenção |
|-------|------|-------------------|
| `tokio` | runtime assíncrono | features partilhadas + `io-util` |
| `ractor` | atores | features por defeito |
| `tokio-serial` | ligação PROFIBUS DP | **dependência direta, não controlada por feature** (ver §2); `default-features = false` (sem enumeração `libudev`) |
| `eframe`/`egui` | IU | versões ligadas entre si, feature `gui` |
| `egui_plot` | curva | ⚠️ **versionado uma minor à frente de `egui`**: para `egui` 0.33 → `egui_plot` **0.34** |
| `serde`/`toml` | persistência | `mock_lib_control` expõe uma feature `serde` ativada pelo binário |
| `mock_lib_update` (`ureq`/`rustls`) | verif. de atualizações | apenas feature `gui`; ausente em headless |

As versões partilhadas estão centralizadas em `[workspace.dependencies]`
do `Cargo.toml` raiz. Ao subir `egui`/`eframe`, **verificar a versão
correspondente de `egui_plot`** (caso contrário erro «two versions of
crate egui»).

---

## 6. Estender o projeto

### 6.1 Adicionar um serviço PROFIBUS (SAP)

Tudo acontece em **[`profibus.rs`](../../src/profibus.rs)** (fonte de
verdade do protocolo):

1. Adicionar a constante `SAP_*` e a variante correspondente em
   `enum Request`; ligar a descodificação em `decode_request` (e, para os
   testes, em `encode_request`).
2. Tratar o novo pedido em `SlaveFsm::handle` (transição de estado se
   pertinente, `Handled` devolvido).
3. Atualizar o comentário de documentação do módulo e
   **[reference_profibus.md](reference_profibus.md)**.
4. Adicionar um teste no módulo `tests` de `profibus.rs` (e, se a sessão
   completa for afetada, em `profibus_server.rs`).

### 6.2 Modificar os blocos de E/S (`Output`/`Input`)

1. Ajustar a disposição em **[`map.rs`](../../src/map.rs)**
   (`decode_output`/`encode_input`), mantendo `OUTPUT_LEN`/`INPUT_LEN`
   coerentes com `SlaveProfile` (`profibus_server.rs`).
2. Atualizar a tabela de
   **[reference_profibus.md](reference_profibus.md)** §3 (fonte de
   verdade documental, copiada do comentário de documentação de
   `map.rs`).
3. Adicionar um teste de ida e volta em `map.rs`.

### 6.3 Adicionar um comando de negócio / uma definição de IU

1. Variante em `enum Command` (`regulator.rs`) + tratamento em
   `Regulator::apply` (com saneamento).
2. Campo em `RegulatorSnapshot` se o valor tiver de ser observável.
3. Ligação da IU (`gui.rs`) via um `cast` não bloqueante.
4. Se persistente: campo em `AppConfig` (`config.rs`) + saneamento em
   `sanitized` + reporte em `to_regulator_config`.

### 6.4 Adicionar uma cadeia de interface (i18n)

Toda a cadeia de IU **deve** passar por uma chave `Msg` (`i18n.rs`) com as
suas **8 traduções** (array de tamanho fixo verificado em tempo de
compilação). Os identificadores de serviço PROFIBUS e os sufixos de
unidade permanecem codificados de forma fixa.

### 6.5 Adicionar um novo instrumento

1. Criar `mock_bin_<nome>/` e adicioná-lo aos `members` do `Cargo.toml`
   raiz.
2. Reutilizar `mock_lib_control`; fatorizar tudo o que for comum numa
   `mock_lib_*`.
3. Seguir a mesma divisão: modelo síncrono, ator(es) `ractor`, camada de
   protocolo, IU. Convenção de nome: `mock_bin_<tipo>_<protocolo>`.

---

## 7. Estratégia de testes

- **Codec de tramas** (`profibus.rs`): ida e volta de
  `SD1`/`SD2`/`SD3`/`SD4`, rejeição de soma de verificação e comprimento
  incorretos, codificação/descodificação dos pedidos
  (`Slave_Diag`/`Set_Prm`/`Chk_Cfg`/`Data_Exchange`) e do byte de modo.
- **Máquina de estados** (`profibus.rs`): sequência completa
  `Power_On → Wait_Prm → Wait_Cfg → Data_Exchange`, rejeição de um
  `Set_Prm` com identificador errado (permanece em `Wait_Prm`).
- **Blocos de E/S** (`map.rs`): um bloco de saída demasiado curto →
  nenhum comando; ida e volta de referência/modo; o bloco de entrada
  reflete a imagem partilhada (bits de estado, medida).
- **Configuração** (`config.rs`): ida e volta TOML, saneamento (limites
  invertidos, valores não finitos, endereço de estação fora do
  intervalo) sem pânico, erro limpo ao abrir uma porta série ausente.
- **Sessão de rede** (`profibus_server.rs`, `#[tokio::test]` sobre
  `tokio::io::duplex`): handshake completo até `Data_Exchange` com
  aplicação efetiva dos comandos, uma trama endereçada a outra estação
  ignorada (nenhuma atividade marcada), vencimento do watchdog a forçar
  o estado seguro.

Executar: `cargo test -p mock_bin_ru_pbdp` (ou `--workspace`) — **36
testes**, todos **determinísticos e sem IU**, nenhum teste
lento/`#[ignore]` (ao contrário do ORUE, cuja geração RSA justifica
testes ignorados).

---

## 8. Resolução de problemas

| Sintoma | Pista |
|----------|-------|
| «two versions of crate `egui`» | Discrepância `egui_plot` / `egui`: alinhar as versões (§5). |
| A IU não abre | Sem ecrã (headless) ou bibliotecas de sistema em falta (§1). |
| Erro ao abrir a porta série (cabeçalho da IU) | Porta ausente, caminho errado, ou permissões (grupo `dialout` em Linux) — a simulação continua a funcionar sem ligação. |
| A ligação permanece em `Wait_Prm` | O mestre não envia `Set_Prm` com o identificador esperado (`0xEE01`) — ver [reference_profibus.md](reference_profibus.md) §2. |
| A ligação permanece em `Wait_Cfg` | O `Chk_Cfg` recebido não anuncia `out_len=45`/`in_len=17`. |
| O aparelho para sozinho | Watchdog de protocolo acionado (silêncio prolongado do mestre) — estado seguro esperado, não uma falha. |
| Nenhum watchdog mesmo o mestre a solicitar um | `watchdog_enabled = false` na configuração local: o pedido do mestre é deliberadamente ignorado. |

Aumentar a verbosidade: `RUST_LOG=debug` (ou `trace`).

---

## 9. Compilação de distribuição

```bash
cargo build --release -p mock_bin_ru_pbdp
# Binário autónomo:
target/release/ru_pbdp
```

O perfil `release` ativa `lto = "thin"` e `opt-level = 3` (ver o
`Cargo.toml` raiz). Para distribuir: fornecer o binário mais um
`mock_ru_pbdp.toml` de exemplo. Licença **MIT** (ficheiro `LICENSE`).

### Feature `gui` (compilação com / sem interface)

```bash
cargo build --release -p mock_bin_ru_pbdp                       # com IU (posto de trabalho)
cargo build --release -p mock_bin_ru_pbdp --no-default-features  # «headless»: ligação série + simulação, sem IU
```

Ao contrário do OSNE, o modo **headless** não torna a ligação série
opcional (§2): remove apenas a IU. Mantém-se pertinente para uma
implantação sem ecrã ligada a uma porta série/USB real.

### Integração no ambiente de trabalho Linux (ícone da barra de tarefas)

O ícone ORPD (`pic/ru_pbdp-icon.png`, gerado por
[`pic/ru_pbdp-logo.gen.py`](../../../pic/ru_pbdp-logo.gen.py)) está
**incorporado** no binário (`branding.rs` → `window_icon`). Isto basta em
**X11, Windows e macOS**. Em **Wayland**, o compositor **ignora** o
ícone incorporado: associa a janela ao seu **`app_id`** («ru_pbdp»,
definido em `main.rs` via `with_app_id`) a um ficheiro `ru_pbdp.desktop`
do mesmo nome, e mostra o `Icon=ru_pbdp` resolvido no tema de ícones
`hicolor`.

Para obter o ícone em Wayland, instale a entrada de ambiente de trabalho
para o utilizador atual:

```bash
scripts/install-desktop.sh ru_pbdp
```

O script copia:

| Origem | Destino |
|--------|-------------|
| `pic/ru_pbdp-icon.png` | `~/.local/share/icons/hicolor/256x256/apps/ru_pbdp.png` |
| `packaging/ru_pbdp.desktop` | `~/.local/share/applications/ru_pbdp.desktop` |

e depois atualiza as caches. Três nomes **têm de permanecer alinhados**:
o `app_id` (`main.rs`), o ficheiro `ru_pbdp.desktop` (+ o seu
`StartupWMClass`) e o ícone `ru_pbdp.png` (= `Icon=ru_pbdp`).

---

## 10. Compilação «prod» — compilação cruzada a partir de Linux

Tudo é produzido **a partir de Linux** por
[`scripts/build-prod.sh`](../../../scripts/build-prod.sh), que compila
**todos os instrumentos do workspace** (tabela `INSTRUMENTS`, entrada
`mock_bin_ru_pbdp:ru_pbdp:0` — porta `0`: ligação série, sem porta IP):

| Saída | Alvo | IU | Método |
|--------|-------|-----|---------|
| `dist/ru_pbdp-linux-x86_64` | `x86_64-unknown-linux-gnu` | ✅ | `cross` |
| `dist/ru_pbdp-windows-x86_64.exe` | `x86_64-pc-windows-gnu` | ✅ | `cross` (mingw) |
| `dist/ru_pbdp-rpi-arm64` | `aarch64-unknown-linux-gnu` (Pi 3/4/5, Pi OS 64 bits) | ✅ | `cross` |
| Imagem Docker headless `ru_pbdp:headless` | multi-arch `amd64` + `arm64` | ❌ | `docker buildx` |
| `dist/ru_pbdp_<ver>_amd64.deb` / `_arm64.deb` | pacote Debian/Ubuntu | ✅ | `dpkg-deb` |
| `dist/ru_pbdp-setup-x86_64.exe` | instalador Windows | ✅ | NSIS (`makensis`) |

```bash
cargo install cross          # pré-requisito (uma vez) — o Docker tem de estar em execução
scripts/build-prod.sh        # todos os instrumentos, incluindo ru_pbdp
ONLY=ru_pbdp scripts/build-prod.sh   # apenas este instrumento
```

⚠️ **Não misturar `cargo` nativo e `cross`** no mesmo `target/`
(proc-macros incompatíveis → `can't find crate for …_derive`). O script
passa sempre por `cross`.

### Imagem Docker headless: utilidade limitada sem passthrough série

A imagem ([`docker/Dockerfile.headless`](../../../docker/Dockerfile.headless))
é construída como para os outros instrumentos (`EXPOSE 0`, metadado
inerte), mas **só é realmente útil com um dispositivo série montado** no
contentor:

```bash
docker run --rm --device=/dev/ttyUSB0 -v "$PWD/conf:/data" ru_pbdp:headless
```

Sem `--device`, o contentor arranca mas não consegue abrir nenhuma porta
série (mesmo comportamento que a ausência de hardware localmente — ver
§8).

---

## 11. Convenções

- Código e comentários em **francês** (convenção de todo o projeto);
  registos e mensagens de erro em **inglês**.
- `cargo clippy --workspace` **sem avisos** antes de qualquer commit.
- Todo o novo comportamento de negócio ou de protocolo é acompanhado de
  um **teste**.
- O protocolo PROFIBUS DP-V0 é modificado em **`profibus.rs`** (fonte de
  verdade), juntamente com uma atualização de
  **[reference_profibus.md](reference_profibus.md)**.
