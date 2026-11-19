# Die Befehlszeile

```
xy [--cwd-file DATEI] [ORT]
```

## Was geöffnet wird

| `ORT` | Öffnet |
|---|---|
| nichts | das aktuelle Verzeichnis |
| `VERZEICHNIS` | dieses Verzeichnis als Wurzel des Baums |
| `user@host:` oder `user@host:PFAD` | einen Rechner über SSH, im Home-Verzeichnis oder unter `PFAD` |
| `ssh://[user@]host[:port][/PFAD]` | dasselbe mit Portangabe |
| `docker://[user@]container[/PFAD]` | einen laufenden Container über `docker exec` |

Dieselben Adressen funktionieren beim Befehl `L` (Log) innerhalb von Xylem.
Siehe [entfernte Rechner](remote.md).

## Optionen

| Option | Funktion |
|---|---|
| `--cwd-file DATEI` | Schreibt bei Alt+Q das aktuelle Verzeichnis in `DATEI`. Eine Shell-Funktion liest es aus und wechselt das Verzeichnis. [Beenden und Verzeichnis der Shell wechseln](quit.md) enthält die Funktion. |
| `--default-config` | Gibt eine kommentierte Konfigurationsdatei mit allen Optionen und Standardwerten aus. |
| `--man` | Zeigt die Handbuchseite an. Bei umgeleiteter Ausgabe wird deren roff-Quelltext ausgegeben. |
| `-h`, `--help` | Zeigt die Optionen und den Pfad der Konfigurationsdatei an. |
| `-V`, `--version` | Zeigt die Version an. |

## Die Handbuchseite installieren

Die `.deb`- und `.rpm`-Pakete installieren sie automatisch. Aus einer Binärdatei
oder über Cargo:

```bash
mkdir -p ~/.local/share/man/man1 && xy --man > ~/.local/share/man/man1/xy.1
```

## Mit einer kommentierten Konfiguration starten

```bash
mkdir -p ~/.config/xylem && xy --default-config > ~/.config/xylem/config.toml
```

Dies überschreibt eine bestehende `config.toml`.

## Umgebungsvariablen

| Variable | Wirkung |
|---|---|
| `XDG_CONFIG_HOME` | Die Konfiguration wird aus `$XDG_CONFIG_HOME/xylem/` statt `~/.config/xylem/` gelesen. |
| `SHELL` | Die Shell des integrierten Terminals. |
| `VISUAL`, `EDITOR` | Der Editor für `E`, falls in der Konfiguration keiner festgelegt ist. |
| `XY_DOCKER` | Das Programm für `docker://`-Adressen. `XY_DOCKER=podman` verwendet Podman. |

Innerhalb des integrierten Terminals setzt Xylem `XY_TERMINAL=1` sowie in bash,
zsh und fish `XY_TAGGED` mit den markierten Dateien. Siehe
[das integrierte Terminal](terminal.md).

Siehe auch: [Konfiguration](configuration.md).
