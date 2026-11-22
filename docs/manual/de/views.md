# Ansichten (Alt+V)

**Alt+V** öffnet die Ansichten der Liste: andere Arten, die Dateien des
Dateifensters anzusehen, für eine Frage, die Baum und Liste nicht gut
beantworten. Ein kleines Menü fragt, welche, jede mit ihrem Buchstaben:

| Taste | Ansicht | Was sie beantwortet |
|---|---|---|
| `S` | [Scan](#scan) | Was in diesen Dateien verdient einen Blick? |
| `T` | [Zeitleiste](#timeline) | Was hat sich geändert, und wann? |
| `L` | [Tabelle](#table) | Welche Dateien erfüllen diese Bedingung, als Tabelle gesehen? |

Eine Ansicht arbeitet auf **der aktuellen Liste**: dem Verzeichnis, dem Branch
oder dem Showall, mit der Filespec und der `W`-Bedingung, die gelten. Sie
liest nur: Nichts in einer Ansicht löscht, verschiebt oder ändert eine Datei.
Was Sie darin wählen, geht als Filter zurück in die klassische Liste, und von
dort wirken Ctrl+T, Ctrl+C oder Batch wie immer. **W ohne Text** entfernt
diesen Filter, wie bei einer Bedingung.

Jede Ansicht kann auch eine eigene Taste in `[keys]` haben
([Konfiguration](configuration.md)), standardmäßig ohne Taste: `view_scan`,
`view_timeline`, `view_table`.

## Scan

Die Berichte, die der [Betrachter](viewer.md) über möglicherweise feindliche
Dateien erstellt (ausführbare Dateien, Office-Dokumente, Patches), eine Datei
nach der anderen mit `V`, angewandt auf eine ganze Liste: ein
heruntergeladenes ZIP, ein geklontes Repository, ein USB-Stick. **Alt+V, S**
(vorher Branch mit `B`, um die Ordner darunter einzubeziehen).

```
      1  Names with control or text-direction characters
      1  Names that deceive (fatura.pdf.exe, spaces, a trailing dot)
      3  setuid, setgid, writable by anyone
      2  Links that leave the folder, or point to nothing
      1  Executables: packed, without PIE or NX, writable and executable
      1  Office: macros, remote template, embedded objects, hidden text
      0  Patches: dangerous paths, hidden characters, hidden changes
      0  Not analysed: over a limit, or the reader could not take it
```

| Kategorie | Was sie prüft |
|---|---|
| Names with control or text-direction characters | Den Namen jeder Datei und jedes Ordners: Steuerzeichen, Zeichen, die den Text umdrehen (ein Name, der als `exe.pdf` gezeichnet wird und `fdp.exe` ist), Bytes, die kein UTF-8 sind |
| Names that deceive | Die Endung eines Dokuments vor einer, die ausgeführt wird (`fatura.pdf.exe`, `photo.jpg.scr`), Leerzeichen vor der Endung, ein Name, der auf ein Leerzeichen oder einen Punkt endet |
| setuid, setgid, writable by anyone | Den Modus jeder Datei und jedes Ordners: Ein Ordner, in den jeder schreiben kann, zählt nur ohne das Sticky-Bit, das `/tmp` hat |
| Links that leave the folder, or point to nothing | Wohin jeder Link zeigt, nach seinem Text: Dem Link wird nie gefolgt, um zu lesen, was dort ist |
| Executables | ELF, PE und Mach-O, erkannt an ihren ersten Bytes und nicht am Namen: kein PIE, kein NX oder DEP, kein ASLR, ein ausführbarer Stack, gepackte Abschnitte (UPX und andere) oder solche, die zugleich schreibbar und ausführbar sind, Daten nach dem letzten Abschnitt |
| Office | Word, PowerPoint und OpenDocument: Makros, was das Dokument beim Öffnen von außen lädt (eine entfernte Vorlage), eingebettete Objekte, versteckter Text und ausgeblendete Folien |
| Patches | `.patch` und `.diff`: Dateien, die mit `..` hinausklettern, absolut sind, in `.git` gehen oder ausführbar werden; Namen mit versteckten Zeichen; Hunks, deren Zählungen nicht stimmen; Zeilen, die nur in Unsichtbarem geändert sind |
| Not analysed | Die Dateien, die die Leser nicht verarbeiten konnten, jede mit dem Grund: über einem Limit, ein kaputtes Paket, ein Leser, der gescheitert ist |

Die Namen und die Modi der Dateien sind schon im Speicher und erscheinen
sofort. Die Modi der Ordner, die Links und die Inhalte fragen die Festplatte,
im Hintergrund: Der Bildschirm füllt sich, sobald sie kommen, sagt, wie viele
noch fehlen, und **Esc** hält es an.

Die Inhalte gehen durch dieselben Leser wie die Berichte des
[Betrachters](viewer.md) (`V`), mit denselben Limits, eine Datei nach der
anderen. Eine Datei über einem Limit oder eine, die ein Leser nicht
verarbeiten konnte, kommt mit dem Grund unter **Not analysed**: Sie fällt nie
stillschweigend heraus. Höchstens `scan_max_files` Dateien (10000) werden auf
ihren Inhalt gelesen; darüber hinaus werden sie als nicht analysiert
aufgeführt ([Konfiguration](configuration.md)).

| Taste | Funktion |
|---|---|
| ↑ ↓ | bewegen |
| Enter | die Dateien der Kategorie, jede mit dem Grund, warum sie dort ist |
| Enter, auf einer Datei | zurück ins Dateifenster, das nur die Dateien dieser Kategorie behält, mit dem Cursor auf dieser |
| Enter, auf einem Ordner | der Ordner im Baum |
| `V`, auf einer Datei | sie ansehen, mit ihrem Bericht; Esc kehrt zum Scan zurück |
| `R` | erneut scannen |
| Esc | zurück zu den Kategorien, dann zur Liste |

Esc auf der Liste behält den Scan: Alt+V, S auf derselben Liste öffnet ihn
so, wie er war, ohne neu zu lesen. `R` liest neu.

### Wo er nicht hinsieht

- **Windows-Laufwerke und FAT, exFAT, NTFS, SMB, rclone-Mounts.** Dort gibt
  der Treiber jeder Datei denselben Modus, sodass Modi nichts aussagen, und
  der Scan zählt sie nicht: Die Kategorie sagt das stattdessen. Unter WSL
  ist das `/mnt/c` und die anderen Windows-Laufwerke.
- **In einem Archiv** (Alt+F5) sind die Modi der Dateien die, die das Archiv
  speichert, und sie zählen: Eine Setuid-Datei in einem TAR ist wissenswert.
  Jedes Mitglied wird mit einem Limit (64 MiB) entpackt und wie auf der
  Festplatte gelesen. Ordner und Links werden nicht geprüft. In einem TAR
  wird ein weit innen liegendes Mitglied nicht entpackt und als nicht
  analysiert aufgeführt.
- **Auf einem Cloud-Mount** (rclone) werden die Inhalte nicht gelesen: Jede
  Datei würde heruntergeladen. Die Namen und der Rest schon.
- **Auf einem entfernten Rechner** werden vorerst nur die Namen und die Modi
  der Dateien geprüft: Die Inhalte zu lesen würde jede Datei über SSH holen.

Der Scan ist kein Virenscanner. Er zeigt, was einen Blick verdient, und sagt
nicht, dass der Rest sicher ist.

## Zeitleiste

Die Dateien der Liste in einem Histogramm nach dem Datum ihrer Änderung: wann
ein Ordner viel zu tun hatte, was letzte Woche hinzukam, welche Fotos von
dieser Reise sind. Sie liest nichts von der Festplatte: nur die Daten, die
schon im Speicher sind.

```
 2026-11-18: 312 files, 1.4 GiB

                                    █
                                    █▆
 ▁▁▂▁▁▁▁▁▃▁▁▂▁▁▁▁▁                  ██▁
                                    ▲
 11-01     11-05     11-09     11-13     11-17
```

Der Maßstab wird nach den Daten gewählt: Stunden, Tage, Wochen, Monate oder
Jahre. Daten vor 1980 und in der Zukunft kommen in einen eigenen Balken an
jedem Ende, denn falsche oder feindliche Daten gibt es, und sie würden die
Grafik sonst plattdrücken. Dateien ohne Datum werden darunter gezählt.

| Taste | Funktion |
|---|---|
| ← → PgUp PgDn Home End | von Balken zu Balken gehen; die oberste Zeile nennt die Dateien und Bytes des Balkens |
| Shift+← → | einen Bereich von Balken wählen |
| `+` `-` | ein feinerer oder gröberer Maßstab (`+` von Tagen zu Stunden) |
| `B` | die Höhe der Balken nach Bytes statt nach Dateien |
| Enter | zurück ins Dateifenster mit nur den Dateien des Balkens oder Bereichs |
| Esc | zurück, ohne etwas zu ändern |

Enter macht aus der Wahl eine [W-Bedingung](sql.md) auf `modified`, etwa
`modified >= '2026-11-18 00:00:00' AND modified < '2026-11-19 00:00:00'`, mit
AND an die Bedingung angehängt, die schon galt. Der Titel des Dateifensters
zeigt sie, und **W ohne Text** entfernt sie.

## Tabelle

Die Liste öffnet sich im [Tabellenbetrachter](tables.md), eine Datei pro
Zeile, mit den Spalten der [W-Bedingung](sql.md): `name`, `ext`, `dir`,
`size`, `disk`, `modified`, `age_days`, `mode`, `kind` und `tagged`, sowie
`git` und `dup`, wenn eine Datei der Liste einen Git-Status hat oder zu einer
Gruppe von [Duplikaten](duplicates.md) gehört. Die Werte sind die, die `W`
sieht, sodass eine Bedingung, die in einem funktioniert, auch im anderen
funktioniert.

Dort hat die Liste, was Tabellen haben: den SQL-Filter (`\` oder `&`), den
Datensatz als Formular (Enter), das Kopieren eines Datensatzes, Markierungen
(`m`) und [Batch](execute.md) (Ctrl+B), eine Ausgabezeile pro Zeile. Eine
Liste mit Millionen Dateien öffnet sich sofort: Die Tabelle wird in Fenstern
gelesen, wie eine große Datei.

| Taste | Funktion |
|---|---|
| `\` oder `&` | der SQL-Filter der Tabelle |
| `W` | zurück ins Dateifenster, mit dem Filter als seiner W-Bedingung, mit AND an die angehängt, die galt |
| Esc | zurück, ohne etwas zu ändern |

Die Tabelle ist die Liste, wie sie beim Öffnen war: Eine Datei, die danach
markiert oder geändert wurde, erscheint darin beim nächsten Öffnen. `type`,
das jede Datei liest, ist hier keine Spalte; `W` hat es weiterhin.
