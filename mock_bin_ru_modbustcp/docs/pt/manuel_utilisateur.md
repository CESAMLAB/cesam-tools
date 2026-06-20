# Manual do utilizador — ORME (regulador simulado Modbus)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · **PT** · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

> **ORME** — *Open Regulator Modbus Emulator* · binário `mock_bin_ru_modbustcp` ·
> Licença MIT · Editor: **CESAM-Lab** · Identificador de aparelho Modbus: **CESAM-Lab**
>
> *«Abra o barramento.»* Um regulador de terreno que só existe no seu barramento
> Modbus (TCP/RTU) — para testar SCADA, autómatos e IHM sem hardware real.

Este manual destina-se ao **utilizador** do regulador simulado: como o lançar,
comandá-lo a partir da interface, parametrizá-lo e ligá-lo em Modbus TCP.
Nenhum conhecimento de programação é necessário.

---

## 1. Para que serve este software?

Simula um **regulador industrial** (tipo forno ou banho termostático):

- um **processo físico** realista (a «medida» sobe/desce conforme o comando);
- uma **regulação** automática ou manual, em **aquecimento** e/ou em **arrefecimento**;
- um **servidor Modbus TCP** para o comandar/supervisionar a partir de outro software
  (autómato, SCADA, gateway…);
- uma **interface gráfica** de condução e visualização.

É uma ferramenta de **teste**: permite desenvolver e demonstrar um
supervisor ou um autómato **sem hardware real**.

---

## 2. Iniciar o software

Lançar o executável correspondente ao seu sistema:

| Sistema | Ficheiro |
|---------|---------|
| Windows | `orme-windows-x86_64.exe` (duplo-clique) |
| Linux PC | `./orme-linux-x86_64` |
| Raspberry Pi (ecrã) | `./orme-rpi-arm64` |

A janela abre-se e o **servidor Modbus arranca automaticamente** (porta `5502`
por omissão). O cabeçalho indica o estado:

- **● EM MARCHA / ● PARADO**: estado do aparelho;
- **Modbus ● 0.0.0.0:5502** (verde): servidor à escuta; **✖ …** (vermelho) em caso
  de problema de rede.

> Sem ecrã (apenas servidor), ver o **§ 9 (Utilização sem ecrã)**.

---

## 3. A interface num relance

A janela comporta quatro zonas:

```
┌───────────────────────────── Cabeçalho: título, ⚙ Parâmetros, 💾 Guardar, estados ─────────────────────────────┐
├──────────────────┬─────────────────────────────────────────────────┬───────────────────────────────────────────┤
│  COMANDOS         │   SUPERVISÃO                                    │   TABELA DE ENDEREÇOS MODBUS              │
│  (esquerda)       │   - valores instantâneos (Medida / Consigna /  │   (direita)                               │
│  Marcha/Paragem   │     Saída)                                      │   lista ao vivo: designação, tabela,      │
│  Auto/Manual      │   - CURVA de tendência em tempo real            │   endereço, valor, acesso                 │
│  Modos, consignas │                                                 │                                           │
│  regulações PID…  │                                                 │                                           │
└──────────────────┴─────────────────────────────────────────────────┴───────────────────────────────────────────┘
```

---

## 4. Comandar o regulador (painel da esquerda)

### 4.1 Marcha / Paragem
Botão **Marcha / Paragem**. Em paragem, a saída é nula e a medida regressa
lentamente ao valor ambiente.

### 4.2 Auto / Manual
- **Manual**: *você* impõe a saída através da **consigna manual** (em %).
- **Auto**: o regulador calcula a saída para atingir a **consigna auto**.

### 4.3 As consignas
Cada consigna dispõe de um **campo numérico** (entrada precisa pelo teclado) e
de um **cursor**. Ambos são sempre modificáveis; a consigna **ativa**
(conforme o modo) é apresentada a negrito.

| Consigna | Unidade | Papel |
|----------|-------|------|
| **SP auto** | unidade de medida (ex. °C) | alvo a atingir em modo Auto |
| **SP manual** | % de saída, de −100 a +100 | saída imposta em modo Manual (**+** aquecimento / **−** arrefecimento) |

### 4.4 Modos de regulação — sentido 1 (aquecimento) e sentido 2 (arrefecimento)
Cada sentido regula-se independentemente:

- **Desativado** — o sentido não atua;
- **PID** — regulação contínua (saída 0…100 %), precisa e suave;
- **Tudo-ou-nada (TOR)** — relé com histerese: saída 0 % ou 100 %, simples mas
  oscilante em torno da consigna;
- **Relé de ciclo (PWM)** — um PID calcula uma razão cíclica, *picotada* num
  período fixo: a saída física permanece tudo-ou-nada (0/100 %), mas a sua
  **média** segue o PID. É o melhor compromisso para comandar finamente um
  órgão que só sabe abrir ou fechar (relé, válvula TOR).

> 👉 **Importante — ver **§ 6 (Compreender a regulação)****: escolher
> PID/TOR/PWM para o arrefecimento *arma* o arrefecimento, mas este só **debita quando
> a medida ultrapassa a consigna**.

### 4.5 Regulações PID (Kp, Ki, Kd)
Para cada sentido, três ganhos ajustáveis em direto:

- **Kp** (proporcional): quanto maior, mais viva é a reação (risco de oscilação);
- **Ki** (integral): anula o desvio residual ao longo do tempo (demasiado forte → ultrapassagem);
- **Kd** (derivado): amortece/antecipa (demasiado forte → sensível ao ruído).

### 4.6 Regulações TOR / PWM
- **Histerese TOR** — largura da **zona morta** do modo Tudo-ou-nada, centrada
  na consigna (`[SP − h/2, SP + h/2]`): evita que a saída comute sem
  parar. Quanto mais larga, maior é a ondulação mas as comutações
  mais espaçadas.
- **Ciclo mín. TOR (s)** — duração mínima durante a qual o relé permanece num
  estado antes de poder recomutar (**anti-ciclo-curto**). Protege um atuador
  real (relé, compressor) e suaviza o comportamento. `0` = desativado.
- **Período PWM (s)** — duração de um ciclo do **relé de ciclo**. Curto → média
  mais fiel mas comutações frequentes; longo → menos desgaste mas ondulação
  mais marcada. A escolher bem mais pequeno que a constante de tempo do processo.

---

## 5. Ler a curva de tendência

A curva (ao centro) traça em tempo real três grandezas. A **legenda, em cima
à esquerda**, recorda a cor **e o último valor** de cada série:

| Cor | Série | Significado |
|---------|-------|---------------|
| 🔵 azul | **Consigna (SP)** | alvo (em Auto) |
| 🔴 vermelho | **Medida (PV)** | valor do processo |
| 🟢 verde | **Saída (%)** | comando aplicado (**+** aquecimento / **−** arrefecimento) |

Por cima da curva, três cartões apresentam os valores instantâneos
(Medida, Consigna ativa, Saída). É possível ampliar/deslocar a curva com o rato.

---

## 6. Compreender a regulação (aquecimento / arrefecimento)

O regulador atua **num só sentido de cada vez**, escolhido conforme o desvio
`Consigna − Medida`:

| Situação | Sentido que atua | Saída | Indicador |
|-----------|---------------|--------|--------|
| Medida **<** Consigna (é preciso aquecer) | **Sentido 1 (aquecimento)** | **positiva** (0…+100 %) | **Aquecimento ativo = 1** |
| Medida **>** Consigna (é preciso arrefecer) | **Sentido 2 (arrefecimento)** | **negativa** (−100…0 %) | **Arrefecimento ativo = 1** |

Consequências práticas:

- Selecionar **PID/TOR para o arrefecimento** não basta para acender «Arrefecimento ativo»:
  é preciso que **a medida esteja acima da consigna**. Enquanto a medida estiver
  abaixo, é o **aquecimento** que trabalha.
- Para ver o arrefecimento a debitar: em **Auto**, sentido 2 em PID/TOR, **baixe a
  consigna abaixo da medida corrente** (ou aguarde uma ultrapassagem). A saída
  torna-se negativa e **Arrefecimento ativo** passa a 1.
- Em **TOR**, o relé comuta na **meia-histerese** de ambos os lados da
  consigna (zona morta simétrica) e respeita o **ciclo mínimo** entre duas
  comutações. Em **PWM**, a saída picota a 0/100 % mas a sua média segue o PID.

---

## 7. Parâmetros (botão ⚙)

O botão **⚙ Parâmetros** abre uma janela para configurar:

### Transporte Modbus
Escolha do barramento de comunicação — **um só ativo de cada vez**:

**TCP (Ethernet)**
- **IP de escuta** (`0.0.0.0` = todas as interfaces) e **Porta** (predefinição 5502);
- **IP autorizadas**: uma por linha, curinga `*` aceites (ex. `192.168.1.*`).
  **Lista vazia = todas as IP autorizadas.** As restantes são recusadas.

**RTU (RS485)** — necessita de um binário compilado com a feature `rtu`
- **Porta série**: `/dev/ttyUSB0`, `/dev/ttyAMA0` (Raspberry Pi), `COM3` (Windows)…;
- **Baud** (predefinição 19200), **Paridade** (predefinição Par), **Bits de dados** (8),
  **Bits de paragem** (1) — a acordar com o mestre;
- **Endereço de escravo** (1–247).

> ⚠️ **Um só mestre remoto de cada vez.** Em TCP, a ligação de um novo
> mestre **desliga automaticamente** o anterior. A IHM local **não** é
> um mestre: permanece sempre ativa. Em RTU, privilegiar uma **ligação
> ponto-a-ponto** (o aparelho responde independentemente do endereço solicitado).

### Função de transferência (processo)
Comportamento físico simulado `G(s) = K·e^(−L·s) / (1 + T·s)`:
- **Ganho K**: variação de medida por % de saída;
- **Constante T** (s): inércia/rapidez;
- **Atraso L** (s): tempo morto antes da reação;
- **Ambiente**: valor de repouso.

### Limites de consigna
Limites mínimo/máximo da consigna auto.

Botões: **Aplicar** (entra em vigor imediatamente **e** regista),
**Repor predefinições**, **Fechar**.

### Gravação das regulações
As regulações são **guardadas** num ficheiro `mock_ru_modbustcp.toml` (ao lado
do software) e **recarregadas no arranque seguinte**. O botão **💾 Guardar
regulações** do cabeçalho regista também os ganhos PID, a histerese, o ciclo
mínimo TOR e o período PWM modificados a partir do painel da esquerda.

---

## 8. Ligar um cliente Modbus

O software é um **escravo Modbus** (TCP porta 5502 por omissão, ou RTU série
conforme o transporte escolhido no § 7). Um cliente (autómato, SCADA, `mbpoll`…) pode
**ler** o estado e **escrever** as consignas/modos. Lembrete: **um só mestre
remoto de cada vez** (em TCP, um recém-chegado desliga o anterior).

Referências principais (endereços **base 0**):

| Dado | Tabela | Endereço | Tipo | Acesso |
|--------|-------|---------|------|-------|
| Marcha/Paragem | Bobina | 0 | bit | L/E |
| Auto/Manual | Bobina | 1 | bit | L/E |
| Modo sentido 1 / sentido 2 | Holding | 0 / 1 | 0=Off,1=PID,2=TOR,3=PWM | L/E |
| Consigna auto | Holding | 2–3 | flutuante | L/E |
| Consigna manual | Holding | 4–5 | flutuante | L/E |
| Ciclo mín. TOR (s) | Holding | 20–21 | flutuante | L/E |
| Período PWM (s) | Holding | 22–23 | flutuante | L/E |
| Medida (PV) | Input | 0–1 | flutuante | L |
| Saída (%) | Input | 2–3 | flutuante | L |
| Identificador «CESAM-Lab» | Holding | 42–46 | texto ASCII | L |

> A **tabela completa** (ganhos PID, histerese, codificação dos flutuantes, códigos
> de função, exemplos `mbpoll`) está em **[table_modbus.md](table_modbus.md)**.
> A mesma tabela é também visível **em direto** no painel da direita da IHM.

---

## 9. Utilização sem ecrã («headless» / Docker)

Para um despliegue em segundo plano (Raspberry Pi sem ecrã, servidor), existe uma
versão **sem interface**: faz correr a simulação e o servidor
Modbus, comandáveis **unicamente por Modbus**.

```bash
# Imagem Docker (implantável em qualquer lado):
docker run --rm -p 5502:5502 -v "$PWD/conf:/data" orme:headless
```

A pasta montada em `/data` permite fornecer/conservar `mock_ru_modbustcp.toml`.

---

## 10. Perguntas frequentes

| Pergunta / sintoma | Resposta |
|---------------------|---------|
| **«Arrefecimento ativo» não passa a 1 apesar de eu ter posto PID/TOR.** | Normal: o arrefecimento só debita se **a medida ultrapassar a consigna**. Baixe a consigna abaixo da medida (modo Auto). Ver **§ 6 (Compreender a regulação)**. |
| A medida não se mexe. | Verifique que o aparelho está **Em marcha**, e a consigna/saída não nulas. |
| Em manual, mudar os modos sentido 1/2 não faz nada. | Normal: os modos só se aplicam em **Auto**. |
| O cabeçalho mostra **Modbus ✖**. | Porta já em uso ou < 1024 sem privilégios: mude a **porta** em ⚙ Parâmetros. |
| O meu cliente Modbus é recusado. | A sua IP não está na **lista branca**: esvazie a lista ou adicione um padrão (`192.168.1.*`). |
| Os flutuantes lidos são incoerentes. | Problema de **ordem das palavras** do lado do cliente (palavra de maior peso primeiro). Ver table_modbus.md. |
| Uma consigna escrita em Modbus é ignorada. | Um flutuante ocupa **2 registos**: escreva-os **em conjunto**. |
| As minhas regulações não são conservadas. | Clique **Aplicar** / **💾 Guardar**. O ficheiro `mock_ru_modbustcp.toml` deve estar acessível em escrita. |

---

*Documentação técnica associada: [conception.md](conception.md) ·
[table_modbus.md](table_modbus.md) · [maintenance.md](maintenance.md).*
