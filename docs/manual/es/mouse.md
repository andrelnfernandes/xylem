# El ratón

Xylem está hecho para el teclado, y el ratón está apagado hasta que lo
enciendas: **F12**, página 2, **R** (o `mouse = true` en las `[options]` del
[archivo de configuración](configuration.md)), y luego **F3** para guardar.
Funciona al instante.

El ratón nunca tiene comandos propios. Un clic elige una fila, o equivale a la
tecla escrita donde hiciste clic, y esa tecla hace lo que siempre hace: Delete
sigue preguntando, y un comando que no puede ejecutarse aquí sigue siendo
rechazado.

## Qué hace

| Dónde | Clic | Doble clic | Rueda |
|---|---|---|---|
| El árbol | elige el directorio; la ventana de archivos lo sigue | Enter: a sus archivos | mueve el cursor 3 directorios |
| La marca antes de una carpeta | `+` la abre, `?` la lee y la abre, el `─` de una abierta la cierra | | |
| La ventana de archivos | elige el archivo y pasa a la ventana de archivos | View, en el visor propio de Xylem | mueve el cursor 3 archivos (una fila en los modos de columnas) |
| La barra de desplazamiento del borde derecho | encima o debajo del control: una página; arrastra el control para ir a cualquier punto | | |
| El otro lado de una [pantalla dividida](split.md) | lo hace la ventana activa, y luego elige | | mueve el cursor de ese lado; el activo se queda |
| La [terminal integrada](terminal.md), el editor en el marco | al programa, si pidió el ratón (mc, vim, htop, tmux); si no, nada | | al programa, si lo pidió; si no, hacia atrás por las líneas que salieron de la pantalla, como Shift+PgUp |
| El árbol encima de la terminal integrada | vuelve a Xylem, como Alt+X | | |
| El pie de página | la tecla de la entrada: `Copy` es C, `Esc cancel` es Esc, `Yes` responde la pregunta | | |
| F12 | una opción cambia, como con su letra (nada se guarda antes de F3); un tema de la lista (F4) se previsualiza | se elige el tema | mueve la selección |
| Marcadores, la caja de búsqueda, Usage, Duplicates | elige la fila | su Enter | 3 filas |
| El historial de un campo de entrada (↑) | pone esa entrada en el campo | | 3 filas |
| El texto de un campo de entrada | el cursor va allí | | |
| Una tabla en el visor | elige la celda | su Enter: el registro | 3 filas; Shift+rueda, una columna |
| El visor, la ayuda | se sigue un enlace de la ayuda | | desplaza, como ↑ y ↓; Shift+rueda desplaza el visor hacia los lados |
| Donde un menú tenga sentido | el botón derecho lo abre (abajo) | | |

Un doble clic nunca abre un archivo con un programa del sistema: un archivo que
no creaste puede ser hostil, y un clic por error no debe ejecutarlo. Open
(`O`) sigue ahí.

Arrastrar con el botón izquierdo mueve el cursor a lo largo de una lista; más
allá de su borde superior o inferior, la lista se desplaza una fila en cada
movimiento.

## El menú del botón derecho

El botón derecho mueve el cursor al elemento bajo el puntero, como un clic, y
abre junto al puntero un menú de lo que se puede hacer con él. La tecla
**Menú** del teclado, o **Shift+F10**, abre el mismo menú para el elemento bajo
el cursor, sin ratón.

| Bajo el puntero | El menú |
|---|---|
| Un archivo | View, Edit, Open with the system, Info; Tag o Untag; Copy, Move, Rename, Delete, Attributes, New date, Yank the path |
| Un archivo, con archivos marcados | lo mismo, y un grupo para los marcados: Copy, Move, Rename (mask), Delete, View, Yank, Untag all |
| Un directorio del árbol | Files, Log, Relog, Expand all, Collapse, Release; Branch, Showall; Make a directory, Rename, Delete, Prune, Attributes; Tag its files, Untag; Terminal here, Usage, Duplicates, Bookmark it, Yank the path |
| La ventana de archivos, después de sus archivos | Filespec, Sort, File display; Tag all, Untag all, Invert tags |
| El visor | Search, Next match, Go to line, y las teclas de su modo (Wrap, Line numbers, Hex, SQL filter...), Edit, Yank the path, Close |

Cada línea muestra su tecla a la derecha, tal como la tiene tu configuración,
así que el menú también es una forma de aprender las teclas. Lo que no puede
ejecutarse aquí se omite: dentro de un archivo comprimido no hay Delete, Move
ni Rename. Una línea se ejecuta como lo haría su tecla: Delete sigue
preguntando antes.

↑↓ eligen, Enter ejecuta, Esc cierra; un clic fuera del menú solo lo cierra.

## Marcar

| Gesto | Qué hace |
|---|---|
| Botón central, o Ctrl+clic, sobre un archivo | lo marca o lo desmarca; el cursor se queda donde está |
| Lo mismo sobre una carpeta del árbol | la elige y marca sus archivos, o los desmarca cuando todos estaban marcados |
| Shift+clic sobre un archivo | marca todos los archivos desde el cursor hasta ese, y el cursor va allí |

Muchas terminales reservan Shift+clic para su propia selección; allí el botón
central o Ctrl+clic marcan cada archivo.

## El foco, los menús y las preguntas

La rueda mueve la lista bajo el puntero, no la que tiene el foco: la ventana de
archivos pequeña bajo el árbol se desplaza sin salir del árbol.

En los menús Ctrl y Alt (mantenidos pulsados, o abiertos con F4), una palabra
equivale a su letra con Ctrl o Alt: un clic en `Copy` en el menú Ctrl copia los
archivos marcados.

Mientras una pregunta, un campo de entrada o un trabajo en curso está en la
pantalla, las ventanas detrás no reciben clics: solo sus propias entradas en el
pie de página.

## Seleccionar texto en la terminal

Con el ratón encendido, la terminal envía los clics a Xylem, así que arrastrar
ya no selecciona texto en la pantalla. Mantén **Shift** pulsado mientras
arrastras (en xterm, GNOME Terminal, Konsole, kitty, WezTerm, Windows
Terminal), u **Option** en iTerm2. Para copiar la ruta de un archivo, Yank
(`Y`) no necesita ratón.

## Terminales

Xylem pide los clics y los movimientos con un botón pulsado, en el formato SGR
que conoce toda terminal actual; nunca los movimientos del puntero solos, que
enviarían un flujo de eventos para nada, un peso sobre SSH. Un programa en la
terminal integrada o en el editor en el marco recibe el ratón en el formato que
pidió, con las esquinas de su propio panel: nunca llega al resto de la
pantalla, ni a tu terminal. Un arrastre que sale del panel se queda con el
programa, sujeto en su borde. Un programa ejecutado con eXecute o Edit puede
apagar el ratón al terminar; Xylem lo enciende de nuevo cuando recupera la
pantalla.
