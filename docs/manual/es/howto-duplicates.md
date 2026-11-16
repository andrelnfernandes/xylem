# Encontrar duplicados y conservar una copia

Este procedimiento encuentra los archivos con el mismo contenido bajo un
directorio y envía a la papelera todas las copias excepto una de cada uno.

## Paso a paso

1. En el árbol, coloca el cursor sobre el directorio y pulsa `*` para registrar
   su rama.
2. Pulsa Alt+D. Xylem compara los archivos y lista los grupos de archivos
   iguales, ordenando primero el grupo que libera más espacio.
3. Pulsa `A`. En cada grupo, todas las copias excepto una quedan marcadas
   (`T`).
4. Pulsa `D`. Xylem pide confirmación antes de eliminar. Pulsa `T` para enviar
   los archivos a la papelera y después `Y`.

![Los cuatro pasos: registrar la rama, Alt+D listando tres grupos, A marcando todas las copias menos una y D, T, Y enviando cuatro archivos a la papelera](../../screenshots/howto-duplicates.gif)

El pie muestra `Trashed 4 of 4 files`.

## Elegir qué copia se conserva

La tecla `A` marca todas las copias menos una en cada grupo. Antes del paso 4
puedes cambiar cuál permanece:

| Tecla | Efecto |
|---|---|
| `A` otra vez | cambia el criterio de la copia conservada: la más antigua, la más nueva o la de ruta más corta |
| `K` | conserva la copia bajo el cursor y marca las demás de su grupo |
| `F` | en cada grupo que tenga una copia en la carpeta de esta copia, conserva esa |
| `T` o Espacio | marca o desmarca una copia individual |
| `U` | desmarca todo |

Xylem no permite marcar la última copia de un grupo.

## Carpetas enteras

La tecla `G` cambia la lista a carpetas cuyos contenidos son completamente
iguales. La línea inferior indica cuándo las hay: `1 group of folders (G)`.

## Qué significa la pregunta

Con `delete_to_trash` desactivado (la opción predeterminada), el paso 4 pregunta
inicialmente `DELETE 4 tagged files? This cannot be undone.`. Pulsar `T` cambia
la pregunta a `MOVE 4 tagged files to trash?`. Pulsar `Y` confirma lo que dice
la pregunta.

## Cuando no hace lo que esperas

- **Un archivo repetido no aparece en la lista.** Archivos menores de 1 KiB,
  archivos vacíos, enlaces y las carpetas `.git` y `node_modules` se omiten por
  defecto. [Duplicados](duplicates.md) explica cómo cambiar esto.
- **Un grupo no fue eliminado.** Una copia cambió tras la comparación. Ningún
  archivo de ese grupo se toca y el pie lo notifica. Pulsa F3 para comparar de
  nuevo.

Véase también: [duplicados](duplicates.md), [eliminar](delete.md),
[encontrar lo que llena el disco](howto-disk-space.md).
