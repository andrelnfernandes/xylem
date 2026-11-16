# Die Menüs DIR und FILE

Ein Buchstabe allein führt den Befehl auf dem Element unter dem Cursor aus: auf
dem Verzeichnis im Baum (das DIR-Menü), auf der Datei im Dateifenster (das
FILE-Menü).

Manche Buchstaben stehen in jedem Fenster für einen anderen Befehl. `C`
vergleicht im Baum und kopiert im Dateifenster; `M` erstellt ein Verzeichnis im
Baum und verschiebt eine Datei im Dateifenster.

## Befehle

In der Reihenfolge der Menüs. „Baum" und „Dateien" geben an, wo die Taste
funktioniert.

| Taste | Command | Baum | Dateien | Funktion |
|---|---|---|---|---|
| `A` | Available | ja | | Listet die eingehängten Dateisysteme mit Gesamtspeicher, belegtem und freiem Platz auf. [Informationen](info.md) |
| `A` | Attributes | | ja | Ändert die Dateiberechtigungen in Oktalform. [Berechtigungen und Datum](attributes.md) |
| `B` | Branch | ja | ja | Listet die Dateien des Verzeichnisses und aller darunter liegenden Verzeichnisse auf, nachdem der Zweig eingelesen wurde. In Branch oder Showall kehrt die Taste zur reinen Verzeichnisliste zurück. |
| `C` | Compare | ja | | Vergleicht bei geteiltem Bildschirm das Verzeichnis mit dem des anderen Fensters und markiert abweichende Dateien nach dem gewählten Kriterium. [Vergleichen](compare.md) |
| `C` | Copy | | ja | Kopiert die Datei. Die Eingabezeile fragt nach dem Ziel. [Kopieren und Verschieben](copy-move.md) |
| `D` | Delete | ja | ja | Löscht die Datei nach Bestätigung. Im Baum wird das Verzeichnis gelöscht, wenn es leer ist. [Löschen](delete.md) |
| `E` | Edit | | ja | Öffnet die Datei im Editor. [Bearbeiten](execute.md) |
| `F` | Filespec | ja | ja | Legt fest, welche Dateien das Fenster anzeigt, nach Name, Datum und Größe. [Filespec](filespec.md) |
| `I` | Invert | ja | ja | Markiert nicht markierte Dateien und hebt Markierungen markierter Dateien auf: für das Verzeichnis im Baum, für die Liste im Dateifenster. |
| `L` | Log | ja | | Öffnet ein anderes Verzeichnis oder einen entfernten Rechner als Baum dieses Fensters. [Treespec und Log](treespec.md) |
| `M` | Makedir | ja | | Erstellt ein Unterverzeichnis im Verzeichnis unter dem Cursor. |
| `M` | Move | | ja | Verschiebt die Datei. [Kopieren und Verschieben](copy-move.md) |
| `N` | New date | ja | ja | Ändert das Änderungsdatum der Datei oder des Verzeichnisses. [Berechtigungen und Datum](attributes.md) |
| `O` | Oops | ja | | Stellt das zuletzt in den Papierkorb Verschobene wieder her. Im Dateifenster Ctrl+Z. [Löschen](delete.md) |
| `O` | Open | | ja | Öffnet die Datei mit dem Standardprogramm des Systems. [Das Dateifenster](files.md) |
| `P` | Phrase | ja | ja | Öffnet das Suchfeld: listet eingelesene Dateien auf, die die eingegebenen Wörter enthalten. [Das Suchfeld](filespec.md#das-suchfeld-p) |
| `R` | Rename | ja | ja | Benennt die Datei oder das Verzeichnis um. [Umbenennen](rename.md) |
| `S` | Showall | ja | ja | Listet alle eingelesenen Dateien des Fensters auf. |
| `T` | Tag | ja | ja | Dateien: markiert die Datei und rückt nach unten. Baum: markiert die Dateien des Verzeichnisses. [Markierungen und Gruppenoperationen](tagging.md) |
| `U` | Untag | ja | ja | Dateien: hebt die Markierung der Datei auf und rückt nach unten. Baum: hebt die Markierung der Verzeichnisdateien auf. |
| `V` | View | | ja | Öffnet die Datei im Betrachter: Text, Hexadezimal oder Tabelle. [Der Betrachter](viewer.md) |
| `W` | Where | ja | ja | Behält die Dateien in der Liste, die einer SQL-Bedingung entsprechen. [Eine SQL-Bedingung](filespec.md#eine-sql-bedingung-w) |
| `X` | eXecute | ja | ja | Führt einen Shell-Befehl aus. `%f` steht für die Datei, `%t` für die markierten Dateien. [eXecute](execute.md) |
| `Y` | Yank | ja | ja | Kopiert den Pfad der Datei oder des Verzeichnisses in die Zwischenablage. |
| `Q` | Quit | ja | ja | Beendet das Programm nach Bestätigung. [Beenden und Verzeichnis der Shell wechseln](quit.md) |

Die Tasten in der letzten Menüzeile (F1, `*`, `?`, `\`, F3, Enter, F5, F7, F8,
F9) werden unter [Funktionstasten und Symbole](commands-keys.md) beschrieben.

## Tasten außerhalb des Menüs

Dies sind alternative Tasten für Befehle aus der Tabelle.

| Taste | Entspricht |
|---|---|
| Del | `D` |
| Shift+Del | `D`, endgültig löschen ohne Papierkorb |
| F10 | `Q` |
| Ctrl+Insert | `Y` |
| Ctrl+Z | `O` (Oops), in beiden Fenstern |

## In einem Archiv oder auf einem entfernten Rechner

Der Menütitel lautet ARCHIVE oder REMOTE, und Befehle, die dort nicht ausgeführt
werden können, werden nicht aufgeführt.

| Ort | Nicht aufgeführt |
|---|---|
| Archiv | Delete, Edit, Log, Makedir, Move, New date, Oops, Rename, Attributes, eXecute. Copy entpackt. |
| Entfernter Rechner | Edit, Oops, eXecute. Auf entfernten Rechnern gibt es keinen Papierkorb. |

Siehe [Archive](archives.md) und [entfernte Rechner](remote.md).

Siehe auch: [das Strg-Menü](commands-ctrl.md), [das Alt-Menü](commands-alt.md),
[die Menüs](menus.md).
