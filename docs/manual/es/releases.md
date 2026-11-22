# Notas de versión

La versión más reciente primero. Cada sección enumera qué hay de nuevo, qué
cambió para quien usaba la versión anterior y las teclas que cambiaron. Las notas
completas de cada versión se encuentran en
`https://github.com/andrelnfernandes/xylem/releases`.

## 0.9.0

**Novedades.**

- Las Norton Guides (`.ng`), los archivos de ayuda de DOS y Clipper, se abren
  en el visor como un documento: los menús, y después cada entrada numerada
  (`§12`), con sus enlaces y su véase también apuntando a esos números. Tab
  elige un enlace, Enter o un clic lo sigue, Backspace vuelve atrás. [El
  visor](viewer.md)
- Búsqueda aproximada (`|`): escribe trozos de un nombre (`cfgxy` encuentra
  `config/xylem`) y el cursor va a la mejor carpeta, en el árbol, o al mejor
  archivo de la lista, en la ventana de archivos. Los resultados aparecen a
  medida que escribes, el mejor primero, incluso en un árbol de millones de
  carpetas. Ctrl+T marca el resultado elegido y Alt+T la lista. [Búsqueda
  aproximada](fuzzy.md)
- [Encontrar archivos y carpetas](finding.md): una página que dice qué
  búsqueda responde a qué pregunta.
- Libros electrónicos: EPUB y FictionBook (`.fb2`, `.fb2.zip`) se abren como
  un libro para leer, en una columna con los párrafos de un libro impreso: una
  portada, Space para la página siguiente, `]` `[` para los capítulos, Tab
  para el índice, Enter para las notas llamadas en la página, `g 40%` para una
  parte del libro, y cuánto queda del capítulo y del libro. `I` muestra el
  informe sobre el archivo: lo que el libro dice de sí mismo, y si lleva
  scripts, cosas descargadas de fuera o DRM. [El visor](viewer.md#ebooks)
- `R` en un libro recuerda dónde te quedaste, y el libro se abre ahí la
  próxima vez. Si lo activas (página 5 de F12), Xylem también guarda el tiempo
  que pasaste leyendo cada libro recordado, tus sesiones y tu velocidad de
  lectura, solo en tu ordenador. [Tiempo de
  lectura](configuration.md#reading-time)
- F1 abre la ayuda en la sección de lo que estás haciendo: en el Batch del
  visor (Ctrl+B), en "Batch en el visor", no al principio de la página. Los
  enlaces a una sección de una página de la ayuda llevan ahora a ella.
- Xylem puede saber que hay una versión nueva. La primera vez que arranca,
  pregunta si puede buscarla una vez al día; con un sí, solo descarga del
  sitio web el número de la última versión, y la línea superior dice cuándo
  hay una más nueva. No se descarga ni se instala nada. [Versiones
  nuevas](configuration.md#new-versions)
- Vistas (Alt+V): otras formas de mirar la lista. El Escaneo (`S`) señala lo
  que en una carpeta merece una mirada antes de abrirlo: nombres con
  caracteres ocultos o una doble extensión (`fatura.pdf.exe`), archivos
  setuid y lo que cualquiera puede escribir, enlaces que salen de la carpeta
  o no apuntan a nada, y, leídos con los propios lectores y límites del visor,
  ejecutables sin PIE o NX o empaquetados, documentos de Office con macros o
  una plantilla remota, parches que se salen con `..`. Enter en una categoría
  devuelve sus archivos a la lista, para Ctrl+T y lo demás. La Línea de tiempo
  (`T`) muestra los archivos de la lista en un histograma por fecha, por
  horas, días, semanas, meses o años; Enter en una barra, o en un rango, deja
  solo sus archivos, como una condición `W`. La Tabla (`L`) abre la lista en
  el visor de tablas, un archivo por fila con las columnas de `W`, incluso
  una lista de millones de archivos; `W` allí devuelve su filtro SQL a la
  lista. [Vistas](views.md)

**Cambios.**

- `O` en el visor pasa por dos codificaciones más, Windows-1251 y KOI8-R
  (cirílico), antes de UTF-16.
- La visualización Type (Alt+F) dice `EPUB ebook (ZIP)`, `FB2 ebook` y
  `FB2 ebook (ZIP)`.

**Correcciones.**

- Con Git activado, un repositorio grande en un sistema de archivos lento
  (`/mnt/c` desde WSL, una carpeta compartida en red) congelaba la pantalla
  durante minutos mientras se ejecutaba `git status`. El estado se lee ahora
  en segundo plano, y un repositorio lento se vuelve a leer con menos
  frecuencia. [Git](git.md)
- La biblioteca que Xylem usa para las llamadas al sistema (rustix 1.1.6) se
  actualiza por un aviso sobre los atributos extendidos
  ([RUSTSEC-2026-0486](https://rustsec.org/advisories/RUSTSEC-2026-0486)).

## 0.8.0

**Cambio: algunas teclas Alt.** Alt ya no repite Ctrl. Pasa a ser la otra
versión de un comando.

| Tecla | Hasta 0.7 | Desde 0.8 |
|---|---|---|
| Alt+A | Attributes de los archivos marcados | nada (Ctrl+A) |
| Alt+D | elimina los archivos marcados | Duplicates (Ctrl+D los elimina) |
| Alt+R | renombra los archivos marcados | Release de la rama, como `-` (Ctrl+R los renombra) |
| Alt+T | marca todo | nada (Ctrl+T) |
| Alt+U | desmarca todo | Usage (Ctrl+U desmarca) |
| Alt+W | Usage | nada (Alt+U) |
| Alt+Y | Duplicates | nada (Alt+D) |

Para conservar las teclas antiguas, añádelas a `config.toml`, una a una o todas:

```toml
[keys.any]
"alt+a" = "attributes_tagged"
"alt+d" = "delete_tagged"
"alt+r" = "rename_tagged"
"alt+t" = "tag_all"
"alt+u" = "untag_all"
"alt+w" = "usage"
"alt+y" = "duplicates"
```

**Cambio: el pie de página.** Todo comando pertenece ahora a un menú y el pie
consta de tres líneas, o dos en una pantalla suficientemente ancha para mostrar
cada menú en una sola línea. En pantallas estrechas, las palabras se acortan
antes de omitir ningún comando. Dentro de un archivo comprimido y en una máquina
remota, los menús listan solo lo que funciona en ese entorno.
[Los menús](menus.md)

**Cambio: prompts.** ↑ abre el historial de un prompt en forma de lista; ↓ ya no
recorre las entradas individualmente. `B` en un Branch o Showall regresa a la
lista propia del directorio. La ventana de archivos reducida ocupa una cuarta
parte de la altura de la pantalla.

**Novedades.**

- `W` conserva en la lista los archivos que cumplen una condición SQL; Ctrl+W lo
  hace entre los archivos marcados. `contains('texto')` lee el contenido de los
  archivos y `words('texto')` consulta el índice generado con Alt+N.
  [Una condición SQL](sql.md#una-condición-sql-w)
- `P` abre una caja de búsqueda que lista los archivos por las palabras escritas.
  [La caja de búsqueda](sql.md#la-caja-de-búsqueda-p)
- Las tablas CSV, TSV, JSON Lines, DBF y SQLite no tienen límite de filas en el
  visor. Las hojas de cálculo mantienen el límite de 50.000 filas.
  [Tablas](tables.md)
- En una tabla, Enter muestra el registro como ficha, y `c` lo copia; `c`
  también copia la estructura mostrada por `i`. [Tablas](tables.md)
- Un texto de más de 16 MiB se lee a través de una ventana, sin límite de
  tamaño. [Archivos grandes](viewer.md#archivos-grandes)
- Visores nuevos: JSON, YAML y TOML como un árbol que se pliega; logs con los
  colores de sus niveles, y seguimiento; parches con un índice de archivos y
  hunks y lo que hay que revisar; notebooks de Jupyter; documentos de Office,
  con sus macros, plantillas remotas y texto oculto; ejecutables (ELF, PE,
  Mach-O) y certificados y claves como informes. [El visor](viewer.md)
- Se compila en macOS (todavía sin probar en un Mac).
- Alt+L convierte el directorio bajo el cursor en la raíz del árbol.
- En Treespec, Ctrl+← quita la última carpeta de la ruta y Ctrl+→ la restablece.
- `clock_seconds` muestra los segundos en el reloj del encabezado.
- El ratón, apagado por defecto (F12, página 2, R): la rueda, los clics y
  dobles clics en el árbol, las listas y las teclas del pie, marcar con el
  botón central, un menú en el botón derecho (también la tecla Menú y
  Shift+F10), y el ratón pasado a los programas de la terminal integrada y del
  editor en el marco.
  [El ratón](mouse.md)
- Este manual: las páginas sobre la pantalla, los menús, los comandos de cada
  menú, el vocabulario y las recetas paso a paso.

## 0.7.0

**Cambio: registro de directorios (Log).** Un directorio solo se lee cuando se
solicita expresamente. Mover el cursor sobre uno que no ha sido registrado
muestra `Not logged`; `+` o → lo registra. Antes, pasar por encima de un
directorio lo leía automáticamente. [La ventana del árbol](tree.md)

**Cambio: prompts.** Log, y Copy y Move sin pantalla dividida, se abren vacíos.
↑ lista los lugares utilizados anteriormente.

**Novedades.**

- Las máquinas remotas dejan de ser de solo lectura: Copy hacia ellas, Move,
  Rename, Makedir, Delete, Prune, Attributes, New date y Duplicates funcionan
  allí. [Máquinas remotas (SSH)](remote.md)
- `docker://container/path` abre un contenedor mediante `docker exec`.
- Los marcadores pueden tener nombres y un marcador en máquina remota conserva
  la dirección que lo abre. F11 en los prompts de Log, Copy y Move inserta un
  marcador en el prompt. [Marcadores](bookmarks.md)
- Las carpetas montadas por rclone y los recursos compartidos de red se leen en
  segundo plano; el árbol se muestra al instante.
  [Almacenamiento en la nube (rclone)](cloud.md)
- La vista de Uso permite marcar, eliminar, podar ramas (Prune) y ver archivos
  desde su propio interior.
- Pantalla de inicio, nombre y versión en el encabezado, nombres de archivos
  coloreados por tipo y bordes redondeados. [Configuración](configuration.md)

## 0.6.1

Actualiza el lector de hojas de cálculo (calamine 0.38) por el aviso de
seguridad RUSTSEC-2026-0436.

## 0.6.0

**Novedades.**

- El visor muestra hojas de cálculo, CSV, JSON Lines, DBF, SQLite y Parquet como
  tablas con filtro SQL. Una base de datos SQLite admite además una consulta
  completa de solo lectura. [El visor](viewer.md)
- Ctrl+B en el visor escribe un archivo a partir de los registros de una tabla o
  las líneas de un texto: JSON, CSV, XML, HTML, una plantilla o una tabla
  SQLite. [Batch](execute.md)
- Duplicates localiza archivos y carpetas idénticos, en discos locales y mediante
  los hashes que reporta rclone. La tecla era Alt+Y; pasa a ser Alt+D a partir
  de la 0.8. [Duplicados](duplicates.md)
- Los archivos Markdown se abren renderizados en el visor. `T` muestra el
  código fuente.
- El visor filtra líneas mediante una expresión regular (`&`).
  [Filtrar líneas](filter.md)
- `*` en una carpeta montada por rclone solicita toda la rama en una única
  petición.
- Explorar un repositorio Git no ejecuta nada definido por el repositorio, salvo
  que su carpeta figure en `git_trusted`. [Git](git.md)

## 0.5.0

**Cambio.** `-` en el árbol libera la rama de la memoria; ← solo la contrae. `E`
abre el editor dentro del marco de Xylem.

**Novedades.**

- Medidas de seguridad ante archivos hostiles: límites de descompresión,
  comprobaciones en rutas extraídas, caracteres de control retenidos antes de
  llegar a la terminal. [Seguridad](security.md)
- El visor ajusta líneas (`W`), las numera (`#`), salta a una línea (`g`),
  muestra archivos `.gz`, `.xz`, `.zst` y `.bz2` descomprimidos y reconoce
  CP437, CP850, Windows-1252 y UTF-16.
- Ayuda en seis idiomas; `L` en la ayuda permite alternar entre ellos.
- Ctrl+S en el árbol lista solo los archivos marcados de todo lo registrado.
- Opción Graph en el menú Git.
- Iconos con Nerd Fonts, desactivados por defecto.
- Paquetes `.deb` y `.rpm`.

## 0.4.0

**Novedades.**

- Vista de Uso: las carpetas y archivos de una rama según el espacio que ocupan.
  La tecla era Alt+W; pasa a ser Alt+U a partir de la 0.8.
  [Uso de disco](usage.md)
- Alt+K Percent muestra la cuota porcentual de cada directorio respecto a su
  directorio padre.
- Máquinas remotas a través de SSH, de solo lectura en esta versión.
  [Máquinas remotas (SSH)](remote.md)
- Google Drive, S3 y otros almacenamientos mediante rclone.
  [Almacenamiento en la nube (rclone)](cloud.md)
- Temas slate, graphite, daylight y vivid.
- `[editors]` en `config.toml` selecciona el editor por el nombre del archivo y `E` rechaza archivos binarios que carecen de editor configurado.
- `xy --man` muestra la página de manual.

## 0.3.0 y anteriores

- 0.3.0: ayuda en F1 y página de manual; archivos comprimidos 7z, tar.xz,
  tar.bz2 y tar.zst; modos de visualización por propietario y tipo (Alt+F);
  visualización simultánea de ambos archivos en conflicto de copia.
- 0.2.0: cambio de nombre del programa de xt a Xylem y del comando a `xy`.
  Eliminación a la papelera y Oops, renombrado por lotes con máscara y
  portapapeles.
- 0.1.0: primera versión, publicada como xt.
