# Qué es Xylem

Xylem es un gestor de archivos para la terminal, manejado con el teclado: un
árbol de directorios arriba, los archivos del
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

[Comparaciones](comparison.md) pone a Xylem al lado de otros gestores de
archivos para la terminal.

## Archivos en los que no confías

Xylem lee cada archivo como si hubiera sido diseñado para atacarlo. Ver un
archivo no ejecuta nada de su interior, extraer un archivo comprimido no escribe
fuera del destino y cada lector se detiene ante un límite de tamaño.
[Seguridad](security.md) detalla los límites y lo que no está cubierto.
