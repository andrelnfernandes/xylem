# Funktionstasten und Symbole

Diese Tasten stehen in der letzten Zeile der Menüs DIR und FILE, oder sie fehlen
in den Menüs, weil sie lediglich den Cursor bewegen. Sie werden nicht mit den
F9-Menüs kombiniert: nach F9 nimmt nur der nächste Buchstabe Ctrl oder Alt an.

## Funktionstasten

| Taste | Wo | Funktion |
|---|---|---|
| F1 | beide | Öffnet die Hilfe zu dem, was auf dem Bildschirm zu sehen ist. Alt+H bewirkt dasselbe. [Terminals und Tasten](terminals.md) |
| F2 | beide | Liest das Verzeichnis erneut ein, wie F3. In einer Eingabe, die nach einem Ordner fragt, öffnet sich ein Baum zur [Auswahl](picker.md). |
| F3 | beide | Liest das Verzeichnis erneut von der Festplatte ein. |
| F5 | Dateien | Erstellt ein Archiv mit der Datei. Ctrl+F5: mit den markierten Dateien. Alt+F5: öffnet ein Archiv. [Archive](archives.md) |
| F7 | Dateien | Autoview: zeigt den Inhalt der Datei unter dem Cursor neben der Liste an. [Das Dateifenster](files.md) |
| F8 | beide | Öffnet ein zweites Fenster neben dem ersten oder schließt es. [Geteilter Bildschirm](split.md) |
| F9 | beide | Zeigt das CTRL-Menü an, dann das ALT-Menü, und kehrt zurück. [Die Menüs](menus.md) |
| F10 | beide | Beendet das Programm, wie `Q`. |
| F11 | beide | Öffnet die Lesezeichenverwaltung, wie Alt+B. Ctrl+F11 speichert das aktuelle Verzeichnis. [Lesezeichen](bookmarks.md) |
| F12 | beide | Öffnet die Konfiguration. [Konfiguration](configuration.md) |

## Symbole

| Taste | Wo | Funktion |
|---|---|---|
| `*` | Baum | Liest das Verzeichnis unter dem Cursor und alles darunter ein. Esc bricht ab. |
| `+` oder → | Baum | Liest das Verzeichnis ein und zeigt seine Unterverzeichnisse an. |
| `-` | Baum | Gibt den Zweig aus dem Speicher frei. |
| ← | Baum | Klappt das Verzeichnis zu oder wechselt zum übergeordneten Verzeichnis. |
| `?` | beide | Zeigt Statistiken zu Fenster, Festplatte, Rechner und Xylem an. [Informationen](info.md) |
| `\` oder `/` | beide | Treespec: Pfad eintippen und der Baum-Cursor folgt ihm. In Branch oder Showall wird das Verzeichnis der Datei unter dem Cursor angesprungen. [Treespec und Log](treespec.md) |
| `'` | beide | Zu einem Lesezeichen springen: anschließend dessen Taste drücken. |

## Navigation

| Taste | Wo | Funktion |
|---|---|---|
| ↑ ↓ PgUp PgDn Home End | beide | Cursor bewegen. |
| Leertaste | beide | Eine Zeile nach unten. |
| Enter | Baum | Wechselt in das Dateifenster. |
| Enter | Dateien | Kehrt zum Baum zurück. Bei aktivem `enter_views` wird die Datei angezeigt. |
| Esc | Dateien | Kehrt zum Baum zurück, beim Verzeichnis der Datei unter dem Cursor. |
| Esc | Baum | Schließt ein geöffnetes Archiv. |
| Tab | beide | Wechselt bei geteiltem Bildschirm zum anderen Fenster. Andernfalls wie Enter. |
| Backspace | beide | Wechselt zum übergeordneten Verzeichnis. |
| Shift+Buchstabe, 0–9 | beide | Wechselt zum nächsten Verzeichnis oder zur nächsten Datei mit diesem Anfangszeichen. |
| ← → | Dateien | Seitenweise nach oben und unten. In Spaltenmodi vorherige und nächste Datei. |

## Eine Taste umbelegen

`[keys.any]`, `[keys.tree]` und `[keys.files]` in `config.toml` weisen einer
Taste einen anderen Befehl oder `none` zu. **K** in der Hilfe listet die
gültigen Tasten samt dieser Änderungen auf. Siehe
[Konfiguration](configuration.md).

Siehe auch: [die Menüs DIR und FILE](commands.md),
[das Ctrl-Menü](commands-ctrl.md), [das Alt-Menü](commands-alt.md).
