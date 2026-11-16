# Encontrar archivos: SQL y la caja de búsqueda

El [filespec](filespec.md) elige archivos por nombre, fecha y tamaño. Xylem también los encuentra por una condición escrita en SQL, sobre lo que sabe de cada archivo y sobre su contenido (**W**), y por las palabras que contienen (**P**).

## Una condición SQL (W)

**W** toma una condición escrita como el `WHERE` de una consulta SQL, y la lista pasa a mostrar solo los archivos que la cumplen, además del filespec. **Ctrl+W** busca solo en los archivos marcados. Funciona en el directorio, en Branch y en Showall, y únicamente sobre lo que está registrado: un archivo que Xylem no esté mostrando nunca es un resultado.

```sql
ext = 'jpg' AND size > 10000000
ext IN ('md', 'txt') AND age_days < 7
name LIKE 'informe%' AND depth >= 2
```

Cada archivo es una fila con estas columnas:

| Columna | Qué es |
|---|---|
| `name` | el nombre, con su extensión |
| `ext` | la extensión, en minúsculas y sin el punto (`''` cuando no tiene) |
| `path`, `dir` | la ruta completa del archivo y de su carpeta |
| `depth` | cuántas carpetas por debajo de la raíz del árbol está la carpeta del archivo |
| `size`, `disk` | el tamaño en bytes y el espacio que ocupa en disco |
| `modified` | la fecha del último cambio, hora local, como `2026-11-15 09:30:00`: `modified >= '2026-01-01'` |
| `age_days` | días transcurridos desde ese cambio (`age_days < 7`: la última semana) |
| `mode` | los permisos tal como se escriben: `mode = 644` |
| `kind` | `file`, `link` u `other` |
| `tagged` | si el archivo está marcado: `NOT tagged` |
| `git` | el estado de Git en dos letras (`' M'`, `'M '`, `'??'`, `'  '` si está limpio); `NULL` fuera de repositorio o con Git desactivado |
| `type` | qué es el archivo, como en Alt+F (`'PNG image, 800 x 600'`, `'Rust source'`, `'empty'`); se lee solo cuando la condición lo necesita |
| `dup` | número de grupo de duplicados (1, 2...), si Alt+D se ejecutó; `NULL` si no es un duplicado |

Los operadores son los del [filtro de tablas](tables.md#el-filtro-sql): `=`, `<>`, `<`, `<=`, `>`, `>=`, `LIKE`, `ILIKE`, `IN`, `IS NULL`, `AND`, `OR`, `NOT` y paréntesis. Los números se escriben enteros (`10000000`, no `10e6`). Un nombre de columna que no existe se rechaza indicando la lista de las disponibles, y una condición que ningún archivo cumple no cambia nada.

El título de la ventana de archivos muestra la condición después del filespec, y el mensaje dice cuántos archivos la cumplieron. **Ctrl+T** los marca todos, y desde ahí se pueden copiar, mover, eliminar o enviar a un [Batch](execute.md). **W** con el prompt vacío vuelve a mostrar la lista completa. La primera vez, **↑** muestra varios ejemplos para modificar.

### Dentro de los archivos: contains()

`contains('texto')` es verdadero para un archivo que contiene ese texto, sin distinguir mayúsculas de minúsculas, tal como lo encuentra la [búsqueda](search.md) de Ctrl+S (`*` dentro del texto representa cualquier cosa en la misma línea):

```sql
ext IN ('md', 'txt') AND contains('factura')
contains('TODO') AND NOT contains('DONE')
```

`contains` siempre lee los archivos cuando se ejecuta la condición, lo cual lleva tiempo: un índice guarda palabras, no fragmentos de ellas (para el índice, véase `words` abajo). Antes de leer, Xylem pregunta cuántos archivos y bytes va a leer: `No index: read the contents of 1,234 files (2.3 GB)?` (en máquina remota añade que se descargarán por SSH). Solo se leen los archivos cuya respuesta depende de su contenido: con `ext = 'md' AND contains('x')`, solo los archivos `.md`. El pie de pantalla muestra el progreso y **Esc** lo detiene: la lista muestra entonces lo encontrado hasta ese momento, y el mensaje avisa de que se detuvo.

Algunos archivos quedan fuera de la lectura, y el mensaje final cuenta cuántos fueron de cada tipo: binarios (un byte cero en los primeros 8 KiB), enlaces simbólicos (no se siguen), archivos de más de 256 MiB e ilegibles. Un archivo excluido nunca es resultado, ni siquiera de `NOT contains(...)`.

En montajes en la nube, `contains` se rechaza: habría que descargar cada archivo. La lista guarda lo encontrado al momento de leer: un archivo modificado o añadido más tarde solo vuelve a aparecer ejecutando **W** otra vez.

`contains` va solo, como una condición propia unida mediante `AND`, `OR` y `NOT` (`contains('a') = 1` se rechaza), y toma un texto entre comillas.

### Palabras enteras y un índice: words()

`words('factura marzo')` es verdadero para un archivo que contiene todas esas palabras, en cualquier orden y lugar. Una palabra es una secuencia de letras y dígitos, comparada sin mayúsculas ni acentos: `words('accion')` encuentra `Acción`. A diferencia de `contains`, no busca fragmentos de palabra: `words('factura')` no encuentra `facturas`. Un `*` inmediatamente después de una palabra coincide con cualquier palabra que empiece por ella: `words('factura*')` encuentra `factura` y `facturas`, y `words('andre*')` encuentra `André` y `Andrea`.

```sql
words('factura 2024') AND ext IN ('txt', 'md', 'csv')
words('fatur* 2024')
```

`words` puede responderse mediante un **índice** de la rama, y entonces no se lee ningún archivo. **Alt+N** en el árbol (o en la ventana de archivos, para la carpeta del archivo) indexa los archivos registrados de la rama tras pedir confirmación: los lee una vez en segundo plano, pudiendo detenerse con **Esc** conservando lo realizado. Ejecutarlo de nuevo lee únicamente los archivos que cambiaron y elimina los que desaparecieron. En una rama que ya tiene índice, Alt+N ofrece indexarla de nuevo o eliminar el índice.

|  | Sin índice | Con índice |
|---|---|---|
| Respuesta | cada archivo se lee al ejecutar W | al instante |
| Coste | tiempo en cada consulta | tiempo y espacio al indexar |
| Qué busca | fragmento de texto (`contains`) o palabras y prefijos (`words`) | palabras y prefijos (`words`) |
| Archivo modificado después | su contenido actual | leído de nuevo, como sin índice |
| Dónde | en cualquier lugar registrado | solo en las ramas indexadas con Alt+N |

Antes de leer, la confirmación indica ambos datos: `Index: 3,120 files up to date. No index: read the contents of 12 files (1.2 MiB)?`. Cuando el índice responde por todos los archivos, no hay pregunta y el mensaje dice `answered by the index`. El índice nunca genera una respuesta incorrecta: un archivo modificado después de indexarse (otro tamaño o fecha) se vuelve a leer al momento.

Lo que el índice deja fuera: archivos binarios, archivos de más de 64 MiB (W los lee cuando se solicita), enlaces simbólicos y estas carpetas y archivos configurados en `index_ignore` en `config.toml` (o en F12, página 4, opción E; por defecto: `.git`, `target`, `node_modules`, `.ssh`, `.gnupg`, `.env`, `*.key`, `*.pem`, `id_rsa`, `id_ed25519`). Quitar los patrones de secretos copia contraseñas y claves privadas al índice. Los archivos en máquinas remotas, dentro de archivos comprimidos o en montajes en la nube no se indexan.

El índice es una copia de las palabras de los archivos guardada en la caché de Xylem (`~/.cache/xylem/index`, en una carpeta que solo tú puedes leer; `%LOCALAPPDATA%\xylem\cache\index` en Windows) hasta que se elimine con Alt+N o se borre esa carpeta. Un índice ilegible (dañado o de otra versión de Xylem) se avisa en la confirmación, sus archivos se leen directamente y Alt+N lo reconstruye.

## La caja de búsqueda (P)

**P** abre una caja de búsqueda sobre la pantalla, y **Ctrl+P** la abre sobre los archivos marcados. Busca archivos por las palabras que contienen, sin necesidad de una condición SQL.

A medida que escribes palabras, los archivos que coinciden aparecen mientras escribes:
- Las palabras coinciden enteras, y la última palabra busca por prefijo mientras se escribe (`factura mar` busca `factura mar*`).
- Cuando una rama tiene índice (**Alt+N**), los resultados de los archivos indexados aparecen al instante.
- Si hay archivos sin índice, la línea de estado indica cuántos archivos y bytes faltan por indexar. Solo se leen con confirmación explícita (pulsa **Alt+R**, o **Enter** si todavía no hay resultados indexados).
- Cada resultado muestra el indicador de marcado (`*`), nombre del archivo, directorio, fecha de modificación y un fragmento limpio (~80 caracteres, sin caracteres de control ni overrides bidireccionales) de la primera línea que coincide con la búsqueda.
- Teclas en la caja de búsqueda:
  - **↑ / ↓**, **PgUp / PgDn**, **Ctrl+Home / Ctrl+End**: navegan por la lista de resultados.
  - **Enter**: salta directamente al archivo seleccionado en la ventana de archivos y cierra la caja de búsqueda.
  - **Ctrl+V**: abre el archivo en el visor directamente en la línea del fragmento.
  - **Ctrl+T**: conmuta la marca del archivo seleccionado directamente en la caja de búsqueda.
  - **Alt+R**: solicita confirmación e inicia la lectura de archivos no indexados.
  - **Esc**: cancela la lectura de archivos no indexados o cierra la caja de búsqueda.
