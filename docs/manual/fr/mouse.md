# La souris

Xylem est fait pour le clavier, et la souris est désactivée jusqu'à ce que vous
l'activiez : **F12**, page 2, **R** (ou `mouse = true` dans les `[options]` du
[fichier de configuration](configuration.md)), puis **F3** pour enregistrer.
Elle fonctionne aussitôt.

La souris n'a jamais de commandes à elle. Un clic choisit une ligne, ou vaut la
touche écrite là où vous avez cliqué, et cette touche fait ce qu'elle fait
toujours : Delete demande toujours, et une commande qui ne peut pas s'exécuter
ici est toujours refusée.

## Ce qu'elle fait

| Où | Clic | Double clic | Molette |
|---|---|---|---|
| L'arborescence | choisit le répertoire ; la fenêtre de fichiers le suit | Enter : vers ses fichiers | déplace le curseur de 3 répertoires |
| La marque devant un dossier | `+` l'ouvre, `?` le lit et l'ouvre, le `─` d'un dossier ouvert le ferme | | |
| La fenêtre de fichiers | choisit le fichier et passe dans la fenêtre de fichiers | View, dans la visionneuse de Xylem | déplace le curseur de 3 fichiers (une ligne dans les modes en colonnes) |
| La barre de défilement du bord droit | au-dessus ou au-dessous du curseur de la barre : une page ; faites glisser ce curseur pour aller n'importe où | | |
| L'autre côté d'un [écran partagé](split.md) | en fait la fenêtre active, puis choisit | | déplace le curseur de ce côté, l'actif reste |
| Le [terminal intégré](terminal.md), l'éditeur dans le cadre | au programme, s'il a demandé la souris (mc, vim, htop, tmux) ; sinon rien | | au programme, s'il l'a demandée ; sinon en arrière dans les lignes sorties de l'écran, comme Shift+PgUp |
| L'arborescence au-dessus du terminal intégré | revient à Xylem, comme Alt+X | | |
| Le bas de l'écran | la touche de l'entrée : `Copy` est C, `Esc cancel` est Esc, `Yes` répond à la question | | |
| F12 | une option change, comme avec sa lettre (rien n'est enregistré avant F3) ; un thème de la liste (F4) est prévisualisé | le thème est choisi | déplace la sélection |
| Les signets, la boîte de recherche, Usage, Duplicates | choisit la ligne | son Enter | 3 lignes |
| L'historique d'une saisie (↑) | met cette entrée dans la saisie | | 3 lignes |
| Le texte d'une saisie | le curseur y va | | |
| Un tableau dans la visionneuse | choisit la cellule | son Enter : l'enregistrement | 3 lignes ; Shift+molette, une colonne |
| La visionneuse, l'aide | un lien de l'aide est suivi | | fait défiler, comme ↑ et ↓ ; Shift+molette fait défiler la visionneuse de côté |
| Partout où un menu a un sens | le bouton droit l'ouvre (ci-dessous) | | |

Un double clic n'ouvre jamais un fichier avec un programme du système : un
fichier que vous n'avez pas créé peut être hostile, et un clic par erreur ne
doit pas l'exécuter. Open (`O`) est toujours là.

Faire glisser avec le bouton gauche déplace le curseur le long d'une liste ;
au-delà de son bord supérieur ou inférieur, la liste défile d'une ligne à
chaque mouvement.

## Le menu du bouton droit

Le bouton droit place le curseur sur l'élément sous le pointeur, comme un clic,
et ouvre à côté du pointeur un menu de ce qu'on peut en faire. La touche
**Menu** du clavier, ou **Shift+F10**, ouvre le même menu pour l'élément sous
le curseur, sans souris.

| Sous le pointeur | Le menu |
|---|---|
| Un fichier | View, Edit, Open with the system, Info ; Tag ou Untag ; Copy, Move, Rename, Delete, Attributes, New date, Yank the path |
| Un fichier, avec des fichiers marqués | la même chose, et un groupe pour les marqués : Copy, Move, Rename (mask), Delete, View, Yank, Untag all |
| Un répertoire de l'arborescence | Files, Log, Relog, Expand all, Collapse, Release ; Branch, Showall ; Make a directory, Rename, Delete, Prune, Attributes ; Tag its files, Untag ; Terminal here, Usage, Duplicates, Bookmark it, Yank the path |
| La fenêtre de fichiers, après ses fichiers | Filespec, Sort, File display ; Tag all, Untag all, Invert tags |
| La visionneuse | Search, Next match, Go to line, et les touches de son mode (Wrap, Line numbers, Hex, SQL filter...), Edit, Yank the path, Close |

Chaque ligne montre sa touche à droite, telle que votre configuration la
définit : le menu est donc aussi une façon d'apprendre les touches. Ce qui ne
peut pas s'exécuter ici est omis : dans une archive, il n'y a ni Delete, ni
Move, ni Rename. Une ligne s'exécute comme sa touche : Delete demande toujours
d'abord.

↑↓ choisissent, Enter exécute, Esc ferme ; un clic hors du menu ne fait que le
fermer.

## Marquer

| Geste | Ce qu'il fait |
|---|---|
| Bouton du milieu, ou Ctrl+clic, sur un fichier | le marque ou le démarque ; le curseur reste où il est |
| La même chose sur un dossier de l'arborescence | le choisit et marque ses fichiers, ou les démarque quand tous étaient marqués |
| Shift+clic sur un fichier | marque tous les fichiers du curseur jusqu'à celui-ci, et le curseur y va |

Beaucoup de terminaux gardent Shift+clic pour leur propre sélection ; là, le
bouton du milieu ou Ctrl+clic marquent chaque fichier.

## Le focus, les menus et les questions

La molette déplace la liste sous le pointeur, pas celle qui a le focus : la
petite fenêtre de fichiers sous l'arborescence défile sans quitter
l'arborescence.

Dans les menus Ctrl et Alt (maintenus enfoncés, ou ouverts avec F4), un mot
vaut sa lettre avec Ctrl ou Alt : un clic sur `Copy` dans le menu Ctrl copie
les fichiers marqués.

Tant qu'une question, une saisie ou un travail en cours est à l'écran, les
fenêtres derrière ne prennent aucun clic : seules ses propres entrées en bas de
l'écran.

## Sélectionner du texte dans le terminal

Avec la souris activée, le terminal envoie les clics à Xylem, et faire glisser
ne sélectionne plus de texte à l'écran. Maintenez **Shift** en faisant glisser
(dans xterm, GNOME Terminal, Konsole, kitty, WezTerm, Windows Terminal), ou
**Option** dans iTerm2. Pour copier le chemin d'un fichier, Yank (`Y`) n'a pas
besoin de souris.

## Terminaux

Xylem demande les clics et les mouvements avec un bouton enfoncé, au format SGR
que connaît tout terminal actuel ; jamais les mouvements du pointeur seuls, qui
enverraient un flot d'événements pour rien, un poids sur SSH. Un programme dans
le terminal intégré ou l'éditeur dans le cadre reçoit la souris au format qu'il
a demandé, avec les coins de son propre panneau : elle n'atteint jamais le
reste de l'écran, ni votre terminal. Un glissement qui sort du panneau reste au
programme, retenu à son bord. Un programme lancé par eXecute ou Edit peut
désactiver la souris en se terminant ; Xylem la réactive quand il récupère
l'écran.
