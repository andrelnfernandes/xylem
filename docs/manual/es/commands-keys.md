# Teclas de función y símbolos

Estas teclas figuran en la última línea de los menús DIR y FILE, o no aparecen
en los menús porque solo mueven el cursor. No se combinan con los menús de F9:
tras F9, únicamente la siguiente letra admite Ctrl o Alt.

## Teclas de función

| Tecla | Dónde | Qué hace |
|---|---|---|
| F1 | ambas | Abre la ayuda sobre lo que está en pantalla. Alt+H realiza la misma acción. [Terminales y teclas](terminals.md) |
| F2 | ambas | Vuelve a leer el directorio, igual que F3. En un prompt que solicita una carpeta, abre un árbol para [elegirla](picker.md). |
| F3 | ambas | Vuelve a leer el directorio desde el disco. |
| F5 | Archivos | Empaqueta el archivo en un archivo comprimido. Ctrl+F5: con los archivos marcados. Alt+F5: abre un archivo comprimido. [Archivos comprimidos](archives.md) |
| F7 | Archivos | Autoview: muestra el contenido del archivo bajo el cursor al lado de la lista. [La ventana de archivos](files.md) |
| F8 | ambas | Abre una segunda ventana al lado de la primera, o la cierra. [Pantalla dividida](split.md) |
| F9 | ambas | Muestra el menú CTRL, luego el menú ALT y regresa al normal. [Los menús](menus.md) |
| F10 | ambas | Sale del programa, igual que `Q`. |
| F11 | ambas | Abre el gestor de marcadores, igual que Alt+B. Ctrl+F11 guarda el directorio actual. [Marcadores](bookmarks.md) |
| F12 | ambas | Abre la configuración. [Configuración](configuration.md) |

## Símbolos

| Tecla | Dónde | Qué hace |
|---|---|---|
| `*` | Árbol | Registra el directorio bajo el cursor y todo lo que está debajo de él. Esc cancela. |
| `+` o → | Árbol | Registra el directorio y muestra sus subdirectorios. |
| `-` | Árbol | Libera la rama de la memoria. |
| ← | Árbol | Contrae el directorio, o va al directorio padre. |
| `?` | ambas | Muestra las estadísticas de la ventana, del disco, de la máquina y de Xylem. [Información](info.md) |
| `\` o `/` | ambas | Treespec: escribe una ruta y el cursor del árbol la sigue. En un Branch o Showall, va al directorio del archivo bajo el cursor. [Treespec y Log](treespec.md) |
| `'` | ambas | Salta a un marcador: pulsa su tecla asignada a continuación. |

## Moverse

| Tecla | Dónde | Qué hace |
|---|---|---|
| ↑ ↓ PgUp PgDn Home End | ambas | Mueven el cursor. |
| Espacio | ambas | Baja una línea. |
| Enter | Árbol | Va a la ventana de archivos. |
| Enter | Archivos | Vuelve al árbol. Con `enter_views` activo, visualiza el archivo. |
| Esc | Archivos | Vuelve al árbol, en el directorio del archivo bajo el cursor. |
| Esc | Árbol | Dentro de un archivo comprimido, lo cierra. |
| Tab | ambas | Con la pantalla dividida, cambia a la otra ventana. De lo contrario, igual que Enter. |
| Backspace | ambas | Va al directorio padre. |
| Shift+letra, 0–9 | ambas | Va al siguiente directorio o archivo cuyo nombre comience por ese carácter. |
| ← → | Archivos | Página arriba y abajo. En modos de columnas, archivo anterior y siguiente. |

## Cambiar una tecla

`[keys.any]`, `[keys.tree]` y `[keys.files]` en `config.toml` asignan una tecla
a otro comando, o a `none`. **K** en la ayuda lista las teclas vigentes con
dichos cambios. Consulta [configuración](configuration.md).

Véase también: [los menús DIR y FILE](commands.md),
[el menú Ctrl](commands-ctrl.md), [el menú Alt](commands-alt.md).
