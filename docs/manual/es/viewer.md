# El visor

`V` abre el archivo en el visor (Autoview, F7, lo muestra junto a la lista). Los
archivos binarios se abren en hexadecimal. Un archivo grande en disco no se lee entero: el visor lee lo que muestra la pantalla a medida que avanza (consulta [Archivos grandes](#archivos-grandes)). Lo que no se puede leer así muestra solo el principio, hasta el límite de tamaño.
Las hojas de cálculo, CSV, DBF, SQLite y Parquet se abren como tabla (consulta [Tablas](tables.md)).

| Tecla | Qué hace |
|---|---|
| ↑ ↓ PgUp PgDn Home End | desplazar |
| ← → | desplazar lateralmente |
| `H` | modo siguiente: texto y hexadecimal; en un archivo Markdown, renderizado, texto y hexadecimal |
| `O` | siguiente codificación (UTF-8, CP850, 1252, CP437, UTF-16) |
| `C` | activar o desactivar colores de sintaxis |
| `W` | activar o desactivar el ajuste de línea |
| `#` o `L` | activar o desactivar números de línea (en modo texto) |
| `g` o `:` | saltar a una línea (un desplazamiento de byte en hexadecimal, una fila en modo tabla) |
| `&` | muestra solo las líneas que coinciden, texto simple o regex ([filtrar líneas](filter.md)); en modo tabla, el filtro SQL |
| Ctrl+B | una línea de salida por registro o línea mostrada ([Batch en el visor](execute.md)) |
| `E` | editar el archivo; al volver a Xylem, el visor muestra el contenido nuevo en el mismo lugar (archivos binarios solo con un editor en [`[editors]`](execute.md)) |
| `D` | con Git activado: qué cambió desde el último commit, y volver ([Git](git.md)) |
| `T` | tabla o texto, en un archivo que se abrió como tabla; renderizado o fuente, en un archivo Markdown |
| `U` | en la vista de los archivos marcados (Ctrl+V): desmarcar el archivo |
| `y` o Ctrl+Insert | copiar la ruta del archivo al portapapeles |
| Esc | cerrar |

## Búsqueda

Como en ZTreeWin: **F** busca desde el principio, **B** hacia atrás desde el
final, **/** (o `S`) desde la página actual y **\\** hacia atrás desde ella;
**Space** continúa en la misma dirección, **+** y **-** eligen la dirección. La
coincidencia aparece resaltada. En el prompt de búsqueda, **F2** alterna la
distinción de mayúsculas y minúsculas (al ignorarlas, también vale para las
letras con acento: `AÇÃO` encuentra `ação`) y **F4** alterna entre texto y hexadecimal
(pares de bytes como `4d 5a`). Un `*` en mitad del término coincide con
cualquier cosa en la misma línea (`error*disk`).

## Colores de sintaxis

El código fuente y los scripts se colorean en unos 150 lenguajes (Rust, Python,
shell, C, JavaScript, SQL, YAML, Markdown...), reconocidos por extensión, por
nombre (`Makefile`) o por la primera línea (`#!/bin/bash`). El valor
predeterminado es `syntax_highlight` (página 2 de la
[configuración](configuration.md)). Los colores se mantienen legibles sobre el
fondo del tema actual, y solo se colorean las primeras 20.000 líneas, y ninguna
línea de más de 10.000 caracteres, para que los archivos enormes nunca
ralenticen el visor.

## Markdown renderizado

Los archivos `.md` y `.markdown` se abren renderizados, también en Autoview, dentro de archivadores y comprimidos (`x.md.gz`), y el marco dice `RENDERED`. `T` muestra la fuente en el mismo lugar, y vuelve; `H` pasa por renderizado, texto y hexadecimal.

Se ven títulos, negrita, cursiva, tachado, código, bloques de código con colores de sintaxis (`C` los apaga), listas anidadas y numeradas, listas de tareas (`[x]`), citas, líneas horizontales y tablas. Nunca se descarga nada, y ningún enlace abre nada:

- Un enlace muestra su dirección real junto al texto, como en `seguro <https://example.com>`, para que el texto no pase por otra dirección.
- Una imagen es una línea de texto: `[image] leyenda <foto.png>`.
- Un diagrama Mermaid es un bloque de código con la etiqueta `mermaid`.
- El HTML y los comentarios HTML siguen visibles como texto.
- El bloque YAML al inicio de un archivo es un bloque de código con la etiqueta `yaml`.

El texto siempre cabe en la ventana, así que `W` no hace nada aquí; ← → alcanzan las columnas de una tabla demasiado ancha. La búsqueda mira el texto de la pantalla, direcciones incluidas, y encuentra una frase que sigue en la fila siguiente. `g` y `#` cuentan las líneas de la fuente. Ctrl+B actúa sobre las líneas de la fuente, todas. `&` pasa al modo texto y filtra allí.

Un archivo de más de 8 MiB se abre como fuente, con un aviso. El renderizado también se detiene en el límite de tiempo (3 s, 100 ms en Autoview) o en su límite de memoria, y la cabecera dice entonces `(rendered only the start)`. Una tabla muestra como máximo 256 columnas. Los caracteres de control aparecen como `·`, incluidos los escritos como entidades, como `&#27;`.

El propio Markdown oculta algunas cosas: definiciones de enlace que nada usa y celdas de tabla más allá de las columnas de la cabecera. Para verlo todo, `T` muestra la fuente. La cursiva y el tachado dependen de la terminal y de su fuente, por eso también tienen color propio.

## Caracteres de control de dirección

Algunos caracteres invisibles cambian la dirección del texto y pueden hacer que un código parezca distinto de lo que realmente dice (Trojan Source, CVE-2021-42574). El visor nunca los oculta: cada uno aparece como un `·` rojo, y el encabezado indica cuántos hay.

## Codificación del texto

Xylem descubre la codificación solo, en este orden: primero descomprime el archivo, luego busca un BOM (UTF-8, UTF-16 LE o BE), luego UTF-16 sin BOM, luego UTF-8. Un archivo que no es nada de eso es texto antiguo: Xylem elige CP850 (DOS, Europa occidental) o Windows-1252 contando las letras con tilde. Los archivos `.reg` de Windows son UTF-16 y se abren como texto normal.

**O** muestra el archivo en la siguiente codificación: UTF-8, CP850, Windows-1252, CP437, UTF-16 LE, UTF-16 BE y de nuevo UTF-8. El pie dice cuál, y la cabecera muestra su nombre cuando no es UTF-8. Si eliges UTF-16 en un archivo que se abrió en hexadecimal, pasa a texto. Tu elección se mantiene después de editar el archivo y volver.

Para usar siempre una sola codificación en los textos antiguos, en lugar de la suposición, pon `legacy_charset` en `cp850`, `cp1252` o `cp437` (**K** en la página 2 de la [configuración](configuration.md)). El valor por defecto es `auto`.

## Archivos grandes

Con **V**, un archivo en disco de más de 16 MiB no se carga: el visor guarda las líneas cercanas a la pantalla, unas 2000, y lee las demás del archivo a medida que avanza, sin bloquear la pantalla. Una línea aún no leída aparece como `…` por un momento. Así el archivo no tiene límite de tamaño, y la memoria usada no crece con él.

- **Las líneas se cuentan en segundo plano.** El pie muestra `Line 1 of 1,234,567+ (counting…)` y luego el total. **End** y **g** van a cualquier línea ya contada. Esc detiene el recuento, y lo contado se conserva.
- **La búsqueda** (`F`, `B`, `/`, `\`, Space) recorre el archivo en segundo plano: el pie muestra `Searching... line 1,200,000 (Esc stops)`, y la pantalla va a la coincidencia cuando se encuentra.
- **El filtro de líneas** (`&`) recorre todo el archivo en segundo plano, y las líneas aparecen a medida que se encuentran, hasta 1 000 000. Esc lo detiene y conserva lo encontrado.
- **El hexadecimal** (`H`) lee los bytes donde está la pantalla, y su búsqueda también recorre el archivo.
- **Otra codificación** (`O`) vuelve a leer las líneas; con UTF-16 se cuentan de nuevo, porque una línea termina en otro lugar.
- **Sin colores de sintaxis ni Markdown renderizado**: ambos leen el texto desde el principio.
- **Una línea de más de 1 MiB** se corta en pantalla, terminando en `…`; el hexadecimal la muestra entera.
- **Si el archivo cambia** mientras está abierto, el encabezado dice `(the file changed: R reloads)`, y **R** lo vuelve a leer y regresa a la línea en la que estaba.
- **Un Batch** (`Ctrl+B`) sobre el texto sigue leyendo el archivo en memoria, hasta el límite de tamaño de abajo, y avisa cuando eso no es todo el archivo.

Para llegar a una línea, el visor anota dónde empieza cada línea 256.ª, unos 8 bytes por cada 256 líneas, solo en memoria. Los controles de dirección del texto se cuentan en todo el archivo junto con las líneas (en UTF-8), y el encabezado muestra cuántos se han encontrado hasta ahora.

Un archivo comprimido, una entrada de un archivo comprimido, un archivo en una máquina remota y Autoview no se leen así: no se puede saltar dentro de ellos, así que siguen el límite de tamaño de abajo.

## Límite de tamaño

En esos casos, y en los archivos de hasta 16 MiB, el visor lee solo el principio de un archivo demasiado grande, y el encabezado dice `(showing first N MiB)`. Cuánto lee lo decide la opción `viewer_max_size` (**L** en la página 2 de la [configuración](configuration.md)): `auto`, `16`, `64`, `256` o `1024` (MiB). Con `auto`, el valor por defecto, el límite es 1/32 de la RAM de la máquina, nunca menos de 16 MiB ni más de 256 MiB (son 16 MiB si no se puede leer la RAM). La pantalla de configuración muestra el valor en uso, por ejemplo `auto (256 MiB)`.

Dos casos no siguen esta regla. Autoview siempre lee 1 MiB. Un archivo en una máquina remota (SFTP) se lee como máximo hasta 16 MiB, porque Esc no puede cancelar esa lectura.

Un límite mayor cuesta memoria y tiempo: con `1024`, un texto comprimido que se descomprime hasta ese tamaño ocupa unos 2 GiB de RAM. Las líneas muy cortas cuestan más que sus bytes, porque cada línea tiene su propia gestión, y lo mismo pasa con los tabuladores y los caracteres de control de dirección ocultos: el visor guarda líneas hasta cerca del doble del límite de tamaño en memoria y, pasado eso, el encabezado dice `(showing first N lines)`. Un log de líneas normales (unos 100 bytes) nunca se corta por esto.

## Archivos comprimidos

Los archivos que terminan en `.gz`, `.xz`, `.zst` o `.bz2` se abren descomprimidos, como texto. Xylem mira los primeros bytes del archivo, no el nombre, y abre solo un nivel: un `.gz` dentro de otro `.gz` sigue comprimido. El encabezado muestra `gzip → N bytes` (el formato y el tamaño tras descomprimir).

La descompresión tiene límites. El visor lee hasta el límite de tamaño, tarda como máximo 3 s y usa como máximo 64 MiB de memoria para el diccionario de xz o la ventana de zstd. En Autoview los límites son 1 MiB, 100 ms y 16 MiB. Cuando se alcanza el límite de salida o el de tiempo, el encabezado dice `(showing first N bytes decompressed)`. Una nota explica lo que pasó:

- `decompression stopped after 3 s` (100 ms en Autoview)
- `compressed data truncated or corrupt` (datos comprimidos truncados o dañados)
- `needs more than 64 MiB of memory to decompress` (16 MiB en Autoview, con ` (V to view)` cuando los 64 MiB de V pueden bastar)

La nota desaparece con la primera tecla, pero el encabezado conserva el problema: `(truncated or corrupt)`, `(incomplete: …)` cuando el límite de memoria detuvo la descompresión a medias, o `(not decompressed: …)` cuando no se pudo descomprimir nada (entonces los bytes sin procesar se muestran en hexadecimal).

Autoview muestra la nota en su título. En modo hexadecimal (`H`) se ven los bytes descomprimidos. Los colores siguen el nombre sin la extensión de compresión: `app.rs.gz` se colorea como Rust.

Para un `.tar.gz` (o `.tar.xz`...), la nota dice `TAR archive: Alt+F5 opens it as a folder` (Alt+F5 lo abre como una carpeta).

Para ver los bytes comprimidos, desactiva `viewer_decompress` (**J** en la página 2 de la [configuración](configuration.md)).

## Modo tabla

Las hojas de cálculo, CSV, DBF, SQLite y Parquet se abren como una cuadrícula, con filtro SQL: consulta [Tablas](tables.md).

Véase también: [buscar en los archivos marcados](search.md).
