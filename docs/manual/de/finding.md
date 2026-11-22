# Dateien und Ordner finden

Xylem hat mehrere Wege, Dinge zu finden. Jeder beantwortet eine andere Frage,
und sie fallen in drei Gruppen: an einen Ort gehen, die Liste filtern und in
den Dateien suchen.

| Sie wollen | Taste | Was Sie bekommen |
|---|---|---|
| *Diesen* Ordner, über Teile seines Namens | `\|` im Baum | der Baum-Cursor geht dorthin ([Unscharfe Suche](fuzzy.md)) |
| *Diese* Datei der Liste, über Teile ihres Namens | `\|` im Dateifenster | der Datei-Cursor geht dorthin ([Unscharfe Suche](fuzzy.md)) |
| Einen Pfad, den Sie kennen, Ebene für Ebene | `\` oder `/` | der Baum-Cursor folgt dem, was Sie tippen ([Treespec](treespec.md)) |
| Den nächsten Namen, der mit einem Buchstaben beginnt | Shift+Buchstabe | der Cursor geht dorthin ([Das Baumfenster](tree.md)) |
| Die Dateien mit diesem Namen, Datum oder dieser Größe | `F` | die Liste zeigt nur sie, bis zur nächsten Änderung ([Filespec](filespec.md)) |
| Die Dateien, die eine Bedingung erfüllen | `W` | die Liste zeigt nur sie, bis zur nächsten Änderung ([SQL](sql.md)) |
| Die Dateien mit diesen Wörtern darin | `P` | eine Liste von Ergebnissen, die besten zuerst ([Das Suchfeld](sql.md#the-search-box-p)) |
| Die markierten Dateien mit diesem Text darin | Ctrl+S | die anderen verlieren ihre Markierung ([Markierte Dateien durchsuchen](search.md)) |

## An einen Ort gehen

**Unscharfe Suche** (`|`), **Treespec** (`\`, `/`) und **Shift+Buchstabe**
bewegen den Cursor und ändern sonst nichts. Nehmen Sie `|`, wenn Sie sich an
ein Stück des Namens erinnern, aber nicht daran, wo er ist; Treespec, wenn Sie
den Pfad kennen; Shift+Buchstabe, um im aktuellen Ordner zu springen.

## Die Liste filtern

**Filespec** (`F`) und **W** bestimmen, welche Dateien das Dateifenster
zeigt, und der Filter bleibt, bis Sie ihn ändern: Der Titel des Fensters
zeigt ihn. `F` ist für Namen, Daten und Größen; `W` für alles, was Xylem von
einer Datei weiß, einschließlich ihres Inhalts (`contains`, `words`). Sie
gelten zusammen, und Ctrl+T markiert dann alle Dateien, die durchgekommen
sind.

Um im ganzen Baum zu suchen, öffnen Sie zuerst Branch (`B`) oder Showall
(`S`): Die Filter, und `|`, wirken dann auf jede eingelesene Datei davon.

## In den Dateien suchen

**Das Suchfeld** (`P`) findet Dateien über die Wörter darin, während Sie
tippen, und ist sofort fertig bei einem mit Alt+N indizierten Zweig.
**Ctrl+S** durchsucht die markierten Dateien nach einem Stück Text und hebt
die Markierung derer ohne ihn auf, und Ctrl+V zeigt dann jeden Treffer. In
`W` tun `contains('text')` und `words('a b')` dasselbe als Teil einer
Bedingung.

## Markieren, was gefunden wurde

Jeder dieser Wege endet in Markierungen, wenn Sie auf viele Dateien wirken
wollen:

- `|` und `P`: Ctrl+T markiert das gewählte Ergebnis; in `|` markiert Alt+T
  die Liste.
- `F` und `W`: Ctrl+T im Baum markiert jede Datei, die durchgekommen ist.
- Ctrl+S lässt nur die Dateien markiert, die den Text enthalten.

Dann wirken Ctrl+C, Ctrl+M, Ctrl+D und Batch auf die markierten Dateien.
Siehe [Markierungen](tagging.md).
