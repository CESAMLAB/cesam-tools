# Ontwerp — Gesimuleerde procesregelaar (RU/OPC UA)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · [ES](../es/conception.md) · [IT](../it/conception.md) · [PT](../pt/conception.md) · **NL** · [PL](../pl/conception.md)*

> Crate: `mock_bin_ru_opcua` · Uitvoerbaar bestand: **ru_opcua** (*Regulation Unit over OPC UA*)

Architectuur- en modelleringsdocument. Gemodelleerd naar de regelaar **ORME**
(`mock_bin_ru_modbustcp`): dezelfde opdeling **synchroon bedrijfsmodel / ractor-
actoren / protocollaag / egui-GUI**, dezelfde invarianten. Alleen het **transport**
verandert: **OPC UA** in plaats van Modbus.

---

## 1. Doel

Een **procesregelaar** simuleren (PID-lus op een thermisch eersteordeproces) en
deze via **OPC UA** beschikbaar stellen, de standaard voor industriële supervisie
(Industrie 4.0). In tegenstelling tot ORME (Modbus) en OSNE (NAMUR) —
**veldprotocollen zonder beveiliging** — ondersteunt OPC UA van nature
authenticatie, ondertekening en versleuteling (voorzien in Fase 2).

---

## 2. Fysisch model ([`regulator.rs`](../../src/regulator.rs))

Het **proces** hergebruikt [`mock_lib_control::FirstOrderProcess`] (gedeeld met
ORME): eersteorde-overdrachtsfunctie met zuivere dode tijd

```text
PV(s) / U(s) = K · e^(−L·s) / (1 + τ·s)
```

- `PV`: meting (proceseenheid, bijv. °C);
- `U`: stuurwaarde / uitgang (0-100 %);
- `K`: statische versterking; `τ`: tijdconstante; `L`: zuivere dode tijd;
- `ambient`: rustwaarde (nuluitgang).

Een **PID** ([`mock_lib_control::Pid`], eveneens hergebruikt van ORME) regelt de
meting naar het **setpoint** door de uitgang te sturen, begrensd tot `[0, 100]`. Twee modi:
**automatisch** (de PID berekent de uitgang) en **handmatig** (uitgang opgelegd). De
simulatiestap bedraagt **0,5 s** (traag thermisch proces).

Alle schrijfbewerkingen (netwerk of GUI) worden **gesaneerd** in `Regulator::apply`:
niet-eindige floats genegeerd, setpoint begrensd, grenzen herordend (`min ≤ max`),
PID-versterkingen geclampt. **Invariant: nooit `f32::clamp` met niet-gevalideerde
grenzen** (panic als `min > max` of `NaN`).

---

## 3. Architectuur (actoren)

```
GUI (egui) ───Command(cast)──►  SimulationActor ──refresh──► SharedSnapshot ──► GUI
OPC UA-server ─Command(cast)─►   (Regulator)    ──refresh──► SharedSnapshot ──► OPC UA-lezingen
```

- **`SimulationActor`** ([`actors/simulation.rs`](../../src/actors/simulation.rs)):
  **enige** eigenaar van de `Regulator`; laat de simulatie vorderen op een opnieuw
  geladen one-shot timer (geen losgekoppelde timer) en publiceert bij elke stap een
  `SharedSnapshot`.
- **`OpcuaServerActor`** ([`actors/network.rs`](../../src/actors/network.rs)):
  bezit de OPC UA-server (tokio-taak `server.run()`); warm herstartbaar
  (`Reconfigure`: rebind als IP/poort verandert); behoudt de `JoinHandle` (afbreken
  bij stoppen) en de `ServerHandle` (nette annulering van de sessies); publiceert zijn
  luisterstatus voor de GUI.
- **OPC UA-server** ([`opcua_server.rs`](../../src/opcua_server.rs)): bouwt de
  server [`async-opcua`](https://crates.io/crates/async-opcua), declareert de
  adresruimte en koppelt de callbacks. De **lezingen** putten uit de
  `SharedSnapshot`; de **schrijfbewerkingen** zenden een `Command` naar de
  `SimulationActor` via een niet-blokkerende `cast`.

Net als NAMUR (OSNE) en in tegenstelling tot Modbus van ORME is er **geen aparte
geheugentabel**: de OPC UA-nodes lezen rechtstreeks uit de gedeelde momentopname.

---

## 4. OPC UA-stack — technische keuzes

- **`async-opcua`** (server, feature `server`): **tokio-native** implementatie
  (één taak per verbinding), die naadloos past in de ractor/tokio-stack. Crypto
  **100 % Rust** (RustCrypto: `rsa`, `aes`, `sha2`, `x509-cert`) — **geen
  OpenSSL-afhankelijkheid**, wat de cross-compilatie behoudt (Linux/Windows/RPi).
- **Adresruimte**: een `SimpleNodeManager` in het geheugen; `Variable`-nodes
  georganiseerd onder `Objects` (zie [`reference_opcua.md`](reference_opcua.md)).
- **Callbacks**: `add_read_callback` (levende waarde, bemonsterd voor de
  abonnementen) en `add_write_callback` (route naar de simulatie).
- **Licentie**: `async-opcua` valt onder **MPL-2.0** (de hele OPC UA-lijn in Rust
  doet dat). Copyleft **per bestand**: ongewijzigd gebruik → de CESAM-Lab-code blijft
  MIT (zie bestand `NOTICE` in de hoofdmap).

---

## 5. Beveiliging

De beveiliging is **instelbaar** (`SecurityConfig`) en vormt de onderscheidende
factor van OPC UA ten opzichte van de veldprotocollen (Modbus/NAMUR, zonder
beveiliging).

- **Onversleutelde modus (standaard)**: een endpoint `SecurityPolicy::None`,
  **anoniem** token — uitsluitend vertrouwd netwerk, onmiddellijke start, geen
  certificaat. De GUI toont een **oranje waarschuwingsbanner**.
- **Versleutelde modus (Fase 2)**: endpoint `Basic256Sha256` / `SignAndEncrypt`.
  Een zelfondertekend **instantiecertificaat** wordt bij de eerste start gegenereerd
  (`pki/`); de server vertrouwt de clientcertificaten. **Authenticatie** met
  gebruiker/wachtwoord (`ServerUserToken::user_pass`) en/of anoniem. De GUI toont
  een **groene banner** 🔒.

De modus wordt ingesteld in het modale venster *Parameters*; een wijziging
**herstart** de server warm (`OpcuaServerActor`).

---

## 6. Configuratie & persistentie

`AppConfig` (taal / netwerk / proces / regeling / update-controle) geserialiseerd in
**TOML** ([`config.rs`](../../src/config.rs)), **gesaneerd bij het laden**
(`AppConfig::sanitized`: geordende grenzen, `τ ≥ 1e-3`, `dead_time ≥ 0`, eindige
floats). Bestand: `mock_ru_opcua.toml` (te overschrijven via `MOCK_CONFIG`).

---

## 7. Ontwikkelingsrichtingen

- **Fase 2**: OPC UA-beveiliging (certificaten, versleuteling, auth).
- OPC UA-methoden (`Reset`, `Autotune`) naast de variabelen.
- Getypeerd informatiemodel (ObjectType regelaar) in plaats van platte variabelen.
- Historisering / `HistoryRead` op de meting.
- Promotie van het ORME-regelaarmodel naar een gedeelde `mock_lib_*` (het is
  vandaag gedupliceerd tussen ORME en dit instrument).
