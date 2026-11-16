# Treespec y Log

## Treespec (`\` o `/`)

Abre un prompt con la ruta actual. A medida que escribes, el cursor del árbol
va al primer directorio que empieza por lo que has escrito (sin distinguir
mayúsculas de minúsculas):

- `/` o `\` completan el nombre y bajan un nivel;
- Tab y Shift+Tab completan y pasan al directorio siguiente o anterior del mismo
  nivel;
- Backspace sube niveles;
- Ctrl+← quita la última carpeta de la ruta (`~/projects/xylem/docs/` pasa a
  ser `~/projects/xylem/`, con el cursor en `xylem`), y Ctrl+→ devuelve, una a
  una, las carpetas quitadas; escribir otra cosa empieza una ruta nueva;
- Enter confirma y Esc vuelve adonde estabas;
- ↑ abre las rutas anteriores como una [lista](execute.md#historial) (guardadas entre sesiones).

Una ruta fuera del árbol se convierte en un Log al confirmarla. En Branch y
Showall, `\` va directamente al directorio del archivo seleccionado, dejándolo
resaltado.

## Log (L)

Abre otro directorio en la ventana, reemplazando el árbol (un directorio dentro
del árbol actual simplemente mueve el cursor allí). Una dirección como
`user@host:` abre una [máquina remota](remote.md). La ruta puede ser absoluta,
relativa al directorio actual o empezar por `~`; **F2**
[la elige desde un árbol](picker.md) y ↑ lista las anteriores. Las ventanas
nuevas conservan el [orden](sort.md) y el modo de visualización actuales.

**Alt+L** convierte el directorio bajo el cursor en la raíz del árbol, al
instante: `L` hacia un directorio dentro del árbol solo mueve el cursor, y Alt+L
reduce entonces el árbol a él. En la ventana de archivos se toma el directorio
mostrado (en Branch y Showall, el del archivo seleccionado). La ventana se abre
de nuevo allí, así que lo leído y las marcas se pierden; `L ..` vuelve a abrir el
directorio superior. No funciona dentro de archivos comprimidos ni en máquinas
remotas.

El prompt se abre vacío. ↑ abre una [lista](execute.md#historial) de los lugares
usados antes, con el último abajo: los registrados con Log y las carpetas a las
que se copiaron o movieron archivos. Escribir la filtra, Enter pone un lugar en
la línea y Del lo quita de la lista.

**F11** trae un [marcador](bookmarks.md) al prompt, por su nombre, en lugar
de escribir la dirección de una máquina remota o un contenedor.

Véase también: [la ventana del árbol](tree.md), [marcadores](bookmarks.md).
