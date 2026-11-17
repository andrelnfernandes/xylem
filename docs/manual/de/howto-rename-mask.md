# Viele Dateien mit einem Zähler umbenennen

Diese Anleitung benennt die neun Fotos in `photos/2025/trip` von `DSC_01.jpg`
… `DSC_09.jpg` in `trip_001.jpg` … `trip_009.jpg` um.

## Schritte

1. Setzen Sie den Cursor im Baum auf `trip` und drücken Sie Enter, um zu dessen
   Dateien zu wechseln.
2. Drücken Sie Ctrl+T. Jede Datei in der Liste wird markiert.
3. Drücken Sie Ctrl+R. Die Eingabezeile fragt nach der Maske und öffnet sich mit
   `*.*`.
4. Drücken Sie Ctrl+U, um die Eingabe zu leeren, geben Sie `trip_<001>.*` ein
   und drücken Sie Enter.

![Die vier Schritte: die Dateien von trip, Ctrl+T, Ctrl+R und die Maske trip_<001>.*, die trip_001.jpg bis trip_009.jpg erzeugt](../../screenshots/howto-rename-mask.gif)

Die Fußzeile zeigt `Renamed 9 of 9 files` an. Die Dateien werden in der
Reihenfolge der Liste nummeriert; sortieren Sie sie daher vorher (Alt+S), falls
die Reihenfolge eine Rolle spielt.

## Die Maske

| Teil | Bedeutung |
|---|---|
| `trip_` | Text, der in jeden Namen kopiert wird |
| `<001>` | ein Zähler mit drei Ziffern: 001, 002, … |
| `.*` | die Dateiendung jeder Datei, unverändert |

Schritt 4 leert die Eingabe, da das Tippen an das vorhandene `*.*` angehängt
würde: `*.*trip_<001>.*` würde die erste Datei `DSC_01.DSC_01trip_001.jpg`
nennen.

## Andere Masken

| Maske | `DSC_01.jpg` wird zu |
|---|---|
| `*.jpeg` | `DSC_01.jpeg` |
| `2025_*.*` | `2025_DSC_01.jpg` |
| `s/DSC/trip/` | `trip_01.jpg` |
| `*.<upper>` | `DSC_01.JPG` |

[Umbenennen](rename.md) beschreibt alle verfügbaren Varianten.

## Wenn das Ergebnis unerwartet ist

- **Nichts wird umbenannt.** Die Maske würde zwei oder mehr Dateien denselben
  Namen zuweisen. Ohne Zähler und ohne `*` erhielten alle Dateien denselben
  Namen.
- **Nur einige Dateien wurden umbenannt.** Nur markierte Dateien werden
  umbenannt. Ein Filespec blendet Dateien aus und Ctrl+T markiert nur, was die
  Liste zeigt.
- **Ctrl+R benennt nur eine Datei um.** Es wurde versehentlich nur `R` gedrückt,
  was die Datei unter dem Cursor umbenennt.

Siehe auch: [Markierungen und Gruppenoperationen](tagging.md),
[das Ctrl-Menü](commands-ctrl.md).
