# Der Betrachter

`V` öffnet die Datei im Betrachter (Autoview, F7, zeigt sie neben der Liste
an). Binärdateien werden im Hex-Modus geöffnet. Eine große Datei auf der Festplatte wird nicht ganz gelesen: Der Betrachter liest, was der Bildschirm zeigt, während er weiterläuft (siehe [Große Dateien](#große-dateien)). Was sich so nicht lesen lässt, zeigt nur seinen Anfang, bis zum Größenlimit.
Tabellenkalkulationen, CSV, DBF, SQLite und Parquet öffnen sich als Tabelle (siehe [Tabellen](tables.md)).

| Taste | Funktion |
|---|---|
| ↑ ↓ PgUp PgDn Home End | scrollen |
| ← → | seitlich scrollen |
| `H` | nächster Modus: Text und Hex; bei einer Markdown-Datei gerendert, Text und Hex |
| `O` | nächste Kodierung (UTF-8, CP850, 1252, CP437, UTF-16) |
| `C` | Syntaxfarben ein oder aus |
| `W` | Zeilenumbruch ein oder aus |
| `#` oder `L` | Zeilennummern ein oder aus (im Textmodus) |
| `g` oder `:` | zu einer Zeile springen (Byte-Offset im Hex-Modus, Tabellenzeile im Tabellenmodus) |
| `&` | zeigt nur passende Zeilen, einfacher Text oder Regex ([Zeilen filtern](filter.md)); im Tabellenmodus der SQL-Filter |
| Ctrl+B | eine Ausgabezeile pro angezeigtem Datensatz oder Zeile ([Batch im Betrachter](execute.md)) |
| `E` | Datei bearbeiten; zurück in Xylem zeigt der Betrachter den neuen Inhalt an derselben Stelle (Binärdateien nur mit einem Editor unter [`[editors]`](execute.md)) |
| `D` | bei aktiviertem Git: Änderungen seit dem letzten Commit und zurück ([Git](git.md)) |
| `T` | Tabelle oder Text, bei einer Datei, die sich als Tabelle geöffnet hat; gerendert oder Quelltext, bei einer Markdown-Datei |
| `U` | in der Ansicht der markierten Dateien (Ctrl+V): Markierung der Datei aufheben |
| `y` oder Ctrl+Insert | Dateipfad in die Zwischenablage kopieren |
| Esc | schließen |

## Suchen

Wie in ZTreeWin: **F** sucht vom Anfang an, **B** rückwärts vom Ende aus, **/**
(oder `S`) ab der aktuellen Seite und **\\** von dort rückwärts; **Space**
sucht in derselben Richtung weiter, **+** und **-** wählen die Richtung. Der
Treffer wird hervorgehoben. In der Sucheingabezeile schaltet **F2** die
Groß-/Kleinschreibung um (wird sie ignoriert, gilt das auch für Buchstaben mit Akzent: `AÇÃO` findet
`ação`) und **F4** wechselt zwischen Text und Hex
(Byte-Paare wie `4d 5a`). Ein `*` in der Mitte des Suchbegriffs steht für
beliebige Zeichen in derselben Zeile (`error*disk`).

## Syntaxfarben

Quellcode und Skripte werden in rund 150 Sprachen farbig hervorgehoben (Rust,
Python, Shell, C, JavaScript, SQL, YAML, Markdown...), erkannt anhand der
Erweiterung, des Namens (`Makefile`) oder der ersten Zeile (`#!/bin/bash`). Die
Standardeinstellung ist `syntax_highlight` (Seite 2 der
[Konfiguration](configuration.md)). Die Farben bleiben auf dem Hintergrund des
aktuellen Themas gut lesbar, und es werden nur die ersten 20.000 Zeilen
eingefärbt und keine Zeile mit mehr als 10.000 Zeichen, sodass auch riesige
Dateien den Betrachter niemals ausbremsen.

## Gerendertes Markdown

`.md`- und `.markdown`-Dateien öffnen sich gerendert, auch in Autoview, in Archiven und komprimiert (`x.md.gz`), und der Rahmen zeigt `RENDERED`. `T` zeigt den Quelltext an derselben Stelle und wechselt zurück; `H` geht durch gerendert, Text und Hex.

Zu sehen sind Überschriften, Fett, Kursiv, Durchgestrichen, Code, Codeblöcke mit Syntaxfarben (`C` schaltet sie aus), verschachtelte und nummerierte Listen, Aufgabenlisten (`[x]`), Zitate, Trennlinien und Tabellen. Nichts wird je heruntergeladen, und kein Link öffnet etwas:

- Ein Link zeigt seine echte Adresse neben dem Text, etwa `sicher <https://example.com>`, damit der Text nicht als andere Adresse durchgeht.
- Ein Bild ist eine Textzeile: `[image] Beschriftung <foto.png>`.
- Ein Mermaid-Diagramm ist ein Codeblock mit der Beschriftung `mermaid`.
- HTML und HTML-Kommentare bleiben als Text sichtbar.
- Der YAML-Block am Anfang einer Datei ist ein Codeblock mit der Beschriftung `yaml`.

Der Text passt immer ins Fenster, daher bewirkt `W` hier nichts; ← → erreichen die Spalten einer zu breiten Tabelle. Die Suche betrachtet den Text auf dem Bildschirm, einschließlich der Adressen, und findet auch einen Satz, der in der nächsten Reihe weitergeht. `g` und `#` zählen die Zeilen des Quelltexts. Ctrl+B arbeitet auf den Zeilen des Quelltexts, auf allen. `&` wechselt in den Textmodus und filtert dort.

Eine Datei über 8 MiB öffnet sich als Quelltext, mit einem Hinweis. Das Rendern stoppt auch am Zeitlimit (3 s, 100 ms in Autoview) oder an seinem Speicherlimit, und die Kopfzeile zeigt dann `(rendered only the start)`. Eine Tabelle zeigt höchstens 256 Spalten. Steuerzeichen erscheinen als `·`, auch solche, die als Entitäten wie `&#27;` geschrieben sind.

Markdown selbst verbirgt einiges: Linkdefinitionen, die nichts verwendet, und Tabellenzellen jenseits der Spalten der Kopfzeile. Um alles zu sehen, zeigt `T` den Quelltext. Kursiv und Durchgestrichen hängen vom Terminal und seiner Schrift ab, daher haben sie auch eine eigene Farbe.

## Steuerzeichen für die Schreibrichtung

Manche unsichtbaren Zeichen ändern die Schreibrichtung des Textes und können Code anders aussehen lassen, als er wirklich lautet (Trojan Source, CVE-2021-42574). Der Betrachter versteckt sie nie: Jedes erscheint als rotes `·`, und die Kopfzeile nennt ihre Anzahl.

## Textkodierung

Xylem erkennt die Kodierung selbst, in dieser Reihenfolge: zuerst wird die Datei entpackt, dann wird nach einer BOM gesucht (UTF-8, UTF-16 LE oder BE), dann nach UTF-16 ohne BOM, dann nach UTF-8. Passt nichts davon, ist es alter Text: Xylem rät CP850 (DOS, Westeuropa) oder Windows-1252, indem es die Buchstaben mit Akzent zählt. `.reg`-Dateien von Windows sind UTF-16 und öffnen sich als normaler Text.

**O** zeigt die Datei in der nächsten Kodierung: UTF-8, CP850, Windows-1252, CP437, UTF-16 LE, UTF-16 BE und wieder UTF-8. Die Fußzeile sagt, welche, und die Kopfzeile zeigt den Namen, wenn es nicht UTF-8 ist. Wählen Sie UTF-16 bei einer Datei, die in Hex geöffnet wurde, wechselt sie zu Text. Ihre Wahl bleibt, wenn Sie die Datei bearbeiten und zurückkehren.

Um für alte Textdateien immer eine feste Kodierung statt der Vermutung zu nutzen, setzen Sie `legacy_charset` auf `cp850`, `cp1252` oder `cp437` (**K** auf Seite 2 der [Konfiguration](configuration.md)). Standard ist `auto`.

## Große Dateien

Mit **V** wird eine Datei auf der Festplatte, die größer als 16 MiB ist, nicht geladen: Der Betrachter behält die Zeilen in der Nähe des Bildschirms, etwa 2.000, und liest die übrigen aus der Datei, während er weiterläuft, ohne den Bildschirm aufzuhalten. Eine noch nicht gelesene Zeile erscheint kurz als `…`. So hat die Datei keine Größengrenze, und der belegte Speicher wächst nicht mit ihr.

- **Die Zeilen werden im Hintergrund gezählt.** Die Fußzeile zeigt `Line 1 of 1,234,567+ (counting…)` und dann die Gesamtzahl. **End** und **g** gehen zu jeder schon gezählten Zeile. Esc hält die Zählung an, und das Gezählte bleibt.
- **Die Suche** (`F`, `B`, `/`, `\`, Space) läuft im Hintergrund durch die Datei: Die Fußzeile zeigt `Searching... line 1,200,000 (Esc stops)`, und der Bildschirm springt zum Treffer, sobald er gefunden ist.
- **Der Zeilenfilter** (`&`) läuft im Hintergrund durch die ganze Datei, und die Zeilen erscheinen, sobald sie gefunden werden, bis zu 1.000.000. Esc hält ihn an und behält das Gefundene.
- **Hex** (`H`) liest die Bytes dort, wo der Bildschirm steht, und auch seine Suche läuft durch die Datei.
- **Eine andere Kodierung** (`O`) liest die Zeilen neu; bei UTF-16 werden sie neu gezählt, weil eine Zeile an anderer Stelle endet.
- **Keine Syntaxfarben und kein gerendertes Markdown**: Beide lesen einen Text von Anfang an.
- **Eine Zeile über 1 MiB** wird auf dem Bildschirm gekürzt und endet mit `…`; Hex zeigt sie ganz.
- **Ändert sich die Datei**, während sie offen ist, zeigt die Kopfzeile `(the file changed: R reloads)`, und **R** liest sie neu und kehrt zur vorherigen Zeile zurück.
- **Ein Batch** (`Ctrl+B`) über den Text liest die Datei weiterhin in den Speicher, bis zum Größenlimit unten, und sagt es, wenn das nicht die ganze Datei ist.

Um eine Zeile zu erreichen, merkt sich der Betrachter, wo jede 256. Zeile beginnt, etwa 8 Bytes je 256 Zeilen, nur im Speicher. Die Textrichtungszeichen werden zusammen mit den Zeilen über die ganze Datei gezählt (in UTF-8), und die Kopfzeile zeigt, wie viele bisher gefunden wurden.

Eine komprimierte Datei, ein Eintrag eines Archivs, eine Datei auf einem entfernten Rechner und Autoview werden nicht so gelesen: Darin lässt sich nicht springen, daher gilt für sie das Größenlimit unten.

## Größenlimit

In diesen Fällen und bei Dateien bis 16 MiB liest der Betrachter von einer zu großen Datei nur den Anfang, und die Kopfzeile zeigt `(showing first N MiB)`. Wie viel er liest, bestimmt die Option `viewer_max_size` (**L** auf Seite 2 der [Konfiguration](configuration.md)): `auto`, `16`, `64`, `256` oder `1024` (MiB). Bei `auto`, dem Standard, beträgt das Limit 1/32 des Arbeitsspeichers des Rechners, nie weniger als 16 MiB und nie mehr als 256 MiB (16 MiB, wenn sich der Arbeitsspeicher nicht auslesen lässt). Die Konfigurationsseite zeigt den verwendeten Wert, zum Beispiel `auto (256 MiB)`.

Zwei Fälle folgen dieser Regel nicht. Autoview liest immer 1 MiB. Eine Datei auf einem entfernten Rechner (SFTP) wird höchstens bis 16 MiB gelesen, weil Esc dieses Lesen nicht abbrechen kann.

Ein größeres Limit kostet Speicher und Zeit: Mit `1024` belegt ein komprimierter Text, der sich auf diese Größe entpackt, etwa 2 GiB RAM. Sehr kurze Zeilen kosten mehr als ihre Bytes, weil jede Zeile eigene Verwaltungsdaten hat, und dasselbe gilt für Tabulatoren und versteckte Steuerzeichen für die Schreibrichtung: Der Betrachter hält Zeilen bis etwa zum Doppelten des Größenlimits im Speicher, danach steht in der Kopfzeile `(showing first N lines)`. Ein Log mit gewöhnlichen Zeilen (etwa 100 Bytes) wird dadurch nie abgeschnitten.

## Komprimierte Dateien

Dateien mit der Endung `.gz`, `.xz`, `.zst` oder `.bz2` werden entpackt als Text geöffnet. Xylem prüft die ersten Bytes der Datei, nicht den Namen, und öffnet nur eine Ebene: Eine `.gz`-Datei in einer `.gz`-Datei bleibt komprimiert. Die Kopfzeile zeigt `gzip → N bytes` (das Format und die Größe nach dem Entpacken).

Das Entpacken hat Grenzen. Der Betrachter liest bis zum Größenlimit, braucht höchstens 3 s und nutzt höchstens 64 MiB Speicher für das xz-Wörterbuch oder das zstd-Fenster. In Autoview gelten 1 MiB, 100 ms und 16 MiB. Wird die Ausgabe- oder die Zeitgrenze erreicht, steht in der Kopfzeile `(showing first N bytes decompressed)`. Ein Hinweis sagt, was passiert ist:

- `decompression stopped after 3 s` (100 ms in Autoview)
- `compressed data truncated or corrupt` (komprimierte Daten abgeschnitten oder beschädigt)
- `needs more than 64 MiB of memory to decompress` (16 MiB in Autoview, mit ` (V to view)`, wenn die 64 MiB von V reichen können)

Der Hinweis verschwindet beim ersten Tastendruck, die Kopfzeile behält das Problem aber: `(truncated or corrupt)`, `(incomplete: …)`, wenn die Speichergrenze das Entpacken mittendrin gestoppt hat, oder `(not decompressed: …)`, wenn nichts entpackt werden konnte (dann erscheinen die Rohbytes in Hex).

Autoview zeigt den Hinweis in seinem Titel. Im Hex-Modus (`H`) sehen Sie die entpackten Bytes. Die Farben richten sich nach dem Namen ohne die Kompressionsendung: `app.rs.gz` wird wie Rust gefärbt.

Bei einer `.tar.gz` (oder `.tar.xz` ...) lautet der Hinweis `TAR archive: Alt+F5 opens it as a folder` (Alt+F5 öffnet sie als Ordner).

Um die rohen komprimierten Bytes zu sehen, schalten Sie `viewer_decompress` aus (**J** auf Seite 2 der [Konfiguration](configuration.md)).

## Tabellenmodus

Tabellenkalkulationen, CSV, DBF, SQLite und Parquet öffnen sich als Raster, mit einem SQL-Filter: siehe [Tabellen](tables.md).

Siehe auch: [markierte Dateien durchsuchen](search.md).
