# User manual — Simulated process regulator (RU/OPC UA)

*🌍 [FR](../fr/manuel_utilisateur.md) · **EN** · [DE](../de/manuel_utilisateur.md) · [ES](../es/manuel_utilisateur.md) · [IT](../it/manuel_utilisateur.md) · [PT](../pt/manuel_utilisateur.md) · [NL](../nl/manuel_utilisateur.md) · [PL](../pl/manuel_utilisateur.md)*

> Crate: `mock_bin_ru_opcua` · Executable: **ru_opcua**

---

## 1. What this simulator is for

`ru_opcua` simulates a **process regulator** (PID loop over a thermal process)
and exposes it over **OPC UA**, the industrial supervision standard. It is used
to **test an OPC UA client / a SCADA** (reading measurements, writing setpoints,
subscriptions) without real hardware.

The graphical interface lets you **drive** the simulation and **visualize** the
dynamics; the OPC UA server exposes the same quantities to the network.

---

## 2. Getting started

```bash
cargo run -p mock_bin_ru_opcua          # GUI + OPC UA server
```

On launch, the server listens by default on `opc.tcp://0.0.0.0:4840/` (security
None). The window shows the current state and starts the trend curve.

Connect an OPC UA client (UaExpert, etc.) to `opc.tcp://127.0.0.1:4840/`, security
**None**, user **Anonymous**. The nodes are described in the
[OPC UA reference](reference_opcua.md).

---

## 3. The interface

### Header

- **Title** and **⚙ Settings** / **💾 Save settings** buttons.
- On the right: **device state** (RUNNING / STOPPED), **server status**
  (`OPC UA ● opc.tcp://…` in green when listening, ✖ + message on error), and the
  **CESAM-Lab logo**.
- An **orange banner** permanently reminds you that the endpoint is **anonymous
  (security None)**: expose it only on a trusted network.
- If an update is available, a **banner** offers the download.

### Command panel (left)

- **Start / Stop**: starts or stops the regulation. When stopped, the process
  relaxes toward the ambient value.
- **Automatic mode (PID)**: enabled = the PID computes the output; disabled =
  **manual mode** (the output is forced).
- **Setpoint**: slider, bounded by the setpoint limits (adjustable in
  *Settings*).
- **Manual output (%)**: slider active only in **manual mode**.
- **PID settings**: `Kp`, `Ki`, `Kd` gains editable at runtime.

### Central area

- **Cards**: Measurement, Setpoint, Output.
- **Trend curve**: Measurement (PV) and Setpoint on the left axis (process unit),
  Output (%) on the right axis.

---

## 4. Settings (⚙ modal)

- **Language** of the interface (8 languages), persisted.
- **Check for updates at startup** + **Check now** button.
- **Endpoint**: **listening IP** and **port** of the OPC UA server. A change
  **restarts** the server at runtime (ongoing sessions are closed cleanly).
- **Process (transfer function)**: gain `K`, time constant `τ`, pure delay,
  ambient value.
- **Setpoint bounds**: min / max (reordered automatically if inverted).
- **Apply** / **Reset to defaults** / **Close**.

The settings are saved in `mock_ru_opcua.toml` (current directory; overridable via
the `MOCK_CONFIG` environment variable).

---

## 5. Security

OPC UA **can** be secured (certificates, encryption, authentication), but as it
stands (**Phase 1b**) the simulator exposes only a **security None** **anonymous**
endpoint: no protection. **Do not expose on an open network.** The warning banner
reminds you of this permanently. Real security is planned for **Phase 2**.

---

## 6. FAQ

**Is port 4840 mandatory?** No: it is set in *Settings* (or via the TOML file). A
port < 1024 requires root privileges.

**My client does not see the nodes.** Check the connection to `opc.tcp://…:4840/`,
security **None**, user **Anonymous**, then *Browse* under the `Objects` folder
(namespace `urn:cesam-lab:ru-opcua`).

**A write is refused.** The type must match (`Double` for the quantities,
`Boolean` for `Run`/`Auto`); otherwise the server returns `Bad_TypeMismatch`.

**Run without a graphical interface?** Build *headless*:
`cargo run -p mock_bin_ru_opcua --no-default-features` — the OPC UA server and the
simulation run without a GUI.

**An "encrypted endpoints disabled" message appears.** This is normal in
Phase 1b: no instance certificate is provisioned (encrypted endpoints
unavailable). The None endpoint itself works.
