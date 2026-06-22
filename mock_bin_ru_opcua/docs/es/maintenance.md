# Documentación de mantenimiento — RU/OPC UA (workspace `cesam-tools`)

*🌍 [FR](../fr/maintenance.md) · [EN](../en/maintenance.md) · [DE](../de/maintenance.md) · **ES** · [IT](../it/maintenance.md) · [PT](../pt/maintenance.md) · [NL](../nl/maintenance.md) · [PL](../pl/maintenance.md)*

> Crate: `mock_bin_ru_opcua` · Ejecutable: **ru_opcua**

---

## 1. Requisitos previos

- **Rust** reciente. ⚠️ MSRV propia de este crate: **1.91** (`async-opcua` no declara
  ningún `rust-version` y arrastra dependencias recientes; el resto del workspace
  está en 1.85).
- Para la IHM: las dependencias de sistema de `eframe`/`egui` (las mismas que ORME/OSNE).
- Para el build *headless*: ninguna dependencia gráfica.

---

## 2. Comandos habituales

```bash
cargo run -p mock_bin_ru_opcua                       # IHM + servidor OPC UA
cargo run -p mock_bin_ru_opcua --no-default-features # headless (sin IHM)
cargo test -p mock_bin_ru_opcua                      # pruebas unitarias
cargo clippy -p mock_bin_ru_opcua --all-targets      # lint
MOCK_CONFIG=./ma_config.toml cargo run -p mock_bin_ru_opcua  # configuración alternativa
```

### Features Cargo

- **`gui`** (por defecto): interfaz gráfica `egui` + verificación de actualización.
- `--no-default-features`: binario **headless** (servidor OPC UA + simulación,
  sin IHM ni red de actualización).

El servidor `async-opcua` está **siempre** presente (la feature `server` de
`async-opcua`), pues es la razón de ser del instrumento.

---

## 3. Organización del código

```
mock_bin_ru_opcua/src/
├── main.rs            # Ensambla runtime Tokio + actores + IHM/headless
├── regulator.rs       # Modelo de negocio síncrono (PID + proceso), comandos, paso
├── config.rs          # AppConfig (TOML), sanitized(), ServerStatus
├── i18n.rs            # Catálogo i18n (8 idiomas), Lang + Msg + tr()
├── opcua_server.rs    # Servidor OPC UA: build + espacio de direcciones + callbacks
├── gui.rs             # IHM egui (feature gui)
├── branding.rs        # Logos embebidos (feature gui)
└── actors/
    ├── simulation.rs  #   bucle de regulación (tick 0,5 s)
    └── network.rs     #   servidor OPC UA (re)configurable en caliente
```

---

## 4. Configuración

`AppConfig` (idioma / red / proceso / regulación / `check_updates`) está
serializada en **TOML** (`mock_ru_opcua.toml`, anulable mediante `MOCK_CONFIG`),
cargada al arrancar (valores por defecto si está ausente), guardada desde la IHM. Todo valor
se **sanea** al cargar (`AppConfig::sanitized`: límites ordenados,
`τ ≥ 1e-3`, `dead_time ≥ 0`, flotantes finitos).

**Invariante**: nunca usar `f32::clamp` con límites no validados (panic
si `min > max` o `NaN`). Las escrituras de red también pasan por
`Regulator::apply`, que sanea.

### Verificación de actualización

Solo feature `gui`: al arrancar, la IHM consulta la última release
de GitHub mediante la lib compartida `mock_lib_update` (hilo acotado por timeout) y muestra
un banner si existe una versión más reciente. Ajustable mediante `check_updates`.

---

## 5. Dependencias y trampas de versión

- **`async-opcua` 0.18** (servidor). Cripto **100 % Rust** (RustCrypto): **ninguna
  dependencia de OpenSSL** → compilación cruzada limpia. Licencia **MPL-2.0** (cf. `NOTICE`).
- ⚠️ `async-opcua` no declara **ninguna MSRV**: validar en la toolchain destino antes
  de subir la versión.
- ⚠️ La generación de certificado (`create_sample_keypair(true)`) está **deliberadamente
  desactivada**: la generación RSA en Rust puro es muy lenta en *debug* y escribiría
  en `pki/`. En Fase 1b (endpoint None), no se requiere ningún certificado.
- `egui_plot` se mantiene **una versión menor por delante** de `egui` (cf. ORME/OSNE).

---

## 6. Extender el proyecto

### 6.1 Añadir un nodo OPC UA

En [`opcua_server.rs`](../../src/opcua_server.rs): declarar el nodo
(`add_var`), conectar un callback de lectura (`on_read_*`) y, si es escribible, un
callback de escritura (`on_write_*`) que emita un `Command`. Reflejar la tabla en
[`reference_opcua.md`](reference_opcua.md).

### 6.2 Añadir un comando de negocio

Extender el enum `Command` ([`regulator.rs`](../../src/regulator.rs)), gestionar el caso
en `Regulator::apply` (con saneamiento), añadir una prueba.

### 6.3 Añadir una cadena de interfaz (i18n)

Añadir una variante a `Msg` ([`i18n.rs`](../../src/i18n.rs)) y **las 8
traducciones** (tabla de tamaño fijo verificada en compilación).

### 6.4 Fase 2 — seguridad

Activar un endpoint cifrado (`Basic256Sha256`), aprovisionar un certificado
de instancia, añadir la autenticación de usuario. Retirar entonces el filtro de log
`opcua_crypto::certificate_store=off` puesto en [`main.rs`](../../src/main.rs).

---

## 7. Estrategia de pruebas

El núcleo de negocio (`regulator.rs`) y la configuración (`config.rs`) son **puros y
probados**: convergencia PID, clamp de consigna, relajación al parar, cambio de
proceso sin salto de PV, saneamiento TOML, ida y vuelta TOML. La i18n verifica la
no vacuidad y la ida y vuelta de idioma. La lógica async (actores, servidor) se mantiene
ligera y se apoya en estos bloques probados.

---

## 8. Resolución de problemas

| Síntoma | Causa probable | Remedio |
|---|---|---|
| `failed to bind` al arrancar | puerto ya ocupado / < 1024 sin privilegios | cambiar el puerto (*Parámetros*) o ejecutar como root |
| El cliente no ve los nodos | endpoint / seguridad incorrectos | `opc.tcp://…:4840/`, None, Anonymous; *Browse* bajo `Objects` |
| Escritura `Bad_TypeMismatch` | tipo incorrecto | `Double` para las magnitudes, `Boolean` para `Run`/`Auto` |
| WARN «encrypted endpoints disabled» | ningún certificado (Fase 1b) | normal; el endpoint None funciona |

---

## 9. Build «prod» — compilación cruzada desde Linux

El instrumento está integrado en [`scripts/build-prod.sh`](../../../scripts/build-prod.sh)
(tabla `INSTRUMENTS`): exes **con IHM** para Linux x86_64, Windows x86_64 y
Raspberry Pi arm64 (mediante `cross`), más una imagen Docker headless.

⚠️ **Cross Windows y `GetHostNameW`**: la pila OPC UA arrastra `gethostname`, que hace
referencia al símbolo winsock `GetHostNameW`. La biblioteca de importación mingw-w64 de
la imagen `cross` **por defecto** (`:0.2.5`) es demasiado antigua para proporcionarlo →
fallo en la edición de enlaces. El repositorio fija por tanto, en [`Cross.toml`](../../../Cross.toml),
la imagen Windows GNU en **`:main`** (mingw reciente). Validado: los builds headless **y**
IHM producen un `.exe` válido; ORME/OSNE siguen compilando (imagen superconjunto).

---

## 10. Convenciones

- Código y comentarios en **francés**; logs/errores en **inglés**.
- Cadenas IHM mediante `i18n` (8 idiomas); nunca codificadas en duro.
- Lógica de negocio **síncrona y comprobable**; lo asíncrono se limita a los actores
  y a la IO. `cargo clippy --workspace` sin advertencias.
- Invariantes `ractor`: ningún guard `Mutex` a través de un `.await`; ningún
  temporizador/`spawn` desligado sin `JoinHandle` abandonado al detenerse.
