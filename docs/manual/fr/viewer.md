# La visionneuse

`V` ouvre le fichier dans la visionneuse (Autoview, F7, l'affiche à côté de la
liste). Les fichiers binaires s'ouvrent en hexadécimal. Un gros fichier sur disque n'est pas lu en entier : la visionneuse lit ce que l'écran montre à mesure qu'il avance (voir [Gros fichiers](#gros-fichiers)). Ce qui ne peut pas être lu ainsi n'affiche que son début, jusqu'à la limite de taille.
Les feuilles de calcul, CSV, DBF, SQLite et Parquet s'ouvrent en tableau (voir [Tableaux](tables.md)).

| Touche | Action |
|---|---|
| ↑ ↓ PgUp PgDn Home End | faire défiler |
| ← → | faire défiler horizontalement |
| `H` | mode suivant : texte et hexadécimal ; dans un fichier Markdown, rendu, texte et hexadécimal |
| `O` | encodage suivant (UTF-8, CP850, 1252, CP437, UTF-16) |
| `C` | activer ou désactiver la coloration syntaxique |
| `W` | activer ou désactiver le retour à la ligne automatique |
| `#` ou `L` | activer ou désactiver les numéros de ligne (en mode texte) |
| `g` ou `:` | aller à une ligne (un décalage d'octet en hexadécimal, une ligne de la grille en mode tableau) |
| `&` | n'affiche que les lignes qui correspondent, texte simple ou regex ([filtrer les lignes](filter.md)) ; en mode tableau, le filtre SQL |
| Ctrl+B | une ligne de sortie par enregistrement ou ligne affichée ([Batch dans la visionneuse](execute.md)) |
| `E` | éditer le fichier ; de retour dans Xylem, la visionneuse affiche le nouveau contenu au même endroit (fichiers binaires uniquement avec un éditeur dans [`[editors]`](execute.md)) |
| `D` | avec Git activé : ce qui a changé depuis le dernier commit, et retour ([Git](git.md)) |
| `T` | tableau ou texte, dans un fichier ouvert en tableau ; rendu ou source, dans un fichier Markdown |
| `U` | dans la vue des fichiers marqués (Ctrl+V) : démarquer le fichier |
| `y` ou Ctrl+Insert | copier le chemin du fichier dans le presse-papiers |
| Esc | fermer |

## Rechercher

Comme dans ZTreeWin : **F** cherche depuis le début, **B** à rebours depuis la
fin, **/** (ou `S`) depuis la page courante et **\\** à rebours depuis celle-ci ;
**Space** poursuit dans la même direction, **+** et **-** choisissent la
direction. L'occurrence trouvée est mise en surbrillance. À l'invite de
recherche, **F2** active ou désactive la sensibilité à la casse (en ignorant la
casse, cela vaut aussi pour les lettres accentuées : `AÇÃO` trouve `ação`) et **F4**
bascule entre texte et hexadécimal (paires d'octets telles que `4d 5a`). Un `*`
au milieu du terme correspond à n'importe quelle suite de caractères sur la même
ligne (`error*disk`).

## Coloration syntaxique

Le code source et les scripts sont colorés dans environ 150 langages (Rust,
Python, shell, C, JavaScript, SQL, YAML, Markdown...), reconnus par leur
extension, leur nom (`Makefile`) ou leur première ligne (`#!/bin/bash`). Le
réglage par défaut est `syntax_highlight` (page 2 de la
[configuration](configuration.md)). Les couleurs restent lisibles sur l'arrière-plan
du thème actif, et seules les 20 000 premières lignes sont colorées, et aucune
ligne de plus de 10 000 caractères, afin que les très gros fichiers ne
ralentissent jamais la visionneuse.

## Markdown rendu

Les fichiers `.md` et `.markdown` s'ouvrent rendus, aussi dans Autoview, dans les archives et compressés (`x.md.gz`), et le cadre indique `RENDERED`. `T` montre la source au même endroit, et revient ; `H` passe par le rendu, le texte et l'hexadécimal.

Vous voyez les titres, le gras, l'italique, le barré, le code, les blocs de code avec coloration syntaxique (`C` la coupe), les listes imbriquées et numérotées, les listes de tâches (`[x]`), les citations, les lignes horizontales et les tableaux. Rien n'est jamais téléchargé, et aucun lien n'ouvre quoi que ce soit :

- Un lien montre sa véritable adresse à côté du texte, comme dans `sûr <https://example.com>`, pour que le texte ne se fasse pas passer pour une autre adresse.
- Une image est une ligne de texte : `[image] légende <photo.png>`.
- Un diagramme Mermaid est un bloc de code étiqueté `mermaid`.
- Le HTML et les commentaires HTML restent visibles comme texte.
- Le bloc YAML au début d'un fichier est un bloc de code étiqueté `yaml`.

Le texte tient toujours dans la fenêtre, donc `W` ne fait rien ici ; ← → atteignent les colonnes d'un tableau trop large. La recherche regarde le texte à l'écran, adresses comprises, et trouve une phrase qui continue sur la rangée suivante. `g` et `#` comptent les lignes de la source. Ctrl+B agit sur les lignes de la source, toutes. `&` passe en mode texte et y filtre.

Un fichier de plus de 8 Mio s'ouvre en source, avec un avis. Le rendu s'arrête aussi à la limite de temps (3 s, 100 ms dans Autoview) ou à sa limite de mémoire, et l'en-tête indique alors `(rendered only the start)`. Un tableau montre au plus 256 colonnes. Les caractères de contrôle apparaissent en `·`, y compris ceux écrits en entités comme `&#27;`.

Markdown lui-même cache certaines choses : les définitions de liens que rien n'utilise et les cellules au-delà des colonnes de l'en-tête. Pour tout voir, `T` montre la source. L'italique et le barré dépendent du terminal et de sa police, ils ont donc aussi leur propre couleur.

## Caractères de contrôle de direction

Certains caractères invisibles changent le sens d'écriture du texte et peuvent faire paraître un code différent de ce qu'il dit vraiment (Trojan Source, CVE-2021-42574). La visionneuse ne les cache jamais : chacun apparaît sous la forme d'un `·` rouge, et l'en-tête indique combien il y en a.

## Encodage du texte

Xylem trouve l'encodage seul, dans cet ordre : d'abord il décompresse le fichier, puis cherche un BOM (UTF-8, UTF-16 LE ou BE), puis de l'UTF-16 sans BOM, puis de l'UTF-8. Un fichier qui n'est rien de tout cela est un vieux texte : Xylem choisit CP850 (DOS, Europe occidentale) ou Windows-1252 en comptant les lettres accentuées. Les exports `.reg` de Windows sont en UTF-16 et s'ouvrent comme du texte normal.

**O** affiche le fichier dans l'encodage suivant : UTF-8, CP850, Windows-1252, CP437, UTF-16 LE, UTF-16 BE, puis de nouveau UTF-8. Le pied de page dit lequel, et l'en-tête montre son nom quand ce n'est pas de l'UTF-8. Si vous choisissez l'UTF-16 pour un fichier ouvert en hexadécimal, il passe en texte. Votre choix reste après avoir modifié le fichier et être revenu.

Pour toujours utiliser un seul encodage pour les vieux textes, au lieu de la devinette, réglez `legacy_charset` sur `cp850`, `cp1252` ou `cp437` (**K** à la page 2 de la [configuration](configuration.md)). La valeur par défaut est `auto`.

## Gros fichiers

Avec **V**, un fichier sur disque de plus de 16 Mio n'est pas chargé : la visionneuse garde les lignes proches de l'écran, environ 2 000, et lit les autres dans le fichier à mesure qu'il avance, sans bloquer l'écran. Une ligne pas encore lue s'affiche comme `…` un instant. Le fichier n'a ainsi pas de limite de taille, et la mémoire utilisée ne grandit pas avec lui.

- **Les lignes sont comptées en arrière-plan.** Le pied de page affiche `Line 1 of 1,234,567+ (counting…)` puis le total. **End** et **g** vont à toute ligne déjà comptée. Esc arrête le comptage, et ce qui a été compté reste.
- **La recherche** (`F`, `B`, `/`, `\`, Space) parcourt le fichier en arrière-plan : le pied de page affiche `Searching... line 1,200,000 (Esc stops)`, et l'écran va à l'occurrence quand elle est trouvée.
- **Le filtre de lignes** (`&`) parcourt tout le fichier en arrière-plan, et les lignes apparaissent à mesure qu'elles sont trouvées, jusqu'à 1 000 000. Esc l'arrête et garde ce qui a été trouvé.
- **L'hexadécimal** (`H`) lit les octets là où se trouve l'écran, et sa recherche parcourt aussi le fichier.
- **Un autre encodage** (`O`) relit les lignes ; en UTF-16 elles sont recomptées, car une ligne se termine ailleurs.
- **Pas de coloration syntaxique ni de Markdown rendu** : tous deux lisent le texte depuis le début.
- **Une ligne de plus de 1 Mio** est coupée à l'écran et se termine par `…` ; l'hexadécimal la montre en entier.
- **Si le fichier change** pendant qu'il est ouvert, l'en-tête indique `(the file changed: R reloads)`, et **R** le relit et revient à la ligne où il était.
- **Un Batch** (`Ctrl+B`) sur le texte lit toujours le fichier en mémoire, jusqu'à la limite de taille ci-dessous, et le signale quand ce n'est pas tout le fichier.

Pour atteindre une ligne, la visionneuse note où commence chaque 256e ligne, environ 8 octets pour 256 lignes, uniquement en mémoire. Les contrôles de direction du texte sont comptés sur tout le fichier en même temps que les lignes (en UTF-8), et l'en-tête montre combien en ont été trouvés jusque-là.

Un fichier compressé, une entrée d'archive, un fichier sur une machine distante et Autoview ne sont pas lus ainsi : on ne peut pas sauter à l'intérieur, ils suivent donc la limite de taille ci-dessous.

## Limite de taille

Dans ces cas, et pour les fichiers jusqu'à 16 Mio, la visionneuse ne lit que le début d'un fichier trop gros, et l'en-tête indique `(showing first N MiB)`. La quantité lue dépend de l'option `viewer_max_size` (**L** à la page 2 de la [configuration](configuration.md)) : `auto`, `16`, `64`, `256` ou `1024` (Mio). Avec `auto`, la valeur par défaut, la limite est 1/32 de la RAM de la machine, jamais moins de 16 Mio ni plus de 256 Mio (16 Mio si la RAM ne peut pas être lue). L'écran de configuration affiche la valeur utilisée, par exemple `auto (256 MiB)`.

Deux cas font exception. Autoview lit toujours 1 Mio. Un fichier sur une machine distante (SFTP) est lu jusqu'à 16 Mio au plus, car Esc ne peut pas annuler cette lecture.

Une limite plus grande coûte de la mémoire et du temps : avec `1024`, un texte compressé qui se décompresse jusqu'à cette taille occupe environ 2 Gio de RAM. Les lignes très courtes coûtent plus que leurs octets, car chaque ligne a sa propre gestion, et il en va de même pour les tabulations et les caractères de contrôle de direction cachés : la visionneuse garde des lignes jusqu'à environ deux fois la limite de taille en mémoire et, au-delà, l'en-tête indique `(showing first N lines)`. Un journal de lignes ordinaires (environ 100 octets) n'est jamais coupé pour cette raison.

## Fichiers compressés

Les fichiers qui se terminent par `.gz`, `.xz`, `.zst` ou `.bz2` s'ouvrent décompressés, en texte. Xylem regarde les premiers octets du fichier, pas le nom, et n'ouvre qu'un seul niveau : un `.gz` dans un `.gz` reste compressé. L'en-tête affiche `gzip → N bytes` (le format et la taille après décompression).

La décompression a des limites. La visionneuse lit jusqu'à la limite de taille, prend au plus 3 s et utilise au plus 64 Mio de mémoire pour le dictionnaire xz ou la fenêtre zstd. Dans Autoview, les limites sont 1 Mio, 100 ms et 16 Mio. Quand la limite de sortie ou de temps est atteinte, l'en-tête indique `(showing first N bytes decompressed)`. Une note dit ce qui s'est passé :

- `decompression stopped after 3 s` (100 ms dans Autoview)
- `compressed data truncated or corrupt` (données compressées tronquées ou corrompues)
- `needs more than 64 MiB of memory to decompress` (16 Mio dans Autoview, avec ` (V to view)` quand les 64 Mio de V peuvent suffire)

La note disparaît à la première touche, mais l'en-tête garde le problème : `(truncated or corrupt)`, `(incomplete: …)` quand la limite de mémoire a arrêté la décompression en cours de route, ou `(not decompressed: …)` quand rien n'a pu être décompressé (les octets bruts sont alors affichés en hexadécimal).

Autoview affiche la note dans son titre. En mode hexadécimal (`H`), on voit les octets décompressés. Les couleurs suivent le nom sans l'extension de compression : `app.rs.gz` est coloré comme du Rust.

Pour un `.tar.gz` (ou `.tar.xz`...), la note dit `TAR archive: Alt+F5 opens it as a folder` (Alt+F5 l'ouvre comme un dossier).

Pour voir les octets compressés, désactivez `viewer_decompress` (**J** à la page 2 de la [configuration](configuration.md)).

## Mode tableau

Les feuilles de calcul, CSV, DBF, SQLite et Parquet s'ouvrent en grille, avec un filtre SQL : voir [Tableaux](tables.md).

Voir aussi : [rechercher dans les fichiers marqués](search.md).
