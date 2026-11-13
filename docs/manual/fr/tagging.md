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

Comme dans ZTreeWin :

- **Une lettre seule** agit sur l'élément sous le curseur.
- **Ctrl+lettre** agit sur les fichiers marqués : Ctrl+C les copie, Ctrl+D les
  supprime, Ctrl+T marque tout.
- **Alt+lettre** est l'autre version d'une commande : dans l'arbre, la branche
  entière (Alt+P Prune, Alt+R Release, Alt+U utilisation du disque, Alt+D
  Doublons, Alt+C Compare) ; dans la fenêtre des fichiers, les fichiers marqués
  avec leurs chemins (Alt+C, Alt+M) ; et les outils (Alt+S Sort, Alt+F File
  display, Alt+B signets, Alt+H aide, Alt+I Info, Alt+X le terminal).

**Changé dans la version 0.8.** Alt ne répète plus Ctrl, et certaines
touches ont changé :

| Touche | Jusqu'à 0.7 | À partir de 0.8 |
|---|---|---|
| Alt+A | permissions des fichiers marqués | rien (Ctrl+A) |
| Alt+D | supprimer les fichiers marqués | Doublons (supprimer : Ctrl+D) |
| Alt+R | renommer les fichiers marqués | Release de la branche, comme `-` (renommer : Ctrl+R) |
| Alt+T | tout marquer | rien (Ctrl+T) |
| Alt+U | tout démarquer | utilisation du disque (démarquer : Ctrl+U) |
| Alt+W | utilisation du disque | rien (Alt+U) |
| Alt+Y | Doublons | rien (Alt+D) |

Les anciennes touches peuvent revenir dans `[keys.any]` de la
[configuration](configuration.md), par exemple `"alt+w" = "usage"`.

## Menus Ctrl et Alt (F9)

Le pied de page a trois lignes sur un écran ordinaire : les commandes à lettre
par ordre alphabétique sur les deux premières, les touches de fonction et les
symboles sur la troisième. Sur un écran assez large pour tous les menus, il en
a deux : les lettres sur l'une, les touches sur l'autre. Chaque commande est dans l'un des menus (DIR ou FILE, CTRL, ALT) ;
dans une archive ou sur une machine distante, ils omettent ce qui n'y
fonctionne pas.

**F9** passe au menu Ctrl, puis au menu Alt, et revient au menu normal.
Lorsque l'un de ces menus est ouvert, la lettre suivante compte comme
Ctrl+lettre ou Alt+lettre (`F9` `M` = Ctrl+M) ; `Esc` annule. Dans les
terminaux prenant en charge le protocole clavier kitty, il suffit de **maintenir
Ctrl ou Alt** pour afficher le menu correspondant.

Relâcher la touche ramène le menu habituel, comme dans ZTree.

Le terminal doit signaler seuls l'appui et le relâchement de Ctrl et d'Alt, et
seul le protocole clavier kitty le fait : kitty, Ghostty, WezTerm, foot (Wayland)
et Alacritty récent le prennent en charge, et sous Ubuntu ou Debian la plupart
sont à un paquet de distance (`sudo apt install kitty`, ou `ghostty`,
`alacritty`, `foot`). Les terminaux basés sur VTE (GNOME Terminal, Ptyxis,
Terminator, Tilix) ne le prennent pas encore en charge : les menus y restent sur
F9.

**Ctrl+M :** dans la plupart des terminaux, Ctrl+M arrive comme Enter. Xylem
active le protocole clavier kitty lorsque le terminal le prend en charge (kitty,
WezTerm, foot, Ghostty, Alacritty récent, iTerm2), et Ctrl+M déplace alors les
fichiers marqués comme dans ZTree. Dans les terminaux basés sur VTE (GNOME
Terminal, Ptyxis, Terminator, Tilix), utilisez **F9 M** (ou Alt+M puis F6 à
l'invite).
