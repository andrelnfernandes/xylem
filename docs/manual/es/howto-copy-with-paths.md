# Copiar archivos de varias carpetas conservando las rutas

Este procedimiento copia todos los archivos JPG bajo `~/projects` a
`/mnt/backup`, cada uno en la misma subcarpeta que tenía en origen. Los mismos
pasos funcionan con Mover (Alt+M).

## Paso a paso

1. En el árbol, coloca el cursor sobre `projects`.
2. Pulsa `B`. Xylem registra la rama y la ventana de archivos lista los archivos
   de `projects` y de todas las carpetas debajo de él. En ramas grandes el pie
   muestra el progreso y Esc cancela la operación.
3. Pulsa `F`, escribe `*.jpg` y pulsa Enter. La lista muestra ahora solo los JPG.
4. Pulsa Ctrl+T. Todos los archivos de la lista quedan marcados (`■`).
5. Pulsa Alt+C. El prompt pregunta el destino.
6. Escribe `/mnt/backup`. Antes de pulsar Enter, revisa el modo mostrado en el
   prompt y pulsa **F7** hasta que indique `Relative`.
7. Pulsa Enter. Si `/mnt/backup` no existe, Xylem pregunta antes de crearlo.

![Los siete pasos: Branch en projects, el filespec *.jpg, Ctrl+T, Alt+C, el destino con el modo Relative y 21 archivos copiados](../../screenshots/howto-copy-paths.gif)

La franja inferior muestra el paso en ejecución y sus teclas.

## Qué modo de ruta elegir

El paso 6 determina dónde comienza cada ruta bajo el destino. Para
`~/projects/photos/2025/trip/DSC_01.jpg`, con el Branch abierto en `projects`:

| Modo | El archivo se escribe en |
|---|---|
| Full | `/mnt/backup/home/user/projects/photos/2025/trip/DSC_01.jpg` |
| Tree | `/mnt/backup/` y la ruta desde la raíz del árbol de Xylem |
| Current | `/mnt/backup/projects/photos/2025/trip/DSC_01.jpg` |
| Relative | `/mnt/backup/photos/2025/trip/DSC_01.jpg` |

El modo inicial del prompt es `Full`, a menos que `copy_paths` en la
configuración indique otro.

## Variaciones

- Sin recrear rutas: Ctrl+C en lugar de Alt+C coloca todos los JPG directamente
  en `/mnt/backup`. Dos archivos con el mismo nombre entrarán en colisión y
  Xylem preguntará qué hacer con cada uno.
- Por fecha o tamaño: en el paso 3, `*.jpg,>TODAY-30` conserva los JPG
  modificados en los últimos 30 días. [Filespec](filespec.md) describe la
  sintaxis.
- De todo el disco: registra más carpetas con `*` y pulsa `S` (Showall) en el
  paso 2.
- Selección manual: salta el paso 4 y marca cada archivo individual con `T`.

## Cuando no hace lo que esperas

- **Alt+C inicia una comparación (Compare).** El foco está en el árbol. Alt+C
  copia con rutas solo en la ventana de archivos; pulsa Enter para ir a ella.
- **Faltan archivos en la lista.** Un Filespec definido antes sigue activo. Se
  indica en el título de la ventana de archivos; pulsa `F`, `*`, Enter para
  limpiarlo.
- **Las carpetas en el destino empiezan en `/home`.** El modo era `Full`. Consulta
  el paso 6.

Ctrl+U desmarca la lista y `B` regresa a los archivos exclusivos del directorio.

Véase también: [Copiar y mover](copy-move.md),
[Marcas y operaciones en grupo](tagging.md), [La ventana del árbol](tree.md).
