# Das Alt-Menü

Alt mit einem Buchstaben ist die andere Fassung eines Befehls. Im Baum deckt er
den gesamten Zweig unter dem Cursor ab. Im Dateifenster wirken Alt+C und Alt+M
auf die markierten Dateien und behalten deren Pfade bei. Die übrigen Einträge
öffnen Werkzeuge, die sich nicht auf eine einzelne Datei beziehen.

Um dieses Menü anzuzeigen, halten Sie Alt gedrückt oder drücken Sie F9 zweimal.
Siehe [die Menüs](menus.md).

## Befehle

In der Reihenfolge des Menüs. „Baum" und „Dateien" geben an, wo die Taste
funktioniert.

| Taste | Befehl | Baum | Dateien | Funktion |
|---|---|---|---|---|
| Alt+B | Bookmarks | ja | ja | Öffnet die Lesezeichenverwaltung: gespeichertes Verzeichnis anspringen, benennen, löschen. [Lesezeichen](bookmarks.md) |
| Alt+C | Compare | ja | | Vergleicht den gesamten Zweig anhand relativer Pfade mit dem des anderen Fensters und öffnet einen Branch mit markierten Dateien. [Vergleichen](compare.md) |
| Alt+C | Copy | | ja | Kopiert die markierten Dateien und baut den Pfad jeder Datei unter dem Ziel neu auf. F7 in der Eingabe legt fest, wo der Pfad beginnt. [Kopieren und Verschieben](copy-move.md) |
| Alt+D | Duplicates | ja | ja | Findet Dateien und Ordner mit gleichem Inhalt im Zweig. Löscht nichts ohne Bestätigung. [Duplikate](duplicates.md) |
| Alt+F | File display | ja | ja | Ändert den Anzeigemodus im Dateifenster: Namen in Spalten, Größen, Details, Besitzer, Typ, lange Namen. [Das Dateifenster](files.md) |
| Alt+H | Help | ja | ja | Öffnet die Hilfe, wie F1. |
| Alt+I | Info | ja | ja | Zeigt Typ, Größe, Berechtigungen, Besitzer und Datumsangaben der Datei oder des Verzeichnisses. [Informationen](info.md) |
| Alt+K | Kbyte | ja | | Zeigt Größe, Dateianzahl oder prozentualen Anteil neben jedem Verzeichnis im Baum an. [Informationen](info.md) |
| Alt+L | Log | ja | ja | Macht das Verzeichnis unter dem Cursor zur Wurzel des Baums. Eingelesenes und Markierungen werden verworfen. [Treespec und Log](treespec.md) |
| Alt+M | Move | | ja | Verschiebt die markierten Dateien und baut den Pfad jeder Datei unter dem Ziel neu auf. [Kopieren und Verschieben](copy-move.md) |
| Alt+P | Prune | ja | | Löscht das Verzeichnis samt Inhalt endgültig, nachdem ein Bestätigungswort eingetippt wurde. [Löschen](delete.md) |
| Alt+Q | Quit here | ja | ja | Beendet und belässt die Shell im aktuellen Verzeichnis. Erfordert die Shell-Funktion aus [Beenden und Verzeichnis der Shell wechseln](quit.md). |
| Alt+R | Release | ja | | Gibt den Zweig aus dem Speicher frei, wie `-`. Markierungen gehen verloren. |
| Alt+S | Sort | ja | ja | Sortiert das Dateifenster nach Name, Erweiterung, Datum oder Größe. [Sortieren](sort.md) |
| Alt+U | Usage | ja | ja | Listet Ordner und Dateien des Zweigs nach belegtem Speicherplatz auf, größte zuerst. [Speicherplatzbelegung](usage.md) |
| Alt+V | Views | ja | ja | Öffnet die Ansichten der Liste: den Scan, der zeigt, was einen Blick verdient, die Zeitleiste, nach Datum, und die Tabelle, die Liste im Tabellenbetrachter. [Ansichten](views.md) |
| Alt+N | iNdex | ja | ja | Indiziert die Wörter der eingelesenen Dateien des Zweigs, damit `words()` in einer SQL-Bedingung sofort antwortet. [Der Index](sql.md#ganze-wörter-und-ein-index-words) |

## Weitere Tasten

| Taste | Wo | Funktion |
|---|---|---|
| Alt+X | beide | Öffnet das integrierte Terminal und kehrt von dort zurück. [Das integrierte Terminal](terminal.md) |
| Alt+Z | beide | Öffnet das Terminal im Vollbildmodus. |
| Alt+O | beide | Tippt die markierten Dateien in Anführungszeichen in die Befehlszeile des Terminals. |
| Alt+F5 | Dateien | Öffnet das Archiv unter dem Cursor als Baum. [Archive](archives.md) |
| Alt+F10 | beide | Öffnet die Konfiguration, wie F12. Unter GNOME maximiert Alt+F10 das Fenster und erreicht Xylem nicht. [Konfiguration](configuration.md) |
| Alt+Shift+Del | beide | Löscht die markierten Dateien endgültig, wie Ctrl+Shift+Del. |

## In einem Archiv oder auf einem entfernten Rechner

| Ort | Nicht aufgeführt |
|---|---|
| Archiv | Log, Move, Prune, die Terminaltasten. Alt+C entpackt die markierten Dateien mit ihren Pfaden im Archiv. |
| Entfernter Rechner | Log (Alt+L), Alt+O. Alt+X öffnet eine Shell auf diesem Rechner. |

## Geändert in Version 0.8

Bis 0.7 bewirkten Alt+A, Alt+D, Alt+R, Alt+T und Alt+U dasselbe wie Ctrl mit
diesem Buchstaben, Alt+W war Usage und Alt+Y war Duplicates.
[Versionshinweise](releases.md) enthält die Tabelle und die Zeilen für
`config.toml`, die die alten Tasten wiederherstellen.

Siehe auch: [die Menüs DIR und FILE](commands.md),
[das Ctrl-Menü](commands-ctrl.md).
