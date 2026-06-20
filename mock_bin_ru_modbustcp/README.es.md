# ORME — regulador simulado Modbus

*🌍 [English](README.md) · [Français](README.fr.md) · [Deutsch](README.de.md) · **Español** · [Italiano](README.it.md) · [Português](README.pt.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md)*

> *Open Regulator Modbus Emulator* · paquete `mock_bin_ru_modbustcp` · binario `orme`

Regulador industrial **simulado**, esclavo **Modbus TCP/RTU**, con interfaz
gráfica. Forma parte del workspace [`cesam-tools`](../README.es.md).

## Funcionalidades

- Proceso de primer orden + retardo puro (función de transferencia FOPDT).
- Regulación bidireccional (calor / frío), cada sentido en **PID** o
  **todo-o-nada**.
- Modos marcha/paro y auto/manual; consignas auto (física) y manual (%).
- Servidor Modbus TCP que expone la totalidad del estado.
- IHM `egui` con curva de tendencia en tiempo real y ajuste de las ganancias PID.
- **Interfaz multilingüe**: francés, inglés, alemán, español, italiano,
  portugués, neerlandés, polaco (elección en el modal *Parámetros*, persistido).

## Lanzar

```bash
cargo run -p mock_bin_ru_modbustcp
# Archivo de configuración alternativo:
MOCK_CONFIG=./ma_config.toml cargo run -p mock_bin_ru_modbustcp
```

Escucha por defecto en `0.0.0.0:5502`. El puerto, la IP de escucha y la lista blanca
de IP se ajustan en el modal **⚙ Parámetros** y se persisten en TOML.

## Tabla de direcciones Modbus

Codificación de los flotantes: 2 registros, big-endian, palabra de mayor peso primero.

### Bobinas (FC 1/5/15)

| Dir | Rol |
|----|------|
| 0 | Marcha (1) / Paro (0) |
| 1 | Auto (1) / Manual (0) |

### Entradas discretas (FC 2, solo lectura)

| Dir | Rol |
|----|------|
| 0 | En marcha |
| 1 | Sentido 1 (calor) activo |
| 2 | Sentido 2 (frío) activo |

### Registros de mantenimiento (FC 3/6/16)

| Dir | Tipo | Rol |
|-----|------|------|
| 0 | u16 | Modo sentido 1 (0=Off, 1=PID, 2=TOR) |
| 1 | u16 | Modo sentido 2 (0=Off, 1=PID, 2=TOR) |
| 2–3 | f32 | Consigna automática (SP) |
| 4–5 | f32 | Consigna manual (% salida, con signo) |
| 6–7 | f32 | Kp sentido 1 |
| 8–9 | f32 | Ki sentido 1 |
| 10–11 | f32 | Kd sentido 1 |
| 12–13 | f32 | Kp sentido 2 |
| 14–15 | f32 | Ki sentido 2 |
| 16–17 | f32 | Kd sentido 2 |
| 18–19 | f32 | Histéresis TOR |

### Registros de entrada (FC 4, solo lectura)

| Dir | Tipo | Rol |
|-----|------|------|
| 0–1 | f32 | Medida (PV) |
| 2–3 | f32 | Salida aplicada (% con signo: + calor / − frío) |

La fuente de verdad es la cabecera de [`src/map.rs`](src/map.rs).

## Documentación

Documentación propia de esta aplicación (carpeta [`docs/es/`](docs/es/)):

- [**Manual de usuario**](docs/es/manuel_utilisateur.md) — primeros pasos, control, parámetros, FAQ.
- [Documento de diseño](docs/es/conception.md) — arquitectura, decisiones técnicas, teoría de regulación.
- [Tabla de direcciones Modbus](docs/es/table_modbus.md) — plan de direccionamiento completo, codificación, ejemplos.
- [Mantenimiento del software](docs/es/maintenance.md) — build, configuración, ampliación, resolución de problemas.
