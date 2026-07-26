# Diseño — Regulador EtherNet/IP (OREE)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · **ES** · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

---

## 1. Visión general

OREE reutiliza la arquitectura de los demás instrumentos CESAM-Lab: **modelo de negocio
síncrono y comprobable** (PID + proceso), **actores `ractor`** sobre Tokio, **IHM
`egui`** que lee una instantánea compartida. Solo cambia la **capa de transporte**: un
**adaptador EtherNet/IP** (encapsulación + CIP) en lugar de Modbus/OPC UA/S7.

```
        Command (cast)                      refresh cada paso
IHM   ───────────────────►  SimulationActor ───────────────────►  SharedSnapshot (IHM)
CIP Write Tag ───────────►  (Regulator)      ──────────────────►  SharedSnapshot
CIP Read Tag  ◄────────────────────────────────  SharedSnapshot
```

## 2. Actores

- **`SimulationActor`** — posee el único [`Regulator`]; aplica los `Command`
  (IHM o escrituras CIP); publica la instantánea tras cada mutación.
- **`EipServerActor`** — posee el **bucle de escucha TCP**. Una tarea tokio enlaza el
  socket y acepta los clientes; cada sesión (con su *session handle*) la lleva
  un `JoinSet` **interno** (abatido junto con el bucle, sin ninguna tarea
  desvinculada). `Reconfigure` reinicia la escucha si cambia la IP/puerto y actualiza la
  **lista blanca** compartida.

## 3. Capa de protocolo

[`eip_server.rs`](../../src/eip_server.rs) es **puro y síncrono**: encapsulación
EtherNet/IP (`RegisterSession`, `SendRRData`/CPF) y CIP (`Read Tag`/`Write Tag` por
segmento simbólico). Todo es **little-endian**. El parsing está **acotado** (slices
verificados): un paquete malformado proveniente de la red **nunca** provoca un pánico,
solo una ausencia de respuesta. Es el equivalente de `opcua_server.rs`, aislado para
ser **comprobable sin socket**.

### Por qué un adaptador hecho a mano

No existe ninguna biblioteca **servidor/adaptador** EtherNet/IP en Rust (las
crates `rseip`, `rust-ethernet-ip`, `cip` están orientadas a **cliente/scanner**). El
subconjunto necesario (encapsulación + CIP Read/Write Tag sobre tags nombrados) es
compacto: implementarlo a mano da un control total y una superficie comprobable,
coherente con los demás instrumentos.

## 4. Política de sesiones

Se aceptan varios clientes **simultáneos** (comportamiento de un adaptador), al
contrario que el mono-maestro de ORME. Cada sesión recibe un *session handle* y lee
la instantánea actual; «el último que escribe gana».

## 5. Postura de seguridad

- **Ni autenticación ni cifrado** (EtherNet/IP «classic»): solo la **lista
  blanca de IP** y la topología de red protegen el acceso. `0.0.0.0` + lista vacía =
  expuesto → banner de advertencia ([`NetworkConfig::is_exposed`](../../src/config.rs)).
- **Saneamiento TOML** ([`AppConfig::sanitized`](../../src/config.rs)): proceso/
  PID/límites finitos y ordenados. Toda escritura CIP es **acotada/saneada** por
  `Regulator::apply`: la superficie de red no puede producir ni `NaN`/`Inf` ni valor
  aberrante.
- **Parsing de red acotado**: ningún paquete puede provocar un pánico (cf. §3).
