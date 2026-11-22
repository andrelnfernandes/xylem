# Recherche approximative (|)

`|` trouve un dossier, ou un fichier de la liste, par des morceaux de son nom, à
n'importe quel niveau de l'arborescence : `cfgxy` trouve `config/xylem`, `rdme`
trouve `README.md`. Les résultats apparaissent au-dessus de l'invite à mesure
que vous tapez, le meilleur en premier, et le curseur de la fenêtre suit celui
qui est choisi.

Pour aller à un chemin que vous connaissez déjà, niveau par niveau, le
[Treespec](treespec.md) est plus rapide. Pour choisir de nombreux fichiers par
nom, date ou taille, ou par leur contenu, voir [Trouver des fichiers et des
dossiers](finding.md).

## Ce qui est cherché

Ce que cherche `|` dépend de la fenêtre où elle est ouverte :

- **Dans l'arborescence :** les dossiers lus, par leur chemin depuis la racine
  de l'arborescence (`docs plan` trouve `docs/plans`). Les dossiers cachés, et
  ce qu'ils contiennent, ne sont cherchés que tant que l'arborescence les
  montre.
- **Dans la fenêtre de fichiers :** les fichiers de la liste telle qu'elle est,
  avec le [filespec](filespec.md) et la [condition SQL](sql.md) (`W`) en
  vigueur. Dans le répertoire, par le nom ; dans Branch et Showall, aussi par le
  chemin (`src fzrs` trouve `src/fuzzy.rs`).

Pour trouver un fichier n'importe où dans l'arborescence, ouvrez Showall (`S`),
puis `|`.

Seul ce qui est lu est cherché, et rien n'est lu sur le disque ni sur le
réseau : cela fonctionne de la même façon sur une [machine distante](remote.md),
dans une [archive](archives.md) et sur un [montage cloud](cloud.md).

## Comment ça correspond

- **Les lettres dans l'ordre**, pas forcément accolées : `rdme` trouve
  `README.md`.
- **Les espaces séparent les termes**, et chaque terme doit correspondre, dans
  n'importe quel ordre : `xy docs` trouve `xylem/docs` et `docs/xylem`.
- **La casse** ne compte pas, sauf si le terme contient une majuscule : `Rd` ne
  trouve pas `readme`. Chaque terme décide pour lui-même.
- **Le meilleur en premier :** des lettres accolées comptent plus que des
  lettres éloignées ; une lettre au début d'un mot (au début, après `/`, `_`,
  `-`, `.` ou une espace, ou là où une minuscule devient une majuscule, comme
  dans `fileSpec`) compte plus qu'une lettre au milieu ; une correspondance dans
  le nom compte plus qu'une dans les dossiers au-dessus. À égalité, le chemin le
  plus court vient en premier, puis l'ordre de l'arborescence ou de la liste.

Les lettres qui ont correspondu sont affichées dans la couleur des touches. Un
nom trop long pour la fenêtre est coupé à gauche, avec `…`, pour que sa fin
reste visible.

## À l'écran

Le compteur en haut à droite indique où en est la recherche :

| Il indique | Signification |
|---|---|
| `type pieces of a name` | l'invite est vide |
| `120 found · searching 40%` | la recherche parcourt encore l'arborescence ; la liste se remplit |
| `120` | terminé : toutes les correspondances sont dans la liste |
| `500 best of 15,894` | terminé : la liste garde les 500 meilleures |
| `nothing matches` | terminé, sans correspondance |

Une grande arborescence est parcourue par étapes pendant que vous tapez : les
premiers résultats arrivent aussitôt et le reste suit. Quand vous ajoutez des
lettres à la fin, la recherche ne regarde que parmi ce qui correspondait déjà.
Si l'arborescence change pendant que l'invite est ouverte (un dossier lu,
libéré ou relu), la recherche recommence.

Un `*` devant un fichier signifie qu'il est marqué ; après un dossier, `3*` dit
combien de ses fichiers sont marqués.

## Touches

| Touche | Action |
|---|---|
| ↑ ↓ | choisit le résultat précédent ou suivant ; le curseur de la fenêtre suit |
| PgUp PgDn, Home End | choisit une page plus haut ou plus bas, le premier ou le dernier résultat |
| Enter | reste là où est le curseur et ferme l'invite |
| Esc | revient là où était le curseur, et ferme l'invite |
| Ctrl+T | marque le résultat choisi, ou le démarque s'il est marqué, et choisit le suivant |
| Alt+T | marque les fichiers des résultats de la liste |
| Ctrl+U | efface l'invite |

Avec la [souris](mouse.md) activée, un clic choisit un résultat et un double
clic y va.

## Marquer depuis les résultats

**Ctrl+T** sur un fichier le marque ; sur un dossier, il marque les fichiers de
ce dossier que la fenêtre de fichiers montrerait (avec le filespec et `W` en
vigueur). S'ils sont déjà tous marqués, il les démarque. Le choix passe ensuite
au résultat suivant, de sorte que Ctrl+T, Ctrl+T, Ctrl+T marque les résultats
l'un après l'autre.

**Alt+T** marque les fichiers de chaque résultat de la liste : au plus les 500
affichés. Quand il y avait plus de correspondances, la note sous la liste dit
combien ont été laissées de côté. Alt+T sur un dossier qui n'a pas été lu n'y
marque rien, et la note dit combien ont été ignorés.

Les marques restent après la fermeture de l'invite, comme toute marque : Ctrl+C,
Ctrl+M, Ctrl+D et Batch agissent ensuite sur elles. Voir [Marquage](tagging.md).

Marquer toutes les correspondances, pas seulement les 500 de la liste, est une
option, désactivée par défaut : Alt+T pourrait alors marquer des milliers de
fichiers que vous n'avez pas vus. Lisez [Marquer toutes les correspondances de
la recherche approximative](configuration.md#tagging-every-match-of-the-fuzzy-find)
avant de l'activer.
