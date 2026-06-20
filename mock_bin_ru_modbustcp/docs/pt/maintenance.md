# Documentação de manutenção — ORME (workspace `cesam-tools`)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · **PT** · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

> Público: programadores que mantêm, corrigem ou estendem o projeto.
> Ver também: [conception.md](conception.md) · [table_modbus.md](table_modbus.md).

---

## 1. Pré-requisitos

- **Rust stable** (edição 2021, `rust-version` ≥ 1.85). Instalação: <https://rustup.rs>.
- **Dependências de sistema (Linux) para a IHM** (`eframe`/`egui`, OpenGL/winit):
  - Debian/Ubuntu: `libxkbcommon-dev`, `libwayland-dev`, `libxcb1-dev`,
    `libgl1-mesa-dev` (ou equivalentes), mais um servidor gráfico (X11/Wayland).
  - A IHM necessita de um **ecrã**: em ambiente headless, a janela não
    se abre (o servidor Modbus, esse, não depende do ecrã).
- Acesso de rede ao registo crates.io para a primeira compilação.

---

## 2. Comandos correntes

```bash
cargo check --workspace          # Verificação rápida (sem codegen)
cargo build --workspace          # Compilação debug
cargo build --release            # Compilação otimizada (LTO thin)
cargo test  --workspace          # Testes unitários + integração
cargo clippy --workspace --all-targets   # Lint (deve permanecer SEM aviso)
cargo run -p mock_bin_ru_modbustcp       # Lança o regulador

# Ficheiro de configuração alternativo:
MOCK_CONFIG=./ma_config.toml cargo run -p mock_bin_ru_modbustcp
# Registo detalhado:
RUST_LOG=debug cargo run -p mock_bin_ru_modbustcp
```

Binário produzido: `target/debug/orme` ou `target/release/orme` (o pacote Cargo
continua a ser `mock_bin_ru_modbustcp`, mas o executável chama-se **`orme`** — ver
`[[bin]]` no `Cargo.toml` do crate).

### Features Cargo

| Feature | Por omissão | Efeito |
|---------|:---------:|-------|
| `gui` | ✅ | IHM `egui`/`eframe` (caso contrário binário headless) |
| `rtu` | ✅ | Transporte Modbus RTU série (RS485) via `tokio-serial` |

```bash
cargo build --no-default-features                 # headless, Modbus TCP apenas
cargo build --no-default-features --features rtu  # headless TCP + RTU série
cargo build --no-default-features --features gui  # IHM, TCP apenas (sem série)
```

> ⚠️ **`rtu` = dependência nativa.** `tokio-serial` abre a porta via termios
> (Linux); a enumeração `libudev` está desativada (`default-features = false`).
> Em **cross-compilação** (`build-prod.sh`, executáveis desktop com features por
> omissão), a imagem `cross` do target pode mesmo assim exigir os cabeçalhos série
> do sistema; se a cadeia colocar problemas, retirar `rtu` do build em causa. O
> **Docker headless não é afetado** (compila em `--no-default-features`).

---

## 3. Organização do código

```
mock_lib_control/        Biblioteca de regulação (pura, sem IO, testável)
  src/pid.rs             PID anti-saturação
  src/onoff.rs           Tudo-ou-nada com histerese simétrica + anti-ciclo-curto
  src/pwm.rs             Relé de ciclo (PWM / time-proportioning)
  src/process.rs         Função de transferência FOPDT
  src/lib.rs             ControllerKind + reexportações (feature `serde` opcional)

mock_bin_ru_modbustcp/   Binário regulador
  src/main.rs            Arranque: config, runtime Tokio, atores, IHM
  src/regulator.rs       Modelo de negócio síncrono (estado, Command, step)
  src/config.rs          AppConfig (TOML), Transport/SerialConfig, IpFilter, ServerStatus
  src/map.rs             Plano de endereçamento Modbus (FONTE DE VERDADE)
  src/modbus_server.rs   RegulatorService (trait Service) + mono-mestre TCP + serve_rtu
  src/gui.rs             IHM egui (página única + modal Parâmetros)
  src/actors/
    simulation.rs        Laço de regulação (tick)
    network.rs           Servidor Modbus TCP/RTU (re)configurável a quente

docs/                    Conceção, tabela Modbus, manutenção
```

**Regra de ouro**: a lógica de negócio (`mock_lib_control`, `regulator.rs`) permanece
**síncrona e testada**; o assíncrono fica confinado aos atores e à IO.

---

## 4. Configuração

- Ficheiro: `mock_ru_modbustcp.toml` no diretório corrente, ou caminho
  fornecido pela variável de ambiente `MOCK_CONFIG`.
- Carregado no arranque; **valores predefinidos** se ausente ou ilegível (um
  aviso é registado, a aplicação arranca mesmo assim).
- Guardado a partir da IHM (botões *Aplicar* / *Guardar regulações* /
  *Repor predefinições*).

Estrutura (todas as secções são opcionais, completadas por omissão):

```toml
[network]
bind_ip = "0.0.0.0"
port = 5502
allowlist = ["192.168.1.*", "127.0.0.1"]   # vazio = todas as IP autorizadas

[process]   # função de transferência G(s) = K·e^(-L·s)/(1+T·s)
gain = 1.6        # K (unidade/%)
tau = 30.0        # T (s)
dead_time = 2.0   # L (s)
ambient = 20.0

[regulation]
sp_min = 0.0
sp_max = 250.0
hysteresis = 2.0
[regulation.pid_heat]   # sentido 1 (aquecimento)
kp = 4.0 ; ki = 0.25 ; kd = 1.0 ; out_min = 0.0 ; out_max = 100.0
[regulation.pid_cool]   # sentido 2 (arrefecimento)
kp = 4.0 ; ki = 0.25 ; kd = 1.0 ; out_min = 0.0 ; out_max = 100.0
```

> Os **valores predefinidos** têm uma **fonte única**: `RegulatorConfig::default`
> em `regulator.rs`. `ProcessConfig`/`RegulationConfig` (config.rs) derivam dela.
> Para alterar um valor predefinido, modificar apenas `RegulatorConfig::default`.

---

## 5. Dependências e armadilhas de versão

| Crate | Papel | Ponto de atenção |
|-------|------|-------------------|
| `tokio` | runtime async | features: `rt-multi-thread, macros, net, time, sync` |
| `ractor` | atores | features por omissão (async nativo, **não** `async-trait`) |
| `tokio-serial` | Modbus RTU série | opcional (feature `rtu`), `default-features = false` (sem enumeração libudev) |
| `tokio-modbus` | Modbus TCP | `default-features = false`, feature **`tcp-server`** |
| `eframe`/`egui` | IHM | versões ligadas entre si |
| `egui_plot` | curva | ⚠️ **versionado uma minor à frente de `egui`**: para `egui` 0.33 → `egui_plot` **0.34** |
| `serde`/`toml` | persistência | `mock_lib_control` expõe uma feature `serde` ativada pelo binário |

As versões partilhadas estão centralizadas em `[workspace.dependencies]` do
`Cargo.toml` raiz. Para subir `egui`/`eframe`, **verificar a versão
correspondente de `egui_plot`** (caso contrário erro «two versions of crate egui»).

---

## 6. Estender o projeto

### 6.1 Adicionar um ponto Modbus

Tudo se passa em **`map.rs`** (depois o snapshot/Command se necessário):

1. Declarar a constante de endereço e ajustar o `*_COUNT` da tabela em causa.
2. Preencher o valor em `MemoryMap::refresh_from` (estado → registo).
3. Se o ponto for inscritível, descodificá-lo em `coil_to_command` /
   `holdings_to_commands` (registo → `Command`).
4. Atualizar o comentário de documentação do cabeçalho **e** [table_modbus.md](table_modbus.md).
5. Adicionar a linha na tabela ao vivo da IHM (`gui.rs::modbus_rows`).

### 6.2 Adicionar um comando / uma regulação

1. Variante em `enum Command` (`regulator.rs`) + tratamento em `Regulator::apply`.
2. Campo em `RegulatorSnapshot` se o valor deve ser observável.
3. Cablagem IHM (`gui.rs`) e/ou descodificação Modbus (`map.rs`).
4. Se persistente: campo em `AppConfig` (`config.rs`) + `to_regulator_config`.

### 6.3 Adicionar um novo instrumento

1. Criar `mock_bin_<nome>/` e adicioná-lo aos `members` do `Cargo.toml` raiz.
2. Reutilizar `mock_lib_control`; fatorizar todo o comum numa `mock_lib_*`.
3. Seguir a mesma divisão: modelo síncrono, ator(es) ractor, camada
   protocolo, IHM. Convenção de nome: `mock_bin_<tipo>_<protocolo>`.

---

## 7. Estratégia de teste

- **Unitários** (`mock_lib_control`): PID (proporcional, limitação, anti-windup),
  TOR (zona morta), processo (convergência em regime estabelecido).
- **Domínio** (`regulator.rs`): convergência PID em auto, saída em manual,
  retorno ao ambiente em paragem.
- **Mapeamento** (`map.rs`): round-trip `f32`↔registos, descodificação de escrita,
  rejeição de escrita `f32` parcial.
- **Config / rede** (`config.rs`, `actors/network.rs`): round-trip TOML, filtro
  IP (curinga), arranque efetivo do servidor (bind em porta efémera).

Lançar: `cargo test --workspace`. Os testes são **deterministas e sem IHM**.

---

## 8. Resolução de problemas

| Sintoma | Pista |
|----------|-------|
| «two versions of crate `egui`» | Desacordo `egui_plot` / `egui`: alinhar as versões (§5). |
| A IHM não abre | Ecrã ausente (headless) ou bibliotecas de sistema em falta (§1). |
| `Modbus ✖ falha na escuta` no cabeçalho | Porta já em uso ou < 1024 sem privilégios: mudar a porta em *Parâmetros*. |
| Um cliente é recusado | IP fora da **lista branca**: esvaziar a lista ou adicionar um padrão (`192.168.1.*`). |
| Valores `f32` aberrantes do lado do cliente | Ordem das palavras (palavra de maior peso primeiro): ver [table_modbus.md](table_modbus.md). |
| Uma escrita de consigna `f32` é ignorada | Escrever **os dois** registos do par num só pedido. |
| Config não recarregada | Diretório corrente errado ou `MOCK_CONFIG`; verificar o registo no arranque. |
| Sem ícone na barra de tarefas (Linux) | Sessão **Wayland**: o ícone embutido é ignorado. Instalar a entrada de ambiente de trabalho: `scripts/install-desktop.sh` (§9). |

Aumentar a verbosidade: `RUST_LOG=debug` (ou `trace`).

---

## 9. Build de distribuição

```bash
cargo build --release
# Binário autónomo:
target/release/orme
```

O perfil `release` ativa `lto = "thin"` e `opt-level = 3` (ver `Cargo.toml`
raiz). Para distribuir: fornecer o binário + um `mock_ru_modbustcp.toml`
de exemplo. Licença **MIT** (ficheiro `LICENSE`).

### Feature `gui` (build com / sem interface)

A IHM está atrás da feature Cargo **`gui`**, ativada por omissão:

```bash
cargo build --release                       # com IHM (posto de trabalho)
cargo build --release --no-default-features  # «headless»: Modbus + simulação, sem IHM
```

O modo **headless** destina-se aos despliegues sem ecrã (Raspberry Pi em
serviço) e torna a **cross-compilação ARM trivial** (nenhuma dependência
gráfica a ligar).

### Integração no ambiente de trabalho Linux (ícone da barra de tarefas)

O ícone ORME está embutido no binário (`branding.rs` → `with_icon`). Isto basta
sob **X11, Windows e macOS**. Mas sob **Wayland**, o compositor **ignora** o
ícone embutido: associa a janela ao seu **`app_id`** («orme», definido em
`main.rs` via `ViewportBuilder::with_app_id`) a um ficheiro `orme.desktop` com o
mesmo nome, e mostra o `Icon=` desse ficheiro (resolvido no tema de ícones
`hicolor`).

Para obter o ícone sob Wayland, instalar a entrada de ambiente de trabalho para
o utilizador corrente:

```bash
scripts/install-desktop.sh
```

O script copia:

| Origem | Destino |
|--------|---------|
| `pic/orme-icon.png` | `~/.local/share/icons/hicolor/256x256/apps/orme.png` |
| `packaging/orme.desktop` | `~/.local/share/applications/orme.desktop` |

depois atualiza as caches (`gtk-update-icon-cache`, `update-desktop-database`). O
ícone aparece no próximo arranque do ORME (e de forma fiável após uma reentrada
na sessão Wayland).

> ⚠️ Três nomes **devem permanecer alinhados**: o `app_id` (`main.rs`), o nome do
> ficheiro `orme.desktop` e o seu `StartupWMClass`, e o nome do ícone `orme.png`
> (= `Icon=orme`). `packaging/orme.desktop` pressupõe um executável `orme` no
> `PATH` (campo `Exec=`); em dev (`cargo run`) este campo não tem incidência na
> apresentação do ícone.

---

## 10. Build «prod» — cross-compilação a partir de Linux

### Procedimento único

Tudo é produzido **a partir de Linux** por
[`scripts/build-prod.sh`](../../../scripts/build-prod.sh):

| Saída | Alvo | IHM | Método |
|--------|-------|-----|---------|
| `dist/…-linux-x86_64` | `x86_64-unknown-linux-gnu` | ✅ | `cross` |
| `dist/…-windows-x86_64.exe` | `x86_64-pc-windows-gnu` | ✅ | `cross` (mingw) |
| `dist/…-rpi-arm64` | `aarch64-unknown-linux-gnu` (Pi 3/4/5, Pi OS 64b) | ✅ | `cross` |
| Imagem Docker headless | multi-arch `linux/amd64` + `linux/arm64` | ❌ | `docker buildx` |

```bash
# Pré-requisitos (uma vez) — o Docker deve estar a correr:
cargo install cross

# Produzir tudo (executáveis em dist/ + imagem Docker local amd64 carregada):
scripts/build-prod.sh

# Variante: imagem Docker MULTI-ARCH enviada para um registo:
IMAGE=ghcr.io/<conta>/orme:latest scripts/build-prod.sh
```

### Porquê `cross` para TODOS os builds (incluindo Linux x86_64)

`cross` fornece imagens Docker contendo as toolchains de cada alvo: nem
`mingw-w64`, nem toolchain ARM, nem *sysroot* a instalar.

⚠️ **Não misturar `cargo` nativo e `cross` no mesmo `target/`.** Ambos
utilizam versões de `rustc` diferentes (anfitrião vs contentor); as
**proc-macros** compiladas por um são rejeitadas pelo outro, daí erros
`can't find crate for …_derive` (ex. `zerofrom_derive`, `tracing_attributes`).
O script passa, portanto, **sempre por `cross`**, mesmo para Linux x86_64 — uma só
toolchain, builds reproduzíveis. (Se o erro ocorrer mesmo assim após um
build nativo anterior: `rm -rf target/release` e depois relançar.)

### IHM cross-compilada para ARM: porque funciona

`eframe`/`egui` carregam OpenGL, X11/Wayland e xkbcommon **em tempo de execução**
(`dlopen`): o binário só liga, no build, à `libc`. Nenhuma biblioteca gráfica ARM
é, portanto, necessária do lado cross. No Raspberry Pi, prever um ambiente
de secretária (mesa/X11 ou Wayland) — presente no Raspberry Pi OS *Desktop*.

> Para um **Raspbian 32 bits**, visar `armv7-unknown-linux-gnueabihf` (adaptar
> os alvos no script).

### Imagem Docker headless «em qualquer lado»

A imagem ([`docker/Dockerfile.headless`](../../../docker/Dockerfile.headless)) parte de
`debian:bookworm-slim` e **copia** o binário headless da arquitetura desejada
(nenhuma compilação na imagem → sem QEMU). `docker buildx` monta o
multi-arch `amd64`+`arm64`. O servidor escuta em `5502`. Montar um volume em
`/data` para fornecer/persistir `mock_ru_modbustcp.toml`.

```bash
# Sem registo: imagem local amd64 carregada, testável imediatamente
docker run --rm -p 5502:5502 -v "$PWD/conf:/data" orme:headless
```

### Build nativo Windows (MSVC) — opcional

O `.exe` produzido acima é **GNU/mingw** (executável Windows nativo, IHM
incluída). Se um binário **MSVC** for necessário, compilar numa máquina Windows
com [`scripts/build-windows.ps1`](../../../scripts/build-windows.ps1) (pré-requisitos:
Rust + *Visual Studio Build Tools*, carga «Desenvolvimento Desktop em C++»), ou
a partir de Linux via `cargo-xwin` (`cargo xwin build --release --target x86_64-pc-windows-msvc`).

### Notas

- Os binários estão **dinamicamente ligados à glibc**; compilados via `cross`
  (baseline glibc antiga) correm em distribuições recentes (e em
  `debian:bookworm-slim`). Para um binário totalmente estático, visar `*-musl`.
- `dist/` é ignorado pelo git (artefactos de build).

---

## 11. Convenções

- Código e comentários em **francês**.
- `cargo clippy --workspace` **sem aviso** antes de qualquer commit.
- Todo o novo comportamento de negócio ou de mapeamento acompanha-se de um **teste**.
- O plano de endereçamento modifica-se em **`map.rs`** (fonte de verdade), com atualização
  conjunta da documentação.
