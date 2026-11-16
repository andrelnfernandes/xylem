# Dateien mit einer SQL-Bedingung auswählen

Diese Anleitung listet alle JPG-Dateien über 1 MB unterhalb von `~/projects` auf
und markiert sie, bereit zum Kopieren, Verschieben oder Löschen.

## Schritte

1. Setzen Sie den Cursor im Baum auf `projects` und drücken Sie `B`. Das
   Dateifenster listet die Dateien des gesamten Zweigs auf.
2. Drücken Sie `W`. Die Eingabezeile fragt nach einer Bedingung.
3. Geben Sie `ext = 'jpg' AND size > 1000000` ein und drücken Sie Enter. Die
   Liste behält die Dateien, die der Bedingung entsprechen, und die Fußzeile
   nennt deren Anzahl.
4. Drücken Sie Ctrl+T. Die verbliebenen Dateien der Liste werden markiert.

![Die vier Schritte: Branch auf projects, W, die eingegebene Bedingung und die 12 verbliebenen, mit Ctrl+T markierten Dateien](../../screenshots/howto-sql-filter.gif)

Der Titel des Dateifensters zeigt die Bedingung an: `Branch: * where ext =
'jpg' AND size > 1000000 (12)`. Ein Druck auf `W` mit leerer Eingabe zeigt
wieder die gesamte Liste; die Markierungen bleiben erhalten.

## Die Bedingung formulieren

Die Bedingung entspricht dem, was in einer SQL-Abfrage nach `WHERE` steht. Text
steht in einfachen Anführungszeichen, Zahlen werden als Ganzes geschrieben.

| Zum Behalten von | Bedingung |
|---|---|
| Dateien der letzten Woche | `age_days < 7` |
| Markdown- und Textdateien | `ext IN ('md', 'txt')` |
| Namen, die mit `report` beginnen | `name LIKE 'report%'` |
| noch nicht markierten Dateien | `NOT tagged` |
| Dateien, die einen Text enthalten | `contains('invoice')` |

[Eine SQL-Bedingung](filespec.md#eine-sql-bedingung-w) listet alle Spalten auf.

## Wenn das Ergebnis unerwartet ist

- **`No column "jpg"`.** Der Text wurde ohne Anführungszeichen eingetippt.
  `ext = jpg` vergleicht zwei Spalten; `ext = 'jpg'` vergleicht mit dem Text.
- **Erwartete Dateien fehlen.** `W` prüft nur eingelesene Dateien. Öffnen Sie
  vorher den Branch oder lesen Sie weitere Ordner mit `*` ein und nutzen Sie
  `S` (Showall).
- **Es ändert sich nichts.** Keine Datei erfüllt die Bedingung und die Liste
  bleibt unverändert.
- **`contains` fragt vor Ausführung nach.** Es liest jede zu prüfende Datei ein;
  die Abfrage nennt Anzahl der Dateien und Bytes.

Siehe auch: [Filespec](filespec.md),
[Markierungen und Gruppenoperationen](tagging.md),
[Dateien mit Pfaden kopieren](howto-copy-with-paths.md).
