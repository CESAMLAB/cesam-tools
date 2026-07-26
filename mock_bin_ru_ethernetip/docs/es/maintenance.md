# Mantenimiento — Regulador EtherNet/IP (OREE)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · **ES** · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

---

## 1. Build y lanzamiento

```bash
cargo run -p mock_bin_ru_ethernetip                        # IHM + adaptador EtherNet/IP
cargo build -p mock_bin_ru_ethernetip --release            # ejecutable IHM
cargo build -p mock_bin_ru_ethernetip --no-default-features # headless (sin IHM)
```

Features: `gui` (IHM `egui`, por defecto). `--no-default-features` produce un binario
**headless**: adaptador EtherNet/IP + simulación, sin IHM ni verificación de
actualizaciones. El puerto 44818 no requiere **ningún** privilegio.

## 2. Configuración

Archivo TOML `mock_ru_ethernetip.toml` (directorio actual; ruta reemplazable por
`MOCK_CONFIG`). Secciones: `language`, `[network]` (`bind_ip`, `port`, `allowlist`),
`[process]`, `[regulation]`, `check_updates`. Todo valor es **saneado** al
cargar.

## 3. Pruebas

```bash
cargo test -p mock_bin_ru_ethernetip      # unitarias + round-trip TCP local
```

- **Capa de protocolo** (`eip_server`, sin red): RegisterSession, Read/Write Tag,
  escritura BOOL, tag desconocido (`0x05`), escritura de un tag de solo lectura, **no-pánico**
  con paquetes malformados.
- **Actor de red**: bind/escucha y un **round-trip TCP real** (RegisterSession,
  Write y luego Read de la consigna) — sin dependencia de un cliente externo.

## 4. Resolución de problemas

| Síntoma | Pista |
|---|---|
| Cliente rechazado | lista blanca de IP; cortafuegos; IP/puerto (44818) |
| Tag no encontrado | nombre inexacto (mayúsculas/minúsculas); ver la tabla de tags |
| Escritura sin efecto | tag de solo lectura |
| Valores incoherentes | EtherNet/IP es **little-endian** (REAL = `f32` LE) |

## 5. Docker (headless)

Imagen headless mediante `scripts/build-prod.sh` (entrada
`mock_bin_ru_ethernetip:ru_eip:44818`, `EXPOSE 44818`). Montar un volumen sobre el
directorio de trabajo para proporcionar el `mock_ru_ethernetip.toml`.

## 6. Ampliar la tabla de tags

La tabla de tags y el mapeo de las escrituras son la **fuente de verdad** en
[`eip_server.rs`](../../src/eip_server.rs) (`read_tag` + `write_tag`). Para añadir un
tag: añadirlo a `read_tag` (lectura) y, si es controlable, a `write_tag` (escritura →
`Command`), luego reflejarlo aquí y en
[`reference_ethernetip.md`](reference_ethernetip.md). Añadir una prueba en el módulo.

## 7. Cross / Windows

Como los demás instrumentos (cf. `Cross.toml`). Ninguna dependencia nativa
particular: la capa EtherNet/IP es 100 % Rust sobre TCP estándar.
