# Seguridad

Xylem trata como hostil todo archivo que no haya creado él mismo: archivos
descargados, archivos comprimidos abiertos, nombres enviados por servidores
remotos, repositorios clonados. El contenido o el nombre de un archivo nunca debe
hacer que Xylem falle, se cuelgue, ejecute código, escriba fuera del destino o
agote la memoria. Si esto ocurre, se trata de un error que debe notificarse.

## Qué hace Xylem

| Riesgo | Medida |
|---|---|
| Archivo manipulado explota el lector | Texto, hexadecimal, codificaciones, archivos comprimidos, archivadores, Markdown, hojas de cálculo, CSV, JSON Lines y DBF se leen con analizadores escritos en Rust. SQLite y Parquet se leen mediante bibliotecas escritas en C únicamente cuando se abre el archivo con `V`, nunca en Autoview. |
| Archivo ejecuta un programa | Ningún programa externo se ejecuta sobre un archivo a menos que lo solicites con `E`, `O` o `X`. |
| Archivo pequeño se expande para agotar memoria o disco | Cada lector tiene límites sobre cuánto lee, cuánto descomprime, cuánto tiempo tarda y cuánta memoria emplea el descompresor. Solo se deshace un nivel de compresión. Visualizar un archivo no escribe nada en el disco. |
| Nombre oculta lo que realmente es | Los caracteres de control y los controles bidireccionales (Trojan Source) no se envían a la terminal. Los nombres con caracteres de dirección ocultos se marcan en rojo y `O` pide confirmación antes de abrirlos. |
| Archivo comprimido escribe fuera del destino | Las rutas con `..` y las rutas absolutas son rechazadas, los enlaces simbólicos en archivos comprimidos no se extraen y los bits setuid se descartan. Las mismas comprobaciones se aplican a los nombres enviados por un servidor SFTP. |
| Repositorio Git ejecuta sus propios comandos | Examinar un repositorio no ejecuta hooks, ni `core.fsmonitor`, ni diff externo ni conversiones de texto, salvo que su carpeta esté incluida en `git_trusted`. |
| Dependencia con vulnerabilidad conocida | Las dependencias se verifican contra la base de datos de RustSec en la integración continua (CI). |

## La red

Xylem se conecta a la red por su cuenta para una sola cosa: si dijiste que sí
en el primer arranque, pregunta al sitio web una vez al día el número de la
última versión ([Versiones nuevas](configuration.md#new-versions)). Todo lo
demás que usa la red es algo que pediste tú: una máquina remota (`ssh`), la
nube (`rclone`), un contenedor (`docker`), una carpeta compartida en red y los
programas que ejecutas tú mismo (`X`, la terminal integrada).

## Qué puedes hacer tú

- Desactiva la descompresión en el visor (`viewer_decompress = false`, F12
  página 2) al explorar carpetas no confiables con Autoview activo, o mantén
  Autoview cerrado allí.
- Deja Git desactivado (`git = false`, la opción predeterminada) en repositorios
  que no hayas creado tú.
- Di que no a buscar versiones nuevas, o desactívalo en la página 3 de F12, si
  Xylem nunca debe conectarse por sí mismo.
- No ejecutes Xylem como usuario root.
- `O` pasa el archivo a otro programa. Los límites de Xylem no se aplican a
  dicho programa.

## Qué no está cubierto

- Los programas que inician `E`, `O` y `X`.
- `rclone`, `ssh`, `docker` y `git`, que Xylem ejecuta según estén instalados en
  el sistema operativo.
- Una terminal que por sí misma procese incorrectamente el texto recibido.

## Notificar un problema

Notifícalo de forma privada a través de GitHub Security Advisories:
`https://github.com/andrelnfernandes/xylem/security/advisories/new`. No abras
una issue pública para ello.

Véase también: [el visor](viewer.md), [archivos comprimidos](archives.md),
[Git](git.md).
