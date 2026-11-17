# Marcas y operaciones en grupo

Los archivos marcados (señalados con `■`) son aquellos sobre los que actúan las
operaciones en grupo: los archivos marcados **de la lista actual** (el
directorio, o la vista Branch o Showall abierta).

## Marcar

| Tecla | Árbol | Ventana de archivos |
|---|---|---|
| `T` / `U` | marcar / desmarcar los archivos del directorio | marcar / desmarcar el archivo y bajar |
| Ctrl+T / Ctrl+U | marcar / desmarcar todo lo registrado (que coincida con el Filespec) | marcar / desmarcar toda la lista |
| Ctrl+S | Showall solo de los archivos marcados | |
| `I` | invertir las marcas del directorio | invertir las marcas de la lista |
| Ctrl+F7 / F8 / F9 | marcar / desmarcar / invertir los archivos registrados de la rama | |

## Operaciones sobre los archivos marcados

| Tecla | Qué hace |
|---|---|
| Ctrl+C / Ctrl+M | [copiarlos / moverlos](copy-move.md) al destino |
| Alt+C / Alt+M | copiarlos / moverlos, **recreando sus rutas** |
| Ctrl+D | [eliminarlos](delete.md) |
| Ctrl+A | cambiar sus [permisos](attributes.md) |
| Ctrl+N | cambiar su [fecha](attributes.md) |
| Ctrl+R | [renombrarlos](rename.md) con una máscara |
| Ctrl+B | [Batch](execute.md): un script o lista con una línea por archivo |
| Ctrl+F5 | meterlos en un [archivo comprimido](archives.md) |
| Ctrl+S / Ctrl+V | [buscar texto en ellos / verlos](search.md) |
| Ctrl+O | abrirlos en el programa predeterminado |
| Ctrl+Y | copiar sus rutas al portapapeles |
| Alt+O | escribirlos en la [terminal integrada](terminal.md) |

En eJecutar, `%t` representa los archivos marcados ([eJecutar](execute.md)).

## Qué significan Ctrl y Alt

- Una letra sola actúa sobre el elemento bajo el cursor.
- Ctrl+letra actúa sobre los archivos marcados: Ctrl+C los copia, Ctrl+D los
  elimina.
- Alt+letra es la otra versión de un comando: la rama entera en el árbol, los
  archivos marcados con sus rutas en la ventana de archivos.

[Los menús](menus.md) explica la regla y cómo mostrar los menús Ctrl y Alt con
F9. [El menú Ctrl](commands-ctrl.md) y [el menú Alt](commands-alt.md) listan
cada comando.

**Cambió en la versión 0.8.** Alt ya no repite Ctrl: Alt+A, Alt+D, Alt+R,
Alt+T, Alt+U, Alt+W y Alt+Y cambiaron. [Notas de versión](releases.md) detalla el
significado anterior y el nuevo de cada tecla, y las líneas para `config.toml`
que restauran los antiguos.
