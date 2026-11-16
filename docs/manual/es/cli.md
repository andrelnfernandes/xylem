# La línea de comandos

```
xy [--cwd-file ARCHIVO] [DÓNDE]
```

## Qué abrir

| `DÓNDE` | Abre |
|---|---|
| nada | el directorio actual |
| `DIRECTORIO` | ese directorio, como raíz del árbol |
| `user@host:` o `user@host:RUTA` | una máquina por SSH, en el directorio personal o en `RUTA` |
| `ssh://[user@]host[:puerto][/RUTA]` | lo mismo, especificando el puerto |
| `docker://[user@]container[/RUTA]` | un contenedor en ejecución, mediante `docker exec` |

Las mismas direcciones funcionan en el comando `L` (Log) dentro de Xylem. Vea
[máquinas remotas](remote.md).

## Opciones

| Opción | Qué hace |
|---|---|
| `--cwd-file ARCHIVO` | Al pulsar Alt+Q, escribe el directorio actual en `ARCHIVO`. Una función del shell lo lee y cambia de directorio. [Salir y cambiar el directorio de la shell](quit.md) incluye la función. |
| `--default-config` | Muestra un archivo de configuración con cada opción, su valor predeterminado y un comentario explicativo. |
| `--man` | Muestra la página de manual. Con la salida redirigida, imprime su código fuente roff. |
| `-h`, `--help` | Muestra las opciones y la ruta del archivo de configuración. |
| `-V`, `--version` | Muestra la versión. |

## Instalar la página de manual

Los paquetes `.deb` y `.rpm` la instalan automáticamente. Desde un binario o desde Cargo:

```bash
mkdir -p ~/.local/share/man/man1 && xy --man > ~/.local/share/man/man1/xy.1
```

## Iniciar desde una configuración comentada

```bash
mkdir -p ~/.config/xylem && xy --default-config > ~/.config/xylem/config.toml
```

Esto sobrescribe un `config.toml` existente.

## Variables de entorno

| Variable | Efecto |
|---|---|
| `XDG_CONFIG_HOME` | La configuración se lee desde `$XDG_CONFIG_HOME/xylem/` en lugar de `~/.config/xylem/`. |
| `SHELL` | La shell de la terminal integrada. |
| `VISUAL`, `EDITOR` | El editor para `E`, cuando no se especifica ninguno en la configuración. |
| `XY_DOCKER` | El programa usado para direcciones `docker://`. `XY_DOCKER=podman` utiliza Podman. |

Dentro de la terminal integrada, Xylem establece `XY_TERMINAL=1` y, en bash,
zsh y fish, `XY_TAGGED` con los archivos marcados. Vea
[la terminal integrada](terminal.md).

Véase también: [configuración](configuration.md).
