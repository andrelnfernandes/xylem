# Der Bildschirm

![Der Bildschirm mit dem Baum oben, den Dateien des Verzeichnisses darunter, den Statistiken rechts und der Menüleiste unten](../../screenshots/tree-nord.png)

```
 Path: ~/projects/xylem/src            Xylem v0.7.0  2026-11-13 00:49   1
┌ Tree ──────────────────────────────────────┐ ┌──────────────────────┐
│ /home/user/projects                        │ │ FILE *               │
│ ├──notes                                   │ │                      │
│ ├──photos                              2   │ │ DISK /home       4   │
│ └──xylem                                   │ │ Available   286,274… │
│    └──src                                  │ │                      │
└────────────────────────────────────────────┘ │ DISK Statistics      │
┌ Files: * (30) Name↑ ───────────────────────┐ │ Logged, Matching,    │
│ archive.rs      50,321 2026-10-12 13:02    │ │ Tagged               │
│ bidi.rs          3,593 2026-10-12 13:02  3 │ │ Current Directory    │
└────────────────────────────────────────────┘ └──────────────────────┘
 DIR   Available Branch Compare Delete Filespec …                       5
       F1 help   * log branch   ? stats   \ treespec …
```

| | Bereich | Was angezeigt wird |
|---|---|---|
| 1 | Kopfzeile | Pfad des Verzeichnisses unter dem Cursor, Programmname und -version, Datum und Uhrzeit. Name und Version entfallen bei weniger als 72 Spalten. |
| 2 | Baum | Die eingelesenen Verzeichnisse. Die Zahl im oberen Rahmen nennt Cursor- und Gesamtzeilenzahl. |
| 3 | Dateifenster | Dateien des Verzeichnisses unter dem Baum-Cursor. Der Titel enthält Filespec, Dateianzahl und Sortierung. |
| 4 | Statistiken | Filespec, Festplatte sowie die unten beschriebenen Datei- und Byte-Zähler. |
| 5 | Fußzeile | Das Menü des aktiven Fensters. Eine Eingabeaufforderung oder Meldung belegt die erste Zeile, falls vorhanden. |

Das aktive Fenster mit dem Fokus hebt sich durch einen hellen Rahmen ab.

## Der Baum

Jede Zeile steht für ein Verzeichnis. Das Verbindungssymbol vor dem Namen
signalisiert den Zustand:

| Symbol | Bedeutung |
|---|---|
| `├──` | eingelesen und aufgeklappt oder ohne Unterverzeichnisse |
| `├─+` | eingelesen, mit zugeklappten Unterverzeichnissen |
| `├─?` | nicht eingelesen: Dateien und Unterverzeichnisse noch nicht gelesen |

Alt+K blendet eine Spalte mit Größe, Dateianzahl oder prozentualem Anteil jedes
Verzeichnisses ein. Siehe [Informationen](info.md).

## Das Dateifenster: klein und groß

Befindet sich der Fokus im Baum, bleibt das Dateifenster verkleinert darunter
und folgt dem Baum-Cursor. **Enter** übergibt ihm den Fokus und vergrößert es
auf fast die gesamte Bildschirmhöhe; der Baum bleibt bis Enter oder Esc
ausgeblendet.

`B` (Branch) und `S` (Showall) öffnen ebenfalls das große Dateifenster und
fassen Dateien mehrerer Verzeichnisse zusammen. Der Titel beginnt dann mit
`Branch:` oder `Showall:`.

Alt+F ändert die Darstellungsart der Zeilen: Namen in Spalten, Namen und Größen,
Details (Größe, Datum und Berechtigungen), Besitzer, Typ oder lange Namen. Siehe
[das Dateifenster](files.md).

## Die Statistiken

| Block | Was gezählt wird |
|---|---|
| FILE | der aktive Filespec; `*` zeigt alle Dateien an |
| DISK | das Dateisystem des aktuellen Verzeichnisses: freie und gesamte Bytes sowie prozentuale Belegung |
| Logged | die in diesem Fenster in den Speicher eingelesenen Dateien |
| Matching | die eingelesenen Dateien, die dem Filespec entsprechen |
| Tagged | die markierten Dateien, mit Balken für ihren Anteil an den übereinstimmenden Bytes |
| Current Directory | Dateien und Bytes des Verzeichnisses unter dem Cursor sowie dessen Berechtigungen |

`?` ersetzt den Bildschirm durch die vollständigen Statistiken: Maschine,
Netzwerk und Xylem selbst. Siehe [Informationen](info.md).

## Was das Bildschirmlayout verändert

| Taste | Wirkung |
|---|---|
| F8 | Zwei Fenster nebeneinander, jedes mit eigenem Baum und Dateifenster. Tab wechselt zwischen ihnen. [Geteilter Bildschirm](split.md) |
| F7 | Im Dateifenster belegt die Liste die linken 40% der Breite und der Inhalt der Datei unter dem Cursor den Rest. [Autoview](files.md) |
| Alt+X | Ein Terminal belegt den unteren Teil des Bildschirms. Alt+Z vergrößert es auf Vollbild. [Das integrierte Terminal](terminal.md) |
| Alt+U | Die Belegungsansicht ersetzt Baum und Dateifenster. [Speicherplatzbelegung](usage.md) |

Siehe auch: [die Menüs](menus.md), [das Baumfenster](tree.md).
