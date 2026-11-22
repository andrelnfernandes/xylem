# Konfiguration

## Der Konfigurationsbildschirm (F12)

**F12** (oder Alt+F10; unter GNOME maximiert Alt+F10 meist das
Fenster und erreicht Xylem nicht) öffnet fünf Seiten:

| Seite | Optionen |
|---|---|
| 1. Programme und Formate | Editor, Editor im Xylem-Rahmen öffnen, Xylem-Thema im Editor, Binärdateien mit E bearbeiten, Binäreditor, Hilfesprache, Programm zum Öffnen (Opener), Datumsformat, Zahlenformat, Git-Befehle, vertrauenswürdige Git-Ordner |
| 2. Anzeige und Sortierung | versteckte Dateien, anfängliches Sortierkriterium und Richtung, Verhalten von Enter im Dateifenster, anfänglicher Anzeigemodus, Syntaxfarben, Zeilenumbruch, Zeilennummern, komprimierte Dateien, die Kodierung alter Texte, das Größenlimit dessen, was der Betrachter liest, Icons und Symbole (Nerd Fonts), Dateinamen nach Typ gefärbt, Fensterrahmen, Farben des integrierten Terminals, Sekunden in der Uhr, [die Maus](mouse.md) |
| 3. Bestätigungen und Verlauf | beim Beenden nachfragen, in den Papierkorb löschen, Prune-Bestätigung, Einträge mit `@` nicht speichern, eigener Verlauf des integrierten Terminals, [Alt+T in der unscharfen Suche markiert jeden Treffer](#tagging-every-match-of-the-fuzzy-find), [nach einer neuen Version suchen](#new-versions) |
| 4. Kopieren, Vergleichen und Aktualisieren | 2 s ignorieren, 1 Std. ignorieren, automatische Aktualisierung, Startpunkt der Pfade bei Alt+C/Alt+M, vom Index ausgenommene Dateien und Ordner |
| 5. Lesen (E-Books) | die Breite der Textspalte, Absätze wie in einem Buch oder in Blöcken, Blocksatz, die Lesegeschwindigkeit für die verbleibende Zeit, [die Lesezeit](#reading-time), die Minuten ohne Taste, bevor ihre Uhr anhält, die für eine Liste der Bücher behaltenen Titel |

Das Größenlimit des Betrachters (**L** auf Seite 2) kostet Speicher und Zeit: Mit `1024` MiB belegt das Öffnen einer gewöhnlichen Textdatei dieser Größe etwa 2,5 GiB RAM und dauert rund 5 s, und Dateien mit sehr kurzen Zeilen kosten mehr pro Byte (siehe [den Betrachter](viewer.md)).

Icons und Symbole (**M** auf Seite 2) erfordern eine mit Nerd Fonts erweiterte Schriftart. Sie benötigen keine bestimmte Schriftart: Jede dicktengleiche Schrift funktioniert (wie Fira Code, JetBrains Mono, Hack, Meslo oder Cascadia Code), solange Sie die jeweilige „Nerd Font“-Variante wählen. Nerd Fonts fügt tausende Standard-Symbole in den privaten Unicode-Bereich ein. Herkömmliche Schriftarten ohne diese Erweiterung (selbst die originale Fira Code) enthalten diese Symbole nicht und zeigen leere Kästchen (`□`). So richten Sie sie ein:
- Laden Sie eine beliebige Nerd Font Ihrer Wahl von nerdfonts.com herunter oder installieren Sie ein Nerd-Fonts-Paket über Ihre Paketverwaltung.
- Wählen Sie diese Schriftart in den Einstellungen Ihres Terminal-Emulators aus.

Verrutschen die Spalten bei eingeschalteten Icons, stellt das Terminal vermutlich Zeichen „mehrdeutiger Breite“ breit dar (eine bei chinesischen, japanischen und koreanischen Schriften übliche Einstellung), während Xylem jedes Icon als eine Zelle zählt. Stellen Sie diese Option auf schmal oder schalten Sie die Icons aus.

Dateinamen nach Typ gefärbt (**N** auf Seite 2) gibt Archiven, Bildern, Audio und Video, Dokumenten, Code und Daten je eine Farbe des Themes, allein nach dem Namen (es wird nichts gelesen). Ausgeschaltet sieht die Liste klassisch aus. Fensterrahmen (**O**) sind in den Themes nord, gruvbox, slate, graphite, daylight und vivid abgerundet und in den anderen gerade; `plain` oder `rounded` legt sie in jedem Theme fest. Der Startbildschirm (**P**) zeigt den Namen zwei Sekunden lang in Blockbuchstaben, wenn Xylem startet; jede Taste schließt ihn (diese Taste bewirkt sonst nichts), und der Startordner wird währenddessen gelesen. Er erscheint nicht, wenn über die Befehlszeile eine Verbindung zu einem entfernten Rechner oder Container aufgebaut wird, und nicht in einem Terminal kleiner als 58 mal 13.

Der Buchstabe der Option (oder Enter) ändert den Wert, PgUp/PgDn oder 1–5
wechseln die Seite, **F4** wählt das Thema aus einer Liste (mit sofortiger
Vorschau), **F3** speichert und wendet die Einstellungen an, **Esc** bricht ab
(und fragt bei ungespeicherten Änderungen nach).

### Jeden Treffer der unscharfen Suche markieren

In der unscharfen Suche (`|`) markiert **Alt+T** die Ergebnisse der Liste.
Die Liste behält die 500 besten Treffer, und eine Suche kann viel mehr
treffen: `f` kann in einem großen Baum hunderttausend Dateien treffen.
Standardmäßig markiert Alt+T nur die 500 der Liste, und wenn mehr getroffen
wurden, sagt die Zeile unter der Liste das und nennt diese Option.

Mit **Alt+T in der unscharfen Suche markiert jeden Treffer** eingeschaltet
(**F** auf Seite 3, `fuzzy_tag_all = true` in der Datei) markiert Alt+T jeden
Treffer, auch die, die Sie nie gesehen haben. Die Zeile unter der Liste sagt
dann immer, wie viele Treffer nicht darin waren.

Das ist standardmäßig aus, weil Markierungen in das übergehen, was Sie als
Nächstes tun. Ctrl+C, Ctrl+M, Ctrl+D und Batch wirken auf jede markierte
Datei, nicht nur auf die, die Sie gesehen haben. Tausende Dateien, markiert
von einer Suche, auf die Sie nur kurz geschaut haben, und dann gelöscht, sind
tausende verlorene Dateien. Schalten Sie die Option nur ein, wenn Sie prüfen,
was markiert ist, bevor Sie darauf handeln:

- das Statistikfenster zeigt, wie viele Dateien markiert sind, und ihre Größe;
- **Ctrl+S** im Baum listet die markierten Dateien des ganzen Baums auf (ein
  Showall der markierten);
- **Ctrl+V** im Dateifenster zeigt sie nacheinander an.

### Neue Versionen

Beim ersten Start fragt Xylem, sobald der Startbildschirm verschwunden ist:
*Look for a new version of Xylem once a day?* **Y** sagt ja, **N** nein, und
die Antwort wird in der Konfigurationsdatei gespeichert (`check_updates`).
**Esc** speichert nichts, und die Frage kommt beim nächsten Mal wieder. **G**
auf Seite 3 ändert die Antwort später.

Ist sie eingeschaltet, fragt Xylem einmal am Tag die Website
(`https://andrelnfernandes.github.io/xylem/latest.txt`) nach einer Zeile: der
Nummer der neuesten Version. Das geschieht über das `curl` des Systems, im
Hintergrund, nur mit HTTPS und höchstens fünf Sekunden lang; ohne Netzwerk
oder ohne `curl` wird nichts gesagt, und am nächsten Tag folgt ein neuer
Versuch. Die Anfrage enthält nicht mehr als `xylem/` und die Version, die Sie
verwenden; die Website sieht Ihre IP-Adresse, wie bei jedem Besuch. Nichts
wird heruntergeladen oder installiert: Gibt es eine neuere Version, zeigt die
oberste Zeile `0.10.0 is out` neben der Uhr, und eine Meldung nennt die
Adresse der Releases. Aktualisieren Sie so, wie Sie installiert haben: mit
der Paketverwaltung bei einem `.deb` oder `.rpm`, sonst mit einem neuen
Archiv.

Die Antwort der Website wird als feindlich gelesen: Alles außer drei Zahlen,
etwa die Seite des WLANs eines Hotels, wird ignoriert. Das Datum der letzten
Prüfung und die gefundene Version werden im Zustandsordner gespeichert
(`~/.local/state/xylem/update-check`).

`XYLEM_CHECK_UPDATES=0` (oder `1`) in der Umgebung antwortet, ohne zu fragen,
und hat Vorrang vor der Datei, für Installationen auf vielen Rechnern. Ein
Build ohne das Feature `update-check` (`cargo build --no-default-features --features
index`), wie es eine Distribution erstellen kann, hat keine Frage, keine
Prüfung und keine Option.

### Lesezeit

Der Betrachter liest [E-Books](viewer.md#ebooks) im Format EPUB und FB2.
Seite 5 legt fest, wie der Text angeordnet wird und ob Xylem die Zeit
speichert, die Sie mit Lesen verbringen.

**Die Lesezeit speichern** (**E**, `reading_stats = true` in der Datei) ist
standardmäßig aus. Eingeschaltet speichert Xylem für jedes mit `R` im
Betrachter gemerkte Buch:

- die gelesene Zeit und die Sitzungen: Eine Pause von mehr als 30 Minuten
  beginnt eine neue;
- die weiteste erreichte Stelle, den ersten und den letzten Lesetag und ob
  das Buch zu Ende gelesen wurde;
- die Lesegeschwindigkeit in Wörtern pro Minute. Nur vorwärts gelesene Seiten
  zählen, und nur, wenn mindestens 5 Sekunden auf ihnen verbracht wurden,
  sodass Sprünge, Zurückgehen und Überfliegen nicht zählen. Nach 10
  gemessenen Minuten berechnet die Fußzeile die verbleibende Zeit mit der
  Geschwindigkeit dieses Buchs statt mit der auf dieser Seite (**D**).

Die Uhr läuft nur, während Sie lesen: von einer Taste zur nächsten, wenn
dazwischen nicht mehr als **F** Minuten (standardmäßig 3) vergangen sind.
Eine längere Pause hält sie an, und diese Pause zählt nicht. Autoview (F7)
und der Bericht (`I`) zählen nie.

Alles bleibt auf Ihrem Rechner, in `~/.local/state/xylem/reading.toml` (oder
`$XDG_STATE_HOME/xylem/`), nur für Sie lesbar. Ein Buch wird als Hash dessen
gespeichert, was es zu sein angibt, und seiner Größe, dazu Zahlen: nicht sein
Name, nicht sein Ort, nicht sein Titel. Mit **G** eingeschaltet
(`reading_titles = true`) werden auch Titel und Autor gespeichert, und der
Bericht jedes Buchs listet dann alle gelesenen Bücher unter „Your books“ auf.

**G** auszuschalten und zu speichern (F3) löscht die gespeicherten Titel
sofort. **E** auszuschalten und zu speichern fragt, ob die Lesezeit aller
Bücher gelöscht werden soll; ohne das bleibt sie in der Datei und wird nicht
gezeigt. In beiden Fällen bleiben die mit `R` gemerkten Stellen; `R` auf
einem Buch vergisst es.

## Die Konfigurationsdatei

Alle Einstellungen werden in `~/.config/xylem/config.toml` gespeichert (oder
unter `$XDG_CONFIG_HOME/xylem/`; unter Windows in `%APPDATA%\xylem\`). Die
Datei kann auch von Hand bearbeitet werden: Der Konfigurationsbildschirm lässt
Ihre Kommentare unangetastet. Um mit einer kommentierten Datei samt allen
Standardwerten zu beginnen:

```bash
mkdir -p ~/.config/xylem && xy --default-config > ~/.config/xylem/config.toml
```

- `theme`: `classic` (Weiß und Gelb auf Blau), `dark`, `light`, `mono` (die eigenen Farben
  des Terminals), `norton`, `solarized`, `gruvbox`, `nord`, `slate` (im Stil
  von Windows Terminal), `graphite` (macOS im Dunkelmodus), `daylight` (macOS
  im Hellmodus) oder `vivid` (kräftige Farben für Präsentationen und
  Screenshots). F4 zeigt alle Themen mit einer Vorschau an.
- `[options]`: die Optionen des Konfigurationsbildschirms und einige, die es
  nur in der Datei gibt: `scan_max_files` (10000), die Höchstzahl der Dateien,
  deren Inhalt der [Scan](views.md#scan) liest.
- `[colors]`: einzelne Farben aufbauend auf dem Thema, als Namen (`"yellow"`),
  `"#rrggbb"` oder Farbindex von 0 bis 255.
- `[keys.any]`, `[keys.tree]`, `[keys.files]`: Tasten in beiden Fenstern oder
  nur in einem Fenster neu belegen (letzteres hat Vorrang). Beispiel:
  `"f3" = "view"`, `"ctrl+r" = "relog"`; `"none"` deaktiviert eine Taste. Die
  Tastenliste der Hilfe (**K**) zeigt die tatsächlich aktiven Tasten
  einschließlich Ihrer Anpassungen.
- `[editors]`: der Editor für jeden Dateityp (`"*.png *.jpg" = "gimp %f"`,
  `binary = "hexedit"`); siehe [Bearbeiten](execute.md).

Ein Fehler in der Datei hindert Xylem nicht am Starten: Es verwendet dann die
Standardwerte, zeigt den Fehler in der Fußzeile an, und der
Konfigurationsbildschirm speichert erst wieder, wenn die Datei korrigiert
wurde, damit Ihre manuellen Einträge nicht überschrieben werden.

Bis Version 0.1 hieß Xylem noch xt. Beim ersten Start verschiebt es die
Konfiguration, den Verlauf und die Lesezeichen von xt automatisch in seine
eigenen Ordner.
