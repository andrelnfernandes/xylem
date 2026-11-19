# Treespec et Log

## Treespec (`\` ou `/`)

Ouvre une invite contenant le chemin courant. À mesure que vous tapez, le
curseur de l'arborescence se déplace vers le premier répertoire commençant par
ce que vous avez saisi (insensible à la casse) :

- `/` ou `\` complète le nom et descend d'un niveau ;
- Tab et Shift+Tab complètent et passent au répertoire suivant ou précédent au
  même niveau ;
- Backspace remonte d'un niveau ;
- Ctrl+← retire le dernier dossier du chemin (`~/projects/xylem/docs/` devient
  `~/projects/xylem/`, avec le curseur sur `xylem`), et Ctrl+→ rend, un à un,
  les dossiers retirés ; taper autre chose commence un nouveau chemin ;
- Enter valide et Esc revient là où vous étiez ;
- ↑ ouvre les chemins précédents dans une [liste](execute.md#historique) (conservés entre les sessions).

Un chemin extérieur à l'arborescence devient un Log une fois validé. Dans Branch
et Showall, `\` va directement au répertoire du fichier sélectionné, en le
mettant en surbrillance.

## Log (L)

Ouvre un autre répertoire dans la fenêtre, en remplaçant l'arborescence (un
répertoire déjà présent dans l'arborescence courante y déplace simplement le
curseur). Une adresse de la forme `user@host:` ouvre une [machine
distante](remote.md). Le chemin peut être absolu, relatif au répertoire courant
ou commencer par `~` ; **F2** [le choisit dans une arborescence](picker.md) et ↑
liste les précédents. Les nouvelles fenêtres conservent l'[ordre de
tri](sort.md) et le mode d'affichage courants.

**Alt+L** fait du répertoire sous le curseur la racine de l'arborescence,
immédiatement : `L` vers un répertoire déjà dans l'arborescence ne fait que
déplacer le curseur, et Alt+L réduit alors l'arborescence à ce répertoire. Dans
la fenêtre des fichiers, c'est le répertoire affiché qui est pris (en Branch et
Showall, celui du fichier sélectionné). La fenêtre est rouverte à cet endroit :
ce qui avait été lu et les marques sont perdus ; `L ..` rouvre le répertoire du
dessus. Indisponible dans les archives et sur les machines distantes.

L'invite s'ouvre vide. ↑ ouvre une [liste](execute.md#historique) des lieux
utilisés auparavant, le dernier en bas : ceux ouverts avec Log et les dossiers
où des fichiers ont été copiés ou déplacés. Taper la filtre, Enter met un lieu
dans la ligne et Del le retire de la liste.

**F11** insère un [signet](bookmarks.md) dans l'invite, par son nom, plutôt que
de saisir l'adresse d'une machine distante ou d'un conteneur.

Voir aussi : [la fenêtre d'arborescence](tree.md), [signets](bookmarks.md).
