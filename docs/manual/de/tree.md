# Das Baumfenster

Der Baum zeigt die Verzeichnisse an, die eingelesen (in den Speicher geladen)
wurden. Das Dateifenster darunter listet die Dateien des Verzeichnisses unter
dem Cursor auf.

## Navigation

| Taste | Funktion |
|---|---|
| ↑ ↓ PgUp PgDn Home End | Cursor bewegen |
| → oder `+` | Verzeichnis einlesen, falls noch nicht geschehen, und aufklappen: seine Dateien erscheinen, und seine Unterverzeichnisse werden aufgelistet, noch nicht eingelesen |
| ← | Verzeichnis zuklappen (oder zum übergeordneten Verzeichnis) |
| `-` | Zweig aus dem Speicher freigeben: alles darin Eingelesene wird verworfen, samt Markierungen, und das Verzeichnis bleibt nicht eingelesen |
| Backspace | zum übergeordneten Verzeichnis wechseln |
| Space | eine Zeile nach unten |
| Shift+Buchstabe, 0–9 | nächstes Verzeichnis mit diesem Anfangsbuchstaben |
| `\` oder `/` | [Treespec](treespec.md): Verzeichnis durch Eingabe des Pfads anspringen |
| Enter oder Tab | zum Dateifenster wechseln (Tab wechselt das Fenster bei [geteiltem Bildschirm](split.md)) |

Im Baum kennzeichnet ein `+` an der Verbindungslinie (`├─+`) ein zugeklapptes
Verzeichnis, das Unterverzeichnisse enthält, und ein `?` (`├─?`) ein noch nicht
eingelesenes, bei dem unbekannt ist, ob es Unterverzeichnisse hat.

## Einlesen (Logging)

Ein Verzeichnis wird nur eingelesen, wenn Sie es verlangen. Steht der Cursor auf
einem nicht eingelesenen, zeigt das Dateifenster `Not logged`, und → oder `+`
liest es ein. Beim Start und mit `L` wird das Stammverzeichnis eingelesen und
aufgeklappt, seine Unterverzeichnisse nicht. Das Springen zu einem Verzeichnis
über seinen Pfad ([Treespec](treespec.md)) liest es ebenfalls ein.

`*` liest den gesamten Zweig unter dem Cursor ein und klappt ihn auf; bei einem
längeren Einlesevorgang wird der Fortschritt in der Fußzeile angezeigt und `Esc`
bricht ab. **F3** (oder F2) liest das Verzeichnis neu von der Festplatte ein.
Xylem überwacht außerdem das aktuelle Verzeichnis jedes Fensters und liest es
automatisch neu ein, wenn ein anderes Programm es ändert (höchstens einmal pro
halbe Sekunde, nach Abschluss laufender Operationen); andere Verzeichnisse
benötigen weiterhin F3.

`L` ([Log](treespec.md)) öffnet ein anderes Verzeichnis im Fenster.

## Dateilisten

- `B` (Branch): die Dateien des gesamten Zweigs, nachdem dieser eingelesen wurde.
- `S` (Showall): die Dateien aller eingelesenen Verzeichnisse.
- Ctrl+S: Showall nur der markierten Dateien (der Tags-Only-Modus von
  ZTreeWin). Dort entmarkierte Dateien bleiben in der Liste, bis sie neu
  eingelesen wird (F3); ohne Markierungen erscheinen wieder alle Dateien.
- `F` ([Filespec](filespec.md)): welche Dateien nach Name, Datum und Größe angezeigt werden.

In Branch und Showall springt `\` zum Verzeichnis der ausgewählten Datei. Ein erneutes `B`
führt zurück zur Liste des Verzeichnisses selbst.

## Befehle im Baum

| Taste | Funktion |
|---|---|
| `T` / `U` / `I` | Dateien des Verzeichnisses markieren / Markierung aufheben / umkehren ([Markierungen](tagging.md)) |
| `M` | Unterverzeichnis erstellen |
| `R` | Verzeichnis umbenennen |
| `D` oder Del | Verzeichnis löschen, falls leer ([Löschen](delete.md)) |
| Alt+P | Prune: Verzeichnis samt gesamtem Inhalt löschen |
| `O` oder Ctrl+Z | Oops: zuletzt in den Papierkorb verschobenes Element wiederherstellen |
| `N` | Datum des Verzeichnisses ändern ([Datum](attributes.md)) |
| `C` / Alt+C | [Vergleichen](compare.md) mit dem anderen Fenster / dem gesamten Zweig |
| `A` | Avail: eingehängte Dateisysteme ([Informationen](info.md)) |
| Alt+I | Info: Details zum Verzeichnis |
| Alt+K | Größen, Dateianzahl oder Prozentsätze neben jedem Verzeichnis |
| `Y` | Pfad des Verzeichnisses in die Zwischenablage kopieren |
| `X` | Shell-Befehl [ausführen](execute.md) |
| Alt+X | das [integrierte Terminal](terminal.md) |
| F8 | [geteilter Bildschirm](split.md) |
| F11 / Alt+B | [Lesezeichen](bookmarks.md); Ctrl+B speichert eines |

Siehe auch: [das Dateifenster](files.md), [die aktiven Tasten](keys.md).
