# Vocabulario

Las palabras siguientes provienen de los gestores de archivos en árbol clásicos. Los menús y este manual
las emplean con estos significados.

| Término | Significado |
|---|---|
| Log | Registrar (leer) un directorio en la memoria: sus archivos y los nombres de sus subdirectorios. `→` o `+` registra un directorio y `*` una rama entera. `L` (Log) también es el comando que abre otro directorio como árbol. [La ventana del árbol](tree.md) |
| Logged | Registrado en la memoria. Solo los archivos registrados aparecen en las listas, se contabilizan en las estadísticas y pueden marcarse. |
| Not logged | Un directorio cuyo nombre se conoce pero cuyo contenido no ha sido leído. El árbol lo marca con `?`. |
| Release | Liberar una rama de la memoria con `-`. Sus archivos salen de las listas y pierden sus marcas. Nada cambia en el disco. |
| Relog | Volver a leer un directorio desde el disco con F3. |
| Branch | Una rama: un directorio y todo lo que se encuentra debajo de él. `B` lista los archivos de la rama en una sola lista. |
| Showall | La lista de todos los archivos registrados en la ventana, abierta con `S`. |
| Tag | Una marca en un archivo (`■`). Los comandos con Ctrl actúan sobre los archivos marcados. Las marcas se mantienen en memoria y se pierden con Release o al salir. [Marcas y operaciones en grupo](tagging.md) |
| Filespec | El filtro que determina qué archivos muestra la ventana, por nombre, fecha y tamaño. Se define con `F`. [Filespec](filespec.md) |
| Treespec | Una ruta escrita para mover el cursor del árbol. Se abre con `\` o `/`. [Treespec](treespec.md) |
| Ventana | Un árbol con su ventana de archivos. F8 abre una segunda al lado. [Pantalla dividida](split.md) |
| Raíz | El directorio en la parte superior del árbol. `L` lo cambia, y Alt+L convierte el directorio bajo el cursor en la raíz. |
| Matching | Los archivos registrados que coinciden con el Filespec. |
| Máscara | Un patrón que compone un nombre o una línea por archivo: `*.bak`, `foto_<001>.*`. Empleado por Rename y Batch. [Renombrar](rename.md) |
| Batch | Ctrl+B: escribe un archivo con una línea por cada archivo marcado, o por cada registro de una tabla en el visor. [Batch](execute.md) |
| Prune | Alt+P: elimina un directorio junto con todo su contenido. [Eliminar](delete.md) |
| Oops | Restaura el último elemento enviado a la papelera. `O` en el árbol, Ctrl+Z en ambas ventanas. |
| Yank | `Y`: copia una ruta al portapapeles. |
| Autoview | F7: la ventana de archivos mostrando el contenido del archivo bajo el cursor al lado de la lista. |
| eXecute | `X`: ejecuta un comando del shell, con `%f` para el archivo y `%t` para los archivos marcados. [eJecutar](execute.md) |
| Avail | `A` en el árbol: los sistemas de archivos montados con su espacio libre. |

Véase también: [qué es Xylem](overview.md), [los menús](menus.md).
