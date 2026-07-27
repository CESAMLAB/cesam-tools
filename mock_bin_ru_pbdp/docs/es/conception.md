# Diseño — Regulador PROFIBUS DP simulado (ORPD)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · **ES** · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

> Crate: `mock_bin_ru_pbdp` · Ejecutable: **ru_pbdp** (*Regulation Unit over PROFIBUS DP*)

Documento de arquitectura y modelado. Calcado del regulador **ORME**
(`mock_bin_ru_modbus`) para el modelo de negocio y los actores, y de
**OSNE** (`mock_bin_su_namur`) para el enlace serie. Solo cambia la **capa
de protocolo**: un **simulador software de tramas PROFIBUS DP-V0**,
desarrollado desde cero (no existe hasta la fecha ningún crate
`profibus`/`profibus-dp` publicado en el ecosistema Rust).

---

## 1. Objetivo

Simular un **regulador de proceso** (bucle PID sobre un proceso térmico de
primer orden, modelo **idéntico** a ORME) y exponerlo mediante una
**estructura de tramas PROFIBUS DP-V0** sobre un enlace serie
(RS-485/RS-232).

**Este documento asume que el lector ha leído la advertencia de
no-interoperabilidad** (véase [`manuel_utilisateur.md`](manuel_utilisateur.md)
y [`reference_profibus.md`](reference_profibus.md) §6): el PROFIBUS DP real
exige un cumplimiento del temporizado del bus a nivel de bit (*slot time*,
`Tsdr` mín/máx, un vigilante en decenas de milisegundos) que solo un ASIC
dedicado (SPC3/VPC3) puede garantizar. Este simulador no pretende tal cosa
— es una herramienta pedagógica y de pruebas de software, no un controlador
de bus.

---

## 2. Modelo físico ([`regulator.rs`](../../src/regulator.rs))

Reutilizado tal cual del regulador ORME:
[`mock_lib_control::FirstOrderProcess`] (función de transferencia de primer
orden con retardo puro) y [`mock_lib_control::Pid`] (PID anti-windup), con
los mismos modos (Off/PID/Todo-o-nada/PWM) en ambos sentidos (calor/frío).
Paso de simulación: **50 ms**. Todas las escrituras se **sanean** en
`Regulator::apply` (límites reordenados, valores en coma flotante no
finitos ignorados, ganancias PID acotadas) — el mismo invariante que en
cualquier otro lugar del workspace: nunca llamar a `f32::clamp` con
límites no validados.

---

## 3. Arquitectura (actores)

```
IU (egui) ──Command(cast)──►  SimulationActor  ──refresh──► SharedSnapshot ──► IU
Maestro PROFIBUS (simulado) ►  (Regulator)      ──refresh──► SharedSnapshot ──► respuestas Data_Exchange
```

- **`SimulationActor`** ([`actors/simulation.rs`](../../src/actors/simulation.rs)):
  idéntico en forma a los de ORME/OSNE — propietario único del `Regulator`,
  temporizador de un solo disparo rearmado, publica el `SharedSnapshot` en
  cada paso.
- **`ProfibusServerActor`** ([`actors/network.rs`](../../src/actors/network.rs)):
  posee el enlace serie; `Reconfigure` cierra/reabre el transporte si
  cambian el puerto/baudios/dirección de estación; conserva el `JoinHandle`
  de la sesión (abortado al detenerse); publica el estado del enlace
  (`ServerStatus`, incluyendo el estado actual de la máquina de estados
  DP-V0) para la IU.
- **[`profibus.rs`](../../src/profibus.rs)** — **fuente de verdad** del
  protocolo: códec de tramas (`SD1`/`SD2`/`SD3`/`SD4`/`SC`, FCS,
  decodificación de los servicios
  (`Slave_Diag`/`Set_Prm`/`Chk_Cfg`/`Data_Exchange`) y máquina de estados
  del esclavo `SlaveFsm` (`PowerOn → WaitPrm → WaitCfg → DataExchange`).
- **[`map.rs`](../../src/map.rs)** — conversión de los bloques de bytes de
  E/S `Data_Exchange` hacia/desde los `Command` del regulador (véase
  [`reference_profibus.md`](reference_profibus.md) §3).
- **[`profibus_server.rs`](../../src/profibus_server.rs)** — bucle de
  sesión sobre cualquier flujo `AsyncRead + AsyncWrite` (el puerto serie en
  producción, un `tokio::io::duplex` en pruebas): lee una trama, la
  decodifica, llama a `SlaveFsm::handle`, aplica los `Command` resultantes,
  codifica la respuesta y la reenvía. También gestiona el **vigilante de
  protocolo** (`tokio::select!` entre la lectura de trama y un retardo,
  como el vigilante NAMUR de OSNE — pero aquí es una **parte real del
  protocolo DP**, armada por `Set_Prm`, no un añadido casero).

A diferencia de Modbus (ORME, tabla de memoria separada regenerada en cada
tick) y como en OPC UA/NAMUR, **no hay tabla de memoria persistente**: el
bloque de entrada `Data_Exchange` se recalcula al vuelo desde el
`SharedSnapshot` en el momento de responder.

**Sin política multi-maestro que gestionar**: el enlace serie *es* el
único maestro (como el RTU Modbus o el puerto serie NAMUR), a diferencia
del Modbus TCP de ORME (desalojo) o incluso del NAMUR TCP de OSNE
(punto a punto sin desalojo).

---

## 4. Códec PROFIBUS DP-V0 — decisiones y límites aceptados

- **Delimitadores de trama** (`SD1=0x10`, `SD2=0x68`, `SD3=0xA2`,
  `SD4=0xDC`, `SC=0xE5`, `ED=0x16`) y **FCS** (suma módulo 256): conformes
  a la norma, bien documentados públicamente.
- **Números SAP de los servicios de parametrización** (`Slave_Diag=61`,
  `Set_Prm=62`, `Chk_Cfg=63`): conformes.
- **Codificación exacta de los bits del campo FC**, **disposición precisa
  de los bytes de diagnóstico**, y **disposición de los bloques de
  entrada/salida** (`map.rs`): son **convenciones propias de este
  simulador**, no un perfil GSD real registrado en el PNO. El simulador
  utiliza sistemáticamente tramas **SD2** (longitud variable) para todos los
  intercambios `Data_Exchange`, incluso cuando `SD3` (8 bytes fijos)
  bastaría en un perfil real — elección que simplifica el códec sin perder
  cobertura de los conceptos del protocolo.
- **Identificador PROFIBUS** (`Ident_Number = 0xEE01`): **ficticio**, no
  registrado ante el PNO (PROFIBUS & PROFINET International) — no
  representa ningún dispositivo de catálogo real.
- **Sin ningún temporizado de bus**: ni ventana de respuesta (`Tsdr`), ni
  testigo, ni arbitraje multi-maestro están implementados — véase §1.

Detalle completo en [`reference_profibus.md`](reference_profibus.md).

---

## 5. Configuración y persistencia

`AppConfig` (idioma / enlace serie / proceso / regulación / verificación de
actualizaciones) serializado en **TOML** ([`config.rs`](../../src/config.rs)),
**saneado al cargar** (`AppConfig::sanitized`: límites ordenados,
`τ ≥ 1e-3`, `dead_time ≥ 0`, valores en coma flotante finitos, dirección de
estación acotada a `[0, 125]`). Archivo: `mock_ru_pbdp.toml`
(sobrescribible mediante `MOCK_CONFIG`). A diferencia de ORME/OSNE, **no
hay lista blanca de IP** (el enlace serie es intrínsecamente punto a punto,
sin noción de dirección de red).

---

## 6. Vías de evolución

- Una auténtica herramienta de **maestro PROFIBUS DP simulado** (binario
  separado), que utilice las mismas funciones de codificación/decodificación
  expuestas para pruebas en `profibus.rs`, para pilotar este simulador o
  cualquier otro esclavo software sin depender de un script ad hoc.
- Generación de un archivo **GSD** ilustrativo (no funcional del lado del
  simulador) que documente el perfil de E/S simulado, con fines
  pedagógicos.
- Soporte de **DP-V1** (acceso acíclico, alarmas) si surge la necesidad
  pedagógica — fuera del alcance inicial (solo DP-V0).
- Promoción del modelo de regulador a una `mock_lib_*` compartida (hoy
  duplicado entre ORME y este instrumento, como con ORUE).
