# Manual do utilizador — OSNE (agitador de laboratório simulado NAMUR)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · **PT** · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

> **OSNE** — *Open Stirrer NAMUR Emulator* · binário `mock_bin_su_namur`
> (executável `osne`) · Licença MIT · Editor: **CESAM-Lab** · Identidade NAMUR:
> nome `CESAM-STIRRER`, tipo `OSNE`.
>
> *Um agitador de laboratório (estilo IKA) que só existe na sua ligação NAMUR —
> para testar supervisores, scripts e gateways sem hardware real.*

Este manual destina-se ao **utilizador** do agitador simulado: como o iniciar,
comandá-lo a partir da interface, parametrizá-lo e ligá-lo em **NAMUR** (TCP ou
série RS-232). Não é necessário qualquer conhecimento de programação.

---

## 1. Para que serve este software?

Simula um **agitador de laboratório** (agitador de bancada com hélice, estilo IKA):

- um **motor físico** realista: a velocidade sobe/desce consoante o binário
  aplicado, com uma **regulação de velocidade rápida**;
- uma **carga viscosa ajustável**: quanto mais viscoso é o meio, mais elevado é o
  binário necessário — até à **sobrecarga** (consigna inatingível);
- um **servidor NAMUR** (protocolo série ASCII dos aparelhos de laboratório) para
  o comandar/supervisionar a partir de outro software ou de um script;
- uma **interface gráfica** de condução, de visualização e de **teste do
  protocolo** (mini-terminal NAMUR integrado).

É uma ferramenta de **teste**: permite desenvolver e demonstrar um supervisor, um
script de aquisição ou um gateway **sem hardware real**.

---

## 2. Iniciar o software

Lançar o executável correspondente ao seu sistema:

| Sistema | Ficheiro |
|---------|---------|
| Windows | `osne-windows-x86_64.exe` (duplo clique) |
| Linux PC | `./osne-linux-x86_64` |
| Raspberry Pi (ecrã) | `./osne-rpi-arm64` |

A janela abre-se e o **servidor NAMUR arranca automaticamente** (porta `4001` por
defeito). O cabeçalho indica o estado:

- **● EM FUNCIONAMENTO / ● PARADO**: estado do motor;
- **NAMUR ● 0.0.0.0:4001** (verde): servidor à escuta; **✖ …** (vermelho) em caso
  de problema (porta ocupada, série indisponível…);
- um **indicador de ligação**: em TCP mostra o mestre ligado (ou «nenhum mestre»),
  em série um simples ponto. Passa a **verde** quando uma trama foi recebida
  recentemente (ligação ativa), cinzento caso contrário.

> Sem ecrã (apenas servidor), ver o **§ 9 (Utilização sem ecrã)**.

---

## 3. A interface num relance

```
┌──────────────── En-tête : titre OSNE, ⚙ Paramètres, 💾 Sauvegarder, états & voyants ────────────────┐
├──────────────────┬──────────────────────────────────────────────────────────────────────────────────┤
│  COMMANDES        │   SUPERVISION                                                                      │
│  (gauche)         │   - cartes de valeurs (Vitesse / Couple / Viscosité / Surcharge)                  │
│  Marche/Arrêt     │   - COURBE de tendance temps réel (Consigne / Vitesse / Couple)                   │
│  Consigne vitesse │                                                                                   │
│  Viscosité        │                                                                                   │
│  Réglages PID     │                                                                                   │
├──────────────────┴──────────────────────────────────────────────────────────────────────────────────┤
│  ⇄ TRAMES NAMUR : mini-terminal (RX/TX) + ligne de commande + référence du protocole (à droite)       │
└───────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 4. Comandar o agitador (painel da esquerda)

### 4.1 Arranque / Paragem
Botão **Arranque / Paragem**. Em paragem, o motor desacelera livremente até à
imobilização (atrito + carga), com binário do motor nulo.

### 4.2 Consigna de velocidade
Cursor **Consigna de velocidade** (em `tr/min`), limitado pelos limites mín/máx
definidos nos *Parâmetros*. É a mesma grandeza que o comando NAMUR `OUT_SP_4`
(canal 4). Em funcionamento, o controlo conduz a velocidade medida até esta
consigna.

### 4.3 Viscosidade do meio
Cursor **Viscosidade** (escala logarítmica). Representa a **carga** do meio
agitado:

- viscosidade **baixa** → binário baixo, a consigna é atingida rapidamente;
- viscosidade **elevada** → binário de carga importante; se o binário necessário
  ultrapassa o **binário máximo do motor**, a velocidade de consigna **deixa de ser
  atingida** → o indicador **Sobrecarga ⚠** acende-se (comportamento de um agitador
  real perante um meio demasiado espesso).

### 4.4 Ajustes PID (Kp, Ki, Kd)
Os três ganhos do controlo de velocidade, ajustáveis em direto:

- **Kp** (proporcional): quanto maior, mais viva é a subida em velocidade (risco de
  sobreelevação/oscilação);
- **Ki** (integral): anula o desvio residual de velocidade ao longo do tempo;
- **Kd** (derivado): amortece/antecipa (demasiado forte → sensível ao ruído).

> Os ganhos por defeito são deliberadamente «agressivos»: a saída satura no binário
> máximo enquanto o erro é grande (subida rápida), depois o termo integral
> estabiliza. A saída do PID **é** o binário do motor, limitado a `[0, couple_max]`.

---

## 5. Ler a curva de tendência

A curva (ao centro) traça três grandezas em tempo real. A **legenda, no canto
superior esquerdo**, recorda a cor **e o último valor** de cada série:

| Cor | Série | Significado |
|---------|-------|---------------|
| 🔵 azul | **Consigna** | consigna de velocidade (em funcionamento) |
| 🔴 vermelho | **Velocidade** | velocidade medida (`tr/min`, eixo da esquerda) |
| 🟢 verde | **Binário** | binário medido (`N·cm`, **eixo da direita**) |

> A curva tem **dois eixos verticais**: a **velocidade** (`tr/min`) à esquerda, o
> **binário** (`N·cm`) à direita. O binário é escalado para partilhar o gráfico,
> mas o eixo da direita exibe efetivamente `N·cm`.

Por cima da curva, **cartas** exibem os valores instantâneos: **Velocidade**,
**Binário**, **Viscosidade** e **Sobrecarga ⚠** quando o motor satura. Pode-se
ampliar/deslocar a curva com o rato.

---

## 6. O mini-terminal NAMUR (parte inferior da janela)

O painel **⇄ Tramas NAMUR** permite **testar o protocolo** diretamente a partir da
IHM, sem cliente externo:

- o **registo** mostra as tramas **recebidas** (`← RX`, azul) e **emitidas**
  (`→ TX`, verde), com data e hora;
- a **linha de comando** envia uma trama NAMUR ao simulador (tecla **Enter** ou
  botão **▶ Enviar**). As setas **↑/↓** recordam os comandos anteriores
  (histórico);
- a **referência do protocolo** (painel da direita) lista os comandos: um **clique**
  insere o comando na linha de entrada;
- o botão **🗑 Limpar** esvazia o registo.

> As tramas digitadas aqui são interpretadas exatamente como as de um mestre de
> rede: `OUT_SP_4 500` define a consigna, `START_4`/`STOP_4` arrancam/param,
> `IN_PV_4` lê a velocidade, etc. O **cão de guarda** (`OUT_WD1@…`) só tem, contudo,
> efeito no âmbito de uma verdadeira sessão de rede (cf. § 8).

---

## 7. Parâmetros (botão ⚙)

O botão **⚙ Parâmetros** abre uma janela para configurar:

### Idioma da interface
Seletor no topo: **Français, English, Deutsch, Español, Italiano, Português,
Nederlands, Polski** (8 idiomas). O idioma é persistido.

### Transporte NAMUR
Escolha da ligação — **uma única ativa de cada vez**:

**TCP (Ethernet)**
- **IP de escuta** (`0.0.0.0` = todas as interfaces) e **Porta** (defeito 4001);
- **IP autorizadas**: uma por linha, carateres universais `*` aceites (ex.
  `192.168.1.*`). **Lista vazia = todas as IP autorizadas.** As restantes são
  recusadas.

**Série (RS-232)** — requer um binário compilado com a feature `serial`
- **Porta série**: `/dev/ttyUSB0` (Linux), `COM3` (Windows)…;
- **Baud** (defeito 9600), **Paridade** (defeito Par), **Bits de dados** (7),
  **Bits de stop** (1) — configuração NAMUR de laboratório típica: **9600 7E1**.

> ⚠️ **Um único mestre de cada vez.** Em TCP, um novo mestre **aguarda** até à
> desconexão do anterior (ligação ponto-a-ponto). A IHM local **não é** um mestre.
> Em série, o barramento *é* o único mestre; privilegiar uma **ligação
> ponto-a-ponto** (o servidor responde qualquer que seja o endereço pedido).

### Parâmetros do motor
Comportamento físico simulado `J·dω/dt = T − k·η·ω − frottement`:
- **Inércia** (`J`): reatividade do motor (pequeno ⇒ rápido);
- **Coeficiente de carga** (`k`): peso da viscosidade no binário;
- **Atrito** (`N·cm`): atrito seco residual;
- **Binário máx** (`N·cm`): binário máximo do motor (teto da saída PID).

### Limites de velocidade
Limites mín/máx da consigna de velocidade (`tr/min`).

### Limites de viscosidade
Limites mín/máx do cursor de viscosidade.

Botões: **Aplicar** (entra em vigor imediatamente **e** guarda), **Repor por
defeito**, **Fechar**.

### Gravação dos ajustes
Os ajustes são **guardados** num ficheiro `mock_su_namur.toml` (junto ao software) e
**recarregados no próximo arranque**. O botão **💾 Guardar** do cabeçalho grava
também os ganhos PID e a viscosidade modificados a partir do painel da esquerda.

---

## 8. Ligar um cliente NAMUR

O software é um **escravo NAMUR** (TCP porta 4001 por defeito, ou série consoante o
transporte escolhido no § 7). Um cliente (script, terminal, gateway) **envia uma
linha ASCII por pedido**, terminada por `CR LF`. As **leituras** (`IN_*`) devolvem
um valor; as **escritas/ações** (`OUT_*`, `START_*`, `STOP_*`, `RESET`) são
**silenciosas** (sem resposta), conforme o uso NAMUR.

Pontos principais:

| Comando | Efeito |
|----------|-------|
| `IN_NAME` / `IN_TYPE` | identidade (`CESAM-STIRRER` / `OSNE`) |
| `IN_PV_4` / `IN_PV_5` | ler a velocidade (`tr/min`) / o binário (`N·cm`) |
| `IN_SP_4` | ler a consigna de velocidade |
| `OUT_SP_4 <v>` | **definir** a consigna de velocidade |
| `START_4` / `STOP_4` / `RESET` | arrancar / parar / reiniciar |
| `OUT_WD1@<m>` | **cão de guarda**: paragem segura se silêncio durante `<m>` s |

Exemplo com `nc` (netcat):

```text
$ nc 127.0.0.1 4001
IN_NAME
CESAM-STIRRER
OUT_SP_4 1200          (silencieux)
START_4                (silencieux)
IN_PV_4
1200.0 4
STOP_4                 (silencieux)
```

> O **cão de guarda** `OUT_WD1@30` para automaticamente o motor se **nenhuma linha**
> chegar durante 30 s (proteção em caso de perda de comunicação). `OUT_WD1@0`
> desarma-o. O contador é rearmado a cada comando recebido.

> A **referência completa do protocolo** (canais, codificação, exemplos) está em
> **[commandes_namur.md](commandes_namur.md)**. A mesma lista é recordada **em
> direto** no painel da direita do mini-terminal.

---

## 9. Utilização sem ecrã («headless» / Docker)

Para uma implementação em segundo plano (Raspberry Pi sem ecrã, servidor), existe
uma versão **sem interface**: faz correr a simulação e o servidor NAMUR,
comandáveis **apenas por NAMUR**.

```bash
# Image Docker (déployable n'importe où) :
docker run --rm -p 4001:4001 -v "$PWD/conf:/data" osne:headless
```

A pasta montada em `/data` permite fornecer/conservar `mock_su_namur.toml`.

---

## 10. Perguntas frequentes

| Pergunta / sintoma | Resposta |
|---------------------|---------|
| **Sobrecarga ⚠** acende-se e a velocidade não atinge a consigna. | Normal: a **viscosidade** exige mais binário do que o motor fornece. Reduza a viscosidade ou a consigna, ou aumente o **binário máx** (Parâmetros). |
| A velocidade não se move. | Verifique se o agitador está **Em funcionamento** e a consigna não é nula. |
| O cabeçalho mostra **NAMUR ✖**. | Porta já utilizada ou < 1024 sem privilégios (TCP), ou porta série indisponível: altere o ajuste em ⚙ Parâmetros. |
| O meu cliente NAMUR/TCP é recusado. | O seu IP não está na **lista branca**: esvazie a lista ou adicione um padrão (`192.168.1.*`). |
| `OUT_SP_4 …` não devolve nada. | Normal: as escritas/ações NAMUR são **silenciosas**. Leia com `IN_SP_4` / `IN_PV_4`. |
| O motor para sozinho. | Um **cão de guarda** está armado (`OUT_WD1@…`) e nenhum comando chegou a tempo. Desarme-o (`OUT_WD1@0`) ou envie tramas regularmente. |
| A ligação série não abre. | Binário compilado **sem** a feature `serial`, ou porta/permissões incorretas (grupo `dialout` no Linux). |
| Os meus ajustes não são conservados. | Clique em **Aplicar** / **💾 Guardar**. O ficheiro `mock_su_namur.toml` deve ser acessível para escrita. |

---

*Documentação técnica associada: [conception.md](conception.md) ·
[commandes_namur.md](commandes_namur.md) · [maintenance.md](maintenance.md).*
