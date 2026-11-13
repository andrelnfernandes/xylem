# La terminal integrada (Alt+X)

No está en XTree ni en ZTreeWin: tu shell (`$SHELL`) ejecutándose dentro de la
pantalla de Xylem, en la parte inferior, con el árbol y los archivos aún
visibles arriba.

- **Alt+X** abre la terminal y vuelve a Xylem. La shell sigue ejecutándose
  mientras está oculta, con su historial y todo lo que tenía en pantalla; `exit`
  la cierra.
- **Alt+Z** la abre a pantalla completa; dentro de ella, alterna entre pantalla
  completa y el panel. Alt+X la vuelve a abrir tal como estaba.
- **El árbol sigue a la shell:** un `cd` en la terminal mueve el cursor del
  árbol al mismo directorio (Linux).
- **La shell sigue al árbol:** cuando entras en la terminal desde otro
  directorio, la shell va allí. Esto solo ocurre cuando la shell está esperando
  en el prompt; si hay un programa en ejecución (vim, less...), no se escribe
  nada.
- **`$XY_TAGGED`** contiene los archivos marcados, definidos al entrar en la
  terminal. Es un array: en bash, `"${XY_TAGGED[@]}"`
  (`cp "${XY_TAGGED[@]}" ~/backup`); en zsh y fish, `$XY_TAGGED`. Con otras
  shells (sh, dash) solo se hace el `cd`.
- **Alt+O** escribe los archivos marcados en el cursor, entrecomillados y
  relativos al directorio de la shell (`wc -l 'a.txt' 'b c.txt'`). Desde el
  árbol o la ventana de archivos, abre la terminal y los escribe.
- **Shift+PgUp/PgDn** desplazan hacia atrás lo que salió de la pantalla.
- Al salir de la terminal, los directorios abiertos se vuelven a leer.
- **Salir de Xylem termina la shell** y lo que se esté ejecutando en ella. Si
  hay un programa ejecutándose allí (por ejemplo, `vim`), Q y Alt+Q preguntan
  antes, incluso con `confirm_quit` desactivado (Linux y macOS).
- Los programas que ocupan toda la pantalla (vim, htop, less) funcionan dentro
  de ella; todas las teclas van a ellos, salvo Alt+X, Alt+Z y Alt+O. Por eso F1
  y Alt+H no abren esta ayuda dentro de la terminal.

## Varios terminales lado a lado (tmux)

El terminal es un solo shell, pero un multiplexor como **tmux** dentro de él te da
varios, lado a lado o uno sobre otro: ejecuta `tmux` y divide como siempre
(`Ctrl+B %`, `Ctrl+B "`). Las teclas de tmux le llegan, porque Xylem solo se reserva
Alt+X, Alt+Z y Alt+O; Alt+Z da toda la pantalla a los paneles, y tmux los redistribuye.
Comprobado con tmux; otros multiplexores (GNU screen, zellij) no se han probado.

Con tmux en marcha:

- **El árbol solo sigue al shell que abrió Xylem,** no los `cd` hechos en los paneles
  de tmux, y entrar en el terminal desde otro directorio ya no lo cambia (hay un
  programa en marcha, así que no se escribe nada).
- **Alt+O** escribe los archivos marcados en el panel que tiene el foco; `$XY_TAGGED`
  es el de cuando empezó tmux.
- **La sesión sigue después de Xylem.** Salir de Xylem pregunta antes, porque hay un
  programa en marcha, y cierra el shell, pero tmux mantiene la sesión en segundo plano,
  como hace siempre: `tmux attach` la recupera, y `tmux kill-session` (o `exit` en todos
  los paneles) la termina.
- **Memoria:** cada panel es otro shell, ejecutado por tmux; Xylem sigue con una sola
  pantalla para el terminal, con sus 2.000 líneas de desplazamiento.

## Su propio historial

Los comandos escritos en ella van a un historial de shell propio, separado del
de tus otras terminales: `~/.local/state/xylem/bash_history` (`zsh_history`...),
mediante `HISTFILE`; en fish, la sesión `xylem`. Si tu archivo rc define
`HISTFILE` incondicionalmente, prevalece. Puedes desactivarlo con F12 (página 3)
o `terminal_history = false`.

## Teclas y colores

Las teclas se pueden cambiar en `[keys.any]` (`terminal`, `terminal_full`,
`terminal_tagged`): Alt+T, por ejemplo, abre el menú "Terminal" en algunas
terminales. Los colores se eligen con F12 (página 2) o `terminal_theme`:
`native` (los de la terminal en la que se ejecuta Xylem), `xylem` (el fondo y el
texto del tema de Xylem), `solarized`, `gruvbox`, `nord`, `dracula` o
`one_dark`.

Para entrar en la terminal y cambiar de directorio, Xylem escribe un pequeño
script en una carpeta propia (en `~/.cache/xylem/`, legible solo por ti) y
escribe ` . script` en el prompt; el espacio inicial evita que la línea se
guarde en el historial en bash (`ignorespace`) y zsh (`HIST_IGNORE_SPACE`).
Dentro de la terminal se define `XY_TERMINAL=1`, para prompts y scripts que
quieran saberlo; la terminal en una [máquina remota](remote.md) también la
define (útil, por ejemplo, para evitar que un `~/.bashrc` inicie tmux allí).
