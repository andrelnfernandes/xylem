# Trouver des fichiers et des dossiers

Xylem a plusieurs façons de trouver les choses. Chacune répond à une question
différente, et elles forment trois groupes : aller à un endroit, filtrer la
liste, et regarder dans les fichiers.

| Vous voulez | Touche | Ce que vous obtenez |
|---|---|---|
| *Ce* dossier, par des morceaux de son nom | `\|` dans l'arborescence | le curseur de l'arborescence y va ([Recherche approximative](fuzzy.md)) |
| *Ce* fichier de la liste, par des morceaux de son nom | `\|` dans la fenêtre de fichiers | le curseur de fichiers y va ([Recherche approximative](fuzzy.md)) |
| Un chemin que vous connaissez, niveau par niveau | `\` ou `/` | le curseur de l'arborescence suit ce que vous tapez ([Treespec](treespec.md)) |
| Le nom suivant qui commence par une lettre | Shift+lettre | le curseur y va ([La fenêtre d'arborescence](tree.md)) |
| Les fichiers de ce nom, cette date ou cette taille | `F` | la liste ne montre qu'eux, jusqu'à ce que ce soit changé ([Filespec](filespec.md)) |
| Les fichiers qui remplissent une condition | `W` | la liste ne montre qu'eux, jusqu'à ce que ce soit changé ([SQL](sql.md)) |
| Les fichiers qui contiennent ces mots | `P` | une liste de résultats, le meilleur en premier ([La boîte de recherche](sql.md#the-search-box-p)) |
| Les fichiers marqués qui contiennent ce texte | Ctrl+S | les autres sont démarqués ([Rechercher dans les fichiers marqués](search.md)) |

## Aller à un endroit

La **recherche approximative** (`|`), le **Treespec** (`\`, `/`) et
**Shift+lettre** déplacent le curseur, et ne changent rien d'autre. Utilisez `|`
quand vous vous souvenez d'un morceau du nom mais pas de l'endroit ; le Treespec
quand vous connaissez le chemin ; Shift+lettre pour sauter dans le dossier où
vous êtes.

## Filtrer la liste

Le **Filespec** (`F`) et **W** décident des fichiers que montre la fenêtre de
fichiers, et le filtre reste jusqu'à ce que vous le changiez : le titre de la
fenêtre l'affiche. `F` sert aux noms, dates et tailles ; `W` à tout ce que Xylem
sait d'un fichier, contenu compris (`contains`, `words`). Ils s'appliquent
ensemble, et Ctrl+T marque ensuite tous les fichiers qui sont passés.

Pour chercher dans toute l'arborescence, ouvrez d'abord Branch (`B`) ou Showall
(`S`) : les filtres, et `|`, portent alors sur tous ses fichiers lus.

## Regarder dans les fichiers

**La boîte de recherche** (`P`) trouve des fichiers par les mots qu'ils
contiennent, à mesure que vous tapez, et répond instantanément sur une branche
indexée avec Alt+N. **Ctrl+S** cherche un morceau de texte dans les fichiers
marqués et démarque ceux qui ne l'ont pas, et Ctrl+V montre ensuite chaque
correspondance. Dans `W`, `contains('text')` et `words('a b')` font la même
chose dans une condition.

## Marquer ce qui a été trouvé

Chacune de ces recherches aboutit à des marques quand vous voulez agir sur
beaucoup de fichiers :

- `|` et `P` : Ctrl+T marque le résultat choisi ; dans `|`, Alt+T marque la
  liste.
- `F` et `W` : Ctrl+T dans l'arborescence marque tous les fichiers qui sont
  passés.
- Ctrl+S ne garde marqués que les fichiers qui contiennent le texte.

Ensuite Ctrl+C, Ctrl+M, Ctrl+D et Batch agissent sur les fichiers marqués. Voir
[Marquage](tagging.md).
