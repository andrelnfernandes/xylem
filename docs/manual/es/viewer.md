# El visor

> **Aún no cubre la 0.8.0.** Las tablas sin el límite de 50.000 filas están solo en la versión en inglés por ahora. Los textos grandes leídos a medida que la pantalla avanza ("Big files") también.

`V` abre el archivo en el visor (Autoview, F7, lo muestra junto a la lista). Los
archivos binarios se abren en hexadecimal. Los archivos que superan el límite de tamaño
muestran solo el principio.
Las hojas de cálculo, CSV, DBF, SQLite y Parquet se abren como tabla (véase Modo tabla).

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
coincidencia aparece resaltada. En la entrada de búsqueda, **F2** alterna la
distinción de mayúsculas y minúsculas (al ignorarlas, los acentos también se
equiparan: `AÇÃO` encuentra `ação`) y **F4** alterna entre texto y hexadecimal
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

El propio Markdown oculta algunas cosas: definiciones de enlace que nada usa y celdas de tabla más allá de las columnas de la cabecera. Para verlo todo, `T` muestra la fuente. La cursiva y el tachado dependen del terminal y de su fuente, por eso también tienen color propio.

## JSON, YAML y TOML

Un `.json` (también `.geojson`, `.har`, `.webmanifest`, y `.jsonc` y `.json5`, con comentarios y coma final), un `.yaml` o `.yml` y un `.toml` (también `Cargo.lock`) se abren como un árbol que se pliega, y el marco dice `TREE: JSON`. Un archivo minificado en una sola línea se lee tan fácil como cualquier otro. Un árbol de hasta 2.000 nodos se abre todo abierto; uno más grande muestra su primer nivel.

| Tecla | Acción |
|---|---|
| ↑ ↓ RePág AvPág Inicio Fin | mueve el cursor |
| → | abre el nodo (otra vez: entra en él) |
| ← | cierra el nodo, o va al de arriba |
| `Enter` | abre o cierra el nodo |
| `*` | abre todo lo que hay bajo el nodo |
| `k` | copia la ruta del nodo: `.servers[2].name` en JSON, `servers.2.name` en YAML y TOML |

El pie muestra la ruta del nodo bajo el cursor. La búsqueda (`F`, `B`, `/`) mira claves y valores, también dentro de nodos cerrados, y abre el camino hasta lo que encuentra. `T` muestra el texto en la línea del nodo, y vuelve; `H` pasa por árbol, texto y hexadecimal. `&`, `g`, `#` y Ctrl+B actúan sobre las líneas del texto.

- Un nodo cerrado dice lo que contiene: `{12 keys}`, `[340 items]`.
- Un número se muestra como lo escribe el archivo: `12345678901234567890` sigue así, y `1_000` en TOML también.
- Una cadena muestra el salto de línea como `↵`.
- Una clave que aparece dos veces en un mismo objeto se muestra dos veces, marcada `dup key`, y la cabecera las cuenta. Los programas no se ponen de acuerdo sobre cuál de las dos vale, lo que puede ocultar un valor en una revisión.
- Un ancla de YAML se muestra como `&a1` y un alias como `*a1`: el alias nunca se expande, así que un archivo hecho para multiplicarse (un "billion laughs") no cuesta más que su texto. Una etiqueta (`!Ref`) aparece antes del valor. Varios documentos quedan bajo una raíz, como `document 1`, `document 2`...

Un archivo no válido, o anidado a más de 512 niveles, se abre como texto, y el aviso dice por qué y en qué línea. El árbol se arma con el archivo entero: uno por encima del límite de lectura se abre como texto.

## Notebooks de Jupyter

Un `.ipynb` se abre renderizado, celda por celda, y el marco dice `RENDERED`. Arriba: el kernel y cuántas celdas hay de cada tipo.

- Una celda Markdown se renderiza como un archivo Markdown.
- Una celda de código muestra `In [n]:` y su código, con los colores del lenguaje del notebook.
- Las salidas vienen después, marcadas `Out [n]:`: el texto de un stream (`stderr:` cuando son los errores), el texto de un resultado (o su Markdown, renderizado) y un error con su traceback, sin los códigos ANSI del traceback.
- Una salida de imagen, HTML, JavaScript, SVG o LaTeX es una línea que dice qué es y cuánto ocupa, como `[image/png, 34 KiB: not shown]`. Nada de un notebook se ejecuta: ni su código, ni los scripts de sus salidas, ni sus widgets.
- Una celda raw se muestra tal cual.

`T` muestra el JSON del notebook, y `H` sus bytes. Un archivo que no es un notebook que el visor pueda leer (JSON roto, formato desconocido) se abre en su fuente, con un aviso. El formato 3 (con worksheets) también se lee.

## Documentos de Office

Los archivos de Word (`.docx`, `.docm`, `.dotx`), de texto OpenDocument (`.odt`), de PowerPoint (`.pptx`, `.pptm`) y de Impress (`.odp`) se abren como documento: títulos, párrafos, listas, tablas, negrita, cursiva y enlaces, con la dirección real de cada enlace. Una presentación muestra cada diapositiva bajo `Slide n`, con las notas del orador después. Las notas al pie y los comentarios, con sus autores, van al final. Una imagen es una línea, como `[image: media/image1.png]`. El texto de un documento nunca se lee como Markdown.

Antes del texto, un bloque dice lo que el documento trae además de él, ya que mucho de eso no se ve en el programa que lo hizo:

- quién lo escribió, quién lo guardó por última vez, cuándo y con qué;
- `Contains macros`, cuando las tiene. Nunca se leen ni se ejecutan;
- `Fetches from outside when it opens`: una plantilla remota, un objeto OLE o un frame que apuntan fuera del archivo, con sus direcciones. Una plantilla remota es como llegaron ataques como Follina (CVE-2022-30190). Aquí nunca se descarga nada;
- los objetos incrustados, por nombre y tamaño, y cuántos otros enlaces externos hay;
- los cambios controlados: el texto insertado se muestra como `[inserted: ...]` y el borrado como `[deleted: ...]`, porque lo borrado suele ser lo que importa;
- el texto oculto (`[hidden: ...]`), el texto blanco y el de un punto (`[invisible: ...]`), y las diapositivas ocultas.

`H` muestra los bytes, y vuelve; `T` dice que el documento no tiene texto propio. Cada parte del archivo se descomprime solo hasta un límite (64 MiB cada una, 256 MiB en total; 4 MiB en Autoview), y la cabecera dice cuándo el documento quedó cortado. Una parte XML con DTD, o anidada a más de 256 niveles, se rechaza. Los antiguos `.doc` y `.ppt` se abren en hexadecimal, como antes.

## Ejecutables

Un ejecutable, una biblioteca o un archivo objeto se abre como un informe, sea cual sea su nombre: ELF (Linux, BSD), PE (`.exe`, `.dll`, `.sys` de Windows, también .NET) y Mach-O (macOS, también binarios universales con varias arquitecturas). Se reconoce por sus primeros bytes, nunca por su nombre. El informe dice:

- qué es (ejecutable, biblioteca, objeto, core dump), para qué máquina, 32 o 64 bits, su punto de entrada, y su intérprete, subsistema o sistema mínimo;
- **protecciones**, cada una `yes` o `no`, con `⚠` donde falta: en ELF, PIE, una pila que no ejecuta código (NX), RELRO (completo o parcial), el canario de la pila, FORTIFY y las rutas de bibliotecas (RPATH, RUNPATH: una relativa o una en `/tmp` se marca); en PE, ASLR, ASLR de alta entropía, DEP, Control Flow Guard; en Mach-O, PIE y pila ejecutable;
- **firma**: si la hay y de qué tamaño. **No se verifica**, y el informe lo dice;
- **señales** de algo escondido: una sección de entropía alta (empaquetada o cifrada), el nombre de un empaquetador (UPX...), datos después de la última sección (un overlay, donde los instaladores y algunos malware guardan datos), una sección a la vez escribible y ejecutable, un punto de entrada fuera de toda sección ejecutable;
- **identidad**: build ID, debug link, UUID, ruta del PDB, el compilador, la versión de Go; la fecha de compilación de un PE, que escribe el enlazador y puede ser cualquiera;
- las bibliotecas que necesita, y las listas de secciones, segmentos, importaciones, exportaciones y símbolos, de hasta 2.000 filas cada una.

Nada se ejecuta ni se carga para hacer el informe: ni siquiera `ldd`, que en algunos casos ejecuta el programa. `H` muestra los bytes, y vuelve. Un archivo más grande que el límite de lectura se lee solo donde el informe lo necesita, 256 MiB como máximo en total; Autoview hace el informe del comienzo que leyó, y dice que puede dejar cosas fuera.

## Certificados y claves

Un certificado, una cadena o un conjunto de ellos, una solicitud de certificado (CSR), una lista de revocación (CRL), una clave privada o pública, una clave OpenSSH, `authorized_keys` y `known_hosts` se abren como un informe. El PEM se lee en un archivo con nombre para ello (`.pem`, `.crt`, `.cer`, `.key`, `.csr`, `.crl`, `.pub`...) o sin extensión (`id_ed25519`); el DER en un `.der`, `.cer`, `.crt`, `.crl` o `.csr` que sea un valor DER entero. Un bloque PEM dentro de código fuente o de un documento deja ese archivo como está.

- **Un certificado:** su subject y su issuer (y si es autofirmado), el número de serie, la validez con los días que quedan (vencido, todavía no válido, o que vence en 30 días o menos se marcan `⚠`), la clave pública (RSA por debajo de 2048 bits y DSA se marcan), la firma (MD5 y SHA-1 se marcan), los nombres (se señala un nombre en punycode `xn--`), el uso de la clave, el uso extendido, si es una CA, las direcciones de su CRL y de la información de la autoridad, una extensión crítica desconocida (marcada) y su huella SHA-256.
- **Varios certificados:** el informe dice si están en orden, cada uno emitido por el siguiente, comprobado solo por sus nombres e identificadores de clave. **Aquí no se verifica ninguna firma**, y el informe lo dice.
- **Una clave privada:** solo su tipo y su tamaño (RSA 2048 bits, EC P-256, PKCS#8 cifrada con PBES2...) y, en OpenSSH, si está cifrada y su huella. Sus números nunca aparecen en el informe. En Autoview, una clave privada muestra este informe, nunca su texto, para que no aparezca en la pantalla solo por pasar sobre ella; `T` en el visor muestra el texto.
- **Claves públicas, `authorized_keys` y `known_hosts`:** una fila por clave, con su tipo y tamaño, su huella SHA-256 tal como la escribe `ssh-keygen -l`, y su comentario. Una opción `command=` se marca, porque cambia lo que la clave permite; un host con hash se muestra como tal.

Nada se descarga: las direcciones de una CRL o de un OCSP solo se muestran. Antes de que cualquier parser vea un bloque, se comprueban su anidamiento (64 niveles como máximo) y sus longitudes; un archivo tiene como máximo 1.000 bloques de 64 KiB cada uno. No se leen PKCS#12 (`.p12`, `.pfx`) ni claves PGP.

## Logs

Un log se abre como texto, con los colores de sus niveles, y el pie dice `Log`. Un archivo es un log por su nombre (`.log`, `.out`, `.err`, uno rotado como `app.log.1` o `app.log.2.gz`, `syslog`, `messages`, `kern.log`) o por su contenido: la mayoría de sus primeras líneas empieza con fecha y hora (`2026-11-16 10:00`, `[2026-11-16 ...`, o el `Nov 16 10:00:00` de syslog).

- **Niveles:** la primera palabra de nivel en las primeras 120 columnas de una línea, en mayúsculas o no, le da su color: errores (`ERROR`, `ERR`, `FATAL`, `CRIT`, `PANIC`...) en rojo, avisos (`WARN`, `WARNING`) en amarillo, `DEBUG` y `TRACE` atenuados. También cuenta el `<n>` de syslog al comienzo de una línea, y el nivel de un log en JSON (`"level":"error"`). Una línea sin nivel, como un stack trace, toma el de la anterior. `C` apaga los colores.
- **Colores ANSI:** los colores que un programa escribió en su log (`ESC [31m`) se muestran como colores, y sus secuencias salen del texto. Cualquier otra secuencia se descarta, porque podría mover el cursor o escribir en el portapapeles (OSC 52), y la cabecera las cuenta, como en `[ANSI: 3 removed]`. `H` sigue mostrando los bytes tal cual, y la búsqueda mira el texto sin las secuencias. En un archivo de más de 16 MiB, leído a través de una ventana, las secuencias se muestran como `·`, como en cualquier texto, y los colores son los de los niveles.
- **Seguir:** con la última línea en pantalla (Fin), las líneas nuevas entran a medida que el programa las escribe, y la pantalla se queda al final; la cabecera dice `[FOLLOW]`. El archivo se mira cada medio segundo, también en un recurso compartido de red. Subir, o buscar, deja de seguirlo; Fin lo retoma. Con un filtro (`&`), las líneas nuevas pasan por él. Si el archivo se achicó o cambió en lugar de crecer (rotado o reescrito), la cabecera dice `(the file changed: R reloads)`, y no se lee nada hasta R. Un log comprimido no se sigue.

## Parches

Un `.patch` o `.diff`, o cualquier texto que empiece como uno (`diff --git`, o `---`, `+++` y `@@`), se abre como texto con los colores de un parche. El Git diff (`D`) se muestra igual. La cabecera lo resume, como en `[12 files, +340 −98]`, y el pie dice dónde está la parte de arriba de la pantalla: `3/12 src/viewer.rs @@ 2/5`.

| Tecla | Acción |
|---|---|
| `]` `[` | archivo siguiente o anterior |
| `}` `{` | hunk siguiente o anterior |
| `Tab` | la lista de los archivos, con `+` y `−` de cada uno; las letras la filtran, `Enter` va a uno |

Algunos cambios hacen más que cambiar líneas, y es fácil pasarlos por alto en una lectura rápida. Se muestran en rojo, y la cabecera los cuenta como `to check`:

- un archivo que pasa a ser ejecutable (`100755`), un enlace simbólico (`120000`) o un submódulo (`160000`);
- una ruta absoluta, que sube con `..`, que entra en `.git` (un hook es código que Git ejecuta), o con caracteres de control.

La cabecera también cuenta:

- `look unchanged`: una línea quitada y una añadida que solo difieren en lo que no se ve, como espacios, `\r`, caracteres invisibles o letras parecidas (una `а` cirílica en lugar de una `a` latina). El par se muestra en amarillo;
- `hunks with wrong counts`: un hunk cuya línea `@@` no coincide con sus líneas. El índice sigue las líneas que hay, nunca los números;
- `whitespace`: líneas añadidas con un espacio al final, o con un espacio antes de un tabulador en la sangría, como lo señala `git diff --check`.

Una serie de `git format-patch` muestra cada mensaje en la lista, por su asunto. Nunca se aplica nada. En un archivo de más de 16 MiB, leído a través de una ventana, los colores se mantienen, pero no hay índice: `]`, `}` y Tab lo dicen.

## Caracteres de control de dirección

Algunos caracteres invisibles cambian la dirección del texto y pueden hacer que un código parezca distinto de lo que realmente dice (Trojan Source, CVE-2021-42574). El visor nunca los oculta: cada uno aparece como un `·` rojo, y el encabezado indica cuántos hay.

## Codificación del texto

Xylem descubre la codificación solo, en este orden: primero descomprime el archivo, luego busca un BOM (UTF-8, UTF-16 LE o BE), luego UTF-16 sin BOM, luego UTF-8. Un archivo que no es nada de eso es texto antiguo: Xylem elige CP850 (DOS, Europa occidental) o Windows-1252 contando las letras con tilde. Los archivos `.reg` de Windows son UTF-16 y se abren como texto normal.

**O** muestra el archivo en la siguiente codificación: UTF-8, CP850, Windows-1252, CP437, UTF-16 LE, UTF-16 BE y de nuevo UTF-8. El pie dice cuál, y la cabecera muestra su nombre cuando no es UTF-8. Si elige UTF-16 en un archivo que se abrió en hexadecimal, pasa a texto. Su elección se mantiene después de editar el archivo y volver.

Para usar siempre una sola codificación en los textos antiguos, en lugar de la suposición, ponga `legacy_charset` en `cp850`, `cp1252` o `cp437` (**K** en la página 2 de la [configuración](configuration.md)). El valor por defecto es `auto`.

## Límite de tamaño

El visor lee solo el principio de un archivo grande, y el encabezado dice `(showing first N MiB)`. Cuánto lee lo decide la opción `viewer_max_size` (**L** en la página 2 de la [configuración](configuration.md)): `auto`, `16`, `64`, `256` o `1024` (MiB). Con `auto`, el valor por defecto, el límite es 1/32 de la RAM de la máquina, nunca menos de 16 MiB ni más de 256 MiB (son 16 MiB si no se puede leer la RAM). La pantalla de configuración muestra el valor en uso, por ejemplo `auto (256 MiB)`.

Dos casos no siguen esta regla. Autoview siempre lee 1 MiB. Un archivo en una máquina remota (SFTP) se lee como máximo hasta 16 MiB, porque Esc no puede cancelar esa lectura.

Un límite mayor cuesta memoria y tiempo: con `1024`, un archivo de texto normal de ese tamaño ocupa unos 2,5 GiB de RAM y tarda unos 5 s en abrirse. Las líneas muy cortas cuestan más que sus bytes, porque cada línea tiene su propia gestión, y lo mismo pasa con los tabuladores y los caracteres de control de dirección ocultos: el visor guarda líneas hasta cerca del doble del límite de tamaño en memoria y, pasado eso, el encabezado dice `(showing first N lines)`. Un log de líneas normales (unos 100 bytes) nunca se corta por esto.

## Archivos comprimidos

Los archivos que terminan en `.gz`, `.xz`, `.zst` o `.bz2` se abren descomprimidos, como texto. Xylem mira los primeros bytes del archivo, no el nombre, y abre solo un nivel: un `.gz` dentro de otro `.gz` sigue comprimido. El encabezado muestra `gzip → N bytes` (el formato y el tamaño tras descomprimir).

La descompresión tiene límites. El visor lee hasta el límite de tamaño, tarda como máximo 3 s y usa como máximo 64 MiB de memoria para el diccionario de xz o la ventana de zstd. En Autoview los límites son 1 MiB, 100 ms y 16 MiB. Cuando se alcanza el límite de salida o el de tiempo, el encabezado dice `(showing first N bytes decompressed)`. Una nota explica lo que pasó:

- `decompression stopped after 3 s` (100 ms en Autoview)
- `compressed data truncated or corrupt` (datos comprimidos truncados o dañados)
- `needs more than 64 MiB of memory to decompress` (16 MiB en Autoview, con ` (V to view)` cuando los 64 MiB de V pueden bastar)

La nota desaparece con la primera tecla, pero el encabezado conserva el problema: `(truncated or corrupt)`, `(incomplete: …)` cuando el límite de memoria detuvo la descompresión a medias, o `(not decompressed: …)` cuando no se pudo descomprimir nada (entonces los bytes sin procesar se muestran en hexadecimal).

Autoview muestra la nota en su título. En modo hexadecimal (`H`) se ven los bytes descomprimidos. Los colores siguen el nombre sin la extensión de compresión: `app.rs.gz` se colorea como Rust.

Para un `.tar.gz` (o `.tar.xz`...), la nota dice `TAR archive: Alt+F5 opens it as a folder` (Alt+F5 lo abre como una carpeta).

Para ver los bytes comprimidos, desactive `viewer_decompress` (**J** en la página 2 de la [configuración](configuration.md)).

## Modo tabla

Algunos archivos se abren como una cuadrícula en vez de texto. Xylem decide por la extensión, y solo para archivos sin comprimir en este disco: un archivo de una máquina remota o dentro de un paquete se muestra como contenido normal, con la nota `Tables are shown only for files on this disk`.

| Formato | Extensiones | Nombres de las columnas |
|---|---|---|
| CSV, TSV | `.csv` `.tsv` `.tab` | la primera fila, cuando al menos la mitad no son números; si no, A, B, C... |
| JSON Lines | `.jsonl` `.ndjson` | las claves de los objetos |
| Hojas de cálculo | `.xlsx` `.xls` `.xlsb` `.ods` | A, B, C... (la primera fila es una fila más) |
| dBase, Clipper, FoxPro | `.dbf`, con su archivo memo `.dbt` | los nombres de los campos |
| SQLite | `.db` `.sqlite` `.sqlite3` | las columnas de la tabla |
| Parquet | `.parquet` | las columnas del archivo |

El separador del CSV (coma, punto y coma, tabulación o barra vertical) se detecta por el principio del archivo.

| Tecla | Qué hace |
|---|---|
| ↑ ↓ | fila anterior o siguiente |
| ← → | columna anterior o siguiente |
| PgUp PgDn | una página de filas hacia arriba o hacia abajo |
| Home End | primera o última columna |
| Ctrl+Home Ctrl+End | primera o última fila; con un filtro, de las filas que conserva |
| `[` `]` | hoja anterior o siguiente de una hoja de cálculo, o tabla de una base SQLite, dando la vuelta; esto borra el filtro |
| `Tab` | la lista de hojas o tablas: escribe parte de un nombre para reducirla, Enter abre una y Esc vuelve; abrir otra borra el filtro. En una base, las vistas van después de las tablas, marcadas con `(view)` |
| `g` o `:` | saltar a una fila por su número en el archivo; si el filtro la ocultó, a la siguiente que dejó |
| `\` o `&` | el filtro SQL |
| `m` o Insert | marca o desmarca el registro bajo el cursor y pasa al siguiente; **Ctrl+B** escribe entonces solo los marcados |
| `M` | borra las marcas |
| `i` | la estructura de la tabla: lo que el archivo dice de cada columna (ver abajo) |
| Enter | el registro bajo el cursor como una ficha, un campo por línea: ↑ ↓ eligen el campo, ← → van al registro anterior o siguiente, `c` copia el registro entero (`NOMBRE : valor`, una línea por campo) e `y` el valor del campo, para pegarlo en un editor o un terminal; Esc vuelve. Lo copiado es lo que muestra la pantalla: los caracteres de control y de dirección del texto, saltos de línea incluidos, son `·`, así que pegarlo en un terminal no ejecuta nada. Fuera de la tabla, Enter cierra el visor |
| `T` | ir al texto del archivo y volver |
| `H` | de la tabla al texto, luego hexadecimal, luego de vuelta a la tabla |

El marco muestra `TABLE [2/5: nombre]` cuando hay más de una hoja o tabla, y la línea inferior dónde está el cursor: `Cell B7 (Row 7 of 120)`. Con filtro, las filas conservan el número que tienen en el archivo, y esta línea añade qué coincidencia es: `Cell A13 (Row 13 of 31, match 2 of 8)`. La última línea de la cuadrícula trae el contenido completo de la celda y el filtro en uso. La búsqueda (`F`, `B`, `/`) no mira dentro de la cuadrícula y no encuentra nada ahí; funciona en modo texto.

Las marcas se mantienen bajo un filtro, y Batch escribe lo marcado que el filtro conserva. Pertenecen a la hoja o tabla abierta y se pierden al cambiarla. Si el archivo cambia después de marcar, Batch se niega, pues puede haber otros registros en esas posiciones.

### La estructura

`i` muestra, sobre la cuadrícula, las columnas en orden con lo que el archivo dice de cada una, como `DISPLAY STRUCTURE` en dBase y Clipper. Las flechas, PgUp, PgDn, Home y End la desplazan; Esc, Enter o `i` la cierran.

| Formato | Qué muestra |
|---|---|
| dBase, Clipper, FoxPro | nombre, tipo (`C` carácter, `N` numérico, `D` fecha, `L` lógico, `M` memo, `F` coma flotante, `I` entero, `B` double, `Y` moneda, `T` fecha y hora), ancho y decimales; encima, el número de registros y la fecha de la última actualización escritos en la cabecera, y la longitud de un registro |
| SQLite | el tipo declarado de cada columna, `Not null`, el valor por defecto y la posición en la clave primaria; encima, si es una tabla o una vista, y sus índices con sus columnas (como máximo 32). Una vista muestra las columnas que da; una consulta completa, las de su resultado, con el tipo de cada una cuando viene directamente de una columna de tabla |
| Parquet | el tipo Arrow de cada columna y si admite nulos; encima, las filas y los grupos de filas que declara el archivo y el programa que lo escribió |
| CSV, JSON Lines, hojas de cálculo | no declaran tipos: lo que tienen las filas leídas en cada columna (entero, decimal, lógico, texto, vacío) y el valor más ancho |

Nombres, tipos y valores por defecto vienen del archivo y se limpian como las celdas: los caracteres de control y de dirección del texto se ven como `·`. Un `.dbf` cuya cabecera se contradice muestra `?` en los decimales.

### El filtro SQL

`\` o `&` pide una condición, como el `WHERE` de SQL (la palabra `WHERE` es opcional), y muestra solo las filas que la cumplen. El prompt empieza con el filtro actual; déjelo vacío para volver a ver todas las filas. Una nota dice `Filter matched N rows` o `SQL error: ...`; tras un error, el filtro anterior sigue activo.

```
age >= 30 AND name LIKE 'Ali%'
WHERE status IN ('open', 'late') OR total > 1000.50
NOT (city = 'Rio' OR city IS NULL)
B ILIKE '%silva%'
```

| Operador | Qué hace |
|---|---|
| `=` `!=` `<>` `<` `<=` `>` `>=` | comparan |
| `AND` `OR` `NOT` y paréntesis | combinan condiciones (`AND` va antes que `OR`) |
| `LIKE` `ILIKE` | coinciden con un patrón, `%` para cualquier texto y `_` para un carácter; `ILIKE` no distingue mayúsculas |
| `IN (...)` `NOT IN (...)` | está, o no está, en la lista |
| `IS NULL` `IS NOT NULL` | la celda está, o no está, vacía |

Las palabras clave pueden ir en mayúsculas o minúsculas.

- **Columnas:** el nombre de la columna, en mayúsculas o minúsculas y sin comillas, o su letra, de `A` a `Z`. La letra sirve para llegar a un nombre con espacios o símbolos. Una columna que no existe cuenta como vacía, sin error.
- **Valores:** texto entre comillas simples o dobles (`\'` para una comilla dentro de él), números, `TRUE`, `FALSE` y `NULL`. Lo que va entre comillas es siempre texto, nunca una columna.
- **Celdas:** el tipo viene del aspecto de la celda. Vacía es `NULL`, `true` y `false` son booleanos, los números son números y el resto es texto. Por eso escriba `code = 7`, no `code = '7'`: un texto y un número nunca son iguales, y una celda con `007` es el número 7. Las comparaciones de texto distinguen mayúsculas de minúsculas.
- **Tamaño:** un filtro tiene como máximo 1024 bytes y 128 términos.

### Una consulta completa en una base

En una base SQLite, el mismo prompt acepta una consulta completa, que empiece por `SELECT` o `WITH`, con todo lo que SQLite puede hacer: `JOIN`, `GROUP BY`, `ORDER BY`, funciones.

```
SELECT c.name, sum(o.price) FROM customers c JOIN orders o ON o.code = c.code GROUP BY c.name
```

El marco dice `QUERY:`, y la cuadrícula muestra el resultado. Una condición filtra entonces esas filas, y **Ctrl+B** escribe todo lo que da la consulta. Dejar el prompt vacío borra el filtro y luego vuelve a la tabla; `Tab`, `[` y `]` también van a una tabla. La consulta solo lee, una cada vez: cualquier otra cosa se rechaza, y un error deja la pantalla como estaba. Los límites de abajo se aplican, así que una consulta lenta se detiene a los 3 s y muestra las filas obtenidas; un Batch no tiene ese límite. Xylem no modifica bases: `UPDATE`, `DELETE` y similares reciben un mensaje que lo dice. Para modificar una, use otro programa: `O` en la ventana de archivos la abre en el definido en `opener`, y `'*.db' = 'sqlite3 %f'` en `[editors]` hace que `E` la abra en `sqlite3`.

### Límites

Una tabla guarda como máximo 50 000 filas y 256 columnas, y tarda como máximo lo que permiten los demás límites del visor: 3 s (100 ms en Autoview), y una hoja de cálculo que se desempaqueta a más del límite de tamaño (1 MiB en Autoview) no se abre como tabla. Lo que pasa de ahí no aparece y el filtro no lo ve; la cabecera lo avisa: `(showing first 50,000 rows)` o `(stopped by the time limit: showing N rows)`. Mientras se lee la tabla, la pantalla queda libre y una nota lo avisa; Esc detiene la lectura y muestra lo que ya llegó, `(stopped by Esc: showing N rows)`, y el siguiente Esc cierra el visor. Una celda muestra como máximo 2048 caracteres y luego `…`. Los caracteres de control y de dirección del texto aparecen como `·`.

Cuando una hoja de cálculo (XLSX, XLSB, XLS, ODS) tiene más filas que eso, la cabecera y la nota lo dicen: `(the sheet has more than 50,000 rows, the most Xylem shows: open it in Excel or another spreadsheet program to see them all (O))`, es decir, ábrela en Excel o en otro programa de hojas de cálculo para verlas todas. **O** la abre en el programa que tu sistema usa para ella.

## Tablas en bibliotecas C

Las bases SQLite (`.db`, `.sqlite`, `.sqlite3`) y los archivos Parquet se leen con bibliotecas escritas en C, que no tienen la seguridad de memoria de Rust. Por eso solo se abren como tabla con `V`. Autoview lee todo aquello por donde pasa el cursor y muestra estos archivos como contenido normal, con la nota `Parquet or SQLite: V opens the table`. Abre una base de origen no confiable solo si aceptas este riesgo. Los archivos `.ods` y `.xls` también se abren como tabla solo con `V`: su lector monta primero la hoja entera, y esa parte no se puede detener.

Véase también: [buscar en los archivos marcados](search.md).
