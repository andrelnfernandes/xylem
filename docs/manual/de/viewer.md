# Der Betrachter

`V` öffnet die Datei im Betrachter (Autoview, F7, zeigt sie neben der Liste
an). Binärdateien werden im Hex-Modus geöffnet. Eine große Datei auf der Festplatte wird nicht ganz gelesen: Der Betrachter liest, was der Bildschirm zeigt, während er weiterläuft (siehe [Große Dateien](#große-dateien)). Was sich so nicht lesen lässt, zeigt nur seinen Anfang, bis zum Größenlimit.
Tabellenkalkulationen, CSV, DBF, SQLite und Parquet öffnen sich als Tabelle (siehe [Tabellen](tables.md)).

| Taste | Funktion |
|---|---|
| ↑ ↓ PgUp PgDn Home End | scrollen |
| ← → | seitlich scrollen |
| `H` | nächster Modus: Text und Hex; bei einer Markdown-Datei gerendert, Text und Hex |
| `O` | nächste Kodierung (UTF-8, CP850, 1252, CP437, UTF-16) |
| `C` | Syntaxfarben ein oder aus |
| `W` | Zeilenumbruch ein oder aus |
| `#` oder `L` | Zeilennummern ein oder aus (im Textmodus) |
| `g` oder `:` | zu einer Zeile springen (Byte-Offset im Hex-Modus, Tabellenzeile im Tabellenmodus) |
| `&` | zeigt nur passende Zeilen, einfacher Text oder Regex ([Zeilen filtern](filter.md)); im Tabellenmodus der SQL-Filter |
| Ctrl+B | eine Ausgabezeile pro angezeigtem Datensatz oder Zeile ([Batch im Betrachter](execute.md)) |
| `E` | Datei bearbeiten; zurück in Xylem zeigt der Betrachter den neuen Inhalt an derselben Stelle (Binärdateien nur mit einem Editor unter [`[editors]`](execute.md)) |
| `D` | bei aktiviertem Git: Änderungen seit dem letzten Commit und zurück ([Git](git.md)) |
| `T` | Tabelle oder Text, bei einer Datei, die sich als Tabelle geöffnet hat; gerendert oder Quelltext, bei einer Markdown-Datei |
| `U` | in der Ansicht der markierten Dateien (Ctrl+V): Markierung der Datei aufheben |
| `y` oder Ctrl+Insert | Dateipfad in die Zwischenablage kopieren |
| Esc | schließen |

## Suchen

**F** sucht vom Anfang an, **B** rückwärts vom Ende aus, **/**
(oder `S`) ab der aktuellen Seite und **\\** von dort rückwärts; **Space**
sucht in derselben Richtung weiter, **+** und **-** wählen die Richtung. Der
Treffer wird hervorgehoben. In der Sucheingabezeile schaltet **F2** die
Groß-/Kleinschreibung um (wird sie ignoriert, gilt das auch für Buchstaben mit Akzent: `AÇÃO` findet
`ação`) und **F4** wechselt zwischen Text und Hex
(Byte-Paare wie `4d 5a`). Ein `*` in der Mitte des Suchbegriffs steht für
beliebige Zeichen in derselben Zeile (`error*disk`).

**F3** kehrt die Richtung der Suche um, sodass F allein auch tut, was B tut: vorwärts vom Anfang oder rückwärts vom Ende.

## Syntaxfarben

Quellcode und Skripte werden in rund 150 Sprachen farbig hervorgehoben (Rust,
Python, Shell, C, JavaScript, SQL, YAML, Markdown...), erkannt anhand der
Erweiterung, des Namens (`Makefile`) oder der ersten Zeile (`#!/bin/bash`). Die
Standardeinstellung ist `syntax_highlight` (Seite 2 der
[Konfiguration](configuration.md)). Die Farben bleiben auf dem Hintergrund des
aktuellen Themas gut lesbar, und es werden nur die ersten 20.000 Zeilen
eingefärbt und keine Zeile mit mehr als 10.000 Zeichen, sodass auch riesige
Dateien den Betrachter niemals ausbremsen.

## Gerendertes Markdown

`.md`- und `.markdown`-Dateien öffnen sich gerendert, auch in Autoview, in Archiven und komprimiert (`x.md.gz`), und der Rahmen zeigt `RENDERED`. `T` zeigt den Quelltext an derselben Stelle und wechselt zurück; `H` geht durch gerendert, Text und Hex.

Zu sehen sind Überschriften, Fett, Kursiv, Durchgestrichen, Code, Codeblöcke mit Syntaxfarben (`C` schaltet sie aus), verschachtelte und nummerierte Listen, Aufgabenlisten (`[x]`), Zitate, Trennlinien und Tabellen. Nichts wird je heruntergeladen, und kein Link öffnet etwas:

- Ein Link zeigt seine echte Adresse neben dem Text, etwa `sicher <https://example.com>`, damit der Text nicht als andere Adresse durchgeht.
- Ein Bild ist eine Textzeile: `[image] Beschriftung <foto.png>`.
- Ein Mermaid-Diagramm ist ein Codeblock mit der Beschriftung `mermaid`.
- HTML und HTML-Kommentare bleiben als Text sichtbar.
- Der YAML-Block am Anfang einer Datei ist ein Codeblock mit der Beschriftung `yaml`.

Der Text passt immer ins Fenster, daher bewirkt `W` hier nichts; ← → erreichen die Spalten einer zu breiten Tabelle. Die Suche betrachtet den Text auf dem Bildschirm, einschließlich der Adressen, und findet auch einen Satz, der in der nächsten Reihe weitergeht. `g` und `#` zählen die Zeilen des Quelltexts. Ctrl+B arbeitet auf den Zeilen des Quelltexts, auf allen. `&` wechselt in den Textmodus und filtert dort.

Eine Datei über 8 MiB öffnet sich als Quelltext, mit einem Hinweis. Das Rendern stoppt auch am Zeitlimit (3 s, 100 ms in Autoview) oder an seinem Speicherlimit, und die Kopfzeile zeigt dann `(rendered only the start)`. Eine Tabelle zeigt höchstens 256 Spalten. Steuerzeichen erscheinen als `·`, auch solche, die als Entitäten wie `&#27;` geschrieben sind.

Markdown selbst verbirgt einiges: Linkdefinitionen, die nichts verwendet, und Tabellenzellen jenseits der Spalten der Kopfzeile. Um alles zu sehen, zeigt `T` den Quelltext. Kursiv und Durchgestrichen hängen vom Terminal und seiner Schrift ab, daher haben sie auch eine eigene Farbe.

## JSON, YAML und TOML

Eine `.json` (auch `.geojson`, `.har`, `.webmanifest`, sowie `.jsonc` und `.json5` mit Kommentaren und abschließendem Komma), eine `.yaml` oder `.yml` und eine `.toml` (auch `Cargo.lock`) öffnen sich als Baum, der sich auf- und zuklappen lässt, und der Rahmen zeigt `TREE: JSON`. Eine minifizierte Datei in einer einzigen Zeile liest sich so leicht wie jede andere. Ein Baum mit bis zu 2.000 Knoten öffnet sich ganz aufgeklappt; ein größerer zeigt seine erste Ebene.

| Taste | Aktion |
|---|---|
| ↑ ↓ Bild↑ Bild↓ Pos1 Ende | bewegt den Cursor |
| → | klappt den Knoten auf (nochmals: geht hinein) |
| ← | klappt den Knoten zu oder geht zum übergeordneten |
| `Enter` | klappt den Knoten auf oder zu |
| `*` | klappt alles unter dem Knoten auf |
| `k` | kopiert den Pfad des Knotens: `.servers[2].name` in JSON, `servers.2.name` in YAML und TOML. Ein Schlüssel mit einem Zeilenende, einem Steuerzeichen oder einem Zeichen der Textrichtung wird in Anführungszeichen gesetzt, mit diesen Zeichen maskiert (`\n`, `\u202e`), damit das Einfügen des Pfads in eine Shell nie eine Zeile davon ausführt |

Die Fußzeile zeigt den Pfad des Knotens unter dem Cursor. Eine Suche (`F`, `B`, `/`) durchsucht Schlüssel und Werte, auch in zugeklappten Knoten, und öffnet den Weg zum Fund. `T` zeigt den Text in der Zeile des Knotens und kehrt zurück; `H` wechselt zwischen Baum, Text und Hex. `&`, `g`, `#` und Strg+B arbeiten mit den Zeilen des Textes.

- Ein zugeklappter Knoten zeigt, was er enthält: `{12 keys}`, `[340 items]`.
- Eine Zahl erscheint so, wie die Datei sie schreibt: `12345678901234567890` bleibt so, und `1_000` in TOML ebenfalls.
- Eine Zeichenkette zeigt einen Zeilenumbruch als `↵`.
- Ein Schlüssel, der in einem Objekt zweimal vorkommt, wird zweimal gezeigt, mit `dup key` markiert, und die Kopfzeile zählt sie. Programme sind sich uneinig, welcher der beiden gilt, was bei einer Prüfung einen Wert verstecken kann.
- Ein YAML-Anker erscheint als `&a1` und ein Alias als `*a1`: Ein Alias wird nie aufgelöst, sodass eine Datei, die sich vervielfachen soll (ein „Billion Laughs“), nicht mehr kostet als ihr Text. Ein Tag (`!Ref`) steht vor seinem Wert. Mehrere Dokumente stehen unter einer Wurzel, als `document 1`, `document 2`...

Eine ungültige Datei oder eine, die tiefer als 512 Ebenen verschachtelt ist, öffnet sich als Text, und der Hinweis sagt, warum und in welcher Zeile. Der Baum wird aus der ganzen Datei gebaut: Eine Datei über dem Leselimit öffnet sich als Text. Der Baum hält außerdem bei 2.000.000 Knoten an (etwa 170 MiB Speicher) oder wenn sein Aufbau zu lange dauert; er enthält dann nur den Anfang der Datei, und die Kopfzeile zeigt `(tree of the start only)`. `T` zeigt weiterhin den ganzen Text.

## Jupyter-Notebooks

Eine `.ipynb` öffnet sich gerendert, Zelle für Zelle, und der Rahmen zeigt `RENDERED`. Oben: der Kernel und wie viele Zellen jeder Art es gibt.

- Eine Markdown-Zelle wird wie eine Markdown-Datei gerendert.
- Eine Code-Zelle zeigt `In [n]:` und ihren Code, in den Farben der Sprache des Notebooks.
- Ihre Ausgaben folgen, markiert mit `Out [n]:`: der Text eines Streams (`stderr:`, wenn es die Fehler sind), der Text eines Ergebnisses (oder sein Markdown, gerendert) und ein Fehler mit seinem Traceback, ohne dessen ANSI-Codes.
- Eine Ausgabe als Bild, HTML, JavaScript, SVG oder LaTeX ist eine Zeile, die sagt, was sie ist und wie groß, etwa `[image/png, 34 KiB: not shown]`. Nichts in einem Notebook wird ausgeführt: weder sein Code noch die Skripte seiner Ausgaben noch seine Widgets.
- Eine Raw-Zelle wird so gezeigt, wie sie ist.

`T` zeigt das JSON des Notebooks und `H` seine Bytes. Eine Datei, die kein Notebook ist, das der Viewer lesen kann (kaputtes JSON, unbekanntes Format), öffnet sich als Quelltext, mit einem Hinweis. Das Format 3 (mit Worksheets) wird ebenfalls gelesen.

## Office-Dokumente

Word- (`.docx`, `.docm`, `.dotx`), OpenDocument-Text- (`.odt`), PowerPoint- (`.pptx`, `.pptm`) und Impress-Dateien (`.odp`) öffnen sich als Dokument: Überschriften, Absätze, Listen, Tabellen, fett, kursiv und Links, mit der echten Adresse jedes Links. Eine Präsentation zeigt jede Folie unter `Slide n`, gefolgt von den Notizen des Vortragenden. Fußnoten und Kommentare stehen mit ihren Autoren am Ende. Ein Bild ist eine Zeile, etwa `[image: media/image1.png]`. Der Text eines Dokuments wird nie als Markdown gelesen.

Vor dem Text sagt ein Block, was das Dokument außer ihm enthält, denn vieles davon zeigt das Programm, das es erstellt hat, nicht an:

- wer es geschrieben hat, wer es zuletzt gespeichert hat, wann und womit;
- `Contains macros`, wenn es welche hat. Sie werden nie gelesen oder ausgeführt;
- `Fetches from outside when it opens`: eine entfernte Vorlage, ein OLE-Objekt oder ein Frame, die aus der Datei hinaus zeigen, mit ihren Adressen. Über eine entfernte Vorlage kamen Angriffe wie Follina (CVE-2022-30190). Hier wird nie etwas geladen;
- die eingebetteten Objekte mit Name und Größe, und die Anzahl weiterer externer Links;
- Änderungsverfolgung: eingefügter Text erscheint als `[inserted: ...]` und gelöschter als `[deleted: ...]`, denn das Gelöschte ist oft das, worauf es ankommt;
- versteckter Text (`[hidden: ...]`), weißer Text und Text in einem Punkt Größe (`[invisible: ...]`), und ausgeblendete Folien.

`H` zeigt die Bytes und kehrt zurück; `T` sagt, dass das Dokument keinen eigenen Text hat. Jeder Teil der Datei wird nur bis zu einem Limit entpackt (64 MiB je Teil, 256 MiB insgesamt; 4 MiB im Autoview), und die Kopfzeile sagt, wenn das Dokument abgeschnitten wurde. Ein XML-Teil mit einer DTD oder einer Verschachtelung tiefer als 256 Ebenen wird abgelehnt. Die alten `.doc` und `.ppt` öffnen sich wie bisher in Hex.

## Ausführbare Dateien

Eine ausführbare Datei, eine Bibliothek oder eine Objektdatei öffnet sich als Bericht, egal wie sie heißt: ELF (Linux, BSD), PE (Windows-`.exe`, `.dll`, `.sys`, auch .NET) und Mach-O (macOS, auch Universal-Binaries mit mehreren Architekturen). Sie wird an ihren ersten Bytes erkannt, nie am Namen. Der Bericht sagt:

- was sie ist (ausführbare Datei, Bibliothek, Objekt, Core-Dump), für welche Maschine, 32 oder 64 Bit, ihren Einsprungpunkt und ihren Interpreter, ihr Subsystem oder ihr Mindestsystem;
- **Schutzmechanismen**, jeweils `yes` oder `no`, mit `⚠`, wo er fehlt: bei ELF PIE, ein Stack, der keinen Code ausführt (NX), RELRO (vollständig oder teilweise), der Stack-Canary, FORTIFY und die Bibliothekspfade (RPATH, RUNPATH: ein relativer oder einer in `/tmp` wird markiert); bei PE ASLR, ASLR mit hoher Entropie, DEP, Control Flow Guard; bei Mach-O PIE und ein ausführbarer Stack;
- **Signatur**: ob es eine gibt und wie groß. Sie wird **nicht geprüft**, und der Bericht sagt das;
- **Anzeichen** für Verstecktes: ein Abschnitt mit hoher Entropie (gepackt oder verschlüsselt), der Name eines Packers (UPX...), Daten nach dem letzten Abschnitt (ein Overlay, in dem Installer und manche Schadsoftware Daten ablegen), ein Abschnitt, der zugleich schreibbar und ausführbar ist, ein Einsprungpunkt außerhalb jedes ausführbaren Abschnitts;
- **Identität**: Build-ID, Debug-Link, UUID, PDB-Pfad, der Compiler, die Go-Version; das Build-Datum einer PE, das der Linker schreibt und das beliebig sein kann;
- die Bibliotheken, die sie braucht, und die Listen der Abschnitte, Segmente, Importe, Exporte und Symbole, mit je bis zu 2.000 Zeilen.

Für den Bericht wird nichts ausgeführt oder geladen: nicht einmal `ldd`, das das Programm in manchen Fällen ausführt. `H` zeigt die Bytes und kehrt zurück. Eine Datei, die größer als das Leselimit ist, wird nur dort gelesen, wo der Bericht es braucht, höchstens 256 MiB insgesamt; der Autoview berichtet über den gelesenen Anfang und sagt, dass ihm etwas entgehen kann.

## Zertifikate und Schlüssel

Ein Zertifikat, eine Kette oder ein Bündel davon, ein Zertifikatsantrag (CSR), eine Sperrliste (CRL), ein privater oder öffentlicher Schlüssel, ein OpenSSH-Schlüssel, `authorized_keys` und `known_hosts` öffnen sich als Bericht. PEM wird in einer dafür benannten Datei gelesen (`.pem`, `.crt`, `.cer`, `.key`, `.csr`, `.crl`, `.pub`...) oder in einer ohne Endung (`id_ed25519`); DER in einer `.der`, `.cer`, `.crt`, `.crl` oder `.csr`, die ein ganzer DER-Wert ist. Ein PEM-Block in Quellcode oder in einem Dokument lässt diese Datei, wie sie ist.

- **Ein Zertifikat:** Subject und Issuer (und ob es selbstsigniert ist), Seriennummer, Gültigkeit mit den verbleibenden Tagen (abgelaufen, noch nicht gültig oder innerhalb von 30 Tagen ablaufend wird mit `⚠` markiert), öffentlicher Schlüssel (RSA unter 2048 Bit und DSA werden markiert), Signatur (MD5 und SHA-1 werden markiert), Namen (ein Punycode-Name `xn--` wird hervorgehoben), Schlüsselverwendung, erweiterte Schlüsselverwendung, ob es eine CA ist, die Adressen seiner CRL und der Zertifizierungsstellen-Informationen, eine unbekannte kritische Erweiterung (markiert) und sein SHA-256-Fingerabdruck.
- **Mehrere Zertifikate:** Der Bericht sagt, ob sie in der richtigen Reihenfolge sind, jedes vom nächsten ausgestellt, geprüft nur anhand ihrer Namen und Schlüsselkennungen. **Hier wird keine Signatur geprüft**, und der Bericht sagt das.
- **Ein privater Schlüssel:** nur seine Art und Größe (RSA 2048 Bit, EC P-256, PKCS#8 verschlüsselt mit PBES2...) und bei OpenSSH, ob er verschlüsselt ist, und sein Fingerabdruck. Seine Zahlen erscheinen nie im Bericht. Im Autoview zeigt ein privater Schlüssel diesen Bericht, nie seinen Text, damit er nicht auf dem Bildschirm erscheint, nur weil man darüber geht; `T` im Viewer zeigt den Text.
- **Öffentliche Schlüssel, `authorized_keys` und `known_hosts`:** eine Zeile pro Schlüssel, mit Art und Größe, dem SHA-256-Fingerabdruck, wie ihn `ssh-keygen -l` schreibt, und dem Kommentar. Eine Option `command=` wird markiert, da sie ändert, was der Schlüssel erlaubt; ein gehashter Host erscheint als solcher.

Nichts wird geladen: Die Adressen einer CRL oder eines OCSP werden nur gezeigt. Bevor ein Parser einen Block sieht, werden seine Verschachtelung (höchstens 64 Ebenen) und seine Längen geprüft; eine Datei hat höchstens 1.000 Blöcke zu je 64 KiB. PKCS#12 (`.p12`, `.pfx`) und PGP-Schlüssel werden nicht gelesen.

## Logs

Ein Log öffnet sich als Text, mit den Farben seiner Stufen, und die Fußzeile zeigt `Log`. Eine Datei ist ein Log nach ihrem Namen (`.log`, `.out`, `.err`, eine rotierte wie `app.log.1` oder `app.log.2.gz`, `syslog`, `messages`, `kern.log`) oder nach ihrem Inhalt: Die meisten ihrer ersten Zeilen beginnen mit Datum und Uhrzeit (`2026-11-16 10:00`, `[2026-11-16 ...` oder syslogs `Nov 16 10:00:00`).

- **Stufen:** Das erste Stufenwort in den ersten 120 Spalten einer Zeile, egal in welcher Schreibung, gibt ihr die Farbe: Fehler (`ERROR`, `ERR`, `FATAL`, `CRIT`, `PANIC`...) in Rot, Warnungen (`WARN`, `WARNING`) in Gelb, `DEBUG` und `TRACE` abgeschwächt. Syslogs `<n>` am Zeilenanfang zählt auch, ebenso die Stufe eines Logs in JSON (`"level":"error"`). Eine Zeile ohne Stufe, etwa ein Stacktrace, übernimmt die der vorigen. `C` schaltet die Farben aus.
- **ANSI-Farben:** Die Farben, die ein Programm in sein Log geschrieben hat (`ESC [31m`), werden als Farben gezeigt, und ihre Sequenzen werden aus dem Text entfernt. Jede andere Sequenz wird verworfen, da sie den Cursor bewegen oder in die Zwischenablage schreiben könnte (OSC 52), und die Kopfzeile zählt sie, etwa `[ANSI: 3 removed]`. `H` zeigt die Bytes weiterhin, wie sie sind, und eine Suche durchsucht den Text ohne die Sequenzen. In einer Datei über 16 MiB, die durch ein Fenster gelesen wird, erscheinen die Sequenzen wie in jedem Text als `·`, und die Farben sind die der Stufen.
- **Verfolgen:** Steht die letzte Zeile auf dem Bildschirm (Ende), kommen neue Zeilen hinzu, sobald das Programm sie schreibt, und der Bildschirm bleibt am Ende; die Kopfzeile zeigt `[FOLLOW]`. Die Datei wird jede halbe Sekunde geprüft, auch auf einer Netzwerkfreigabe. Nach oben gehen oder suchen beendet das Verfolgen; Ende nimmt es wieder auf. Mit einem Filter (`&`) laufen die neuen Zeilen durch ihn. Ist die Datei geschrumpft oder hat sie sich geändert, statt zu wachsen (rotiert oder neu geschrieben), zeigt die Kopfzeile `(the file changed: R reloads)`, und bis R wird nichts gelesen. Ein komprimiertes Log wird nicht verfolgt.

## Patches

Eine `.patch` oder `.diff`, oder jeder Text, der wie eine beginnt (`diff --git`, oder `---`, `+++` und `@@`), öffnet sich als Text mit den Farben eines Patches. Der Git-Diff (`D`) wird genauso gezeigt. Die Kopfzeile fasst ihn zusammen, etwa `[12 files, +340 −98]`, und die Fußzeile sagt, wo der obere Bildschirmrand steht: `3/12 src/viewer.rs @@ 2/5`.

| Taste | Aktion |
|---|---|
| `]` `[` | nächste oder vorige Datei |
| `}` `{` | nächster oder voriger Hunk |
| `Tab` | die Liste der Dateien, mit `+` und `−` für jede; Buchstaben grenzen sie ein, `Enter` springt zu einer |

Manche Änderungen tun mehr, als Zeilen zu ändern, und gehen beim schnellen Lesen leicht unter. Sie werden rot gezeigt, und die Kopfzeile zählt sie als `to check`:

- eine Datei, die ausführbar wird (`100755`), ein symbolischer Link (`120000`) oder ein Submodul (`160000`);
- ein Pfad, der absolut ist, mit `..` nach oben steigt, in `.git` geht (ein Hook ist Code, den Git ausführt) oder Steuerzeichen, Zeichen der Textrichtung oder unsichtbare Zeichen enthält (ein Name wie `invoice‮fdp.exe` versteckt seine wahre Endung).

Die Kopfzeile zählt außerdem:

- `look unchanged`: eine entfernte und eine hinzugefügte Zeile, die sich nur in Unsichtbarem unterscheiden, etwa Leerzeichen, `\r`, unsichtbaren Zeichen oder ähnlich aussehenden Buchstaben (ein kyrillisches `а` statt eines lateinischen `a`). Das Paar wird gelb gezeigt;
- `hunks with wrong counts`: ein Hunk, dessen `@@`-Zeile nicht zu seinen Zeilen passt. Der Index folgt den Zeilen, die da sind, nie den Zahlen;
- `whitespace`: hinzugefügte Zeilen mit einem Leerzeichen am Ende oder einem Leerzeichen vor einem Tab in der Einrückung, wie `git diff --check` sie meldet.

Eine Serie von `git format-patch` zeigt jede Nachricht mit ihrem Betreff in der Liste. Nichts wird je angewendet. In einer Datei über 16 MiB, die durch ein Fenster gelesen wird, bleiben die Farben, aber es gibt keinen Index: `]`, `}` und Tab sagen das.

## Steuerzeichen für die Schreibrichtung

Manche unsichtbaren Zeichen ändern die Schreibrichtung des Textes und können Code anders aussehen lassen, als er wirklich lautet (Trojan Source, CVE-2021-42574). Der Betrachter versteckt sie nie: Jedes erscheint als rotes `·`, und die Kopfzeile nennt ihre Anzahl.

## Textkodierung

Xylem erkennt die Kodierung selbst, in dieser Reihenfolge: zuerst wird die Datei entpackt, dann wird nach einer BOM gesucht (UTF-8, UTF-16 LE oder BE), dann nach UTF-16 ohne BOM, dann nach UTF-8. Passt nichts davon, ist es alter Text: Xylem rät CP850 (DOS, Westeuropa) oder Windows-1252, indem es die Buchstaben mit Akzent zählt. `.reg`-Dateien von Windows sind UTF-16 und öffnen sich als normaler Text.

**O** zeigt die Datei in der nächsten Kodierung: UTF-8, CP850, Windows-1252, CP437, UTF-16 LE, UTF-16 BE und wieder UTF-8. Die Fußzeile sagt, welche, und die Kopfzeile zeigt den Namen, wenn es nicht UTF-8 ist. Wählen Sie UTF-16 bei einer Datei, die in Hex geöffnet wurde, wechselt sie zu Text. Ihre Wahl bleibt, wenn Sie die Datei bearbeiten und zurückkehren.

Um für alte Textdateien immer eine feste Kodierung statt der Vermutung zu nutzen, setzen Sie `legacy_charset` auf `cp850`, `cp1252` oder `cp437` (**K** auf Seite 2 der [Konfiguration](configuration.md)). Standard ist `auto`.

## Große Dateien

Mit **V** wird eine Datei auf der Festplatte, die größer als 16 MiB ist, nicht geladen: Der Betrachter behält die Zeilen in der Nähe des Bildschirms, etwa 2.000, und liest die übrigen aus der Datei, während er weiterläuft, ohne den Bildschirm aufzuhalten. Eine noch nicht gelesene Zeile erscheint kurz als `…`. So hat die Datei keine Größengrenze, und der belegte Speicher wächst nicht mit ihr.

- **Die Zeilen werden im Hintergrund gezählt.** Die Fußzeile zeigt `Line 1 of 1,234,567+ (counting…)` und dann die Gesamtzahl. **End** und **g** gehen zu jeder schon gezählten Zeile. Esc hält die Zählung an, und das Gezählte bleibt.
- **Die Suche** (`F`, `B`, `/`, `\`, Space) läuft im Hintergrund durch die Datei: Die Fußzeile zeigt `Searching... line 1,200,000 (Esc stops)`, und der Bildschirm springt zum Treffer, sobald er gefunden ist.
- **Der Zeilenfilter** (`&`) läuft im Hintergrund durch die ganze Datei, und die Zeilen erscheinen, sobald sie gefunden werden, bis zu 1.000.000. Esc hält ihn an und behält das Gefundene.
- **Hex** (`H`) liest die Bytes dort, wo der Bildschirm steht, und auch seine Suche läuft durch die Datei.
- **Eine andere Kodierung** (`O`) liest die Zeilen neu; bei UTF-16 werden sie neu gezählt, weil eine Zeile an anderer Stelle endet.
- **Keine Syntaxfarben und kein gerendertes Markdown**: Beide lesen einen Text von Anfang an.
- **Eine Zeile über 1 MiB** wird auf dem Bildschirm gekürzt und endet mit `…`; Hex zeigt sie ganz.
- **Ändert sich die Datei**, während sie offen ist, zeigt die Kopfzeile `(the file changed: R reloads)`, und **R** liest sie neu und kehrt zur vorherigen Zeile zurück.
- **Ein Batch** (`Ctrl+B`) über den Text liest die Datei weiterhin in den Speicher, bis zum Größenlimit unten, und sagt es, wenn das nicht die ganze Datei ist.

Um eine Zeile zu erreichen, merkt sich der Betrachter, wo jede 256. Zeile beginnt, etwa 8 Bytes je 256 Zeilen, nur im Speicher. Die Textrichtungszeichen werden zusammen mit den Zeilen über die ganze Datei gezählt (in UTF-8), und die Kopfzeile zeigt, wie viele bisher gefunden wurden.

Eine komprimierte Datei, ein Eintrag eines Archivs, eine Datei auf einem entfernten Rechner und Autoview werden nicht so gelesen: Darin lässt sich nicht springen, daher gilt für sie das Größenlimit unten.

## Größenlimit

In diesen Fällen und bei Dateien bis 16 MiB liest der Betrachter von einer zu großen Datei nur den Anfang, und die Kopfzeile zeigt `(showing first N MiB)`. Wie viel er liest, bestimmt die Option `viewer_max_size` (**L** auf Seite 2 der [Konfiguration](configuration.md)): `auto`, `16`, `64`, `256` oder `1024` (MiB). Bei `auto`, dem Standard, beträgt das Limit 1/32 des Arbeitsspeichers des Rechners, nie weniger als 16 MiB und nie mehr als 256 MiB (16 MiB, wenn sich der Arbeitsspeicher nicht auslesen lässt). Die Konfigurationsseite zeigt den verwendeten Wert, zum Beispiel `auto (256 MiB)`.

Zwei Fälle folgen dieser Regel nicht. Autoview liest immer 1 MiB. Eine Datei auf einem entfernten Rechner (SFTP) wird höchstens bis 16 MiB gelesen, weil Esc dieses Lesen nicht abbrechen kann.

Ein größeres Limit kostet Speicher und Zeit: Mit `1024` belegt ein komprimierter Text, der sich auf diese Größe entpackt, etwa 2 GiB RAM. Sehr kurze Zeilen kosten mehr als ihre Bytes, weil jede Zeile eigene Verwaltungsdaten hat, und dasselbe gilt für Tabulatoren und versteckte Steuerzeichen für die Schreibrichtung: Der Betrachter hält Zeilen bis etwa zum Doppelten des Größenlimits im Speicher, danach steht in der Kopfzeile `(showing first N lines)`. Ein Log mit gewöhnlichen Zeilen (etwa 100 Bytes) wird dadurch nie abgeschnitten.

## Komprimierte Dateien

Dateien mit der Endung `.gz`, `.xz`, `.zst` oder `.bz2` werden entpackt als Text geöffnet. Xylem prüft die ersten Bytes der Datei, nicht den Namen, und öffnet nur eine Ebene: Eine `.gz`-Datei in einer `.gz`-Datei bleibt komprimiert. Die Kopfzeile zeigt `gzip → N bytes` (das Format und die Größe nach dem Entpacken).

Das Entpacken hat Grenzen. Der Betrachter liest bis zum Größenlimit, braucht höchstens 3 s und nutzt höchstens 64 MiB Speicher für das xz-Wörterbuch oder das zstd-Fenster. In Autoview gelten 1 MiB, 100 ms und 16 MiB. Wird die Ausgabe- oder die Zeitgrenze erreicht, steht in der Kopfzeile `(showing first N bytes decompressed)`. Ein Hinweis sagt, was passiert ist:

- `decompression stopped after 3 s` (100 ms in Autoview)
- `compressed data truncated or corrupt` (komprimierte Daten abgeschnitten oder beschädigt)
- `needs more than 64 MiB of memory to decompress` (16 MiB in Autoview, mit ` (V to view)`, wenn die 64 MiB von V reichen können)

Der Hinweis verschwindet beim ersten Tastendruck, die Kopfzeile behält das Problem aber: `(truncated or corrupt)`, `(incomplete: …)`, wenn die Speichergrenze das Entpacken mittendrin gestoppt hat, oder `(not decompressed: …)`, wenn nichts entpackt werden konnte (dann erscheinen die Rohbytes in Hex).

Autoview zeigt den Hinweis in seinem Titel. Im Hex-Modus (`H`) sehen Sie die entpackten Bytes. Die Farben richten sich nach dem Namen ohne die Kompressionsendung: `app.rs.gz` wird wie Rust gefärbt.

Bei einer `.tar.gz` (oder `.tar.xz` ...) lautet der Hinweis `TAR archive: Alt+F5 opens it as a folder` (Alt+F5 öffnet sie als Ordner).

Um die rohen komprimierten Bytes zu sehen, schalten Sie `viewer_decompress` aus (**J** auf Seite 2 der [Konfiguration](configuration.md)).

## Tabellenmodus

Tabellenkalkulationen, CSV, DBF, SQLite und Parquet öffnen sich als Raster, mit einem SQL-Filter: siehe [Tabellen](tables.md).

Siehe auch: [markierte Dateien durchsuchen](search.md).
