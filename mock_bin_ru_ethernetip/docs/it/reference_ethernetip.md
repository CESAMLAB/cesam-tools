# Riferimento EtherNet/IP — tag & protocollo (RU/EtherNet/IP)

*🌍 [FR](../fr/reference_ethernetip.md) · [EN](../en/reference_ethernetip.md) · [DE](../de/reference_ethernetip.md) · [ES](../es/reference_ethernetip.md) · **IT** · [PT](../pt/reference_ethernetip.md) · [NL](../nl/reference_ethernetip.md) · [PL](../pl/reference_ethernetip.md)*

> Fonte di verità: [`eip_server.rs`](../../src/eip_server.rs) (incapsulamento,
> dispatch CIP, tabella dei tag). Ogni evoluzione si fa **in questo file** e si
> ripercuote qui.

---

## 1. Endpoint

Adattatore **EtherNet/IP** (messaggistica esplicita **CIP** non connessa) su TCP.
Ascolta di default su `0.0.0.0:44818` (porta standard EtherNet/IP, > 1024 → nessun
privilegio richiesto). Impostazioni nella sezione `[network]` del TOML / nel modale
*Parametri*:

| Chiave | Default | Ruolo |
|---|---|---|
| `bind_ip` | `0.0.0.0` | IP di ascolto |
| `port` | `44818` | porta TCP (EtherNet/IP standard) |
| `allowlist` | *(vuoto)* | lista bianca di IP (motivi `*` per ottetto; vuoto = tutto consentito) |

> ⚠️ **Nessuna autenticazione né cifratura** (EtherNet/IP «classic»). L'unico
> controllo di accesso è la **lista bianca di IP** + la topologia di rete. `0.0.0.0` +
> lista vuota = **esposto**: l'IHM mostra un banner di avvertimento.

⚠️ EtherNet/IP / CIP è **little-endian** (al contrario di Modbus/S7). I `REAL`
sono `f32` IEEE-754 little-endian.

## 2. Sessioni

Più client **simultanei** sono accettati. Ogni sessione: `RegisterSession`
(il server assegna un *session handle* non nullo) → `SendRRData` che porta le richieste
CIP → `UnRegisterSession` (o disconnessione TCP).

## 3. Sottoinsieme di protocollo implementato

- **Incapsulamento**: `RegisterSession` (0x0065), `UnRegisterSession` (0x0066),
  `SendRRData` (0x006F, messaggistica esplicita non connessa, CPF).
- **CIP**: `Read Tag` (servizio 0x4C) e `Write Tag` (servizio 0x4D) su **tag
  nominati** (segmento simbolico ANSI `0x91`).

## 4. Tabella dei tag

| Tag | Tipo CIP | Accesso | Grandezza | Scrittura → comando |
|---|---|:--:|---|---|
| `Setpoint` | REAL (0x00CA) | R/W | setpoint | `SetSetpoint` |
| `ProcessValue` | REAL | R | misura | — |
| `Output` | REAL | R | uscita (%) | — |
| `ManualOutput` | REAL | R/W | uscita manuale (%) | `SetManualOutput` |
| `Run` | BOOL (0x00C1) | R/W | avvio | `SetRun` |
| `Auto` | BOOL | R/W | modalità auto | `SetAuto` |
| `SetpointMin` | REAL | R | setpoint min | — |
| `SetpointMax` | REAL | R | setpoint max | — |
| `Kp` / `Ki` / `Kd` | REAL | R | guadagni PID | — |

Un tag noto in **sola lettura** scritto viene **accettato** (stato CIP successo) ma senza
effetto; un **tag sconosciuto** restituisce lo stato CIP `0x05` (*path destination unknown*).
Ogni scrittura pilotabile è **clampata/sanificata** dalla simulazione.

## 5. Esempio client

Con un client EtherNet/IP (ad es. `pycomm3`, `rseip`, `rust-ethernet-ip`) puntato
sull'IP/porta del server, i tag si leggono/scrivono per nome:

```python
from pycomm3 import CIPDriver  # o LogixDriver a seconda dello strumento
# Leggere la misura, scrivere il setpoint e avviare la regolazione:
#   read  Tag "ProcessValue" (REAL)
#   write Tag "Setpoint" = 80.0 (REAL)
#   write Tag "Run" = True (BOOL)
```

Il server risponde ai servizi generici Read/Write Tag indirizzati per segmento
simbolico ANSI; non espone alcun albero di oggetti CIP oltre ai tag
qui sopra.
