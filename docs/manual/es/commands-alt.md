# El menú Alt

Alt con una letra es la otra versión de un comando. En el árbol abarca toda la
rama bajo el cursor. En la ventana de archivos, Alt+C y Alt+M actúan sobre los
archivos marcados conservando sus rutas. Las demás opciones abren herramientas
que no operan sobre un único archivo.

Para mostrar este menú, mantén pulsado Alt o pulsa F9 dos veces. Consulta
[los menús](menus.md).

## Comandos

En el orden del menú. "Árbol" y "Archivos" indican dónde funciona la tecla.

| Tecla | Comando | Árbol | Archivos | Qué hace |
|---|---|---|---|---|
| Alt+B | Bookmarks | sí | sí | Abre el gestor de marcadores: saltar a un directorio guardado, asignarle nombre o eliminarlo. [Marcadores](bookmarks.md) |
| Alt+C | Compare | sí | | Compara toda la rama con la de la otra ventana por ruta relativa y abre un Branch con los archivos marcados. [Comparar](compare.md) |
| Alt+C | Copy | | sí | Copia los archivos marcados recreando la ruta de cada uno bajo el destino. F7 en el prompt define dónde inicia la ruta. [Copiar y mover](copy-move.md) |
| Alt+D | Duplicates | sí | sí | Localiza archivos y carpetas con el mismo contenido en la rama. No elimina nada sin confirmación. [Duplicados](duplicates.md) |
| Alt+F | File display | sí | sí | Cambia el modo de visualización de la ventana de archivos: nombres en columnas, tamaños, detalles, propietario, tipo o nombres largos. [La ventana de archivos](files.md) |
| Alt+H | Help | sí | sí | Abre la ayuda contextual, igual que F1. |
| Alt+I | Info | sí | sí | Muestra tipo, tamaño, permisos, propietario y fechas del archivo o directorio. [Información](info.md) |
| Alt+K | Kbyte | sí | | Muestra el tamaño, número de archivos o porcentaje junto a cada directorio del árbol. [Información](info.md) |
| Alt+L | Log | sí | sí | Convierte el directorio bajo el cursor en la raíz del árbol. Lo registrado y las marcas se descartan. [Treespec y Log](treespec.md) |
| Alt+M | Move | | sí | Mueve los archivos marcados recreando la ruta de cada uno bajo el destino. [Copiar y mover](copy-move.md) |
| Alt+P | Prune | sí | | Elimina el directorio junto con todo su contenido, definitivamente, tras confirmar escribiendo la palabra solicitada. [Eliminar](delete.md) |
| Alt+Q | Quit here | sí | sí | Sale del programa y deja la shell en el directorio actual. Requiere la función de shell descrita en [Salir y cambiar el directorio de la shell](quit.md). |
| Alt+R | Release | sí | | Libera la rama de la memoria, como hace `-`. Las marcas se pierden. |
| Alt+S | Sort | sí | sí | Ordena la ventana de archivos por nombre, extensión, fecha o tamaño. [Ordenar](sort.md) |
| Alt+U | Usage | sí | sí | Lista las carpetas y archivos de la rama según el espacio que ocupan, de mayor a menor. [Uso de disco](usage.md) |
| Alt+N | iNdex | sí | sí | Indexa las palabras de los archivos registrados de la rama para que `words()` en una condición SQL responda al instante. [El índice](sql.md#palabras-enteras-y-un-índice-words) |

## Otras teclas

| Tecla | Dónde | Qué hace |
|---|---|---|
| Alt+X | ambas | Abre la terminal integrada y regresa de ella. [La terminal integrada](terminal.md) |
| Alt+Z | ambas | Abre la terminal a pantalla completa. |
| Alt+O | ambas | Escribe los archivos marcados en la línea de comandos de la terminal, protegidos por comillas. |
| Alt+F5 | Archivos | Abre el archivo comprimido bajo el cursor como si fuera un árbol. [Archivos comprimidos](archives.md) |
| Alt+F10 | ambas | Abre la configuración, igual que F12. En GNOME, Alt+F10 maximiza la ventana y no llega a Xylem. [Configuración](configuration.md) |
| Alt+Shift+Del | ambas | Elimina los archivos marcados definitivamente, igual que Ctrl+Shift+Del. |

## Dentro de un archivo comprimido o en máquina remota

| Contexto | No listados |
|---|---|
| Archivo comprimido | Log, Move, Prune, las teclas de terminal. Alt+C extrae los archivos marcados con sus rutas dentro del archivo comprimido. |
| Máquina remota | Log (Alt+L), Alt+O. Alt+X abre una shell en esa máquina remota. |

## Cambios en la versión 0.8

Hasta la versión 0.7, Alt+A, Alt+D, Alt+R, Alt+T y Alt+U realizaban la misma
acción que Ctrl con dicha letra, Alt+W era Usage y Alt+Y era Duplicates.
[Notas de versión](releases.md) incluye la tabla y las líneas para `config.toml`
que restauran las teclas anteriores.

Véase también: [los menús DIR y FILE](commands.md),
[el menú Ctrl](commands-ctrl.md).
