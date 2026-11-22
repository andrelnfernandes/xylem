# Treespec und Log

## Treespec (`\` oder `/`)

Öffnet eine Eingabezeile mit dem aktuellen Pfad. Während Sie tippen, springt
der Baum-Cursor zum ersten Verzeichnis, das mit Ihrer Eingabe beginnt (ohne
Beachtung der Groß-/Kleinschreibung):

- `/` oder `\` vervollständigen den Namen und gehen eine Ebene tiefer;
- Tab und Shift+Tab vervollständigen und wechseln zum nächsten bzw. vorherigen
  Verzeichnis auf derselben Ebene;
- Backspace geht Ebenen nach oben zurück;
- Ctrl+← nimmt den letzten Ordner vom Pfad (`~/projects/xylem/docs/` wird zu
  `~/projects/xylem/`, mit dem Cursor auf `xylem`), und Ctrl+→ bringt die
  entfernten Ordner einzeln zurück; etwas anderes zu tippen beginnt einen neuen
  Pfad;
- Enter bestätigt und Esc kehrt zum Ausgangspunkt zurück;
- ↑ öffnet die früheren Pfade als [Liste](execute.md#verlauf) (bleiben zwischen Programmstarts erhalten).

Ein Pfad außerhalb des Baums wird beim Bestätigen wie ein Log geöffnet. In
Branch und Showall springt `\` direkt zum Verzeichnis der ausgewählten Datei
und hebt diese hervor.

Um einen Ordner über Teile seines Namens zu finden, auf jeder Ebene und ohne
seinen Pfad zu tippen: [Unscharfe Suche](fuzzy.md) (`|`).

## Log (L)

Öffnet ein anderes Verzeichnis im Fenster und ersetzt den Baum (ein Verzeichnis
innerhalb des aktuellen Baums bewegt lediglich den Cursor dorthin). Eine
Adresse wie `user@host:` öffnet einen [entfernten Rechner](remote.md). Der Pfad
kann absolut, relativ zum aktuellen Verzeichnis sein oder mit `~` beginnen;
**F2** [wählt ihn aus einem Baum aus](picker.md) und ↑ listet frühere Pfade auf.
Neue Fenster behalten die aktuelle [Sortierreihenfolge](sort.md) und den
Anzeigemodus bei.

**Alt+L** macht das Verzeichnis unter dem Cursor sofort zur Wurzel des Baums:
`L` zu einem Verzeichnis innerhalb des Baums bewegt nur den Cursor, Alt+L
verkleinert den Baum dann darauf. Im Dateifenster gilt das angezeigte
Verzeichnis (in Branch und Showall das der ausgewählten Datei). Das Fenster wird
dort neu geöffnet, das Eingelesene und die Markierungen gehen also verloren;
`L ..` öffnet wieder das Verzeichnis darüber. Nicht in Archiven und nicht auf
entfernten Rechnern.

Die Eingabe öffnet sich leer. ↑ öffnet eine [Liste](execute.md#verlauf) der
zuvor benutzten Orte, der letzte ganz unten: die mit Log geöffneten und die
Ordner, in die Dateien kopiert oder verschoben wurden. Tippen filtert sie, Enter
setzt einen Ort in die Zeile und Del entfernt ihn aus der Liste.

**F11** holt ein [Lesezeichen](bookmarks.md) über seinen Namen in die Eingabe,
statt die Adresse eines entfernten Rechners oder Containers einzutippen.

Siehe auch: [das Baumfenster](tree.md), [Lesezeichen](bookmarks.md).
