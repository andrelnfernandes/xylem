# Los menús, Ctrl y Alt

El pie de página lista los comandos disponibles en la ventana que tiene el foco.
Existen cuatro menús. El título a la izquierda indica cuál de ellos se muestra.

| Título | Se muestra cuando | Sus comandos actúan sobre |
|---|---|---|
| DIR | el árbol tiene el foco | el directorio bajo el cursor |
| FILE | la ventana de archivos tiene el foco | el archivo bajo el cursor |
| CTRL | se mantiene pulsado Ctrl, o tras pulsar F9 | los archivos marcados |
| ALT | se mantiene pulsado Alt, o tras pulsar F9 dos veces | consulte [el menú Alt](#alt-la-otra-version-de-un-comando) |

Dentro de un archivo comprimido el título es ARCHIVE, y en una máquina remota
es REMOTE. Ambos omiten los comandos que no funcionan en ese contexto.

## Cómo leer un menú

Cada palabra representa un comando y su letra mayúscula indica la tecla: `Copy`
es `C`, `eXecute` es `X`, `iNdex` es `N`. En los menús CTRL y ALT, la tecla es
dicha letra combinada con Ctrl o Alt.

Las palabras siguen orden alfabético por su tecla asignada, con Quit al final.
Debajo aparece una línea con teclas de función y símbolos (`F3 relog`,
`? stats`).

El pie de página tiene tres líneas. En una pantalla suficientemente ancha para
mostrar todas las palabras en una sola fila, se reduce a dos. En pantallas
estrechas, las palabras se acortan por igual y los comandos que no caben se
sustituyen por `…`.

## Ctrl: los archivos marcados

Una letra sola actúa sobre un único elemento. La misma letra con Ctrl actúa
sobre los archivos marcados de la lista:

| Tecla | Sola | Con Ctrl |
|---|---|---|
| `C` | copia el archivo | copia los archivos marcados |
| `D` | elimina el archivo | elimina los archivos marcados |
| `R` | renombra el archivo | renombra los archivos marcados con una máscara |
| `T` | marca el archivo | marca todos los archivos de la lista |

En el árbol, "la lista" se refiere al directorio bajo el cursor. Tres comandos
tienen alcance mayor: Ctrl+T y Ctrl+U marcan y desmarcan todos los archivos
registrados que coinciden con el Filespec, y Ctrl+S lista los archivos marcados
de todo lo registrado.

[El menú Ctrl](commands-ctrl.md) describe cada comando.

## Alt: la otra versión de un comando

Alt no repite a Ctrl. Su acción depende de la ventana activa:

- En el árbol, el comando abarca toda la rama. Alt+P (Prune) elimina el
  directorio junto con su contenido, Alt+R libera la rama de la memoria y Alt+C
  compara la rama con la otra ventana.
- En la ventana de archivos, Alt+C y Alt+M copian y mueven los archivos marcados
  recreando la ruta de cada uno bajo el destino.
- En ambas ventanas, Alt abre herramientas no asociadas a un único archivo:
  Alt+S (Sort), Alt+F (File display), Alt+B (Bookmarks), Alt+I (Info) y Alt+X
  (la terminal integrada).

[El menú Alt](commands-alt.md) describe cada comando.

## Mostrar los menús Ctrl y Alt

**F9** muestra el menú CTRL; pulsar F9 nuevamente muestra el menú ALT, y una
tercera vez regresa al menú original. Mientras se muestra uno de ellos, la
siguiente letra se interpreta con dicho modificador: `F9` `C` equivale a
Ctrl+C. Esc cancela sin ejecutar ninguna acción.

En kitty, Ghostty, WezTerm, foot y versiones recientes de Alacritty, mantener
pulsado Ctrl o Alt despliega el menú correspondiente, y soltarlo recupera DIR o
FILE. Estas terminales informan eventos de teclas modificadoras solitarias (el
protocolo de teclado de kitty). GNOME Terminal, Ptyxis, Terminator y Tilix no lo
soportan, de modo que en ellas los menús solo se alternan mediante F9.

## Ctrl+M

La mayoría de las terminales envían el mismo byte para Ctrl+M y Enter, por lo
que Xylem no puede distinguirlos. En terminales compatibles con el protocolo de
teclado de kitty, las dos teclas son distintas y Ctrl+M mueve los archivos
marcados. En las demás, pulse `F9` `M`.

## Cambios en la versión 0.8

Alt+A, Alt+D, Alt+R, Alt+T y Alt+U solían realizar la misma acción que Ctrl con
la misma letra. [Notas de versión](releases.md) detalla el significado anterior
y el nuevo de cada tecla, así como las líneas para `config.toml` que permiten
restaurar los anteriores.
