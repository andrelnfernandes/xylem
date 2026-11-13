# Markierungen und Gruppenoperationen

Markierte Dateien (gekennzeichnet mit `■`) sind das Ziel der
Gruppenoperationen: bearbeitet werden die markierten Dateien **der aktuellen
Liste** (des Verzeichnisses oder der geöffneten Branch- bzw. Showall-Ansicht).

## Markieren

| Taste | Baum | Dateifenster |
|---|---|---|
| `T` / `U` | Dateien des Verzeichnisses markieren / Markierung aufheben | Datei markieren / Markierung aufheben und nach unten gehen |
| Ctrl+T / Ctrl+U | alles Eingelesene (passend zum Filespec) markieren / Markierung aufheben | gesamte Liste markieren / Markierung aufheben |
| Ctrl+S | Showall nur der markierten Dateien | |
| `I` | Markierungen des Verzeichnisses umkehren | Markierungen der Liste umkehren |
| Ctrl+F7 / F8 / F9 | eingelesene Dateien des Zweigs markieren / Markierung aufheben / umkehren | |

## Operationen auf markierten Dateien

| Taste | Funktion |
|---|---|
| Ctrl+C / Ctrl+M | in das Zielverzeichnis [kopieren / verschieben](copy-move.md) |
| Alt+C / Alt+M | kopieren / verschieben, **unter Wiederherstellung ihrer Pfade** |
| Ctrl+D | [löschen](delete.md) |
| Ctrl+A | [Berechtigungen](attributes.md) ändern |
| Ctrl+N | [Datum](attributes.md) ändern |
| Ctrl+R | mit einer Maske [umbenennen](rename.md) |
| Ctrl+B | [Batch](execute.md): ein Skript oder eine Liste mit einer Zeile pro Datei |
| Ctrl+F5 | in ein [Archiv](archives.md) packen |
| Ctrl+S / Ctrl+V | [nach Text durchsuchen / betrachten](search.md) |
| Ctrl+O | im Standardprogramm öffnen |
| Ctrl+Y | ihre Pfade in die Zwischenablage kopieren |
| Alt+O | in das [integrierte Terminal](terminal.md) einfügen |

Bei eXecute steht `%t` für die markierten Dateien ([Ausführen](execute.md)).

## Was Ctrl und Alt bedeuten

Wie in ZTreeWin:

- **Ein Buchstabe allein** wirkt auf das Element unter dem Cursor.
- **Ctrl+Buchstabe** wirkt auf die markierten Dateien: Ctrl+C kopiert sie,
  Ctrl+D löscht sie, Ctrl+T markiert alles.
- **Alt+Buchstabe** ist die andere Fassung eines Befehls: im Baum der ganze
  Zweig (Alt+P Prune, Alt+R Release, Alt+U Speicherbelegung, Alt+D Duplikate,
  Alt+C Compare); im Dateifenster die markierten Dateien mit ihren Pfaden
  (Alt+C, Alt+M); und die Werkzeuge (Alt+S Sort, Alt+F File display, Alt+B
  Lesezeichen, Alt+H Hilfe, Alt+I Info, Alt+X das Terminal).

**Geändert in Version 0.8.** Alt wiederholt Ctrl nicht mehr, und einige
Tasten haben sich geändert:

| Taste | Bis 0.7 | Ab 0.8 |
|---|---|---|
| Alt+A | Berechtigungen der markierten Dateien | nichts (Ctrl+A) |
| Alt+D | markierte Dateien löschen | Duplikate (löschen: Ctrl+D) |
| Alt+R | markierte Dateien umbenennen | Release des Zweigs, wie `-` (umbenennen: Ctrl+R) |
| Alt+T | alles markieren | nichts (Ctrl+T) |
| Alt+U | alle Markierungen aufheben | Speicherbelegung (aufheben: Ctrl+U) |
| Alt+W | Speicherbelegung | nichts (Alt+U) |
| Alt+Y | Duplikate | nichts (Alt+D) |

Die alten Tasten lassen sich in `[keys.any]` der
[Konfiguration](configuration.md) zurückholen, etwa `"alt+w" = "usage"`.

## Ctrl- und Alt-Menüs (F9)

Die Fußzeile hat auf einem gewöhnlichen Bildschirm drei Zeilen: die
Buchstabenbefehle in alphabetischer Reihenfolge in den ersten beiden, die
Funktionstasten und Symbole in der dritten. Ist der Bildschirm breit genug für
alle Menüs, hat sie zwei: die Buchstaben in einer, die Tasten in der anderen. Jeder Befehl steht in einem der Menüs (DIR oder FILE, CTRL, ALT); in
einem Archiv oder auf einem entfernten Rechner lassen sie weg, was dort nicht
geht.

**F9** schaltet zum Ctrl-Menü um, dann zum Alt-Menü und wieder
zurück. Ist eines dieser Menüs geöffnet, zählt der nächste Buchstabe als
Ctrl+Buchstabe bzw. Alt+Buchstabe (`F9` `M` = Ctrl+M); `Esc` bricht ab. In
Terminals mit dem Kitty-Tastaturprotokoll genügt es, **Ctrl oder Alt gedrückt
zu halten**, um das entsprechende Menü anzuzeigen.

Beim Loslassen erscheint wieder das übliche Menü, wie in ZTree.

Das Terminal muss das Drücken und Loslassen von Ctrl und Alt allein melden, und
das kann nur das Kitty-Tastaturprotokoll: kitty, Ghostty, WezTerm, foot (Wayland)
und aktuelles Alacritty unterstützen es, und unter Ubuntu oder Debian sind die
meisten nur ein Paket entfernt (`sudo apt install kitty`, oder `ghostty`,
`alacritty`, `foot`). VTE-basierte Terminals (GNOME Terminal, Ptyxis, Terminator,
Tilix) unterstützen es noch nicht, dort bleiben die Menüs also auf F9.

**Ctrl+M:** In den meisten Terminals wird Ctrl+M genau wie Enter übertragen.
Xylem aktiviert das Kitty-Tastaturprotokoll, wenn das Terminal es unterstützt
(kitty, WezTerm, foot, Ghostty, aktuelles Alacritty, iTerm2), sodass Ctrl+M die
markierten Dateien wie in ZTree verschiebt. Verwenden Sie in VTE-basierten
Terminals (GNOME Terminal, Ptyxis, Terminator, Tilix) stattdessen **F9 M**
(oder Alt+M und F6 in der Eingabezeile).
