# Riferimento S7 — piano di indirizzamento e protocollo (RU/S7)

*🌍 [FR](../fr/reference_s7.md) · [EN](../en/reference_s7.md) · [DE](../de/reference_s7.md) · [ES](../es/reference_s7.md) · **IT** · [PT](../pt/reference_s7.md) · [NL](../nl/reference_s7.md) · [PL](../pl/reference_s7.md)*

> Fonte di verità: [`s7_server.rs`](../../src/s7_server.rs) (analisi delle trame,
> piano di indirizzamento DB1, mapping delle scritture). Ogni evoluzione si fa **in questo
> file** e si ripercuote qui.

---

## 1. Endpoint

Server **S7comm** su **ISO-on-TCP / RFC1006**. Ascolta in modo predefinito su
`0.0.0.0:102` (porta standard S7; **< 1024 → permessi root** richiesti, altrimenti scegliere una
porta alta). Impostazioni nella sezione `[network]` del TOML / nel modale *Parametri*:

| Chiave | Predefinito | Ruolo |
|---|---|---|
| `bind_ip` | `0.0.0.0` | IP di ascolto |
| `port` | `102` | porta TCP (S7 standard) |
| `allowlist` | *(vuota)* | lista bianca di IP (modelli `*` per byte; vuota = tutto consentito) |

> ⚠️ **Nessuna autenticazione né cifratura** (S7 «classic»). L'unico controllo
> di accesso è la **lista bianca di IP** + la topologia di rete. `0.0.0.0` + lista vuota
> = **esposto a tutta la rete**: l'IHM mostra un banner di avvertimento.

## 2. Sessioni

Al contrario di ORME (mono-master), il server S7 accetta **più sessioni
client simultanee** (comportamento usuale di un PLC). Ogni sessione negozia
COTP (Connection Request → Confirm) poi S7 *Setup Communication*, prima degli
scambi *Read Var* / *Write Var*.

## 3. Sottoinsieme del protocollo implementato

- **COTP**: Connection Request (CR) → Connection Confirm (CC); Data (DT).
- **S7comm**: *Setup Communication*, *Read Var* (funzione `0x04`), *Write Var*
  (funzione `0x05`) sul blocco dati **DB1**.

Il server espone un'**immagine di byte di DB1** (40 byte). Le letture servono
una porzione di questa immagine; le scritture sugli offset pilotabili producono
comandi sanificati per la simulazione.

## 4. Piano di indirizzamento DB1

REAL = `f32` big-endian (IEEE-754). Indirizzamento per byte (`DBDx`) o per bit
(`DBXx.y`).

| Indirizzo | Tipo | Accesso | Grandezza | Scrittura → comando |
|---|---|:--:|---|---|
| `DB1.DBD0`  | REAL | R/W | Setpoint (Setpoint) | `SetSetpoint` |
| `DB1.DBD4`  | REAL | R   | Misura (ProcessValue) | — |
| `DB1.DBD8`  | REAL | R   | Uscita (Output, %) | — |
| `DB1.DBD12` | REAL | R/W | Uscita manuale (ManualOutput, %) | `SetManualOutput` |
| `DB1.DBX16.0` | BOOL | R/W | Marcia (Run) | `SetRun` |
| `DB1.DBX16.1` | BOOL | R/W | Modalità auto (Auto) | `SetAuto` |
| `DB1.DBD20` | REAL | R | Setpoint min | — |
| `DB1.DBD24` | REAL | R | Setpoint max | — |
| `DB1.DBD28` | REAL | R | PID Kp | — |
| `DB1.DBD32` | REAL | R | PID Ki | — |
| `DB1.DBD36` | REAL | R | PID Kd | — |

La scrittura di `DB1.DBB16` (byte) è accettata: bit 0 = Run, bit 1 = Auto. Ogni scrittura
su un offset in sola lettura è **accettata ma ignorata** (codice di ritorno di successo).
Una lettura/scrittura fuori da DB1 restituisce il codice di ritorno S7 `0x0A` (oggetto inesistente).

## 5. Esempio client

Con un client S7 (Snap7, `python-snap7`, nodes7…) configurato sull'IP/porta del
server, **rack 0 / slot 1** (valori usuali; il server non impone il TSAP):

```python
import snap7, struct
c = snap7.client.Client()
c.connect("127.0.0.1", 0, 1, 102)
c.db_write(1, 0, struct.pack(">f", 80.0))   # Setpoint = 80.0
c.db_write(1, 16, bytes([0x01]))            # Run = true (bit 0)
pv = struct.unpack(">f", c.db_read(1, 4, 4))[0]  # Misura
```
