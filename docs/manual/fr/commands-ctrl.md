# Le menu Ctrl

Ctrl avec une lettre exécute la commande sur les fichiers marqués. Dans la
fenêtre de fichiers, ce sont les fichiers marqués de la liste (le répertoire, ou
le Branch ou Showall ouvert). Dans l'arborescence, ce sont les fichiers marqués
du répertoire sous le curseur. Les lignes indiquant « tout ce qui est lu » font
exception.

Sans aucun fichier marqué, ces commandes affichent `No tagged files` et
n'agissent pas.

Pour afficher ce menu, maintenez Ctrl enfoncé ou appuyez sur F9. Voir
[les menus](menus.md).

## Commandes

Dans l'ordre du menu. « Arborescence » et « Fichiers » indiquent où la touche
fonctionne.

| Touche | Command | Arborescence | Fichiers | Action |
|---|---|---|---|---|
| Ctrl+A | Attributes | oui | oui | Modifie les permissions des fichiers marqués. [Permissions et dates](attributes.md) |
| Ctrl+B | Bookmark | oui | | Enregistre le répertoire sous le curseur comme signet. N'utilise pas les marques. [Signets](bookmarks.md) |
| Ctrl+B | Batch | | oui | Écrit un script ou une liste avec une ligne par fichier marqué, depuis un masque. [Batch](execute.md) |
| Ctrl+C | Copy | oui | oui | Copie les fichiers marqués dans un dossier de destination, sans leurs chemins. Alt+C conserve les chemins. [Copier et Déplacer](copy-move.md) |
| Ctrl+D | Delete | oui | oui | Supprime les fichiers marqués, après confirmation. Ils vont à la corbeille si `delete_to_trash` est actif. [Supprimer](delete.md) |
| Ctrl+G | Git | oui | oui | Ouvre le menu Git. Affiché seulement si l'option Git est activée. [Git](git.md) |
| Ctrl+M | Move | oui | oui | Déplace les fichiers marqués dans un dossier de destination. Sur les terminaux sans protocole clavier kitty, appuyez sur `F9` `M`. [Copier et Déplacer](copy-move.md) |
| Ctrl+N | New date | | oui | Modifie la date de modification des fichiers marqués. [Permissions et dates](attributes.md) |
| Ctrl+O | Open | | oui | Ouvre chaque fichier marqué dans le programme par défaut du système. [La fenêtre de fichiers](files.md) |
| Ctrl+P | Phrase | oui | oui | Ouvre la boîte de recherche sur les fichiers marqués : liste ceux contenant les mots saisis. [La boîte de recherche](filespec.md#la-boite-de-recherche-p) |
| Ctrl+R | Rename | oui | oui | Renomme les fichiers marqués avec un masque. [Renommer](rename.md) |
| Ctrl+S | Showall | oui | | Liste les fichiers marqués de tout ce qui a été lu. [La fenêtre d'arborescence](tree.md) |
| Ctrl+S | Search | | oui | Cherche un texte dans les fichiers marqués et démarque ceux qui ne le contiennent pas. [Rechercher dans les fichiers marqués](search.md) |
| Ctrl+T | Tag | oui | oui | Fichiers : marque toute la liste. Arborescence : marque tout ce qui est lu et correspond au Filespec. [Marquage et opérations groupées](tagging.md) |
| Ctrl+U | Untag | oui | oui | Fichiers : démarque toute la liste. Arborescence : démarque tout ce qui est lu et correspond au Filespec. |
| Ctrl+V | View | | oui | Ouvre les fichiers marqués dans la visionneuse, l'un après l'autre ; `N` et `P` font défiler les fichiers. [Rechercher dans les fichiers marqués](search.md) |
| Ctrl+W | Where | oui | oui | Conserve dans la liste les fichiers marqués répondant à une condition SQL. [Une condition SQL](filespec.md#une-condition-sql-w) |
| Ctrl+Y | Yank | oui | oui | Copie les chemins des fichiers marqués dans le presse-papiers. |

## Touches de fonction

| Touche | Où | Action |
|---|---|---|
| Ctrl+F5 | Fichiers | Place les fichiers marqués dans une nouvelle archive. [Archives](archives.md) |
| Ctrl+F7 | Arborescence | Marque les fichiers lus de la branche sous le curseur. |
| Ctrl+F8 | Arborescence | Les démarque. |
| Ctrl+F9 | Arborescence | Inverse leurs marques. |
| Ctrl+F11 | les deux | Enregistre un signet, comme Ctrl+B dans l'arborescence. |
| Ctrl+Shift+Del | les deux | Supprime les fichiers marqués définitivement, sans passer par la corbeille. |
| Ctrl+Z | les deux | Oops : restaure le dernier élément envoyé à la corbeille. |

## Dans une archive ou sur une machine distante

Le menu omet ce qui ne peut s'y exécuter.

| Contexte | Non listées |
|---|---|
| Archive | Attributes, Delete, Git, Move, New date, Rename, Ctrl+F5. Copy extrait. |
| Machine distante | Batch, Git, Ctrl+F5. |
