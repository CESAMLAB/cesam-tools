# Manual do utilizador — Regulador PROFIBUS DP simulado (ORPD)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · **PT** · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

> Crate: `mock_bin_ru_pbdp` · Executável: **ru_pbdp** · Marca: **ORPD**

---

## ⚠️ Antes de começar: o que este simulador NÃO é

`ru_pbdp` **não é** um escravo PROFIBUS DP conforme ao hardware real. O
PROFIBUS DP é um barramento de testemunho cujo cumprimento das janelas
temporais (*slot time*, `Tsdr`, watchdog) exige um circuito dedicado
(ASIC SPC3/VPC3, placa mestra Hilscher/Softing/Siemens CP). Um programa
Tokio comum, mesmo ligado a uma porta RS-485 real, **não consegue cumprir
estas restrições**: um autómato real (um Siemens S7 com placa mestra, por
exemplo) **nunca** reconhecerá este simulador como escravo válido num
barramento real.

O que o `ru_pbdp` faz efetivamente: implementa, **em software e sem
restrições de tempo real**, a estrutura de tramas e a máquina de estados
de um escravo DP-V0 (parametrização, configuração, diagnóstico, troca
cíclica). É uma ferramenta para **compreender o protocolo** e **testar um
desenvolvimento de software** (codec, máquina de estados, ferramentas) —
não para pilotar equipamento de campo. Ver
[reference_profibus.md](reference_profibus.md) §6 para o detalhe das
limitações.

---

## 1. Para que serve este simulador

O `ru_pbdp` simula um **regulador de processo** (ciclo PID sobre um
processo térmico, modelo idêntico ao ORME/Modbus) e expõe-o através de um
conjunto simulado de tramas PROFIBUS DP-V0, sobre uma ligação série
(RS-485/RS-232). A interface gráfica permite **pilotar** a simulação e
**visualizar** a sua dinâmica; o registo de tramas mostra o tráfego
trocado em hexadecimal.

---

## 2. Primeiros passos

```bash
cargo run -p mock_bin_ru_pbdp          # IU + ligação série PROFIBUS DP
```

No arranque, o simulador tenta abrir a porta série configurada (por
defeito `/dev/ttyUSB0` ou `COM3`, 500 kbit/s, endereço de estação 3). Se
a porta não existir (caso frequente sem hardware série), a IU mostra o
erro de abertura no cabeçalho — a simulação do regulador continua a
funcionar, apenas a ligação está indisponível. Ajuste a **porta série**
em *Definições* para apontar para um pseudoterminal ou um adaptador
USB-série disponível.

---

## 3. A interface

### Cabeçalho

- **Título** e botões **⚙ Definições** / **💾 Guardar definições**.
- À direita: **estado do aparelho** (EM FUNCIONAMENTO / PARADO), **estado
  da ligação** (`PROFIBUS ● <porta> [<estado>]` a verde se aberta — o
  estado mostrado é o da máquina de estados DP-V0:
  `Power_On`/`Wait_Prm`/`Wait_Cfg`/`Data_Exchange`), e o **logótipo
  CESAM-Lab**.
- Uma **faixa laranja permanente** recorda a não-interoperabilidade com
  hardware real (ver o aviso acima).

### Mini-terminal (parte inferior da janela)

Registo apenas de leitura das tramas **recebidas** (← RX) e **emitidas**
(→ TX), com marca temporal e visualização hexadecimal. Botão **Limpar**
para esvaziar o registo.

### Painel de comandos (esquerda)

Idêntico ao ORME: **Marcha/Paragem**, **Auto/Manual**, modos de
regulação **sentido 1 (aquecer)** / **sentido 2 (arrefecer)**
(Off/PID/Tudo-ou-nada/PWM), **valores de referência** (automático e
manual), **ajustes PID** de ambos os sentidos, **histerese**, **ciclo
mínimo tudo-ou-nada**, **período PWM**.

### Painel direito: blocos de E/S PROFIBUS

Tabela em direto dos blocos *Output* (mestre→escravo) e *Input*
(escravo→mestre), com a disposição de bytes utilizada por este
simulador — ver [reference_profibus.md](reference_profibus.md) §3.

### Zona central

Cartões **Medida**, **Referência ativa**, **Saída**, e curva de
tendência.

---

## 4. Definições (modal ⚙)

- **Idioma** da interface (8 idiomas), persistido.
- **Verificar atualizações ao arrancar** + botão **Verificar agora**.
- **Porta série**, **velocidade** (baud — utilizar um valor normalizado
  PROFIBUS DP: 9600, 19200, 45450, 93750, 187500, 500000, 1500000,
  3000000, 6000000 ou 12000000), **endereço de estação** (0-125).
- **Watchdog de protocolo (permitido)**: caixa de verificação — se
  desmarcada, o watchdog solicitado pelo mestre via `Set_Prm` é
  **ignorado** (nunca armado).
- **Função de transferência do processo**: ganho `K`, constante de tempo
  `τ`, atraso puro, valor ambiente.
- **Limites de referência**: mín / máx (reordenados automaticamente se
  invertidos).
- **Aplicar** / **Repor predefinições** / **Fechar**.

Uma alteração de porta/velocidade/endereço **fecha e reabre** a ligação
série. As definições são guardadas em `mock_ru_pbdp.toml` (diretório
atual; substituível via a variável de ambiente `MOCK_CONFIG`).

**O formato de trama (8E1) é fixado pela norma PROFIBUS DP** e não é
ajustável aqui, ao contrário do Modbus RTU ou NAMUR série.

---

## 5. O mini-terminal como ferramenta pedagógica

Sem hardware PROFIBUS real, a melhor forma de observar o protocolo é
fazer dialogar **duas instâncias** desta ferramenta entre si — ou
escrever um pequeno script que reproduza uma sequência `Slave_Diag` →
`Set_Prm` → `Chk_Cfg` → `Data_Exchange` sobre um pseudoterminal
(`socat -d -d pty,raw,echo=0 pty,raw,echo=0`) — e ler o mini-terminal para
ver as tramas trocadas em hexadecimal, com a sua descodificação em
[reference_profibus.md](reference_profibus.md).

---

## 6. Perguntas frequentes

**Posso ligar este simulador a um autómato PROFIBUS DP real?** Não — ver
o aviso no início deste documento e o §6 de
[reference_profibus.md](reference_profibus.md).

**A porta série não abre.** O ficheiro/dispositivo indicado não existe ou
as permissões são insuficientes (grupo `dialout` em Linux). O erro exato
é mostrado no cabeçalho da IU.

**A ligação permanece em `Wait_Prm`.** O mestre ainda não enviou um
`Set_Prm` com o identificador esperado (`0xEE01`, identificador
**fictício**, não registado junto do PNO). Ver
[reference_profibus.md](reference_profibus.md) §2.

**A ligação permanece em `Wait_Cfg`.** O `Chk_Cfg` recebido não anuncia
os comprimentos de E/S esperados (45 bytes de saída, 17 de entrada para
este simulador).

**O aparelho para sozinho.** O watchdog de protocolo (armado pelo mestre
via `Set_Prm`) expirou por falta de troca cíclica recebida a tempo — é o
estado seguro esperado, não uma falha.

**Lançar sem interface gráfica?** Compile em modo *headless*:
`cargo run -p mock_bin_ru_pbdp --no-default-features` — a ligação série e
a simulação funcionam sem IU.
