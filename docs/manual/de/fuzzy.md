# Unscharfe Suche (|)

`|` findet einen Ordner oder eine Datei der Liste über Teile ihres Namens, auf
jeder Ebene des Baums: `cfgxy` findet `config/xylem`, `rdme` findet
`README.md`. Die Ergebnisse erscheinen über der Eingabezeile, während Sie
tippen, die besten zuerst, und der Cursor des Fensters folgt dem gewählten.

Um zu einem Pfad zu gehen, den Sie schon kennen, Ebene für Ebene, ist
[Treespec](treespec.md) schneller. Um viele Dateien nach Name, Datum oder
Größe oder nach ihrem Inhalt auszuwählen, siehe [Dateien und Ordner
finden](finding.md).

## Was durchsucht wird

Was `|` durchsucht, hängt von dem Fenster ab, in dem es geöffnet wird:

- **Im Baum:** die eingelesenen Ordner, nach ihrem Pfad ab der Wurzel des
  Baums (`docs plan` findet `docs/plans`). Versteckte Ordner und was darin
  liegt werden nur durchsucht, solange der Baum sie zeigt.
- **Im Dateifenster:** die Dateien der Liste, so wie sie ist, mit der
  [Filespec](filespec.md) und der [SQL-Bedingung](sql.md) (`W`), die gelten.
  Im Verzeichnis nach dem Namen; in Branch und Showall auch nach dem Pfad
  (`src fzrs` findet `src/fuzzy.rs`).

Um eine Datei irgendwo im Baum zu finden, öffnen Sie Showall (`S`) und dann
`|`.

Nur Eingelesenes wird durchsucht, und nichts wird von der Festplatte oder aus
dem Netzwerk gelesen: Es funktioniert genauso auf einem [entfernten
Rechner](remote.md), in einem [Archiv](archives.md) und auf einem
[Cloud-Mount](cloud.md).

## Wie verglichen wird

- **Die Buchstaben in ihrer Reihenfolge**, nicht unbedingt zusammen: `rdme`
  findet `README.md`.
- **Leerzeichen trennen Begriffe**, und jeder Begriff muss passen, in
  beliebiger Reihenfolge: `xy docs` findet `xylem/docs` und `docs/xylem`.
- **Groß- und Kleinschreibung** zählt nicht, außer wenn der Begriff einen
  Großbuchstaben hat: `Rd` findet `readme` nicht. Jeder Begriff entscheidet
  für sich.
- **Die besten zuerst:** Buchstaben beisammen zählen mehr als getrennte; ein
  Buchstabe am Anfang eines Worts (am Anfang, nach `/`, `_`, `-`, `.` oder
  einem Leerzeichen, oder wo Klein- zu Großschreibung wechselt, wie in
  `fileSpec`) zählt mehr als einer in der Mitte; ein Treffer im Namen zählt
  mehr als einer in den Ordnern darüber. Bei Gleichstand kommt der kürzere
  Pfad zuerst, dann die Reihenfolge des Baums oder der Liste.

Die getroffenen Buchstaben erscheinen in der Tastenfarbe. Ein Name, der für
das Fenster zu lang ist, wird links mit `…` gekürzt, damit sein Ende sichtbar
bleibt.

## Auf dem Bildschirm

Der Zähler oben rechts sagt, wie die Suche vorankommt:

| Er sagt | Bedeutung |
|---|---|
| `type pieces of a name` | die Eingabezeile ist leer |
| `120 found · searching 40%` | die Suche geht noch durch den Baum; die Liste füllt sich |
| `120` | fertig: jeder Treffer steht in der Liste |
| `500 best of 15,894` | fertig: die Liste behält die 500 besten |
| `nothing matches` | fertig, ohne Treffer |

Ein großer Baum wird schrittweise durchsucht, während Sie tippen, sodass die
ersten Ergebnisse sofort kommen und der Rest folgt. Wenn Sie am Ende
Buchstaben hinzufügen, sucht die Suche nur unter dem, was schon getroffen
hat. Ändert sich der Baum, während die Eingabezeile offen ist (ein Ordner
eingelesen, freigegeben oder neu gelesen), beginnt die Suche von vorn.

Ein `*` vor einer Datei bedeutet, dass sie markiert ist; nach einem Ordner
sagt `3*`, wie viele seiner Dateien markiert sind.

## Tasten

| Taste | Funktion |
|---|---|
| ↑ ↓ | das vorige oder nächste Ergebnis wählen; der Cursor des Fensters folgt |
| PgUp PgDn, Home End | eine Seite nach oben oder unten, das erste oder das letzte Ergebnis wählen |
| Enter | bleibt, wo der Cursor ist, und schließt die Eingabezeile |
| Esc | geht zurück, wo der Cursor war, und schließt die Eingabezeile |
| Ctrl+T | markiert das gewählte Ergebnis, oder hebt die Markierung auf, wenn es markiert ist, und wählt das nächste |
| Alt+T | markiert die Dateien der Ergebnisse in der Liste |
| Ctrl+U | leert die Eingabezeile |

Mit eingeschalteter [Maus](mouse.md) wählt ein Klick ein Ergebnis, und ein
Doppelklick geht dorthin.

## Aus den Ergebnissen markieren

**Ctrl+T** auf einer Datei markiert sie; auf einem Ordner markiert es die
Dateien dieses Ordners, die das Dateifenster zeigen würde (mit der Filespec
und `W`, die gelten). Sind sie alle schon markiert, hebt es ihre Markierung
auf. Die Wahl geht dann zum nächsten Ergebnis, sodass Ctrl+T, Ctrl+T, Ctrl+T
die Ergebnisse eines nach dem anderen markiert.

**Alt+T** markiert die Dateien jedes Ergebnisses in der Liste: höchstens die
500, die gezeigt werden. Wenn mehr getroffen haben, sagt der Hinweis unter
der Liste, wie viele ausgelassen wurden. Alt+T auf einem nicht eingelesenen
Ordner markiert dort nichts, und der Hinweis sagt, wie viele übersprungen
wurden.

Die Markierungen bleiben nach dem Schließen der Eingabezeile, wie jede
Markierung: Ctrl+C, Ctrl+M, Ctrl+D und Batch wirken dann auf sie. Siehe
[Markierungen](tagging.md).

Jeden Treffer zu markieren, nicht nur die 500 der Liste, ist eine Option,
standardmäßig aus: Alt+T könnte dann tausende Dateien markieren, die Sie
nicht gesehen haben. Lesen Sie [Jeden Treffer der unscharfen Suche
markieren](configuration.md#tagging-every-match-of-the-fuzzy-find), bevor
Sie sie einschalten.
