# Manual do utilizador — Regulador Sparkplug B (ORSE)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · **PT** · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

---

## 1. Para que serve o instrumento

O **ORSE** simula uma **unidade de regulação** de processo (PID + processo térmico de
primeira ordem) e publica o seu estado em **MQTT Sparkplug B**, como um **edge node**
que se liga a um **broker** e expõe métricas a um SCADA. Serve para testar
uma cadeia de aquisição Sparkplug B (Ignition, Chariot, EMQX, Node-RED…) sem
hardware real.

## 2. Pré-requisito: um broker MQTT

Sendo o ORSE um **cliente**, é necessário um broker MQTT acessível. Localmente:

```bash
docker run -it --rm -p 1883:1883 eclipse-mosquitto
```

## 3. Primeiros passos

```bash
cargo run -p mock_bin_ru_sparkplugb        # IHM + edge node Sparkplug B
```

No arranque, a IHM tenta ligar-se ao broker (`localhost:1883` por defeito).
O cabeçalho indica o estado: **Ligado** (verde) assim que o `NBIRTH` for publicado, ou
**Desligado** (vermelho) com o motivo. Uma faixa laranja **⚠ MQTT em claro** recorda
a ausência de TLS.

## 4. Interface

- **Cabeçalho**: título, botões *Parâmetros* / *Guardar*, estado ligado/desligado, estado
  de ligação Sparkplug B, faixa TLS/claro.
- **Painel esquerdo (Comandos)**: *Ligar/Desligar*, *Modo automático (PID)*,
  *Consigna*, *Saída manual* (modo manual), ajustes **PID** (Kp/Ki/Kd).
- **Painel central**: cartões *Medida / Consigna / Saída* + **curva** em tempo real.
- **Modal *Parâmetros***: idioma, verificação de atualizações, **Broker MQTT / Sparkplug B**
  (host, porta, client_id, group_id, edge_node_id, keepalive, TLS, utilizador/palavra-
  passe, publicação por alteração/periódica), **processo** (K, τ, atraso, ambiente),
  **limites de consigna**. *Aplicar* relança a ligação e guarda o TOML.

## 5. Comandar a partir de um SCADA

O SCADA subscreve `spBv1.0/<group_id>/#` e recebe `NBIRTH` e depois `NDATA`. Para
**comandar** o regulador, publica um `NCMD` em
`spBv1.0/<group_id>/NCMD/<edge_node_id>` com as métricas comandáveis (`Setpoint`,
`Run`, `Auto`, `ManualOutput`) ou `Node Control/Rebirth = true` para forçar um
renascimento. Detalhes: [`reference_sparkplugb.md`](reference_sparkplugb.md).

## 6. FAQ

- **"Desligado" permanentemente** → broker inacessível: verificar `broker_host`/
  `broker_port`, a firewall, e que o broker está em execução.
- **O SCADA não vê nada** → verificar o `group_id`/`edge_node_id` e a subscrição
  `spBv1.0/<group>/#`; as payloads são **protobuf** (descodificador Sparkplug necessário).
- **As minhas escritas NCMD são ignoradas** → métrica não comandável ou tipo errado (cf.
  tabela das métricas). Apenas `Setpoint`/`Run`/`Auto`/`ManualOutput` e `Rebirth`
  são aceites.
- **Onde está o ficheiro de config?** → `mock_ru_sparkplugb.toml` (diretório atual;
  substituível por `MOCK_CONFIG`).
