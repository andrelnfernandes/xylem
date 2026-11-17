# Usar los archivos marcados en un comando del shell

Este procedimiento ejecuta `wc -c` sobre los archivos marcados en Xylem desde la
terminal integrada. Los mismos pasos funcionan con cualquier comando que acepte
nombres de archivos.

## Paso a paso

1. En el árbol, coloca el cursor sobre el directorio, pulsa → para leerlo y
   Enter para ir a sus archivos.
2. Marca los archivos: `T` en cada uno, o Ctrl+T para toda la lista.
3. Pulsa Alt+X. Se abre una terminal en la parte inferior de la pantalla, con la
   shell situada en el directorio mostrado arriba.
4. Escribe el comando y un espacio, `wc -c `, y pulsa Alt+O. Xylem escribe los
   nombres de los archivos marcados en la posición del cursor, protegidos por
   comillas.
5. Pulsa Enter.

![Los cinco pasos: los archivos de notes, Ctrl+T, Alt+X abriendo la terminal, wc -c con Alt+O escribiendo los dos nombres de archivo y la salida obtenida](../../screenshots/howto-terminal-tagged.gif)

Alt+X regresa a Xylem. La shell sigue ejecutándose, con su historial, hasta
`exit`.

## Qué escribe Alt+O

Los nombres son relativos al directorio de la shell y van entre comillas, de modo
que nombres con espacios se pasan como un único argumento cada uno:

```
wc -c 'ideas.md' 'todo.md'
```

En un Branch o Showall los archivos marcados proceden de varios directorios y
cada nombre incluye su ruta desde el directorio de la shell.

## La variable

Al entrar en la terminal Xylem define además `XY_TAGGED` con los archivos
marcados.

| Shell | Cómo usarla |
|---|---|
| bash | `cp "${XY_TAGGED[@]}" ~/backup` |
| zsh, fish | `cp $XY_TAGGED ~/backup` |
| sh, dash | no se define |

La variable contiene los archivos marcados al momento de ingresar a la terminal.
Si marcas más archivos, se actualizará la próxima vez que entres.

## Un único comando con eXecute

`X` (eXecute) solicita una línea de comando y la ejecuta. En ella `%t` representa
los archivos marcados y `%f` el archivo bajo el cursor:

```
tar czf photos.tar.gz %t
```

[eJecutar](execute.md) describe los demás marcadores.

## Cuando no hace lo que esperas

- **La primera línea de la terminal muestra ` . '…/sync.sh'`.** Xylem la escribe
  para desplazar la shell al directorio del árbol y establecer `XY_TAGGED`. El
  espacio inicial evita que se registre en el historial de bash y zsh.
- **Alt+O no escribe nada.** No hay ningún archivo marcado.
- **La shell no cambió de directorio.** Había un programa ejecutándose en la
  terminal. Xylem solo envía el comando `cd` cuando la shell está esperando en su
  prompt.
- **F1 no abre la ayuda.** En la terminal cada tecla se envía directamente a la
  shell, excepto Alt+X, Alt+Z y Alt+O.

Véase también: [la terminal integrada](terminal.md), [eJecutar](execute.md),
[Marcas y operaciones en grupo](tagging.md).
