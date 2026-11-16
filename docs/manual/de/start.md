# Installieren und die ersten fünf Minuten

## Installation

Xylem läuft unter Linux (x86_64 und ARM64). Der Windows-Build besteht die
Tests, wurde jedoch nicht im Alltag erprobt; siehe [Windows](windows.md).

**Binärdatei.** Laden Sie das Archiv für Ihr System von
`https://github.com/andrelnfernandes/xylem/releases` herunter, entpacken Sie es
und legen Sie `xy` in einem Verzeichnis Ihres `PATH` ab:

```bash
tar xzf xy-v0.7.0-x86_64-unknown-linux-musl.tar.gz
install -m 755 xy-v0.7.0-x86_64-unknown-linux-musl/xy ~/.local/bin/
```

Die Linux-Binärdateien sind statisch kompiliert und laufen auf jeder
Distribution. Zu jedem Archiv gehört eine `.sha256`-Datei zur Überprüfung mit
`sha256sum -c`.

**Paket.** Auf derselben Seite finden sich `.deb`- und `.rpm`-Pakete, die auch
den Menüeintrag, die Icons und die Manpage installieren:

```bash
sudo apt install ./xylem-fm_*_amd64.deb
sudo dnf install ./xylem-fm-*.x86_64.rpm
```

Das Paket stammt nicht aus einem Distributions-Repository und wird daher nicht
automatisch aktualisiert. Installieren Sie Folgeversionen auf dieselbe Weise.

**Cargo.** Erfordert Rust und einen C-Compiler (für SQLite und zstd). Das Crate
heißt `xylem-fm`, der installierte Befehl `xy`:

```bash
cargo install xylem-fm
```

## Starten

```bash
xy
```

öffnet das aktuelle Verzeichnis. `xy ~/projects` öffnet ein anderes, und
`xy user@host:` öffnet einen [entfernten Rechner](remote.md).
[Die Befehlszeile](cli.md) beschreibt die weiteren Aufrufvarianten.

## Im Baum bewegen

Der Baum zeigt das Startverzeichnis und seine Unterverzeichnisse an. Nur das
Startverzeichnis wurde eingelesen.

| Taste | Funktion |
|---|---|
| ↑ ↓ | Cursor bewegen |
| → | Verzeichnis unter dem Cursor einlesen und Inhalt anzeigen |
| ← | Verzeichnis zuklappen |
| `*` | Verzeichnis und alles darunter einlesen |

Das untere Fenster listet die Dateien des Verzeichnisses unter dem Cursor auf.
Bei einem noch nicht eingelesenen Verzeichnis steht dort `Not logged`.

## Zu den Dateien wechseln

**Enter** wechselt in das Dateifenster, das daraufhin fast den gesamten
Bildschirm einnimmt. Enter oder Esc kehrt zum Baum zurück.

| Taste | Funktion |
|---|---|
| `V` | Datei im Betrachter anzeigen |
| `E` | Datei im Editor öffnen |
| `C`, `M`, `D`, `R` | kopieren, verschieben, löschen oder umbenennen |

## Dateien markieren und kopieren

1. Drücken Sie im Dateifenster `T` auf jeder gewünschten Datei. Ein `■`
   erscheint vor dem Namen und der Cursor rückt nach unten.
2. Drücken Sie Ctrl+C. Die Eingabeaufforderung fragt nach dem Zielverzeichnis.
3. Geben Sie ein Verzeichnis ein und drücken Sie Enter.

Die markierten Dateien werden kopiert. `U` hebt die Markierung einer einzelnen
Datei auf, Ctrl+U die aller Dateien.

## Beenden

`Q` beendet nach Bestätigung. Alt+Q beendet und belässt die Shell in dem
Verzeichnis, in dem Sie sich befanden, sofern die Shell-Funktion aus
[Beenden und Verzeichnis der Shell wechseln](quit.md) eingerichtet ist.

## Weiterführendes

- [Der Bildschirm](screen.md) benennt jeden sichtbaren Bereich.
- [Die Menüs](menus.md) erläutert die Wirkung einzelner Buchstaben, mit Strg und
  mit Alt.
- [Vokabular](glossary.md) definiert Log, Branch, Tag und die übrigen Begriffe.
- **F1** öffnet die kontextbezogene Hilfe zum Bildschirminhalt.
