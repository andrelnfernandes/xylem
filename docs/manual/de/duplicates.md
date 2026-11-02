# Duplikate (Alt+Y)

Dateien mit gleichem Inhalt, egal mit welchen Namen und in welchen Ordnern, und
der Platz, den die Kopien verschwenden. **Alt+Y** sucht im Zweig des aktuellen
Verzeichnisses; **D** in der Belegungsansicht sucht im dort gezeigten Ordner.

```
── 3 × 1.2 GiB (2.4 GiB to free) ──────────────────────
     1.2 GiB  2023-08-14  Backups/phone-2023.zip
 T   1.2 GiB  2023-08-14  Downloads/phone-2023.zip
 T   1.2 GiB  2023-08-14  Old/phone-2023 (1).zip
── 2 × 850.0 MiB (850.0 MiB to free) ──────────────────
     850.0 MiB  2021-12-25  Videos/christmas.mp4
 T   850.0 MiB  2022-01-03  WhatsApp/VID-20211225.mp4
```

Was am meisten freigibt, steht zuerst. Nichts wird markiert, bevor Sie es
verlangen, und eine Kopie jeder Gruppe bleibt immer: Xylem lässt die letzte nicht
markieren.

## Auswählen, was geht

| Taste | Wirkung |
|---|---|
| ↑ ↓ PgUp PgDn Home End | bewegen |
| `T` oder Leertaste | markiert die Kopie oder hebt die Markierung auf |
| `K` | behält diese Kopie: markiert die anderen der Gruppe |
| `A` | in jeder Gruppe alle bis auf eine markieren; erneut `A` ändert, welche bleibt: die älteste, die neueste, der kürzeste Pfad |
| `F` | in jeder Gruppe mit einer Kopie im Ordner dieser Kopie jene behalten |
| `U` | alle Markierungen aufheben |
| `D`, Entf oder Strg+D | löscht die markierten, wie der Befehl Delete (in den Papierkorb, wenn `delete_to_trash` an ist) |
| `M` oder Strg+M | verschiebt die markierten Dateien |
| `G` | Dateien oder ganze Ordner |
| `V` oder Enter | zeigt die Datei |
| `E` | schreibt die Liste in eine Datei (nie über eine vorhandene): Größe, Datum, Markierung und Pfad, eine Leerzeile zwischen den Gruppen |
| `\` | schließt die Ansicht, mit dem Baum auf diesem Ordner |
| F3 | liest die Ordner neu und vergleicht erneut |
| Esc oder Alt+Y | schließt |

Vor dem Löschen oder Verschieben wird jede Gruppe erneut geprüft: Hat sich eine
Kopie seit der Analyse geändert oder fehlt die, die bleiben sollte, wird von
dieser Gruppe nichts angerührt, und die Fußzeile sagt es.

## Ganze Ordner

Komplett wiederholte Sicherungen belegen am meisten Platz, und eine lange
Dateiliste verbirgt das. **G** zeigt die Ordner mit genau gleichem Inhalt: gleiche
Namen, gleicher Inhalt jeder Datei, ob groß oder klein, und gleiche Links, die auf
dasselbe Ziel zeigen. Aufgeführt werden nur die obersten, nicht die Ordner darin.

Ein Ordner gilt nie als Kopie, wenn etwas darin nicht verglichen wurde: ein
unlesbarer Ordner, ein darin eingehängtes anderes Laufwerk, eine Spezialdatei oder
ein ignorierter Ordner (siehe unten). Zwei Klone eines Repositorys haben dieselben
Dateien, aber ihr `.git` kann unterschiedliche Arbeit enthalten.

## Wie Dateien verglichen werden

Die Größen kommen aus dem Speicher. Gelesen werden nur Dateien, deren Größe mit
einer anderen übereinstimmt: zuerst beide Enden, dann die Mitte, in Teilen, und
eine Gruppe wird geteilt, sobald sich ihre Dateien unterscheiden, sodass
verschiedene Dateien nicht bis zum Ende gelesen werden. Verglichen wird per
SHA-256. Das läuft im Hintergrund: Die Fußzeile zeigt den Fortschritt, und Esc
bricht ab.

Nicht in der Liste stehen:

- Dateien kleiner als `duplicates_min_size` (standardmäßig 1 KiB) und leere Dateien;
- Links und Spezialdateien, denen nicht gefolgt wird;
- weitere Hardlinks derselben Datei: Sie teilen sich den Platz schon, und eine zu
  löschen gibt nichts frei;
- andere im Zweig eingehängte Laufwerke;
- die Ordner aus `duplicates_ignore` (standardmäßig `.git` und `node_modules`).

Eine Datei, die sich seit dem Einlesen des Baums geändert hat oder nicht lesbar
ist, wird nicht verglichen; die untere Zeile sagt, wie viele es sind. F3 liest
alles neu.

In einem Netzwerkordner (NFS, SMB, sshfs...) wird zuerst nach der zu lesenden
Menge gefragt. Auf einem [entfernten Rechner](remote.md) ebenso: Die Dateien
werden dort über SSH gelesen und vor dem Löschen erneut geprüft, wie hier. Zwei
Dinge sind dort anders: SFTP liefert keine Inode, daher erscheinen zwei harte
Links auf eine Datei als Paar (eines zu löschen gibt nichts frei und verliert
nichts); und andere darin eingehängte Datenträger werden über ihr Dateisystem
ausgelassen, wofür die `statvfs`-Erweiterung von OpenSSH nötig ist. In einem
Archiv ist Duplikate nicht verfügbar.

## In der Cloud (rclone)

In einem mit `rclone mount` eingehängten Ordner bedeutet das Lesen einer Datei,
sie herunterzuladen. Deshalb fragt Xylem rclone nach den Prüfsummen, die der Dienst
bereits speichert (`rclone lsjson --hash`), und lädt nichts herunter: Ein ganzes
Drive dauert Minuten. Es verwendet die stärkste Prüfsumme, die der Dienst liefert
(SHA-256, SHA-1, MD5...), zusammen mit der Größe, und rclone muss im PATH sein.
Andernfalls, oder wenn das Remote keine Prüfsummen liefert (ein verschlüsseltes
etwa), nennt Xylem den Grund und fragt, bevor es die Dateien über die Einhängung
liest, mit der Menge, die es herunterladen würde.

Ganze Ordner werden nur auf lokalen Laufwerken verglichen.

Bei Google Drive:

- Eine Datei in zwei Ordnern zugleich oder eine Verknüpfung ist eine Datei: Sie wird
  nie gegen sich selbst angeboten.
- Docs, Sheets und Slides haben keine Prüfsumme: Die untere Zeile sagt, wie viele
  ausgelassen wurden.
- Was andere mit Ihnen geteilt haben, erscheint mit `s` und wird von „alle bis auf
  eine“ nicht markiert: Es zu löschen gibt nichts von Ihrem Speicher frei.
- Zwei Dateien mit einem Namen in einem Ordner lassen sich über die Einhängung nicht
  unterscheiden; die untere Zeile schlägt `rclone dedupe` vor.
- Löschen schickt in den Papierkorb des Drive, der mitzählt, bis er geleert wird.
  Xylem leert ihn nicht, denn das nähme alles darin mit; es zeigt den Befehl:
  `rclone cleanup gdrive:`.

Die Prüfsumme des Dienstes wird vertrauensvoll übernommen. MD5 lässt sich absichtlich
fälschen: Bei Dateien, die andere dort abgelegt haben könnten, vergleichen Sie vor
dem Löschen.
