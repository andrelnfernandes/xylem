# Comparaciones

## Un agradecimiento

Uso alguna variante de XTree desde los años noventa y tengo una licencia
pagada de ZTreeWin desde 2013. Para mí, ZTreeWin sigue siendo el mejor gestor
de archivos en modo texto que existe: rápido, predecible, manejado por
completo con el teclado y construido en torno a una forma de trabajar (leer el
árbol, marcar archivos en ramas enteras, actuar sobre las marcas) que ningún
otro gestor de archivos ha igualado. Kim Henkel mantiene viva la forma de
trabajar de XTree en Windows desde hace décadas, y Xylem no existiría sin ese
trabajo.

Xylem no pretende reemplazar a ZTreeWin. Empezó porque yo quería la misma
forma de trabajar en Linux, donde ZTreeWin no funcionaba. Desde la
[beta 2.9.12](https://www.ztree.com/html/beta.htm), de noviembre de 2026, funciona, como ZTreeLin (y en macOS,
como ZTreeMac). Xylem sigue como otra versión de la misma forma de trabajar:
de código abierto, hecho primero para Linux, con lo que está en
[Donde Xylem va más allá](#where-xylem-goes-further), más abajo. Xylem
también compila para Windows, porque Rust lo hace fácil, pero esa no es una
meta mía: en Windows, ZTreeWin vale con creces su licencia:
[ztree.com](https://www.ztree.com/), y el [foro](https://groups.io/g/ZTreeWin)
de sus usuarios.

El visor lee Norton Guides, los archivos de ayuda de DOS y Clipper, gracias a
Dave Pearson, que descifró su formato en los años 90 y los ha mantenido
legibles desde entonces:
[davep.org/norton-guides](https://www.davep.org/norton-guides/).

## Otros gestores de archivos para la terminal

Hay muchos buenos gestores de archivos para la terminal, y la mayoría de
quienes prueban Xylem ya usan uno. Así se diferencian, con la mayor justicia
que puedo. Verificado en la documentación de cada proyecto en noviembre de
2026; estos programas cambian rápido, así que si algo aquí está
desactualizado, abre una issue.

| Programa | Disposición | Conocido por |
|---|---|---|
| ranger | tres columnas (directorio padre, actual, vista previa) | teclas de vi, vistas previas mediante scripts, Python para extenderlo |
| lf | tres columnas, como ranger | un único binario pequeño, rápido, comandos de shell como extensión |
| yazi | tres columnas | velocidad (todo asíncrono), vista previa de imágenes, plugins en Lua con gestor de paquetes, SFTP integrado |
| superfile | varios paneles lado a lado | un aspecto moderno, vista previa de imágenes, temas |
| nnn | una lista | diminuto y rápido, una selección entre directorios, un modo de uso del disco, muchos plugins |
| Midnight Commander (mc) | dos paneles | el clásico: visor y editor propios, SFTP y archivos comprimidos como carpetas, los resultados de Find file en un panel |
| vifm | dos paneles | teclas y comandos de vim, los resultados de `:find` como una lista sobre la que trabajar, una vista de árbol |
| broot | un árbol que cabe en la pantalla | búsqueda aproximada en todo el árbol, tamaños, un área de preparación (staging) para actuar sobre archivos de muchas carpetas |
| far2l | dos paneles | Far Manager para Linux, con sus plugins: SFTP, SMB, NFS, WebDAV y S3 (NetRocks), un editor hexadecimal, scripts en Python |
| ZTreeLin | un árbol de directorios sobre la lista de archivos, pantalla dividida | la forma de trabajar de XTree de ZTreeWin en Linux, de sus mismos autores: leer el árbol, marcar en ramas enteras, actuar sobre las marcas; un editor hexadecimal y archivos comprimidos; en beta desde noviembre de 2026 |

**Lo que distingue a Xylem.** El árbol se lee en memoria, y una tecla muestra
en una sola lista los archivos de toda una rama, o de todo lo leído (Branch,
Showall), acotados por nombre, fecha y tamaño (Filespec) o por una condición
SQL (`W`), con los bytes que coinciden y los marcados contados sobre la
marcha. Marca allí lo que quieras y cópialo, muévelo, renómbralo o elimínalo
todo de una vez. Varios de los programas de arriba también pueden reunir en
una lista archivos de muchas carpetas (el panelize de mc, las vistas
personalizadas de vifm, los resultados de búsqueda de yazi, el área de
preparación de broot), o mantener una selección entre carpetas (nnn, yazi); en
Xylem, como en ZTreeLin, esa lista es la forma de trabajar todo el tiempo, no el resultado de una
búsqueda. Además, ninguno de los demás reúne en un mismo programa:

- tablas con filtro SQL para hojas de cálculo, CSV, SQLite (con `JOIN`) y
  Parquet, con sus registros convertidos en un script, una página o una base
  de datos (Batch). Un plugin de yazi basado en DuckDB muestra una vista previa
  y un resumen de archivos CSV y Parquet, sin filtro, consulta ni exportación;
- informes sobre archivos en los que no confías: las macros, plantillas remotas
  y texto oculto de los documentos de Office, las protecciones y señales de
  empaquetado de los ejecutables, los nombres que revisar en un parche, sin
  ejecutar nada de lo que contienen;
- archivos y carpetas duplicados, incluso en Google Drive sin descargarlos;
- una terminal integrada cuyo árbol sigue el `cd` del shell;
- los archivos de fuera tratados como hostiles en todas partes, con un límite
  para todo lo que se lee ([Seguridad](security.md)).

**Donde los demás van por delante.** Vista previa de imágenes (yazi,
superfile, ranger). Un sistema de plugins y los plugins ya escritos para él
(yazi, nnn, ranger, lf, far2l): Xylem no tiene, solo eXecute y Batch. Teclas
de vi desde el principio (ranger, lf, yazi, vifm), donde Xylem tiene sus
propios comandos de una letra. Años de uso por mucha gente, y versiones para
macOS y Windows probadas, mientras que Xylem es nuevo, no se ha probado en un
Mac y es experimental en Windows.

## ZTreeWin

Esta parte compara los dos comando por comando, con base en el manual oficial
de ZTreeWin v2.2 ([ztwhlp.pdf](https://www.zedtek.com/download/ztwhlp.pdf), de
2011, en especial la "Quick Command Table" y la sección 3.7 sobre
configuración). Las versiones más recientes de ZTreeWin pueden haber cambiado
algunos detalles; si algo aquí está desactualizado, abre una issue.

### La misma forma de trabajar

Árbol con lectura bajo demanda (`+`, `*`), ventana de archivos, Filespec,
Branch, Showall (también solo de los marcados, Ctrl+S), marcas (T, U, I,
Ctrl+T/U), Copiar/Mover/Eliminar/Renombrar con las versiones Ctrl (marcados) y
Alt (marcados con rutas), crear directorio, eliminar directorio, Prune,
Atributos, Visor, Editar, eJecutar, Comparar (directorio y rama), Ordenar, Log,
pantalla dividida (F8 y Tab), releer (F3), Abrir (`O`), actualización
automática, exploración y extracción de archivos comprimidos, historial en los prompts, marcadores, ayuda (F1) y pantalla de configuración. Las teclas son las
de ZTreeWin siempre que es posible; las excepciones están en "Teclas
diferentes", al final.

### Lo que Xylem todavía no tiene

- **Árbol:** Global (G, y Ctrl+G para los marcados), Graft (Alt+G), Mirror (Alt+M), liberación parcial (Ctrl+Backspace), Hide (Alt+H), Wash (Alt+W), Volume
  (Ctrl+V), F5/F6 (ocultar niveles de subdirectorios), marcar por atributo
  (Alt+T/U), Treespec en modo de edición (F5, comodines) y carpetas especiales
  (F6), búsqueda directa en modo activo (sin prompt), cambio de unidad (`<`
  y `>`). La búsqueda directa con prompt (`|`) existe, con coincidencia
  aproximada y también en la ventana de archivos: [Búsqueda aproximada](fuzzy.md).
- **Archivos:** Abrir en una nueva ventana de terminal (Alt+O), Batch del
  archivo actual (Alt+B), JFC (J, comparar el contenido de dos archivos),
  sHortcut (H), Imprimir (P), propietario/grupo (chown), Megaview (Ctrl+F7 en la
  ventana de archivos), combinar archivos marcados entre ventanas (Ctrl+F6),
  desmarcado parcial (Ctrl+F8), Tags-Only fuera de Showall (Ctrl+Enter y
  Ctrl+F4, Ctrl+B en el árbol).
- **General:** macros (F12), menú de aplicaciones (F9), salir a la shell
  (Ctrl+Q), guardar y cargar el registro (Zlist, Ctrl/Alt+Z), colores por nombre
  y atributo (ZCOLORS.INI), opciones de línea de comandos de ZTreeWin,
  comandos de vídeo (Alt+F6–F9).

Para algunas de ellas hay un sustituto en otro programa; consulta "Lo que Xylem
deja a otros programas", más abajo.

### Donde Xylem es más sencillo

| Función | En ZTreeWin | En Xylem |
|---|---|---|
| Filespec | nombre con reglas propias de nombre y extensión, fecha, tamaño, atributo, filtro invertido (Ctrl+I), etiquetas | glob para nombres, fecha y tamaño como en ZTreeWin (con K/M/G); sin atributos, inversión ni etiquetas |
| Visor | modos Alpha, Dump, Hex, Junk y Wrap; juegos de caracteres; edición hexadecimal; marcadores; desplazamiento automático | texto, hexadecimal, tablas con filtro SQL y Markdown renderizado, con la búsqueda de ZTreeWin (F, B, /, \\, Space, + y -), ajuste de línea (W), números de línea (#), ir a línea (g / :), filtro de líneas con regex (&); un archivo en disco de cualquier tamaño, leído a medida que avanza la pantalla; sin edición hexadecimal, por decisión propia (consulta "Lo que Xylem deja a otros programas") |
| Ctrl+S | texto, hexadecimal o Unicode; en archivos ZIP busca solo en el índice | texto o hexadecimal |
| Atributos | atributos de Windows (R, A, S, H) | permisos Unix en octal (`644`); sin propietario/grupo. En Windows, R, H, S y A, como en ZTreeWin |
| Ordenar | Name, Ext, Date, Time, Length, Alpha, nuMber, Size, Value, Unsorted; Charset; igual/diferente en pantalla dividida | Name, Ext, Date, Size, ascendente/descendente, Path primero; cada ventana tiene su propio orden |
| Comparar | opciones Identical, Unique, Newer, Older, Size, Binary, Case; filtro de visualización | marca únicos, más recientes, más antiguos, idénticos, diferentes o únicos+más recientes; sin comparación binaria |
| Prune | elimina solo los subdirectorios o también el actual; solo directorios vacíos; papelera de reciclaje | elimina siempre todo el directorio, permanentemente; confirmación configurable |
| Log | varias unidades registradas, Global, cambio de unidad | una raíz por ventana; `L` abre otro directorio |
| eJecutar | parámetros de lote `%1`–`%8`, ventana nueva (NTW) | `sh` con `%f`, `%t` y `%d`; siempre en la terminal de Xylem |
| Historial | ventana de historial con marcadores | ↑ en el prompt abre una lista (escribir filtra, Del olvida), un historial por tipo, guardado entre sesiones |
| Configuración | 7 páginas, editor de color por elemento (F4), Alt-Export | 4 páginas con las opciones que tienen sentido en Linux; F4 elige entre 12 temas listos para usar; colores individuales en `config.toml` |
| Pantalla dividida | paneles de estadísticas opcionales por lado | sin panel de estadísticas |
| Menús | menú Ctrl/Alt al mantener pulsada la tecla | igual en terminales con el protocolo de kitty; F9 en cualquier terminal |

Opciones de configuración de ZTreeWin que no se trajeron por ser específicas de
Windows: atributos RSH, uniones (junctions), OEM/ANSI, EBCDIC, tamaño de
ventana, accesos directos (.lnk).

### Donde Xylem va más allá

Cosas para las que el manual de ZTreeWin v2.2 no tiene equivalente:

- **Una terminal integrada** (Alt+X): la shell sigue en marcha mientras usas
  el árbol; un `cd` en la shell lleva el árbol consigo, y la shell sigue al
  árbol cuando vuelves a ella. Alt+O escribe los archivos marcados en la línea
  de comandos. Consulta [la terminal integrada](terminal.md).
- **Máquinas remotas por SSH** (`xy usuario@host:`): el árbol, Branch, el visor
  y Copiar funcionan en otra máquina, que también tiene su propia terminal.
  Consulta [máquinas remotas](remote.md).
- **Almacenamiento en la nube:** Google Drive, S3 y los demás servicios que admite rclone, montados y mostrados como un árbol. Consulta [almacenamiento en la nube](cloud.md).
- **SQL sobre la lista de archivos** (`W`): `ext = 'jpg' AND size > 10000000` deja los archivos que cumplen la condición, y `contains()` y `words()` miran dentro de ellos; la caja de búsqueda (`P`) encuentra archivos por las palabras que contienen. Consulta [encontrar archivos](sql.md).
- **Git** (opcional): letras de estado en la lista de archivos, un menú de Git
  (Ctrl+G) y diffs en el visor. Consulta [Git](git.md).
- **Uso del disco** (Alt+U): las carpetas y archivos de una rama juntos, los
  más grandes primero, hasta llegar a lo que llena el disco; Alt+K muestra en
  el árbol la parte de cada carpeta. Consulta [uso del disco](usage.md).
- **Duplicados** (Alt+D): archivos y carpetas enteras con el mismo contenido,
  comparados por SHA-256, o por los hashes del propio servicio en una carpeta
  montada con rclone, sin descargar nada. Siempre queda una copia de cada grupo.
  Consulta [duplicados](duplicates.md).
- **Un visor que lee más:** colores de sintaxis, la codificación del texto
  detectada sola (UTF-8, UTF-16, CP850, Windows-1252, CP437), archivos `.gz`,
  `.xz`, `.zst` y `.bz2` mostrados descomprimidos, y los caracteres ocultos de
  dirección del texto (Trojan Source) mostrados en vez de obedecidos. Consulta
  [el visor](viewer.md).
- **Tablas y Markdown:** hojas de cálculo, CSV, JSON Lines, DBF, SQLite y
  Parquet se abren en una cuadrícula con filtro SQL, y los archivos Markdown se
  abren renderizados. Consulta [tablas](tables.md).
- **Archivos comprimidos sin compresor externo:** ZIP, 7z y TAR (sin
  comprimir, gzip, xz, bzip2, zstd) se abren como carpetas, y ZIP y TAR se
  crean, todo por el propio Xylem. Consulta [archivos comprimidos](archives.md).
- **Renombrado por lotes** con buscar y reemplazar (`s/foo/bar/`), contadores
  y cambios de mayúsculas/minúsculas. Consulta [renombrar](rename.md).
- **Batch sobre registros:** Ctrl+B en el visor convierte los registros de una
  tabla o las líneas de un texto en un script, JSON, CSV, XML, HTML, SQL o una
  base SQLite, con archivos de plantilla si quieres. Consulta [Batch](execute.md).
- **Oops:** O (o Ctrl+Z) devuelve a su lugar lo último que se envió a la
  papelera del sistema. Consulta [eliminar](delete.md).
- **El portapapeles en todas partes:** Y copia la ruta incluso por SSH o
  dentro de tmux (OSC 52).
- **Qué es cada archivo:** un modo de visualización (Alt+F) que dice, en el
  espíritu del comando `file`, qué contiene cada archivo, y otro con
  propietarios y permisos.
- **Aspecto:** 12 temas listos para usar e iconos Nerd Fonts opcionales. Toda
  tecla se puede reasignar en `[keys]` de `config.toml`. Consulta
  [configuración](configuration.md).
- **Archivos hostiles:** los archivos comprimidos, los nombres y los contenidos
  se tratan como potencialmente hostiles, con límites en todo lo que se lee y
  se descomprime (consulta la declaración de seguridad en el README).
- **Ayuda en seis idiomas** (F1 y luego L), el mismo texto de este manual y de
  la página man.

### Lo que Xylem deja a otros programas

Algunas cosas Xylem no las hace por sí mismo, a propósito: un programa
dedicado las hace mejor, y Xylem lo integra.

- **Edición hexadecimal:** ZTreeWin edita bytes en su propio visor. Xylem no
  tiene un editor hexadecimal propio: escribir y mantener uno sería mucho
  trabajo para poco uso, y editar bytes a mano es una forma fácil de corromper
  un archivo. En su lugar, E sobre un archivo binario abre `hexedit`, `heh`,
  `bvi` o `dhex` (el que esté instalado, o el que elijas) dentro del marco y
  los colores de Xylem, o uno gráfico como ImHex o GHex. Consulta
  [Editar](execute.md).
- **Editar texto:** tu editor (`$VISUAL`, `$EDITOR` o el definido en F12), y un
  editor distinto por tipo de archivo si quieres (`[editors]`).
- **SSH:** el `ssh` del sistema, así que tus claves, el agente,
  `~/.ssh/config` y los *jump hosts* funcionan como en cualquier otro sitio.
- **Almacenamiento en la nube:** Google Drive, S3 y otros mediante
  `rclone mount`; Xylem reconoce el montaje y se adapta. Consulta
  [almacenamiento en la nube](cloud.md).
- **Git:** el propio programa `git`.
- **Abrir archivos** (O): el programa del escritorio para cada tipo
  (`xdg-open`).
- **Comparar el contenido de dos archivos** (JFC en ZTreeWin): mientras no esté
  integrado, marca los dos archivos y ejecuta `diff %t` con eJecutar (X), o usa
  la terminal.
- **Imprimir** (P en ZTreeWin): `lp %t` con eJecutar.
- **Otros formatos de archivo comprimido** (el ZAAP de ZTreeWin): tu compresor,
  con eJecutar o en la terminal.

### Teclas diferentes

Las teclas siguen las de ZTreeWin. Las excepciones:

| Función | ZTreeWin | Xylem |
|---|---|---|
| Ayuda | F1 | F1 o Alt+H (en GNOME Terminal, F1 abre la ayuda de la terminal) |
| Configuración | Alt+F10 | F12 o Alt+F10 (en GNOME, Alt+F10 maximiza la ventana y nunca llega a Xylem) |
| Marcadores | F11 | F11, Alt+B o comilla simple `'` (en GNOME Terminal, F11 alterna la pantalla completa) |
| Mover marcados | Ctrl+M | Ctrl+M en terminales con el protocolo de kitty; en otras, `F9` `M` (o Alt+M y F6) |
| F2 | (nada) | vuelve a leer el directorio, igual que F3 |
| F9 | menú de aplicaciones (no implementado) | menús Ctrl/Alt |
| F12 | macros (no implementadas) | configuración |
| Alt+B | Batch del archivo actual (no implementado) | marcadores |
| Alt+H | Hide (no implementado) | ayuda |
| Alt+O | Abrir en una ventana de terminal nueva (no implementado) | escribe los archivos marcados en la terminal integrada |
| Alt+D | (nada) | Duplicados |
| Alt+T | marcar por atributo (no implementado) | (nada) |
| Alt+U | desmarcar por atributo (no implementado) | uso del disco |
| Ctrl+G | Global de los marcados (no implementado) | el menú Git (opcional) |
| Alt+Z | Zlist (no implementado) | la terminal integrada a pantalla completa |
| `/` | igual que `?` (estadísticas) | Treespec, como `\` (`?` sigue mostrando las estadísticas) |
| Ctrl+C, Ctrl+M en el árbol | (nada) | Copiar/Mover los archivos marcados del directorio actual |
| Copiar ruta al portapapeles | Ctrl+Insert | Y o Ctrl+Insert (Ctrl+Y para los marcados; funciona mediante OSC 52 y el portapapeles del sistema) |

Todas las teclas se pueden reasignar bajo `[keys]` en `config.toml`.

## Nombres

Xylem es un proyecto independiente, escrito desde cero. No está afiliado,
avalado ni vinculado a Zedtek, Inc. (ZTreeWin), a los propietarios de XTree ni
a los autores de los demás programas mencionados en esta página, y no contiene
código de ninguno de ellos. Sus nombres se mencionan solo para describir lo
que inspiró a Xylem y cómo se compara. XTree, ZTree y ZTreeWin son marcas de
sus respectivos propietarios.
