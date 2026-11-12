# Lesezeichen

Lesezeichen bringen Sie mit jeweils einer Taste (`0`–`9`, `a`–`z`) direkt zu
häufig genutzten Verzeichnissen zurück. Sie bleiben zwischen Programmstarts
zusammen mit dem Eingabeverlauf gespeichert.

- **Ctrl+F11** (oder Ctrl+B im Baum) speichert das aktuelle Verzeichnis:
  Drücken Sie die Taste, der es zugewiesen werden soll.
- **`'`** springt zu einem Lesezeichen: Drücken Sie die Taste des Lesezeichens.
  In derselben Eingabezeile öffnet `'` oder Enter die Liste und `+` speichert
  das aktuelle Verzeichnis.
- **F11** (oder Alt+B) öffnet die Lesezeichenverwaltung: ↑ ↓ wählen aus, Enter
  (oder die Taste des Lesezeichens) springt hin, **F2** gibt ihm einen Namen,
  Del entfernt den Eintrag, Insert oder `+` speichert das aktuelle Verzeichnis,
  Esc schließt das Fenster.
- **F11 in den Eingaben von [Log](treespec.md), [Copy und Move](copy-move.md)**
  öffnet dieselbe Liste, um einen Ort zu wählen: Enter (oder die Taste des
  Lesezeichens) setzt ihn in die Eingabe, wo Sie ihn noch bestätigen; Esc kehrt
  zu dem zurück, was Sie getippt hatten.

Ein Verzeichnis außerhalb des aktuellen Baums wird wie bei [Log](treespec.md)
als neuer Baum geöffnet. Im Dateifenster ist Ctrl+B mit [Batch](execute.md)
belegt; verwenden Sie dort Ctrl+F11.

## Namen

Ein Name sagt, wofür ein Lesezeichen da ist: „Oracle-Server“, „Postgres-Container
Umg. 1“. In der Verwaltung fragt **F2** auf einem Lesezeichen nach dem Namen,
mit dem aktuellen zum Bearbeiten; ein leerer Name entfernt ihn. Der Name steht
vor dem Ort, und der Ort steht immer daneben, damit ein Name nicht verbirgt,
wohin ein Lesezeichen führt. Ein Name hat höchstens 60 Zeichen, ohne Steuer-
und Schreibrichtungszeichen.

## Entfernte Rechner und Container

In einem Fenster auf einem [entfernten Rechner](remote.md) oder in einem
Container speichert Ctrl+F11 den Ordner unter der Adresse, die ihn öffnet:
`ssh://oracle@db01/u01/app`, `docker://postgres-umg1/var/lib/postgresql`. Dorthin
zu gehen verbindet wie das Eintippen der Adresse in Log und fragt nach dem
Passwort, wenn ssh eines braucht; kein Passwort wird gespeichert. Ist das
Fenster schon auf diesem Rechner und der Ordner in seinem Baum, bewegt sich nur
der Cursor. Ein Ordner, dessen Pfad ein Leerzeichen enthält, lässt sich so nicht
speichern, da eine Adresse keine Leerzeichen aufnimmt.

Die Verwaltung markiert einen lokalen Ordner, der nicht mehr existiert, mit
`[not found]`. Entfernte Lesezeichen prüft sie nicht, ebenso wenig Ordner in
einem Cloud-Mount oder auf einer Netzwerkfreigabe: Das würde auf das Netzwerk
warten.
