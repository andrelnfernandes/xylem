# Duplikate finden und eine Kopie behalten

Diese Anleitung findet Dateien mit gleichem Inhalt unter einem Verzeichnis und
verschiebt alle bis auf eine Kopie in den Papierkorb.

## Schritte

1. Setzen Sie den Cursor im Baum auf das Verzeichnis und drücken Sie `*`, um
   seinen Zweig einzulesen.
2. Drücken Sie Alt+D. Xylem vergleicht die Dateien und listet Gruppen gleicher
   Dateien auf, die Gruppe mit dem meisten freizugebenden Platz zuerst.
3. Drücken Sie `A`. In jeder Gruppe werden alle Kopien bis auf eine markiert
   (`T`).
4. Drücken Sie `D`. Xylem fragt vor dem Löschen nach. Drücken Sie `T`, um die
   Dateien in den Papierkorb zu verschieben, und anschließend `Y`.

![Die vier Schritte: Zweig einlesen, Alt+D mit drei Gruppen, A markiert alle Kopien bis auf eine und D, T, Y verschiebt vier Dateien in den Papierkorb](../../screenshots/howto-duplicates.gif)

Die Fußzeile meldet `Trashed 4 of 4 files`.

## Auswählen, welche Kopie erhalten bleibt

Die Taste `A` markiert alle Kopien bis auf eine in jeder Gruppe. Vor Schritt 4
kann die Auswahl angepasst werden:

| Taste | Wirkung |
|---|---|
| `A` erneut | wechselt das Kriterium: älteste, neueste oder die mit dem kürzesten Pfad |
| `K` | behält die Kopie unter dem Cursor und markiert die übrigen der Gruppe |
| `F` | behält in jeder Gruppe, die eine Kopie im Ordner dieser Kopie hat, genau jene |
| `T` oder Leertaste | markiert oder entmarkiert eine einzelne Kopie |
| `U` | hebt alle Markierungen auf |

Xylem lässt Sie die letzte verbleibende Kopie einer Gruppe nicht markieren.

## Ganze Ordner

Die Taste `G` schaltet auf Ordner um, deren gesamter Inhalt identisch ist. Die
Zeile am Ende der Liste weist darauf hin: `1 group of folders (G)`.

## Bedeutung der Sicherheitsabfrage

Bei deaktiviertem `delete_to_trash` (Standard) fragt Schritt 4 zunächst
`DELETE 4 tagged files? This cannot be undone.`. Die Taste `T` ändert die Frage
in `MOVE 4 tagged files to trash?`. Ein anschließendes `Y` bestätigt die
dargestellte Aktion.

## Wenn das Ergebnis unerwartet ist

- **Eine bekannte doppelte Datei wird nicht aufgelistet.** Dateien unter 1 KiB,
  leere Dateien, Links sowie die Ordner `.git` und `node_modules` werden
  standardmäßig ausgelassen. [Duplikate](duplicates.md) beschreibt, wie dies
  angepasst werden kann.
- **Eine Gruppe wurde nicht gelöscht.** Eine Kopie hat sich nach dem Vergleich
  geändert. An dieser Gruppe wird nichts verändert und die Fußzeile meldet dies.
  F3 führt den Vergleich erneut aus.

Siehe auch: [Duplikate](duplicates.md), [Löschen](delete.md),
[finden, was die Platte füllt](howto-disk-space.md).
