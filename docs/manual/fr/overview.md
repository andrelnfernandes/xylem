# Ce qu'est Xylem

Xylem est un gestionnaire de fichiers pour le terminal. Son écran et ses touches
s'inspirent de XTreeGold et ZTreeWin : une arborescence de répertoires en haut,
les fichiers du répertoire sous le curseur en dessous, des statistiques à droite
et un menu de commandes d'une lettre dans le pied de page. La commande est `xy`.

## Lecture de répertoires (Logging)

Xylem lit un répertoire en mémoire quand vous le demandez et l'y conserve. C'est
la lecture (*logging*). `→` lit un répertoire, `*` lit toute la branche sous le
curseur et `-` libère une branche de la mémoire.

Deux commandes listent des fichiers lus de plusieurs répertoires à la fois : `B`
(Branch) liste les fichiers du répertoire et de tout ce qui est en dessous, et
`S` (Showall) liste tous les fichiers lus. Dans ces listes, vous marquez des
fichiers avec `T`, puis les commandes Ctrl agissent sur les fichiers marqués :
Ctrl+C les copie, Ctrl+D les supprime, Ctrl+R les renomme avec un masque.

Les fichiers d'un répertoire non lu n'apparaissent dans aucune liste et ne sont
affectés par aucune commande.

## Ce que couvrent les commandes

| Pour faire ceci | Utiliser | Sujet |
|---|---|---|
| Copier ou déplacer, avec ou sans chemins sources | `C`, `M`, Ctrl+C, Alt+C | [Copier et Déplacer](copy-move.md) |
| Supprimer, et restaurer ce qui a été mis à la corbeille | `D`, Ctrl+Z | [Supprimer](delete.md) |
| Renommer un fichier, ou plusieurs avec un masque | `R`, Ctrl+R | [Renommer](rename.md) |
| Afficher certains fichiers par nom, date ou taille | `F` | [Filespec](filespec.md) |
| Comparer deux répertoires | `C` dans l'arbre | [Comparer](compare.md) |
| Voir un fichier en texte, hexadécimal ou tableau | `V` | [La visionneuse](viewer.md) |
| Ouvrir ZIP, 7z et TAR comme des répertoires | Alt+F5 | [Archives](archives.md) |

[Les menus](menus.md) explique l'organisation des touches et
[les commandes](commands.md) les liste toutes.

## Ce que XTreeGold et ZTreeWin n'ont pas

- Un terminal intégré dans l'écran (Alt+X). L'arborescence suit le `cd` du shell,
  et Alt+O insère les fichiers marqués sur la ligne de commande.
  Voir [le terminal intégré](terminal.md).
- Des machines distantes via SSH et des conteneurs via `docker exec`, affichés
  comme une arborescence. Copy, Move, Rename, Delete, Attributes et New date y
  fonctionnent ; Edit et Batch non. Voir [machines distantes](remote.md).
- Google Drive, S3 et les autres stockages pris en charge par rclone.
  Voir [stockage cloud](cloud.md).
- Une condition SQL sur la liste de fichiers (`W`) :
  `ext = 'jpg' AND size > 10000000` retient les fichiers correspondants.
  Voir [Filespec](sql.md#une-condition-sql-w).
- SQL sur le contenu d'un fichier : la visionneuse affiche feuilles de calcul,
  CSV, JSON Lines, DBF, SQLite et Parquet comme tableaux et les filtre avec une
  condition `WHERE`. Voir [la visionneuse](viewer.md).
- Doublons (Alt+D) : fichiers et dossiers au contenu identique dans une branche.
  Sur un dossier rclone, compare les empreintes déjà disponibles sans rien
  télécharger. Voir [doublons](duplicates.md).
- Utilisation du disque (Alt+U) : dossiers et fichiers d'une branche triés selon
  l'espace occupé. Voir [utilisation du disque](usage.md).
- Un menu Git (Ctrl+G), désactivé par défaut. Voir [Git](git.md).

[Comparaison avec ZTreeWin](ztreewin.md) recense les commandes de ZTreeWin que
Xylem ne possède pas et les touches qui diffèrent.

## Fichiers non fiables

Xylem lit chaque fichier comme s'il avait été conçu pour l'attaquer. Afficher un
fichier n'exécute rien de son contenu, extraire une archive n'écrit jamais hors
de la destination et chaque lecteur s'arrête à une limite de taille.
[Sécurité](security.md) liste les limites et ce qui n'est pas couvert.
