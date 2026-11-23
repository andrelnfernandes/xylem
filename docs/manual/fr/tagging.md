# Marquage et opérations groupées

Les fichiers marqués (signalés par `■`) sont ceux sur lesquels agissent les
opérations groupées : les fichiers marqués **de la liste courante** (le
répertoire, ou la vue Branch ou Showall ouverte).

## Marquer

| Touche | Arborescence | Fenêtre de fichiers |
|---|---|---|
| `T` / `U` | marquer / démarquer les fichiers du répertoire | marquer / démarquer le fichier et descendre |
| Ctrl+T / Ctrl+U | marquer / démarquer tout ce qui est lu (et correspond au Filespec) | marquer / démarquer toute la liste |
| Ctrl+S | Showall des seuls fichiers marqués | |
| `I` | inverser les marques du répertoire | inverser les marques de la liste |
| Ctrl+F7 / F8 / F9 | marquer / démarquer / inverser les fichiers lus de la branche | |

## Opérations sur les fichiers marqués

| Touche | Action |
|---|---|
| Ctrl+C / Ctrl+M | les [copier / déplacer](copy-move.md) dans la destination |
| Alt+C / Alt+M | les copier / déplacer **en recréant leurs chemins** |
| Ctrl+D | les [supprimer](delete.md) |
| Ctrl+A | modifier leurs [permissions](attributes.md) |
| Ctrl+N | modifier leur [date](attributes.md) |
| Ctrl+R | les [renommer](rename.md) avec un masque |
| Ctrl+B | [Batch](execute.md) : un script ou une liste avec une ligne par fichier |
| Ctrl+F5 | les mettre dans une [archive](archives.md) |
| Ctrl+S / Ctrl+V | [y rechercher du texte / les afficher](search.md) |
| Ctrl+O | les ouvrir dans le programme par défaut |
| Ctrl+Y | copier leurs chemins dans le presse-papiers |
| Alt+O | les insérer dans le [terminal intégré](terminal.md) |

Dans eXecute, `%t` représente les fichiers marqués ([eXecute](execute.md)).

## Ce que veulent dire Ctrl et Alt

- Une lettre seule agit sur l'élément sous le curseur.
- Ctrl+lettre agit sur les fichiers marqués : Ctrl+C les copie, Ctrl+D les
  supprime.
- Alt+lettre est l'autre version d'une commande : la branche entière dans
  l'arborescence, les fichiers marqués avec leurs chemins dans la fenêtre de
  fichiers.

[Les menus](menus.md) explique la règle et comment afficher les menus Ctrl et
Alt avec F4. [Le menu Ctrl](commands-ctrl.md) et [le menu Alt](commands-alt.md)
détaillent chaque commande.

**Modifié dans la version 0.8.** Alt ne répète plus Ctrl : Alt+A, Alt+D, Alt+R,
Alt+T, Alt+U, Alt+W et Alt+Y ont changé. [Notes de version](releases.md) donne
l'ancienne et la nouvelle signification de chaque touche, et les lignes pour
`config.toml` qui restaurent les anciennes.
