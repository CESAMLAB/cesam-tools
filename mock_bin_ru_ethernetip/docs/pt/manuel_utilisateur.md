# Manual do utilizador — Regulador EtherNet/IP (OREE)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · **PT** · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

---

## 1. Para que serve o instrumento

O **OREE** simula uma **unidade de regulação** de processo (PID + processo térmico de
primeira ordem) e expõe-a como um **adaptador EtherNet/IP** (mensagens explícitas
CIP). Serve para testar uma supervisão ou um cliente EtherNet/IP (pycomm3, RSLinx em
leitura, rseip…) sem material real.

## 2. Primeiros passos

```bash
cargo run -p mock_bin_ru_ethernetip        # IHM + adaptateur EtherNet/IP
```

O servidor escuta por defeito em `0.0.0.0:44818` (nenhum privilégio necessário). O cabeçalho
indica o estado: **EtherNet/IP ●** (verde) com o endereço de escuta, ou uma mensagem
de erro (vermelho). Uma faixa laranja avisa se o servidor estiver **exposto** (todas as
interfaces + lista branca vazia).

## 3. Interface

- **Cabeçalho**: título, botões *Parâmetros* / *Guardar*, estado em funcionamento/paragem, estado
  de escuta EtherNet/IP, faixa de exposição de rede.
- **Painel esquerdo (Comandos)**: *Funcionamento/Paragem*, *Modo automático (PID)*,
  *Consigna*, *Saída manual* (modo manual), ajustes **PID** (Kp/Ki/Kd).
- **Painel central**: cartões *Medida / Consigna / Saída* + **curva** em tempo real.
- **Modal *Parâmetros***: idioma, verificação de atualizações, **rede EtherNet/IP** (IP
  de escuta, porta, **lista branca** de IP — um padrão por linha, `*` = curinga),
  **processo** (K, τ, atraso, ambiente), **limites de consigna**. *Aplicar* reinicia
  a escuta se o IP/porta mudar e guarda o TOML.

## 4. Ligar um cliente EtherNet/IP

O cliente liga-se ao IP/porta do servidor (`RegisterSession` automático), depois
lê/escreve os **tags nomeados** por mensagens explícitas: `Setpoint`, `ProcessValue`,
`Output`, `ManualOutput`, `Run`, `Auto`, etc. (ver
[`reference_ethernetip.md`](reference_ethernetip.md)). ⚠️ Os valores estão em
**little-endian** (REAL = `f32` LE).

## 5. FAQ

- **O cliente não se liga** → verificar IP/porta (44818), a **lista branca**,
  a firewall.
- **Tag não encontrado** → apenas os tags documentados existem; os nomes são
  sensíveis a maiúsculas/minúsculas.
- **As minhas escritas não têm efeito** → apenas os tags pilotáveis atuam
  (`Setpoint`, `ManualOutput`, `Run`, `Auto`); os outros são só de leitura.
- **Onde está o ficheiro de configuração?** → `mock_ru_ethernetip.toml` (diretório atual;
  substituível por `MOCK_CONFIG`).
