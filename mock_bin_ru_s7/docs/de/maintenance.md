# Wartung — S7-Regler (ORSS)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · **DE** · [ES](../es/maintenance.md) · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

---

## 1. Build & Start

```bash
cargo run -p mock_bin_ru_s7                        # IHM + S7-Server
cargo build -p mock_bin_ru_s7 --release            # ausführbare Datei mit IHM
cargo build -p mock_bin_ru_s7 --no-default-features # headless (ohne IHM)
```

Features: `gui` (IHM `egui`, standardmäßig). `--no-default-features` erzeugt eine
**headless**-Binärdatei: S7-Server + Simulation, ohne IHM und ohne Update-Prüfung.

⚠️ Der Port **102** (S7-Standard) ist privilegiert (< 1024): mit den passenden Rechten
ausführen oder einen hohen Port in der Konfiguration wählen.

## 2. Konfiguration

TOML-Datei `mock_ru_s7.toml` (aktuelles Verzeichnis; Pfad über `MOCK_CONFIG`
überschreibbar). Abschnitte: `language`, `[network]` (`bind_ip`, `port`, `allowlist`),
`[process]`, `[regulation]`, `check_updates`. Jeder Wert wird beim Laden **bereinigt**.

## 3. Tests

```bash
cargo test -p mock_bin_ru_s7      # Unit-Tests + lokaler TCP-Round-Trip
```

- **Protokollschicht** (`s7_server`, ohne Netz): CR→CC, Setup, Read/Write Var,
  Bit-Schreibvorgang, Rückgabecode außerhalb der Zone, **Nicht-Panik** bei
  fehlerhaften Telegrammen, Round-Trip des DB-Abbilds.
- **Netz-Aktor**: Bind/Lauschen sowie ein **echter TCP-Round-Trip** (COTP-Verbindung,
  Schreiben und anschließendes Wiederlesen des Sollwerts über rohe S7-Telegramme) —
  ohne Abhängigkeit von einem externen Client.

## 4. Fehlersuche

| Symptom | Hinweis |
|---|---|
| Bind schlägt fehl (`permission denied`) | Port 102 < 1024 → Root-Rechte oder hoher Port |
| Client abgewiesen | IP-Whitelist; Firewall; IP/Port |
| Keine Antwort | Rack/Slot (0/1, 0/2 testen); Telegramme außerhalb der Teilmenge werden ignoriert |
| Schreibvorgang ohne Wirkung | schreibgeschützter Offset (siehe Adressbelegung) |

## 5. Docker (headless)

Headless-Image über `scripts/build-prod.sh` (Eintrag `mock_bin_ru_s7:ru_s7:102`,
`EXPOSE 102`). Ein Volume auf das Arbeitsverzeichnis einbinden, um die `mock_ru_s7.toml`
bereitzustellen. Der Container veröffentlicht Port 102; bei Bedarf hostseitig auf einen
hohen Port mappen.

## 6. Adressbelegung erweitern

Die DB1-Belegung und das Mapping der Schreibvorgänge sind die **Quelle der Wahrheit** in
[`s7_server.rs`](../../src/s7_server.rs) (`db_image` + `handle_write`). Um eine Größe
hinzuzufügen: sie in `db_image` (Lesen) schreiben und, falls steuerbar, dem `match` von
`handle_write` (Schreiben → `Command`) hinzufügen, dann hier und in
[`reference_s7.md`](reference_s7.md) widerspiegeln. Einen Test im Modul hinzufügen.

## 7. Cross / Windows

Wie die übrigen Instrumente (siehe `Cross.toml`). Keine besondere native Abhängigkeit:
die S7-Schicht ist zu 100 % Rust über Standard-TCP.
