# Finden, was die Platte füllt

Diese Anleitung führt vom Home-Verzeichnis bis zu den größten Dateien darin und
löscht eine davon.

## Schritte

1. Setzen Sie den Cursor im Baum auf das zu untersuchende Verzeichnis und
   drücken Sie Alt+U. Xylem liest den Zweig ein und listet dessen Ordner und
   Dateien nach belegtem Speicherplatz auf, größte zuerst.
2. Bewegen Sie sich zu einem Ordner (sein Name beginnt mit `/`) und drücken Sie
   Enter, um seinen Inhalt aufzulisten.
3. Drücken Sie erneut Enter auf dem größten Ordner darin und wiederholen Sie
   dies, bis die Liste Dateien anzeigt.
4. Bewegen Sie sich zu der zu löschenden Datei und drücken Sie Del. Xylem fragt
   vor dem Löschen nach.
5. Drücken Sie `T`, um sie in den Papierkorb zu verschieben, und danach `Y`.

![Die fünf Schritte: Alt+U im Home-Verzeichnis, Enter auf projects, Enter auf backup, Del auf dem älteren Archiv und T, Y verschieben es in den Papierkorb](../../screenshots/howto-disk-space.gif)

Die Fußzeile meldet `Trashed 1 of 1 files` und die Prozentanteile werden ohne
diese Datei neu berechnet.

## Eine Zeile lesen

```
     19.0 MiB ████████████████    51%   projects-2026-10.tar.gz
```

| Teil | Bedeutung |
|---|---|
| `19.0 MiB` | der von dem Eintrag auf der Festplatte belegte Platz |
| der Balken | der Eintrag im Verhältnis zum größten Eintrag der Liste |
| `51%` | der Anteil des Eintrags am gelisteten Verzeichnis |
| der Name | ein Ordner, wenn er mit `/` beginnt |

## Tasten in der Liste

| Taste | Funktion |
|---|---|
| Enter | öffnet den Ordner; bei einer Datei zeigt sie diese im Betrachter an |
| Backspace oder ← | kehrt zum übergeordneten Ordner zurück |
| `T` | markiert die Datei bzw. alle Dateien des Ordners |
| Ctrl+D | löscht die markierten Dateien |
| Alt+P | löscht den Ordner samt Inhalt endgültig |
| `D` | sucht nach Duplikaten im gelisteten Ordner |
| `\` | schließt die Liste und setzt den Baum-Cursor auf den gelisteten Ordner |
| Esc | schließt die Liste |

## Wenn das Ergebnis unerwartet ist

- **Eine Zeile beginnt mit `>`.** Dort ist eine andere Festplatte eingehängt. Sie
  wird nicht mitgezählt, und Enter darauf öffnet eine separate Liste für dieses
  Dateisystem.
- **Eine Zeile beginnt mit `!` oder `.`.** Der Ordner oder Inhalte darin konnten
  nicht gelesen werden, meist wegen fehlender Berechtigungen. Er belegt
  möglicherweise mehr Platz als ausgewiesen.
- **Größen weichen vom Dateifenster ab.** Diese Liste zeigt den Platzbedarf auf
  der Festplatte in ganzen Blöcken. `A` schaltet auf die eigentlichen
  Dateigrößen um.

## Ein Cloud-Laufwerk

Dieselben Schritte funktionieren in einem von rclone eingehängten Google-Drive-
oder S3-Ordner. [Cloud-Speicher (rclone)](cloud.md) beschreibt die Einrichtung
und Unterschiede.

Siehe auch: [Speicherplatzbelegung](usage.md),
[Duplikate finden](howto-duplicates.md).
