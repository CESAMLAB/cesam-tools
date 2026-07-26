# Benutzerhandbuch — EtherNet/IP-Regler (OREE)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · **DE** · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

---

## 1. Wozu das Instrument dient

**OREE** simuliert eine **Regelungseinheit** eines Prozesses (PID + thermischer Prozess
erster Ordnung) und stellt sie als **EtherNet/IP-Adapter** dar (explizites CIP-Messaging).
Es dient zum Testen einer Leitwarte oder eines EtherNet/IP-Clients (pycomm3, RSLinx
lesend, rseip …) ohne reale Hardware.

## 2. Erste Schritte

```bash
cargo run -p mock_bin_ru_ethernetip        # IHM + EtherNet/IP-Adapter
```

Der Server lauscht standardmäßig auf `0.0.0.0:44818` (keine Privilegien erforderlich).
Die Kopfzeile zeigt den Zustand an: **EtherNet/IP ●** (grün) mit der Lauschadresse oder
eine Fehlermeldung (rot). Ein orangefarbenes Banner warnt, wenn der Server **exponiert**
ist (alle Schnittstellen + leere Whitelist).

## 3. Oberfläche

- **Kopfzeile**: Titel, Schaltflächen *Parameter* / *Speichern*, Ein-/Aus-Zustand,
  EtherNet/IP-Lauschzustand, Banner zur Netzexposition.
- **Linkes Feld (Befehle)**: *Ein/Aus*, *Automatikmodus (PID)*, *Sollwert*,
  *Manuelle Ausgabe* (Handbetrieb), **PID**-Einstellungen (Kp/Ki/Kd).
- **Mittleres Feld**: Karten *Messwert / Sollwert / Ausgabe* + Echtzeit-**Kurve**.
- **Modal *Parameter***: Sprache, Update-Prüfung, **EtherNet/IP-Netzwerk** (Lausch-IP,
  Port, **Whitelist** von IPs — ein Muster pro Zeile, `*` = Platzhalter),
  **Prozess** (K, τ, Verzögerung, Umgebung), **Sollwertgrenzen**. *Anwenden* startet
  das Lauschen neu, wenn sich IP/Port ändert, und speichert das TOML.

## 4. Einen EtherNet/IP-Client verbinden

Der Client verbindet sich mit IP/Port des Servers (`RegisterSession` automatisch) und
liest/schreibt dann die **benannten Tags** per explizitem Messaging: `Setpoint`,
`ProcessValue`, `Output`, `ManualOutput`, `Run`, `Auto` usw. (siehe
[`reference_ethernetip.md`](reference_ethernetip.md)). ⚠️ Die Werte sind
**little-endian** (REAL = `f32` LE).

## 5. FAQ

- **Der Client verbindet sich nicht** → IP/Port (44818), die **Whitelist** und die
  Firewall prüfen.
- **Tag nicht gefunden** → nur die dokumentierten Tags existieren; die Namen sind
  groß-/kleinschreibungsempfindlich.
- **Meine Schreibvorgänge haben keine Wirkung** → nur die steuerbaren Tags wirken
  (`Setpoint`, `ManualOutput`, `Run`, `Auto`); die anderen sind schreibgeschützt.
- **Wo ist die Konfigurationsdatei?** → `mock_ru_ethernetip.toml` (aktuelles
  Verzeichnis; überschreibbar durch `MOCK_CONFIG`).
