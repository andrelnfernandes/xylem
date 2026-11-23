# Le menu Alt

Alt avec une lettre est l'autre version d'une commande. Dans l'arborescence,
elle couvre toute la branche sous le curseur. Dans la fenêtre de fichiers, Alt+C
et Alt+M agissent sur les fichiers marqués en préservant leurs chemins. Les
autres entrées ouvrent des outils ne s'appliquant pas à un seul fichier.

Pour afficher ce menu, maintenez Alt enfoncé ou appuyez sur F4 deux fois. Voir
[les menus](menus.md).

## Commandes

Dans l'ordre du menu. « Arborescence » et « Fichiers » indiquent où la touche
fonctionne.

| Touche | Commande | Arborescence | Fichiers | Action |
|---|---|---|---|---|
| Alt+B | Bookmarks | oui | oui | Ouvre le gestionnaire de signets : aller à un répertoire enregistré, le nommer, le supprimer. [Signets](bookmarks.md) |
| Alt+C | Compare | oui | | Compare toute la branche avec celle de l'autre fenêtre, par chemin relatif, et ouvre un Branch avec les fichiers marqués. [Comparer](compare.md) |
| Alt+C | Copy | | oui | Copie les fichiers marqués en recréant le chemin de chacun sous la destination. F7 à l'invite définit le point de départ du chemin. [Copier et Déplacer](copy-move.md) |
| Alt+D | Duplicates | oui | oui | Cherche les fichiers et dossiers au contenu identique dans la branche. Ne supprime rien sans confirmation. [Doublons](duplicates.md) |
| Alt+F | File display | oui | oui | Modifie l'affichage de la fenêtre de fichiers : noms en colonnes, tailles, détails, propriétaire, type, noms longs. [La fenêtre de fichiers](files.md) |
| Alt+H | Help | oui | oui | Ouvre l'aide, comme F1. |
| Alt+I | Info | oui | oui | Affiche type, taille, permissions, propriétaire et dates du fichier ou répertoire. [Informations](info.md) |
| Alt+K | Kbyte | oui | | Affiche la taille, le nombre de fichiers ou le pourcentage à côté de chaque répertoire de l'arborescence. [Informations](info.md) |
| Alt+L | Log | oui | oui | Fait du répertoire sous le curseur la racine de l'arborescence. Ce qui avait été lu et les marques sont oubliés. [Treespec et Log](treespec.md) |
| Alt+M | Move | | oui | Déplace les fichiers marqués en recréant le chemin de chacun sous la destination. [Copier et Déplacer](copy-move.md) |
| Alt+P | Prune | oui | | Supprime le répertoire avec tout son contenu, définitivement, après confirmation par saisie d'un mot. [Supprimer](delete.md) |
| Alt+Q | Quit here | oui | oui | Quitte et laisse le shell dans le répertoire courant. Nécessite la fonction shell décrite dans [Quitter et changer le répertoire du shell](quit.md). |
| Alt+R | Release | oui | | Libère la branche de la mémoire, comme `-`. Les marques sont perdues. |
| Alt+S | Sort | oui | oui | Trie la fenêtre de fichiers par nom, extension, date ou taille. [Trier](sort.md) |
| Alt+U | Usage | oui | oui | Liste les dossiers et fichiers de la branche selon l'espace occupé, les plus volumineux en premier. [Utilisation du disque](usage.md) |
| Alt+V | Views | oui | oui | Ouvre les vues de la liste : le Scan, qui montre ce qui mérite un coup d'œil, la Timeline, par date, et la Table, la liste dans la visionneuse de tableaux. [Vues](views.md) |
| Alt+N | iNdex | oui | oui | Indexe les mots des fichiers lus de la branche, pour que `words()` dans une condition SQL réponde instantanément. [L'index](sql.md#mots-entiers-et-un-index--words) |

## Autres touches

| Touche | Où | Action |
|---|---|---|
| Alt+X | les deux | Ouvre le terminal intégré, et en revient. [Le terminal intégré](terminal.md) |
| Alt+Z | les deux | Ouvre le terminal en plein écran. |
| Alt+O | les deux | Tape les fichiers marqués sur la ligne de commande du terminal, protégés par des guillemets. |
| Alt+F5 | Fichiers | Ouvre l'archive sous le curseur comme une arborescence. [Archives](archives.md) |
| Alt+F10 | les deux | Ouvre la configuration, comme F12. Sous GNOME, Alt+F10 maximise la fenêtre et n'atteint pas Xylem. [Configuration](configuration.md) |
| Alt+Shift+Del | les deux | Supprime les fichiers marqués définitivement, comme Ctrl+Shift+Del. |

## Dans une archive ou sur une machine distante

| Contexte | Non listées |
|---|---|
| Archive | Log, Move, Prune, les touches du terminal. Alt+C extrait les fichiers marqués avec leurs chemins dans l'archive. |
| Machine distante | Log (Alt+L), Alt+O. Alt+X ouvre un shell sur cette machine. |

## Modifié dans la version 0.8

Jusqu'à la version 0.7, Alt+A, Alt+D, Alt+R, Alt+T et Alt+U effectuaient la même
action que Ctrl avec cette lettre, Alt+W correspondait à Usage et Alt+Y à
Duplicates. [Notes de version](releases.md) donne la table et les lignes pour
`config.toml` rétablissant les anciennes touches.

Voir aussi : [les menus DIR et FILE](commands.md),
[le menu Ctrl](commands-ctrl.md).
