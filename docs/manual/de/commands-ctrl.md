# Das Strg-Menü

Ctrl mit einem Buchstaben führt den Befehl auf den markierten Dateien aus. Im
Dateifenster sind dies die markierten Dateien der Liste (des Verzeichnisses oder
der geöffneten Branch- bzw. Showall-Ansicht). Im Baum sind es die markierten
Dateien des Verzeichnisses unter dem Cursor. Die mit „alles Eingelesene"
gekennzeichneten Zeilen bilden die Ausnahme.

Sind keine Dateien markiert, zeigen diese Befehle `No tagged files` an und tun
nichts.

Um dieses Menü anzuzeigen, halten Sie Ctrl gedrückt oder drücken Sie F9. Siehe
[die Menüs](menus.md).

## Befehle

In der Reihenfolge des Menüs. „Baum" und „Dateien" geben an, wo die Taste
funktioniert.

| Taste | Command | Baum | Dateien | Funktion |
|---|---|---|---|---|
| Ctrl+A | Attributes | ja | ja | Ändert die Berechtigungen der markierten Dateien. [Berechtigungen und Datum](attributes.md) |
| Ctrl+B | Bookmark | ja | | Speichert das Verzeichnis unter dem Cursor als Lesezeichen. Nutzt keine Markierungen. [Lesezeichen](bookmarks.md) |
| Ctrl+B | Batch | | ja | Schreibt anhand einer Maske ein Skript oder eine Liste mit einer Zeile pro markierter Datei. [Batch](execute.md) |
| Ctrl+C | Copy | ja | ja | Kopiert die markierten Dateien in ein Zielverzeichnis, ohne Pfade. Alt+C behält die Pfade bei. [Kopieren und Verschieben](copy-move.md) |
| Ctrl+D | Delete | ja | ja | Löscht die markierten Dateien nach Bestätigung. Sie wandern in den Papierkorb, wenn `delete_to_trash` aktiv ist. [Löschen](delete.md) |
| Ctrl+G | Git | ja | ja | Öffnet das Git-Menü. Erscheint nur bei aktivierter Git-Option. [Git](git.md) |
| Ctrl+M | Move | ja | ja | Verschiebt die markierten Dateien in ein Zielverzeichnis. Drücken Sie in Terminals ohne Kitty-Tastaturprotokoll `F9` `M`. [Kopieren und Verschieben](copy-move.md) |
| Ctrl+N | New date | | ja | Ändert das Änderungsdatum der markierten Dateien. [Berechtigungen und Datum](attributes.md) |
| Ctrl+O | Open | | ja | Öffnet jede markierte Datei im Standardprogramm des Systems. [Das Dateifenster](files.md) |
| Ctrl+P | Phrase | ja | ja | Öffnet das Suchfeld für markierte Dateien: listet diejenigen auf, die die eingegebenen Wörter enthalten. [Das Suchfeld](filespec.md#das-suchfeld-p) |
| Ctrl+R | Rename | ja | ja | Benennt die markierten Dateien mit einer Maske um. [Umbenennen](rename.md) |
| Ctrl+S | Showall | ja | | Listet die markierten Dateien von allem Eingelesenen auf. [Das Baumfenster](tree.md) |
| Ctrl+S | Search | | ja | Durchsucht die markierten Dateien nach einem Text und hebt die Markierung bei fehlender Übereinstimmung auf. [Markierte Dateien durchsuchen](search.md) |
| Ctrl+T | Tag | ja | ja | Dateien: markiert die gesamte Liste. Baum: markiert alles Eingelesene, was zum Filespec passt. [Markierungen und Gruppenoperationen](tagging.md) |
| Ctrl+U | Untag | ja | ja | Dateien: hebt die Markierung der gesamten Liste auf. Baum: hebt die Markierung von allem Eingelesenen auf, was zum Filespec passt. |
| Ctrl+V | View | | ja | Öffnet die markierten Dateien nacheinander im Betrachter; `N` und `P` wechseln zwischen den Dateien. [Markierte Dateien durchsuchen](search.md) |
| Ctrl+W | Where | ja | ja | Behält die markierten Dateien in der Liste, die einer SQL-Bedingung entsprechen. [Eine SQL-Bedingung](filespec.md#eine-sql-bedingung-w) |
| Ctrl+Y | Yank | ja | ja | Kopiert die Pfade der markierten Dateien in die Zwischenablage. |

## Funktionstasten

| Taste | Wo | Funktion |
|---|---|---|
| Ctrl+F5 | Dateien | Packt die markierten Dateien in ein neues Archiv. [Archive](archives.md) |
| Ctrl+F7 | Baum | Markiert die eingelesenen Dateien des Zweigs unter dem Cursor. |
| Ctrl+F8 | Baum | Hebt deren Markierung auf. |
| Ctrl+F9 | Baum | Kehrt deren Markierung um. |
| Ctrl+F11 | beide | Speichert ein Lesezeichen, wie Ctrl+B im Baum. |
| Ctrl+Shift+Del | beide | Löscht die markierten Dateien endgültig, ohne Papierkorb. |
| Ctrl+Z | beide | Oops: stellt das zuletzt in den Papierkorb Verschobene wieder her. |

## In einem Archiv oder auf einem entfernten Rechner

Das Menü lässt weg, was dort nicht ausgeführt werden kann.

| Ort | Nicht aufgeführt |
|---|---|
| Archiv | Attributes, Delete, Git, Move, New date, Rename, Ctrl+F5. Copy entpackt. |
| Entfernter Rechner | Batch, Git, Ctrl+F5. |
