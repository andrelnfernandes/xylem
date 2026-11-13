# Uso de disco (Alt+U)

Adónde va el espacio. **Alt+U** registra toda la rama del directorio actual y
lista sus carpetas y archivos según el espacio que ocupan, de mayor a menor, con
una barra y su proporción dentro del directorio. Enter entra en una carpeta;
carpeta a carpeta, llegas hasta lo que llena el disco.


> Hasta la versión 0.7, el uso de disco era Alt+W, y Alt+U desmarcaba todo
> (ahora solo Ctrl+U). Vea [Ctrl y Alt](tagging.md#qué-significan-ctrl-y-alt).

```
       32.9 GiB ████████████████   45%   wsl-ubuntu-backup.tar
       18.9 GiB █████████▏         26%  /projetos
 .      5.5 GiB ██▋               7.6%  /.local
 >    154.6 MiB                      -  /boot
```

- **La barra** se dibuja respecto al elemento más grande del directorio, para
  que se aprecien las diferencias entre elementos sobre una pista del ancho del
  mayor; **el porcentaje** es la parte que representa del directorio. El color
  indica cuánto supone esa parte: hasta el 10% (verde, en la mayoría de los
  temas), hasta el 40% (ámbar) o más (rojo). Cada tema tiene los suyos
  (`bar_low`, `bar_mid` y `bar_high` en `[colors]`).
- **Las carpetas y los archivos se mezclan**, por tamaño: un único archivo
  grande (una copia de seguridad antigua, una imagen ISO) aparece entre las
  carpetas. Las carpetas empiezan por `/`.
- **La línea inferior** muestra los totales del directorio: en disco, aparente,
  el número de archivos y cuánto representan del espacio usado del disco y del
  disco entero. En la raíz de un disco, indica cuánto del espacio usado se ha
  encontrado: lo que falta está en carpetas que no se pudieron leer, en los
  propios datos del sistema de archivos o en archivos eliminados que siguen
  abiertos.
- **El panel de la derecha** muestra el elemento bajo el cursor: el espacio en
  disco y aparente, archivos, fecha y su proporción respecto al espacio usado y
  al disco.

## En disco o aparente

Por defecto, los tamaños son **el espacio en disco**, tal como lo cuenta `du`:
un archivo ocupa bloques enteros (normalmente de 4 KiB), así que un archivo
pequeño ocupa más que su tamaño, y un archivo disperso (el disco de una máquina
virtual) o comprimido ocupa menos. **A** cambia al tamaño **aparente**, el
tamaño propio de los archivos. La línea inferior siempre muestra ambos.

## Qué cuenta el análisis

- **Un disco cada vez.** Otro sistema de archivos montado dentro de la rama
  (otro disco, `/boot`, una unidad USB, una carpeta montada por rclone, `/proc`)
  no se registra ni se suma: se muestra con `>` y el espacio que usa ese sistema
  de archivos, y Enter inicia un análisis del mismo.
- **Los enlaces duros se cuentan una vez.** Dos nombres para el mismo archivo
  ocupan su espacio una sola vez: el primero que se encuentra cuenta, y los
  demás se muestran con `H` y sin tamaño.
- **Los archivos ocultos cuentan**, incluso cuando el árbol no los muestra, para
  que los totales cuadren.
- Los enlaces (`@`) cuentan por el propio enlace; no se siguen.

## Marcas

| Marca | Significado |
|---|---|
| `>` | otro disco, no contado |
| `!` | no se pudo leer la carpeta (normalmente por permisos) |
| `.` | algo dentro no se pudo leer: puede ocupar más |
| `H` | otro enlace duro de un archivo ya contado |
| `@` | un enlace simbólico o archivo especial |
| `e` | una carpeta vacía |

## Actuar sobre lo que encuentre

Las preguntas son las mismas que en la ventana de archivos, y los totales se
rehacen enseguida, con la vista todavía abierta.

- **Supr en una carpeta** la borra con todo su contenido, a la papelera si esa es
  su configuración; Alt+P la borra de forma permanente, con la confirmación
  escrita de Prune.
- **Marcar una carpeta** marca todos sus archivos, también los ocultos. `■` indica
  un archivo o una carpeta entera marcada, `□` una carpeta marcada en parte, y la
  línea inferior dice cuántos hay marcados en la carpeta listada y cuánto ocupan.
  Ctrl+D borra esos. Las marcas siguen al cerrar la vista, para Copy o Move.
- **Otro disco nunca se toca.** Una línea `>` no se puede marcar ni borrar, y marcar
  o borrar una rama se detiene donde hay otro disco montado. Supr rechaza una
  carpeta con un disco montado dentro; Alt+P borra todo menos ese disco.
- En un archivo comprimido o en una máquina remota, solo funcionan marcar y ver.

## Teclas

| Tecla | Qué hace |
|---|---|
| ↑ ↓ PgUp PgDn Home End | moverse |
| Enter o → | entrar en la carpeta (en `>`: analizar ese disco); en un archivo, verlo |
| Backspace o ← | volver a la carpeta superior |
| `\` | cerrar la vista, dejando el árbol en la carpeta listada |
| `A` | espacio en disco o aparente |
| `C` | mostrar u ocultar el número de archivos |
| `S` | ordenar por tamaño, nombre, número de archivos o fecha |
| `T` / `U` | marcar o desmarcar el archivo, o todos los archivos de la carpeta |
| Ctrl+T / Ctrl+U | marcar o desmarcar todo en la carpeta listada |
| Supr, Mayús+Supr | borrar el archivo o la carpeta (Mayús: de forma permanente) |
| Ctrl+D, Ctrl+Mayús+Supr | borrar los archivos marcados en la carpeta listada (Mayús: de forma permanente) |
| Alt+P | [Prune](delete.md) de la carpeta |
| `V` | ver el archivo |
| `D` | buscar [duplicados](duplicates.md) en la carpeta listada |
| F3 | volver a leer la carpeta listada |
| Esc o Alt+U | cerrar |

Tab, F8, Alt+X (la terminal) y F12 funcionan como de costumbre con la vista
abierta.

Analizar un disco grande registra cada carpeta en memoria: lleva un rato (el pie
de página muestra el progreso y Esc cancela) y algo de memoria, unos pocos
cientos de megabytes para millones de archivos. Como usuario normal, algunas
carpetas del sistema no se pueden leer (`!`), por lo que los totales de `/` se
quedan cortos: la línea inferior indica cuántas.
