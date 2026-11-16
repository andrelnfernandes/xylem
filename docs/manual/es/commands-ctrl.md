# El menú Ctrl

Ctrl con una letra ejecuta el comando sobre los archivos marcados. En la
ventana de archivos, actúa sobre los archivos marcados de la lista (el
directorio actual, o la vista Branch o Showall abierta). En el árbol, actúa
sobre los archivos marcados del directorio bajo el cursor. Las filas indicadas
como "todo lo registrado" son las excepciones.

Si no hay ningún archivo marcado, estos comandos muestran `No tagged files` y
no realizan ninguna acción.

Para mostrar este menú, mantén pulsado Ctrl o pulsa F9. Consulta
[los menús](menus.md).

## Comandos

En el orden del menú. "Árbol" y "Archivos" indican dónde funciona la tecla.

| Tecla | Comando | Árbol | Archivos | Qué hace |
|---|---|---|---|---|
| Ctrl+A | Attributes | sí | sí | Modifica los permisos de los archivos marcados. [Permisos y fechas](attributes.md) |
| Ctrl+B | Bookmark | sí | | Guarda el directorio bajo el cursor como marcador. No utiliza las marcas. [Marcadores](bookmarks.md) |
| Ctrl+B | Batch | | sí | Escribe un script o una lista con una línea por cada archivo marcado, a partir de una máscara. [Batch](execute.md) |
| Ctrl+C | Copy | sí | sí | Copia los archivos marcados en un directorio de destino, sin recrear sus rutas. Alt+C conserva las rutas. [Copiar y mover](copy-move.md) |
| Ctrl+D | Delete | sí | sí | Elimina los archivos marcados, tras pedir confirmación. Van a la papelera si `delete_to_trash` está activo. [Eliminar](delete.md) |
| Ctrl+G | Git | sí | sí | Abre el menú Git. Solo se muestra si la opción de Git está activada. [Git](git.md) |
| Ctrl+M | Move | sí | sí | Mueve los archivos marcados a un directorio de destino. En terminales sin el protocolo de teclado de kitty, pulsa `F9` `M`. [Copiar y mover](copy-move.md) |
| Ctrl+N | New date | | sí | Modifica la fecha de modificación de los archivos marcados. [Permisos y fechas](attributes.md) |
| Ctrl+O | Open | | sí | Abre cada archivo marcado en la aplicación predeterminada del sistema. [La ventana de archivos](files.md) |
| Ctrl+P | Phrase | sí | sí | Abre la caja de búsqueda sobre los archivos marcados: lista aquellos que contienen las palabras escritas. [La caja de búsqueda](sql.md#la-caja-de-búsqueda-p) |
| Ctrl+R | Rename | sí | sí | Renombra los archivos marcados con una máscara. [Renombrar](rename.md) |
| Ctrl+S | Showall | sí | | Lista los archivos marcados de todo lo registrado. [La ventana del árbol](tree.md) |
| Ctrl+S | Search | | sí | Busca un texto en los archivos marcados y desmarca los que no lo contienen. [Buscar en archivos marcados](search.md) |
| Ctrl+T | Tag | sí | sí | Archivos: marca toda la lista. Árbol: marca todo lo registrado que coincide con el Filespec. [Marcas y operaciones en grupo](tagging.md) |
| Ctrl+U | Untag | sí | sí | Archivos: desmarca toda la lista. Árbol: desmarca todo lo registrado que coincide con el Filespec. |
| Ctrl+V | View | | sí | Abre los archivos marcados en el visor, uno tras otro; `N` y `P` cambian de archivo. [Buscar en archivos marcados](search.md) |
| Ctrl+W | Where | sí | sí | Mantiene en la lista los archivos marcados que cumplen una condición SQL. [Una condición SQL](sql.md#una-condición-sql-w) |
| Ctrl+Y | Yank | sí | sí | Copia las rutas de los archivos marcados al portapapeles. |

## Teclas de función

| Tecla | Dónde | Qué hace |
|---|---|---|
| Ctrl+F5 | Archivos | Empaqueta los archivos marcados en un archivo comprimido nuevo. [Archivos comprimidos](archives.md) |
| Ctrl+F7 | Árbol | Marca los archivos registrados de la rama bajo el cursor. |
| Ctrl+F8 | Árbol | Desmarca dichos archivos. |
| Ctrl+F9 | Árbol | Invierte las marcas de dichos archivos. |
| Ctrl+F11 | ambas | Guarda un marcador, igual que Ctrl+B en el árbol. |
| Ctrl+Shift+Del | ambas | Elimina los archivos marcados definitivamente, sin pasar por la papelera. |
| Ctrl+Z | ambas | Oops: restaura lo último enviado a la papelera. |

## Dentro de un archivo comprimido o en máquina remota

El menú omite las opciones que no pueden ejecutarse allí.

| Contexto | No listados |
|---|---|
| Archivo comprimido | Attributes, Delete, Git, Move, New date, Rename, Ctrl+F5. Copy extrae archivos. |
| Máquina remota | Batch, Git, Ctrl+F5. |
