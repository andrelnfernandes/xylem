# Filtrar un CSV o una hoja de cálculo con SQL

Este procedimiento abre `customers.csv` como una tabla y conserva las filas cuyo
plan sea `team` o cuyo saldo supere 1000.

## Paso a paso

1. En la ventana de archivos, coloca el cursor sobre `customers.csv` y pulsa
   `V`. El visor muestra el archivo como una tabla, con la primera fila como
   encabezados de columna.
2. Pulsa `&`. El prompt solicita la condición.
3. Escribe `plan = 'team' OR balance > 1000` y pulsa Enter.

![Los tres pasos: V en customers.csv mostrando nueve filas, el prompt & y la condición conservando tres filas](../../screenshots/howto-table-sql.gif)

El pie muestra `Filter matched 3 rows`. Las filas conservan los números que
tienen en el archivo. Pulsar `&` con el prompt vacío vuelve a mostrar todas las
filas.

## Qué archivos se abren como tabla

| Formato | Extensiones |
|---|---|
| CSV, TSV | `.csv` `.tsv` `.tab` |
| JSON Lines | `.jsonl` `.ndjson` |
| Hojas de cálculo | `.xlsx` `.xls` `.xlsb` `.ods` |
| dBase | `.dbf` |
| SQLite | `.db` `.sqlite` `.sqlite3` |
| Parquet | `.parquet` |

El archivo debe residir en este disco local. Un archivo en máquina remota o
dentro de un archivo comprimido se abre como texto.

## Escribir la condición

| Para conservar filas donde | Condición |
|---|---|
| una columna sea igual a un texto | `plan = 'team'` |
| un número esté en un rango | `balance >= 0 AND balance < 100` |
| un texto comience con un prefijo | `name LIKE 'Ana%'` |
| un texto contenga un fragmento, sin distinguir mayúsculas | `city ILIKE '%paulo%'` |
| una celda esté vacía | `email IS NULL` |
| una columna no tenga nombre utilizable | usa su letra: `C = 'pro'` |

Una celda con apariencia numérica se trata como número: escribe `code = 7`, no
`code = '7'`. [El filtro SQL](tables.md#el-filtro-sql) describe los operadores.

## En una hoja de cálculo o una base de datos

- Las hojas de cálculo no tienen nombres de columna: las columnas son `A`, `B`,
  `C`. Las teclas `[` y `]` van a la hoja anterior y siguiente, y Tab las lista.
- En una base de datos SQLite, Tab lista las tablas y vistas. El mismo prompt
  admite además una consulta completa que comience por `SELECT` o `WITH`:

```
SELECT plan, count(*), sum(balance) FROM customers GROUP BY plan
```

Xylem únicamente lee la base de datos. Instrucciones como `UPDATE` o `DELETE`
son rechazadas.

## Guardar el resultado

Ctrl+B escribe las filas que conserva el filtro en un archivo: CSV, JSON, HTML,
un script SQL o una tabla SQLite. [Batch](execute.md) describe los formatos.

## Cuando no hace lo que esperas

- **`SQL error`.** La condición no pudo interpretarse. Se mantiene el filtro
  anteriormente activo.
- **Ninguna fila coincide con una columna visible.** Un nombre de columna
  inexistente cuenta como vacío sin generar error. Comprueba la ortografía o usa
  la letra de la columna.
- **El archivo se abre como texto.** Su extensión no figura en la tabla superior,
  o el archivo está comprimido, remoto o en un archivo empaquetado. `T` alterna
  entre tabla y texto cuando ambos existen.

Véase también: [el visor](viewer.md),
[elegir archivos con una condición SQL](howto-sql-filter.md).
