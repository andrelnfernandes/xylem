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

[Comparaison avec ZTreeWin](ztreewin.md) recense les commandes de ZTreeWin que
Xylem ne possède pas et les touches qui diffèrent.

## Fichiers non fiables

Xylem lit chaque fichier comme s'il avait été conçu pour l'attaquer. Afficher un
fichier n'exécute rien de son contenu, extraire une archive n'écrit jamais hors
de la destination et chaque lecteur s'arrête à une limite de taille.
[Sécurité](security.md) liste les limites et ce qui n'est pas couvert.
