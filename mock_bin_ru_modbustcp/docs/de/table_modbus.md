# Modbus-Adresstabelle — Simulierter Regler

*🌍 [FR](../fr/table_modbus.md) · [EN](../en/table_modbus.md) · **DE** · [ES](../es/table_modbus.md) · [IT](../it/table_modbus.md) · [PT](../pt/table_modbus.md) · [NL](../nl/table_modbus.md) · [PL](../pl/table_modbus.md)*

> Crate: `mock_bin_ru_modbustcp` · Protokoll: **Modbus TCP** (Slave / Server)

Dieses Dokument ist die funktionale Referenz des Adressplans. Die **technische
Quelle der Wahrheit** bleibt der Kopf von [`src/map.rs`](../../src/map.rs): Jede
Abweichung muss vorrangig im Code korrigiert werden.

---

## 1. Allgemeines

| Element | Wert |
|---------|------|
| Transport | Modbus **TCP** oder **seriell RTU / RS485** (nur eines gleichzeitig aktiv) |
| Rolle | **Slave** (Server) |
| Standard-Port | TCP `5502` (konfigurierbar, Modal *Parameter*) |
| Seriell (RTU) | Port + Baud + Parität + Bits, konfigurierbar (Feature `rtu`) |
| Unit ID / Adresse | TCP: gleichgültig. RTU: `slave_id` konfigurierbar, aber **nicht gefiltert** (siehe Hinweis) |
| Master | **nur ein entfernter Master gleichzeitig**; in TCP trennt ein Neuankömmling den vorherigen (die lokale IHM ist kein Master) |
| Adressierung | **Basis 0** (Adresse `0` = 1. Element der Tabelle) |
| Filterung | optionale IP-Whitelist (Joker `*`, nur TCP) |

> **Hinweis RTU / Slave-Adresse**: Der RTU-Server antwortet **unabhängig von der
> angeforderten Adresse** (die Adresse wird nicht an den Anwendungsdienst
> übermittelt). Eine **Punkt-zu-Punkt-Verbindung** verwenden. Die `slave_id` wird
> beibehalten/angezeigt, führt aber keine Filterung durch.

### Adressierung Basis 0 vs. Basis 1

Die nachstehenden Adressen sind die **Protokolladressen (Basis 0)**, so wie sie im
Telegramm gesendet werden. Viele Werkzeuge zeigen eine „konventionelle"
Basis-1-Nummerierung an (`4xxxx` für Holdings, `3xxxx` für Inputs…). So entspricht
das Holding-Register Adresse `2` dem konventionellen Bezug `40003`.

---

## 2. Kodierung der Gleitkommazahlen (`f32`)

Die Analoggrößen sind **`f32` IEEE-754 auf 2 aufeinanderfolgenden Registern**:

- **Wortreihenfolge**: **höchstwertiges Wort zuerst** (big-endian, genannt *ABCD*);
- **Byte-Reihenfolge** in jedem Register: big-endian (Modbus-Standard).

Beispiel: `80.0` → Bytes `42 A0 00 00` → Register `n` = `0x42A0`,
Register `n+1` = `0x0000`.

> Wenn Ihr Client unsinnige Werte liest, ist es fast immer ein Problem der
> Wortreihenfolge (*word swap* / *CDAB* versuchen).

---

## 3. Spulen — *Coils* (Lesen/Schreiben)

Funktionscodes: `0x01` (Lesen), `0x05` (Einzelschreiben), `0x0F` (Mehrfachschreiben).

| Adresse | Bezeichnung | Werte | Wirkung |
|---------|-------------|-------|---------|
| `0` | Start / Stopp | `0` = Stopp, `1` = Start | Aktiviert die Regelung |
| `1` | Auto / Manuell | `0` = manuell, `1` = auto | Moduswahl |

---

## 4. Diskrete Eingänge — *Discrete Inputs* (nur Lesen)

Funktionscode: `0x02`.

| Adresse | Bezeichnung | Bedeutung |
|---------|-------------|-----------|
| `0` | In Betrieb | Das Gerät ist in Betrieb |
| `1` | Richtung 1 (heiß) aktiv | Ausgang > 0 |
| `2` | Richtung 2 (kalt) aktiv | Ausgang < 0 |

---

## 5. Halteregister — *Holding Registers* (Lesen/Schreiben)

Funktionscodes: `0x03` (Lesen), `0x06` (Einzelschreiben), `0x10` (Mehrfachschreiben).

| Adresse | Bezeichnung | Typ | Einheit / Werte |
|---------|-------------|-----|-----------------|
| `0` | Regelungsmodus Richtung 1 (heiß) | `u16` | `0`=Off, `1`=PID, `2`=TOR, `3`=PWM |
| `1` | Regelungsmodus Richtung 2 (kalt) | `u16` | `0`=Off, `1`=PID, `2`=TOR, `3`=PWM |
| `2`–`3` | Automatischer Sollwert (SP) | `f32` | Messeinheit |
| `4`–`5` | Manueller Sollwert | `f32` | % Ausgang, vorzeichenbehaftet (−100…+100) |
| `6`–`7` | `Kp` Richtung 1 | `f32` | Proportionalverstärkung |
| `8`–`9` | `Ki` Richtung 1 | `f32` | Integralverstärkung (s⁻¹) |
| `10`–`11` | `Kd` Richtung 1 | `f32` | Differenzialverstärkung (s) |
| `12`–`13` | `Kp` Richtung 2 | `f32` | Proportionalverstärkung |
| `14`–`15` | `Ki` Richtung 2 | `f32` | Integralverstärkung (s⁻¹) |
| `16`–`17` | `Kd` Richtung 2 | `f32` | Differenzialverstärkung (s) |
| `18`–`19` | TOR-Hysterese | `f32` | Messeinheit |
| `20`–`21` | Minimale TOR-Zykluszeit | `f32` | Sekunden (Anti-Kurzzyklus, `0` = deaktiviert) |
| `22`–`23` | PWM-Zyklusperiode | `f32` | Sekunden (> 0) |
| `42`–`46` | Gerätekennung | `ASCII` | „CESAM-Lab" (nur Lesen, 2 Zeichen/Register, höchstwertiges zuerst) |

> Register `24`–`41` reserviert (als `0` gelesen).

> **Partielles Schreiben eines `f32`**: Es müssen **beide Register** eines
> Gleitkommawerts geschrieben werden, damit er berücksichtigt wird. Ein
> Schreibvorgang eines einzelnen Registers eines `f32`-Paars wird ignoriert (und
> gibt die Ausnahme *Illegal Data Address* zurück, falls er kein gültiges Feld abdeckt).
>
> Die geschriebenen Verstärkungen werden auf endliche Werte ≥ 0 begrenzt (Robustheit).

---

## 6. Eingangsregister — *Input Registers* (nur Lesen)

Funktionscode: `0x04`.

| Adresse | Bezeichnung | Typ | Einheit |
|---------|-------------|-----|---------|
| `0`–`1` | Messwert (PV — *process value*) | `f32` | Messeinheit |
| `2`–`3` | Angewandter Ausgang | `f32` | % vorzeichenbehaftet (+ heiß / − kalt) |
| `4`–`5` | Rücklesung Auto-Sollwert (nur Lesen) | `f32` | Messeinheit |
| `6`–`7` | Rücklesung Manueller Sollwert (nur Lesen) | `f32` | % Ausgang, vorzeichenbehaftet (−100…+100) |

> **Sollwert-Rücklesungen**: Die Register `4`–`7` stellen den aktuellen Wert der
> Auto-/Manuell-Sollwerte **nur lesend** bereit (Spiegel der Halteregister `2`–`5`).
> Praktisch für ein Leitsystem, das nur **überwacht**, ohne zu schreiben.

---

## 7. Modbus-Ausnahmen

| Code | Name | Ursache in diesem Gerät |
|------|------|--------------------------|
| `0x01` | Illegal Function | Nicht behandelter Funktionscode (z. B. Maske, FIFO) |
| `0x02` | Illegal Data Address | Adresse / Menge außerhalb der Tabelle oder Schreibvorgang, der kein Feld trifft |
| `0x04` | Server Device Failure | Interne Sperre nicht verfügbar (anormaler Fall) |

---

## 8. Beispiele mit `mbpoll`

`mbpoll` adressiert in **Basis 1**; man addiert daher `1` zu den Basis-0-Adressen.

```bash
# Starten (Coil base0 0 -> -t 0 -r 1), dann auf auto schalten (Coil 1)
mbpoll -m tcp -p 5502 -a 1 -t 0 -r 1 127.0.0.1 1     # On/Off = 1
mbpoll -m tcp -p 5502 -a 1 -t 0 -r 2 127.0.0.1 1     # Auto/Manuell = 1 (auto)

# Auto-Sollwert schreiben (HR base0 2-3 -> -t 4:float -r 3) auf 80.0
mbpoll -m tcp -p 5502 -a 1 -t 4:float -r 3 127.0.0.1 80.0

# Messwert PV lesen (IR base0 0-1 -> -t 3:float -r 1)
mbpoll -m tcp -p 5502 -a 1 -t 3:float -r 1 127.0.0.1

# Ausgang lesen (IR base0 2-3 -> -t 3:float -r 3)
mbpoll -m tcp -p 5502 -a 1 -t 3:float -r 3 127.0.0.1
```

> Je nach `mbpoll`-Version kann die Wortreihenfolge der Gleitkommawerte die
> Vertauschungsoption erfordern. Bei inkonsistentem Wert die Wortreihenfolge prüfen.

---

## 9. Verdichtete Speicherkarte

```
Coils (RW)            DiscreteInputs (RO)     Holding (RW)              Input (RO)
0  On/Off             0  In Betrieb           0  Modus Richt.1 (u16)    0-1 PV (f32)
1  Auto/Manuell       1  Heiß aktiv           1  Modus Richt.2 (u16)    2-3 Ausgang (f32)
                      2  Kalt aktiv           2-3  SP auto (f32)         4-5 SP auto (Rücklesung, RO)
                                              4-5  SP manuell (f32)       6-7 SP manuell (Rücklesung, RO)
                                              6-7  Kp1  8-9  Ki1  10-11 Kd1
                                              12-13 Kp2 14-15 Ki2 16-17 Kd2
                                              18-19 Hysterese (f32)
                                              20-21 Min. Zyklus TOR (f32, s)
                                              22-23 PWM-Periode (f32, s)
                                              42-46 ASCII-Kennung "CESAM-Lab"
```

> **ASCII-Kennung** (`HR 42-46`): „CESAM-Lab" kodiert mit 2 Zeichen pro Register,
> höchstwertiges Zeichen zuerst (`42`=`'CE'`, `43`=`'SA'`, `44`=`'M-'`,
> `45`=`'La'`, `46`=`'b\0'`). Nur Lesen. Beispiel:
> `mbpoll -m tcp -p 5502 -a 1 -t 4 -r 43 -c 5 127.0.0.1` (Register Basis 1 43..47).
