# Die Maus

Xylem ist für die Tastatur gemacht, und die Maus ist aus, bis Sie sie
einschalten: **F12**, Seite 2, **R** (oder `mouse = true` in `[options]` der
[Konfigurationsdatei](configuration.md)), dann **F3** zum Speichern. Sie
funktioniert sofort.

Die Maus hat nie eigene Befehle. Ein Klick wählt eine Zeile oder steht für die
Taste, die dort steht, wo Sie geklickt haben, und diese Taste tut, was sie
immer tut: Delete fragt weiterhin nach, und ein Befehl, der hier nicht laufen
kann, wird weiterhin abgelehnt.

## Was sie tut

| Wo | Klick | Doppelklick | Mausrad |
|---|---|---|---|
| Der Baum | wählt das Verzeichnis; das Dateifenster folgt ihm | Enter: zu seinen Dateien | bewegt den Cursor um 3 Verzeichnisse |
| Das Zeichen vor einem Ordner | `+` öffnet ihn, `?` liest ihn ein und öffnet ihn, das `─` eines offenen schließt ihn | | |
| Das Dateifenster | wählt die Datei und wechselt ins Dateifenster | View, im eigenen Betrachter von Xylem | bewegt den Cursor um 3 Dateien (eine Zeile in den Spaltenmodi) |
| Die Bildlaufleiste am rechten Rand | über oder unter dem Schieber: eine Seite; den Schieber ziehen, um überallhin zu gelangen | | |
| Die andere Seite eines [geteilten Bildschirms](split.md) | macht sie zum aktiven Fenster, dann wählt er | | bewegt den Cursor dieser Seite, die aktive bleibt |
| Das [integrierte Terminal](terminal.md), der Editor im Rahmen | an das Programm, wenn es die Maus angefordert hat (mc, vim, htop, tmux); sonst nichts | | an das Programm, wenn es sie angefordert hat; sonst zurück durch die Zeilen, die aus dem Bild gelaufen sind, wie Shift+PgUp |
| Der Baum über dem integrierten Terminal | kehrt zu Xylem zurück, wie Alt+X | | |
| Die Fußzeile | die Taste des Eintrags: `Copy` ist C, `Esc cancel` ist Esc, `Yes` beantwortet die Frage | | |
| F12 | eine Option ändert sich, wie mit ihrem Buchstaben (nichts wird vor F3 gespeichert); ein Thema der Liste (F4) wird in der Vorschau gezeigt | das Thema wird gewählt | bewegt die Auswahl |
| Lesezeichen, das Suchfeld, Usage, Duplicates | wählt die Zeile | ihr Enter | 3 Zeilen |
| Der Verlauf einer Eingabe (↑) | setzt diesen Eintrag in die Eingabe | | 3 Zeilen |
| Der Text einer Eingabe | der Cursor geht dorthin | | |
| Eine Tabelle im Betrachter | wählt die Zelle | ihr Enter: der Datensatz | 3 Zeilen; Shift+Mausrad, eine Spalte |
| Der Betrachter, die Hilfe | einem Link der Hilfe wird gefolgt | | scrollt, wie ↑ und ↓; Shift+Mausrad scrollt den Betrachter seitwärts |
| Überall, wo ein Menü sinnvoll ist | die rechte Taste öffnet es (siehe unten) | | |

Ein Doppelklick öffnet eine Datei nie mit einem Programm des Systems: Eine
Datei, die Sie nicht selbst erstellt haben, kann feindselig sein, und ein
versehentlicher Klick darf sie nicht ausführen. Open (`O`) gibt es weiterhin.

Ziehen mit der linken Taste bewegt den Cursor entlang einer Liste; über ihren
oberen oder unteren Rand hinaus scrollt die Liste bei jeder Bewegung um eine
Zeile.

## Das Menü der rechten Taste

Die rechte Taste bewegt den Cursor auf das Element unter dem Mauszeiger, wie
ein Klick, und öffnet neben dem Zeiger ein Menü dessen, was sich damit tun
lässt. Die **Menü**-Taste der Tastatur oder **Shift+F10** öffnet dasselbe Menü
für das Element unter dem Cursor, ganz ohne Maus.

| Unter dem Zeiger | Das Menü |
|---|---|
| Eine Datei | View, Edit, Open with the system, Info; Tag oder Untag; Copy, Move, Rename, Delete, Attributes, New date, Yank the path |
| Eine Datei, mit markierten Dateien | dasselbe, und eine Gruppe für die markierten: Copy, Move, Rename (mask), Delete, View, Yank, Untag all |
| Ein Verzeichnis des Baums | Files, Log, Relog, Expand all, Collapse, Release; Branch, Showall; Make a directory, Rename, Delete, Prune, Attributes; Tag its files, Untag; Terminal here, Usage, Duplicates, Bookmark it, Yank the path |
| Das Dateifenster, nach seinen Dateien | Filespec, Sort, File display; Tag all, Untag all, Invert tags |
| Der Betrachter | Search, Next match, Go to line, und die Tasten seines Modus (Wrap, Line numbers, Hex, SQL filter...), Edit, Yank the path, Close |

Jede Zeile zeigt rechts ihre Taste, so wie Ihre Konfiguration sie hat; so ist
das Menü auch ein Weg, die Tasten zu lernen. Was hier nicht laufen kann, wird
weggelassen: In einem Archiv gibt es kein Delete, Move oder Rename. Eine Zeile
läuft wie ihre Taste: Delete fragt weiterhin zuerst.

↑↓ wählen, Enter führt aus, Esc schließt; ein Klick neben das Menü schließt es
nur.

## Markieren

| Geste | Was sie tut |
|---|---|
| Mittlere Taste, oder Ctrl+Klick, auf eine Datei | markiert sie oder hebt die Markierung auf; der Cursor bleibt, wo er ist |
| Dasselbe auf einen Ordner des Baums | wählt ihn und markiert seine Dateien, oder hebt die Markierung auf, wenn alle markiert waren |
| Shift+Klick auf eine Datei | markiert jede Datei vom Cursor bis zu dieser, und der Cursor geht dorthin |

Viele Terminals behalten Shift+Klick für ihre eigene Auswahl; dort markieren
die mittlere Taste oder Ctrl+Klick jede Datei einzeln.

## Der Fokus, die Menüs und die Fragen

Das Mausrad bewegt die Liste unter dem Zeiger, nicht die mit dem Fokus: Das
kleine Dateifenster unter dem Baum scrollt, ohne den Baum zu verlassen.

In den Ctrl- und Alt-Menüs (gehalten oder mit F4 geöffnet) steht ein Wort für
seinen Buchstaben mit Ctrl oder Alt: Ein Klick auf `Copy` im Ctrl-Menü kopiert
die markierten Dateien.

Solange eine Frage, eine Eingabe oder eine laufende Arbeit auf dem Bildschirm
ist, nehmen die Fenster dahinter keine Klicks an: nur ihre eigenen Einträge in
der Fußzeile.

## Text im Terminal auswählen

Mit eingeschalteter Maus sendet das Terminal die Klicks an Xylem, daher wählt
Ziehen keinen Text auf dem Bildschirm mehr aus. Halten Sie beim Ziehen
**Shift** gedrückt (in xterm, GNOME Terminal, Konsole, kitty, WezTerm, Windows
Terminal), oder **Option** in iTerm2. Um den Pfad einer Datei zu kopieren,
braucht Yank (`Y`) keine Maus.

## Terminals

Xylem fordert Klicks und Bewegungen mit gedrückter Taste an, im SGR-Format, das
jedes aktuelle Terminal kennt; nie die Bewegungen des Zeigers allein, die einen
Strom von Ereignissen für nichts senden würden, eine Last über SSH. Ein
Programm im integrierten Terminal oder im Editor im Rahmen bekommt die Maus in
dem Format, das es angefordert hat, mit den Ecken seines eigenen Bereichs: Sie
erreicht nie den Rest des Bildschirms und auch nicht Ihr Terminal. Ein Ziehen,
das den Bereich verlässt, bleibt beim Programm, an seinem Rand gehalten. Ein
Programm, das über eXecute oder Edit läuft, kann die Maus beim Beenden
ausschalten; Xylem schaltet sie wieder ein, wenn es den Bildschirm
zurückbekommt.
