# Documentação de manutenção — OSNE (workspace `cesam-tools`)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · **PT** · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

> Público: programadores que mantêm, corrigem ou estendem o projeto.
> Ver também: [conception.md](conception.md) · [commandes_namur.md](commandes_namur.md).

---

## 1. Pré-requisitos

- **Rust stable** (edição 2021, `rust-version` ≥ 1.85). Instalação: <https://rustup.rs>.
- **Dependências do sistema (Linux) para a IHM** (`eframe`/`egui`, OpenGL/winit):
  - Debian/Ubuntu: `libxkbcommon-dev`, `libwayland-dev`, `libxcb1-dev`,
    `libgl1-mesa-dev` (ou equivalentes), além de um servidor gráfico (X11/Wayland).
  - A IHM requer um **ecrã**: em ambiente headless, a janela não se abre (o servidor
    NAMUR, esse, não depende do ecrã).
- **Ligação série** (feature `serial`): acesso à porta (`/dev/ttyUSB*`, grupo
  `dialout` no Linux). Sem hardware, usar o transporte **TCP**.
- Acesso à rede ao registo crates.io para a primeira compilação.

---

## 2. Comandos correntes

```bash
cargo check -p mock_bin_su_namur          # Verificação rápida (sem codegen)
cargo build -p mock_bin_su_namur          # Compilação debug
cargo build --release -p mock_bin_su_namur   # Compilação otimizada (LTO thin)
cargo test  -p mock_bin_su_namur          # Testes unitários + integração
cargo clippy --workspace --all-targets    # Lint (deve permanecer SEM aviso)
cargo run   -p mock_bin_su_namur          # Lança o agitador (IHM + NAMUR/TCP)

# Ficheiro de configuração alternativo:
MOCK_CONFIG=./ma_config.toml cargo run -p mock_bin_su_namur
# Registo detalhado:
RUST_LOG=debug cargo run -p mock_bin_su_namur
```

Binário produzido: `target/debug/osne` ou `target/release/osne` (o pacote Cargo
mantém-se `mock_bin_su_namur`, mas o executável chama-se **`osne`** — ver `[[bin]]`
no `Cargo.toml` do crate).

### Features Cargo

| Feature | Por defeito | Efeito |
|---------|:---------:|-------|
| `gui` | ✅ | IHM `egui`/`eframe` (caso contrário, binário headless) |
| `serial` | ✅ | Transporte NAMUR sobre ligação série RS-232 via `tokio-serial` |

```bash
cargo build -p mock_bin_su_namur --no-default-features                  # headless, NAMUR/TCP apenas
cargo build -p mock_bin_su_namur --no-default-features --features serial # headless TCP + série
cargo build -p mock_bin_su_namur --no-default-features --features gui    # IHM, TCP apenas (sem série)
```

> ⚠️ **`serial` = dependência nativa.** `tokio-serial` abre a porta via termios
> (Linux); a enumeração `libudev` está desativada (`default-features = false`). Em
> **cross-compilação** (`build-prod.sh`, exes desktop com features por defeito), a
> imagem `cross` do target pode ainda assim exigir os cabeçalhos série; se a
> toolchain criar problemas, retirar `serial` do build em causa. O **Docker headless
> não é afetado** (compila em `--no-default-features`).

---

## 3. Organização do código

```
mock_lib_control/        Biblioteca de regulação (pura, sem IO, testável)
  src/pid.rs             PID anti-windup (reutilizado para o asservimento de velocidade)
  src/lib.rs             re-exportações (feature `serde` opcional)

mock_bin_su_namur/       Binário agitador (executável `osne`)
  src/main.rs            Arranque : config, runtime Tokio, atores, IHM
  src/motor.rs           Modelo físico do motor (dinâmica rotacional, Euler)
  src/stirrer.rs         Modelo de negócio síncrono (estado, Command, step) — possui o PID
  src/config.rs          AppConfig (TOML), Transport/SerialConfig, IpFilter, ServerStatus
  src/namur.rs           Protocolo NAMUR : handle_line (FONTE DE VERDADE do conjunto de comandos)
  src/namur_server.rs    Serviço NAMUR (linhas ASCII) + mono-mestre TCP + serviço série + cão de guarda
  src/trace.rs           Registo circular das tramas (mini-terminal IHM)
  src/gui.rs             IHM egui (página única + mini-terminal + modal Definições)
  src/branding.rs        Logos embutidos (feature `gui`)
  src/i18n.rs            Catálogo i18n tipado (8 línguas), sem dependência
  src/actors/
    simulation.rs        Ciclo de simulação (tick 20 ms)
    network.rs           Servidor NAMUR TCP/série (re)configurável a quente

docs/                    Conceção, comandos NAMUR, manual, manutenção (multilíngue)
```

**Regra de ouro**: a lógica de negócio (`mock_lib_control`, `motor.rs`,
`stirrer.rs`) mantém-se **síncrona e testada**; o assíncrono fica confinado aos
atores e à IO. Decalque exato do regulador **ORME** (`mock_bin_ru_modbustcp`) —
mesmos invariantes.

---

## 4. Configuração

- Ficheiro: `mock_su_namur.toml` no diretório atual, ou caminho fornecido pela
  variável de ambiente `MOCK_CONFIG`.
- Carregado no arranque; **valores por defeito** se ausente ou ilegível (um aviso é
  registado, a aplicação arranca à mesma).
- **Qualquer valor proveniente do TOML é higienizado** (`AppConfig::sanitized`):
  limites reordenados (`min ≤ max`), flutuantes forçados a finitos,
  inércia/binário/viscosidade estritamente positivos. **Invariante: nunca fazer
  `f32::clamp` com limites não validados** (entra em pânico se `min > max` ou `NaN`).
- Guardado a partir da IHM (botões *Aplicar* / *Guardar* / *Repor*).

Estrutura (todas as secções são opcionais, completadas por defeito):

```toml
language = "pt"
check_updates = true       # verificar no arranque se existe uma release mais recente (IHM)

[network]
transport = "tcp"          # "tcp" ou "serial"
bind_ip = "0.0.0.0"
port = 4001
allowlist = ["192.168.1.*", "127.0.0.1"]   # vazio = todos os IP autorizados
[network.serial]
port = "/dev/ttyUSB0"
baud = 9600 ; parity = "even" ; data_bits = 7 ; stop_bits = 1   # NAMUR 7E1

[motor]   # J·dω/dt = T − k·η·ω − atrito
inertia = 0.02      # J (reatividade)
load_coeff = 0.05   # k (peso da viscosidade)
friction = 2.0      # N·cm
torque_max = 100.0  # N·cm (limite máximo da saída PID)

[regulation]
speed_min = 0.0 ; speed_max = 2000.0
viscosity = 1.0 ; viscosity_min = 0.1 ; viscosity_max = 20.0
[regulation.pid]
kp = ... ; ki = ... ; kd = ... ; out_min = 0.0 ; out_max = 100.0
```

> Os **valores por defeito** têm uma **fonte única**: `StirrerConfig::default` em
> `stirrer.rs`. `MotorConfig`/`RegulationConfig` (config.rs) derivam dela. Os limites
> de saída do PID (`out_min`/`out_max`) são **forçados** a `[0, couple_max]` no
> momento de construir o agitador (`to_stirrer_config`).

### Verificação de atualização

Se `check_updates = true` (predefinição) **e** o binário for compilado com a
feature `gui`, a IHM consulta **no arranque** a última release publicada no
GitHub (`CESAMLAB/cesam-tools`) e compara o seu número com a versão atual. Uma
versão mais recente mostra um banner clicável «🔔 Atualização disponível». O
botão *Verificar agora* (modal *Definições*) reinicia a verificação.

- O pedido HTTPS executa-se numa **thread dedicada**, limitada por um timeout
  (5 s): offline ou GitHub inacessível nunca impede o arranque.
- A lógica reside na crate partilhada **`mock_lib_update`** (`ureq`/`rustls`,
  raízes Mozilla embutidas → cross-compilação limpa sob `cross`).
- **Build headless** (`--no-default-features`): a verificação — e toda a
  dependência rede/TLS — está **ausente**. No servidor, gerir as atualizações
  via apt/Docker. Desativável pelo operador (caixa de seleção do modal).

---

## 5. Dependências e armadilhas de versão

| Crate | Papel | Ponto de atenção |
|-------|------|-------------------|
| `tokio` | runtime async | features partilhadas + **`io-util`** (BufReader / linhas ASCII NAMUR) |
| `ractor` | atores | features por defeito (async nativo, **não** `async-trait`) |
| `tokio-serial` | NAMUR série | opcional (feature `serial`), `default-features = false` (sem enumeração libudev) |
| `eframe`/`egui` | IHM | versões ligadas entre si |
| `egui_plot` | curva | ⚠️ **versionado uma minor à frente do `egui`**: para `egui` 0.33 → `egui_plot` **0.34** |
| `serde`/`toml` | persistência | `mock_lib_control` expõe uma feature `serde` ativada pelo binário |
| `mock_lib_update` (`ureq`/`rustls`) | verif. de atualização | **feature `gui` apenas**; rustls 0.23 (webpki atualizado); ausente em headless |

As versões partilhadas estão centralizadas em `[workspace.dependencies]` do
`Cargo.toml` raiz. Para subir `egui`/`eframe`, **verificar a versão correspondente
de `egui_plot`** (caso contrário, erro «two versions of crate egui»).

---

## 6. Estender o projeto

### 6.1 Adicionar um comando NAMUR

Tudo se passa em **`namur.rs`** (fonte de verdade do protocolo):

1. Adicionar o ramo em `handle_line` (leitura → `Reply`, escrita/ação →
   `Apply(Command)` ou `SetWatchdog`).
2. Se for uma **ação**, adicionar a variante em `enum Command` (`stirrer.rs`) e o
   seu tratamento em `Stirrer::apply`.
3. Atualizar o doc-comentário de cabeçalho, **[commandes_namur.md](commandes_namur.md)**
   e a tabela de referência do mini-terminal (`gui.rs`, tabela `rows`).
4. Adicionar um teste no módulo `tests` de `namur.rs`.

### 6.2 Adicionar um comando / um ajuste IHM

1. Variante em `enum Command` (`stirrer.rs`) + tratamento em `Stirrer::apply`.
2. Campo em `StirrerSnapshot` se o valor deve ser observável.
3. Ligação IHM (`gui.rs`) via um `cast` não bloqueante.
4. Se persistente: campo em `AppConfig` (`config.rs`) + higienização em `sanitized`
   + repercussão em `to_stirrer_config`.

### 6.3 Adicionar uma cadeia de interface (i18n)

Qualquer cadeia da IHM **deve** passar por uma chave `Msg` (`i18n.rs`) com as suas
**8 traduções** (tabela de tamanho fixo verificada na compilação). Os acrónimos
NAMUR, sufixos de unidade e nomes de comandos mantêm-se codificados de forma fixa.

### 6.4 Adicionar um novo instrumento

1. Criar `mock_bin_<nom>/` e adicioná-lo aos `members` do `Cargo.toml` raiz.
2. Reutilizar `mock_lib_control`; fatorizar tudo o que for comum numa `mock_lib_*`
   (ex. promoção do modelo `motor.rs` se servir um segundo instrumento).
3. Seguir a mesma divisão: modelo síncrono, ator(es) ractor, camada de protocolo,
   IHM. Convenção de nome: `mock_bin_<type>_<protocole>`.

---

## 7. Estratégia de teste

- **Unitários** (`mock_lib_control`): PID (proporcional, limitação, anti-windup).
- **Motor** (`motor.rs`): dinâmica rotacional, convergência em regime estabelecido,
  efeito da viscosidade no binário, saturação/sobrecarga.
- **Domínio** (`stirrer.rs`): convergência da velocidade até à consigna,
  desaceleração na paragem, deteção de sobrecarga.
- **Protocolo** (`namur.rs`): descodificação das leituras (`IN_*`), das escritas
  (`OUT_SP_4`), das ações (`START/STOP/RESET`), do cão de guarda e dos comandos
  desconhecidos.
- **Config / rede** (`config.rs`, `actors/network.rs`): round-trip TOML, filtro IP
  (carateres universais, IPv4-mapped), higienização sem pânico, abertura série em
  erro com porta ausente.

Lançar: `cargo test -p mock_bin_su_namur` (ou `--workspace`). Os testes são
**determinísticos e sem IHM**.

---

## 8. Resolução de problemas

| Sintoma | Pista |
|----------|-------|
| «two versions of crate `egui`» | Desacordo `egui_plot` / `egui`: alinhar as versões (§5). |
| A IHM não abre | Ecrã ausente (headless) ou libs do sistema em falta (§1). |
| `NAMUR ✖` no cabeçalho | Porta TCP já utilizada / < 1024 sem privilégios, ou porta série indisponível: alterar em *Parâmetros*. |
| Um cliente TCP é recusado | IP fora da **lista branca**: esvaziar a lista ou adicionar um padrão (`192.168.1.*`). |
| A série não abre | Feature `serial` ausente, porta errada, ou permissões (`dialout`). |
| O motor para sozinho | **Cão de guarda** armado (`OUT_WD1@…`) sem tráfego: enviar tramas ou `OUT_WD1@0`. |
| Sobrecarga permanente | Viscosidade demasiado elevada vs `torque_max`: ajustar os parâmetros do motor. |
| Config não recarregada | Diretório atual errado ou `MOCK_CONFIG`; verificar o registo no arranque. |

Aumentar a verbosidade: `RUST_LOG=debug` (ou `trace`).

---

## 9. Build de distribuição

```bash
cargo build --release -p mock_bin_su_namur
# Binário autónomo:
target/release/osne
```

O perfil `release` ativa `lto = "thin"` e `opt-level = 3` (ver `Cargo.toml` raiz).
Para distribuir: fornecer o binário + um `mock_su_namur.toml` de exemplo. Licença
**MIT** (ficheiro `LICENSE`).

### Feature `gui` (build com / sem interface)

```bash
cargo build --release -p mock_bin_su_namur                       # com IHM (posto de trabalho)
cargo build --release -p mock_bin_su_namur --no-default-features  # «headless»: NAMUR + simulação, sem IHM
```

O modo **headless** destina-se às implementações sem ecrã e torna a
**cross-compilação ARM trivial** (nenhuma dependência gráfica a ligar).

### Integração no ambiente de trabalho Linux (ícone da barra de tarefas)

O ícone OSNE (`pic/osne-icon.png`, motivo de agitador, gerado por
[`pic/osne-logo.gen.py`](../../../pic/osne-logo.gen.py)) está **embebido** no
binário (`branding.rs` → `window_icon`). Isto é suficiente sob **X11, Windows e
macOS**. Sob **Wayland**, o compositor **ignora** o ícone embebido: associa a janela
ao seu **`app_id`** («osne», definido em `main.rs` via `with_app_id`) a um ficheiro
`osne.desktop` com o mesmo nome, e mostra o `Icon=osne` resolvido no tema de ícones
`hicolor`.

Para obter o ícone sob Wayland, instalar a entrada de ambiente de trabalho para o
utilizador atual:

```bash
scripts/install-desktop.sh osne
```

O script copia:

| Origem | Destino |
|--------|-------------|
| `pic/osne-icon.png` | `~/.local/share/icons/hicolor/256x256/apps/osne.png` |
| `packaging/osne.desktop` | `~/.local/share/applications/osne.desktop` |

e em seguida atualiza as caches. Três nomes **devem permanecer alinhados**: o
`app_id` (`main.rs`), o ficheiro `osne.desktop` (+ o seu `StartupWMClass`) e o ícone
`osne.png` (= `Icon=osne`). O mesmo script instala o ORME sem argumento
(`scripts/install-desktop.sh`).

---

## 10. Build «prod» — cross-compilação a partir do Linux

### Procedimento único

Tudo é produzido **a partir do Linux** por
[`scripts/build-prod.sh`](../../../scripts/build-prod.sh), que constrói **todos os
instrumentos do workspace** (ORME *e* OSNE):

| Saída | Alvo | IHM | Método |
|--------|-------|-----|---------|
| `dist/osne-linux-x86_64` | `x86_64-unknown-linux-gnu` | ✅ | `cross` |
| `dist/osne-windows-x86_64.exe` | `x86_64-pc-windows-gnu` | ✅ | `cross` (mingw) |
| `dist/osne-rpi-arm64` | `aarch64-unknown-linux-gnu` (Pi 3/4/5, Pi OS 64b) | ✅ | `cross` |
| Imagem Docker headless `osne:headless` | multi-arch `amd64` + `arm64` | ❌ | `docker buildx` |
| `dist/osne_<ver>_amd64.deb` / `_arm64.deb` | pacote Debian/Ubuntu | ✅ | `dpkg-deb` |
| `dist/osne-setup-x86_64.exe` | instalador Windows | ✅ | NSIS (`makensis`) |

```bash
# Pré-requisitos (uma vez) — o Docker deve estar a correr:
cargo install cross

# Produzir tudo (executáveis ORME + OSNE + instaladores em dist/ + imagens Docker amd64):
scripts/build-prod.sh

# Variante: imagens Docker MULTI-ARCH enviadas para um registo:
IMAGE_PREFIX=ghcr.io/<conta> scripts/build-prod.sh

# Sem construir os instaladores:
INSTALLERS=0 scripts/build-prod.sh
```

### Porquê `cross` para TODOS os builds (incluindo Linux x86_64)

`cross` fornece imagens Docker contendo as toolchains de cada alvo. ⚠️ **Não
misturar `cargo` nativo e `cross` no mesmo `target/`.** As **proc-macros**
compiladas por um são rejeitadas pelo outro (`can't find crate for …_derive`). O
script passa **sempre por `cross`**. (Se o erro surgir: `rm -rf target/release` e
relançar.)

### IHM cross-compilada para ARM: porque funciona

`eframe`/`egui` carregam OpenGL, X11/Wayland e xkbcommon **em tempo de execução**
(`dlopen`): o binário só liga ao build a `libc`. Nenhuma lib gráfica ARM é
necessária no lado cross; prever um ambiente de trabalho no alvo.

### Imagem Docker headless

A imagem ([`docker/Dockerfile.headless`](../../../docker/Dockerfile.headless)) parte
de `debian:bookworm-slim` e **copia** o binário headless da arquitetura pretendida
(nenhuma compilação na imagem → sem QEMU). O nome do binário e a porta exposta são
passados por `--build-arg` (`BIN=osne`, `PORT=4001`). Montar um volume em `/data`
para fornecer/persistir `mock_su_namur.toml`.

```bash
docker run --rm -p 4001:4001 -v "$PWD/conf:/data" osne:headless
```

### Instaladores (`.deb` Linux/RPi + setup Windows)

No fim de cada build, `build-prod.sh` chama
[`scripts/make-installers.sh <bin>`](../../../scripts/make-installers.sh), que
transforma os executáveis release de `dist/` em **instaladores**:

| Instalador | Origem | Conteúdo | Ferramenta |
|------------|--------|----------|------------|
| `osne_<ver>_amd64.deb` | `dist/osne-linux-x86_64` | binário → `/usr/bin`, entrada de secretária, ícone hicolor | `dpkg-deb` |
| `osne_<ver>_arm64.deb` | `dist/osne-rpi-arm64` | idem (Raspberry Pi OS 64 bits) | `dpkg-deb` |
| `osne-setup-x86_64.exe` | `dist/osne-windows-x86_64.exe` | exe + atalhos (menu Iniciar/secretária) + desinstalador | NSIS (`makensis`) |

- Os `.deb` instalam o ícone e o `.desktop`; um `postinst` atualiza as caches
  (`update-desktop-database`, `gtk-update-icon-cache`). Dependências: `libc6`;
  recomendações gráficas (`libgl1`, `libxkbcommon0`, `libwayland-client0`).
- O instalador Windows é gerado a partir de
  [`packaging/windows/installer.nsi`](../../../packaging/windows/installer.nsi);
  os atalhos usam um ícone `.ico` multi-resolução derivado de
  `pic/osne-icon.png` (via Pillow).
- **Pré-requisitos**: `dpkg-deb` (presente em Debian/Ubuntu) para os `.deb`,
  **`makensis`** (`sudo apt install nsis`) para o setup Windows, `python3`+Pillow
  para o `.ico`. Cada alvo cuja ferramenta ou artefacto falte é **avisado e
  saltado** (o build não quebra). Desativar via `INSTALLERS=0`. Pode também
  (re)gerar apenas os instaladores de um instrumento:
  `scripts/make-installers.sh osne`.
- A **versão** dos pacotes vem de `[workspace.package].version` do `Cargo.toml`
  raiz.

### Notas

- Os binários estão **dinamicamente ligados à glibc**; compilados via `cross`
  (baseline glibc antiga) correm em distribuições recentes.
- `dist/` é ignorado pelo git (artefactos de build).

---

## 11. Convenções

- Código e comentários em **francês**; logs e mensagens de erro em **inglês**.
- `cargo clippy --workspace` **sem avisos** antes de qualquer commit.
- Qualquer novo comportamento de negócio, de motor ou de protocolo acompanha-se de
  um **teste**.
- O conjunto de comandos NAMUR modifica-se em **`namur.rs`** (fonte de verdade), com
  atualização conjunta da documentação.
