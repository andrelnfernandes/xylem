# Los menús DIR y FILE

Una letra sola ejecuta el comando sobre el elemento bajo el cursor: el
directorio en el árbol (el menú DIR), el archivo en la ventana de archivos (el
menú FILE).

Algunas letras representan comandos distintos en cada ventana. `C` compara en
el árbol y copia en la ventana de archivos; `M` crea un directorio en el árbol
y mueve un archivo en la ventana de archivos.

## Comandos

En el orden de los menús. "Árbol" y "Archivos" indican dónde funciona la tecla.

| Tecla | Command | Árbol | Archivos | Qué hace |
|---|---|---|---|---|
| `A` | Available | sí | | Lista los sistemas de archivos montados con espacio total, usado y libre. [Información](info.md) |
| `A` | Attributes | | sí | Modifica los permisos del archivo, en octal. [Permisos y fechas](attributes.md) |
| `B` | Branch | sí | sí | Lista los archivos del directorio y de todo lo que está debajo de él, leyendo la rama primero. En un Branch o Showall, regresa a la lista exclusiva del directorio. |
| `C` | Compare | sí | | Con la pantalla dividida, compara el directorio con el de la otra ventana y marca los archivos que difieren según el criterio elegido. [Comparar](compare.md) |
| `C` | Copy | | sí | Copia el archivo. El prompt solicita el destino. [Copiar y mover](copy-move.md) |
| `D` | Delete | sí | sí | Elimina el archivo, tras pedir confirmación. En el árbol, elimina el directorio si está vacío. [Eliminar](delete.md) |
| `E` | Edit | | sí | Abre el archivo en tu editor. [Editar](execute.md) |
| `F` | Filespec | sí | sí | Define qué archivos muestra la ventana, por nombre, fecha y tamaño. [Filespec](filespec.md) |
| `I` | Invert | sí | sí | Marca los archivos no marcados y desmarca los marcados: del directorio en el árbol, de la lista en la ventana de archivos. |
| `L` | Log | sí | | Abre otro directorio, o una máquina remota, como árbol de esta ventana. [Treespec y Log](treespec.md) |
| `M` | Makedir | sí | | Crea un subdirectorio en el directorio bajo el cursor. |
| `M` | Move | | sí | Mueve el archivo. [Copiar y mover](copy-move.md) |
| `N` | New date | sí | sí | Modifica la fecha de modificación del archivo o directorio. [Permisos y fechas](attributes.md) |
| `O` | Oops | sí | | Restaura el último elemento enviado a la papelera. En la ventana de archivos, Ctrl+Z. [Eliminar](delete.md) |
| `O` | Open | | sí | Abre el archivo en la aplicación predeterminada del sistema. [La ventana de archivos](files.md) |
| `P` | Phrase | sí | sí | Abre la caja de búsqueda: lista los archivos registrados que contienen las palabras escritas. [La caja de búsqueda](filespec.md#la-caja-de-busqueda-p) |
| `R` | Rename | sí | sí | Renombra el archivo o directorio. [Renombrar](rename.md) |
| `S` | Showall | sí | sí | Lista todos los archivos registrados de la ventana. |
| `T` | Tag | sí | sí | Archivos: marca el archivo y baja una línea. Árbol: marca los archivos del directorio. [Marcas y operaciones en grupo](tagging.md) |
| `U` | Untag | sí | sí | Archivos: desmarca el archivo y baja una línea. Árbol: desmarca los archivos del directorio. |
| `V` | View | | sí | Abre el archivo en el visor: texto, hexadecimal o tabla. [El visor](viewer.md) |
| `W` | Where | sí | sí | Mantiene en la lista los archivos que cumplen una condición SQL. [Una condición SQL](filespec.md#una-condicion-sql-w) |
| `X` | eXecute | sí | sí | Ejecuta un comando del shell. `%f` representa el archivo y `%t` los archivos marcados. [eJecutar](execute.md) |
| `Y` | Yank | sí | sí | Copia la ruta del archivo o directorio al portapapeles. |
| `Q` | Quit | sí | sí | Sale del programa, tras pedir confirmación. [Salir y cambiar el directorio de la shell](quit.md) |

Las teclas de la última línea del menú (F1, `*`, `?`, `\`, F3, Enter, F5, F7,
F8, F9) se describen en [teclas de función y símbolos](commands-keys.md).

## Teclas que no figuran en el menú

Son teclas alternativas para comandos de la tabla.

| Tecla | Equivalente a |
|---|---|
| Del | `D` |
| Shift+Del | `D`, eliminando definitivamente sin pasar por la papelera |
| F10 | `Q` |
| Ctrl+Insert | `Y` |
| Ctrl+Z | `O` (Oops), en ambas ventanas |

## Dentro de un archivo comprimido o en máquina remota

El título del menú es ARCHIVE o REMOTE, y los comandos que no pueden ejecutarse
allí no aparecen listados.

| Contexto | No listados |
|---|---|
| Archivo comprimido | Delete, Edit, Log, Makedir, Move, New date, Oops, Rename, Attributes, eXecute. Copy extrae archivos. |
| Máquina remota | Edit, Oops, eXecute. No hay papelera en máquinas remotas. |

Vea [archivos comprimidos](archives.md) y [máquinas remotas](remote.md).

Véase también: [el menú Ctrl](commands-ctrl.md), [el menú Alt](commands-alt.md),
[los menús](menus.md).
