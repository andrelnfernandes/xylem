# Layouts: archivos de ancho fijo y multirregistro

Muchos archivos que se intercambian entre sistemas son texto sin nombres de columna: cada línea es un registro, y qué caracteres guardan qué campo está escrito en un manual. Los archivos bancarios (CNAB 240), los fiscales (SPED) y la exportación SDF de dBase y Clipper son así. Un layout le dice a Xylem dónde está cada campo, y el visor muestra el archivo como tabla: una hoja por tipo de registro, cada fila empezando por su línea en el archivo.

## Abrir un archivo con un layout

Xylem reconoce algunos archivos por su primera línea y los abre como tabla por sí solo, con `V` y en Autoview (F7). La nota dice qué layout usó: `Layout: FEBRABAN CNAB 240 — Padrão Geral  ·  K changes it`. Para cualquier otro archivo, pulse `K`.

| Tecla | Qué hace |
|---|---|
| `K` | la lista de layouts: escriba parte de un nombre para acotarla, Enter aplica uno, **F2** pide la ruta de un archivo de layout, Esc vuelve. Con un layout en uso, el primer elemento lo desactiva y muestra el archivo como texto |
| `[` `]` | tipo de registro anterior o siguiente |
| `Tab` | la lista de tipos de registro, con cuántas líneas tiene cada uno |
| `i` | los campos del registro: posición, tamaño, tipo, descripción y el significado de cada código; encima, el layout, su fuente, las comprobaciones de los totales y sus resultados |
| `T` | al texto, en la línea del registro bajo el cursor, y de vuelta |
| `\` o `&` | el filtro SQL, por nombre de campo: `valor_pagamento > 1000` |
| Ctrl+B | un Batch de los registros de la hoja abierta ([Batch en el visor](execute.md)) |

Las hojas aparecen en el orden en que cada tipo de registro aparece por primera vez en el archivo: `0 - Header de Arquivo (1)`, `3A - Segmento A (420)`... Una línea que no corresponde a ningún registro va entera a la hoja `! No record type`. La primera columna, `#line`, es la línea en el archivo, que es como los validadores de bancos y del fisco señalan un error. Cuando un campo tiene códigos con significado, la última línea de la cuadrícula lo muestra: `Content: 2 — Retorno (Banco → Cliente)`.

Los valores se leen como dice el layout: los códigos conservan sus ceros (`00012345`), las fechas pasan a `2026-11-09`, las horas a `14:30:00`, y los importes conservan sus decimales exactos (`1234.50`); nada se redondea. Un valor que no es lo que dice el layout (letras en un número, el 31 de febrero) se muestra tal como está escrito, e `i` los cuenta.

### Comprobación de los totales

Los trailers llevan totales: las líneas de un lote, el número de lotes, la suma de los pagos. Xylem los cuenta y los suma al leer, en enteros, y dice en `i` cuáles coinciden y cuáles no, con la línea, lo que dice el archivo y lo que se contó. Cuando alguno no coincide, la cabecera lo avisa: `1 check failed: i shows`. Un archivo leído solo en parte (detenido con Esc o por el límite de tiempo) no tiene comprobaciones.

## De dónde vienen los layouts

| Dónde | ¿Se usa solo? |
|---|---|
| El catálogo de Xylem, incluido en el programa | sí, cuando la primera línea corresponde a su `[detect]` |
| Su carpeta: `~/.config/xylem/layouts/` (`%APPDATA%\xylem\layouts\` en Windows), hasta tres niveles de subcarpetas | sí, y va antes que el catálogo |
| Junto al archivo (`K`, sección `Here`), o cualquier ruta (`K` y luego F2) | solo cuando usted lo elige |

Un archivo de layout en la carpeta que se está viendo nunca se lee solo: una carpeta descargada o extraída podría traer uno hecho para engañar. Cuando usted lo elige, se lee con el mismo rigor que los datos (vea Seguridad, abajo).

El catálogo tiene, por ahora:

| Layout | Qué lee |
|---|---|
| FEBRABAN CNAB 240 — Padrão Geral | cabecera y trailer de archivo; pagos por crédito en cuenta, cheque, OP, TED y Pix (segmentos A, B, C, Z); pago de boletos de cobro (J, J-52); cobro (P, Q, R en el envío; T y U en el retorno). Del manual de FEBRABAN, versión 11.0 |
| SPED — genérico | cualquier archivo SPED de campos entre barras verticales (EFD ICMS/IPI, EFD-Contribuições, ECD, ECF...): una hoja por registro, la apertura y el cierre de cada bloque y el bloque 9 con sus comprobaciones. Los registros que no declara tienen los campos numerados (`CAMPO_02`, `CAMPO_03`...). Lo que sigue al `9999` en los archivos firmados (la firma digital) no se lee |

La carpeta `layouts/` del [repositorio](https://github.com/andrelnfernandes/xylem/tree/main/layouts) lista lo que está previsto y cómo contribuir un layout, sin necesidad de Rust.

### SDF a partir de un .dbf

`COPY TO archivo SDF` en dBase y Clipper escribe los registros uno junto a otro, cada campo con su ancho. En `K`, un `.dbf` junto al archivo (o uno indicado con F2) da su layout: los campos en orden, las fechas como `YYYYMMDD`, los números con punto. Los campos memo no los exporta SDF y quedan fuera.

## Cómo escribir un layout

Un layout es un archivo TOML cuyo nombre termina en `.xlayout.toml`. Uno posicional:

```
schema = 1
name = "Clientes del sistema antiguo"
line_length = 40
[discriminator]
pos = [1, 1]
[[records]]
id = "0"
name = "Cabecera"
role = "header"
fields = [
  { name = "tipo", pos = [1, 1], type = "code" },
  { name = "creado", pos = [2, 9], type = "date", format = "YYYYMMDD" },
  { pos = [10, 40], type = "filler" },
]
[[records]]
id = "1"
name = "Cliente"
fields = [
  { name = "tipo", pos = [1, 1], type = "code" },
  { name = "codigo", pos = [2, 7], type = "code" },
  { name = "nombre", pos = [8, 30], type = "text" },
  { name = "saldo", pos = [31, 40], type = "decimal", decimals = 2, sign = "trailing" },
]
```

Las posiciones empiezan en 1 e incluyen ambos extremos: `pos = [8, 30]` va del 8.º al 30.º carácter. `start = 8, len = 23` dice lo mismo, y un `len` junto a `pos` debe coincidir con él, lo que detecta errores de tecleo. Con `strict_positions` (activado por defecto), los campos de un registro deben cubrir la línea entera, sin huecos ni solapamientos: dé a las partes vacías el tipo `filler`.

| Clave | Qué dice |
|---|---|
| `schema` | siempre `1` |
| `name` | el nombre que se muestra en `K` y en la cabecera |
| `format` | `"positional"` (por defecto) o `"delimited"` |
| `encoding` | `"auto"` (la que encontró el visor; `O` la cambia), `"utf-8"`, `"cp1252"`, `"cp850"` o `"cp437"` |
| `offset_unit` | posicional: `"chars"` (por defecto) o `"bytes"` |
| `line_length`, `strict_positions` | posicional: la longitud de una línea, y si los campos deben cubrirla |
| `hide_filler` | los campos `filler` quedan fuera de la cuadrícula (por defecto `true`); `i` siempre los lista |
| `delimiter`, `outer_delimiters` | delimitado: el separador, un carácter, y si las líneas también empiezan y terminan con él, como en SPED |
| `allow_undeclared_records` | delimitado: los registros que el layout no declara tienen hoja propia |
| `decimal_sep`, `date_format` | valores por defecto de todos los campos: `"implied"`, `","` o `"."`; y el formato de las fechas |
| `stop_after_trailer` | detenerse en el registro `trailer` (lo que sigue no se lee) |
| `[detect]` | cómo reconocer el archivo por su primera línea: `line_length`, `first_line` (condiciones) y `extensions` |
| `[discriminator]` | dónde está el código del registro: `pos` o, en delimitado, `col` |
| `description`, `country`, `category`, `spec_source`, `spec_version`, `spec_url`, `status` | sobre el layout y su manual; `status` es `"verified"` una vez comprobado con él |

Cada `[[records]]` tiene un `id` (su código para el discriminador; `ids` añade otros), un `name`, un `role` (`header`, `batch_header`, `detail`, `batch_trailer` o `trailer`) y sus `fields`. Cuando el código solo no distingue los registros, `match` añade condiciones sobre la línea, y `batch_match` condiciones sobre la línea de la cabecera de su lote: en CNAB 240, el segmento A de un pago y el de un débito solo difieren por la operación de la cabecera de lote. Una condición es un lugar (`pos` o `col`) y uno de `value = "3"`, `in = ["1", "2"]` o `regex = "^[0-9]+$"`. Entre registros con el mismo código, se prueba primero el que tiene más condiciones.

| Clave del campo | Qué dice |
|---|---|
| `name` | el nombre de la columna, también para el filtro SQL |
| `pos`, `start` y `len`, o `col` | dónde está |
| `type` | `text`, `code` (dígitos que no son una cantidad: documento, cuenta, código postal), `int`, `decimal`, `date`, `time` o `filler` |
| `decimals`, `decimal_sep` | decimales, y dónde está la coma: `"implied"` (el `V99` de COBOL), `","` o `"."` |
| `sign` | `"none"`, `"leading"`, `"trailing"` u `"overpunch"` (el signo zonado de COBOL: `{`, `A`-`I`, `}`, `J`-`R`) |
| `format` | fechas: `YYYYMMDD`, `DDMMYYYY`, `YYMMDD`, `DDMMYY` o `YYYYDDD`; horas: `HHMMSS` o `HHMM`. Los años de dos cifras de 00 a 69 son de los 2000 |
| `trim` | espacios que se quitan del texto: `"right"` (por defecto), `"left"`, `"both"` o `"none"` |
| `desc`, `values` | la descripción, y qué significa cada código: `values = { "0" = "Alta", "9" = "Baja" }` |
| `check` | con qué debe coincidir el valor, abajo |

| `check` | El valor debe ser |
|---|---|
| `total_lines` | las líneas del archivo hasta esta |
| `record_count:1*` | las líneas de los registros con este id; `*` toma todo id que empieza así, `+` añade otros |
| `sum:3A.valor` | la suma de un campo de un registro en todo el archivo; `+` añade otros |
| `batch_lines` | las líneas desde la cabecera del lote hasta esta |
| `batch_sum:3A.valor+3J.valor` | la suma en el lote |
| `count_by_record:REG_BLC` | las líneas cuyo código es el valor del campo indicado (el 9900 de SPED) |
| `block_lines` | las líneas cuyo código empieza como el de esta (el x990 de SPED) |

Los copybooks de COBOL (`copybook = ...`), EBCDIC y los números empaquetados (`COMP-3`) están previstos y todavía no se leen.

## Seguridad

Tanto los datos como un layout elegido fuera de su carpeta pueden ser hostiles. Un layout se lee con el mismo rigor que los datos: como máximo 1 MiB, se rechaza cualquier clave o tipo desconocido, se comprueban todas las posiciones y tamaños, los nombres no pueden tener caracteres de control ni de dirección del texto, y registros, campos, códigos, condiciones y expresiones regulares tienen límites. Las expresiones regulares se ejecutan en tiempo lineal. Un layout no puede leer otros archivos: `copybook` se rechaza por ahora.

Los datos se leen con los límites del modo tabla: 3 s (100 ms en Autoview), 50.000 filas por hoja, 256 columnas, y las líneas de más de 512 KiB no se dividen en campos. Esc detiene la lectura. Las celdas se vuelven seguras para el terminal como cualquier otra: los caracteres de control y de dirección del texto se muestran como `·`.

Vea también: [el visor](viewer.md).
