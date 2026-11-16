# Terminals und Tasten

Xylem empfängt die Tasten, die das Terminal sendet. Manche Terminals belegen
eine Taste für eigene Zwecke, und manche senden für zwei verschiedene Tasten
dieselben Bytes. Diese Seite listet diese Fälle und die Ausweichtasten auf.

## Vom Terminal oder Desktop belegte Tasten

| Taste | Belegt durch | Stattdessen verwenden |
|---|---|---|
| F1 | GNOME Terminal (eigene Hilfe) | Alt+H |
| F11 | GNOME Terminal (Vollbild) | Alt+B, oder `'` zum Anspringen eines Lesezeichens |
| Alt+F10 | GNOME (Fenster maximieren) | F12 |
| Alt+T | Terminals mit „Terminal"-Menü | andere Taste in `[keys.any]` belegen |

## Ctrl+M

Die meisten Terminals senden für Ctrl+M und Enter dasselbe Byte. Xylem sieht
dann Enter. Um markierte Dateien dort zu verschieben, drücken Sie `F9` und dann
`M`, oder nutzen Sie Alt+M und drücken F6 in der Eingabezeile, um die Pfade
abzuschalten.

In einem Terminal mit dem Kitty-Tastaturprotokoll sind die beiden Tasten
unterschiedlich und Ctrl+M verschiebt die markierten Dateien direkt.

## Das Kitty-Tastaturprotokoll

Mit diesem Protokoll meldet das Terminal das Drücken und Loslassen von Ctrl und
Alt separat. Xylem aktiviert es automatisch, wenn das Terminal Unterstützung
signalisiert.

| Mit dem Protokoll | Ohne das Protokoll |
|---|---|
| Ctrl oder Alt gedrückt halten zeigt das CTRL- oder ALT-Menü | F9 zeigt sie an |
| Ctrl+M verschiebt Dateien | Ctrl+M entspricht Enter |

| Terminal | Protokoll |
|---|---|
| kitty, Ghostty, WezTerm, foot (Wayland), aktuelles Alacritty | ja |
| iTerm2 | Ctrl+M verschiebt; Halten von Ctrl oder Alt ungeprüft |
| GNOME Terminal, Ptyxis, Terminator, Tilix (VTE-basiert) | nein |

## Die Zwischenablage

`Y` und Ctrl+Y schreiben in die Zwischenablage. Über SSH und in tmux nutzt Xylem
die OSC-52-Sequenz, die das Terminal annehmen muss. Unter X11 und Wayland bleibt
der Text nach dem Beenden von Xylem in der Zwischenablage erhalten.

## Symbole und Rahmenlinien

Icons (F12, Seite 2) benötigen eine Schriftart mit Nerd-Fonts-Glyphen. Fehlen
diese, zeichnet das Terminal für jedes Symbol ein leeres Kästchen. Verschieben
sich die Spalten bei aktiven Symbolen, zeichnet das Terminal Zeichen mit
„zweideutiger Breite" zu breit; stellen Sie diese Option im Terminal auf schmal
(*narrow*) oder schalten Sie Icons ab.

## Bildschirmgröße

| Breite oder Höhe | Wirkung |
|---|---|
| unter 72 Spalten | Name und Version entfallen in der Kopfzeile |
| zu schmal für eine Menüzeile | Menüwörter werden gekürzt, letzte Einträge durch `…` ersetzt |
| unter 58 Spalten oder 13 Zeilen | Startbildschirm wird nicht angezeigt |

Siehe auch: [die Menüs](menus.md), [Konfiguration](configuration.md),
[das integrierte Terminal](terminal.md).
