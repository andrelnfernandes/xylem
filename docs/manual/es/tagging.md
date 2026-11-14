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

Como en ZTreeWin:

- **Una letra sola** actúa sobre el elemento bajo el cursor.
- **Ctrl+letra** actúa sobre los archivos marcados: Ctrl+C los copia, Ctrl+D
  los borra, Ctrl+T marca todo.
- **Alt+letra** es la otra versión de un comando: en el árbol, la rama entera
  (Alt+P Prune, Alt+R Release, Alt+U uso de disco, Alt+D Duplicados, Alt+C
  Compare); en la ventana de archivos, los marcados con sus rutas (Alt+C,
  Alt+M); y las herramientas (Alt+S Sort, Alt+F File display, Alt+B
  marcadores, Alt+H ayuda, Alt+I Info, Alt+X el terminal).

**Cambió en la versión 0.8.** Alt ya no repite Ctrl, y algunas teclas
cambiaron:

| Tecla | Hasta 0.7 | Desde 0.8 |
|---|---|---|
| Alt+A | permisos de los marcados | nada (Ctrl+A) |
| Alt+D | borrar los marcados | Duplicados (borrar: Ctrl+D) |
| Alt+R | renombrar los marcados | Release de la rama, como `-` (renombrar: Ctrl+R) |
| Alt+T | marcar todo | nada (Ctrl+T) |
| Alt+U | desmarcar todo | uso de disco (desmarcar: Ctrl+U) |
| Alt+W | uso de disco | nada (Alt+U) |
| Alt+Y | Duplicados | nada (Alt+D) |

Las teclas antiguas pueden volver en `[keys.any]` de la
[configuración](configuration.md), una o todas. Todas, como hasta la 0.7:

```toml
[keys.any]
"alt+a" = "attributes_tagged"
"alt+d" = "delete_tagged"
"alt+r" = "rename_tagged"
"alt+t" = "tag_all"
"alt+u" = "untag_all"
"alt+w" = "usage"
"alt+y" = "duplicates"
```

Valen en las dos ventanas, también Alt+R en el árbol; Release queda en `-`.
**K** en la ayuda (F1) muestra las teclas vigentes.

## Menús Ctrl y Alt (F9)

El pie de página tiene tres líneas en una pantalla común: los comandos de letra
en orden alfabético en las dos primeras, las teclas de función y los símbolos
en la tercera. En una pantalla lo bastante ancha para todos los menús, tiene
dos: las letras en una, las teclas en la otra. Todo
comando está en uno de los menús (DIR o FILE, CTRL, ALT); dentro de un archivo
comprimido o en una máquina remota, dejan fuera lo que allí no funciona.

**F9**
cambia al menú Ctrl, luego al menú Alt y después vuelve al normal. Con uno de
estos menús abierto, la siguiente letra cuenta como Ctrl+letra o Alt+letra
(`F9` `M` = Ctrl+M); `Esc` cancela. En terminales con el protocolo de teclado de
kitty, basta con **mantener pulsado Ctrl o Alt** para mostrar el menú
correspondiente.

Al soltar la tecla vuelve el menú habitual, como en ZTree.

El terminal tiene que informar de Ctrl y Alt pulsados y soltados por sí solos, y
solo el protocolo de teclado de kitty lo hace: kitty, Ghostty, WezTerm, foot
(Wayland) y el Alacritty reciente lo admiten, y en Ubuntu o Debian la mayoría
está a un paquete de distancia (`sudo apt install kitty`, o `ghostty`,
`alacritty`, `foot`). Los terminales basados en VTE (GNOME Terminal, Ptyxis,
Terminator, Tilix) aún no lo admiten, así que en ellos los menús siguen en F9.

**Ctrl+M:** en la mayoría de las terminales, Ctrl+M llega igual que Enter. Xylem
activa el protocolo de teclado de kitty cuando la terminal lo admite (kitty,
WezTerm, foot, Ghostty, Alacritty reciente, iTerm2), y entonces Ctrl+M mueve los
archivos marcados como en ZTree. En terminales basadas en VTE (GNOME Terminal,
Ptyxis, Terminator, Tilix) usa **F9 M** (o Alt+M y F6 en la entrada).
