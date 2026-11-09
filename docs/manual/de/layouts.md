# Layouts: Dateien mit fester Breite und mehreren Satzarten

Viele Dateien, die zwischen Systemen ausgetauscht werden, sind Text ohne Spaltennamen: Jede Zeile ist ein Datensatz, und welche Zeichen welches Feld enthalten, steht in einem Handbuch. Bankdateien (CNAB 240), Steuerdateien (SPED) und der SDF-Export von dBase und Clipper sind so aufgebaut. Ein Layout sagt Xylem, wo jedes Feld steht, und der Betrachter zeigt die Datei dann als Tabelle: ein Blatt pro Satzart, jede Zeile beginnt mit ihrer Zeile in der Datei.

## Eine Datei über ein Layout öffnen

Xylem erkennt manche Dateien an ihrer ersten Zeile und öffnet sie von selbst als Tabelle, mit `V` und im Autoview (F7). Der Hinweis nennt das verwendete Layout: `Layout: FEBRABAN CNAB 240 — Padrão Geral  ·  K changes it`. Bei jeder anderen Datei drücken Sie `K`.

| Taste | Was sie tut |
|---|---|
| `K` | die Liste der Layouts: einen Teil eines Namens tippen grenzt sie ein, Enter wendet eines an, **F2** fragt nach dem Pfad einer Layout-Datei, Esc geht zurück. Ist ein Layout in Gebrauch, schaltet der erste Eintrag es ab und zeigt die Datei als Text |
| `[` `]` | vorherige oder nächste Satzart |
| `Tab` | die Liste der Satzarten, mit der Zahl ihrer Zeilen |
| `i` | die Felder des Datensatzes: Position, Größe, Typ, Beschreibung und die Bedeutung jedes Codes; darüber das Layout, seine Quelle, die Prüfungen der Summen und ihre Ergebnisse |
| `T` | zum Text, an die Zeile des Datensatzes unter dem Cursor, und zurück |
| `\` oder `&` | der SQL-Filter, über den Feldnamen: `valor_pagamento > 1000` |
| Strg+B | ein Batch der Datensätze des offenen Blatts ([Batch im Betrachter](execute.md)) |

Die Blätter kommen in der Reihenfolge, in der jede Satzart zum ersten Mal in der Datei vorkommt: `0 - Header de Arquivo (1)`, `3A - Segmento A (420)`... Eine Zeile, die zu keinem Datensatz passt, landet vollständig im Blatt `! No record type`. Die erste Spalte, `#line`, ist die Zeile in der Datei; so zeigen die Prüfprogramme von Banken und Steuerbehörden einen Fehler an. Hat ein Feld Codes mit Bedeutung, zeigt die letzte Zeile des Rasters sie an: `Content: 2 — Retorno (Banco → Cliente)`.

Werte werden so gelesen, wie das Layout es sagt: Codes behalten ihre Nullen (`00012345`), Daten werden zu `2026-11-09`, Uhrzeiten zu `14:30:00`, und Beträge behalten ihre genauen Nachkommastellen (`1234.50`); nichts wird gerundet. Ein Wert, der nicht dem Layout entspricht (Buchstaben in einer Zahl, der 31. Februar), erscheint so, wie er geschrieben steht, und `i` zählt diese Fälle.

### Prüfung der Summen

Trailer enthalten Summen: die Zeilen eines Stapels, die Zahl der Stapel, die Summe der Zahlungen. Xylem zählt und addiert sie beim Lesen, in ganzen Zahlen, und sagt in `i`, welche stimmen und welche nicht, mit der Zeile, dem Wert in der Datei und dem gezählten Wert. Stimmt eine nicht, meldet es die Kopfzeile: `1 check failed: i shows`. Eine nur teilweise gelesene Datei (durch Esc oder das Zeitlimit gestoppt) wird nicht geprüft.

## Woher Layouts kommen

| Wo | Von selbst verwendet? |
|---|---|
| Der Katalog von Xylem, im Programm enthalten | ja, wenn die erste Zeile zu seinem `[detect]` passt |
| Ihr Ordner: `~/.config/xylem/layouts/` (`%APPDATA%\xylem\layouts\` unter Windows), bis zu drei Ebenen tief | ja, und er kommt vor dem Katalog |
| Neben der Datei (`K`, Abschnitt `Here`), oder ein beliebiger Pfad (`K`, dann F2) | nur, wenn Sie es auswählen |

Eine Layout-Datei im gerade angezeigten Ordner wird nie von selbst gelesen: Ein heruntergeladener oder entpackter Ordner könnte eine enthalten, die zum Täuschen gemacht ist. Wenn Sie sie auswählen, wird sie so streng gelesen wie die Daten (siehe Sicherheit, unten).

Der Katalog enthält vorerst:

| Layout | Was es liest |
|---|---|
| FEBRABAN CNAB 240 — Padrão Geral | Datei-Header und -Trailer; Zahlungen per Kontogutschrift, Scheck, OP, TED und Pix (Segmente A, B, C, Z); Bezahlung von Inkassobelegen (J, J-52); Inkasso (P, Q, R beim Senden; T und U in der Rückmeldung). Nach dem Handbuch der FEBRABAN, Version 11.0 |
| SPED — genérico | jede SPED-Datei mit Feldern zwischen senkrechten Strichen (EFD ICMS/IPI, EFD-Contribuições, ECD, ECF...): ein Blatt pro Satzart, Eröffnung und Abschluss jedes Blocks und Block 9 mit seinen Prüfungen. Satzarten, die es nicht deklariert, haben nummerierte Felder (`CAMPO_02`, `CAMPO_03`...). Was in signierten Dateien nach dem `9999` folgt (die digitale Signatur), wird nicht gelesen |

Der Ordner `layouts/` im [Repository](https://github.com/andrelnfernandes/xylem/tree/main/layouts) listet, was geplant ist und wie man ein Layout beiträgt, ganz ohne Rust.

### SDF aus einer .dbf

`COPY TO datei SDF` in dBase und Clipper schreibt die Datensätze Feld an Feld, jedes mit seiner Breite. In `K` liefert eine `.dbf` neben der Datei (oder eine mit F2 angegebene) ihr Layout: die Felder in Reihenfolge, Daten als `YYYYMMDD`, Zahlen mit Punkt. Memofelder exportiert SDF nicht; sie fehlen.

## Ein Layout schreiben

Ein Layout ist eine TOML-Datei, deren Name auf `.xlayout.toml` endet. Ein positionales:

```
schema = 1
name = "Kunden des alten Systems"
line_length = 40
[discriminator]
pos = [1, 1]
[[records]]
id = "0"
name = "Kopf"
role = "header"
fields = [
  { name = "art", pos = [1, 1], type = "code" },
  { name = "erstellt", pos = [2, 9], type = "date", format = "YYYYMMDD" },
  { pos = [10, 40], type = "filler" },
]
[[records]]
id = "1"
name = "Kunde"
fields = [
  { name = "art", pos = [1, 1], type = "code" },
  { name = "nummer", pos = [2, 7], type = "code" },
  { name = "name", pos = [8, 30], type = "text" },
  { name = "saldo", pos = [31, 40], type = "decimal", decimals = 2, sign = "trailing" },
]
```

Positionen beginnen bei 1 und schließen beide Enden ein: `pos = [8, 30]` reicht vom 8. bis zum 30. Zeichen. `start = 8, len = 23` sagt dasselbe, und ein `len` neben `pos` muss dazu passen, was Tippfehler aufdeckt. Mit `strict_positions` (standardmäßig an) müssen die Felder eines Datensatzes die ganze Zeile abdecken, ohne Lücke und ohne Überlappung: Geben Sie leeren Teilen den Typ `filler`.

| Schlüssel | Was er angibt |
|---|---|
| `schema` | immer `1` |
| `name` | der Name in `K` und in der Kopfzeile |
| `format` | `"positional"` (Standard) oder `"delimited"` |
| `encoding` | `"auto"` (die vom Betrachter erkannte; `O` ändert sie), `"utf-8"`, `"cp1252"`, `"cp850"` oder `"cp437"` |
| `offset_unit` | positional: `"chars"` (Standard) oder `"bytes"` |
| `line_length`, `strict_positions` | positional: die Länge einer Zeile, und ob die Felder sie abdecken müssen |
| `hide_filler` | `filler`-Felder bleiben aus dem Raster (Standard `true`); `i` listet sie immer |
| `delimiter`, `outer_delimiters` | getrennt: das Trennzeichen, ein Zeichen, und ob Zeilen auch damit beginnen und enden, wie bei SPED |
| `allow_undeclared_records` | getrennt: Satzarten, die das Layout nicht deklariert, bekommen ein eigenes Blatt |
| `decimal_sep`, `date_format` | Vorgaben für alle Felder: `"implied"`, `","` oder `"."`; und das Format der Daten |
| `stop_after_trailer` | am Datensatz `trailer` aufhören (was folgt, wird nicht gelesen) |
| `[detect]` | wie die Datei an ihrer ersten Zeile erkannt wird: `line_length`, `first_line` (Bedingungen) und `extensions` |
| `[discriminator]` | wo der Code der Satzart steht: `pos` oder, getrennt, `col` |
| `description`, `country`, `category`, `spec_source`, `spec_version`, `spec_url`, `status` | über das Layout und sein Handbuch; `status` ist `"verified"`, sobald es damit abgeglichen ist |

Jeder `[[records]]` hat eine `id` (sein Code für den Diskriminator; `ids` fügt weitere hinzu), einen `name`, eine `role` (`header`, `batch_header`, `detail`, `batch_trailer` oder `trailer`) und seine `fields`. Wenn der Code allein die Satzarten nicht unterscheidet, fügt `match` Bedingungen an die Zeile hinzu und `batch_match` Bedingungen an die Zeile des Stapel-Headers: Im CNAB 240 unterscheiden sich Segment A einer Zahlung und einer Lastschrift nur durch die Operation im Stapel-Header. Eine Bedingung ist eine Stelle (`pos` oder `col`) und eines von `value = "3"`, `in = ["1", "2"]` oder `regex = "^[0-9]+$"`. Unter Satzarten mit demselben Code wird die mit den meisten Bedingungen zuerst versucht.

| Feldschlüssel | Was er angibt |
|---|---|
| `name` | der Spaltenname, auch für den SQL-Filter |
| `pos`, `start` und `len`, oder `col` | wo es steht |
| `type` | `text`, `code` (Ziffern, die keine Menge sind: Steuernummer, Konto, Postleitzahl), `int`, `decimal`, `date`, `time` oder `filler` |
| `decimals`, `decimal_sep` | Nachkommastellen, und wo das Komma steht: `"implied"` (das `V99` von COBOL), `","` oder `"."` |
| `sign` | `"none"`, `"leading"`, `"trailing"` oder `"overpunch"` (das gezonte Vorzeichen von COBOL: `{`, `A`-`I`, `}`, `J`-`R`) |
| `format` | Daten: `YYYYMMDD`, `DDMMYYYY`, `YYMMDD`, `DDMMYY` oder `YYYYDDD`; Uhrzeiten: `HHMMSS` oder `HHMM`. Zweistellige Jahre 00 bis 69 liegen in den 2000ern |
| `trim` | vom Text entfernte Leerzeichen: `"right"` (Standard), `"left"`, `"both"` oder `"none"` |
| `desc`, `values` | die Beschreibung, und was jeder Code bedeutet: `values = { "0" = "Neu", "9" = "Löschen" }` |
| `check` | womit der Wert übereinstimmen muss, unten |

| `check` | Der Wert muss sein |
|---|---|
| `total_lines` | die Zeilen der Datei bis zu dieser |
| `record_count:1*` | die Zeilen der Satzarten mit dieser id; `*` nimmt jede id, die so beginnt, `+` fügt weitere hinzu |
| `sum:3A.valor` | die Summe eines Felds einer Satzart in der ganzen Datei; `+` fügt weitere hinzu |
| `batch_lines` | die Zeilen vom Stapel-Header bis zu dieser |
| `batch_sum:3A.valor+3J.valor` | die Summe im Stapel |
| `count_by_record:REG_BLC` | die Zeilen, deren Code der Wert des genannten Felds ist (das 9900 von SPED) |
| `block_lines` | die Zeilen, deren Code so beginnt wie der dieser Zeile (das x990 von SPED) |

COBOL-Copybooks (`copybook = ...`), EBCDIC und gepackte Zahlen (`COMP-3`) sind geplant und werden noch nicht gelesen.

## Sicherheit

Sowohl die Daten als auch ein außerhalb Ihres Ordners gewähltes Layout können feindlich sein. Ein Layout wird so streng gelesen wie Daten: höchstens 1 MiB, jeder unbekannte Schlüssel oder Typ wird abgelehnt, jede Position und Größe wird geprüft, Namen dürfen keine Steuer- oder Textrichtungszeichen enthalten, und Satzarten, Felder, Codes, Bedingungen und reguläre Ausdrücke haben Grenzen. Reguläre Ausdrücke laufen in linearer Zeit. Ein Layout kann keine anderen Dateien lesen: `copybook` wird vorerst abgelehnt.

Die Daten werden mit den Grenzen des Tabellenmodus gelesen: 3 s (100 ms im Autoview), 50.000 Zeilen pro Blatt, 256 Spalten, und Zeilen über 512 KiB werden nicht in Felder zerlegt. Esc stoppt das Lesen. Zellen werden wie alle anderen für das Terminal unschädlich gemacht: Steuer- und Textrichtungszeichen erscheinen als `·`.

Siehe auch: [der Betrachter](viewer.md).
