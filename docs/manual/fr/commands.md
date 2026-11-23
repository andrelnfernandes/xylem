# Les menus DIR et FILE

Une lettre seule exécute la commande sur l'élément sous le curseur : le
répertoire dans l'arborescence (le menu DIR), le fichier dans la fenêtre de
fichiers (le menu FILE).

Certaines lettres correspondent à des commandes différentes selon la fenêtre.
`C` compare dans l'arborescence et copie dans les fichiers ; `M` crée un
dossier dans l'arborescence et déplace un fichier dans la fenêtre de fichiers.

## Commandes

Dans l'ordre des menus. « Arborescence » et « Fichiers » indiquent où la touche
fonctionne.

| Touche | Commande | Arborescence | Fichiers | Action |
|---|---|---|---|---|
| `A` | Available | oui | | Liste les systèmes de fichiers montés avec espace total, utilisé et libre. [Informations](info.md) |
| `A` | Attributes | | oui | Modifie les permissions du fichier, en octal. [Permissions et dates](attributes.md) |
| `B` | Branch | oui | oui | Liste les fichiers du répertoire et de tout ce qui est en dessous, en lisant la branche d'abord. Dans un Branch ou Showall, revient aux fichiers du seul répertoire. |
| `C` | Compare | oui | | Avec l'écran partagé, compare le répertoire avec celui de l'autre fenêtre et marque les fichiers qui diffèrent selon le critère choisi. [Comparer](compare.md) |
| `C` | Copy | | oui | Copie le fichier. L'invite demande la destination. [Copier et Déplacer](copy-move.md) |
| `D` | Delete | oui | oui | Supprime le fichier, après confirmation. Dans l'arborescence, supprime le répertoire s'il est vide. [Supprimer](delete.md) |
| `E` | Edit | | oui | Ouvre le fichier dans votre éditeur. [Éditer](execute.md) |
| `F` | Filespec | oui | oui | Définit les fichiers affichés par la fenêtre, par nom, date et taille. [Filespec](filespec.md) |
| `I` | Invert | oui | oui | Marque les fichiers non marqués et démarque les marqués : du répertoire dans l'arborescence, de la liste dans les fichiers. |
| `L` | Log | oui | | Ouvre un autre répertoire, ou une machine distante, comme arborescence de cette fenêtre. [Treespec et Log](treespec.md) |
| `M` | Makedir | oui | | Crée un sous-répertoire dans le répertoire sous le curseur. |
| `M` | Move | | oui | Déplace le fichier. [Copier et Déplacer](copy-move.md) |
| `N` | New date | oui | oui | Modifie la date de modification du fichier ou répertoire. [Permissions et dates](attributes.md) |
| `O` | Oops | oui | | Restaure le dernier élément envoyé à la corbeille. Dans la fenêtre de fichiers, Ctrl+Z. [Supprimer](delete.md) |
| `O` | Open | | oui | Ouvre le fichier dans l'application par défaut du système. [La fenêtre de fichiers](files.md) |
| `P` | Phrase | oui | oui | Ouvre la boîte de recherche : liste les fichiers lus contenant les mots saisis. [La boîte de recherche](sql.md#la-boîte-de-recherche-p) |
| `R` | Rename | oui | oui | Renomme le fichier ou le répertoire. [Renommer](rename.md) |
| `S` | Showall | oui | oui | Liste tous les fichiers lus de la fenêtre. |
| `T` | Tag | oui | oui | Fichiers : marque le fichier et descend. Arborescence : marque les fichiers du répertoire. [Marquage et opérations groupées](tagging.md) |
| `U` | Untag | oui | oui | Fichiers : démarque le fichier et descend. Arborescence : démarque les fichiers du répertoire. |
| `V` | View | | oui | Ouvre le fichier dans la visionneuse : texte, hexadécimal ou tableau. [La visionneuse](viewer.md) |
| `W` | Where | oui | oui | Conserve dans la liste les fichiers répondant à une condition SQL. [Une condition SQL](sql.md#une-condition-sql-w) |
| `X` | eXecute | oui | oui | Lance une commande shell. `%f` représente le fichier et `%t` les fichiers marqués. [eXecute](execute.md) |
| `Y` | Yank | oui | oui | Copie le chemin du fichier ou répertoire dans le presse-papiers. |
| `Q` | Quit | oui | oui | Quitte l'application après confirmation. [Quitter et changer le répertoire du shell](quit.md) |

Les touches de la dernière ligne du menu (F1, `*`, `?`, `\`, F3, Enter, F5, F7,
F8, F4) sont décrites dans [touches de fonction et symboles](commands-keys.md).

## Touches absentes du menu

Ce sont des touches alternatives pour des commandes du tableau.

| Touche | Équivalent à |
|---|---|
| Del | `D` |
| Shift+Del | `D`, suppression définitive sans passer par la corbeille |
| F10 | `Q` |
| Ctrl+Insert | `Y` |
| Ctrl+Z | `O` (Oops), dans les deux fenêtres |

## Dans une archive ou sur une machine distante

Le titre du menu devient ARCHIVE ou REMOTE, et les commandes qui ne peuvent s'y
exécuter sont omises.

| Contexte | Non listées |
|---|---|
| Archive | Delete, Edit, Log, Makedir, Move, New date, Oops, Rename, Attributes, eXecute. Copy extrait. |
| Machine distante | Edit, Oops, eXecute. Il n'y a pas de corbeille sur une machine distante. |

Voir [archives](archives.md) et [machines distantes](remote.md).

Voir aussi : [le menu Ctrl](commands-ctrl.md), [le menu Alt](commands-alt.md),
[les menus](menus.md).
