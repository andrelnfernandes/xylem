# Terminales y teclas

Xylem recibe las teclas que envía la terminal. Algunas terminales reservan una
tecla para sí mismas y otras envían los mismos bytes para dos teclas distintas.
Esta página describe esos casos y la combinación alternativa que se debe usar.

## Teclas reservadas por la terminal o el escritorio

| Tecla | Reservada por | Usar en su lugar |
|---|---|---|
| F1 | GNOME Terminal (su propia ayuda) | Alt+H |
| F11 | GNOME Terminal (pantalla completa) | Alt+B, o `'` para saltar a un marcador |
| Alt+F10 | GNOME (maximiza la ventana) | F12 |
| Alt+T | terminales con menú "Terminal" | asignar otra tecla en `[keys.any]` |

## Ctrl+M

La mayoría de las terminales envían el mismo byte para Ctrl+M y Enter. Xylem
interpreta Enter. Para mover los archivos marcados en ellas, pulsa `F4` y luego
`M`, o utiliza Alt+M y pulsa F6 en el prompt para no recrear las rutas.

En una terminal con el protocolo de teclado de kitty las dos teclas son
distintas y Ctrl+M mueve los archivos marcados directamente.

## El protocolo de teclado de kitty

Con este protocolo la terminal informa de Ctrl y Alt pulsados y soltados por sí
solos. Xylem lo activa automáticamente cuando la terminal responde que lo
soporta.

| Con el protocolo | Sin el protocolo |
|---|---|
| Mantener pulsado Ctrl o Alt muestra el menú CTRL o ALT | F4 los muestra |
| Ctrl+M mueve archivos | Ctrl+M actúa como Enter |

| Terminal | Protocolo |
|---|---|
| kitty, Ghostty, WezTerm, foot (Wayland), Alacritty reciente | sí |
| iTerm2 | Ctrl+M mueve; mantener pulsado Ctrl o Alt no ha sido verificado |
| GNOME Terminal, Ptyxis, Terminator, Tilix (basados en VTE) | no |

## El portapapeles

`Y` y Ctrl+Y escriben en el portapapeles. A través de SSH y dentro de tmux Xylem
emplea la secuencia OSC 52, que la terminal debe admitir. En X11 y Wayland el
texto permanece en el portapapeles tras cerrar Xylem.

## Iconos y líneas de marco

Los iconos (F12, página 2) requieren una tipografía con glifos de Nerd Fonts. Sin
ellos, la terminal dibuja un recuadro vacío por cada icono. Si las columnas se
desalinean al activar los iconos, la terminal está dibujando caracteres de "ancho
ambiguo" como anchos; configura esa opción en la terminal como estrecho
(*narrow*) o desactiva los iconos.

## Tamaño de pantalla

| Ancho o alto | Efecto |
|---|---|
| menos de 72 columnas | el nombre y la versión desaparecen del encabezado |
| demasiado estrecha para una línea de menú | las palabras del menú se abrevian y los últimos comandos se reemplazan por `…` |
| menos de 58 columnas o 13 líneas | la pantalla de inicio no se muestra |

Véase también: [los menús](menus.md), [configuración](configuration.md),
[la terminal integrada](terminal.md).
