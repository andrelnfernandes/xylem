# Die Menüs, Strg und Alt

Die Fußzeile listet die Befehle auf, die im aktiven Fenster zur Verfügung stehen.
Es gibt vier Menüs. Der Titel auf der linken Seite gibt an, welches angezeigt
wird.

| Titel | Angezeigt wenn | Seine Befehle wirken auf |
|---|---|---|
| DIR | der Baum den Fokus hat | das Verzeichnis unter dem Cursor |
| FILE | das Dateifenster den Fokus hat | die Datei unter dem Cursor |
| CTRL | Ctrl gehalten wird, oder nach F9 | die markierten Dateien |
| ALT | Alt gehalten wird, oder nach zweimal F9 | siehe [das Alt-Menü](#alt-die-andere-fassung-eines-befehls) |

In einem Archiv lautet der Titel ARCHIVE, auf einem entfernten Rechner REMOTE.
Beide lassen die Befehle weg, die dort nicht funktionieren.

## Ein Menü lesen

Jedes Wort ist ein Befehl, und sein Großbuchstabe ist die Taste: `Copy` ist `C`,
`eXecute` ist `X`, `iNdex` ist `N`. In den Menüs CTRL und ALT ist die Taste
dieser Buchstabe mit Ctrl oder Alt.

Die Befehle sind alphabetisch nach ihrer Taste geordnet, mit Quit am Ende.
Darunter befindet sich eine Zeile mit den Funktionstasten und Symbolen
(`F3 relog`, `? stats`).

Die Fußzeile umfasst drei Zeilen. Auf einem Bildschirm, der breit genug ist, um
alle Wörter in eine Zeile zu fassen, hat sie zwei Zeilen. Auf einem schmalen
Bildschirm werden die Wörter gleichmäßig gekürzt, und nicht passende Einträge
werden durch `…` ersetzt.

## Ctrl: die markierten Dateien

Ein Buchstabe allein wirkt auf ein einzelnes Element. Derselbe Buchstabe mit
Ctrl wirkt auf die markierten Dateien der Liste:

| Taste | Allein | Mit Ctrl |
|---|---|---|
| `C` | kopiert die Datei | kopiert die markierten Dateien |
| `D` | löscht die Datei | löscht die markierten Dateien |
| `R` | benennt die Datei um | benennt die markierten Dateien mit einer Maske um |
| `T` | markiert die Datei | markiert alle Dateien in der Liste |

Im Baum bezieht sich „die Liste" auf das Verzeichnis unter dem Cursor. Drei
Befehle haben dort eine größere Reichweite: Ctrl+T und Ctrl+U markieren und
heben die Markierung aller eingelesenen Dateien auf, die dem Filespec
entsprechen, und Ctrl+S listet die markierten Dateien von allem Eingelesenen auf.

[Das Strg-Menü](commands-ctrl.md) beschreibt jeden Befehl.

## Alt: die andere Fassung eines Befehls

Alt wiederholt Ctrl nicht. Seine Wirkung hängt vom aktiven Fenster ab:

- Im Baum deckt der Befehl den gesamten Zweig ab. Alt+P (Prune) löscht das
  Verzeichnis samt Inhalt, Alt+R gibt den Zweig aus dem Speicher frei, Alt+C
  vergleicht den Zweig mit dem anderen Fenster.
- Im Dateifenster kopieren und verschieben Alt+C und Alt+M die markierten
  Dateien und stellen den Pfad jeder Datei unter dem Ziel wieder her.
- In beiden Fenstern öffnet Alt die Werkzeuge, die nicht auf eine einzelne Datei
  bezogen sind: Alt+S (Sort), Alt+F (File display), Alt+B (Bookmarks), Alt+I
  (Info), Alt+X (das integrierte Terminal).

[Das Alt-Menü](commands-alt.md) beschreibt jeden Befehl.

## Die Menüs Ctrl und Alt anzeigen

**F9** zeigt das CTRL-Menü an, ein weiteres F9 das ALT-Menü, und ein drittes
kehrt zurück. Während eines dieser Menüs sichtbar ist, gilt der nächste
Buchstabe mit diesem Modifikator: `F9` `C` entspricht Ctrl+C. Esc bricht ab,
ohne etwas auszuführen.

Unter kitty, Ghostty, WezTerm, foot und aktuellem Alacritty zeigt das Halten von
Ctrl oder Alt das Menü an, und das Loslassen bringt DIR oder FILE zurück. Diese
Terminals melden das isolierte Drücken einer Modifikatortaste
(Kitty-Tastaturprotokoll). GNOME Terminal, Ptyxis, Terminator und Tilix tun dies
nicht; dort wechseln die Menüs daher nur mit F9.

## Ctrl+M

Die meisten Terminals senden für Ctrl+M und Enter dasselbe Byte. Xylem kann sie
dann nicht unterscheiden. In Terminals mit dem Kitty-Tastaturprotokoll sind die
beiden Tasten getrennt und Ctrl+M verschiebt die markierten Dateien. In den
übrigen drücken Sie `F9` `M`.

## Geändert in Version 0.8

Alt+A, Alt+D, Alt+R, Alt+T und Alt+U bewirkten früher dasselbe wie Ctrl mit dem
gleichen Buchstaben. [Versionshinweise](releases.md) enthält die alte und neue
Bedeutung jeder Taste sowie die Zeilen für `config.toml`, die die alten wieder
herstellen.
