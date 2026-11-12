# Marcadores

Los marcadores te llevan de vuelta a los directorios que usas a menudo, con una
sola tecla para cada uno (`0`–`9`, `a`–`z`). Se guardan entre sesiones junto con
el historial de entradas.

- **Ctrl+F11** (o Ctrl+B en el árbol) guarda el directorio actual: pulsa la
  tecla que quieras asignarle.
- **`'`** salta: pulsa la tecla del marcador. En esa misma entrada, `'` o Enter
  abre la lista y `+` guarda el directorio actual.
- **F11** (o Alt+B) abre el gestor de marcadores: ↑ ↓ eligen, Enter (o la tecla
  del marcador) salta, **F2** le da un nombre, Del elimina, Insert o `+` guarda
  el directorio actual y Esc cierra.
- **F11 en las entradas de [Log](treespec.md), [Copy y Move](copy-move.md)**
  abre la misma lista para elegir un lugar: Enter (o la tecla del marcador) lo
  pone en la entrada, donde aún lo confirmas; Esc vuelve a lo que escribiste.

Un directorio fuera del árbol se abre como un árbol nuevo, igual que con
[Log](treespec.md). En la ventana de archivos, Ctrl+B es [Batch](execute.md);
allí usa Ctrl+F11.

## Nombres

Un nombre dice para qué sirve un marcador: "Servidor de Oracle", "Docker de
Postgres Amb 1". En el gestor, **F2** sobre un marcador pide el nombre, con el
actual para editar; un nombre vacío lo quita. El nombre aparece antes del
lugar, y el lugar aparece siempre al lado, para que un nombre no oculte adónde
lleva el marcador. Un nombre tiene como máximo 60 caracteres, sin caracteres de
control ni de dirección del texto.

## Máquinas remotas y contenedores

En una ventana de una [máquina remota](remote.md) o de un contenedor, Ctrl+F11
guarda la carpeta por la dirección que la abre: `ssh://oracle@db01/u01/app`,
`docker://postgres-amb1/var/lib/postgresql`. Ir a él conecta como escribir la
dirección en Log, pidiendo la contraseña si ssh la necesita; no se guarda
ninguna contraseña. Si la ventana ya está en esa máquina y la carpeta está en
su árbol, solo se mueve el cursor. Una carpeta cuya ruta tiene un espacio no se
puede guardar así, porque una dirección no admite espacios.

El gestor marca `[not found]` una carpeta local que ya no existe. No comprueba
los marcadores remotos, ni carpetas en un montaje de nube o un recurso
compartido de red: eso esperaría a la red.
