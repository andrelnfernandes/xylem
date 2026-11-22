# Configuración

## La pantalla de configuración (F12)

**F12** (o Alt+F10; en GNOME, Alt+F10 suele maximizar la
ventana y nunca llega a Xylem) abre cinco páginas:

| Página | Opciones |
|---|---|
| 1. Programas y formatos | editor, abrir el editor dentro del marco de Xylem, tema de Xylem en el editor, editar archivos binarios con E, editor binario, idioma de la ayuda, programa para abrir, formato de fecha, formato numérico, comandos de Git, carpetas Git de confianza |
| 2. Visualización y orden | archivos ocultos, orden y dirección iniciales, qué hace Enter en la ventana de archivos, modo de visualización inicial, colores de sintaxis, ajuste de línea, números de línea, archivos comprimidos, la codificación de textos antiguos, el límite de tamaño de lo que lee el visor, iconos y glifos (Nerd Fonts), nombres de archivo coloreados por tipo, bordes de las ventanas, colores de la terminal integrada, segundos en el reloj, [el ratón](mouse.md) |
| 3. Confirmaciones e historial | confirmar al salir, eliminar a la papelera, confirmación de Prune, no guardar entradas con `@`, historial propio de la terminal integrada, [Alt+T en la búsqueda aproximada marca todas las coincidencias](#tagging-every-match-of-the-fuzzy-find), [buscar una versión nueva](#new-versions) |
| 4. Copiar, comparar y actualizar | ignorar 2 s, ignorar 1 h, actualización automática, dónde empiezan las rutas de Alt+C/Alt+M, archivos y carpetas que quedan fuera del índice |
| 5. Lectura (libros electrónicos) | el ancho de la columna de texto, párrafos como en un libro o en bloques, texto justificado, la velocidad de lectura para el tiempo restante, [el tiempo dedicado a leer](#reading-time), los minutos sin pulsar una tecla antes de que su reloj se detenga, los títulos guardados para una lista de los libros |

El límite de tamaño del visor (**L** en la página 2) cuesta memoria y tiempo: con `1024` MiB, abrir un archivo de texto normal de ese tamaño ocupa unos 2,5 GiB de RAM y tarda unos 5 s, y los archivos de líneas muy cortas cuestan más por byte (consulta [el visor](viewer.md)).

Los iconos y glifos (**M** en la página 2) requieren una fuente con el parche de Nerd Fonts. No necesitas una fuente específica: cualquier fuente monoespaciada sirve (como Fira Code, JetBrains Mono, Hack, Meslo o Cascadia Code), siempre que sea su edición "Nerd Font". Nerd Fonts añade miles de glifos e iconos estándar a la zona privada de Unicode. Las fuentes comunes sin el parche (incluso la Fira Code original) no contienen estos dibujos y mostrarán cuadros vacíos (`□`). Para usarlos:
- Descarga cualquier Nerd Font de tu preferencia desde nerdfonts.com o instala un paquete de Nerd Fonts con el gestor de paquetes de tu distribución.
- Selecciona esa fuente en las preferencias de tu emulador de terminal.

Si las columnas se desalinean con los iconos activados, probablemente la terminal dibuja los caracteres de "ancho ambiguo" como anchos (un ajuste habitual con fuentes chinas, japonesas y coreanas), mientras que Xylem cuenta cada icono como una celda. Pon esa opción en estrecho o desactiva los iconos.

Los nombres coloreados por tipo (**N** en la página 2) dan a los archivos comprimidos, imágenes, audio y vídeo, documentos, código y datos un color del tema cada uno, solo por el nombre (no se lee nada). Desactivado, la lista tiene el aspecto clásico. Los bordes de las ventanas (**O**) son redondeados en los temas nord, gruvbox, slate, graphite, daylight y vivid, y rectos en los demás; `plain` o `rounded` los fija en todos los temas. La pantalla de inicio (**P**) muestra el nombre en letras de bloque durante dos segundos cuando Xylem se abre; cualquier tecla la cierra (esa tecla no hace nada más), y mientras tanto se lee la carpeta inicial. No aparece al conectarse a una máquina remota o a un contenedor desde la línea de comandos, ni en una terminal de menos de 58 por 13.

La letra de la opción (o Enter) cambia el valor, PgUp/PgDn o 1–5 cambian de
página, **F4** elige el tema de una lista (con vista previa instantánea), **F3**
guarda y aplica, y **Esc** cancela (pregunta si hay cambios).

### Marcar todas las coincidencias de la búsqueda aproximada

En la búsqueda aproximada (`|`), **Alt+T** marca los resultados de la lista. La
lista guarda las 500 mejores coincidencias, y una búsqueda puede encontrar
muchas más: `f` en un árbol grande puede coincidir con cien mil archivos. Por
defecto, Alt+T marca solo las 500 de la lista, y cuando coincidieron más, la
línea bajo la lista lo dice y nombra esta opción.

Con **Alt+T en la búsqueda aproximada marca todas las coincidencias** activada
(**F** en la página 3, `fuzzy_tag_all = true` en el archivo), Alt+T marca todas
las coincidencias, incluidas las que nunca viste. La línea bajo la lista dice
entonces siempre cuántas coincidencias no estaban en ella.

Está desactivada por defecto, porque las marcas pasan a lo que hagas después.
Ctrl+C, Ctrl+M, Ctrl+D y el Batch actúan sobre todos los archivos marcados, no
solo sobre los que viste. Miles de archivos marcados por una búsqueda que solo
miraste por encima, y luego eliminados, son miles de archivos perdidos. Activa
la opción solo si compruebas lo que está marcado antes de actuar:

- el panel de estadísticas muestra cuántos archivos están marcados y su tamaño;
- **Ctrl+S** en el árbol lista los archivos marcados de todo el árbol (un
  Showall de los marcados);
- **Ctrl+V** en la ventana de archivos los muestra uno tras otro.

### Versiones nuevas

La primera vez que Xylem arranca, cuando desaparece la pantalla de inicio,
pregunta: *Look for a new version of Xylem once a day?* **Y** dice que sí,
**N** que no, y la respuesta se guarda en el archivo de configuración
(`check_updates`). **Esc** no guarda nada, y la pregunta vuelve la próxima vez.
**G** en la página 3 cambia la respuesta más adelante.

Con ella activada, una vez al día Xylem pide al sitio web
(`https://andrelnfernandes.github.io/xylem/latest.txt`) una línea: el número
de la última versión. Lo hace mediante el `curl` del sistema, en segundo plano,
solo con HTTPS y en cinco segundos como máximo; sin red, o sin `curl`, no se
dice nada y lo intenta de nuevo al día siguiente. La petición no lleva más que
`xylem/` y la versión que usas; el sitio ve tu dirección IP, como en cualquier
visita. No se descarga ni se instala nada: cuando hay una versión más nueva, la
línea superior dice `0.10.0 is out` junto al reloj, y un mensaje da la
dirección de las versiones. Actualiza como instalaste: el gestor de paquetes
para un `.deb` o un `.rpm`, un archivo nuevo en los demás casos.

La respuesta del sitio web se lee como hostil: cualquier cosa que no sean tres
números, como la página del Wi-Fi de un hotel, se ignora. La fecha de la última
comprobación y la versión encontrada se guardan en la carpeta de estado
(`~/.local/state/xylem/update-check`).

`XYLEM_CHECK_UPDATES=0` (o `1`) en el entorno responde sin preguntar y tiene
prioridad sobre el archivo, para instalaciones en muchas máquinas. Una
compilación sin la función `update-check` (`cargo build --no-default-features
--features index`), como la que puede hacer una distribución, no tiene
pregunta, ni comprobación, ni opción.

### Tiempo de lectura

El visor lee [libros electrónicos](viewer.md#ebooks) EPUB y FB2. La página 5
define cómo se dispone el texto, y si Xylem guarda el tiempo que pasas leyendo.

**Guardar el tiempo dedicado a leer** (**E**, `reading_stats = true` en el
archivo) está desactivado por defecto. Activado, Xylem guarda, para cada libro
recordado con `R` en el visor:

- el tiempo leído, y las sesiones: una pausa de más de 30 minutos empieza una
  nueva;
- el punto más lejano alcanzado, el primer y el último día de lectura, y si el
  libro se terminó;
- la velocidad de lectura, en palabras por minuto. Solo cuentan las páginas
  leídas hacia delante, y solo cuando se pasaron al menos 5 segundos en ellas,
  así que los saltos, volver atrás y hojear no cuentan. Tras 10 minutos
  medidos, el pie calcula el tiempo restante con la velocidad de ese libro en
  lugar de la de esta página (**D**).

El reloj corre solo mientras lees: de una tecla a la siguiente, si entre ellas
no pasaron más de **F** minutos (3 por defecto). Una pausa más larga lo
detiene, y esa pausa no cuenta. Autoview (F7) y el informe (`I`) nunca cuentan.

Todo queda en tu ordenador, en `~/.local/state/xylem/reading.toml` (o
`$XDG_STATE_HOME/xylem/`), legible solo por ti. Un libro se guarda como un hash
de lo que dice ser y de su tamaño, y números: no su nombre, no dónde está, no
su título. Con **G** activado (`reading_titles = true`), también se guardan el
título y el autor, y el informe de cualquier libro lista entonces todos los
libros leídos, bajo "Your books".

Desactivar **G** y guardar (F3) borra al instante los títulos guardados.
Desactivar **E** y guardar pregunta si se borra el tiempo leído de todos los
libros; si no, queda en el archivo y no se muestra. En ambos casos los lugares
recordados con `R` se mantienen; `R` en un libro lo olvida.

## El archivo de configuración

Todo se guarda en `~/.config/xylem/config.toml` (o `$XDG_CONFIG_HOME/xylem/`;
`%APPDATA%\xylem\` en Windows), que también se puede editar a mano: la pantalla
conserva tus comentarios. Para empezar a partir de un archivo comentado con
todos los valores predeterminados:

```bash
mkdir -p ~/.config/xylem && xy --default-config > ~/.config/xylem/config.toml
```

- `theme`: `classic` (blanco y amarillo sobre azul), `dark`, `light`, `mono` (los colores propios
  de la terminal), `norton`, `solarized`, `gruvbox`, `nord`, `slate` (al estilo
  de Windows Terminal), `graphite` (macOS en modo oscuro), `daylight` (macOS en
  modo claro) o `vivid` (colores intensos, para presentaciones y capturas de
  pantalla). F4 los muestra todos con vista previa.
- `[options]`: las opciones de la pantalla de configuración, y algunas que
  solo están en el archivo: `scan_max_files` (10000), el máximo de archivos
  cuyo contenido lee el [Escaneo](views.md#scan).
- `[colors]`: colores individuales sobre el tema, como nombres (`"yellow"`),
  `"#rrggbb"` o índices de 0 a 255.
- `[keys.any]`, `[keys.tree]`, `[keys.files]`: reasignan teclas en ambas
  ventanas o solo en una (que tiene prioridad). Por ejemplo, `"f3" = "view"`,
  `"ctrl+r" = "relog"`; `"none"` desactiva una tecla. La lista de teclas de la
  ayuda (**K**) muestra las teclas activas con tus cambios.
- `[editors]`: el editor para cada tipo de archivo
  (`"*.png *.jpg" = "gimp %f"`, `binary = "hexedit"`); véase
  [Editar](execute.md).

Un error en el archivo no impide que Xylem arranque: usa los valores
predeterminados, muestra el error en el pie de página y la pantalla de
configuración no guardará hasta que se corrija el archivo, para no borrar lo que
hayas escrito.

Xylem se llamaba xt hasta la versión 0.1. En su primera ejecución mueve la
configuración, el historial y los marcadores de xt a sus propias carpetas.
