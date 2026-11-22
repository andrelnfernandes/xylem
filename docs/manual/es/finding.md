# Encontrar archivos y carpetas

Xylem tiene varias formas de encontrar cosas. Cada una responde a una pregunta
distinta, y se dividen en tres grupos: ir a un sitio, filtrar la lista y mirar
dentro de los archivos.

| Quieres | Tecla | Qué obtienes |
|---|---|---|
| *Esa* carpeta, por trozos de su nombre | `\|` en el árbol | el cursor del árbol va allí ([Búsqueda aproximada](fuzzy.md)) |
| *Ese* archivo de la lista, por trozos de su nombre | `\|` en la ventana de archivos | el cursor de archivos va allí ([Búsqueda aproximada](fuzzy.md)) |
| Una ruta que conoces, nivel a nivel | `\` o `/` | el cursor del árbol sigue lo que escribes ([Treespec](treespec.md)) |
| El siguiente nombre que empieza por una letra | Shift+letra | el cursor va allí ([La ventana del árbol](tree.md)) |
| Los archivos con este nombre, fecha o tamaño | `F` | la lista muestra solo esos, hasta que se cambie ([Filespec](filespec.md)) |
| Los archivos que cumplen una condición | `W` | la lista muestra solo esos, hasta que se cambie ([SQL](sql.md)) |
| Los archivos que contienen estas palabras | `P` | una lista de resultados, el mejor primero ([La caja de búsqueda](sql.md#the-search-box-p)) |
| Los archivos marcados que contienen este texto | Ctrl+S | los demás se desmarcan ([Buscar en archivos marcados](search.md)) |

## Ir a un sitio

La **búsqueda aproximada** (`|`), **Treespec** (`\`, `/`) y **Shift+letra**
mueven el cursor, y no cambian nada más. Usa `|` cuando recuerdas un trozo del
nombre pero no dónde está; Treespec cuando conoces la ruta; Shift+letra para
saltar dentro de la carpeta en la que estás.

## Filtrar la lista

El **Filespec** (`F`) y **W** deciden qué archivos muestra la ventana de
archivos, y el filtro se mantiene hasta que lo cambias: el título de la ventana
lo muestra. `F` es para nombres, fechas y tamaños; `W` para todo lo que Xylem
sabe de un archivo, incluido su contenido (`contains`, `words`). Se aplican
juntos, y Ctrl+T marca después todos los archivos que pasaron.

Para buscar en todo el árbol, abre antes Branch (`B`) o Showall (`S`): los
filtros, y `|`, funcionan entonces sobre todos sus archivos registrados.

## Mirar dentro de los archivos

**La caja de búsqueda** (`P`) encuentra archivos por las palabras que
contienen, a medida que escribes, y es instantánea en una rama indexada con
Alt+N. **Ctrl+S** busca un trozo de texto en los archivos marcados y desmarca
los que no lo tienen, y Ctrl+V muestra después cada coincidencia. En `W`,
`contains('text')` y `words('a b')` hacen lo mismo como parte de una
condición.

## Marcar lo encontrado

Todas estas formas terminan en marcas cuando quieres actuar sobre muchos
archivos:

- `|` y `P`: Ctrl+T marca el resultado elegido; en `|`, Alt+T marca la lista.
- `F` y `W`: Ctrl+T en el árbol marca todos los archivos que pasaron.
- Ctrl+S deja marcados solo los archivos que tienen el texto.

Después Ctrl+C, Ctrl+M, Ctrl+D y Batch actúan sobre los archivos marcados.
Consulta [Marcas](tagging.md).
