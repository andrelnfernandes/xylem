# Manuel de Xylem

Xylem est un gestionnaire de fichiers en mode texte avec l'écran et les touches
de XTree et ZTreeWin : une arborescence de répertoires en haut, les fichiers du
répertoire courant en bas et des statistiques à droite. Il lit (charge) les
répertoires en mémoire, ce qui permet à une seule liste de rassembler des
fichiers de toute une branche et à une seule commande de copier, déplacer,
supprimer, comparer ou renommer tous les fichiers marqués.

Xylem traite les fichiers qu'il n'a pas créés comme hostiles : le contenu ou le
nom d'un fichier ne doit jamais le faire planter, se figer, exécuter du code,
écrire hors de la destination ni épuiser la mémoire. [Sécurité](security.md)
détaille les mesures prises et ce qui n'est pas couvert.

**F1** (ou **Alt+H**, pour les terminaux qui réservent F1 pour eux) ouvre l'aide
sur ce qui se trouve à l'écran : dans une invite, le sujet de cette invite ;
dans l'arborescence ou la fenêtre de fichiers, le sujet de cette fenêtre.

| Touche dans l'aide | Action |
|---|---|
| Tab, Shift+Tab | sélectionne le lien suivant ou précédent |
| Enter | suit le lien |
| Backspace | revient en arrière |
| `I` | cet index |
| `K` | les touches en vigueur |
| `L` | change la langue |
| Esc | ferme |

Les versions de ce manuel en espagnol, français, allemand et chinois n'ont pas
été relues par des locuteurs natifs. Corrections :
`https://github.com/andrelnfernandes/xylem/issues`.

## Pour commencer

- [Ce qu'est Xylem](overview.md) : lecture, marques et ce que couvrent les commandes
- [Installer et les cinq premières minutes](start.md)
- [L'écran](screen.md) : l'arborescence, la fenêtre de fichiers, les statistiques, le menu
- [Les menus, Ctrl et Alt](menus.md) : une lettre seule, avec Ctrl, avec Alt, et F9
- [The mouse](mouse.md) (in English for now): turning it on in F12, the wheel, clicks and the footer
- [Vocabulaire](glossary.md) : log, branch, tag, filespec

## Commandes

- [Les menus DIR et FILE](commands.md) : les lettres seules
- [Le menu Ctrl](commands-ctrl.md) : les fichiers marqués
- [Le menu Alt](commands-alt.md) : la branche, les chemins et les outils
- [Git](git.md) : le menu Git (Ctrl+G)
- [Touches de fonction et symboles](commands-keys.md)
- [Les touches en vigueur](keys.md) : avec les modifications de configuration

## Fenêtres et vues

- [La fenêtre d'arborescence](tree.md) : déplacements, lecture, Branch et Showall
- [La fenêtre de fichiers](files.md) : modes d'affichage, Autoview, ouverture de fichiers
- [Écran partagé](split.md) : deux fenêtres côte à côte
- [La visionneuse](viewer.md) : texte, hexadécimal et tableaux, recherche, coloration
- [Filtrer les lignes](filter.md) : `&` dans la visionneuse, avec un guide des expressions régulières
- [Rechercher dans les fichiers marqués](search.md) : Ctrl+S et Ctrl+V
- [Informations](info.md) : Info, Avail, statistiques et tailles des répertoires
- [Utilisation du disque](usage.md) : où va l'espace (Alt+U)
- [Doublons](duplicates.md) : fichiers et dossiers au même contenu (Alt+D)

## Travailler avec les fichiers

- [Marquage et opérations groupées](tagging.md)
- [Copier et Déplacer](copy-move.md), [choisir un dossier](picker.md)
- [Supprimer, corbeille et Prune](delete.md)
- [Renommer et renommage par lot](rename.md)
- [Permissions et dates](attributes.md)
- [Filespec : le filtre de fichiers](filespec.md)
- [Treespec et Log](treespec.md)
- [Comparer](compare.md) et [Trier](sort.md)
- [Éditer, eXécuter et Batch](execute.md)
- [Archives](archives.md) : ZIP, 7z et TAR

## Au-delà de XTree

- [Le terminal intégré](terminal.md)
- [Machines distantes (SSH)](remote.md) : une autre machine ou un conteneur comme arborescence
- [Stockage cloud (rclone)](cloud.md) : Google Drive, S3 et autres
- [Trouver des fichiers avec SQL](sql.md) : une condition sur les fichiers et leur contenu (W), et les mots qu'ils contiennent (P)
- [Tableaux](tables.md) : feuilles de calcul, CSV, dBase, SQLite et Parquet en grille, avec un filtre SQL et chaque enregistrement en fiche
- [Signets](bookmarks.md)

## Comment faire

- [Copier des fichiers de plusieurs dossiers en gardant les chemins](howto-copy-with-paths.md)
- [Renommer plusieurs fichiers avec un compteur](howto-rename-mask.md)
- [Choisir des fichiers avec une condition SQL](howto-sql-filter.md)
- [Trouver les doublons et garder une copie](howto-duplicates.md)
- [Trouver ce qui remplit le disque](howto-disk-space.md)
- [Filtrer un CSV ou un tableur avec SQL](howto-table-sql.md)
- [Copier des fichiers d'une autre machine par SSH](howto-ssh-copy.md)
- [Utiliser les fichiers marqués dans une commande du shell](howto-terminal-tagged.md)

## Référence

- [Configuration](configuration.md) : l'écran F12, thèmes, touches
- [La ligne de commande](cli.md)
- [Terminaux et touches](terminals.md) : F1, Ctrl+M, le protocole clavier kitty
- [Sécurité](security.md)
- [Quitter et changer le répertoire du shell](quit.md)
- [Windows](windows.md)
- [Comparaison avec ZTreeWin](ztreewin.md)
- [Notes de version](releases.md)
