# Renombrar muchos archivos con un contador

Este procedimiento renombra las nueve fotos de `photos/2025/trip` de
`DSC_01.jpg` … `DSC_09.jpg` a `trip_001.jpg` … `trip_009.jpg`.

## Paso a paso

1. En el árbol, coloca el cursor sobre `trip` y pulsa Enter para ir a sus
   archivos.
2. Pulsa Ctrl+T. Todos los archivos de la lista quedan marcados.
3. Pulsa Ctrl+R. El prompt solicita la máscara y se abre con `*.*`.
4. Pulsa Ctrl+U para borrar el prompt, escribe `trip_<001>.*` y pulsa Enter.

![Los cuatro pasos: los archivos de trip, Ctrl+T, Ctrl+R y la máscara trip_<001>.* dando trip_001.jpg a trip_009.jpg](../../screenshots/howto-rename-mask.gif)

El pie muestra `Renamed 9 of 9 files`. Los archivos se numeran en el orden de
la lista, de modo que ordénala antes (Alt+S) si el orden importa.

## La máscara

| Parte | Significado |
|---|---|
| `trip_` | texto copiado a cada nombre |
| `<001>` | un contador con tres dígitos: 001, 002, … |
| `.*` | la extensión de cada archivo, sin cambios |

El paso 4 borra el prompt porque escribir se añade al `*.*` existente:
`*.*trip_<001>.*` renombraría el primer archivo como `DSC_01.DSC_01trip_001.jpg`.

## Otras máscaras

| Máscara | `DSC_01.jpg` pasa a ser |
|---|---|
| `*.jpeg` | `DSC_01.jpeg` |
| `2025_*.*` | `2025_DSC_01.jpg` |
| `s/DSC/trip/` | `trip_01.jpg` |
| `*.<upper>` | `DSC_01.JPG` |

[Renombrar](rename.md) contiene todas las variantes disponibles.

## Cuando no hace lo que esperas

- **No se renombra nada.** La máscara asignaría el mismo nombre a dos o más
  archivos. Sin contador y sin `*`, todos reciben el mismo nombre.
- **Solo se renombraron algunos archivos.** Únicamente se renombran los archivos
  marcados. Un Filespec puede ocultar archivos de la lista y Ctrl+T solo marca
  lo visible.
- **Ctrl+R renombra un solo archivo.** Se pulsó `R` sola, que renombra el
  archivo bajo el cursor.

Véase también: [Marcas y operaciones en grupo](tagging.md),
[el menú Ctrl](commands-ctrl.md).
