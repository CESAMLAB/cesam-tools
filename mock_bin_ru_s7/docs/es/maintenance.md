# Mantenimiento — Regulador S7 (ORSS)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · **ES** · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

---

## 1. Compilación y lanzamiento

```bash
cargo run -p mock_bin_ru_s7                        # IHM + servidor S7
cargo build -p mock_bin_ru_s7 --release            # ejecutable IHM
cargo build -p mock_bin_ru_s7 --no-default-features # headless (sin IHM)
```

Features: `gui` (IHM `egui`, por defecto). `--no-default-features` produce un binario
**headless**: servidor S7 + simulación, sin IHM ni verificación de actualizaciones.

⚠️ El puerto **102** (S7 estándar) es privilegiado (< 1024): ejecutar con los derechos
adecuados o elegir un puerto alto en la configuración.

## 2. Configuración

Archivo TOML `mock_ru_s7.toml` (directorio actual; ruta reemplazable mediante
`MOCK_CONFIG`). Secciones: `language`, `[network]` (`bind_ip`, `port`, `allowlist`),
`[process]`, `[regulation]`, `check_updates`. Todo valor se **sanea** en la
carga.

## 3. Pruebas

```bash
cargo test -p mock_bin_ru_s7      # unitarias + round-trip TCP local
```

- **Capa de protocolo** (`s7_server`, sin red): CR→CC, Setup, Read/Write Var,
  escritura de bit, código de retorno fuera de zona, **no-panic** ante tramas mal
  formadas, round-trip de la imagen DB.
- **Actor de red**: bind/escucha, y un **round-trip TCP real** (conexión COTP,
  escritura y posterior relectura de la consigna mediante tramas S7 en bruto) — sin
  dependencia de un cliente externo.

## 4. Resolución de problemas

| Síntoma | Pista |
|---|---|
| El bind falla (`permission denied`) | puerto 102 < 1024 → derechos root o puerto alto |
| Cliente rechazado | lista blanca de IP; cortafuegos; IP/puerto |
| Sin respuesta | rack/slot (probar 0/1, 0/2); tramas fuera del subconjunto ignoradas |
| Escritura sin efecto | offset de solo lectura (cf. plan de direccionamiento) |

## 5. Docker (headless)

Imagen headless mediante `scripts/build-prod.sh` (entrada `mock_bin_ru_s7:ru_s7:102`,
`EXPOSE 102`). Montar un volumen en el directorio de trabajo para proporcionar el
`mock_ru_s7.toml`. El contenedor publica el puerto 102; mapear hacia un puerto alto del
lado del host si es necesario.

## 6. Ampliar el plan de direccionamiento

El plan DB1 y el mapeo de las escrituras son la **fuente de verdad** en
[`s7_server.rs`](../../src/s7_server.rs) (`db_image` + `handle_write`). Para añadir
una magnitud: escribirla en `db_image` (lectura) y, si es pilotable, añadirla al
`match` de `handle_write` (escritura → `Command`), después reflejarlo aquí y en
[`reference_s7.md`](reference_s7.md). Añadir una prueba en el módulo.

## 7. Cross / Windows

Como los demás instrumentos (cf. `Cross.toml`). Ninguna dependencia nativa
particular: la capa S7 es 100 % Rust sobre TCP estándar.
