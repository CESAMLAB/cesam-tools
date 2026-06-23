# Manual do utilizador — Regulador de processo simulado (RU/OPC UA)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · **PT** · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

> Crate: `mock_bin_ru_opcua` · Executável: **ru_opcua**

---

## 1. Para que serve este simulador

`ru_opcua` simula um **regulador de processo** (malha PID sobre um processo
térmico) e expõe-no em **OPC UA**, o padrão de supervisão industrial.
Serve para **testar um cliente OPC UA / um SCADA** (leitura de medições, escrita de
referências, subscrições) sem material real.

A interface gráfica permite **pilotar** a simulação e **visualizar** a
dinâmica; o servidor OPC UA expõe as mesmas grandezas à rede.

---

## 2. Primeiros passos

```bash
cargo run -p mock_bin_ru_opcua          # IHM + servidor OPC UA
```

No arranque, o servidor escuta por predefinição em `opc.tcp://0.0.0.0:4840/`
(segurança None). A janela exibe o estado atual e inicia a curva de
tendência.

Ligue um cliente OPC UA (UaExpert, etc.) a `opc.tcp://127.0.0.1:4840/`,
segurança **None**, utilizador **Anonymous**. Os nós estão descritos na
[referência OPC UA](reference_opcua.md).

---

## 3. A interface

### Cabeçalho

- **Título** e botões **⚙ Parâmetros** / **💾 Guardar as definições**.
- À direita: **estado do aparelho** (EM MARCHA / PARADO), **estado do servidor**
  (`OPC UA ● opc.tcp://…` a verde se à escuta, ✖ + mensagem em caso de erro), e
  o **logótipo CESAM-Lab**.
- Um **aviso laranja** lembra permanentemente que o endpoint é **anónimo
  (segurança None)**: a expor apenas em rede de confiança.
- Se houver uma atualização disponível, um **aviso** propõe a transferência.

### Painel de comandos (esquerda)

- **Marcha / Paragem**: inicia ou para a regulação. Parado, o processo
  relaxa para o valor ambiente.
- **Modo automático (PID)**: ativado = o PID calcula a saída; desativado =
  **modo manual** (a saída é imposta).
- **Referência**: cursor, limitado pelos limites de referência (reguláveis em
  *Parâmetros*).
- **Saída manual (%)**: cursor ativo apenas em **modo manual**.
- **Definições PID**: ganhos `Kp`, `Ki`, `Kd` editáveis a quente.

### Zona central

- **Cartões**: Medição, Referência, Saída.
- **Curva de tendência**: Medição (PV) e Referência no eixo esquerdo (unidade
  de processo), Saída (%) no eixo direito.

---

## 4. Parâmetros (modal ⚙)

- **Idioma** da interface (8 idiomas), persistido.
- **Verificar as atualizações no arranque** + botão **Verificar agora**.
- **Endpoint**: **IP de escuta** e **porta** do servidor OPC UA. Uma alteração
  **reinicia** o servidor a quente (as sessões em curso são fechadas limpamente).
- **Segurança OPC UA**: **Cifragem** (`Basic256Sha256`), **Permitir o anónimo**,
  **Utilizador** / **Palavra-passe** (campos ativos quando a cifragem está marcada).
  Ativar a cifragem gera um certificado no primeiro arranque (alguns
  segundos) e reinicia o servidor.
- **Processo (função de transferência)**: ganho `K`, constante de tempo `τ`, atraso
  puro, valor ambiente.
- **Limites de referência**: min / max (reordenados automaticamente se invertidos).
- **Aplicar** / **Repor predefinições** / **Fechar**.

As definições são guardadas em `mock_ru_opcua.toml` (diretório atual;
substituível pela variável de ambiente `MOCK_CONFIG`).

---

## 5. Segurança

A segurança OPC UA é **regulável** em *Parâmetros*:

- **Sem cifragem** (predefinição): endpoint **segurança None**, acesso **anónimo** —
  nenhuma proteção. **Não expor numa rede aberta.** Um aviso **laranja**
  lembra-o.
- **Com cifragem**: endpoint **`Basic256Sha256`** (assinado + cifrado). O
  servidor gera o seu certificado no primeiro arranque e aceita os certificados
  de cliente. Pode exigir-se um **utilizador / palavra-passe** e/ou permitir
  o anónimo. Um aviso **verde 🔒** confirma a cifragem. Para se ligar, o
  cliente deve então utilizar a política `Basic256Sha256` e confiar no
  certificado do servidor (primeira troca).

A palavra-passe é armazenada **em claro** no ficheiro TOML: é um
**simulador**, a utilizar numa rede de confiança.

---

## 6. FAQ

**A porta 4840 é obrigatória?** Não: regula-se em *Parâmetros* (ou via o
ficheiro TOML). Uma porta < 1024 requer direitos de root.

**O meu cliente não vê os nós.** Verifique a conexão a `opc.tcp://…:4840/`,
segurança **None**, utilizador **Anonymous**, depois *Browse* sob a pasta
`Objects` (namespace `urn:cesam-lab:ru-opcua`).

**Uma escrita é recusada.** O tipo deve corresponder (`Double` para as
grandezas, `Boolean` para `Run`/`Auto`); senão o servidor devolve
`Bad_TypeMismatch`.

**Lançar sem interface gráfica?** Compile em *headless*:
`cargo run -p mock_bin_ru_opcua --no-default-features` — o servidor OPC UA e a
simulação funcionam sem IHM.

**Aparece uma mensagem «encrypted endpoints disabled».** É normal na
Fase 1b: nenhum certificado de instância é provisionado (endpoints cifrados
indisponíveis). O endpoint None, esse, funciona.
