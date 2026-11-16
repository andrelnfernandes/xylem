# Instalar y los primeros cinco minutos

## Instalación

Xylem se ejecuta en Linux (x86_64 y ARM64). La compilación para Windows pasa
las pruebas pero no ha sido utilizada en el día a día; vea [Windows](windows.md).

**Binario.** Descarga el archivo comprimido para tu sistema desde
`https://github.com/andrelnfernandes/xylem/releases`, extráelo y coloca `xy`
en un directorio de tu `PATH`:

```bash
tar xzf xy-v0.7.0-x86_64-unknown-linux-musl.tar.gz
install -m 755 xy-v0.7.0-x86_64-unknown-linux-musl/xy ~/.local/bin/
```

Los binarios para Linux son estáticos y funcionan en cualquier distribución.
Cada paquete incluye un archivo `.sha256` para verificarlo con `sha256sum -c`.

**Paquete.** La misma página ofrece paquetes `.deb` y `.rpm`, que además
instalan la entrada del menú, los iconos y la página de manual:

```bash
sudo apt install ./xylem-fm_*_amd64.deb
sudo dnf install ./xylem-fm-*.x86_64.rpm
```

El paquete no proviene de un repositorio de distribución, por lo que no se
actualiza automáticamente. Instala las versiones posteriores del mismo modo.

**Cargo.** Requiere Rust y un compilador de C (para SQLite y zstd). El crate es
`xylem-fm`; el comando instalado es `xy`:

```bash
cargo install xylem-fm
```

## Primeros pasos

```bash
xy
```

abre el directorio actual. `xy ~/projects` abre otro directorio, y
`xy user@host:` abre una [máquina remota](remote.md).
[La línea de comandos](cli.md) describe las demás formas de invocación.

## Moverse en el árbol

El árbol muestra el directorio inicial y sus subdirectorios. Solo el directorio
inicial ha sido leído en memoria.

| Tecla | Qué hace |
|---|---|
| ↑ ↓ | mueve el cursor |
| → | lee el directorio bajo el cursor y muestra su contenido |
| ← | lo contrae |
| `*` | lee el directorio y todo lo que está debajo de él |

La ventana inferior lista los archivos del directorio bajo el cursor. En un
directorio no leído aún, muestra `Not logged`.

## Ir a los archivos

**Enter** pasa a la ventana de archivos, que ocupa entonces la mayor parte de la
pantalla. Enter o Esc regresa al árbol.

| Tecla | Qué hace |
|---|---|
| `V` | visualiza el archivo |
| `E` | lo abre en tu editor |
| `C`, `M`, `D`, `R` | copia, mueve, elimina o renombra el archivo |

## Marcar archivos y copiarlos

1. En la ventana de archivos, pulsa `T` sobre cada archivo deseado. Aparece un
   `■` antes del nombre y el cursor baja a la siguiente línea.
2. Pulsa Ctrl+C. El prompt pregunta adónde copiar.
3. Escribe un directorio de destino y pulsa Enter.

Los archivos marcados se copian. `U` desmarca un archivo individual y Ctrl+U
desmarca todos.

## Salir

`Q` sale del programa, tras pedir confirmación. Alt+Q sale y deja la shell en
el directorio donde estabas, siempre que la función del shell descrita en
[Salir y cambiar el directorio de la shell](quit.md) esté configurada.

## Siguientes pasos

- [La pantalla](screen.md) detalla cada zona visible.
- [Los menús](menus.md) explica el efecto de una letra sola, con Ctrl y con Alt.
- [Vocabulario](glossary.md) define log, branch, tag y los demás términos.
- **F1** abre la ayuda contextual sobre lo que está en pantalla.
