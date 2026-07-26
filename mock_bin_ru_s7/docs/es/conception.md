# Diseño — Regulador S7 (ORSS)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · **ES** · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

---

## 1. Visión general

ORSS reutiliza la arquitectura de los demás instrumentos CESAM-Lab: **modelo de
negocio síncrono y comprobable** (PID + proceso), **actores `ractor`** sobre Tokio,
**IHM `egui`** que lee una instantánea compartida. Solo cambia la **capa de
transporte**: un **servidor S7comm** (ISO-on-TCP / RFC1006) en lugar de Modbus/OPC UA.

```
        Command (cast)                      refresh cada paso
IHM   ───────────────────►  SimulationActor ───────────────────►  SharedSnapshot (IHM)
S7 Write Var ────────────►  (Regulator)      ──────────────────►  SharedSnapshot
S7 Read Var  ◄────────────────────────────────  SharedSnapshot (imagen DB1)
```

## 2. Actores

- **`SimulationActor`** — posee el único [`Regulator`]. Bucle de paso fijo; aplica
  los `Command` (IHM o escrituras S7); publica la instantánea tras cada mutación.
- **`S7ServerActor`** — posee el **bucle de escucha TCP**. Una tarea tokio dedicada
  enlaza el socket y acepta los clientes; cada sesión se sostiene mediante un `JoinSet`
  **interno** (por tanto abatida con el bucle — ninguna tarea separada). `Reconfigure`
  reinicia la escucha si cambia la IP/puerto y actualiza la **lista blanca** compartida.

## 3. Capa de protocolo

[`s7_server.rs`](../../src/s7_server.rs) es **puro y síncrono** (sin ninguna
dependencia de red): framing TPKT, COTP (CR→CC, DT) y S7comm (Setup, Read Var, Write
Var) sobre una **imagen de bytes DB1**. El análisis está **acotado** (acceso mediante
`get`/slices verificados): una trama mal formada procedente de la red no provoca
**nunca** un panic, solo una ausencia de respuesta. Es el equivalente S7 de
`opcua_server.rs`, aislado para ser **comprobable sin socket**.

### Por qué un servidor hecho a mano

No existe ninguna biblioteca **servidor** S7 en Rust (las crates `s7`/`s7-comm`
están orientadas al **cliente**). El subconjunto necesario (COTP clase 0 + S7 Read/
Write Var sobre un DB) es compacto y está bien especificado: implementarlo a mano
proporciona un control total y una superficie comprobable, coherente con los demás
instrumentos.

## 4. Política de sesiones

Se aceptan varios clientes S7 **simultáneos** (comportamiento de autómata), al
contrario que el maestro único de ORME (expulsión) y el punto a punto de OSNE (squat).
Cada sesión lee la imagen DB1 actual y enruta sus escrituras hacia la simulación;
«el último que escribe gana», como un autómata real.

## 5. Postura de seguridad

- **Ni autenticación ni cifrado** (S7 «classic»): solo la **lista blanca de IP** y la
  topología de red protegen el acceso. `0.0.0.0` + lista vacía = expuesto →
  banner de advertencia en la IHM ([`NetworkConfig::is_exposed`](../../src/config.rs)).
- **Saneamiento TOML** ([`AppConfig::sanitized`](../../src/config.rs)): proceso/
  PID/límites finitos y ordenados. Toda escritura S7 está **acotada/saneada** por
  `Regulator::apply`: la superficie de red no puede producir ni `NaN`/`Inf` ni valor
  aberrante.
- **Análisis de red acotado**: ninguna trama puede provocar un panic (cf. §3).
