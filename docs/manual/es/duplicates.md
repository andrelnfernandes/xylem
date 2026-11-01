# Duplicados (Alt+Y)

Archivos con el mismo contenido, sean cuales sean sus nombres y carpetas, y el
espacio que desperdician las copias. **Alt+Y** busca en la rama del directorio
actual; **D** en la vista de Uso busca en la carpeta listada allí.

```
── 3 × 1.2 GiB (2.4 GiB to free) ──────────────────────
     1.2 GiB  2023-08-14  Backups/phone-2023.zip
 T   1.2 GiB  2023-08-14  Downloads/phone-2023.zip
 T   1.2 GiB  2023-08-14  Old/phone-2023 (1).zip
── 2 × 850.0 MiB (850.0 MiB to free) ──────────────────
     850.0 MiB  2021-12-25  Videos/christmas.mp4
 T   850.0 MiB  2022-01-03  WhatsApp/VID-20211225.mp4
```

Lo que más libera aparece primero. Nada queda marcado hasta que lo pidas, y una
copia de cada grupo siempre queda: Xylem no deja marcar la última.

## Elegir lo que se va

| Tecla | Qué hace |
|---|---|
| ↑ ↓ PgUp PgDn Home End | mover |
| `T` o Espacio | marca o desmarca la copia |
| `K` | conserva esta copia: marca las otras del grupo |
| `A` | en cada grupo, marca todas menos una; `A` otra vez cambia cuál queda: la más antigua, la más nueva, la ruta más corta |
| `F` | en cada grupo con una copia en la carpeta de esta copia, conserva esa |
| `U` | desmarca todo |
| `D`, Supr o Ctrl+D | borra las marcadas, como el comando Delete (a la papelera si `delete_to_trash` está activo) |
| `M` o Ctrl+M | mueve los archivos marcados |
| `G` | archivos o carpetas enteras |
| `V` o Enter | muestra el archivo |
| `E` | escribe la lista en un archivo (nunca sobre uno que exista): tamaño, fecha, marca y ruta, con una línea en blanco entre grupos |
| `\` | cierra la vista, con el árbol en esa carpeta |
| F3 | lee las carpetas otra vez y compara otra vez |
| Esc o Alt+Y | cierra |

Antes de borrar o mover, cada grupo se revisa otra vez: si una copia cambió desde
el análisis, o la que iba a quedar desapareció, no se toca nada de ese grupo, y
el pie lo indica.

## Carpetas enteras

Las copias de seguridad repetidas enteras son las que más ocupan, y una lista
larga de archivos lo oculta. **G** muestra las carpetas con exactamente el mismo
contenido: los mismos nombres, el mismo contenido de cada archivo, grande o
pequeño, y los mismos enlaces apuntando al mismo lugar. Solo se listan las de más
arriba, no las carpetas que están dentro de ellas.

Una carpeta nunca se llama copia si algo en ella no se comparó: una carpeta
ilegible, otro disco montado dentro, un archivo especial o una carpeta ignorada
(abajo). Dos clones de un repositorio tienen los mismos archivos, pero su `.git`
puede guardar trabajo distinto.

## Cómo se comparan los archivos

Los tamaños vienen de la memoria. Solo se leen los archivos cuyo tamaño coincide
con el de otro: primero los dos extremos, luego el medio, por partes, y un grupo
se divide en cuanto sus archivos difieren, así que los archivos distintos no se
leen hasta el final. La comparación es por SHA-256. Corre en segundo plano: el pie
muestra el progreso y Esc cancela.

Quedan fuera de la lista:

- los archivos menores que `duplicates_min_size` (1 KiB por defecto) y los vacíos;
- los enlaces y archivos especiales, que no se siguen;
- otros enlaces duros del mismo archivo: ya comparten el espacio, y borrar uno no
  libera nada;
- otros discos montados dentro de la rama;
- las carpetas de `duplicates_ignore` (`.git` y `node_modules` por defecto).

Un archivo que cambió desde que se leyó el árbol, o que no se puede leer, no se
compara; la línea de abajo dice cuántos. F3 lee todo otra vez.

En una carpeta de red (NFS, SMB, sshfs...), se pregunta antes el total a leer. En
una máquina remota o dentro de un archivo comprimido, Duplicados aún no está
disponible.

## En la nube (rclone)

En una carpeta montada con `rclone mount`, leer un archivo significa descargarlo.
Por eso Xylem pide a rclone los hashes que el servicio ya guarda (`rclone lsjson
--hash`) y no descarga nada: un Drive entero tarda minutos. Usa el hash más fuerte
que dé el servicio (SHA-256, SHA-1, MD5...), junto con el tamaño, y rclone debe
estar en el PATH. Si no lo está, o el remoto no da hashes (uno cifrado, por
ejemplo), Xylem explica el motivo y pregunta antes de leer los archivos por el
montaje, con el total que descargaría.

Las carpetas enteras solo se comparan en discos locales.

En Google Drive:

- Un archivo en dos carpetas a la vez, o un acceso directo, es un solo archivo:
  nunca se ofrece contra sí mismo.
- Docs, Hojas y Presentaciones no tienen hash: la línea de abajo dice cuántos
  quedaron fuera.
- Lo que otros compartieron contigo aparece con `s`, y marcar todas menos una no
  lo toca: borrarlo no libera nada de tu almacenamiento.
- Dos archivos con un mismo nombre en una carpeta no se pueden distinguir por el
  montaje; la línea de abajo sugiere `rclone dedupe`.
- Borrar envía a la papelera del Drive, que sigue contando hasta que se vacía.
  Xylem no la vacía, porque eso se lleva todo lo que hay en ella; muestra el
  comando: `rclone cleanup gdrive:`.

El hash del servicio se acepta por confianza. MD5 puede falsificarse a propósito:
en archivos que otros pudieron poner allí, compara antes de borrar.
