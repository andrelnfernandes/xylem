# Filespec: der Dateifilter

`F` legt fest, welche Dateien im Dateifenster angezeigt werden (jedes Fenster
hat seinen eigenen Filter). Der aktuelle Filespec erscheint im Fenstertitel.
Angaben werden durch Leerzeichen, `;` oder `,` getrennt und gehören wie in
ZTreeWin zu einer von drei Arten:

- **Name:** ein Glob-Muster (`*.rs`, `a?c*`, `[abc]*`), ohne Beachtung der
  Groß-/Kleinschreibung; in Anführungszeichen darf ein Name Leerzeichen
  enthalten (`"meine datei*"`);
- **Datum:** `<`, `=`, `>`, `<=`, `>=` oder `<>` und ein Datum in der
  Reihenfolge des konfigurierten Formats, mit `/`, `.` oder `-` und einer 2-
  oder 4-stelligen Jahreszahl (`>01/05/2026`; `2026-05-01` funktioniert immer),
  oder `TODAY` und `TODAY-n` (`>TODAY-7`: die letzten 7 Tage);
- **Größe:** dasselbe mit einem `s` nach dem Operator (`>=s200000`); `K`, `M`
  und `G` funktionieren ebenfalls (`>s100M`).

Ein vorangestelltes `-` negiert die Angabe (`-*.bak`; `-<s1K` entspricht
`>=s1K`). Innerhalb jeder Art werden positive Angaben mit ODER verknüpft, außer
wenn auf `>a` unmittelbar `<b` folgt, was für den Bereich zwischen beiden steht
(`>=01/05/2026,<=31/05/2026`: Mai); `<>` schließt aus. Zwischen verschiedenen
Arten gilt UND: `*.txt,=TODAY,<s1000` sind alle `.txt`-Dateien von heute, die
kleiner als 1000 Bytes sind.

`*` (oder eine leere Eingabezeile) zeigt alles an. **↑** ruft frühere Filespecs
auf.

## Eine SQL-Bedingung (W)

**W** nimmt eine Bedingung entgegen, die wie der `WHERE`-Teil einer SQL-Abfrage formuliert ist. Die Liste zeigt dann zusätzlich zum Filespec nur noch die Dateien an, die diese Bedingung erfüllen. **Ctrl+W** filtert nur unter den markierten Dateien. Dies funktioniert im Verzeichnis, in Branch und in Showall, und nur über das, was eingelesen (*logged*) ist: Eine Datei, die Xylem nicht anzeigt, wird niemals zurückgegeben.

```sql
ext = 'jpg' AND size > 10000000
ext IN ('md', 'txt') AND age_days < 7
name LIKE 'bericht%' AND depth >= 2
```

Jede Datei entspricht einer Zeile mit folgenden Spalten:

| Spalte | Bedeutung |
|---|---|
| `name` | der Dateiname mit Erweiterung |
| `ext` | die Dateiendung in Kleinbuchstaben ohne Punkt (`''`, wenn keine vorhanden) |
| `path`, `dir` | der vollständige Pfad der Datei bzw. ihres Verzeichnisses |
| `depth` | wie viele Verzeichnisebenen unterhalb der Baumwurzel sich die Datei befindet |
| `size`, `disk` | die Dateigröße in Bytes und der belegte Speicherplatz auf dem Datenträger |
| `modified` | Datum der letzten Änderung, Ortszeit, im Format `2026-11-15 09:30:00`: `modified >= '2026-01-01'` |
| `age_days` | Tage seit der letzten Änderung (`age_days < 7`: die letzte Woche) |
| `mode` | Dateiberechtigungen als Zahl: `mode = 644` |
| `kind` | `file`, `link` oder `other` |
| `tagged` | ob die Datei markiert ist: `NOT tagged` |
| `git` | der zweistellige Git-Status (`' M'`, `'M '`, `'??'`, `'  '` wenn sauber); `NULL` außerhalb eines Repositories oder bei deaktiviertem Git |
| `type` | Dateityp wie in Alt+F (`'PNG image, 800 x 600'`, `'Rust source'`, `'empty'`); wird nur gelesen, wenn die Bedingung dies erfordert |
| `dup` | Gruppennummer der Duplikate (1, 2...), falls Alt+D ausgeführt wurde; `NULL`, wenn kein Duplikat |

Als Operatoren stehen die des [Tabellenfilters](viewer.md#der-sql-filter) zur Verfügung: `=`, `<>`, `<`, `<=`, `>`, `>=`, `LIKE`, `ILIKE`, `IN`, `IS NULL`, `AND`, `OR`, `NOT` und Klammern. Zahlen werden ganz ausgeschrieben (`10000000`, nicht `10e6`). Ungültige Spaltennamen werden unter Nennung der gültigen Spalten abgewiesen, und eine Bedingung, die keine Datei erfüllt, ändert die Liste nicht.

Die Titelleiste des Dateifensters zeigt die Bedingung nach dem Filespec an, und die Statusmeldung nennt die Anzahl der Treffer. **Ctrl+T** markiert alle Treffer, sodass sie kopiert, verschoben, gelöscht oder an ein [Batch](execute.md) übergeben werden können. **W** mit leerer Eingabe zeigt wieder die vollständige Liste. Beim ersten Mal zeigt **↑** einige anpassbare Beispiele.

### Im Dateiinhalt: contains()

`contains('text')` ist wahr für jede Datei, die diesen Text enthält (ohne Beachtung von Groß-/Kleinschreibung), genau wie die [Suche](search.md) mit Ctrl+S (`*` innerhalb des Textes steht für beliebige Zeichen auf derselben Zeile):

```sql
ext IN ('md', 'txt') AND contains('rechnung')
contains('TODO') AND NOT contains('DONE')
```

`contains` liest beim Ausführen immer die Dateien, was Zeit beansprucht: Ein Index speichert Wörter, keine Bruchstücke (zum Index siehe `words` unten). Vor dem Lesen fragt Xylem nach und nennt Datei- und Byteanzahl: `No index: read the contents of 1,234 files (2.3 GB)?` (bei Remote-Verbindungen wird ergänzt, dass die Dateien über SSH geladen werden). Es werden nur Dateien gelesen, deren Ergebnis vom Inhalt abhängt: Bei `ext = 'md' AND contains('x')` nur die `.md`-Dateien. Die Fußzeile zeigt den Fortschritt, und **Esc** bricht ab: Die Liste zeigt dann alle bisherigen Treffer.

Manche Dateien werden ausgelassen, und die Abschlussmeldung listet die Anzahl auf: Binärdateien (Nullbyte in den ersten 8 KiB), symbolische Links (werden nicht verfolgt), Dateien über 256 MiB und unlesbare Dateien. Eine ausgelassene Datei ist niemals ein Treffer, auch nicht bei `NOT contains(...)`.

Auf Cloud-Mounts wird `contains` verweigert, da jede Datei heruntergeladen werden müsste. Die Liste behält den Stand des Lesevorgangs bei: Später geänderte oder hinzugefügte Dateien erscheinen erst bei erneuter Ausführung von **W**.

`contains` steht als eigenständige Bedingung verknüpft mit `AND`, `OR` und `NOT` (`contains('a') = 1` ist unzulässig) und nimmt einen Text in Anführungszeichen entgegen.

### Ganze Wörter und ein Index: words()

`words('rechnung marz')` ist wahr für eine Datei, die alle diese Wörter in beliebiger Reihenfolge und an beliebiger Stelle enthält. Ein Wort ist eine Folge von Buchstaben und Ziffern, verglichen ohne Groß-/Kleinschreibung und ohne Umlaute/Akzente: `words('uber')` findet `über`. Anders als `contains` sucht es keine Teilwörter: `words('rechnung')` findet `rechnungen` nicht. Ein `*` direkt am Wortende fungiert als Präfixsuche: `words('rechn*')` findet `rechnung` und `rechnungen`, und `words('andre*')` findet `André` und `Andrea`.

```sql
words('rechnung 2024') AND ext IN ('txt', 'md', 'csv')
words('rechn* 2024')
```

`words` kann über einen **Index** des Zweigs beantwortet werden, wodurch kein direktes Lesen erforderlich ist. **Alt+N** im Baum (oder im Dateifenster für den Ordner der Datei) indiziert nach Bestätigung die eingelesenen Dateien des Zweigs: Er liest sie einmalig im Hintergrund ein, wobei **Esc** abbrechen kann und den bisherigen Stand sichert. Eine erneute Indizierung liest nur geänderte Dateien und entfernt gelöschte. Auf einem bereits indizierten Zweig bietet Alt+N Neuindizierung oder Löschen des Index an.

|  | Ohne Index | Mit Index |
|---|---|---|
| Antwort | jede Datei wird bei W gelesen | sofort |
| Kosten | Zeit bei jeder Abfrage | Zeit und Speicher beim Indizieren |
| Sucht nach | Textfragment (`contains`) oder Wörter und Präfixe (`words`) | Wörter und Präfixe (`words`) |
| Datei danach geändert | aktueller Inhalt | wird neu gelesen, wie ohne Index |
| Wo | überall, wo eingelesen ist | nur in Zweigen mit Alt+N-Index |

Vor dem Lesen nennt die Abfrage beides: `Index: 3,120 files up to date. No index: read the contents of 12 files (1.2 MiB)?`. Wenn der Index für alle Dateien antwortet, entfällt die Abfrage und die Meldung lautet `answered by the index`. Der Index liefert niemals veraltete Ergebnisse: Eine nach der Indizierung geänderte Datei (andere Größe oder Datum) wird sofort neu gelesen.

Vom Index ausgeschlossen sind: Binärdateien, Dateien über 64 MiB (W liest diese bei Bedarf), symbolische Links sowie Ordner und Dateien gemäß `index_ignore` in `config.toml` (oder in F12, Seite 4, Option E; standardmäßig: `.git`, `target`, `node_modules`, `.ssh`, `.gnupg`, `.env`, `*.key`, `*.pem`, `id_rsa`, `id_ed25519`). Das Entfernen von Geheimnismustern übernimmt Passwörter und Schlüssel in den Index. Dateien auf Remote-Maschinen, in Archiven oder Cloud-Mounts werden nicht indiziert.

Der Index ist eine Kopie der Wörter im Xylem-Cache (`~/.cache/xylem/index`, in einem nur für Sie lesbaren Ordner; `%LOCALAPPDATA%\xylem\cache\index` unter Windows), bis er mit Alt+N oder durch Löschen des Ordners entfernt wird. Ein unlesbarer Index wird gemeldet, die Dateien werden direkt gelesen und Alt+N baut ihn neu auf.

## Das Suchfeld (P)

Das Google-Desktop-Erlebnis ohne SQL: Drücken Sie **P** (oder **Ctrl+P** für markierte Dateien), um ein schwebendes Suchfeld zu öffnen.

Während Sie Wörter eingeben, erscheinen passende Dateien sofort beim Tippen:
- Wörter stimmen als ganze Wörter überein, und das letzte Wort sucht während des Tippens nach Präfixen (`rechnung mar` sucht nach `rechnung mar*`).
- Besitzt ein Zweig einen Index (**Alt+N**), erscheinen die Ergebnisse indizierter Dateien ohne Verzögerung.
- Gibt es nicht indizierte Dateien, nennt die Statuszeile deren Anzahl und Gesamtgröße in Bytes. Sie werden nur nach ausdrücklicher Bestätigung gelesen (drücken Sie **Alt+R** oder **Enter**, wenn noch keine indizierten Ergebnisse vorliegen).
- Jede Ergebniszeile zeigt die Markierung (`*`), den Dateinamen, das Verzeichnis, das Änderungsdatum und einen bereinigten Textausschnitt (~80 Zeichen, frei von Steuerzeichen und bidirektionalen Zeichen) der ersten Zeile mit den Suchwörtern.
- Tasten im Suchfeld:
  - **↑ / ↓**, **PgUp / PgDn**, **Ctrl+Home / Ctrl+End**: navigieren in der Ergebnisliste.
  - **Enter**: springt direkt zur ausgewählten Datei im Dateifenster und schließt das Suchfeld.
  - **Ctrl+V**: öffnet die Datei im Viewer an der Zeile des Textausschnitts.
  - **Ctrl+T**: schaltet die Markierung (Tag) der Datei direkt im Suchfeld um.
  - **Alt+R**: bestätigt und startet das Einlesen nicht indizierter Dateien.
  - **Esc**: bricht das Einlesen ab oder schließt das Suchfeld.

W ist ein erster Schritt zur Abfrage des Dateisystems: Tabellarische Ergebnisse folgen später.

Siehe auch: [das Dateifenster](files.md), [Markierungen](tagging.md) (Ctrl+T im
Baum markiert alles, was zum Filespec passt).
