# Manual de Xylem

Xylem es un gestor de archivos en modo texto con la pantalla y las teclas de
XTree y ZTreeWin: un árbol de directorios arriba, los archivos del directorio
actual abajo y estadísticas a la derecha. Lee (registra) directorios en
memoria, de modo que una sola lista puede reunir archivos de toda una rama y un
único comando copia, mueve, elimina, compara o renombra todos los archivos
marcados en ella.

Xylem trata los archivos que no creó como hostiles: el contenido o el nombre de
un archivo nunca debe hacerlo fallar, colgarse, ejecutar código, escribir fuera
del destino ni agotar la memoria. [Seguridad](security.md) enumera las medidas
adoptadas y lo que no está cubierto.

**F1** (o **Alt+H**, para terminales que reservan F1 para sí mismas) abre la
ayuda sobre lo que está en pantalla: dentro de un prompt, el tema de ese prompt;
en el árbol o en la ventana de archivos, el tema de esa ventana.

| Tecla en la ayuda | Qué hace |
|---|---|
| Tab, Shift+Tab | selecciona el enlace siguiente o el anterior |
| Enter | sigue el enlace |
| Backspace | vuelve atrás |
| `I` | este índice |
| `K` | las teclas vigentes |
| `L` | cambia el idioma |
| Esc | cierra |

Las versiones de este manual en español, francés, alemán y chino no han sido
revisadas por hablantes nativos. Correcciones:
`https://github.com/andrelnfernandes/xylem/issues`.

## Empieza por aquí

- [Qué es Xylem](overview.md): lectura, marcas y lo que cubren los comandos
- [Instalar y los primeros cinco minutos](start.md)
- [La pantalla](screen.md): el árbol, la ventana de archivos, las estadísticas, el pie
- [Los menús, Ctrl y Alt](menus.md): una letra sola, con Ctrl, con Alt, y F9
- [The mouse](mouse.md) (in English for now): turning it on in F12, the wheel, clicks and the footer
- [Vocabulario](glossary.md): log, branch, tag, filespec

## Comandos

- [Los menús DIR y FILE](commands.md): las letras solas
- [El menú Ctrl](commands-ctrl.md): los archivos marcados
- [El menú Alt](commands-alt.md): la rama, las rutas y las herramientas
- [Git](git.md): el menú Git (Ctrl+G)
- [Teclas de función y símbolos](commands-keys.md)
- [Las teclas vigentes](keys.md): con los cambios de la configuración

## Ventanas y vistas

- [La ventana del árbol](tree.md): navegación, lectura (Log), Branch y Showall
- [La ventana de archivos](files.md): modos de visualización, Autoview, abrir archivos
- [Pantalla dividida](split.md): dos ventanas lado a lado
- [El visor](viewer.md): texto, hexadecimal y tablas, búsqueda, colores de sintaxis
- [Filtrar líneas](filter.md): `&` en el visor, con una guía de expresiones regulares
- [Buscar en archivos marcados](search.md): Ctrl+S y Ctrl+V
- [Información](info.md): Info, Avail, estadísticas y tamaños de directorios
- [Uso de disco](usage.md): adónde va el espacio (Alt+U)
- [Duplicados](duplicates.md): archivos y carpetas con el mismo contenido (Alt+D)

## Trabajar con archivos

- [Marcas y operaciones en grupo](tagging.md)
- [Copiar y mover](copy-move.md), [elegir una carpeta](picker.md)
- [Eliminar, papelera y Prune](delete.md)
- [Renombrar y renombrado por lotes](rename.md)
- [Permisos y fechas](attributes.md)
- [Filespec: el filtro de archivos](filespec.md)
- [Treespec y Log](treespec.md)
- [Comparar](compare.md) y [Ordenar](sort.md)
- [Editar, eJecutar y Batch](execute.md)
- [Archivos comprimidos](archives.md): ZIP, 7z y TAR

## Más allá de XTree

- [La terminal integrada](terminal.md)
- [Máquinas remotas (SSH)](remote.md): otra máquina o un contenedor como árbol
- [Almacenamiento en la nube (rclone)](cloud.md): Google Drive, S3 y otros
- [Encontrar archivos con SQL](sql.md): una condición sobre los archivos y su contenido (W), y las palabras que contienen (P)
- [Tablas](tables.md): hojas de cálculo, CSV, dBase, SQLite y Parquet como cuadrícula, con filtro SQL y cada registro como ficha
- [Marcadores](bookmarks.md)

## Cómo hacer

- [Copiar archivos de varias carpetas conservando las rutas](howto-copy-with-paths.md)
- [Renombrar muchos archivos con un contador](howto-rename-mask.md)
- [Elegir archivos con una condición SQL](howto-sql-filter.md)
- [Encontrar duplicados y conservar una copia](howto-duplicates.md)
- [Encontrar lo que llena el disco](howto-disk-space.md)
- [Filtrar un CSV o una hoja de cálculo con SQL](howto-table-sql.md)
- [Copiar archivos de otra máquina por SSH](howto-ssh-copy.md)
- [Usar los archivos marcados en un comando del shell](howto-terminal-tagged.md)

## Referencia

- [Configuración](configuration.md): la pantalla F12, temas, teclas
- [La línea de comandos](cli.md)
- [Terminales y teclas](terminals.md): F1, Ctrl+M, el protocolo de teclado de kitty
- [Seguridad](security.md)
- [Salir y cambiar el directorio de la shell](quit.md)
- [Windows](windows.md)
- [Comparación con ZTreeWin](ztreewin.md)
- [Notas de versión](releases.md)
