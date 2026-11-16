# Tabellen: Tabellenkalkulation, CSV und Datenbanken

Manche Dateien öffnen sich als Raster statt als Text. Xylem richtet sich nach der Endung, und nur bei unkomprimierten Dateien auf dieser Festplatte: Eine Datei auf einem entfernten Rechner oder in einem Archiv erscheint als gewöhnlicher Inhalt, mit dem Hinweis `Tables are shown only for files on this disk`.

| Format | Endungen | Spaltennamen |
|---|---|---|
| CSV, TSV | `.csv` `.tsv` `.tab` | die erste Zeile, wenn mindestens die Hälfte davon keine Zahlen sind; sonst A, B, C... |
| JSON Lines | `.jsonl` `.ndjson` | die Schlüssel der Objekte |
| Tabellenkalkulationen | `.xlsx` `.xls` `.xlsb` `.ods` | A, B, C... (die erste Zeile ist eine Zeile wie jede andere) |
| dBase, Clipper, FoxPro | `.dbf`, mit der Memodatei `.dbt` | die Feldnamen |
| SQLite | `.db` `.sqlite` `.sqlite3` | die Spalten der Tabelle |
| Parquet | `.parquet` | die Spalten der Datei |

Das CSV-Trennzeichen (Komma, Semikolon, Tabulator oder senkrechter Strich) wird am Anfang der Datei erkannt.

| Taste | Funktion |
|---|---|
| ↑ ↓ | vorherige oder nächste Zeile |
| ← → | vorherige oder nächste Spalte |
| PgUp PgDn | eine Seite Zeilen nach oben oder unten |
| Home End | erste oder letzte Spalte |
| Ctrl+Home Ctrl+End | erste oder letzte Zeile; unter einem Filter von den Zeilen, die er behält |
| `[` `]` | vorheriges oder nächstes Blatt einer Tabellenkalkulation oder vorherige oder nächste Tabelle einer SQLite-Datenbank, im Kreis; das löscht den Filter |
| `Tab` | die Liste der Blätter oder Tabellen: Teil eines Namens tippen grenzt sie ein, Enter öffnet einen Eintrag und Esc geht zurück; ein anderes Blatt zu öffnen löscht den Filter. In einer Datenbank stehen Views nach den Tabellen, markiert mit `(view)` |
| `g` oder `:` | zu einer Zeile über ihre Nummer in der Datei springen; hat der Filter sie ausgeblendet, zur nächsten behaltenen |
| `\` oder `&` | der SQL-Filter |
| `m` oder Insert | markiert den Datensatz unter dem Cursor oder hebt die Markierung auf und geht zum nächsten; **Ctrl+B** schreibt dann nur die markierten |
| `M` | löscht die Markierungen |
| `i` | die Struktur der Tabelle: was die Datei über jede Spalte sagt (siehe unten) |
| Enter | der Datensatz unter dem Cursor als Formular, ein Feld pro Zeile: ↑ ↓ wählen das Feld, ← → gehen zum vorigen oder nächsten Datensatz, `c` kopiert den ganzen Datensatz (`NAME : Wert`, eine Zeile pro Feld) und `y` den Wert des Feldes, zum Einfügen in einen Editor oder ein Terminal; Esc geht zurück. Kopiert wird, was der Bildschirm zeigt: Steuer- und Textrichtungszeichen, auch Zeilenumbrüche, werden zu `·`, sodass das Einfügen in ein Terminal nichts ausführt. Außerhalb der Tabelle schließt Enter den Viewer |
| `T` | zum Text der Datei und zurück |
| `H` | von der Tabelle zum Text, dann Hex, dann zurück zur Tabelle |

Der Rahmen zeigt `TABLE [2/5: Name]`, wenn es mehrere Blätter oder Tabellen gibt, und die untere Zeile die Position des Cursors: `Cell B7 (Row 7 of 120)`. Mit Filter behalten die Zeilen ihre Nummer aus der Datei, und diese Zeile nennt zusätzlich den Treffer: `Cell A13 (Row 13 of 31, match 2 of 8)`. Die letzte Zeile des Rasters enthält den ganzen Zellinhalt und den aktiven Filter. Die Suche (`F`, `B`, `/`) schaut nicht in das Raster und findet dort nichts; sie funktioniert im Textmodus.

Markierungen bleiben unter einem Filter erhalten, und Batch schreibt, was markiert ist und vom Filter behalten wird. Sie gehören zum geöffneten Blatt oder zur Tabelle und gehen beim Wechsel verloren. Ändert sich die Datei nach dem Markieren, verweigert Batch, da an diesen Stellen andere Datensätze stehen können.

## Die Struktur

`i` zeigt über dem Raster die Spalten der Reihe nach mit dem, was die Datei über jede sagt, wie `DISPLAY STRUCTURE` in dBase und Clipper. Pfeile, PgUp, PgDn, Home und End scrollen; Esc, Enter oder `i` schließen sie.

| Format | Was sie zeigt |
|---|---|
| dBase, Clipper, FoxPro | Name, Typ (`C` Zeichen, `N` numerisch, `D` Datum, `L` logisch, `M` Memo, `F` Gleitkomma, `I` Ganzzahl, `B` Double, `Y` Währung, `T` Datum und Uhrzeit), Breite und Dezimalstellen; darüber die Anzahl der Datensätze und das Datum der letzten Änderung aus dem Kopf sowie die Länge eines Datensatzes |
| SQLite | den deklarierten Typ jeder Spalte, `Not null`, den Standardwert und die Stelle im Primärschlüssel; darüber, ob es eine Tabelle oder eine Sicht ist, und ihre Indizes mit ihren Spalten (höchstens 32). Eine Sicht zeigt die Spalten, die sie liefert; eine ganze Abfrage die ihres Ergebnisses, mit dem Typ jeder Spalte, die direkt aus einer Tabellenspalte kommt |
| Parquet | den Arrow-Typ jeder Spalte und ob sie Nullwerte erlaubt; darüber die Zeilen und Zeilengruppen, die die Datei angibt, und das Programm, das sie geschrieben hat |
| CSV, JSON Lines, Tabellenkalkulationen | sie deklarieren keine Typen: was die gelesenen Zeilen in jeder Spalte haben (Ganzzahl, Dezimalzahl, logisch, Text, leer) und den breitesten Wert |

Namen, Typen und Standardwerte kommen aus der Datei und werden wie die Zellen bereinigt: Steuer- und Textrichtungszeichen erscheinen als `·`. Eine `.dbf`, deren Kopf sich widerspricht, zeigt `?` bei den Dezimalstellen.

## Der SQL-Filter

`\` oder `&` fragt nach einer Bedingung, wie beim `WHERE` von SQL (das Wort `WHERE` ist optional), und zeigt nur die Zeilen, die sie erfüllen. Die Eingabe beginnt mit dem aktuellen Filter; lassen Sie sie leer, um wieder alle Zeilen zu sehen. Ein Hinweis meldet `Filter matched N rows` oder `SQL error: ...`; nach einem Fehler bleibt der vorherige Filter aktiv.

```
age >= 30 AND name LIKE 'Ali%'
WHERE status IN ('open', 'late') OR total > 1000.50
NOT (city = 'Rio' OR city IS NULL)
B ILIKE '%silva%'
```

| Operator | Funktion |
|---|---|
| `=` `!=` `<>` `<` `<=` `>` `>=` | vergleichen |
| `AND` `OR` `NOT` und Klammern | verknüpfen Bedingungen (`AND` vor `OR`) |
| `LIKE` `ILIKE` | passen auf ein Muster, `%` für beliebigen Text und `_` für ein Zeichen; `ILIKE` ignoriert die Groß- und Kleinschreibung |
| `IN (...)` `NOT IN (...)` | steht, oder steht nicht, in der Liste |
| `IS NULL` `IS NOT NULL` | die Zelle ist, oder ist nicht, leer |

Schlüsselwörter können groß oder klein geschrieben werden.

- **Spalten:** der Name der Spalte, ohne Anführungszeichen und in beliebiger Schreibung, oder ihr Buchstabe von `A` bis `Z`. Mit dem Buchstaben erreichen Sie einen Namen mit Leerzeichen oder Symbolen. Eine Spalte, die es nicht gibt, gilt ohne Fehler als leer.
- **Werte:** Text in einfachen oder doppelten Anführungszeichen (`\'` für ein Anführungszeichen darin), Zahlen, `TRUE`, `FALSE` und `NULL`. Was in Anführungszeichen steht, ist immer Text, nie eine Spalte.
- **Zellen:** Der Typ ergibt sich aus dem Aussehen der Zelle. Leer ist `NULL`, `true` und `false` sind Wahrheitswerte, Zahlen sind Zahlen und der Rest ist Text. Schreiben Sie also `code = 7`, nicht `code = '7'`: Text und Zahl sind nie gleich, und eine Zelle mit `007` ist die Zahl 7. Textvergleiche unterscheiden Groß- und Kleinschreibung.
- **Größe:** Ein Filter hat höchstens 1.024 Byte und 128 Terme.

## Eine ganze Abfrage in einer Datenbank

In einer SQLite-Datenbank nimmt dieselbe Eingabe auch eine ganze Abfrage an, die mit `SELECT` oder `WITH` beginnt, mit allem, was SQLite kann: `JOIN`, `GROUP BY`, `ORDER BY`, Funktionen.

```
SELECT c.name, sum(o.price) FROM customers c JOIN orders o ON o.code = c.code GROUP BY c.name
```

Der Rahmen zeigt `QUERY:`, und das Raster das Ergebnis. Eine Bedingung filtert dann diese Zeilen, und **Ctrl+B** schreibt alles, was die Abfrage liefert. Eine leere Eingabe löscht den Filter und führt danach zurück zur Tabelle; auch `Tab`, `[` und `]` führen zu einer Tabelle. Eine Abfrage liest nur, und nur eine auf einmal: Alles andere wird abgelehnt, und ein Fehler lässt den Bildschirm unverändert. Die Grenzen unten gelten, also stoppt eine langsame Abfrage nach 3 s und zeigt die bisherigen Zeilen; ein Batch hat diese Grenze nicht. Xylem ändert keine Datenbanken: `UPDATE`, `DELETE` und Ähnliches erhalten eine Meldung, die das sagt. Zum Ändern nehmen Sie ein anderes Programm: `O` im Dateifenster öffnet sie in dem unter `opener` eingestellten, und `'*.db' = 'sqlite3 %f'` unter `[editors]` lässt `E` sie in `sqlite3` öffnen.

## Große Tabellen

Mit **V** haben diese Tabellen keine Zeilengrenze: eine CSV- oder TSV-Datei, eine JSON-Lines-Datei, eine dBase-Datei (`.dbf`) sowie eine Tabelle oder View einer SQLite-Datenbank oder eine darin eingegebene Abfrage. Das Raster behält die Zeilen in der Nähe des Cursors, höchstens 2.000 und 32 MiB Text, und liest die übrigen aus der Datei, während sich der Cursor bewegt, ohne den Bildschirm aufzuhalten: Eine noch nicht gelesene Zeile erscheint kurz als `…`.

- **Die Gesamtzahl** erscheint, während gezählt wird: `Row 1 of 2,000+ (counting…)`, dann `Row 1 of 8,412,903`. **Ctrl+End** geht zur letzten Zeile und `g` zu jeder; bevor die Zählung endet, meldet `g` darüber hinaus, dass die Zählung noch nicht so weit ist.
- **Der Filter** (`&`) läuft im Hintergrund durch die ganze Tabelle. Die Zeile unter dem Raster zeigt, wie weit er ist, `1,203 matches so far, row 2,300,000 of 8,412,903`, und die Zeilen erscheinen, sobald sie gefunden werden. Er behält höchstens 1.000.000 Treffer und zeigt dann `first 1,000,000 matches`.
- **Esc** hält die Zählung oder den Filter an und behält das bisher Gefundene; das nächste Esc schließt den Betrachter.
- Die Spaltenbreiten kommen aus den ersten Zeilen, und eine Spalte wird breiter, wenn eine breitere Zeile auf den Bildschirm kommt. Eine JSON-Lines-Datei kann beim Zählen auch Spalten hinzugewinnen, wenn ein Schlüssel erst weiter hinten auftaucht.
- **CSV, TSV und JSON Lines** werden beim Zählen einmal von Anfang bis Ende gelesen, und Xylem merkt sich, wo jede 256. Zeile beginnt, sodass jede Zeile erreicht wird, indem höchstens 255 andere gelesen werden. Diese Notiz liegt nur im Speicher, etwa 8 Bytes je 256 Zeilen, und verschwindet beim Schließen des Betrachters. Ein CSV-Datensatz mit einem Zeilenumbruch in Anführungszeichen bleibt eine Zeile. Ein CSV-Datensatz über 16 MiB beendet das Lesen, und eine JSON-Lines-Zeile über 512 KiB wird übersprungen.
- **dBase-Datensätze** sind alle gleich groß, daher geht Xylem direkt zu jedem. Die Zählung prüft nur, ob ein Datensatz gelöscht ist, denn ein gelöschter Datensatz ist keine Zeile: Xylem merkt sich auf dieselbe Weise, wo jede 256. Zeile liegt.
- **Ändert sich die Datei**, während sie offen ist (sie wächst, schrumpft oder wird ersetzt), gilt nicht mehr, was Xylem sich von ihr gemerkt hat: Es wird nichts mehr aus ihr gelesen, und die Kopfzeile zeigt `(the file changed: R reloads)`. **R** liest sie neu.
- **Text und Hex** einer Datei, die als Tabelle öffnet, werden erst gelesen, wenn Sie dorthin wechseln (**T**, **H**): Bis dahin liest Xylem nur den Anfang der Datei, sodass sich eine große Tabelle sofort öffnet.

Autoview (F7) öffnet keine Datenbanken und liest von den übrigen Tabellen nur die ersten Zeilen. Die anderen Formate behalten vorerst die Grenze unten.

## Grenzen

Abgesehen von den großen Tabellen oben fasst eine Tabelle höchstens 50.000 Zeilen und 256 Spalten und braucht höchstens, was die übrigen Grenzen des Betrachters erlauben: 3 s (100 ms in Autoview), und eine Tabellenkalkulation, die entpackt größer als das Größenlimit ist (1 MiB in Autoview), öffnet sich nicht als Tabelle. Was darüber hinausgeht, erscheint nicht, und der Filter sieht es nicht; die Kopfzeile weist darauf hin: `(showing first 50,000 rows)` oder `(stopped by the time limit: showing N rows)`. Während die Tabelle gelesen wird, bleibt der Bildschirm frei und ein Hinweis sagt es; Esc stoppt das Lesen und zeigt, was schon da ist, `(stopped by Esc: showing N rows)`, und das nächste Esc schließt den Betrachter. Eine Zelle zeigt höchstens 2.048 Zeichen und dann `…`. Steuerzeichen und Zeichen für die Schreibrichtung erscheinen als `·`.

Hat eine Tabellenkalkulation (XLSX, XLSB, XLS, ODS) mehr Zeilen, sagen es Kopfzeile und Hinweis: `(the sheet has more than 50,000 rows, the most Xylem shows: open it in Excel or another spreadsheet program to see them all (O))`, also: öffnen Sie es in Excel oder einem anderen Tabellenprogramm, um alle zu sehen. **O** öffnet es in dem Programm, das Ihr System dafür verwendet.

## Tabellen in C-Bibliotheken

SQLite-Datenbanken (`.db`, `.sqlite`, `.sqlite3`) und Parquet-Dateien werden von in C geschriebenen Bibliotheken gelesen, die nicht die Speichersicherheit von Rust bieten. Deshalb öffnen sie sich nur mit `V` als Tabelle. Autoview liest alles, worüber der Cursor läuft, und zeigt diese Dateien als gewöhnlichen Inhalt mit dem Hinweis `Parquet or SQLite: V opens the table`. Öffnen Sie eine Datenbank aus nicht vertrauenswürdiger Quelle nur, wenn Sie dieses Risiko akzeptieren. Auch `.ods`- und `.xls`-Dateien öffnen sich nur mit `V` als Tabelle: Ihr Leser baut zuerst das ganze Blatt auf, und dieser Teil lässt sich nicht anhalten.

Siehe auch: [der Betrachter](viewer.md).
