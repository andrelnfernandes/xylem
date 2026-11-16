# Qué es Xylem

Xylem es un gestor de archivos para la terminal. Su pantalla y sus teclas siguen
a XTreeGold y ZTreeWin: un árbol de directorios arriba, los archivos del
directorio bajo el cursor abajo, estadísticas a la derecha y un menú de comandos
de una letra en el pie de página. El comando es `xy`.

## Registro de directorios (Log)

Xylem lee un directorio en la memoria cuando lo solicitas y lo mantiene allí.
Esto se denomina registrar (*logging*). `→` registra un directorio, `*` registra
toda la rama bajo el cursor y `-` libera la rama de la memoria.

Dos comandos listan archivos registrados de más de un directorio a la vez: `B`
(Branch) lista los archivos del directorio y de todo lo que está debajo de él, y
`S` (Showall) lista todos los archivos registrados. En estas listas puedes
marcar archivos con `T`, y los comandos con Ctrl actúan sobre los archivos
marcados: Ctrl+C los copia, Ctrl+D los elimina, Ctrl+R los renombra con una
máscara.

Los archivos de un directorio que no ha sido registrado no aparecen en ninguna
lista ni se ven afectados por ningún comando.

## Qué abarcan los comandos

| Para hacer esto | Usa | Tema |
|---|---|---|
| Copiar o mover, con o sin las rutas de origen | `C`, `M`, Ctrl+C, Alt+C | [Copiar y mover](copy-move.md) |
| Eliminar, y restaurar lo enviado a la papelera | `D`, Ctrl+Z | [Eliminar](delete.md) |
| Renombrar un archivo, o varios con una máscara | `R`, Ctrl+R | [Renombrar](rename.md) |
| Mostrar solo ciertos archivos, por nombre, fecha o tamaño | `F` | [Filespec](filespec.md) |
| Comparar dos directorios | `C` en el árbol | [Comparar](compare.md) |
| Ver un archivo como texto, hexadecimal o tabla | `V` | [El visor](viewer.md) |
| Abrir ZIP, 7z y TAR como directorios | Alt+F5 | [Archivos comprimidos](archives.md) |

[Los menús](menus.md) explica cómo se organizan las teclas y
[los comandos](commands.md) lista todos ellos.

## Lo que XTreeGold y ZTreeWin no tienen

- Una terminal dentro de la pantalla (Alt+X). El árbol sigue al comando `cd` de
  la shell, y Alt+O escribe los archivos marcados en la línea de comandos.
  Vea [la terminal integrada](terminal.md).
- Otras máquinas mediante SSH y contenedores a través de `docker exec`, mostrados
  como un árbol. Copy, Move, Rename, Delete, Attributes y New date funcionan
  allí; Edit y Batch no. Vea [máquinas remotas](remote.md).
- Google Drive, S3 y demás almacenamientos soportados por rclone.
  Vea [almacenamiento en la nube](cloud.md).
- Una condición SQL sobre la lista de archivos (`W`):
  `ext = 'jpg' AND size > 10000000` conserva los archivos que la cumplen.
  Vea [Filespec](filespec.md#una-condicion-sql-w).
- SQL sobre el contenido de un archivo: el visor muestra hojas de cálculo, CSV,
  JSON Lines, DBF, SQLite y Parquet como tablas y las filtra con una condición
  `WHERE`. Vea [el visor](viewer.md).
- Duplicados (Alt+D): archivos y carpetas con el mismo contenido en una rama.
  En una carpeta montada por rclone, compara los hashes que el servicio ya
  almacena y no descarga nada. Vea [duplicados](duplicates.md).
- Uso de disco (Alt+U): las carpetas y archivos de una rama ordenados por el
  espacio que ocupan. Vea [uso de disco](usage.md).
- Un menú Git (Ctrl+G), desactivado por defecto. Vea [Git](git.md).

[Comparación con ZTreeWin](ztreewin.md) enumera los comandos de ZTreeWin que
Xylem no incluye y las teclas que difieren.

## Archivos en los que no confías

Xylem lee cada archivo como si hubiera sido diseñado para atacarlo. Ver un
archivo no ejecuta nada de su interior, extraer un archivo comprimido no escribe
fuera del destino y cada lector se detiene ante un límite de tamaño.
[Seguridad](security.md) detalla los límites y lo que no está cubierto.
