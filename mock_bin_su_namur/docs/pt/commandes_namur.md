# Conjunto de comandos NAMUR — Agitador simulado (OSNE)

*🌍 [FR](../fr/commandes_namur.md) · [EN](../en/commandes_namur.md) · [DE](../de/commandes_namur.md) · [ES](../es/commandes_namur.md) · [IT](../it/commandes_namur.md) · **PT** · [NL](../nl/commandes_namur.md) · [PL](../pl/commandes_namur.md)*

> Crate: `mock_bin_su_namur` · Executável: **OSNE** · Protocolo: **NAMUR** (ASCII, escravo)

Referência funcional do protocolo. A **fonte de verdade técnica** é o cabeçalho
de [`src/namur.rs`](../../src/namur.rs).

---

## 1. Generalidades

| Elemento | Valor |
|---------|--------|
| Transporte | **TCP** (porta `4001` por defeito) ou **série RS-232** (feature `serial`) |
| Papel | **Escravo** (responde aos pedidos do mestre) |
| Trama | uma **linha ASCII** por pedido, terminada por `CR LF` |
| Leituras | `IN_*` → devolvem `valor canal` (ex. `1200.0 4`) |
| Escritas / ações | `OUT_*`, `START_*`, `STOP_*`, `RESET` → **silenciosas** (sem resposta) |
| Mestres | **um único de cada vez** (ponto-a-ponto); em TCP um novo mestre aguarda até à desconexão do anterior |
| Filtragem | lista branca de IP opcional (TCP) |

> Configuração série NAMUR típica: **9600 bauds, 7 bits, paridade par, 1 stop (7E1)**.

### Canais

| Canal | Grandeza | Unidade |
|-------|----------|-------|
| `4` | Velocidade | tr/min |
| `5` | Binário | N·cm |

---

## 2. Comandos

| Comando | Tipo | Efeito | Resposta |
|----------|------|-------|---------|
| `IN_NAME` | leitura | Nome do aparelho | `CESAM-STIRRER` |
| `IN_TYPE` | leitura | Tipo de aparelho | `OSNE` |
| `IN_SW_VERSION` | leitura | Versão do firmware simulado | ex. `0.1.0` |
| `IN_PV_4` | leitura | Velocidade **medida** | `<v> 4` |
| `IN_PV_5` | leitura | Binário **medido** | `<c> 5` |
| `IN_SP_4` | leitura | Consigna de velocidade | `<v> 4` |
| `OUT_SP_4 <v>` | escrita | **Definir** a consigna de velocidade (tr/min) | — |
| `START_4` | ação | Arrancar o motor | — |
| `STOP_4` | ação | Parar o motor | — |
| `RESET` | ação | Paragem + regresso ao comando local | — |
| `OUT_WD1@<m>` | escrita | **Cão de guarda**: paragem segura se nenhum comando durante `<m>` s | — |
| `OUT_WD2@<m>` | escrita | Cão de guarda (idem v1: paragem segura) | — |

> Qualquer comando desconhecido ou argumento inválido é **ignorado** (sem resposta)
> e registado em `debug`.

### Cão de guarda

Após `OUT_WD1@30`, se **nenhuma linha** chegar durante 30 s, o motor é **parado**
(`STOP`) automaticamente — proteção em caso de perda de comunicação com o
supervisor. `OUT_WD1@0` desarma o cão de guarda. O contador é **rearmado a cada
comando recebido**.

---

## 3. Exemplos (`nc` / netcat)

```text
$ nc 127.0.0.1 4001
IN_NAME
CESAM-STIRRER
OUT_SP_4 1200          (silencieux)
START_4                (silencieux)
IN_PV_4
1200.0 4
IN_PV_5
62.0 5
STOP_4                 (silencieux)
```

> O **binário** lido cresce com a **viscosidade** definida (no lado da IHM) e com a
> velocidade: `couple ≈ coeff_charge · viscosité · vitesse + frottement`. Com
> viscosidade elevada, o binário satura no máximo do motor: a velocidade de consigna
> deixa de ser atingida (**sobrecarga**), comportamento que reproduz um agitador
> real.
