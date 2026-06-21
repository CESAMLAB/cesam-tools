# Tabela de endereços Modbus — Regulador simulado

*🌍 [FR](../fr/table_modbus.md) · [EN](../en/table_modbus.md) · [DE](../de/table_modbus.md) · [ES](../es/table_modbus.md) · [IT](../it/table_modbus.md) · **PT** · [NL](../nl/table_modbus.md) · [PL](../pl/table_modbus.md)*

> Crate: `mock_bin_ru_modbustcp` · Protocolo: **Modbus TCP** (escravo / servidor)

Este documento é a referência funcional do plano de endereçamento. A **fonte de
verdade técnica** continua a ser o cabeçalho de [`src/map.rs`](../../src/map.rs): qualquer
divergência deve ser corrigida no código em prioridade.

---

## 1. Generalidades

| Elemento | Valor |
|---------|--------|
| Transporte | Modbus **TCP** ou **RTU série / RS485** (um só ativo de cada vez) |
| Papel | **Escravo** (servidor) |
| Porta por omissão | TCP `5502` (configurável, modal *Parâmetros*) |
| Série (RTU) | porta + baud + paridade + bits, configuráveis (feature `rtu`) |
| Unit ID / endereço | TCP: indiferente. RTU: `slave_id` configurável mas **não filtrado** (ver nota) |
| Mestres | **um só mestre remoto de cada vez**; em TCP um recém-chegado desliga o anterior (a IHM local não é um mestre) |
| Endereçamento | **base 0** (o endereço `0` = 1º elemento da tabela) |
| Filtragem | lista branca de IP opcional (curinga `*`, apenas TCP) |

> **Nota RTU / endereço de escravo**: o servidor RTU responde **independentemente
> do endereço** solicitado (o endereço não é transmitido ao serviço aplicacional).
> Utilizar uma **ligação ponto-a-ponto**. O `slave_id` é conservado/apresentado mas
> não efetua qualquer filtragem.

### Endereçamento base 0 vs base 1

Os endereços abaixo são os **endereços protocolares (base 0)**, tal
como enviados no quadro. Muitas ferramentas apresentam uma numeração base 1
«convencional» (`4xxxx` para os holdings, `3xxxx` para os inputs…). Assim
o registo de retenção de endereço `2` corresponde à referência convencional `40003`.

---

## 2. Codificação dos números flutuantes (`f32`)

As grandezas analógicas são **`f32` IEE-754 em 2 registos consecutivos**:

- **ordem das palavras**: palavra de **maior peso primeiro** (big-endian, dito *ABCD*);
- **ordem dos octetos** em cada registo: big-endian (padrão Modbus).

Exemplo: `80.0` → octetos `42 A0 00 00` → registo `n` = `0x42A0`,
registo `n+1` = `0x0000`.

> Se o seu cliente ler valores aberrantes, é quase sempre um problema
> de ordem das palavras (experimentar *word swap* / *CDAB*).

---

## 3. Bobinas — *Coils* (leitura/escrita)

Códigos de função: `0x01` (leitura), `0x05` (escrita simples), `0x0F` (escrita múltipla).

| Endereço | Designação | Valores | Efeito |
|---------|-------------|---------|-------|
| `0` | Marcha / Paragem | `0` = paragem, `1` = marcha | Ativa a regulação |
| `1` | Auto / Manual | `0` = manual, `1` = auto | Escolha do modo |

---

## 4. Entradas discretas — *Discrete Inputs* (leitura apenas)

Código de função: `0x02`.

| Endereço | Designação | Significado |
|---------|-------------|---------------|
| `0` | Em marcha | O aparelho está em marcha |
| `1` | Sentido 1 (aquecimento) ativo | Saída > 0 |
| `2` | Sentido 2 (arrefecimento) ativo | Saída < 0 |

---

## 5. Registos de retenção — *Holding Registers* (leitura/escrita)

Códigos de função: `0x03` (leitura), `0x06` (escrita simples), `0x10` (escrita múltipla).

| Endereço | Designação | Tipo | Unidade / valores |
|---------|-------------|------|-----------------|
| `0` | Modo de regulação sentido 1 (aquecimento) | `u16` | `0`=Off, `1`=PID, `2`=TOR, `3`=PWM |
| `1` | Modo de regulação sentido 2 (arrefecimento) | `u16` | `0`=Off, `1`=PID, `2`=TOR, `3`=PWM |
| `2`–`3` | Consigna automática (SP) | `f32` | unidade de medida |
| `4`–`5` | Consigna manual | `f32` | % de saída, com sinal (−100…+100) |
| `6`–`7` | `Kp` sentido 1 | `f32` | ganho proporcional |
| `8`–`9` | `Ki` sentido 1 | `f32` | ganho integral (s⁻¹) |
| `10`–`11` | `Kd` sentido 1 | `f32` | ganho derivado (s) |
| `12`–`13` | `Kp` sentido 2 | `f32` | ganho proporcional |
| `14`–`15` | `Ki` sentido 2 | `f32` | ganho integral (s⁻¹) |
| `16`–`17` | `Kd` sentido 2 | `f32` | ganho derivado (s) |
| `18`–`19` | Histerese TOR | `f32` | unidade de medida |
| `20`–`21` | Tempo de ciclo mínimo TOR | `f32` | segundos (anti-ciclo-curto, `0` = desativado) |
| `22`–`23` | Período do ciclo PWM | `f32` | segundos (> 0) |
| `42`–`46` | Identificador de aparelho | `ASCII` | «CESAM-Lab» (leitura apenas, 2 car./registo, maior peso primeiro) |

> Registos `24`–`41` reservados (lidos a `0`).

> **Escrita parcial de um `f32`**: é preciso escrever **os dois registos** de um
> flutuante para que seja tido em conta. Uma escrita de um só registo de um
> par `f32` é ignorada (e devolve a exceção *Illegal Data Address* se não
> recobrir nenhum campo válido).
>
> Os ganhos escritos são limitados a valores finitos ≥ 0 (robustez).

---

## 6. Registos de entrada — *Input Registers* (leitura apenas)

Código de função: `0x04`.

| Endereço | Designação | Tipo | Unidade |
|---------|-------------|------|-------|
| `0`–`1` | Medida (PV — *process value*) | `f32` | unidade de medida |
| `2`–`3` | Saída aplicada | `f32` | % com sinal (+ aquecimento / − arrefecimento) |
| `4`–`5` | Releitura consigna auto (leitura apenas) | `f32` | unidade de medida |
| `6`–`7` | Releitura consigna manual (leitura apenas) | `f32` | % de saída, com sinal (−100…+100) |

> **Releituras das consignas**: os registos `4`–`7` expõem em **leitura apenas** o
> valor atual das consignas auto/manual (espelhos dos holdings `2`–`5`).
> Prático para um supervisor que apenas **monitoriza** sem escrever.

---

## 7. Exceções Modbus

| Código | Nome | Causa neste aparelho |
|------|-----|--------------------------|
| `0x01` | Illegal Function | Código de função não suportado (ex. máscara, FIFO) |
| `0x02` | Illegal Data Address | Endereço / quantidade fora da tabela, ou escrita que não visa nenhum campo |
| `0x04` | Server Device Failure | Bloqueio interno indisponível (caso anormal) |

---

## 8. Exemplos com `mbpoll`

`mbpoll` endereça em **base 1**; adiciona-se, portanto, `1` aos endereços base 0.

```bash
# Pôr em marcha (bobina base0 0 -> -t 0 -r 1) e depois passar a auto (bobina 1)
mbpoll -m tcp -p 5502 -a 1 -t 0 -r 1 127.0.0.1 1     # On/Off = 1
mbpoll -m tcp -p 5502 -a 1 -t 0 -r 2 127.0.0.1 1     # Auto/Manual = 1 (auto)

# Escrever a consigna auto (HR base0 2-3 -> -t 4:float -r 3) a 80.0
mbpoll -m tcp -p 5502 -a 1 -t 4:float -r 3 127.0.0.1 80.0

# Ler a medida PV (IR base0 0-1 -> -t 3:float -r 1)
mbpoll -m tcp -p 5502 -a 1 -t 3:float -r 1 127.0.0.1

# Ler a saída (IR base0 2-3 -> -t 3:float -r 3)
mbpoll -m tcp -p 5502 -a 1 -t 3:float -r 3 127.0.0.1
```

> Conforme as versões de `mbpoll`, a ordem das palavras flutuantes pode necessitar
> da opção de permutação. Em caso de valor incoerente, verificar a ordem das palavras.

---

## 9. Mapa de memória condensado

```
Coils (RW)            DiscreteInputs (RO)     Holding (RW)              Input (RO)
0  On/Off             0  Em marcha            0  Modo sent1 (u16)       0-1 PV (f32)
1  Auto/Manual        1  Aquec. ativo         1  Modo sent2 (u16)       2-3 Saída (f32)
                      2  Arref. ativo          2-3  SP auto (f32)         4-5 SP auto (releitura, RO)
                                              4-5  SP manual (f32)        6-7 SP manual (releitura, RO)
                                              6-7  Kp1  8-9  Ki1  10-11 Kd1
                                              12-13 Kp2 14-15 Ki2 16-17 Kd2
                                              18-19 Histerese (f32)
                                              20-21 Ciclo mín. TOR (f32, s)
                                              22-23 Período PWM (f32, s)
                                              42-46 Identificador ASCII "CESAM-Lab"
```

> **Identificador ASCII** (`HR 42-46`): «CESAM-Lab» codificado 2 caracteres por
> registo, caractere de maior peso primeiro (`42`=`'CE'`, `43`=`'SA'`, `44`=`'M-'`,
> `45`=`'La'`, `46`=`'b\0'`). Leitura apenas. Exemplo:
> `mbpoll -m tcp -p 5502 -a 1 -t 4 -r 43 -c 5 127.0.0.1` (registos base 1 43..47).
