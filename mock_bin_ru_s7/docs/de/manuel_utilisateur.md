# Benutzerhandbuch — S7-Regler (ORSS)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · **DE** · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

---

## 1. Wozu das Instrument dient

**ORSS** simuliert eine **Prozess-Regeleinheit** (PID + thermischer Prozess erster
Ordnung) und stellt sie als **Siemens-S7-Steuerung** bereit (S7comm-Server über
ISO-on-TCP). Es dient dazu, eine Leitwarte oder einen S7-Client (Snap7, TIA Portal
lesend, nodes7 …) ohne echte Steuerung zu testen.

## 2. Erste Schritte

```bash
cargo run -p mock_bin_ru_s7        # IHM + S7-Server
```

Der Server lauscht standardmäßig auf `0.0.0.0:102`. ⚠️ Der **Port 102 erfordert
Root-Rechte**; andernfalls stellen Sie im Modal *Parameter* einen hohen Port ein
(z. B. 1102).

Die Kopfzeile zeigt den Zustand an: **S7 ●** (grün) mit der Lauschadresse oder eine
Fehlermeldung (rot), wenn der Bind fehlschlägt. Ein orangefarbenes Banner warnt, wenn
der Server **exponiert** ist (alle Schnittstellen + leere Whitelist).

## 3. Oberfläche

- **Kopfzeile**: Titel, Schaltflächen *Parameter* / *Speichern*, Lauf-/Stopp-Zustand,
  S7-Lauschzustand, Banner zur Netzexposition.
- **Linkes Panel (Befehle)**: *Start/Stopp*, *Automatikmodus (PID)*, *Sollwert*,
  *Manueller Ausgang* (Handbetrieb), **PID**-Einstellungen (Kp/Ki/Kd).
- **Mittleres Panel**: Karten *Messwert / Sollwert / Ausgang* + **Echtzeitkurve**.
- **Modal *Parameter***: Sprache, Update-Prüfung, **S7-Netz** (Lausch-IP, Port,
  **IP-Whitelist** — ein Muster pro Zeile, `*` = Platzhalter), **Prozess**
  (K, τ, Verzögerung, Umgebung), **Sollwertgrenzen**. *Anwenden* startet das Lauschen
  neu, wenn sich IP/Port ändert, und speichert die TOML-Datei.

## 4. Einen S7-Client verbinden

Der Client verbindet sich mit IP/Port des Servers. Die üblichen **Rack/Slot**-Werte
(0/1 oder 0/2) funktionieren: der Server erzwingt keinen TSAP. Die Größen befinden sich
in **DB1** (siehe [`reference_s7.md`](reference_s7.md)): Sollwert in `DB1.DBD0`, Messwert
in `DB1.DBD4`, Start in `DB1.DBX16.0` usw.

## 5. FAQ

- **„Permission denied“ beim Start** → der Port 102 erfordert Root-Rechte; verwenden
  Sie einen hohen Port oder starten Sie mit den passenden Privilegien.
- **Der Client verbindet sich nicht** → IP/Port, die **Whitelist** und die Firewall
  prüfen. Rack/Slot 0/1 dann 0/2 testen.
- **Meine Schreibvorgänge haben keine Wirkung** → nur die steuerbaren Offsets wirken
  (Sollwert, manueller Ausgang, Start, Auto); die anderen sind schreibgeschützt.
- **Wo liegt die Konfigurationsdatei?** → `mock_ru_s7.toml` (aktuelles Verzeichnis;
  über `MOCK_CONFIG` überschreibbar).
