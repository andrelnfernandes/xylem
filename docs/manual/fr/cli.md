# La ligne de commande

```
xy [--cwd-file FICHIER] [LIEU]
```

## Que charger

| `LIEU` | Ouvre |
|---|---|
| rien | le répertoire courant |
| `RÉPERTOIRE` | ce répertoire, comme racine de l'arborescence |
| `user@host:` ou `user@host:CHEMIN` | une machine via SSH, dans son dossier personnel ou sur `CHEMIN` |
| `ssh://[user@]host[:port][/CHEMIN]` | identique, en précisant le port |
| `docker://[user@]container[/CHEMIN]` | un conteneur en cours d'exécution, via `docker exec` |

Les mêmes adresses fonctionnent dans la commande `L` (Log) au sein de Xylem.
Voir [machines distantes](remote.md).

## Options

| Option | Action |
|---|---|
| `--cwd-file FICHIER` | Avec Alt+Q, écrit le répertoire courant dans `FICHIER`. Une fonction shell le lit et change de répertoire. [Quitter et changer le répertoire du shell](quit.md) fournit cette fonction. |
| `--default-config` | Affiche un fichier de configuration commenté listant chaque option avec sa valeur par défaut. |
| `--man` | Affiche la page de manuel. Si la sortie est redirigée, imprime sa source roff. |
| `-h`, `--help` | Affiche les options et l'emplacement du fichier de configuration. |
| `-V`, `--version` | Affiche la version. |

## Installer la page de manuel

Les paquets `.deb` et `.rpm` l'installent automatiquement. À partir d'un binaire ou de Cargo :

```bash
mkdir -p ~/.local/share/man/man1 && xy --man > ~/.local/share/man/man1/xy.1
```

## Démarrer avec une configuration commentée

```bash
mkdir -p ~/.config/xylem && xy --default-config > ~/.config/xylem/config.toml
```

Cela écrase un éventuel `config.toml` existant.

## Variables d'environnement

| Variable | Effet |
|---|---|
| `XDG_CONFIG_HOME` | La configuration est lue depuis `$XDG_CONFIG_HOME/xylem/` au lieu de `~/.config/xylem/`. |
| `SHELL` | Le shell du terminal intégré. |
| `VISUAL`, `EDITOR` | L'éditeur pour `E`, lorsqu'aucun n'est défini dans la configuration. |
| `XY_DOCKER` | Le programme utilisé pour les adresses `docker://`. `XY_DOCKER=podman` utilise Podman. |

Dans le terminal intégré, Xylem définit `XY_TERMINAL=1` et, sous bash, zsh et
fish, `XY_TAGGED` contenant les fichiers marqués. Voir
[le terminal intégré](terminal.md).

Voir aussi : [configuration](configuration.md).
