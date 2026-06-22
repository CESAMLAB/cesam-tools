# Set di comandi NAMUR — Agitatore simulato (OSNE)

*🌍 [FR](../fr/commandes_namur.md) · [EN](../en/commandes_namur.md) · [DE](../de/commandes_namur.md) · [ES](../es/commandes_namur.md) · **IT** · [PT](../pt/commandes_namur.md) · [NL](../nl/commandes_namur.md) · [PL](../pl/commandes_namur.md)*

> Crate: `mock_bin_su_namur` · Eseguibile: **OSNE** · Protocollo: **NAMUR** (ASCII, slave)

Riferimento funzionale del protocollo. La **fonte di verità tecnica** è
l'intestazione di [`src/namur.rs`](../../src/namur.rs).

---

## 1. Generalità

| Elemento | Valore |
|---------|--------|
| Trasporto | **TCP** (porta `4001` predefinita) o **seriale RS-232** (feature `serial`) |
| Ruolo | **Slave** (risponde alle richieste del master) |
| Trama | una **riga ASCII** per richiesta, terminata da `CR LF` |
| Letture | `IN_*` → restituiscono `valore canale` (es. `1200.0 4`) |
| Scritture / azioni | `OUT_*`, `START_*`, `STOP_*`, `RESET` → **silenziose** (nessuna risposta) |
| Master | **uno solo alla volta** (punto-punto); in TCP un nuovo master attende fino alla disconnessione del precedente |
| Filtraggio | lista bianca di IP opzionale (TCP) |

> Impostazione seriale NAMUR tipica: **9600 baud, 7 bit, parità pari, 1 stop (7E1)**.

### Canali

| Canale | Grandezza | Unità |
|-------|----------|-------|
| `4` | Velocità | tr/min |
| `5` | Coppia | N·cm |

---

## 2. Comandi

| Comando | Tipo | Effetto | Risposta |
|----------|------|-------|---------|
| `IN_NAME` | lettura | Nome dell'apparecchio | `CESAM-STIRRER` |
| `IN_TYPE` | lettura | Tipo di apparecchio | `OSNE` |
| `IN_SW_VERSION` | lettura | Versione del firmware simulato | es. `0.1.0` |
| `IN_PV_4` | lettura | Velocità **misurata** | `<v> 4` |
| `IN_PV_5` | lettura | Coppia **misurata** | `<c> 5` |
| `IN_SP_4` | lettura | Riferimento di velocità | `<v> 4` |
| `OUT_SP_4 <v>` | scrittura | **Impostare** il riferimento di velocità (tr/min) | — |
| `START_4` | azione | Avviare il motore | — |
| `STOP_4` | azione | Arrestare il motore | — |
| `RESET` | azione | Arresto + ritorno al comando locale | — |
| `OUT_WD1@<m>` | scrittura | **Watchdog**: arresto sicuro se nessun comando entro `<m>` s | — |
| `OUT_WD2@<m>` | scrittura | Watchdog (idem v1: arresto sicuro) | — |

> Ogni comando sconosciuto o argomento non valido è **ignorato** (nessuna
> risposta) e registrato a livello `debug`.

### Watchdog

Dopo `OUT_WD1@30`, se **nessuna riga** arriva per 30 s, il motore viene
**arrestato** (`STOP`) automaticamente — protezione in caso di perdita di
comunicazione con il supervisore. `OUT_WD1@0` disarma il watchdog. Il contatore è
**ri-armato a ogni comando ricevuto**.

---

## 3. Esempi (`nc` / netcat)

```text
$ nc 127.0.0.1 4001
IN_NAME
CESAM-STIRRER
OUT_SP_4 1200          (silenzioso)
START_4                (silenzioso)
IN_PV_4
1200.0 4
IN_PV_5
62.0 5
STOP_4                 (silenzioso)
```

> La **coppia** letta cresce con la **viscosità** impostata (lato IHM) e la
> velocità: `coppia ≈ coeff_carico · viscosità · velocità + attrito`. Ad alta
> viscosità, la coppia satura al massimo del motore: la velocità di riferimento non
> viene più raggiunta (**sovraccarico**), comportamento che riproduce un agitatore
> reale.
