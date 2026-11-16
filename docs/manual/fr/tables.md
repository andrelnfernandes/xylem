# Tableaux : feuilles de calcul, CSV et bases de données

Certains fichiers s'ouvrent sous forme de grille plutôt que de texte. Xylem se fie à l'extension, et seulement pour les fichiers non compressés de ce disque : un fichier d'une machine distante ou d'une archive s'affiche comme un contenu ordinaire, avec la note `Tables are shown only for files on this disk`.

| Format | Extensions | Noms des colonnes |
|---|---|---|
| CSV, TSV | `.csv` `.tsv` `.tab` | la première ligne, si au moins la moitié n'est pas numérique ; sinon A, B, C... |
| JSON Lines | `.jsonl` `.ndjson` | les clés des objets |
| Feuilles de calcul | `.xlsx` `.xls` `.xlsb` `.ods` | A, B, C... (la première ligne est une ligne comme les autres) |
| dBase, Clipper, FoxPro | `.dbf`, avec son fichier mémo `.dbt` | les noms des champs |
| SQLite | `.db` `.sqlite` `.sqlite3` | les colonnes de la table |
| Parquet | `.parquet` | les colonnes du fichier |

Le séparateur du CSV (virgule, point-virgule, tabulation ou barre verticale) est détecté d'après le début du fichier.

| Touche | Action |
|---|---|
| ↑ ↓ | ligne précédente ou suivante |
| ← → | colonne précédente ou suivante |
| PgUp PgDn | une page de lignes vers le haut ou vers le bas |
| Home End | première ou dernière colonne |
| Ctrl+Home Ctrl+End | première ou dernière ligne ; sous un filtre, parmi les lignes qu'il garde |
| `[` `]` | feuille précédente ou suivante d'un classeur, ou table d'une base SQLite, en boucle ; cela efface le filtre |
| `Tab` | la liste des feuilles ou tables : tapez une partie d'un nom pour la réduire, Enter en ouvre une et Esc revient ; en ouvrir une autre efface le filtre. Dans une base, les vues viennent après les tables, marquées `(view)` |
| `g` ou `:` | aller à une ligne par son numéro dans le fichier ; si le filtre l'a masquée, à la suivante qu'il a gardée |
| `\` ou `&` | le filtre SQL |
| `m` ou Insert | marque ou démarque l'enregistrement sous le curseur et passe au suivant ; **Ctrl+B** n'écrit alors que les marqués |
| `M` | efface les marques |
| `i` | la structure du tableau : ce que le fichier dit de chaque colonne (voir plus bas) |
| Entrée | l'enregistrement sous le curseur sous forme de fiche, un champ par ligne : ↑ ↓ choisissent le champ, ← → vont à l'enregistrement précédent ou suivant, `c` copie l'enregistrement entier (`NOM : valeur`, une ligne par champ) et `y` la valeur du champ, pour la coller dans un éditeur ou un terminal ; Échap revient. Ce qui est copié est ce que montre l'écran : les caractères de contrôle et de direction du texte, sauts de ligne compris, deviennent `·`, et coller dans un terminal n'exécute rien. Hors du tableau, Entrée ferme la visionneuse |
| `T` | aller au texte du fichier et revenir |
| `H` | du tableau au texte, puis à l'hexadécimal, puis retour au tableau |

Le cadre affiche `TABLE [2/5: nom]` s'il y a plusieurs feuilles ou tables, et la ligne du bas la position du curseur : `Cell B7 (Row 7 of 120)`. Avec un filtre, les lignes gardent leur numéro dans le fichier, et cette ligne indique aussi de quelle correspondance il s'agit : `Cell A13 (Row 13 of 31, match 2 of 8)`. La dernière ligne de la grille donne le contenu entier de la cellule et le filtre en cours. La recherche (`F`, `B`, `/`) ne regarde pas dans la grille et n'y trouve rien ; elle fonctionne en mode texte.

Les marques restent sous un filtre, et Batch écrit ce qui est marqué et conservé par le filtre. Elles appartiennent à la feuille ou à la table ouverte et sont perdues en en changeant. Si le fichier change après le marquage, Batch refuse, car d'autres enregistrements peuvent occuper ces places.

## La structure

`i` affiche, par-dessus la grille, les colonnes dans l'ordre avec ce que le fichier dit de chacune, comme `DISPLAY STRUCTURE` dans dBase et Clipper. Les flèches, PgUp, PgDn, Home et End la font défiler ; Esc, Enter ou `i` la ferment.

| Format | Ce qui est affiché |
|---|---|
| dBase, Clipper, FoxPro | nom, type (`C` caractère, `N` numérique, `D` date, `L` logique, `M` mémo, `F` virgule flottante, `I` entier, `B` double, `Y` monétaire, `T` date et heure), largeur et décimales ; au-dessus, le nombre d'enregistrements et la date de dernière mise à jour écrits dans l'en-tête, et la longueur d'un enregistrement |
| SQLite | le type déclaré de chaque colonne, `Not null`, la valeur par défaut et la place dans la clé primaire ; au-dessus, s'il s'agit d'une table ou d'une vue, et ses index avec leurs colonnes (32 au plus). Une vue montre les colonnes qu'elle donne ; une requête complète, celles de son résultat, avec le type de chacune quand elle vient directement d'une colonne de table |
| Parquet | le type Arrow de chaque colonne et s'il accepte les nulls ; au-dessus, les lignes et groupes de lignes que déclare le fichier et le programme qui l'a écrit |
| CSV, JSON Lines, tableurs | ils ne déclarent pas de types : ce que contiennent les lignes lues dans chaque colonne (entier, décimal, logique, texte, vide) et la valeur la plus large |

Noms, types et valeurs par défaut viennent du fichier et sont nettoyés comme les cellules : les caractères de contrôle et de direction du texte s'affichent `·`. Un `.dbf` dont l'en-tête se contredit affiche `?` pour les décimales.

## Le filtre SQL

`\` ou `&` demande une condition, comme le `WHERE` de SQL (le mot `WHERE` est facultatif), et n'affiche que les lignes qui la remplissent. L'invite commence avec le filtre en cours ; laissez-la vide pour revoir toutes les lignes. Une note indique `Filter matched N rows` ou `SQL error: ...` ; après une erreur, le filtre précédent reste actif.

```
age >= 30 AND name LIKE 'Ali%'
WHERE status IN ('open', 'late') OR total > 1000.50
NOT (city = 'Rio' OR city IS NULL)
B ILIKE '%silva%'
```

| Opérateur | Action |
|---|---|
| `=` `!=` `<>` `<` `<=` `>` `>=` | comparent |
| `AND` `OR` `NOT` et parenthèses | combinent des conditions (`AND` passe avant `OR`) |
| `LIKE` `ILIKE` | correspondent à un motif, `%` pour n'importe quel texte et `_` pour un caractère ; `ILIKE` ignore la casse |
| `IN (...)` `NOT IN (...)` | est, ou n'est pas, dans la liste |
| `IS NULL` `IS NOT NULL` | la cellule est, ou n'est pas, vide |

Les mots-clés s'écrivent en majuscules ou en minuscules.

- **Colonnes :** le nom de la colonne, sans guillemets et quelle que soit la casse, ou sa lettre, de `A` à `Z`. La lettre permet d'atteindre un nom avec des espaces ou des symboles. Une colonne qui n'existe pas compte comme vide, sans erreur.
- **Valeurs :** du texte entre apostrophes ou guillemets (`\'` pour une apostrophe à l'intérieur), des nombres, `TRUE`, `FALSE` et `NULL`. Ce qui est entre guillemets est toujours du texte, jamais une colonne.
- **Cellules :** le type vient de l'apparence de la cellule. Vide, c'est `NULL`, `true` et `false` sont des booléens, les nombres sont des nombres et le reste est du texte. Écrivez donc `code = 7`, et non `code = '7'` : un texte et un nombre ne sont jamais égaux, et une cellule contenant `007` est le nombre 7. Les comparaisons de texte respectent la casse.
- **Taille :** un filtre compte au plus 1 024 octets et 128 termes.

## Une requête complète dans une base

Dans une base SQLite, la même invite accepte une requête complète, commençant par `SELECT` ou `WITH`, avec tout ce que SQLite sait faire : `JOIN`, `GROUP BY`, `ORDER BY`, fonctions.

```
SELECT c.name, sum(o.price) FROM customers c JOIN orders o ON o.code = c.code GROUP BY c.name
```

Le cadre indique `QUERY:`, et la grille montre le résultat. Une condition filtre alors ces lignes, et **Ctrl+B** écrit tout ce que donne la requête. Laisser l'invite vide efface le filtre, puis revient à la table ; `Tab`, `[` et `]` vont aussi à une table. Une requête ne fait que lire, une à la fois : tout le reste est refusé, et une erreur laisse l'écran tel quel. Les limites ci-dessous s'appliquent : une requête lente s'arrête à 3 s et montre les lignes obtenues ; un Batch n'a pas cette limite. Xylem ne modifie pas les bases : `UPDATE`, `DELETE` et les autres reçoivent un message qui le dit. Pour en modifier une, utilisez un autre programme : `O` dans la fenêtre des fichiers l'ouvre dans celui défini par `opener`, et `'*.db' = 'sqlite3 %f'` dans `[editors]` fait que `E` l'ouvre dans `sqlite3`.

## Gros tableaux

Avec **V**, ces tableaux n'ont pas de limite de lignes : un fichier CSV ou TSV, un fichier JSON Lines, un fichier dBase (`.dbf`), et une table ou vue d'une base SQLite, ou une requête tapée dedans. La grille garde les lignes proches du curseur, au plus 2 000 et 32 Mio de texte, et lit les autres dans le fichier à mesure que le curseur se déplace, sans bloquer l'écran : une ligne pas encore lue s'affiche comme `…` un instant.

- **Le total** s'affiche pendant le comptage : `Row 1 of 2,000+ (counting…)`, puis `Row 1 of 8,412,903`. **Ctrl+End** va à la dernière ligne et `g` à n'importe laquelle ; avant la fin du comptage, `g` au-delà indique que le comptage n'y est pas encore arrivé.
- **Le filtre** (`&`) parcourt tout le tableau, en arrière-plan. La ligne sous la grille indique où il en est, `1,203 matches so far, row 2,300,000 of 8,412,903`, et les lignes s'affichent à mesure qu'elles sont trouvées. Il garde au plus 1 000 000 de correspondances, puis indique `first 1,000,000 matches`.
- **Esc** arrête le comptage ou le filtre, en gardant ce qu'ils ont trouvé ; l'Esc suivant ferme la visionneuse.
- La largeur des colonnes vient des premières lignes, et une colonne s'élargit quand une ligne plus large arrive à l'écran. Un fichier JSON Lines peut aussi gagner des colonnes pendant le comptage, quand une clé apparaît pour la première fois plus loin.
- **CSV, TSV et JSON Lines** sont lus une fois du début à la fin pendant le comptage, et Xylem note où commence chaque 256e ligne, de sorte qu'on atteint n'importe quelle ligne en en lisant au plus 255 autres. Cette note est gardée uniquement en mémoire, environ 8 octets pour 256 lignes, et disparaît à la fermeture de la visionneuse. Un enregistrement CSV avec un saut de ligne entre guillemets reste une seule ligne. Un enregistrement CSV de plus de 16 Mio arrête la lecture, et une ligne JSON Lines de plus de 512 Kio est sautée.
- **Les enregistrements dBase** ont tous la même taille, donc Xylem va directement à n'importe lequel. Le comptage ne regarde que si chaque enregistrement a été supprimé, car un enregistrement supprimé n'est pas une ligne : Xylem note où se trouve chaque 256e ligne, de la même façon.
- **Quand le fichier change** pendant qu'il est ouvert (il grandit, rétrécit ou est remplacé), ce que Xylem en avait noté ne vaut plus : plus rien n'y est lu, et l'en-tête indique `(the file changed: R reloads)`. **R** le relit.
- **Le texte et l'hexadécimal** d'un fichier qui s'ouvre en tableau ne sont lus que lorsque vous y allez (**T**, **H**) : jusque-là Xylem ne lit que le début du fichier, et un gros tableau s'ouvre donc aussitôt.

Autoview (F7) n'ouvre pas les bases de données, et ne lit que les premières lignes des autres tableaux. Les autres formats gardent pour l'instant la limite ci-dessous.

## Limites

Hormis les gros tableaux ci-dessus, un tableau contient au plus 50 000 lignes et 256 colonnes, et prend au plus ce que permettent les autres limites de la visionneuse : 3 s (100 ms dans Autoview), et un classeur qui se décompresse au-delà de la limite de taille (1 Mio dans Autoview) ne s'ouvre pas en tableau. Ce qui dépasse n'apparaît pas et le filtre ne le voit pas ; l'en-tête le signale : `(showing first 50,000 rows)` ou `(stopped by the time limit: showing N rows)`. Pendant la lecture du tableau, l'écran reste libre et une note le signale ; Esc arrête la lecture et montre ce qui est déjà arrivé, `(stopped by Esc: showing N rows)`, et l'Esc suivant ferme la visionneuse. Une cellule affiche au plus 2 048 caractères, puis `…`. Les caractères de contrôle et de direction du texte s'affichent comme `·`.

Quand un classeur (XLSX, XLSB, XLS, ODS) a plus de lignes que cela, l'en-tête et la note le disent : `(the sheet has more than 50,000 rows, the most Xylem shows: open it in Excel or another spreadsheet program to see them all (O))`, c'est-à-dire : ouvrez-la dans Excel ou un autre tableur pour les voir toutes. **O** l'ouvre dans le programme que votre système utilise pour elle.

## Tableaux lus par des bibliothèques C

Les bases SQLite (`.db`, `.sqlite`, `.sqlite3`) et les fichiers Parquet sont lus par des bibliothèques écrites en C, qui n'ont pas la sûreté mémoire de Rust. Ils ne s'ouvrent donc en tableau qu'avec `V`. Autoview lit tout ce que le curseur survole et affiche ces fichiers comme un contenu ordinaire, avec la note `Parquet or SQLite: V opens the table`. N'ouvrez une base de source non fiable que si vous acceptez ce risque. Les fichiers `.ods` et `.xls` ne s'ouvrent eux aussi en tableau qu'avec `V` : leur lecteur construit d'abord la feuille entière, et cette partie ne peut pas être interrompue.

Voir aussi : [la visionneuse](viewer.md).
