# Vokabular

Die folgenden Begriffe stammen von den klassischen Baum-Dateimanagern. Die Menüs und dieses
Handbuch verwenden sie in diesen Bedeutungen.

| Begriff | Bedeutung |
|---|---|
| Log | Ein Verzeichnis in den Speicher einlesen: seine Dateien und die Namen seiner Unterverzeichnisse. `→` oder `+` liest ein Verzeichnis ein und `*` einen ganzen Zweig. `L` (Log) ist auch der Befehl, der ein anderes Verzeichnis als Baum öffnet. [Das Baumfenster](tree.md) |
| Logged | In den Speicher eingelesen. Nur eingelesene Dateien erscheinen in Listen, werden in den Statistiken gezählt und können markiert werden. |
| Not logged | Ein Verzeichnis, dessen Name bekannt ist, dessen Inhalt jedoch noch nicht eingelesen wurde. Der Baum kennzeichnet es mit `?`. |
| Release | Einen Zweig mit `-` aus dem Speicher freigeben. Seine Dateien verlassen die Listen und verlieren ihre Markierungen. Auf der Festplatte ändert sich nichts. |
| Relog | Ein Verzeichnis mit F3 erneut von der Festplatte einlesen. |
| Branch | Ein Zweig: ein Verzeichnis und alles darunter. `B` listet die Dateien des Zweigs in einer einzigen Liste auf. |
| Showall | Die Liste aller im Fenster eingelesenen Dateien, geöffnet mit `S`. |
| Tag | Eine Markierung an einer Datei (`■`). Die Ctrl-Befehle wirken auf die markierten Dateien. Markierungen bleiben im Speicher und gehen bei Release oder beim Beenden verloren. [Markierungen und Gruppenoperationen](tagging.md) |
| Filespec | Der Filter, welche Dateien das Fenster anzeigt, nach Name, Datum und Größe. Festgelegt mit `F`. [Filespec](filespec.md) |
| Treespec | Ein eingegebener Pfad, um den Baum-Cursor zu bewegen. Geöffnet mit `\` oder `/`. [Treespec](treespec.md) |
| Fenster | Ein Baum mit seinem Dateifenster. F8 öffnet ein zweites daneben. [Geteilter Bildschirm](split.md) |
| Wurzel | Das Verzeichnis an der Spitze des Baums. `L` ändert es, und Alt+L macht das Verzeichnis unter dem Cursor zur Wurzel. |
| Matching | Die eingelesenen Dateien, die dem Filespec entsprechen. |
| Maske | Ein Muster, das einen Namen oder eine Zeile pro Datei erzeugt: `*.bak`, `foto_<001>.*`. Verwendet von Rename und Batch. [Umbenennen](rename.md) |
| Batch | Ctrl+B: schreibt eine Datei mit einer Zeile pro markierter Datei oder pro Datensatz einer Tabelle im Betrachter. [Batch](execute.md) |
| Prune | Alt+P: löscht ein Verzeichnis samt allem, was sich darin befindet. [Löschen](delete.md) |
| Oops | Stellt das zuletzt in den Papierkorb Verschobene wieder her. `O` im Baum, Ctrl+Z in beiden Fenstern. |
| Yank | `Y`: kopiert einen Pfad in die Zwischenablage. |
| Autoview | F7: das Dateifenster mit dem Inhalt der Datei unter dem Cursor neben der Liste. |
| eXecute | `X`: führt einen Shell-Befehl aus, mit `%f` für die Datei und `%t` für die markierten Dateien. [eXecute](execute.md) |
| Avail | `A` im Baum: die eingehängten Dateisysteme mit ihrem freien Speicherplatz. |

Siehe auch: [was Xylem ist](overview.md), [die Menüs](menus.md).
