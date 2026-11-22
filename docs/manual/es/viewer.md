# El visor

`V` abre el archivo en el visor (Autoview, F7, lo muestra junto a la lista). Los
archivos binarios se abren en hexadecimal. Un archivo grande en disco no se lee entero: el visor lee lo que muestra la pantalla a medida que avanza (consulta [Archivos grandes](#archivos-grandes)). Lo que no se puede leer así muestra solo el principio, hasta el límite de tamaño.
Las hojas de cálculo, CSV, DBF, SQLite y Parquet se abren como tabla (consulta [Tablas](tables.md)).

| Tecla | Qué hace |
|---|---|
| ↑ ↓ PgUp PgDn Home End | desplazar |
| ← → | desplazar lateralmente |
| `H` | modo siguiente: texto y hexadecimal; en un archivo Markdown, renderizado, texto y hexadecimal |
| `O` | siguiente codificación (UTF-8, CP850, 1252, CP437, 1251, KOI8-R, UTF-16) |
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

**F** busca desde el principio, **B** hacia atrás desde el
final, **/** (o `S`) desde la página actual y **\\** hacia atrás desde ella;
**Space** continúa en la misma dirección, **+** y **-** eligen la dirección. La
coincidencia aparece resaltada. En el prompt de búsqueda, **F2** alterna la
distinción de mayúsculas y minúsculas (al ignorarlas, también vale para las
letras con acento: `AÇÃO` encuentra `ação`) y **F4** alterna entre texto y hexadecimal
(pares de bytes como `4d 5a`). Un `*` en mitad del término coincide con
cualquier cosa en la misma línea (`error*disk`).

**F3** cambia la dirección de la búsqueda, así que F solo hace lo que hace B: hacia delante desde el principio o hacia atrás desde el final.

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

## JSON, YAML y TOML

Un `.json` (también `.geojson`, `.har`, `.webmanifest`, y `.jsonc` y `.json5`, con comentarios y coma final), un `.yaml` o `.yml` y un `.toml` (también `Cargo.lock`) se abren como un árbol que se pliega, y el marco dice `TREE: JSON`. Un archivo minificado en una sola línea se lee tan fácil como cualquier otro. Un árbol de hasta 2.000 nodos se abre todo abierto; uno más grande muestra su primer nivel.

| Tecla | Acción |
|---|---|
| ↑ ↓ RePág AvPág Inicio Fin | mueve el cursor |
| → | abre el nodo (otra vez: entra en él) |
| ← | cierra el nodo, o va al de arriba |
| `Enter` | abre o cierra el nodo |
| `*` | abre todo lo que hay bajo el nodo |
| `k` | copia la ruta del nodo: `.servers[2].name` en JSON, `servers.2.name` en YAML y TOML. Una clave con un fin de línea, un carácter de control o de dirección del texto va entre comillas, con esos caracteres escapados (`\n`, `\u202e`), para que pegar la ruta en una shell nunca ejecute una línea de ella |

El pie muestra la ruta del nodo bajo el cursor. La búsqueda (`F`, `B`, `/`) mira claves y valores, también dentro de nodos cerrados, y abre el camino hasta lo que encuentra. `T` muestra el texto en la línea del nodo, y vuelve; `H` pasa por árbol, texto y hexadecimal. `&`, `g`, `#` y Ctrl+B actúan sobre las líneas del texto.

- Un nodo cerrado dice lo que contiene: `{12 keys}`, `[340 items]`.
- Un número se muestra como lo escribe el archivo: `12345678901234567890` sigue así, y `1_000` en TOML también.
- Una cadena muestra el salto de línea como `↵`.
- Una clave que aparece dos veces en un mismo objeto se muestra dos veces, marcada `dup key`, y la cabecera las cuenta. Los programas no se ponen de acuerdo sobre cuál de las dos vale, lo que puede ocultar un valor en una revisión.
- Un ancla de YAML se muestra como `&a1` y un alias como `*a1`: el alias nunca se expande, así que un archivo hecho para multiplicarse (un "billion laughs") no cuesta más que su texto. Una etiqueta (`!Ref`) aparece antes del valor. Varios documentos quedan bajo una raíz, como `document 1`, `document 2`...

Un archivo no válido, o anidado a más de 512 niveles, se abre como texto, y el aviso dice por qué y en qué línea. El árbol se arma con el archivo entero: uno por encima del límite de lectura se abre como texto. El árbol también se detiene en 2.000.000 nodos (unos 170 MiB de memoria) o cuando construirlo tarda demasiado; entonces contiene solo el comienzo del archivo, y la cabecera dice `(tree of the start only)`. `T` sigue mostrando todo el texto.

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

## Libros electrónicos

EPUB (`.epub`, versiones 2 y 3) y FictionBook (`.fb2`, y `.fb2.zip` con un solo libro dentro) se abren como un libro para leer. Las imágenes no se muestran: cada una es una línea, como `[image: cover.jpg]`.

**El libro.** Se abre en una portada: el título, el autor, quién lo publicó y cuándo, cuántos capítulos y palabras tiene, y de qué avisa el informe, si de algo. El texto es una columna en el centro de la ventana (70 celdas por defecto), con los párrafos sangrados como en un libro impreso y el título de cada capítulo centrado bajo un adorno. Una separación entre escenas (`* * *`) se dibuja como tal. El pie dice el capítulo en que estás, cuánto llevas del libro, y el tiempo que queda del capítulo y del libro.

| Tecla | Qué hace |
|---|---|
| Space, PgDn | la página siguiente (PgUp la anterior) |
| `]` `[` | el capítulo siguiente o el anterior |
| Tab | el índice: ↑↓ eligen, las letras reducen la lista, Enter va, Esc cierra; Backspace vuelve después adonde estabas |
| Enter | las notas llamadas en la página, en un recuadro: ↑↓ lo desplazan, cualquier otra tecla lo cierra |
| `g` | ir a una línea, o a una parte del libro: `40%` |
| `R` | recordar dónde se quedó el libro; `R` otra vez olvida el libro |
| `I` | el informe sobre el archivo, y de vuelta al libro |
| `Z` | solo el texto, sin el marco ni el pie; Esc vuelve |
| `/` `\` | buscar hacia delante o hacia atrás; `+` y `-` van a la coincidencia siguiente o a la anterior (aquí Space pasa la página) |
| `H` | los bytes, y de vuelta |
| Esc | cerrar |

Las notas de un libro (un `aside` de tipo footnote en EPUB, el cuerpo `notes` de un FB2) no están en el texto: cada llamada es un superíndice (`¹`), y Enter muestra las notas llamadas en la página. En el informe están donde el libro las tiene.

**Recordar dónde te quedaste.** `R` recuerda el libro: al cerrarlo, Xylem guarda el capítulo y la línea en que estabas, y la próxima vez el libro se abre ahí, diciendo `Back at chapter 7 (R forgets this book)`. El pie de un libro recordado empieza por `[R]`. El libro se reconoce por lo que dice ser (su identificador) y su tamaño, no por su nombre, así que un libro movido o renombrado sigue reconociéndose, y otro archivo con el mismo nombre es otro libro. `R` otra vez olvida el libro y borra lo que se guardó de él. Un libro en el que nunca pulsaste `R` no deja nada.

**El tiempo dedicado a leer** se guarda para los libros recordados con `R`, si lo activas (**E** en la página 5 de la [configuración](configuration.md#reading-time)). La portada y el informe dicen entonces cuánto tiempo has leído el libro, en cuántas sesiones, hasta dónde llegaste y desde cuándo; el informe lista las sesiones. El pie muestra el tiempo de esta sesión.

**El informe** (`I`) es lo que el archivo es antes del texto: lo que el libro dice de sí mismo (título, autores, editorial, idioma, fecha, identificador, temas, serie, el programa que lo hizo), y lo que lleva además del texto, ya que un libro de cualquier parte es un archivo de cualquier parte:

- scripts: elementos `<script>`, `onload` y los demás atributos de evento, enlaces `javascript:`, y las partes que un EPUB declara con scripts. Nunca se ejecutan; un lector que ejecuta scripts sí lo haría;
- lo que un lector descargaría de fuera al abrir el libro: imágenes, hojas de estilo, fuentes y marcos con una dirección en internet, listados. Descargarlos le dice a quien esté allí que el libro se abrió, y desde dónde. Aquí no se descarga nada;
- formularios, marcos y objetos incrustados, contados, y el número de enlaces externos;
- DRM: un EPUB cuyos capítulos están cifrados dice con qué (Adobe ADEPT, Readium LCP, Apple FairPlay) y no se puede leer. Las fuentes ofuscadas, habituales en libros vendidos sin DRM, solo se cuentan;
- en un EPUB, las partes que el paquete lista pero el archivo no tiene, las que tiene pero no lista, y las direcciones que salen del paquete;
- en un FB2, los archivos incrustados en él (su número, tipos y tamaño), nunca decodificados, y una codificación declarada que Xylem no conoce.

Después vienen el índice, como enlaces (Tab y Enter los siguen), y el texto completo, con las notas donde el libro las tiene.

**Ajustes** (página 5 de la [configuración](configuration.md)): el ancho de la columna (o la ventana entera), párrafos sangrados y juntos o separados por una fila en blanco, texto justificado, y la velocidad de lectura con que se calcula el tiempo restante. Con Nerd Fonts activado (**M** en la página 2), los adornos son hojas, la barra de progreso se dibuja con los glifos propios de la fuente, y el pie y el índice llevan iconos.

**Qué se lee.** Un FB2 se lee en la codificación que declara (UTF-8, UTF-16, Windows-1251, KOI8-R y las demás de `O`), y `T` muestra su XML. Un EPUB es un ZIP: cada parte se descomprime solo hasta un límite (64 MiB cada una, 256 MiB en total; 4 MiB en Autoview, que dice `V shows it as a book` cuando un libro es más grande), un FB2 se lee hasta 64 MiB, y el informe dice cuándo solo se leyó el principio del libro. Ninguna DTD se lee ni se descarga nunca, un XML con entidades propias se rechaza, el texto nunca entra como Markdown, y las direcciones de un libro se toman solo como nombres de sus propias partes, nunca como rutas en el disco. MOBI y AZW3 (Kindle), PDF y cómics (CBZ, CBR) no se leen aquí: `O` en la ventana de archivos los abre en otro programa.

## Norton Guides

Una Norton Guide o una guía de Expert Help (`.ng`), la ayuda en hipertexto de DOS y Clipper, se abre como un documento. Un archivo necesita tanto el nombre `.ng` como la firma de la guía; cualquier otro se abre como antes.

- **Arriba:** el título y los créditos de la guía, y lo que la creó.
- **Los menús:** cada uno con sus elementos, y adónde lleva cada elemento.
- **Las entradas:** todas, en el orden del archivo, cada una bajo un número: `§12`. Una entrada de lista muestra sus líneas; una entrada de texto muestra su texto con la disposición de la guía, y después sus enlaces de véase también. Cada enlace dice adónde va (`→ §12`, `See also: Two() §3`), así que una búsqueda de `§12` te lleva allí. Un enlace que no apunta a ninguna parte dice `(broken link)`.

**Seguir enlaces.** **Tab** y **Shift+Tab** eligen el enlace siguiente o el anterior (desde la pantalla, cuando el elegido se ha salido de ella); **Enter** va a la entrada a la que lleva, con el primer enlace de esa entrada elegido; **Backspace** vuelve adonde estabas, a través de hasta 1.000 enlaces. Con el ratón activado, un clic en un enlace lo sigue, y un clic en `Backspace` en el pie vuelve atrás. En una guía Enter nunca cierra el visor: lo hace Esc. Como toda la guía es un solo documento, una búsqueda (`/`) sigue recorriéndola entera.

La negrita, el subrayado y el vídeo inverso salen con los estilos del tema; los colores DOS de una guía no se envían a la terminal. Los caracteres de la guía son de la página de códigos 437, con los glifos que DOS dibujaba para los bytes 1 a 31 (`☺ ♥ ► ▲`); nada de una guía llega a la terminal como carácter de control.

Una guía es una red de desplazamientos que podrían mentir: cada uno se comprueba contra el archivo, cada recuento contra su propio registro, y los espacios en que se expande una línea tienen un límite. Lo que está mal en una guía se lista arriba, bajo "Problems in this guide", y el resto se sigue mostrando. Una guía se lee entera, hasta 64 MiB; `H` muestra los bytes.

El formato se conoce gracias a Dave Pearson, que lo descifró en los años 90 y ha mantenido legibles las Norton Guides desde entonces: <https://www.davep.org/norton-guides/>. El lector de Xylem es código propio, escrito a partir de su descripción del formato; no incluye código de sus lectores.

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
- una ruta absoluta, que sube con `..`, que entra en `.git` (un hook es código que Git ejecuta), o con caracteres de control, de dirección del texto o invisibles (un nombre como `invoice‮fdp.exe` oculta su extensión real).

La cabecera también cuenta:

- `look unchanged`: una línea quitada y una añadida que solo difieren en lo que no se ve, como espacios, `\r`, caracteres invisibles o letras parecidas (una `а` cirílica en lugar de una `a` latina). El par se muestra en amarillo;
- `hunks with wrong counts`: un hunk cuya línea `@@` no coincide con sus líneas. El índice sigue las líneas que hay, nunca los números;
- `whitespace`: líneas añadidas con un espacio al final, o con un espacio antes de un tabulador en la sangría, como lo señala `git diff --check`.

Una serie de `git format-patch` muestra cada mensaje en la lista, por su asunto. Nunca se aplica nada. En un archivo de más de 16 MiB, leído a través de una ventana, los colores se mantienen, pero no hay índice: `]`, `}` y Tab lo dicen.

## Caracteres de control de dirección

Algunos caracteres invisibles cambian la dirección del texto y pueden hacer que un código parezca distinto de lo que realmente dice (Trojan Source, CVE-2021-42574). El visor nunca los oculta: cada uno aparece como un `·` rojo, y el encabezado indica cuántos hay.

## Codificación del texto

Xylem descubre la codificación solo, en este orden: primero descomprime el archivo, luego busca un BOM (UTF-8, UTF-16 LE o BE), luego UTF-16 sin BOM, luego UTF-8. Un archivo que no es nada de eso es texto antiguo: Xylem elige CP850 (DOS, Europa occidental) o Windows-1252 contando las letras con tilde. Los archivos `.reg` de Windows son UTF-16 y se abren como texto normal.

**O** muestra el archivo en la siguiente codificación: UTF-8, CP850, Windows-1252, CP437, Windows-1251, KOI8-R, UTF-16 LE, UTF-16 BE y de nuevo UTF-8. El pie dice cuál, y la cabecera muestra su nombre cuando no es UTF-8. Si eliges UTF-16 en un archivo que se abrió en hexadecimal, pasa a texto. Tu elección se mantiene después de editar el archivo y volver.

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
