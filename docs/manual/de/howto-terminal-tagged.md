# Die markierten Dateien in einem Shell-Befehl verwenden

Diese Anleitung führt aus dem integrierten Terminal heraus `wc -c` auf den in
Xylem markierten Dateien aus. Dieselben Schritte funktionieren mit jedem Befehl,
der Dateinamen entgegennimmt.

## Schritte

1. Setzen Sie den Cursor im Baum auf das Verzeichnis, drücken Sie → zum
   Einlesen und Enter, um zu dessen Dateien zu wechseln.
2. Markieren Sie die gewünschten Dateien: `T` auf jeder einzelnen oder Ctrl+T
   für die gesamte Liste.
3. Drücken Sie Alt+X. Ein Terminal öffnet sich im unteren Bildschirmbereich,
   dessen Shell im darüber angezeigten Verzeichnis steht.
4. Tippen Sie den Befehl und ein Leerzeichen ein, `wc -c `, und drücken Sie
   Alt+O. Xylem tippt die Namen der markierten Dateien an der Cursorposition ein,
   jeweils in Anführungszeichen.
5. Drücken Sie Enter.

![Die fünf Schritte: die Dateien von notes, Ctrl+T, Alt+X öffnet das Terminal, wc -c mit Alt+O tippt die zwei Dateinamen und die Ausgabe](../../screenshots/howto-terminal-tagged.gif)

Alt+X kehrt zu Xylem zurück. Die Shell läuft samt Verlauf im Hintergrund weiter,
bis `exit` eingegeben wird.

## Was Alt+O eintippt

Die Namen sind relativ zum Verzeichnis der Shell und in Anführungszeichen
gesetzt, sodass Namen mit Leerzeichen als jeweils ein einziges Argument
übergeben werden:

```
wc -c 'ideas.md' 'todo.md'
```

In einem Branch oder Showall stammen die markierten Dateien aus mehreren
Ordnern; jeder Name erhält dann seinen relativen Pfad ab dem Verzeichnis der
Shell.

## Die Variable

Beim Wechsel in das Terminal setzt Xylem zudem `XY_TAGGED` mit den markierten
Dateien.

| Shell | Verwendung |
|---|---|
| bash | `cp "${XY_TAGGED[@]}" ~/backup` |
| zsh, fish | `cp $XY_TAGGED ~/backup` |
| sh, dash | nicht gesetzt |

Die Variable enthält die Dateien, die beim Betreten des Terminals markiert
waren. Wenn Sie weitere Dateien markieren, wird sie beim nächsten Wechsel in das
Terminal aktualisiert.

## Ein Einzelbefehl mit eXecute

`X` (eXecute) fragt nach einer einzelnen Befehlszeile und führt sie aus. Darin
steht `%t` für die markierten Dateien und `%f` für die Datei unter dem Cursor:

```
tar czf photos.tar.gz %t
```

[eXecute](execute.md) beschreibt die übrigen Platzhalter.

## Wenn das Ergebnis unerwartet ist

- **Die erste Zeile im Terminal lautet ` . '…/sync.sh'`.** Xylem tippt dies
  automatisch ein, um die Shell in das Verzeichnis des Baums zu bewegen und
  `XY_TAGGED` zu setzen. Das führende Leerzeichen verhindert in bash und zsh den
  Eintrag in die Verlaufsdatei.
- **Alt+O tippt nichts ein.** Es sind keine Dateien markiert.
- **Die Shell hat das Verzeichnis nicht gewechselt.** Im Terminal lief ein
  Programm. Xylem sendet das `cd` nur, wenn die Shell an ihrer
  Eingabeaufforderung wartet.
- **F1 öffnet nicht die Hilfe.** Im Terminal geht jede Taste direkt an die Shell,
  außer Alt+X, Alt+Z und Alt+O.

Siehe auch: [das integrierte Terminal](terminal.md), [eXecute](execute.md),
[Markierungen und Gruppenoperationen](tagging.md).
