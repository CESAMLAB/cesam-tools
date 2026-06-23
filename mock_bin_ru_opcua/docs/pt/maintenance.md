# Documentação de manutenção — RU/OPC UA (workspace `cesam-tools`)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · **PT** · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

> Crate: `mock_bin_ru_opcua` · Executável: **ru_opcua**

---

## 1. Pré-requisitos

- **Rust** recente. ⚠️ MSRV próprio deste crate: **1.91** (`async-opcua` não declara
  nenhum `rust-version` e puxa dependências recentes; o resto do workspace
  está em 1.85).
- Para a IHM: as dependências de sistema do `eframe`/`egui` (as mesmas que ORME/OSNE).
- Para o build *headless*: nenhuma dependência gráfica.

---

## 2. Comandos correntes

```bash
cargo run -p mock_bin_ru_opcua                       # IHM + servidor OPC UA
cargo run -p mock_bin_ru_opcua --no-default-features # headless (sem IHM)
cargo test -p mock_bin_ru_opcua                      # testes unitários
cargo clippy -p mock_bin_ru_opcua --all-targets      # lint
MOCK_CONFIG=./ma_config.toml cargo run -p mock_bin_ru_opcua  # config alternativa
```

### Features Cargo

- **`gui`** (predefinição): interface gráfica `egui` + verificação de atualização.
- `--no-default-features`: binário **headless** (servidor OPC UA + simulação,
  sem IHM nem rede de atualização).

O servidor `async-opcua` está **sempre** presente (a feature `server` do
`async-opcua`), pois é a razão de ser do instrumento.

---

## 3. Organização do código

```
mock_bin_ru_opcua/src/
├── main.rs            # Monta runtime Tokio + atores + IHM/headless
├── regulator.rs       # Modelo de negócio síncrono (PID + processo), comandos, passo
├── config.rs          # AppConfig (TOML), sanitized(), ServerStatus
├── i18n.rs            # Catálogo i18n (8 idiomas), Lang + Msg + tr()
├── opcua_server.rs    # Servidor OPC UA: build + espaço de endereçamento + callbacks
├── gui.rs             # IHM egui (feature gui)
├── branding.rs        # Logótipos embebidos (feature gui)
└── actors/
    ├── simulation.rs  #   malha de regulação (tick 0,5 s)
    └── network.rs     #   servidor OPC UA (re)configurável a quente
```

---

## 4. Configuração

`AppConfig` (idioma / rede / processo / regulação / `check_updates`) é
serializada em **TOML** (`mock_ru_opcua.toml`, substituível por `MOCK_CONFIG`),
carregada no arranque (predefinições se ausente), guardada a partir da IHM. Todo o valor
é **higienizado** no carregamento (`AppConfig::sanitized`: limites ordenados,
`τ ≥ 1e-3`, `dead_time ≥ 0`, flutuantes finitos).

**Invariante**: nunca chamar `f32::clamp` com limites não validados (panic
se `min > max` ou `NaN`). As escritas de rede passam também por
`Regulator::apply`, que higieniza.

### Verificação de atualização

Feature `gui` apenas: no arranque, a IHM interroga a última release
GitHub via a lib partilhada `mock_lib_update` (thread limitada por timeout) e exibe
um aviso se uma versão mais recente existir. Regulável por `check_updates`.

---

## 5. Dependências e armadilhas de versão

- **`async-opcua` 0.18** (servidor). Cripto **100 % Rust** (RustCrypto): **nenhuma
  dependência OpenSSL** → compilação cruzada limpa. Licença **MPL-2.0** (cf. `NOTICE`).
- ⚠️ `async-opcua` não declara **nenhum MSRV**: validar na toolchain alvo antes
  de subir a versão.
- ⚠️ O certificado de instância (`create_sample_keypair(true)` + `pki/`) só é gerado
  **em modo cifrado** (`security.encryption`). Em modo None (predefinição), nenhum
  certificado (arranque instantâneo). ⚠️ A geração RSA em Rust puro é lenta em
  *debug*: contar alguns segundos na primeira passagem para modo cifrado.
- `egui_plot` permanece **à frente uma versão menor** do `egui` (cf. ORME/OSNE).

---

## 6. Estender o projeto

### 6.1 Adicionar um nó OPC UA

Em [`opcua_server.rs`](../../src/opcua_server.rs): declarar o nó
(`add_var`), ligar um callback de leitura (`on_read_*`) e, se inscritível, um
callback de escrita (`on_write_*`) que emite um `Command`. Refletir a tabela em
[`reference_opcua.md`](reference_opcua.md).

### 6.2 Adicionar um comando de negócio

Estender o enum `Command` ([`regulator.rs`](../../src/regulator.rs)), tratar o caso
em `Regulator::apply` (com higienização), adicionar um teste.

### 6.3 Adicionar uma cadeia de interface (i18n)

Adicionar uma variante a `Msg` ([`i18n.rs`](../../src/i18n.rs)) e **as 8
traduções** (tabela de tamanho fixo verificada na compilação).

### 6.4 Segurança (`SecurityConfig`)

A segurança está implementada em [`opcua_server.rs`](../../src/opcua_server.rs):
`security.encryption` adiciona um endpoint `Basic256Sha256`/`SignAndEncrypt` com
certificado auto-gerado e tokens anónimo e/ou utilizador/palavra-passe
(`ServerUserToken::user_pass`). O filtro de log `opcua_crypto::certificate_store=off`
([`main.rs`](../../src/main.rs)) só diz respeito ao modo None (sem certificado);
em modo cifrado não tem efeito. Pistas: políticas `Aes256Sha256RsaPss`, lista
de confiança PKI explícita em vez de `trust_client_certs`, tokens X.509.

---

## 7. Estratégia de teste

O núcleo de negócio (`regulator.rs`) e a configuração (`config.rs`) são **puros e
testados**: convergência PID, clamp de referência, relaxação à paragem, mudança de
processo sem salto de PV, higienização TOML, ida e volta TOML. O i18n verifica a
não-vacuidade e a ida e volta de idioma. A lógica async (atores, servidor) permanece
fina e apoia-se nestes blocos testados.

---

## 8. Resolução de problemas

| Sintoma | Causa provável | Remédio |
|---|---|---|
| `failed to bind` no arranque | porta já ocupada / < 1024 sem direitos | mudar a porta (*Parâmetros*) ou lançar como root |
| Cliente não vê os nós | mau endpoint / segurança | `opc.tcp://…:4840/`, None, Anonymous; *Browse* sob `Objects` |
| Escrita `Bad_TypeMismatch` | tipo incorreto | `Double` para as grandezas, `Boolean` para `Run`/`Auto` |
| WARN «encrypted endpoints disabled» | nenhum certificado (Fase 1b) | normal; o endpoint None funciona |

---

## 9. Build «prod» — compilação cruzada a partir de Linux

O instrumento está integrado em [`scripts/build-prod.sh`](../../../scripts/build-prod.sh)
(tabela `INSTRUMENTS`): exes **com IHM** para Linux x86_64, Windows x86_64 e
Raspberry Pi arm64 (via `cross`), mais uma imagem Docker headless.

⚠️ **Cross Windows e `GetHostNameW`**: a pilha OPC UA puxa `gethostname`, que faz
referência ao símbolo winsock `GetHostNameW`. A biblioteca de importação mingw-w64 da
imagem `cross` **predefinida** (`:0.2.5`) é demasiado antiga para o fornecer →
falha na edição de ligações. O repositório fixa portanto, em [`Cross.toml`](../../../Cross.toml),
a imagem Windows GNU em **`:main`** (mingw recente). Validado: builds headless **e**
IHM produzem um `.exe` válido; ORME/OSNE compilam sempre (imagem superconjunto).

---

## 10. Convenções

- Código e comentários em **francês**; logs/erros em **inglês**.
- Cadeias IHM via `i18n` (8 idiomas); nunca codificadas no código.
- Lógica de negócio **síncrona e testável**; o assíncrono é confinado aos atores
  e ao IO. `cargo clippy --workspace` sem avisos.
- Invariantes `ractor`: nenhuma guarda `Mutex` através de um `.await`; nenhum
  temporizador/`spawn` desanexado sem `JoinHandle` abandonado à paragem.
