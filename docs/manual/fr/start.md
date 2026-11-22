# Installer et les cinq premières minutes

## Installation

Xylem fonctionne sous Linux (x86_64 et ARM64). La compilation Windows passe les
tests mais n'a pas été éprouvée au quotidien ; voir [Windows](windows.md).

**Binaire.** Téléchargez l'archive pour votre système depuis
`https://github.com/andrelnfernandes/xylem/releases`, extrayez-la et placez `xy`
dans un répertoire de votre `PATH` :

```bash
tar xzf xy-v0.7.0-x86_64-unknown-linux-musl.tar.gz
install -m 755 xy-v0.7.0-x86_64-unknown-linux-musl/xy ~/.local/bin/
```

Les binaires Linux sont statiques et fonctionnent sur toute distribution. Chaque
archive est accompagnée d'un fichier `.sha256` pour vérification avec
`sha256sum -c`.

**Paquet.** La même page propose des paquets `.deb` et `.rpm`, qui installent
également l'entrée de menu, les icônes et la page de manuel :

```bash
sudo apt install ./xylem-fm_*_amd64.deb
sudo dnf install ./xylem-fm-*.x86_64.rpm
```

Le paquet ne provenant pas d'un dépôt de distribution, il ne se met pas à jour
automatiquement. Installez les versions suivantes de la même façon.

**Cargo.** Nécessite Rust et un compilateur C (pour SQLite et zstd). La crate est
`xylem-fm` ; la commande installée est `xy` :

```bash
cargo install xylem-fm
```

## Démarrer

```bash
xy
```

ouvre le répertoire courant. `xy ~/projects` ouvre un autre répertoire, et
`xy user@host:` ouvre une [machine distante](remote.md).
[La ligne de commande](cli.md) présente les autres options.

La première fois, Xylem demande s'il peut chercher une nouvelle version une fois
par jour : **Y** ou **N** (voir [Nouvelles versions](configuration.md#new-versions)).

## Se déplacer dans l'arborescence

L'arborescence affiche le répertoire de départ et ses sous-répertoires. Seul le
répertoire de départ a été lu.

| Touche | Action |
|---|---|
| ↑ ↓ | déplacer le curseur |
| → | lire le répertoire sous le curseur et afficher son contenu |
| ← | le replier |
| `*` | lire le répertoire et tout ce qui est en dessous |

La fenêtre inférieure liste les fichiers du répertoire sous le curseur. Sur un
répertoire non encore lu, elle indique `Not logged`.

## Aller aux fichiers

**Enter** bascule vers la fenêtre de fichiers, qui prend alors la hauteur de
l'écran. Enter ou Esc revient à l'arborescence.

| Touche | Action |
|---|---|
| `V` | afficher le fichier dans la visionneuse |
| `E` | l'ouvrir dans votre éditeur |
| `C`, `M`, `D`, `R` | copier, déplacer, supprimer ou renommer |

## Marquer des fichiers et les copier

1. Dans la fenêtre de fichiers, appuyez sur `T` sur chaque fichier voulu. Un `■`
   apparaît devant le nom et le curseur descend.
2. Appuyez sur Ctrl+C. L'invite demande la destination.
3. Tapez un répertoire et appuyez sur Enter.

Les fichiers marqués sont copiés. `U` démarque un fichier individuel et Ctrl+U
démarque toute la liste.

## Quitter

`Q` quitte après confirmation. Alt+Q quitte et laisse le shell dans le
répertoire où vous vous trouviez, sous réserve d'avoir installé la fonction shell
décrite dans [Quitter et changer le répertoire du shell](quit.md).

## Ensuite

- [L'écran](screen.md) détaille chaque zone visible.
- [Les menus](menus.md) explique le rôle d'une lettre seule, avec Ctrl et avec
  Alt.
- [Vocabulaire](glossary.md) définit log, branch, tag et les autres termes.
- **F1** ouvre l'aide contextuelle sur ce qui se trouve à l'écran.
