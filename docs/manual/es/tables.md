# Tablas: hojas de cálculo, CSV y bases de datos

Algunos archivos se abren como una cuadrícula en vez de texto. Xylem decide por la extensión, y solo para archivos sin comprimir en este disco: un archivo de una máquina remota o dentro de un archivo comprimido se muestra como contenido normal, con la nota `Tables are shown only for files on this disk`.

| Formato | Extensiones | Nombres de las columnas |
|---|---|---|
| CSV, TSV | `.csv` `.tsv` `.tab` | la primera fila, cuando al menos la mitad no son números; si no, A, B, C... |
| JSON Lines | `.jsonl` `.ndjson` | las claves de los objetos |
| Hojas de cálculo | `.xlsx` `.xls` `.xlsb` `.ods` | A, B, C... (la primera fila es una fila más) |
| dBase, Clipper, FoxPro | `.dbf`, con su archivo memo `.dbt` | los nombres de los campos |
| SQLite | `.db` `.sqlite` `.sqlite3` | las columnas de la tabla |
| Parquet | `.parquet` | las columnas del archivo |

El separador del CSV (coma, punto y coma, tabulación o barra vertical) se detecta por el principio del archivo.

| Tecla | Qué hace |
|---|---|
| ↑ ↓ | fila anterior o siguiente |
| ← → | columna anterior o siguiente |
| PgUp PgDn | una página de filas hacia arriba o hacia abajo |
| Home End | primera o última columna |
| Ctrl+Home Ctrl+End | primera o última fila; con un filtro, de las filas que conserva |
| `[` `]` | hoja anterior o siguiente de una hoja de cálculo, o tabla de una base SQLite, dando la vuelta; esto borra el filtro |
| `Tab` | la lista de hojas o tablas: escribe parte de un nombre para reducirla, Enter abre una y Esc vuelve; abrir otra borra el filtro. En una base, las vistas van después de las tablas, marcadas con `(view)` |
| `g` o `:` | saltar a una fila por su número en el archivo; si el filtro la ocultó, a la siguiente que dejó |
| `\` o `&` | el filtro SQL |
| `m` o Insert | marca o desmarca el registro bajo el cursor y pasa al siguiente; **Ctrl+B** escribe entonces solo los marcados |
| `M` | borra las marcas |
| `i` | la estructura de la tabla: lo que el archivo dice de cada columna (ver abajo) |
| Enter | el registro bajo el cursor como una ficha, un campo por línea: ↑ ↓ eligen el campo, ← → van al registro anterior o siguiente, `c` copia el registro entero (`NOMBRE : valor`, una línea por campo) e `y` el valor del campo, para pegarlo en un editor o un terminal; Esc vuelve. Lo copiado es lo que muestra la pantalla: los caracteres de control y de dirección del texto, saltos de línea incluidos, son `·`, así que pegarlo en un terminal no ejecuta nada. Fuera de la tabla, Enter cierra el visor |
| `T` | ir al texto del archivo y volver |
| `H` | de la tabla al texto, luego hexadecimal, luego de vuelta a la tabla |

El marco muestra `TABLE [2/5: nombre]` cuando hay más de una hoja o tabla, y la línea inferior dónde está el cursor: `Cell B7 (Row 7 of 120)`. Con filtro, las filas conservan el número que tienen en el archivo, y esta línea añade qué coincidencia es: `Cell A13 (Row 13 of 31, match 2 of 8)`. La última línea de la cuadrícula trae el contenido completo de la celda y el filtro en uso. La búsqueda (`F`, `B`, `/`) no mira dentro de la cuadrícula y no encuentra nada ahí; funciona en modo texto.

Las marcas se mantienen bajo un filtro, y Batch escribe lo marcado que el filtro conserva. Pertenecen a la hoja o tabla abierta y se pierden al cambiarla. Si el archivo cambia después de marcar, Batch se niega, pues puede haber otros registros en esas posiciones.

## La estructura

`i` muestra, sobre la cuadrícula, las columnas en orden con lo que el archivo dice de cada una, como `DISPLAY STRUCTURE` en dBase y Clipper. Las flechas, PgUp, PgDn, Home y End la desplazan; Esc, Enter o `i` la cierran.

| Formato | Qué muestra |
|---|---|
| dBase, Clipper, FoxPro | nombre, tipo (`C` carácter, `N` numérico, `D` fecha, `L` lógico, `M` memo, `F` coma flotante, `I` entero, `B` double, `Y` moneda, `T` fecha y hora), ancho y decimales; encima, el número de registros y la fecha de la última actualización escritos en la cabecera, y la longitud de un registro |
| SQLite | el tipo declarado de cada columna, `Not null`, el valor por defecto y la posición en la clave primaria; encima, si es una tabla o una vista, y sus índices con sus columnas (como máximo 32). Una vista muestra las columnas que da; una consulta completa, las de su resultado, con el tipo de cada una cuando viene directamente de una columna de tabla |
| Parquet | el tipo Arrow de cada columna y si admite nulos; encima, las filas y los grupos de filas que declara el archivo y el programa que lo escribió |
| CSV, JSON Lines, hojas de cálculo | no declaran tipos: lo que tienen las filas leídas en cada columna (entero, decimal, lógico, texto, vacío) y el valor más ancho |

Nombres, tipos y valores por defecto vienen del archivo y se limpian como las celdas: los caracteres de control y de dirección del texto se ven como `·`. Un `.dbf` cuya cabecera se contradice muestra `?` en los decimales.

## El filtro SQL

`\` o `&` pide una condición, como el `WHERE` de SQL (la palabra `WHERE` es opcional), y muestra solo las filas que la cumplen. El prompt empieza con el filtro actual; déjalo vacío para volver a ver todas las filas. Una nota dice `Filter matched N rows` o `SQL error: ...`; tras un error, el filtro anterior sigue activo.

```
age >= 30 AND name LIKE 'Ali%'
WHERE status IN ('open', 'late') OR total > 1000.50
NOT (city = 'Rio' OR city IS NULL)
B ILIKE '%silva%'
```

| Operador | Qué hace |
|---|---|
| `=` `!=` `<>` `<` `<=` `>` `>=` | comparan |
| `AND` `OR` `NOT` y paréntesis | combinan condiciones (`AND` va antes que `OR`) |
| `LIKE` `ILIKE` | coinciden con un patrón, `%` para cualquier texto y `_` para un carácter; `ILIKE` no distingue mayúsculas |
| `IN (...)` `NOT IN (...)` | está, o no está, en la lista |
| `IS NULL` `IS NOT NULL` | la celda está, o no está, vacía |

Las palabras clave pueden ir en mayúsculas o minúsculas.

- **Columnas:** el nombre de la columna, en mayúsculas o minúsculas y sin comillas, o su letra, de `A` a `Z`. La letra sirve para llegar a un nombre con espacios o símbolos. Una columna que no existe cuenta como vacía, sin error.
- **Valores:** texto entre comillas simples o dobles (`\'` para una comilla dentro de él), números, `TRUE`, `FALSE` y `NULL`. Lo que va entre comillas es siempre texto, nunca una columna.
- **Celdas:** el tipo viene del aspecto de la celda. Vacía es `NULL`, `true` y `false` son booleanos, los números son números y el resto es texto. Por eso escribe `code = 7`, no `code = '7'`: un texto y un número nunca son iguales, y una celda con `007` es el número 7. Las comparaciones de texto distinguen mayúsculas de minúsculas.
- **Tamaño:** un filtro tiene como máximo 1024 bytes y 128 términos.

## Una consulta completa en una base

En una base SQLite, el mismo prompt acepta una consulta completa, que empiece por `SELECT` o `WITH`, con todo lo que SQLite puede hacer: `JOIN`, `GROUP BY`, `ORDER BY`, funciones.

```
SELECT c.name, sum(o.price) FROM customers c JOIN orders o ON o.code = c.code GROUP BY c.name
```

El marco dice `QUERY:`, y la cuadrícula muestra el resultado. Una condición filtra entonces esas filas, y **Ctrl+B** escribe todo lo que da la consulta. Dejar el prompt vacío borra el filtro y luego vuelve a la tabla; `Tab`, `[` y `]` también van a una tabla. La consulta solo lee, una cada vez: cualquier otra cosa se rechaza, y un error deja la pantalla como estaba. Los límites de abajo se aplican, así que una consulta lenta se detiene a los 3 s y muestra las filas obtenidas; un Batch no tiene ese límite. Xylem no modifica bases: `UPDATE`, `DELETE` y similares reciben un mensaje que lo dice. Para modificar una, usa otro programa: `O` en la ventana de archivos la abre en el definido en `opener`, y `'*.db' = 'sqlite3 %f'` en `[editors]` hace que `E` la abra en `sqlite3`.

## Tablas grandes

Con **V**, estas tablas no tienen límite de filas: un archivo CSV o TSV, un archivo JSON Lines, un archivo dBase (`.dbf`) y una tabla o vista de una base SQLite, o una consulta escrita en ella. La cuadrícula guarda las filas cercanas al cursor, como máximo 2000 y 32 MiB de texto, y lee las demás del archivo a medida que se mueve el cursor, sin bloquear la pantalla: una fila aún no leída aparece como `…` por un momento.

- **El total** se muestra mientras se cuenta: `Row 1 of 2,000+ (counting…)`, y luego `Row 1 of 8,412,903`. **Ctrl+End** va a la última fila y `g` a cualquiera; antes de que termine el recuento, `g` más allá avisa de que el recuento aún no ha llegado.
- **El filtro** (`&`) recorre toda la tabla, en segundo plano. La línea bajo la cuadrícula dice hasta dónde ha llegado, `1,203 matches so far, row 2,300,000 of 8,412,903`, y las filas aparecen a medida que se encuentran. Conserva como máximo 1 000 000 de coincidencias, y entonces dice `first 1,000,000 matches`.
- **Esc** detiene el recuento o el filtro, conservando lo encontrado; el siguiente Esc cierra el visor.
- El ancho de las columnas sale de las primeras filas, y una columna se ensancha cuando aparece en pantalla una fila más ancha. Un archivo JSON Lines también puede ganar columnas durante el recuento, cuando una clave aparece por primera vez más adelante.
- **CSV, TSV y JSON Lines** se leen una vez de principio a fin durante el recuento, y Xylem anota dónde empieza cada fila 256.ª, de modo que a cualquier fila se llega leyendo como mucho otras 255. Esa anotación se guarda solo en memoria, unos 8 bytes por cada 256 filas, y desaparece al cerrar el visor. Un registro CSV con un salto de línea entre comillas sigue siendo una fila. Un registro CSV de más de 16 MiB detiene la lectura, y una línea de JSON Lines de más de 512 KiB se omite.
- **Los registros dBase** tienen todos el mismo tamaño, así que Xylem va directamente a cualquiera. El recuento solo mira si cada registro está eliminado, porque un registro eliminado no es una fila: Xylem anota dónde está cada fila 256.ª, igual que antes.
- **Cuando el archivo cambia** mientras está abierto (crece, se reduce o se reemplaza), lo que Xylem anotó de él deja de valer: no se lee nada más, y el encabezado dice `(the file changed: R reloads)`. **R** lo vuelve a leer.
- **El texto y el hexadecimal** de un archivo que se abre como tabla solo se leen cuando vas a ellos (**T**, **H**): hasta entonces Xylem lee solo el principio del archivo, así que una tabla grande se abre al instante.

Autoview (F7) no abre bases de datos, y lee solo las primeras filas de las demás tablas. Los demás formatos mantienen por ahora el límite de abajo.

## Límites

Salvo las tablas grandes de arriba, una tabla guarda como máximo 50 000 filas y 256 columnas, y tarda como máximo lo que permiten los demás límites del visor: 3 s (100 ms en Autoview), y una hoja de cálculo que se desempaqueta a más del límite de tamaño (1 MiB en Autoview) no se abre como tabla. Lo que pasa de ahí no aparece y el filtro no lo ve; la cabecera lo avisa: `(showing first 50,000 rows)` o `(stopped by the time limit: showing N rows)`. Mientras se lee la tabla, la pantalla queda libre y una nota lo avisa; Esc detiene la lectura y muestra lo que ya llegó, `(stopped by Esc: showing N rows)`, y el siguiente Esc cierra el visor. Una celda muestra como máximo 2048 caracteres y luego `…`. Los caracteres de control y de dirección del texto aparecen como `·`.

Cuando una hoja de cálculo (XLSX, XLSB, XLS, ODS) tiene más filas que eso, la cabecera y la nota lo dicen: `(the sheet has more than 50,000 rows, the most Xylem shows: open it in Excel or another spreadsheet program to see them all (O))`, es decir, ábrela en Excel o en otro programa de hojas de cálculo para verlas todas. **O** la abre en el programa que tu sistema usa para ella.

## Tablas en bibliotecas C

Las bases SQLite (`.db`, `.sqlite`, `.sqlite3`) y los archivos Parquet se leen con bibliotecas escritas en C, que no tienen la seguridad de memoria de Rust. Por eso solo se abren como tabla con `V`. Autoview lee todo aquello por donde pasa el cursor y muestra estos archivos como contenido normal, con la nota `Parquet or SQLite: V opens the table`. Abre una base de origen no confiable solo si aceptas este riesgo. Los archivos `.ods` y `.xls` también se abren como tabla solo con `V`: su lector monta primero la hoja entera, y esa parte no se puede detener.

Consulta también: [el visor](viewer.md).
