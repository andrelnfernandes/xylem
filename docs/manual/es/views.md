# Vistas (Alt+V)

**Alt+V** abre las vistas de la lista: otras formas de mirar los archivos que
tiene la ventana de archivos, para una pregunta a la que el árbol y la lista no
responden bien. Un pequeño menú pregunta cuál, cada una con su letra:

| Tecla | Vista | Qué responde |
|---|---|---|
| `S` | [Escaneo](#scan) | ¿Qué hay en estos archivos que merezca una mirada? |
| `T` | [Línea de tiempo](#timeline) | ¿Qué cambió, y cuándo? |
| `L` | [Tabla](#table) | ¿Qué archivos cumplen esta condición, vistos como una tabla? |

Una vista trabaja sobre **la lista actual**: el directorio, el Branch o el
Showall, con el filespec y la condición `W` que estén en vigor. Solo lee: nada
en una vista elimina, mueve ni cambia un archivo. Lo que eliges en ella vuelve
a la lista clásica como un filtro, y desde ahí Ctrl+T, Ctrl+C o Batch actúan
como siempre. **W sin texto** quita ese filtro, como hace con una condición.

Cada vista puede tener también una tecla propia en `[keys]`
([configuración](configuration.md)), sin tecla por defecto: `view_scan`,
`view_timeline`, `view_table`.

## Escaneo

Los informes que el [visor](viewer.md) hace de archivos que pueden ser hostiles
(ejecutables, documentos de Office, parches), de uno en uno con `V`, aplicados
a toda una lista: un ZIP que descargaste, un repositorio que clonaste, un
pendrive. **Alt+V, S** (antes Branch, con `B`, para incluir las carpetas de
debajo).

```
      1  Names with control or text-direction characters
      1  Names that deceive (fatura.pdf.exe, spaces, a trailing dot)
      3  setuid, setgid, writable by anyone
      2  Links that leave the folder, or point to nothing
      1  Executables: packed, without PIE or NX, writable and executable
      1  Office: macros, remote template, embedded objects, hidden text
      0  Patches: dangerous paths, hidden characters, hidden changes
      0  Not analysed: over a limit, or the reader could not take it
```

| Categoría | Qué mira |
|---|---|
| Nombres con caracteres de control o de dirección del texto | El nombre de cada archivo y carpeta: caracteres de control, los que dan la vuelta al texto (un nombre dibujado como `exe.pdf` que es `fdp.exe`), bytes que no son UTF-8 |
| Nombres que engañan | La extensión de un documento antes de una que se ejecuta (`fatura.pdf.exe`, `photo.jpg.scr`), espacios antes de la extensión, un nombre que termina en espacio o en punto |
| setuid, setgid, escribible por cualquiera | El modo de cada archivo, y de cada carpeta: una carpeta en la que cualquiera puede escribir solo cuenta sin el sticky bit, que `/tmp` tiene |
| Enlaces que salen de la carpeta, o no apuntan a nada | Adónde apunta cada enlace, por su texto: el enlace nunca se sigue para leer lo que hay allí |
| Ejecutables | ELF, PE y Mach-O, reconocidos por sus primeros bytes y no por el nombre: sin PIE, sin NX o DEP, sin ASLR, una pila ejecutable, secciones empaquetadas (UPX y otros) o a la vez escribibles y ejecutables, datos después de la última sección |
| Office | Word, PowerPoint y OpenDocument: macros, lo que el documento descarga de fuera al abrirse (una plantilla remota), objetos incrustados, texto oculto y diapositivas ocultas |
| Parches | `.patch` y `.diff`: archivos que se salen con `..`, son absolutos, entran en `.git` o pasan a ser ejecutables; nombres con caracteres ocultos; hunks cuyos recuentos no cuadran; líneas cambiadas solo en lo que no se ve |
| No analizados | Los archivos que los lectores no pudieron procesar, cada uno con el motivo: por encima de un límite, un paquete roto, un lector que falló |

Los nombres y los modos de los archivos ya están en memoria, y aparecen al
instante. Los modos de las carpetas, los enlaces y los contenidos se piden al
disco, en segundo plano: la pantalla se va llenando a medida que llegan, dice
cuántos faltan, y **Esc** lo detiene.

Los contenidos pasan por los mismos lectores que los informes del
[visor](viewer.md) (`V`), con los mismos límites, de uno en uno. Un archivo por
encima de un límite, o uno que un lector no pudo procesar, va a **No
analizados** con el motivo: nunca desaparece en silencio. Como mucho se lee el
contenido de `scan_max_files` archivos (10000); a partir de ahí, se listan como
no analizados ([configuración](configuration.md)).

| Tecla | Qué hace |
|---|---|
| ↑ ↓ | moverse |
| Enter | los archivos de la categoría, cada uno con el motivo por el que está ahí |
| Enter, sobre un archivo | vuelve a la ventana de archivos, que conserva solo los archivos de esa categoría, con el cursor en este |
| Enter, sobre una carpeta | la carpeta en el árbol |
| `V`, sobre un archivo | verlo, con su informe; Esc vuelve al Escaneo |
| `R` | escanear de nuevo |
| Esc | vuelve a las categorías, y después a la lista |

Esc en la lista conserva el Escaneo: Alt+V, S sobre la misma lista lo abre tal
como estaba, sin leer de nuevo. `R` vuelve a leer.

### Dónde no mira

- **Unidades de Windows y FAT, exFAT, NTFS, SMB, montajes de rclone.** Ahí el
  controlador da a todos los archivos el mismo modo, así que los modos no dicen
  nada, y el Escaneo no los cuenta: en su lugar, la categoría lo dice. En WSL
  esto es `/mnt/c` y las demás unidades de Windows.
- **Dentro de un archivo comprimido** (Alt+F5) los modos de los archivos son
  los que el archivo comprimido registra, y cuentan: un archivo setuid en un
  TAR merece conocerse. Cada miembro se extrae con un límite (64 MiB) y se lee
  como en el disco. Las carpetas y los enlaces no se miran. En un TAR, un
  miembro muy adentro no se extrae, y se lista como no analizado.
- **En un montaje en la nube** (rclone) los contenidos no se leen: cada archivo
  tendría que descargarse. Los nombres y lo demás sí.
- **En una máquina remota** solo se miran los nombres y los modos de los
  archivos, por ahora: leer los contenidos traería cada archivo por SSH.

El Escaneo no es un antivirus. Señala lo que merece una mirada, y no dice que
el resto sea seguro.

## Línea de tiempo

Los archivos de la lista en un histograma por la fecha en que cambiaron:
cuándo hubo movimiento en una carpeta, qué llegó la semana pasada, qué fotos
son de aquel viaje. No lee nada del disco: solo las fechas que ya están en
memoria.

```
 2026-11-18: 312 files, 1.4 GiB

                                    █
                                    █▆
 ▁▁▂▁▁▁▁▁▃▁▁▂▁▁▁▁▁                  ██▁
                                    ▲
 11-01     11-05     11-09     11-13     11-17
```

La escala se elige según las fechas: horas, días, semanas, meses o años. Las
fechas anteriores a 1980 y las futuras van a una barra propia en cada extremo,
porque existen fechas erróneas u hostiles que, si no, aplastarían el gráfico.
Los archivos sin fecha se cuentan debajo.

| Tecla | Qué hace |
|---|---|
| ← → PgUp PgDn Home End | moverse de barra en barra; la línea superior dice los archivos y bytes de la barra |
| Shift+← → | elegir un rango de barras |
| `+` `-` | una escala más fina o más gruesa (`+` de días a horas) |
| `B` | la altura de las barras por bytes en lugar de por archivos |
| Enter | vuelve a la ventana de archivos solo con los archivos de la barra o del rango |
| Esc | vuelve, sin cambiar nada |

Enter convierte la elección en una [condición W](sql.md) sobre `modified`,
como `modified >= '2026-11-18 00:00:00' AND modified < '2026-11-19 00:00:00'`,
unida con AND a la condición que ya estaba en vigor. El título de la ventana de
archivos la muestra, y **W sin texto** la quita.

## Tabla

La lista se abre en el [visor de tablas](tables.md), un archivo por fila, con
las columnas de la [condición W](sql.md): `name`, `ext`, `dir`, `size`,
`disk`, `modified`, `age_days`, `mode`, `kind` y `tagged`, y `git` y `dup`
cuando algún archivo de la lista tiene estado de Git o está en un grupo de
[duplicados](duplicates.md). Los valores son los que ve `W`, así que una
condición que funciona en uno funciona en el otro.

Allí la lista tiene lo que tienen las tablas: el filtro SQL (`\` o `&`), el
registro como ficha (Enter), copiar un registro, marcas (`m`) y
[Batch](execute.md) (Ctrl+B), una línea de salida por fila. Una lista de
millones de archivos se abre al instante: la tabla se lee por ventanas, como un
archivo grande.

| Tecla | Qué hace |
|---|---|
| `\` o `&` | el filtro SQL de la tabla |
| `W` | vuelve a la ventana de archivos, con el filtro como su condición W, unido con AND a la que estaba en vigor |
| Esc | vuelve, sin cambiar nada |

La tabla es la lista tal como estaba al abrirse: un archivo marcado o cambiado
después aparece en ella la próxima vez que se abre. `type`, que lee cada
archivo, no es una columna aquí; `W` sigue teniéndolo.
