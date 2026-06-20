# Entwurfsdokument — Simulierter Modbus-TCP-Regler

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · **DE** · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

> Produkt: **ORME** · Crate: `mock_bin_ru_modbustcp` · Workspace: `cesam-tools` · Lizenz: MIT

Dieses Dokument beschreibt die Architektur, die technischen Entscheidungen und die
Funktionsprinzipien des simulierten Industriereglers. Es richtet sich an
Entwickler, die das Projekt pflegen oder erweitern.

---

## 1. Ziel und Umfang

Bereitstellung eines **virtuellen Industrieinstruments**: ein Prozessregler, der
sich realistisch verhält und über **Modbus TCP** (Slave) kommuniziert, um
Leitsysteme / SPSen / Gateways **ohne Hardware** zu entwickeln und zu testen.

Der Simulator deckt ab:

- einen **physikalischen Prozess**, modelliert durch eine Übertragungsfunktion;
- eine **bidirektionale Regelung** (Heizen / Kühlen): PID, Zweipunkt (TOR) oder
  Taktrelais (PWM);
- eine **Modbus-TCP-Schnittstelle**, die den vollständigen Zustand bereitstellt;
- eine **Bedienoberfläche** für Steuerung, Visualisierung und Parametrierung;
- die **Persistenz** der Parameter.

Außerhalb des aktuellen Umfangs: Modbus RTU, Redundanz, Langzeit-Historisierung,
starke Authentifizierung (es wird nur eine IP-Whitelist bereitgestellt).

---

## 2. Überblick

```
┌──────────────────────────────────────────────────────────────────────┐
│                       Prozess (Haupt-Thread)                           │
│                                                                        │
│   ┌─────────────────────────┐         liest (Mutex)                    │
│   │   IHM  egui / eframe     │◄──────────────── SharedSnapshot         │
│   │   (gui.rs)               │◄──────────────── SharedStatus           │
│   └───────────┬─────────────┘                                          │
│               │ cast (nicht blockierend)                               │
└───────────────┼────────────────────────────────────────────────────────┘
                │
   ┌────────────┼──────────── Tokio-Runtime (Hintergrund-Threads) ───────┐
   │            ▼                                                         │
   │   ┌──────────────────┐  refresh  ┌──────────────┐                   │
   │   │ SimulationActor   ├──────────►│ SharedSnapshot│ (IHM)            │
   │   │  (ractor)         ├──────────►│ SharedMap     │ (Modbus)         │
   │   │  besitzt den       │           └──────┬───────┘                  │
   │   │  Regulator         │◄── Command ──┐    │ liest                   │
   │   └──────────────────┘              │    ▼                          │
   │          ▲ Command (cast)            │  ┌──────────────────────┐     │
   │          │                           └──┤ RegulatorService      │     │
   │   ┌──────┴───────────┐  verwaltet/rebind│ (Trait Service)       │     │
   │   │ ModbusServerActor ├─────────────────►  Modbus-TCP-Server    │◄──── Clients
   │   │  (ractor)         │  IP-Filter ──────► (tokio-modbus)        │     │
   │   └──────────────────┘   (SharedAllowlist)└──────────────────────┘     │
   └────────────────────────────────────────────────────────────────────┘
```

Leitprinzip: **ein einziger Eigentümer des Geschäftszustands**. Der `Regulator`
wird niemals geteilt; er lebt im `SimulationActor`. Alle Schreibvorgänge
(IHM oder Modbus) sind `Command`-**Nachrichten**. Die Lesevorgänge erfolgen auf
**Kopien**, die bei jedem Schritt aktualisiert werden (`SharedSnapshot`,
`SharedMap`), wodurch Sperren auf der Logik und Wettlaufsituationen entfallen.

---

## 3. Technische Entscheidungen

| Bedarf | Wahl | Begründung |
|--------|------|------------|
| Nebenläufigkeit | **`ractor`** (Aktoren) auf **Tokio** | Isoliert den veränderbaren Zustand in einem Aktor; Mutationen über Nachrichten serialisiert, ohne Anwendungssperre. Projektpräferenz. |
| Modbus-TCP-Slave | **`tokio-modbus`** (`tcp-server`) | Ausgereifte async-Implementierung; das Trait `Service` bildet Anfrage→Antwort sauber ab. |
| IHM | **`egui` / `eframe`** + `egui_plot` | Immediate-Mode, plattformübergreifend, ohne komplexen UI-Zustand zu synchronisieren. |
| Prozess | **FOPDT** (1. Ordnung + Totzeit) | Standardmodell und ausreichend für einen thermischen Prozess; wenige Parameter, intuitiv. |
| Persistenz | **`serde` + `toml`** | Lesbares/von Hand editierbares Format, ideal für Geräteparameter. |

### Warum synchrone und asynchrone Logik getrennt werden

`mock_lib_control` und `regulator.rs` sind **rein synchron** (keine IO, kein
async). Vorteile: deterministisch unit-testbar, durch andere Instrumente
wiederverwendbar und gut zu lesen. Das Asynchrone bleibt auf die **Aktoren** und
die **Netzwerkschicht** beschränkt.

---

## 4. Datenmodell

### Geschäftszustand (`regulator.rs`)

- `Regulator` — besitzendes Aggregat: Modi, Sollwerte, Regler (`Pid`,
  `OnOff`) und Prozess (`FirstOrderProcess`). Nicht `Clone`, nicht geteilt.
- `RegulatorConfig` — statische Konfiguration (Prozess, Verstärkungen, Grenzen, `dt`).
  **Einzige Quelle** der Standardwerte (die TOML-Konfiguration leitet sich daraus ab).
- `RegulatorSnapshot` — **unveränderliche Kopie** (`Copy`) des beobachtbaren
  Zustands, bei jedem Schritt veröffentlicht. Das ist der Lesevertrag für die IHM
  und die Modbus-Tabelle.
- `Command` — Aufzählung der möglichen Mutationen (Betrieb, Modus, Sollwerte,
  Einstellungen, Prozess, Grenzen).

### Geteilte Strukturen (`actors/mod.rs`, `config.rs`)

| Typ | Inhalt | Geschrieben von | Gelesen von |
|-----|--------|-----------------|-------------|
| `SharedSnapshot` | typisierter `RegulatorSnapshot` | SimulationActor | IHM |
| `SharedMap` | `MemoryMap` (Abbilder der 4 Modbus-Tabellen) | SimulationActor | RegulatorService |
| `SharedAllowlist` | `IpFilter` | ModbusServerActor | Annahme von Verbindungen |
| `SharedStatus` | `ServerStatus` (lauscht / Fehler) | ModbusServerActor | IHM |

Alle sind `Arc<Mutex<…>>`: **kurze** kritische Abschnitte (Kopie / refresh),
niemals während einer Berechnung oder einer IO gehalten.

---

## 5. Komponenten

### 5.1 `mock_lib_control` (Bibliothek)

- `Pid` — zeitdiskreter PID, Ableitung auf den Fehler, **Anti-Windup** durch
  Begrenzung des Integralterms. API: `step(sp, pv, dt)` oder `step_with_error(err, dt)`
  (für die Kühlrichtung wiederverwendet).
- `OnOff` — Zweipunkt mit **symmetrischer Hysterese** (Totzone) **und
  Anti-Kurzzyklus**: eine minimale Zyklusdauer (`min_cycle`, s) verbietet jede
  Umschaltung, solange das Relais nicht lange genug in seinem Zustand verharrt hat,
  was den Schutz eines realen Stellglieds modelliert. Das Relais **rastet** seinen
  Zustand ein: Es ist der Aufrufer, der ihm den vorzeichenbehafteten Fehler
  übergeben muss, ohne es beim Vorzeichenwechsel zurückzusetzen (siehe § 5.2).
- `Pwm` — Pulsweitenmodulator (**Taktrelais** /
  *time-proportioning*): Über eine feste Periode `T_c` ist der Zweipunkt-Ausgang
  während des Anteils `duty` des Zyklus aktiv (`duty` **einmal pro Zyklus
  abgetastet**, um eine Verzerrung im eingeschwungenen Zustand zu vermeiden).
  Ermöglicht das feine Regeln eines TOR-Organs.
- `FirstOrderProcess` — Übertragungsfunktion `K·e^(-L·s)/(1+T·s)`,
  Euler-Integration + Totzeitleitung. `reconfigure(...)` ändert die Parameter ohne Sprung.
- `ControllerKind` — `Off` / `Pid` / `OnOff` / `Pwm`, mit Modbus-Kodierung
  (`to_code`/`from_code`).

### 5.2 `regulator.rs`

Orchestrierung der Regelung bei jedem Schritt (`step`):

1. wenn **gestoppt** → Ausgang 0, Regler zurückgesetzt;
2. wenn **manuell** → Ausgang = manueller Sollwert (% vorzeichenbehaftet);
3. wenn **auto** → es werden **getrennt** der Beitrag der Heizrichtung (Richtung 1,
   Fehler `SP − PV`) und der Kühlrichtung (Richtung 2, Fehler `PV − SP`) berechnet,
   jeweils ≥ 0, dann `Ausgang = heiß − kalt`:
   - **PID**: Ausgang begrenzt auf `[0, 100]` (`out_min = 0`) — die inaktive
     Richtung (negativer Fehler) gibt 0 aus und ihr Integral **leert sich auf
     natürliche Weise** durch die Begrenzung. Es wird **nicht** gewaltsam auf null
     zurückgesetzt: Bei der starken PWM-Welligkeit würde ein Löschen bei jedem
     Überschreiten des Sollwerts einen statischen Fehler einführen;
   - **TOR**: Das Relais wird auf den vorzeichenbehafteten Fehler ausgewertet und
     behält seinen Zustand beim Durchlaufen des Sollwerts bei, was ein
     **symmetrisches** Hystereseband `[SP − h/2, SP + h/2]` wiederherstellt
     (die Heiz-/Kühlbänder bleiben disjunkt, sodass die beiden Relais sich
     gegenseitig ausschließen);
   - **PWM**: Ein PID berechnet das Tastverhältnis, moduliert durch das Taktrelais;
     der physikalische Ausgang ist strikt 0 % oder 100 %, aber sein Mittelwert
     folgt dem PID.
4. der Ausgang steuert den Prozess, der den neuen Messwert (PV) erzeugt.

> **Historie**: Vor dieser Überarbeitung erfolgte die Heiz-/Kühl-Umschaltung über
> das Vorzeichen des Fehlers und **setzte** das TOR-Relais beim Durchlaufen des
> Sollwerts **zurück** — was die Hysterese auf `[SP − h/2, SP]` verkürzte (halbes
> Band, asymmetrisch) und die TOR-Regelung mittelmäßig machte. Die getrennte
> Berechnung pro Richtung behebt diesen Mangel.

### 5.3 `actors/simulation.rs`

`SimulationActor` (ractor). `pre_start` bewaffnet ein `send_interval(dt)`, das
`Tick` aussendet. `handle` verarbeitet `Tick` (treibt die Simulation voran) und
`Command` (wendet eine Mutation an), dann **veröffentlicht** es den Zustand in
`SharedSnapshot` und `SharedMap`.

### 5.4 `actors/network.rs`

`ModbusServerActor` besitzt den Modbus-Server. `Reconfigure(NetworkConfig)`:
- aktualisiert die geteilte **Whitelist** (sofortige Wirkung, ohne Neustart);
- wenn sich der **Transport** (TCP/RTU), der **Port / die IP** oder die
  **seriellen Parameter** ändern, **stoppt** es die Server-Task und **startet**
  sie neu (`start_tcp` oder `start_rtu`); veröffentlicht den Zustand in
  `SharedStatus` (Erfolg oder Fehler).

Es ist **nur ein Transport** gleichzeitig aktiv (`Transport::Tcp` oder `Rtu`). Das
RTU steckt hinter der **Feature `rtu`**; ohne sie veröffentlicht die Auswahl von
RTU einen expliziten Statusfehler.

### 5.5 `modbus_server.rs`

`RegulatorService` implementiert `tokio_modbus::server::Service` auf
**synchrone** Weise (`future::Ready`): Lesevorgänge = Ausschnitt aus `SharedMap`;
Schreibvorgänge = Dekodierung in `Command` (über `map.rs`), dann `cast` an den
`SimulationActor`.

**Single-Master-Politik.** `serve` (TCP) erlaubt **nur einen entfernten Master
gleichzeitig**: Bei jeder neuen Verbindung (durch die Whitelist erlaubte IP) wird
die vorherige geschlossen. Mechanismus: Der `TcpStream` wird in einen
`CancellableStream` gehüllt, der beim Empfang eines `oneshot`-Signals **EOF beim
Lesen** zurückgibt — die Verarbeitungsschleife von `tokio-modbus` endet dann und
schließt den Socket. `serve_rtu` (Feature `rtu`) bedient den seriellen Bus über
`rtu::Server::serve_forever`: Der RS485-Bus *ist* der einzige Master (nichts zu
verdrängen).

> ⚠️ Die IHM nimmt diesen Pfad nicht: Sie sendet ihre `Command` direkt an den
> Aktor, sie wird daher nie als Master gezählt.
>
> ⚠️ Der RTU-Server von `tokio-modbus` 0.17 übermittelt dem Dienst die
> Slave-Adresse nicht: Das Gerät antwortet daher unabhängig von der angeforderten
> Adresse. Eine **Punkt-zu-Punkt-Verbindung** wird empfohlen. `slave_id` wird
> persistiert und angezeigt, aber nicht zum Filtern verwendet (Einschränkung
> stromaufwärts).

### 5.6 `map.rs`

**Quelle der Wahrheit** des Modbus-Adressplans. Adresskonstanten,
`MemoryMap` (Abbilder der Tabellen), `refresh_from(snapshot)` (Zustand→Register)
und `*_to_command(s)` (Schreibvorgänge→Befehle). Kodierung der `f32` auf 2
Registern, big-endian, höchstwertiges Wort zuerst.

### 5.7 `config.rs`

`AppConfig` (Netzwerk / Prozess / Regelung) ⇄ TOML. `IpFilter` (Joker `*` pro
IPv4-Oktett). `ServerStatus`. `to_regulator_config()` schlägt die Brücke zur Domäne.

### 5.8 `gui.rs`

**Einseitige** IHM: Kopfzeile (Zustände + Schaltflächen), Befehlspanel (links),
Überwachung + Kurve (Mitte), Live-Modbus-Tabelle (rechts), Parameter-Modal.
Liest die `Shared*`, sendet `Command` per nicht blockierendem `cast`.

---

## 6. Szenarien (Sequenzen)

**Modbus-Lesevorgang (PV)**: Client → `RegulatorService::call(ReadInputRegisters)` →
Lesen von `SharedMap` → `Response`. Keine Interaktion mit dem Aktor (minimale Latenz).

**Modbus-Schreibvorgang (Sollwert)**: Client → `call(WriteMultipleRegisters)` →
`map::holdings_to_commands` → `cast(Command::SetSpAuto)` → der Aktor wendet es im
nächsten Schritt an → veröffentlicht `SharedMap`/`SharedSnapshot` neu.

**IHM-Befehl**: Interaktion → `cast(Command)` → ebenso.

**Netzwerk-Rekonfiguration**: Modal *Anwenden* → `cast(Reconfigure)` →
ModbusServerActor bindet bei Bedarf neu → `SharedStatus` aktualisiert → die
Kopfzeile der IHM spiegelt den Zustand wider.

**Tick**: Timer → `Tick` → `Regulator::step` → Veröffentlichung.

---

## 7. Regelungstheorie

**Prozess (FOPDT)**: `v[k+1] = v[k] + (dt/T)·(Ziel − v[k])`, mit
`Ziel = Umgebung + K·u` und `u` um `L` Sekunden verzögert (Totzeitleitung).

**PID**: `u = Kp·e + Ki·∫e + Kd·de/dt`, Integral begrenzt auf `[out_min, out_max]`
(Anti-Windup). Ableitung auf den Fehler (Kompromiss Einfachheit/Heiz-Kühl-Symmetrie).

**TOR**: aktiv wenn `e > +H/2`, inaktiv wenn `e < −H/2`, sonst Zustand beibehalten.

**Bidirektional**: Es wirkt nur eine Richtung gleichzeitig, ausgewählt durch das
Vorzeichen des Fehlers; der Gesamtausgang ist vorzeichenbehaftet (+ heiß / − kalt).

---

## 8. Entscheidungen und Kompromisse

- **Doppelte Veröffentlichung (`Snapshot` + `Map`)** statt einer einzigen Struktur:
  Die IHM verarbeitet Geschäftstypen, das Modbus rohe Register; beide bleiben
  einfach und entkoppelt, zum Preis eines geringfügigen, vernachlässigbaren
  Kopier-Mehraufwands.
- **Modbus-Lesevorgänge ohne den Aktor**: `SharedMap` wird direkt gelesen, um die
  Latenz zu minimieren; der Aktor bleibt der einzige **Schreiber**, also kein Wettlauf.
- **Synchroner Modbus-Dienst** (`future::Ready`): Die gesamte Arbeit ist nicht
  blockierend (kurze Sperre + cast), kein Boxen eines Future nötig.
- **Rebind bei Portwechsel**: Ein Socket ändert seinen Port nicht; eine kurze
  Dienstunterbrechung bei der Rekonfiguration wird in Kauf genommen.
- **Ableitung auf den Fehler** (und nicht auf den Messwert): leichter „Peitschen-
  schlag" beim Sollwertwechsel, akzeptiert, um den Algorithmus symmetrisch und
  einfach zu halten.

---

## 9. Mögliche Weiterentwicklungen

- Modbus RTU / seriell (`RegulatorService` wiederverwenden, Transport ändern).
- Sollwertrampe, PID-Auto-Tuning, simulierte Fehler (Sensor defekt, Sättigung).
- Historisierung / CSV-Export des Trends.
- Umstellung der IHM auf **Registerkarten**, falls die Einzelseite zu dicht wird.
- Neue Instrumente: `mock_bin_<name>` erstellen und das Gemeinsame in
  `mock_lib_*` auslagern (siehe [maintenance.md](maintenance.md)).
