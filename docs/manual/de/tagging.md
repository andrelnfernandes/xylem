# Markierungen und Gruppenoperationen

Markierte Dateien (gekennzeichnet mit `■`) sind das Ziel der
Gruppenoperationen: bearbeitet werden die markierten Dateien **der aktuellen
Liste** (des Verzeichnisses oder der geöffneten Branch- bzw. Showall-Ansicht).

## Markieren

| Taste | Baum | Dateifenster |
|---|---|---|
| `T` / `U` | Dateien des Verzeichnisses markieren / Markierung aufheben | Datei markieren / Markierung aufheben und nach unten gehen |
| Ctrl+T / Ctrl+U | alles Eingelesene (passend zum Filespec) markieren / Markierung aufheben | gesamte Liste markieren / Markierung aufheben |
| Ctrl+S | Showall nur der markierten Dateien | |
| `I` | Markierungen des Verzeichnisses umkehren | Markierungen der Liste umkehren |
| Ctrl+F7 / F8 / F9 | eingelesene Dateien des Zweigs markieren / Markierung aufheben / umkehren | |

## Operationen auf markierten Dateien

| Taste | Funktion |
|---|---|
| Ctrl+C / Ctrl+M | in das Zielverzeichnis [kopieren / verschieben](copy-move.md) |
| Alt+C / Alt+M | kopieren / verschieben, **unter Wiederherstellung ihrer Pfade** |
| Ctrl+D | [löschen](delete.md) |
| Ctrl+A | [Berechtigungen](attributes.md) ändern |
| Ctrl+N | [Datum](attributes.md) ändern |
| Ctrl+R | mit einer Maske [umbenennen](rename.md) |
| Ctrl+B | [Batch](execute.md): ein Skript oder eine Liste mit einer Zeile pro Datei |
| Ctrl+F5 | in ein [Archiv](archives.md) packen |
| Ctrl+S / Ctrl+V | [nach Text durchsuchen / betrachten](search.md) |
| Ctrl+O | im Standardprogramm öffnen |
| Ctrl+Y | ihre Pfade in die Zwischenablage kopieren |
| Alt+O | in das [integrierte Terminal](terminal.md) einfügen |

Bei eXecute steht `%t` für die markierten Dateien ([Ausführen](execute.md)).

## Was Ctrl und Alt bedeuten

- Ein Buchstabe allein wirkt auf das Element unter dem Cursor.
- Ctrl+Buchstabe wirkt auf die markierten Dateien: Ctrl+C kopiert sie, Ctrl+D
  löscht sie.
- Alt+Buchstabe ist die andere Fassung eines Befehls: der gesamte Zweig im Baum,
  die markierten Dateien mit ihren Pfaden im Dateifenster.

[Die Menüs](menus.md) erklärt die Regel und wie die Menüs Ctrl und Alt mit F9
angezeigt werden. [Das Strg-Menü](commands-ctrl.md) und
[das Alt-Menü](commands-alt.md) listen jeden Befehl auf.

**Geändert in Version 0.8.** Alt wiederholt Ctrl nicht mehr: Alt+A, Alt+D,
Alt+R, Alt+T, Alt+U, Alt+W und Alt+Y haben sich geändert.
[Versionshinweise](releases.md) enthält die alte und neue Bedeutung jeder Taste
sowie die Zeilen für `config.toml`, die die alten wiederherstellen.
