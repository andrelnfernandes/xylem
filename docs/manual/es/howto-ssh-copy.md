# Copiar archivos de otra máquina por SSH

Este procedimiento muestra `deploy@webserver` al lado de esta máquina y copia
tres archivos de copia de seguridad desde ella a un directorio local.

## Paso a paso

1. Pulsa F8. La pantalla se divide en dos ventanas, ambas en el directorio
   actual. La ventana izquierda tiene el foco.
2. Pulsa `L`, escribe `deploy@webserver:` y pulsa Enter. La ventana izquierda
   muestra ahora el directorio personal de ese usuario en esa máquina.
3. Coloca el cursor sobre `backups`, pulsa → para leerlo y Enter para ir a sus
   archivos.
4. Pulsa `T` sobre cada archivo que deseas copiar.
5. Pulsa Ctrl+C. El prompt sugiere el directorio de la otra ventana.
6. Pulsa Enter.

![Los seis pasos: F8, Log hacia deploy@webserver, los archivos de backups, tres archivos marcados, Ctrl+C sugiriendo la ventana local y tres archivos copiados](../../screenshots/howto-ssh-copy.gif)

El pie muestra `Copied 3 of 3 files`.

## La dirección

| Dirección | Abre |
|---|---|
| `user@host:` | el directorio personal |
| `user@host:/srv/www` | ese directorio |
| `ssh://user@host:2222/srv/www` | lo mismo, en otro puerto |
| `docker://container/path` | un contenedor en ejecución, mediante `docker exec` |

Xylem ejecuta tu propio cliente `ssh`. Los alias de host, claves, el agente SSH
y `known_hosts` de `~/.ssh/config` se aplican, y nada se instala en la máquina
remota.

## Contraseñas

Si ssh solicita una contraseña o una frase de paso (*passphrase*), la petición
se muestra en el pie y lo que escribes permanece oculto. Xylem se lo pasa a ssh
y no lo almacena. Ante un host visitado por primera vez, el pie muestra la
huella (*fingerprint*) de su clave y espera `yes`.

## En la otra dirección

Para copiar hacia la máquina remota, pasa a la ventana local con Tab, marca los
archivos y pulsa Ctrl+C: el prompt sugerirá el directorio de la ventana remota.
Move, Rename, Delete, Makedir, Attributes y New date también funcionan en el
lado remoto.

## Cuando no hace lo que esperas

- **La ventana de archivos dice `Not logged`.** El directorio no se leyó. Vuelve
  con Esc y pulsa → sobre él. El paso 3 realiza esto.
- **Ctrl+C solicita un destino sin sugerir nada.** La pantalla no está dividida.
  Escribe una ruta local o pulsa F8 primero.
- **Edit y eXecute no figuran en el menú.** No funcionan en una máquina remota.
  Alt+X abre una shell allí.
- **No hay papelera.** Delete en una máquina remota es definitivo y la pregunta
  lo advierte.

Para regresar más tarde sin escribir la dirección, guarda el directorio como un
marcador con Ctrl+F11.

Véase también: [máquinas remotas (SSH)](remote.md),
[pantalla dividida](split.md), [marcadores](bookmarks.md).
