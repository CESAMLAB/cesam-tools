# Ontwerp — Gesimuleerde laboratoriumroerder (OSNE)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · **NL** · [PL](../pl/conception.md)*

> Crate: `mock_bin_su_namur` · Uitvoerbaar bestand: **OSNE** (*Open Stirrer NAMUR Emulator*)

Architectuur- en modelleringsdocument. Gemodelleerd naar de **ORME**-regelaar
(`mock_bin_ru_modbustcp`): dezelfde opdeling **synchrone bedrijfslogica / ractor-
acteurs / protocollaag / egui-GUI**, dezelfde invarianten.

---

## 1. Doel

Een **laboratoriumroerder** simuleren (type IKA) bestuurd via het seriële
**NAMUR**-protocol. De motor heeft een **overdrachtsfunctie** (snelheidsdynamiek)
geregeld door een **snelle regeling**, en de **viscositeit** van het medium is
instelbaar en beïnvloedt het koppel.

---

## 2. Fysiek model

### Motor ([`motor.rs`](../../src/motor.rs))

Koppelbalans, geïntegreerd via expliciete Euler:

```text
J · dω/dt = T_moteur − k · η · ω − T_frottement
```

- `ω`: snelheid (tr/min);
- `T_moteur`: motorkoppel (besturing, N·cm, ≥ 0);
- `k · η · ω`: **visceus belastingskoppel** (∝ viscositeit `η` en snelheid);
- `T_frottement`: resterende droge wrijving;
- `J` (`inertia`): regelt de **reactiviteit** (klein ⇒ snel).

In stationaire toestand geldt `T_moteur = k·η·ω + T_frottement`: het koppel dat
nodig is om een snelheid aan te houden **stijgt met de viscositeit**. Als dit
koppel het **maximale koppel** overschrijdt, is het setpoint niet meer haalbaar →
**overbelasting**.

### Regeling ([`stirrer.rs`](../../src/stirrer.rs))

Een **PID** ([`mock_lib_control::Pid`], hergebruikt uit ORME) neemt de
snelheidsfout `setpoint − meting` en produceert het **motorkoppel**, begrensd tot
`[0, couple_max]`. De standaardversterkingen zijn bewust « stug »: de uitgang
verzadigt bij het maximale koppel zolang de fout groot is (snelle stijging),
waarna de integrerende term stabiliseert. De simulatiestap is **20 ms** (50 Hz),
fijner dan die van ORME omdat de dynamiek van een motor snel is.

---

## 3. Architectuur (acteurs)

```
GUI (egui) ──Command(cast)──►  SimulationActor ──refresh──► SharedSnapshot ──► GUI
NAMUR-server ──Command(cast)─►   (Stirrer)     ──refresh──► SharedSnapshot ──► NAMUR-uitlezingen
```

- **`SimulationActor`** ([`actors/simulation.rs`](../../src/actors/simulation.rs)):
  enige eigenaar van de `Stirrer`; voert de simulatie verder op een opnieuw
  ingestelde one-shot-timer (geen losgekoppelde timer) en publiceert een
  `SharedSnapshot`.
- **`NamurServerActor`** ([`actors/network.rs`](../../src/actors/network.rs)):
  bezit de NAMUR-server, warm herstartbaar (`Reconfigure`); gedeelde IP-witlijst;
  luisterstatus gepubliceerd voor de GUI.
- **NAMUR-server** ([`namur_server.rs`](../../src/namur_server.rs)): leest de
  ASCII-regels, interpreteert ze ([`namur.rs`](../../src/namur.rs)), beantwoordt de
  leesopdrachten en geeft de schrijfopdrachten/acties door aan de acteur. **Eén
  master tegelijk** (punt-tot-punt). **Waakhond** per sessie.

De NAMUR-leesopdrachten putten uit de `SharedSnapshot` (geen aparte
geheugentabel zoals de Modbus van ORME: het NAMUR-protocol is « commando »-
georiënteerd, niet « registers »).

---

## 4. Configuratie & beveiliging

- `AppConfig` (taal / netwerk-serieel / motor / regeling) geserialiseerd in **TOML**
  ([`config.rs`](../../src/config.rs)), **bij het laden gesaneerd**
  (`AppConfig::sanitized`: geordende grenzen, eindige floats) — invariant gedeeld
  met ORME (nooit `clamp` met niet-gevalideerde grenzen).
- NAMUR heeft **geen authenticatie en geen versleuteling**: vertrouwd netwerk +
  IP-witlijst (TCP). Standaard `0.0.0.0` + lege lijst ⇒ blootgesteld: de GUI toont
  een **waarschuwingsbalk**.

---

## 5. Mogelijke uitbreidingen

- Draairichting (CW/CCW) en versnellingsramp.
- Temperatuursensor (`IN_PV_2/3`) als er een thermisch model wordt toegevoegd.
- Niet-lineair belastingskoppel (turbulent regime ∝ ω²).
- Promotie van het motormodel naar `mock_lib_control` als het een tweede
  instrument dient.
