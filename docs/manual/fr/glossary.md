# Vocabulaire

Les termes ci-dessous proviennent de XTree et de ZTreeWin. Les menus et ce
manuel les emploient avec les significations suivantes.

| Terme | Signification |
|---|---|
| Log | Lire un répertoire en mémoire : ses fichiers et les noms de ses sous-répertoires. `→` ou `+` lit un répertoire et `*` toute une branche. `L` (Log) est aussi la commande ouvrant un autre répertoire comme arborescence. [La fenêtre d'arborescence](tree.md) |
| Logged | Lu en mémoire. Seuls les fichiers lus apparaissent dans les listes, sont comptabilisés dans les statistiques et peuvent être marqués. |
| Not logged | Un répertoire dont le nom est connu mais dont le contenu n'a pas encore été lu. L'arborescence le signale par `?`. |
| Release | Libérer une branche de la mémoire avec `-`. Ses fichiers quittent les listes et perdent leurs marques. Rien n'est modifié sur le disque. |
| Relog | Relire un répertoire depuis le disque avec F3. |
| Branch | Une branche : un répertoire et tout ce qui se trouve en dessous. `B` liste les fichiers de la branche dans une liste unique. |
| Showall | La liste de tous les fichiers lus dans la fenêtre, ouverte avec `S`. |
| Tag | Une marque posée sur un fichier (`■`). Les commandes Ctrl agissent sur les fichiers marqués. Les marques restent en mémoire et sont perdues lors d'un Release ou en quittant. [Marquage et opérations groupées](tagging.md) |
| Filespec | Le filtre déterminant les fichiers affichés par la fenêtre, par nom, date et taille. Défini avec `F`. [Filespec](filespec.md) |
| Treespec | Un chemin saisi pour déplacer le curseur de l'arborescence. Ouvert avec `\` ou `/`. [Treespec](treespec.md) |
| Fenêtre | Une arborescence avec sa fenêtre de fichiers. F8 en ouvre une seconde à côté. [Écran partagé](split.md) |
| Racine | Le répertoire au sommet de l'arborescence. `L` le modifie, et Alt+L fait du répertoire sous le curseur la racine. |
| Matching | Les fichiers lus qui correspondent au Filespec. |
| Masque | Un motif générant un nom ou une ligne par fichier : `*.bak`, `photo_<001>.*`. Utilisé par Rename et Batch. [Renommer](rename.md) |
| Batch | Ctrl+B : écrit un fichier avec une ligne par fichier marqué, ou par enregistrement d'un tableau dans la visionneuse. [Batch](execute.md) |
| Prune | Alt+P : supprime un répertoire avec tout ce qu'il contient. [Supprimer](delete.md) |
| Oops | Restaure le dernier élément envoyé à la corbeille. `O` dans l'arborescence, Ctrl+Z dans les deux fenêtres. |
| Yank | `Y` : copie un chemin dans le presse-papiers. |
| Autoview | F7 : la fenêtre de fichiers affichant le contenu du fichier sous le curseur à côté de la liste. |
| eXecute | `X` : lance une commande shell, avec `%f` pour le fichier et `%t` pour les fichiers marqués. [eXecute](execute.md) |
| Avail | `A` dans l'arborescence : les systèmes de fichiers montés avec leur espace libre. |

Voir aussi : [ce qu'est Xylem](overview.md), [les menus](menus.md).
