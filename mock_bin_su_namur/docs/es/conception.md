# Diseño — Agitador de laboratorio simulado (OSNE)

*🌍 [FR](../fr/conception.md) · [EN](../en/conception.md) · [DE](../de/conception.md) · **ES** · [IT](../it/conception.md) · [PT](../pt/conception.md) · [NL](../nl/conception.md) · [PL](../pl/conception.md)*

> Crate: `mock_bin_su_namur` · Ejecutable: **OSNE** (*Open Stirrer NAMUR Emulator*)

Documento de arquitectura y modelado. Calcado del regulador **ORME**
(`mock_bin_ru_modbustcp`): mismo reparto **modelo de negocio síncrono / actores
ractor / capa de protocolo / IHM egui**, mismos invariantes.

---

## 1. Objetivo

Simular un **agitador de laboratorio** (estilo IKA) controlado por el protocolo
serie **NAMUR**. El motor posee una **función de transferencia** (dinámica de
velocidad) regulada por un **lazo de regulación rápido**, y la **viscosidad** del
medio es ajustable e influye sobre el par.

---

## 2. Modelo físico

### Motor ([`motor.rs`](../../src/motor.rs))

Equilibrio de pares, integrado por Euler explícito:

```text
J · dω/dt = T_moteur − k · η · ω − T_frottement
```

- `ω`: velocidad (tr/min);
- `T_moteur`: par motor (consigna, N·cm, ≥ 0);
- `k · η · ω`: **par de carga viscoso** (∝ viscosidad `η` y velocidad);
- `T_frottement`: rozamiento seco residual;
- `J` (`inertia`): ajusta la **reactividad** (pequeño ⇒ rápido).

En régimen permanente, `T_moteur = k·η·ω + T_frottement`: el par necesario para
mantener una velocidad **crece con la viscosidad**. Si ese par supera el **par
máximo**, la consigna deja de ser alcanzable → **sobrecarga**.

### Lazo de regulación ([`stirrer.rs`](../../src/stirrer.rs))

Un **PID** ([`mock_lib_control::Pid`], reutilizado de ORME) toma el error de
velocidad `consigna − medida` y produce el **par motor**, acotado a
`[0, couple_max]`. Las ganancias por defecto son deliberadamente «duras»: la
salida satura al par máximo mientras el error es grande (subida rápida), y luego
el término integral estabiliza. El paso de simulación es de **20 ms** (50 Hz), más
fino que el de ORME porque la dinámica de un motor es rápida.

---

## 3. Arquitectura (actores)

```
IHM (egui) ──Command(cast)──►  SimulationActor ──refresh──► SharedSnapshot ──► IHM
Serveur NAMUR ──Command(cast)─►   (Stirrer)     ──refresh──► SharedSnapshot ──► lectures NAMUR
```

- **`SimulationActor`** ([`actors/simulation.rs`](../../src/actors/simulation.rs)):
  propietario único del `Stirrer`; avanza la simulación sobre un temporizador
  one-shot rearmado (sin temporizador desacoplado) y publica un `SharedSnapshot`.
- **`NamurServerActor`** ([`actors/network.rs`](../../src/actors/network.rs)):
  posee el servidor NAMUR, relanzable en caliente (`Reconfigure`); lista blanca de
  IP compartida; estado de escucha publicado para la IHM.
- **Servidor NAMUR** ([`namur_server.rs`](../../src/namur_server.rs)): lee las
  líneas ASCII, las interpreta ([`namur.rs`](../../src/namur.rs)), responde a las
  lecturas y reenvía las escrituras/acciones al actor. **Un maestro a la vez**
  (punto a punto). **Perro guardián** por sesión.

Las lecturas NAMUR se sirven del `SharedSnapshot` (sin tabla de memoria separada
como el Modbus de ORME: el protocolo NAMUR está orientado a «comandos», no a
«registros»).

---

## 4. Configuración y seguridad

- `AppConfig` (idioma / red-serie / motor / regulación) serializada en **TOML**
  ([`config.rs`](../../src/config.rs)), **saneada al cargar**
  (`AppConfig::sanitized`: límites ordenados, flotantes finitos) — invariante
  compartido con ORME (nunca hacer `clamp` con límites no validados).
- NAMUR **no tiene ni autenticación ni cifrado**: red de confianza + lista blanca
  de IP (TCP). Por defecto `0.0.0.0` + lista vacía ⇒ expuesto: la IHM muestra un
  **banner de advertencia**.

---

## 5. Posibles evoluciones

- Sentido de rotación (CW/CCW) y rampa de aceleración.
- Sensor de temperatura (`IN_PV_2/3`) si se añade un modelo térmico.
- Par de carga no lineal (régimen turbulento ∝ ω²).
- Promoción del modelo del motor a `mock_lib_control` si sirve a un segundo
  instrumento.
