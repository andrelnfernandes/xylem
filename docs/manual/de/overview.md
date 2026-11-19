# Was Xylem ist

Xylem ist ein Dateimanager für das Terminal. Bildschirmaufbau und Tastenkürzel
orientieren sich an XTreeGold und ZTreeWin: ein Verzeichnisbaum oben, die
Dateien des Verzeichnisses unter dem Cursor darunter, Statistiken rechts und ein
Menü mit Ein-Buchstaben-Befehlen in der Fußzeile. Der Befehl lautet `xy`.

## Einlesen von Verzeichnissen (Logging)

Xylem liest ein Verzeichnis in den Speicher ein, wenn Sie es anfordern, und
behält es dort. Das nennt man Einlesen (*logging*). `→` liest ein Verzeichnis
ein, `*` liest den gesamten Zweig unter dem Cursor ein, und `-` gibt einen
Zweig aus dem Speicher frei.

Zwei Befehle listen eingelesene Dateien aus mehreren Verzeichnissen gleichzeitig
auf: `B` (Branch) listet die Dateien des Verzeichnisses und aller darunter
liegenden Verzeichnisse auf, und `S` (Showall) listet alle eingelesenen Dateien
auf. In diesen Listen markieren Sie Dateien mit `T`, und die Ctrl-Befehle wirken
dann auf die markierten Dateien: Ctrl+C kopiert sie, Ctrl+D löscht sie, Ctrl+R
benennt sie mit einer Maske um.

Dateien in einem nicht eingelesenen Verzeichnis erscheinen in keiner Liste und
werden von keinem Befehl berührt.

## Was die Befehle abdecken

| Aufgabe | Tastenkürzel | Thema |
|---|---|---|
| Kopieren oder Verschieben, mit oder ohne Quellpfade | `C`, `M`, Ctrl+C, Alt+C | [Kopieren und Verschieben](copy-move.md) |
| Löschen und Wiederherstellen aus dem Papierkorb | `D`, Ctrl+Z | [Löschen](delete.md) |
| Eine oder mehrere Dateien mit Maske umbenennen | `R`, Ctrl+R | [Umbenennen](rename.md) |
| Bestimmte Dateien nach Name, Datum oder Größe filtern | `F` | [Filespec](filespec.md) |
| Zwei Verzeichnisse vergleichen | `C` im Baum | [Vergleichen](compare.md) |
| Datei als Text, Hexadezimal oder Tabelle ansehen | `V` | [Der Betrachter](viewer.md) |
| ZIP, 7z und TAR wie Verzeichnisse öffnen | Alt+F5 | [Archive](archives.md) |

[Die Menüs](menus.md) erklärt die Anordnung der Tasten und
[die Befehle](commands.md) listet alle auf.

[Vergleich mit ZTreeWin](ztreewin.md) listet die ZTreeWin-Befehle auf, die Xylem
nicht besitzt, sowie abweichende Tastenkürzel.

## Nicht vertrauenswürdige Dateien

Xylem liest jede Datei so ein, als wäre sie darauf ausgelegt, das Programm
anzugreifen. Das Anzeigen einer Datei führt nichts daraus aus, das Entpacken
eines Archivs schreibt nicht außerhalb des Zielorts und jeder Leser stoppt an
einem Größenlimit. [Sicherheit](security.md) listet die Limits auf und was nicht
abgedeckt ist.
