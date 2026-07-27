# Konzeption — Simulierter PROFIBUS-DP-Regler (ORPD)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · **DE** · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

> Crate: `mock_bin_ru_pbdp` · Ausführbare Datei: **ru_pbdp** (*Regulation Unit over PROFIBUS DP*)

Architektur- und Modellierungsdokument. Angelehnt an den **ORME**-Regler
(`mock_bin_ru_modbus`) für das Fachmodell und die Akteure, und an **OSNE**
(`mock_bin_su_namur`) für die serielle Verbindung. Nur die
**Protokollschicht** ändert sich: ein von Grund auf entwickelter
**Software-Simulator von PROFIBUS-DP-V0-Telegrammen** (bis heute existiert
keine veröffentlichte `profibus`/`profibus-dp`-Crate im Rust-Ökosystem).

---

## 1. Zweck

Einen **Prozessregler** simulieren (PID-Regelkreis auf einem thermischen
Prozess erster Ordnung, Modell **identisch** zu ORME) und ihn über eine
**PROFIBUS-DP-V0-Telegrammstruktur** auf einer seriellen Verbindung
(RS-485/RS-232) bereitstellen.

**Dieses Dokument setzt voraus, dass der Leser den Hinweis zur
Nicht-Interoperabilität** gelesen hat (siehe
[`manuel_utilisateur.md`](manuel_utilisateur.md) und
[`reference_profibus.md`](reference_profibus.md) §6): echtes PROFIBUS DP
verlangt eine Einhaltung des Bus-Timings auf Bit-Ebene (Slot Time, `Tsdr`
min/max, ein Watchdog im Bereich von Millisekunden), die nur ein
dedizierter ASIC (SPC3/VPC3) garantieren kann. Dieser Simulator erhebt
diesen Anspruch nicht — er ist ein pädagogisches und
Software-Testwerkzeug, kein Bustreiber.

---

## 2. Physikalisches Modell ([`regulator.rs`](../../src/regulator.rs))

Unverändert vom ORME-Regler übernommen:
[`mock_lib_control::FirstOrderProcess`] (Übertragungsfunktion erster Ordnung
mit reiner Totzeit) und [`mock_lib_control::Pid`] (Anti-Windup-PID), mit
denselben Modi (Aus/PID/Zweipunkt/PWM) in beiden Richtungen (heizen/kühlen).
Simulationsschritt: **50 ms**. Alle Schreibvorgänge werden in
`Regulator::apply` **bereinigt** (Grenzen neu geordnet, nicht endliche
Gleitkommawerte ignoriert, PID-Verstärkungen begrenzt) — dieselbe Invariante
wie überall sonst im Workspace: niemals `f32::clamp` mit ungeprüften Grenzen
aufrufen.

---

## 3. Architektur (Akteure)

```
GUI (egui) ──Command(cast)──►  SimulationActor  ──refresh──► SharedSnapshot ──► GUI
Simulierter PROFIBUS-Master ─►  (Regulator)      ──refresh──► SharedSnapshot ──► Data_Exchange-Antworten
```

- **`SimulationActor`** ([`actors/simulation.rs`](../../src/actors/simulation.rs)):
  in Form identisch zu ORME/OSNE — alleiniger Eigentümer des `Regulator`,
  neu bewaffneter Einmal-Timer, veröffentlicht bei jedem Schritt den
  `SharedSnapshot`.
- **`ProfibusServerActor`** ([`actors/network.rs`](../../src/actors/network.rs)):
  besitzt die serielle Verbindung; `Reconfigure` schließt/öffnet den
  Transport neu, wenn sich Port/Baudrate/Stationsadresse ändern; behält den
  `JoinHandle` der Sitzung (wird beim Herunterfahren abgebrochen);
  veröffentlicht den Verbindungsstatus (`ServerStatus`, einschließlich des
  aktuellen Zustands der DP-V0-Zustandsmaschine) für die GUI.
- **[`profibus.rs`](../../src/profibus.rs)** — **Quelle der Wahrheit** für
  das Protokoll: Telegramm-Codec (`SD1`/`SD2`/`SD3`/`SD4`/`SC`, FCS),
  Dekodierung der Dienste (`Slave_Diag`/`Set_Prm`/`Chk_Cfg`/`Data_Exchange`)
  und die Zustandsmaschine des Slaves `SlaveFsm`
  (`PowerOn → WaitPrm → WaitCfg → DataExchange`).
- **[`map.rs`](../../src/map.rs)** — Umwandlung der `Data_Exchange`-E/A-
  Byteblöcke von/zu den `Command`s des Reglers (siehe
  [`reference_profibus.md`](reference_profibus.md) §3).
- **[`profibus_server.rs`](../../src/profibus_server.rs)** — Sitzungsschleife
  über einen beliebigen `AsyncRead + AsyncWrite`-Stream (in Produktion der
  serielle Port, in Tests ein `tokio::io::duplex`): liest ein Telegramm,
  dekodiert es, ruft `SlaveFsm::handle` auf, wendet die resultierenden
  `Command`s an, kodiert die Antwort und sendet sie zurück. Behandelt
  außerdem den **Protokoll-Watchdog** (`tokio::select!` zwischen
  Telegrammlesen und einer Verzögerung, wie OSNEs NAMUR-Watchdog — hier
  jedoch ein **echter Bestandteil des DP-Protokolls**, durch `Set_Prm`
  bewaffnet, kein selbstgemachter Zusatz).

Anders als bei Modbus (ORME, eine separate, bei jedem Tick neu erzeugte
Speichertabelle) und wie bei OPC UA/NAMUR gibt es **keine persistente
Speichertabelle**: Der `Data_Exchange`-Eingangsblock wird im Moment der
Antwort spontan aus dem `SharedSnapshot` neu berechnet.

**Keine Multi-Master-Richtlinie zu verwalten**: Die serielle Verbindung
*ist* der alleinige Master (wie Modbus RTU oder der serielle NAMUR-Port),
im Gegensatz zu ORMEs Modbus TCP (Verdrängung) oder sogar OSNEs NAMUR-TCP
(Punkt-zu-Punkt ohne Verdrängung).

---

## 4. PROFIBUS-DP-V0-Codec — Entscheidungen und akzeptierte Grenzen

- **Telegrammbegrenzer** (`SD1=0x10`, `SD2=0x68`, `SD3=0xA2`, `SD4=0xDC`,
  `SC=0xE5`, `ED=0x16`) und **FCS** (Summe modulo 256): normkonform, öffentlich
  gut dokumentiert.
- **SAP-Nummern der Parametrierungsdienste** (`Slave_Diag=61`,
  `Set_Prm=62`, `Chk_Cfg=63`): normkonform.
- **Exakte Kodierung der FC-Feld-Bits**, **genaue Anordnung der
  Diagnosebytes** und **Anordnung der Eingangs-/Ausgangsblöcke** (`map.rs`):
  dies sind **simulatorspezifische Konventionen**, kein beim PNO registriertes
  echtes GSD-Profil. Der Simulator verwendet für alle `Data_Exchange`-
  Austausche systematisch **SD2**-Telegramme (variable Länge), auch wenn
  `SD3` (8 feste Bytes) in einem echten Profil ausreichen würde — eine
  Entscheidung, die den Codec vereinfacht, ohne an Abdeckung der
  Protokollkonzepte zu verlieren.
- **PROFIBUS-Kennung** (`Ident_Number = 0xEE01`): **fiktiv**, nicht beim PNO
  registriert (PROFIBUS & PROFINET International) — stellt kein reales
  Katalog-Gerät dar.
- **Kein Bus-Timing**: weder ein Antwortfenster (`Tsdr`) noch ein Token
  noch eine Multi-Master-Arbitrierung sind implementiert — siehe §1.

Vollständige Details in [`reference_profibus.md`](reference_profibus.md).

---

## 5. Konfiguration & Persistenz

`AppConfig` (Sprache / serielle Verbindung / Prozess / Regelung /
Update-Prüfung) wird als **TOML** serialisiert ([`config.rs`](../../src/config.rs)),
**beim Laden bereinigt** (`AppConfig::sanitized`: Grenzen geordnet,
`τ ≥ 1e-3`, `dead_time ≥ 0`, endliche Gleitkommawerte, Stationsadresse
begrenzt auf `[0, 125]`). Datei: `mock_ru_pbdp.toml` (überschreibbar über
`MOCK_CONFIG`). Anders als bei ORME/OSNE gibt es **keine IP-Positivliste**
(die serielle Verbindung ist von Natur aus Punkt-zu-Punkt, es gibt keinen
Begriff einer Netzwerkadresse).

---

## 6. Weiterentwicklungsmöglichkeiten

- Ein echtes Werkzeug für einen **simulierten PROFIBUS-DP-Master**
  (eigenständige ausführbare Datei), das dieselben zum Testen in
  `profibus.rs` bereitgestellten Kodier-/Dekodierfunktionen nutzt, um diesen
  Simulator oder jeden anderen Software-Slave zu steuern, ohne von einem
  Ad-hoc-Skript abhängig zu sein.
- Erzeugung einer illustrativen **GSD**-Datei (auf Simulatorseite
  funktionslos), die das simulierte E/A-Profil zu pädagogischen Zwecken
  dokumentiert.
- Unterstützung von **DP-V1** (azyklischer Zugriff, Alarme), falls der
  pädagogische Bedarf entsteht — zunächst außerhalb des Umfangs (nur DP-V0).
- Beförderung des Reglermodells in eine gemeinsame `mock_lib_*` (heute
  zwischen ORME und diesem Instrument dupliziert, wie bei ORUE).
