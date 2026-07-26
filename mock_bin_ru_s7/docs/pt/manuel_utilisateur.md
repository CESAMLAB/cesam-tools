# Manual do utilizador — Regulador S7 (ORSS)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · **PT** · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

---

## 1. Para que serve o instrumento

O **ORSS** simula uma **unidade de regulação** de processo (PID + processo térmico de
primeira ordem) e expõe-na como um **autómato Siemens S7** (servidor S7comm sobre
ISO-on-TCP). Serve para testar uma supervisão ou um cliente S7 (Snap7, TIA Portal em
leitura, nodes7…) sem autómato real.

## 2. Primeiros passos

```bash
cargo run -p mock_bin_ru_s7        # IHM + servidor S7
```

O servidor escuta por defeito em `0.0.0.0:102`. ⚠️ A **porta 102 requer os
direitos root**; caso contrário, defina uma porta alta (ex. 1102) no modal *Parâmetros*.

O cabeçalho indica o estado: **S7 ●** (verde) com o endereço de escuta, ou uma mensagem
de erro (vermelho) se o bind falhar. Uma faixa laranja avisa se o servidor estiver
**exposto** (todas as interfaces + lista branca vazia).

## 3. Interface

- **Cabeçalho**: título, botões *Parâmetros* / *Guardar*, estado funcionamento/paragem, estado
  de escuta S7, faixa de exposição de rede.
- **Painel esquerdo (Comandos)**: *Funcionamento/Paragem*, *Modo automático (PID)*,
  *Setpoint*, *Saída manual* (modo manual), ajustes **PID** (Kp/Ki/Kd).
- **Painel central**: cartões *Medida / Setpoint / Saída* + **curva** em tempo real.
- **Modal *Parâmetros***: idioma, verificação de atualizações, **rede S7** (IP de escuta,
  porta, **lista branca** de IP — um padrão por linha, `*` = wildcard), **processo**
  (K, τ, atraso, ambiente), **limites de setpoint**. *Aplicar* relança a escuta se
  o IP/porta mudar e guarda o TOML.

## 4. Ligar um cliente S7

O cliente liga-se ao IP/porta do servidor. Os valores **rack/slot** habituais
(0/1 ou 0/2) funcionam: o servidor não impõe TSAP. As grandezas estão em
**DB1** (ver [`reference_s7.md`](reference_s7.md)): setpoint em `DB1.DBD0`, medida
em `DB1.DBD4`, funcionamento em `DB1.DBX16.0`, etc.

## 5. FAQ

- **"Permission denied" no arranque** → a porta 102 exige os direitos root;
  utilize uma porta alta ou lance com os privilégios adequados.
- **O cliente não se liga** → verificar IP/porta, a **lista branca**, a
  firewall. Testar rack/slot 0/1 e depois 0/2.
- **As minhas escritas não têm efeito** → apenas os offsets controláveis atuam
  (setpoint, saída manual, funcionamento, auto); os restantes são apenas de leitura.
- **Onde está o ficheiro de configuração?** → `mock_ru_s7.toml` (diretório atual;
  substituível por `MOCK_CONFIG`).
