# Versionshinweise

Die neueste Version steht oben. Jeder Abschnitt listet auf, was neu ist, was
sich für bisherige Nutzer geändert hat und welche Tasten umbelegt wurden. Die
vollständigen Versionshinweise finden sich unter
`https://github.com/andrelnfernandes/xylem/releases`.

## 0.9.1

**Behoben.**

- Showall (und Branch) eines Baums mit Millionen von Dateien hielt den
  Bildschirm sekundenlang fest, während die Liste sortiert wurde: etwa 5 s bei
  2.000.000 Dateien. Jetzt dauert es deutlich unter einer Sekunde (0,66 s auf
  derselben Maschine), in jeder Sortierung, und die Reihenfolge der Dateien ist
  dieselbe wie vorher.

## 0.9.0

**Neu.**

- Norton Guides (`.ng`), die Hilfedateien von DOS und Clipper, öffnen sich
  im Betrachter als Dokument: die Menüs, dann jeder Eintrag nummeriert
  (`§12`), mit seinen Links und seinem Siehe-auch, die auf diese Nummern
  zeigen. Tab wählt einen Link, Enter oder ein Klick folgt ihm, Backspace
  geht zurück. [Der Betrachter](viewer.md)
- Unscharfe Suche (`|`): Teile eines Namens tippen (`cfgxy` findet
  `config/xylem`), und der Cursor geht zum besten Ordner, im Baum, oder zur
  besten Datei der Liste, im Dateifenster. Die Ergebnisse erscheinen beim
  Tippen, die besten zuerst, auch in einem Baum mit Millionen Ordnern.
  Ctrl+T markiert das gewählte Ergebnis und Alt+T die Liste. [Unscharfe
  Suche](fuzzy.md)
- [Dateien und Ordner finden](finding.md): eine Seite, die sagt, welche
  Suche welche Frage beantwortet.
- E-Books: EPUB und FictionBook (`.fb2`, `.fb2.zip`) öffnen sich als Buch
  zum Lesen, in einer Spalte mit den Absätzen eines gedruckten Buchs: eine
  Titelseite, Space für die nächste Seite, `]` `[` für Kapitel, Tab für das
  Inhaltsverzeichnis, Enter für die auf der Seite aufgerufenen Anmerkungen,
  `g 40%` für eine Stelle im Buch, und wie viel vom Kapitel und vom Buch
  bleibt. `I` zeigt den Bericht über die Datei: was das Buch über sich sagt
  und ob es Skripte, von außen Geladenes oder DRM enthält. [Der
  Betrachter](viewer.md#ebooks)
- `R` in einem Buch merkt sich, wo Sie aufgehört haben, und das Buch öffnet
  sich beim nächsten Mal dort. Wenn Sie es einschalten (Seite 5 von F12),
  speichert Xylem auch die Zeit, die Sie mit jedem gemerkten Buch verbracht
  haben, Ihre Sitzungen und Ihre Lesegeschwindigkeit, nur auf Ihrem Rechner.
  [Lesezeit](configuration.md#reading-time)
- F1 öffnet die Hilfe beim Abschnitt dessen, was Sie gerade tun: im Batch
  des Betrachters (Ctrl+B) bei „Batch im Betrachter“, nicht am Anfang der
  Seite. Links auf einen Abschnitt einer Hilfeseite führen jetzt dorthin.
- Xylem kann erkennen, dass eine neue Version erschienen ist. Beim ersten
  Start fragt es, ob es einmal am Tag nachsehen darf; bei Ja lädt es von der
  Website nur die Nummer der neuesten Version, und die oberste Zeile sagt,
  wenn eine neuere erschienen ist. Nichts wird heruntergeladen oder
  installiert. [Neue Versionen](configuration.md#new-versions)
- Ansichten (Alt+V): andere Arten, die Liste anzusehen. Der Scan (`S`) zeigt,
  was in einem Ordner einen Blick verdient, bevor Sie ihn öffnen: Namen mit
  versteckten Zeichen oder doppelter Endung (`fatura.pdf.exe`),
  Setuid-Dateien und was jeder schreiben kann, Links, die den Ordner
  verlassen oder ins Leere zeigen, und, gelesen mit den eigenen Lesern und
  Limits des Betrachters, ausführbare Dateien ohne PIE oder NX oder gepackt,
  Office-Dokumente mit Makros oder einer entfernten Vorlage, Patches, die mit
  `..` hinausklettern. Enter auf einer Kategorie bringt ihre Dateien zurück
  in die Liste, für Ctrl+T und den Rest. Die Zeitleiste (`T`) zeigt die
  Dateien der Liste in einem Histogramm nach Datum, nach Stunden, Tagen,
  Wochen, Monaten oder Jahren; Enter auf einem Balken oder einem Bereich
  behält nur dessen Dateien, als `W`-Bedingung. Die Tabelle (`L`), mit `s` nach jeder Spalte sortierbar, öffnet die
  Liste im Tabellenbetrachter, eine Datei pro Zeile mit den Spalten von `W`,
  auch eine Liste mit Millionen Dateien; `W` dort bringt ihren SQL-Filter
  zurück in die Liste. [Ansichten](views.md)

**Geändert.**

- `O` im Betrachter geht vor UTF-16 durch zwei weitere Kodierungen,
  Windows-1251 und KOI8-R (Kyrillisch).
- Die Typanzeige (Alt+F) sagt `EPUB ebook (ZIP)`, `FB2 ebook` und
  `FB2 ebook (ZIP)`.

**Behoben.**

- Mit eingeschaltetem Git fror ein großes Repository auf einem langsamen
  Dateisystem (`/mnt/c` aus WSL, eine Netzwerkfreigabe) den Bildschirm ein,
  solange `git status` lief, minutenlang. Der Status wird jetzt im
  Hintergrund gelesen, und ein langsames Repository wird seltener neu
  gelesen. [Git](git.md)
- Die Bibliothek, die Xylem für Systemaufrufe verwendet (rustix 1.1.6), ist
  wegen eines Sicherheitshinweises zu erweiterten Attributen aktualisiert
  ([RUSTSEC-2026-0486](https://rustsec.org/advisories/RUSTSEC-2026-0486)).

## 0.8.0

**Geändert: einige Alt-Tasten.** Alt wiederholt Ctrl nicht mehr. Es ist die
andere Fassung eines Befehls.

| Taste | Bis 0.7 | Ab 0.8 |
|---|---|---|
| Alt+A | Attributes der markierten Dateien | nichts (Ctrl+A) |
| Alt+D | löscht die markierten Dateien | Duplicates (Ctrl+D löscht sie) |
| Alt+R | benennt die markierten Dateien um | Release des Zweigs, wie `-` (Ctrl+R benennt sie um) |
| Alt+T | alles markieren | nichts (Ctrl+T) |
| Alt+U | alle Markierungen aufheben | Usage (Ctrl+U hebt Markierungen auf) |
| Alt+W | Usage | nichts (Alt+U) |
| Alt+Y | Duplicates | nichts (Alt+D) |

Um die alten Tasten beizubehalten, tragen Sie sie in `config.toml` ein, einzeln
oder alle:

```toml
[keys.any]
"alt+a" = "attributes_tagged"
"alt+d" = "delete_tagged"
"alt+r" = "rename_tagged"
"alt+t" = "tag_all"
"alt+u" = "untag_all"
"alt+w" = "usage"
"alt+y" = "duplicates"
```

**Geändert: die Fußzeile.** Jeder Befehl gehört nun zu einem Menü und die
Fußzeile hat drei Zeilen bzw. zwei auf Bildschirmen, die breit genug sind, um
jedes Menü auf einer Zeile darzustellen. Auf schmalen Bildschirmen werden Wörter
gekürzt, bevor Befehle entfallen. In Archiven und auf entfernten Rechnern führen
die Menüs nur auf, was dort ausgeführt werden kann. [Die Menüs](menus.md)

**Geändert: Eingaben.** ↑ öffnet den Verlauf einer Eingabezeile als Liste; ↓
blättert Einträge nicht mehr einzeln durch. `B` in einem Branch oder Showall
kehrt zur Liste des Verzeichnisses selbst zurück. Das verkleinerte Dateifenster
belegt ein Viertel der Bildschirmhöhe.

**Neu.**

- `W` behält die Dateien in der Liste, die einer SQL-Bedingung entsprechen;
  Ctrl+W filtert unter den markierten Dateien. `contains('text')` liest die
  Dateiinhalte, und `words('text')` nutzt den mit Alt+N erstellten Index.
  [Eine SQL-Bedingung](sql.md#eine-sql-bedingung-w)
- `P` öffnet ein Suchfeld, das Dateien nach den eingetippten Wörtern auflistet.
  [Das Suchfeld](sql.md#das-suchfeld-p)
- CSV-, TSV-, JSON-Lines-, DBF- und SQLite-Tabellen haben im Betrachter kein
  Zeilenlimit mehr. Tabellenkalkulationen behalten das Limit von 50.000 Zeilen.
  [Tabellen](tables.md)
- In einer Tabelle zeigt Enter den Datensatz als Formular, und `c` kopiert ihn;
  `c` kopiert auch die mit `i` gezeigte Struktur. [Tabellen](tables.md)
- Ein Text über 16 MiB wird durch ein Fenster gelesen, ohne Größenlimit.
  [Große Dateien](viewer.md#große-dateien)
- Neue Betrachter: JSON, YAML und TOML als faltbarer Baum; Logs in den Farben
  ihrer Stufen, und Mitverfolgen; Patches mit einem Index der Dateien und Hunks
  und dem, was zu prüfen ist; Jupyter-Notebooks; Office-Dokumente, mit ihren
  Makros, entfernten Vorlagen und verborgenem Text; ausführbare Dateien (ELF,
  PE, Mach-O) sowie Zertifikate und Schlüssel als Berichte. [Der Betrachter](viewer.md)
- Es lässt sich auf macOS bauen (noch nicht auf einem Mac getestet).
- Alt+L macht das Verzeichnis unter dem Cursor zur Wurzel des Baums.
- In Treespec entfernt Ctrl+← den letzten Ordner aus dem Pfad und Ctrl+→ stellt
  ihn wieder her.
- `clock_seconds` blendet Sekunden in der Uhr der Kopfzeile ein.
- Die Maus, standardmäßig aus (F12, Seite 2, R): das Mausrad, Klicks und
  Doppelklicks auf den Baum, die Listen und die Tasten der Fußzeile, Markieren
  mit der mittleren Taste, ein Menü auf der rechten Taste (auch die Menü-Taste
  und Shift+F10), und die Maus an die Programme des integrierten Terminals und
  des Editors im Rahmen weitergereicht.
  [Die Maus](mouse.md)
- Dieses Handbuch: Seiten zu Bildschirm, Menüs, Befehlen jedes Menüs,
  Vokabular und Schritt-für-Schritt-Anleitungen.

## 0.7.0

**Geändert: Einlesen (Logging).** Ein Verzeichnis wird nur noch auf ausdrückliche
Anforderung eingelesen. Wird der Cursor auf ein nicht eingelesenes Verzeichnis
bewegt, erscheint `Not logged`; `+` oder → liest es ein. Zuvor las das
Überfahren ein Verzeichnis automatisch ein. [Das Baumfenster](tree.md)

**Geändert: Eingaben.** Log sowie Copy und Move ohne geteilten Bildschirm öffnen
sich leer. ↑ listet die zuvor genutzten Orte auf.

**Neu.**

- Entfernte Rechner sind nicht mehr schreibgeschützt: Copy dorthin, Move, Rename,
  Makedir, Delete, Prune, Attributes, New date und Duplicates funktionieren dort.
  [Entfernte Rechner (SSH)](remote.md)
- `docker://container/pfad` öffnet einen Container über `docker exec`.
- Lesezeichen können Namen tragen und ein Lesezeichen auf einem entfernten
  Rechner speichert die Verbindungsadresse. F11 in den Log-, Copy- und
  Move-Eingaben fügt ein Lesezeichen in die Eingabezeile ein.
  [Lesezeichen](bookmarks.md)
- Von rclone eingehängte Ordner und Netzwerkfreigaben werden im Hintergrund
  gelesen; der Baum erscheint sofort.
  [Cloud-Speicher (rclone)](cloud.md)
- Die Belegungsansicht erlaubt das Markieren, Löschen, Beschneiden (Prune) und
  Anzeigen direkt aus ihrer Ansicht heraus.
- Startbildschirm, Name und Version in der Kopfzeile, nach Typ gefärbte
  Dateinamen und abgerundete Rahmen. [Konfiguration](configuration.md)

## 0.6.1

Aktualisiert den Tabellenkalkulationsleser (calamine 0.38) wegen der
Sicherheitsmeldung RUSTSEC-2026-0436.

## 0.6.0

**Neu.**

- Der Betrachter stellt Tabellenkalkulationen, CSV, JSON Lines, DBF, SQLite und
  Parquet als Tabellen mit SQL-Filter dar. Eine SQLite-Datenbank akzeptiert
  zudem eine vollständige Leseabfrage. [Der Betrachter](viewer.md)
- Ctrl+B im Betrachter schreibt eine Datei aus Tabellendatensätzen oder
  Textzeilen: JSON, CSV, XML, HTML, Vorlage oder SQLite-Tabelle.
  [Batch](execute.md)
- Duplicates findet gleiche Dateien und Ordner auf lokalen Festplatten und über
  von rclone gemeldete Hashes. Die Taste war Alt+Y; ab 0.8 ist es Alt+D.
  [Duplikate](duplicates.md)
- Markdown-Dateien öffnen sich formatiert im Betrachter. `T` zeigt den
  Quelltext.
- Der Betrachter filtert Zeilen mit regulären Ausdrücken (`&`).
  [Zeilen filtern](filter.md)
- `*` auf einem von rclone eingehängten Ordner fragt den gesamten Zweig in einer
  einzigen Anfrage ab.
- Das Betrachten eines Git-Repositories führt keine repository-eigenen Befehle
  aus, es sei denn, sein Ordner steht in `git_trusted`. [Git](git.md)

## 0.5.0

**Geändert.** `-` im Baum gibt den Zweig aus dem Speicher frei; ← klappt ihn
weiterhin nur zu. `E` öffnet den Editor im Rahmen von Xylem.

**Neu.**

- Sicherheitsmaßnahmen für potenziell feindliche Dateien: Limits für
  Dekomprimierung, Pfadprüfungen beim Entpacken, Abfangen von Steuerzeichen vor
  dem Terminal. [Sicherheit](security.md)
- Der Betrachter bricht Zeilen um (`W`), nummeriert sie (`#`), springt zu
  Zeilen (`g`), zeigt `.gz`-, `.xz`-, `.zst`- und `.bz2`-Dateien dekomprimiert an
  und erkennt CP437, CP850, Windows-1252 und UTF-16.
- Hilfe in sechs Sprachen; `L` in der Hilfe wechselt die Sprache.
- Ctrl+S im Baum listet nur die markierten Dateien von allem Eingelesenen auf.
- Option Graph im Git-Menü.
- Nerd-Fonts-Symbole, standardmäßig deaktiviert.
- `.deb`- und `.rpm`-Pakete.

## 0.4.0

**Neu.**

- Belegungsansicht: Ordner und Dateien eines Zweigs sortiert nach belegtem
  Platz. Die Taste war Alt+W; ab 0.8 ist es Alt+U.
  [Speicherplatzbelegung](usage.md)
- Alt+K Percent zeigt den prozentualen Anteil jedes Verzeichnisses am
  übergeordneten an.
- Entfernte Rechner über SSH, in dieser Version schreibgeschützt.
  [Entfernte Rechner (SSH)](remote.md)
- Google Drive, S3 und weitere Speicher über rclone.
  [Cloud-Speicher (rclone)](cloud.md)
- Themen slate, graphite, daylight und vivid.
- `[editors]` in `config.toml` wählt den Editor nach Dateiname und `E` weist
  Binärdateien ohne konfigurierten Editor ab.
- `xy --man` zeigt die Handbuchseite an.

## 0.3.0 und früher

- 0.3.0: Hilfe über F1 und Manpage; 7z-, tar.xz-, tar.bz2- und tar.zst-Archive;
  Anzeigemodi für Besitzer und Typ (Alt+F); Gegenüberstellung beider Dateien bei
  Kopierkonflikten.
- 0.2.0: Umbenennung des Programms von xt in Xylem und des Befehls in `xy`.
  Löschen in den Papierkorb und Oops, Stapelumbenennung mit Maske und
  Zwischenablage.
- 0.1.0: Erste Version unter dem Namen xt.
