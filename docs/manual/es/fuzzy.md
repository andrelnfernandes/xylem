# Búsqueda aproximada (|)

`|` encuentra una carpeta, o un archivo de la lista, por trozos de su nombre,
en cualquier nivel del árbol: `cfgxy` encuentra `config/xylem`, `rdme`
encuentra `README.md`. Los resultados aparecen sobre el prompt a medida que
escribes, el mejor primero, y el cursor de la ventana sigue al elegido.

Para ir a una ruta que ya conoces, nivel a nivel, [Treespec](treespec.md) es
más rápido. Para elegir muchos archivos por nombre, fecha o tamaño, o por lo
que contienen, consulta [Encontrar archivos y carpetas](finding.md).

## Qué busca

Lo que busca `|` depende de la ventana en que se abre:

- **En el árbol:** las carpetas registradas, por su ruta desde la raíz del
  árbol (`docs plan` encuentra `docs/plans`). Las carpetas ocultas, y lo que
  hay dentro de ellas, solo se buscan mientras el árbol las muestra.
- **En la ventana de archivos:** los archivos de la lista tal como está, con el
  [filespec](filespec.md) y la [condición SQL](sql.md) (`W`) en vigor. En el
  directorio, por el nombre; en Branch y Showall, también por la ruta
  (`src fzrs` encuentra `src/fuzzy.rs`).

Para encontrar un archivo en cualquier parte del árbol, abre Showall (`S`) y
después `|`.

Solo se busca lo registrado, y no se lee nada del disco ni de la red: funciona
igual en una [máquina remota](remote.md), dentro de un
[archivo comprimido](archives.md) y en un [montaje en la nube](cloud.md).

## Cómo coincide

- **Las letras en orden**, no necesariamente juntas: `rdme` encuentra
  `README.md`.
- **Los espacios separan términos**, y cada término tiene que coincidir, en
  cualquier orden: `xy docs` encuentra `xylem/docs` y `docs/xylem`.
- **Las mayúsculas** no cuentan, salvo que el término tenga una letra
  mayúscula: `Rd` no encuentra `readme`. Cada término decide por sí mismo.
- **El mejor primero:** las letras juntas cuentan más que las separadas; una
  letra al principio de una palabra (al comienzo, después de `/`, `_`, `-`,
  `.` o un espacio, o donde las minúsculas pasan a mayúsculas, como en
  `fileSpec`) cuenta más que una en medio; una coincidencia en el nombre
  cuenta más que una en las carpetas de encima. Entre iguales, la ruta más
  corta va primero, y después el orden del árbol o de la lista.

Las letras que coincidieron se muestran en el color de las teclas. Un nombre
demasiado largo para la ventana se corta por la izquierda, con `…`, para que
su final quede a la vista.

## En la pantalla

El contador de arriba a la derecha dice cómo va la búsqueda:

| Dice | Significado |
|---|---|
| `type pieces of a name` | el prompt está vacío |
| `120 found · searching 40%` | la búsqueda todavía recorre el árbol; la lista se va llenando |
| `120` | terminada: todas las coincidencias están en la lista |
| `500 best of 15,894` | terminada: la lista guarda las 500 mejores |
| `nothing matches` | terminada, sin ninguna coincidencia |

Un árbol grande se busca por pasos mientras escribes, así que los primeros
resultados llegan al instante y el resto los sigue. Cuando añades letras al
final, la búsqueda mira solo entre lo que ya coincidía. Si el árbol cambia
mientras el prompt está abierto (una carpeta registrada, liberada o leída de
nuevo), la búsqueda vuelve a empezar.

Un `*` delante de un archivo significa que está marcado; detrás de una
carpeta, `3*` dice cuántos de sus archivos están marcados.

## Teclas

| Tecla | Qué hace |
|---|---|
| ↑ ↓ | elige el resultado anterior o el siguiente; el cursor de la ventana lo sigue |
| PgUp PgDn, Home End | elige una página arriba o abajo, el primer o el último resultado |
| Enter | se queda donde está el cursor y cierra el prompt |
| Esc | vuelve adonde estaba el cursor, y cierra el prompt |
| Ctrl+T | marca el resultado elegido, o lo desmarca si está marcado, y elige el siguiente |
| Alt+T | marca los archivos de los resultados de la lista |
| Ctrl+U | borra el prompt |

Con el [ratón](mouse.md) activado, un clic elige un resultado y un doble clic
va allí.

## Marcar desde los resultados

**Ctrl+T** sobre un archivo lo marca; sobre una carpeta, marca los archivos de
esa carpeta que mostraría la ventana de archivos (con el filespec y `W` en
vigor). Si ya están todos marcados, los desmarca. La elección pasa entonces al
resultado siguiente, de modo que Ctrl+T, Ctrl+T, Ctrl+T marca los resultados
uno tras otro.

**Alt+T** marca los archivos de todos los resultados de la lista: como mucho
los 500 que se muestran. Cuando coincidieron más, la nota bajo la lista dice
cuántos quedaron fuera. Alt+T sobre una carpeta que no se ha registrado no
marca nada allí, y la nota dice cuántas se omitieron.

Las marcas se mantienen después de cerrar el prompt, como cualquier marca:
Ctrl+C, Ctrl+M, Ctrl+D y Batch actúan entonces sobre ellas. Consulta
[Marcas](tagging.md).

Marcar todas las coincidencias, no solo las 500 de la lista, es una opción,
desactivada por defecto: Alt+T podría entonces marcar miles de archivos que no
has visto. Lee
[Marcar todas las coincidencias de la búsqueda aproximada](configuration.md#tagging-every-match-of-the-fuzzy-find)
antes de activarla.
