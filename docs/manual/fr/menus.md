# Les menus, Ctrl et Alt

Le bas de l'écran liste les commandes disponibles dans la fenêtre active. Il
existe quatre menus. Le titre à gauche indique lequel est affiché.

| Titre | Affiché quand | Ses commandes agissent sur |
|---|---|---|
| DIR | l'arborescence a le focus | le répertoire sous le curseur |
| FILE | la fenêtre de fichiers a le focus | le fichier sous le curseur |
| CTRL | Ctrl est maintenu enfoncé, ou après F9 | les fichiers marqués |
| ALT | Alt est maintenu enfoncé, ou après F9 deux fois | voir [le menu Alt](#alt-lautre-version-dune-commande) |

Dans une archive, le titre devient ARCHIVE, et sur une machine distante REMOTE.
Tous deux masquent les commandes qui n'y fonctionnent pas.

## Lire un menu

Chaque mot désigne une commande et sa lettre majuscule indique la touche :
`Copy` est `C`, `eXecute` est `X`, `iNdex` est `N`. Dans les menus CTRL et ALT,
la touche correspond à cette lettre combinée avec Ctrl ou Alt.

Les commandes sont rangées par ordre alphabétique de touche, avec Quit à la fin.
En dessous se trouve une ligne avec les touches de fonction et les symboles
(`F3 relog`, `? stats`).

Le pied de page comporte trois lignes. Sur un écran suffisamment large pour
accueillir tous les mots sur une seule ligne, il passe à deux lignes. Sur un
écran étroit, les mots sont raccourcis de façon uniforme et les commandes qui ne
tiennent pas sont remplacées par `…`.

## Ctrl : les fichiers marqués

Une lettre seule agit sur un unique élément. La même lettre avec Ctrl agit sur
les fichiers marqués de la liste :

| Touche | Seule | Avec Ctrl |
|---|---|---|
| `C` | copie le fichier | copie les fichiers marqués |
| `D` | supprime le fichier | supprime les fichiers marqués |
| `R` | renomme le fichier | renomme les fichiers marqués avec un masque |
| `T` | marque le fichier | marque tous les fichiers de la liste |

Dans l'arborescence, « la liste » désigne le répertoire sous le curseur. Trois
commandes ont une portée plus large : Ctrl+T et Ctrl+U marquent et démarquent
tout ce qui est lu et correspond au Filespec, et Ctrl+S liste les fichiers
marqués de tout ce qui est lu.

[Le menu Ctrl](commands-ctrl.md) décrit chaque commande.

## Alt : l'autre version d'une commande

Alt ne répète pas Ctrl. Son action dépend de la fenêtre active :

- Dans l'arborescence, la commande s'étend à toute la branche. Alt+P (Prune)
  supprime le répertoire et tout son contenu, Alt+R libère la branche de la
  mémoire, Alt+C compare la branche avec l'autre fenêtre.
- Dans la fenêtre de fichiers, Alt+C et Alt+M copient et déplacent les fichiers
  marqués en recréant le chemin de chacun sous la destination.
- Dans les deux fenêtres, Alt ouvre les outils globaux : Alt+S (Sort), Alt+F
  (File display), Alt+B (Bookmarks), Alt+I (Info), Alt+X (le terminal intégré).

[Le menu Alt](commands-alt.md) décrit chaque commande.

## Afficher les menus Ctrl et Alt

**F9** affiche le menu CTRL, un second appui sur F9 affiche le menu ALT, et un
troisième revient au menu initial. Pendant qu'un de ces menus est visible, la
lettre suivante est prise avec ce modificateur : `F9` `C` équivaut à Ctrl+C.
Esc annule sans rien lancer.

Sous kitty, Ghostty, WezTerm, foot et Alacritty récent, maintenir Ctrl ou Alt
affiche le menu correspondant, et relâcher la touche ramène DIR ou FILE. Ces
terminaux transmettent l'appui d'un modificateur isolé (protocole clavier
kitty). GNOME Terminal, Ptyxis, Terminator et Tilix ne le font pas ; les menus
n'y changent donc qu'avec F9.

## Ctrl+M

La plupart des terminaux envoient le même octet pour Ctrl+M et Enter. Xylem ne
peut alors pas les distinguer. Dans les terminaux gérant le protocole clavier
kitty, les deux touches sont distinctes et Ctrl+M déplace les fichiers marqués.
Dans les autres, appuyez sur `F9` `M`.

## Modifié dans la version 0.8

Alt+A, Alt+D, Alt+R, Alt+T et Alt+U effectuaient auparavant la même action que
Ctrl avec la même lettre. [Notes de version](releases.md) présente l'ancienne et
la nouvelle signification de chaque touche, ainsi que les lignes à insérer dans
`config.toml` pour rétablir les anciennes.
