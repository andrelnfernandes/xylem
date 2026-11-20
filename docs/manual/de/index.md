# Xylem-Handbuch

Xylem ist ein Dateimanager für das Terminal, der mit der Tastatur bedient wird:
oben ein Verzeichnisbaum, darunter die Dateien des aktuellen
Verzeichnisses und rechts die Statistiken. Er liest Verzeichnisse in den
Speicher ein, sodass eine einzige Liste Dateien eines ganzen Zweigs vereinen
kann und ein einziger Befehl alle markierten Dateien kopiert, verschiebt, löscht,
vergleicht oder umbenennt.

Xylem behandelt Dateien, die es nicht selbst erstellt hat, als potenziell
feindlich: Weder der Inhalt noch der Name einer Datei darf dazu führen, dass
Xylem abstürzt, einfriert, Code ausführt, außerhalb des Zielorts schreibt oder
den Speicher erschöpft. [Sicherheit](security.md) beschreibt die Maßnahmen und
was nicht abgedeckt ist.

**F1** (oder **Alt+H** für Terminals, die F1 selbst belegen) öffnet die Hilfe zu
dem, was auf dem Bildschirm zu sehen ist: in einer Eingabezeile das Thema
dieser Eingabezeile; im Baum- oder Dateifenster das Thema dieses Fensters.

| Taste in der Hilfe | Funktion |
|---|---|
| Tab, Shift+Tab | wählt den nächsten oder vorherigen Link aus |
| Enter | folgt dem Link |
| Backspace | geht zurück |
| `I` | dieser Index |
| `K` | die gültigen Tasten |
| `L` | wechselt die Sprache |
| Esc | schließt die Hilfe |

Die spanische, französische, deutsche und chinesische Fassung dieses Handbuchs
wurden nicht von Muttersprachlern geprüft. Korrekturen:
`https://github.com/andrelnfernandes/xylem/issues`.

## Hier starten

- [Was Xylem ist](overview.md): Einlesen, Markierungen und was die Befehle abdecken
- [Installieren und die ersten fünf Minuten](start.md)
- [Der Bildschirm](screen.md): Baum, Dateifenster, Statistiken, Menüleiste
- [Die Menüs, Ctrl und Alt](menus.md): Buchstabe allein, mit Ctrl, mit Alt, und F9
- [Die Maus](mouse.md): Einschalten in F12, das Mausrad, Klicks und die Fußzeile
- [Vokabular](glossary.md): Log, Branch, Tag, Filespec

## Befehle

- [Die Menüs DIR und FILE](commands.md): die einzelnen Buchstaben
- [Das Ctrl-Menü](commands-ctrl.md): die markierten Dateien
- [Das Alt-Menü](commands-alt.md): der Zweig, die Pfade und die Werkzeuge
- [Git](git.md): das Git-Menü (Ctrl+G)
- [Funktionstasten und Symbole](commands-keys.md)
- [Die gültigen Tasten](keys.md): mit den Änderungen aus der Konfiguration

## Fenster und Ansichten

- [Das Baumfenster](tree.md): Navigation, Einlesen, Branch und Showall
- [Das Dateifenster](files.md): Anzeigemodi, Autoview, Dateien öffnen
- [Geteilter Bildschirm](split.md): zwei Fenster nebeneinander
- [Der Betrachter](viewer.md): Text, Hex und Tabellen, Suche, Syntaxfarben
- [Zeilen filtern](filter.md): `&` im Betrachter, mit einer Anleitung zu regulären Ausdrücken
- [Markierte Dateien durchsuchen](search.md): Ctrl+S und Ctrl+V
- [Informationen](info.md): Info, Avail, Statistiken und Verzeichnisgrößen
- [Speicherplatzbelegung](usage.md): wohin der Platz geht (Alt+U)
- [Duplikate](duplicates.md): Dateien und Ordner mit gleichem Inhalt (Alt+D)

## Arbeiten mit Dateien

- [Dateien und Ordner finden](finding.md): welche Suche welche Frage beantwortet
- [Markierungen und Gruppenoperationen](tagging.md)
- [Kopieren und Verschieben](copy-move.md), [Ordner auswählen](picker.md)
- [Löschen, Papierkorb und Prune](delete.md)
- [Umbenennen und Stapelumbenennung](rename.md)
- [Berechtigungen und Datum](attributes.md)
- [Filespec: der Dateifilter](filespec.md)
- [Treespec und Log](treespec.md)
- [Unscharfe Suche](fuzzy.md): ein Ordner oder eine Datei nach Teilen des Namens (`|`)
- [Vergleichen](compare.md) und [Sortieren](sort.md)
- [Bearbeiten, Ausführen und Batch](execute.md)
- [Archive](archives.md): ZIP, 7z und TAR

## Weitere Werkzeuge

- [Das integrierte Terminal](terminal.md)
- [Entfernte Rechner (SSH)](remote.md): ein entfernter Rechner oder Container als Baum
- [Cloud-Speicher (rclone)](cloud.md): Google Drive, S3 und andere
- [Dateien mit SQL finden](sql.md): eine Bedingung über die Dateien und ihren Inhalt (W), und die Wörter darin (P)
- [Tabellen](tables.md): Tabellenkalkulationen, CSV, dBase, SQLite und Parquet als Raster, mit SQL-Filter und jedem Datensatz als Formular
- [Lesezeichen](bookmarks.md)

## Anleitungen

- [Dateien aus mehreren Ordnern mit ihren Pfaden kopieren](howto-copy-with-paths.md)
- [Viele Dateien mit einem Zähler umbenennen](howto-rename-mask.md)
- [Dateien mit einer SQL-Bedingung auswählen](howto-sql-filter.md)
- [Duplikate finden und eine Kopie behalten](howto-duplicates.md)
- [Finden, was die Platte füllt](howto-disk-space.md)
- [Eine CSV-Datei oder Tabelle mit SQL filtern](howto-table-sql.md)
- [Dateien von einem anderen Rechner per SSH kopieren](howto-ssh-copy.md)
- [Die markierten Dateien in einem Shell-Befehl verwenden](howto-terminal-tagged.md)

## Referenz

- [Konfiguration](configuration.md): der F12-Bildschirm, Themen, Tasten
- [Die Befehlszeile](cli.md)
- [Terminals und Tasten](terminals.md): F1, Ctrl+M, das Kitty-Tastaturprotokoll
- [Sicherheit](security.md)
- [Beenden und Verzeichnis der Shell wechseln](quit.md)
- [Windows](windows.md)
- [Vergleiche](comparison.md): andere Dateimanager und ein Dankeschön
- [Versionshinweise](releases.md)
