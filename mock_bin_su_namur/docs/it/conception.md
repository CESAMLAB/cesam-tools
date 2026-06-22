# Progettazione — Agitatore da laboratorio simulato (OSNE)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · **IT** · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

> Crate: `mock_bin_su_namur` · Eseguibile: **OSNE** (*Open Stirrer NAMUR Emulator*)

Documento di architettura e modellazione. Ricalcato sul regolatore **ORME**
(`mock_bin_ru_modbustcp`): stessa suddivisione **modello di dominio sincrono /
attori ractor / livello protocollo / IHM egui**, stessi invarianti.

---

## 1. Obiettivo

Simulare un **agitatore da laboratorio** (tipo IKA) pilotato dal protocollo
seriale **NAMUR**. Il motore possiede una **funzione di trasferimento** (dinamica
di velocità) asservita da una **regolazione rapida**, e la **viscosità** del mezzo
è regolabile e influisce sulla coppia.

---

## 2. Modello fisico

### Motore ([`motor.rs`](../../src/motor.rs))

Bilancio delle coppie, integrato per Eulero esplicito:

```text
J · dω/dt = T_moteur − k · η · ω − T_frottement
```

- `ω`: velocità (tr/min);
- `T_moteur`: coppia motore (comando, N·cm, ≥ 0);
- `k · η · ω`: **coppia di carico viscoso** (∝ viscosità `η` e velocità);
- `T_frottement`: attrito secco residuo;
- `J` (`inertia`): regola la **reattività** (piccolo ⇒ rapido).

A regime stabilizzato, `T_moteur = k·η·ω + T_frottement`: la coppia necessaria per
mantenere una velocità **cresce con la viscosità**. Se questa coppia supera la
**coppia massima**, il riferimento non è più raggiungibile → **sovraccarico**.

### Asservimento ([`stirrer.rs`](../../src/stirrer.rs))

Un **PID** ([`mock_lib_control::Pid`], riutilizzato da ORME) prende l'errore di
velocità `riferimento − misura` e produce la **coppia motore**, limitata a
`[0, couple_max]`. I guadagni predefiniti sono volutamente «duri»: l'uscita satura
alla coppia massima finché l'errore è grande (salita rapida), poi il termine
integrale stabilizza. Il passo di simulazione è di **20 ms** (50 Hz), più fine di
quello di ORME perché la dinamica di un motore è rapida.

---

## 3. Architettura (attori)

```
IHM (egui) ──Command(cast)──►  SimulationActor ──refresh──► SharedSnapshot ──► IHM
Server NAMUR ──Command(cast)─►   (Stirrer)     ──refresh──► SharedSnapshot ──► letture NAMUR
```

- **`SimulationActor`** ([`actors/simulation.rs`](../../src/actors/simulation.rs)):
  proprietario unico dello `Stirrer`; fa avanzare la simulazione su un timer
  one-shot ri-armato (nessun timer staccato) e pubblica uno `SharedSnapshot`.
- **`NamurServerActor`** ([`actors/network.rs`](../../src/actors/network.rs)):
  possiede il server NAMUR, ri-avviabile a caldo (`Reconfigure`); lista bianca di
  IP condivisa; stato di ascolto pubblicato per l'IHM.
- **Server NAMUR** ([`namur_server.rs`](../../src/namur_server.rs)): legge le righe
  ASCII, le interpreta ([`namur.rs`](../../src/namur.rs)), risponde alle letture e
  inoltra le scritture/azioni all'attore. **Un master alla volta**
  (punto-punto). **Watchdog** per sessione.

Le letture NAMUR attingono dallo `SharedSnapshot` (nessuna tabella di memoria
separata come il Modbus di ORME: il protocollo NAMUR è orientato ai «comandi», non
ai «registri»).

---

## 4. Configurazione e sicurezza

- `AppConfig` (lingua / rete-seriale / motore / regolazione) serializzata in
  **TOML** ([`config.rs`](../../src/config.rs)), **sanificata al caricamento**
  (`AppConfig::sanitized`: limiti ordinati, float finiti) — invariante condiviso
  con ORME (non eseguire mai `clamp` con limiti non validati).
- NAMUR non ha **né autenticazione né cifratura**: rete di fiducia + lista bianca
  di IP (TCP). Predefinito `0.0.0.0` + lista vuota ⇒ esposto: l'IHM mostra un
  **banner di avvertimento**.

---

## 5. Piste di evoluzione

- Senso di rotazione (CW/CCW) e rampa di accelerazione.
- Sensore di temperatura (`IN_PV_2/3`) se viene aggiunto un modello termico.
- Coppia di carico non lineare (regime turbolento ∝ ω²).
- Promozione del modello motore in `mock_lib_control` se serve a un secondo
  strumento.
