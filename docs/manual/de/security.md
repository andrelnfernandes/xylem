# Sicherheit

Xylem behandelt jede Datei, die es nicht selbst erstellt hat, als potenziell
feindlich: heruntergeladene Dateien, geöffnete Archive, von entfernten Servern
gesendete Namen, geklonte Repositorien. Weder Inhalt noch Name einer Datei
dürfen dazu führen, dass Xylem abstürzt, einfriert, Code ausführt, außerhalb des
Zielorts schreibt oder den Arbeitsspeicher erschöpft. Geschieht dies dennoch,
handelt es sich um einen meldepflichtigen Fehler.

## Was Xylem unternimmt

| Risiko | Maßnahme |
|---|---|
| Manipulierte Datei greift den Parser an | Text, Hexadezimal, Zeichensätze, komprimierte Dateien, Archive, Markdown, Tabellenkalkulationen, CSV, JSON Lines und DBF werden von in Rust geschriebenen Parsern gelesen. SQLite und Parquet werden über C-Bibliotheken ausschließlich beim Öffnen mit `V` eingelesen, niemals in Autoview. |
| Datei führt ein Programm aus | Kein externes Programm wird auf einer Datei ausgeführt, es sei denn, Sie fordern dies explizit mit `E`, `O` oder `X` an. |
| Kleine Datei dekomprimiert sich ins Unendliche | Jeder Leser unterliegt Limits für Lesemenge, Dekomprimierung, Laufzeit und Speicherverbrauch des Dekomprimierers. Nur eine Kompressionsstufe wird aufgelöst. Das Anzeigen einer Datei schreibt nichts auf die Festplatte. |
| Dateiname verschleiert seinen tatsächlichen Typ | Steuerzeichen und bidirektionale Steuerzeichen (Trojan Source) werden nicht an das Terminal übertragen. Namen mit verborgenen Richtungszeichen werden rot markiert und `O` verlangt vor dem Öffnen eine Bestätigung. |
| Archiv schreibt außerhalb des Zielorts | Pfade mit `..` und absolute Pfade werden abgewiesen, symbolische Links in Archiven werden nicht extrahiert und Setuid-Bits werden verworfen. Dieselben Prüfungen gelten für Namen, die ein SFTP-Server meldet. |
| Git-Repository führt eigene Skripte aus | Das Betrachten eines Repositories führt keine Hooks, kein `core.fsmonitor`, kein externes Diff und keine Textkonvertierung aus, es sei denn, sein Ordner ist in `git_trusted` eingetragen. |
| Abhängigkeit weist bekannte Schwachstelle auf | Die Abhängigkeiten werden in der Continuous Integration (CI) mit der RustSec-Datenbank abgeglichen. |

## Was Sie tun können

- Schalten Sie die Dekomprimierung im Betrachter aus (`viewer_decompress = false`,
  F12 Seite 2), wenn Sie nicht vertrauenswürdige Ordner mit Autoview durchsuchen,
  oder lassen Sie Autoview dort geschlossen.
- Lassen Sie Git deaktiviert (`git = false`, Standard) in Repositorien, die Sie
  nicht selbst erstellt haben.
- Führen Sie Xylem nicht als Root-Benutzer aus.
- `O` übergibt die Datei an ein anderes Programm. Die Limits von Xylem gelten
  für dieses Fremdprogramm nicht.

## Was nicht abgedeckt ist

- Programme, die von `E`, `O` und `X` gestartet werden.
- `rclone`, `ssh`, `docker` und `git`, die Xylem so ausführt, wie sie auf dem
  System installiert sind.
- Ein Terminal, das den empfangenen Text fehlerhaft verarbeitet.

## Ein Problem melden

Melden Sie Sicherheitsprobleme vertraulich über GitHub Security Advisories:
`https://github.com/andrelnfernandes/xylem/security/advisories/new`. Eröffnen Sie
dafür kein öffentliches Issue.

Siehe auch: [der Betrachter](viewer.md), [Archive](archives.md), [Git](git.md).
