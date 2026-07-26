# Gebruikershandleiding — Sparkplug B-regelaar (ORSE)

*🌍 [FR](../fr/manuel_utilisateur.md) · [EN](../en/manuel_utilisateur.md) · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · **NL** · [PL](../pl/manuel_utilisateur.md)*

---

## 1. Waarvoor dient het instrument

**ORSE** simuleert een **procesregeleenheid** (PID + thermisch proces van de eerste orde)
en publiceert zijn toestand in **MQTT Sparkplug B**, als een **edge node** die verbinding
maakt met een **broker** en metrics blootstelt aan een SCADA. Het dient om een
Sparkplug B-acquisitieketen (Ignition, Chariot, EMQX, Node-RED…) te testen zonder echte
hardware.

## 2. Vereiste: een MQTT-broker

Aangezien ORSE een **client** is, is er een bereikbare MQTT-broker nodig. Lokaal:

```bash
docker run -it --rm -p 1883:1883 eclipse-mosquitto
```

## 3. Aan de slag

```bash
cargo run -p mock_bin_ru_sparkplugb        # GUI + Sparkplug B edge node
```

Bij het opstarten probeert de GUI verbinding te maken met de broker (`localhost:1883`
standaard). De koptekst geeft de toestand aan: **Verbonden** (groen) zodra de `NBIRTH`
gepubliceerd is, of **Verbroken** (rood) met de reden. Een oranje banner **⚠ MQTT
onversleuteld** herinnert aan het ontbreken van TLS.

## 4. Interface

- **Koptekst**: titel, knoppen *Parameters* / *Opslaan*, aan/uit-toestand,
  Sparkplug B-verbindingstoestand, TLS/onversleuteld-banner.
- **Linkerpaneel (Commando's)**: *Aan/Uit*, *Automatische modus (PID)*,
  *Setpoint*, *Handmatige uitgang* (handmatige modus), **PID**-instellingen (Kp/Ki/Kd).
- **Centraal paneel**: kaarten *Meting / Setpoint / Uitgang* + **realtime curve**.
- **Modaal *Parameters***: taal, updatecontrole, **MQTT-broker / Sparkplug B**
  (host, poort, client_id, group_id, edge_node_id, keepalive, TLS, gebruiker/wachtwoord,
  publicatie bij-wijziging/periodiek), **proces** (K, τ, vertraging, omgeving),
  **setpointgrenzen**. *Toepassen* herstart de verbinding en slaat de TOML op.

## 5. Aansturen vanaf een SCADA

De SCADA abonneert zich op `spBv1.0/<group_id>/#` en ontvangt `NBIRTH` gevolgd door `NDATA`.
Om de regelaar te **commanderen**, publiceert hij een `NCMD` op
`spBv1.0/<group_id>/NCMD/<edge_node_id>` met de aanstuurbare metrics (`Setpoint`,
`Run`, `Auto`, `ManualOutput`) of `Node Control/Rebirth = true` om een wedergeboorte
te forceren. Details: [`reference_sparkplugb.md`](reference_sparkplugb.md).

## 6. FAQ

- **"Verbroken" permanent** → broker onbereikbaar: controleer `broker_host`/
  `broker_port`, de firewall, en dat de broker draait.
- **De SCADA ziet niets** → controleer de `group_id`/`edge_node_id` en het abonnement
  `spBv1.0/<group>/#`; de payloads zijn **protobuf** (Sparkplug-decoder vereist).
- **Mijn NCMD-schrijfbewerkingen worden genegeerd** → niet-aanstuurbare metric of verkeerd
  type (zie metrictabel). Alleen `Setpoint`/`Run`/`Auto`/`ManualOutput` en `Rebirth`
  worden geaccepteerd.
- **Waar staat het configuratiebestand?** → `mock_ru_sparkplugb.toml` (huidige map;
  overschrijfbaar via `MOCK_CONFIG`).
