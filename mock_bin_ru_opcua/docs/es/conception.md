# Diseño — Regulador de proceso simulado (RU/OPC UA)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · **ES** · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

> Crate: `mock_bin_ru_opcua` · Ejecutable: **ru_opcua** (*Regulation Unit over OPC UA*)

Documento de arquitectura y de modelado. Calcado del regulador **ORME**
(`mock_bin_ru_modbustcp`): mismo reparto **modelo de negocio síncrono / actores
ractor / capa de protocolo / IHM egui**, mismos invariantes. Solo cambia el
**transporte**: **OPC UA** en lugar de Modbus.

---

## 1. Objeto

Simular un **regulador de proceso** (bucle PID sobre un proceso térmico de
primer orden) y exponerlo mediante **OPC UA**, el estándar de supervisión
industrial (Industria 4.0). A diferencia de ORME (Modbus) y OSNE (NAMUR) —
protocolos **de campo sin seguridad** — OPC UA incorpora de forma nativa
la autenticación, la firma y el cifrado (previstos en la Fase 2).

---

## 2. Modelo físico ([`regulator.rs`](../../src/regulator.rs))

El **proceso** reutiliza [`mock_lib_control::FirstOrderProcess`] (compartido con
ORME): función de transferencia de primer orden con retardo puro

```text
PV(s) / U(s) = K · e^(−L·s) / (1 + τ·s)
```

- `PV`: medida (unidad de proceso, p. ej. °C);
- `U`: mando / salida (0-100 %);
- `K`: ganancia estática; `τ`: constante de tiempo; `L`: retardo puro;
- `ambient`: valor en reposo (salida nula).

Un **PID** ([`mock_lib_control::Pid`], también reutilizado de ORME) regula la
medida hacia la **consigna** controlando la salida, acotada a `[0, 100]`. Dos modos:
**automático** (el PID calcula la salida) y **manual** (salida impuesta). El paso
de simulación es de **0,5 s** (proceso térmico lento).

Todas las escrituras (red o IHM) se **sanean** en `Regulator::apply`:
flotantes no finitos ignorados, consigna acotada, límites reordenados (`min ≤ max`),
ganancias PID acotadas. **Invariante: nunca usar `f32::clamp` con límites no
validados** (panic si `min > max` o `NaN`).

---

## 3. Arquitectura (actores)

```
IHM (egui) ───Command(cast)──►  SimulationActor ──refresh──► SharedSnapshot ──► IHM
Servidor OPC UA ─Command(cast)─►   (Regulator)    ──refresh──► SharedSnapshot ──► lecturas OPC UA
```

- **`SimulationActor`** ([`actors/simulation.rs`](../../src/actors/simulation.rs)):
  propietario **único** del `Regulator`; avanza la simulación con un temporizador
  one-shot rearmado (sin temporizador desligado) y publica un `SharedSnapshot` en cada
  paso.
- **`OpcuaServerActor`** ([`actors/network.rs`](../../src/actors/network.rs)):
  posee el servidor OPC UA (tarea tokio `server.run()`); reiniciable en caliente
  (`Reconfigure`: rebind si la IP/puerto cambia); conserva el `JoinHandle` (abandono
  al detenerse) y el `ServerHandle` (cancelación limpia de las sesiones); publica su
  estado de escucha para la IHM.
- **Servidor OPC UA** ([`opcua_server.rs`](../../src/opcua_server.rs)): construye el
  servidor [`async-opcua`](https://crates.io/crates/async-opcua), declara el espacio
  de direcciones y conecta los callbacks. Las **lecturas** toman datos del
  `SharedSnapshot`; las **escrituras** emiten un `Command` hacia el
  `SimulationActor` por `cast` no bloqueante.

Como NAMUR (OSNE) y a diferencia del Modbus de ORME, no hay **tabla de
memoria separada**: los nodos OPC UA leen directamente la instantánea compartida.

---

## 4. Pila OPC UA — decisiones técnicas

- **`async-opcua`** (servidor, feature `server`): implementación **nativa de tokio**
  (una tarea por conexión), que se integra en la pila ractor/tokio. Cripto
  **100 % Rust** (RustCrypto: `rsa`, `aes`, `sha2`, `x509-cert`) — **ninguna
  dependencia de OpenSSL**, lo que preserva la compilación cruzada (Linux/Windows/RPi).
- **Espacio de direcciones**: un `SimpleNodeManager` en memoria; nodos `Variable`
  organizados bajo `Objects` (cf. [`reference_opcua.md`](reference_opcua.md)).
- **Callbacks**: `add_read_callback` (valor vivo, muestreado para las
  suscripciones) y `add_write_callback` (encamina hacia la simulación).
- **Licencia**: `async-opcua` está bajo **MPL-2.0** (toda la estirpe OPC UA en Rust
  lo está). Copyleft **por archivo**: uso sin modificar → el código CESAM-Lab sigue
  siendo MIT (cf. archivo `NOTICE` en la raíz).

---

## 5. Seguridad

La seguridad es **configurable** (`SecurityConfig`) y constituye el diferenciador
de OPC UA frente a los protocolos de campo (Modbus/NAMUR, sin seguridad).

- **Modo sin cifrar (por defecto)**: un endpoint `SecurityPolicy::None`, token
  **anónimo** — red de confianza únicamente, arranque instantáneo, ningún
  certificado. La IHM muestra un **banner naranja** de advertencia.
- **Modo cifrado (Fase 2)**: endpoint `Basic256Sha256` / `SignAndEncrypt`. Se
  genera un **certificado de instancia** autofirmado en el primer arranque (`pki/`);
  el servidor confía en los certificados de cliente. **Autenticación** por
  usuario/contraseña (`ServerUserToken::user_pass`) o anónima. La IHM
  muestra un **banner verde** 🔒.

El modo se configura en el modal *Parámetros*; un cambio **reinicia** el servidor
en caliente (`OpcuaServerActor`).

---

## 6. Configuración y persistencia

`AppConfig` (idioma / red / proceso / regulación / verif. actualización) serializada en
**TOML** ([`config.rs`](../../src/config.rs)), **saneada al cargar**
(`AppConfig::sanitized`: límites ordenados, `τ ≥ 1e-3`, `dead_time ≥ 0`, flotantes
finitos). Archivo: `mock_ru_opcua.toml` (anulable mediante `MOCK_CONFIG`).

---

## 7. Líneas de evolución

- **Fase 2**: seguridad OPC UA (certificados, cifrado, auth).
- Métodos OPC UA (`Reset`, `Autotune`) además de las variables.
- Modelo de información tipado (ObjectType regulador) en lugar de variables planas.
- Historización / `HistoryRead` sobre la medida.
- Promoción del modelo regulador de ORME a una `mock_lib_*` compartida (hoy está
  duplicado entre ORME y este instrumento).
