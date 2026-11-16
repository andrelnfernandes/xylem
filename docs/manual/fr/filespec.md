# Filespec : le filtre de fichiers

`F` définit quels fichiers s'affichent dans la fenêtre de fichiers (chaque
fenêtre a son propre filtre). Le filespec actif apparaît dans le titre de la
fenêtre. Les spécifications sont séparées par des espaces, `;` ou `,`, et sont
de trois types, comme dans ZTreeWin :

- **nom :** un motif glob (`*.rs`, `a?c*`, `[abc]*`), insensible à la casse ;
  entre guillemets, un nom peut contenir des espaces (`"mon fichier*"`) ;
- **date :** `<`, `=`, `>`, `<=`, `>=` ou `<>` suivi d'une date dans l'ordre du
  format configuré, avec `/`, `.` ou `-` et une année à 2 ou 4 chiffres
  (`>01/05/2026` ; `2026-05-01` fonctionne toujours), ou `TODAY` et `TODAY-n`
  (`>TODAY-7` : les 7 derniers jours) ;
- **taille :** de même avec un `s` après l'opérateur (`>=s200000`) ; `K`, `M` et
  `G` fonctionnent aussi (`>s100M`).

Un `-` au début inverse la condition (`-*.bak` ; `-<s1K` équivaut à `>=s1K`). Au
sein de chaque type, les spécifications positives sont combinées par OU (OR),
sauf `>a` immédiatement suivi de `<b`, qui désigne l'intervalle entre les deux
(`>=01/05/2026,<=31/05/2026` : le mois de mai) ; `<>` exclut. Entre différents
types, ET (AND) s'applique : `*.txt,=TODAY,<s1000` désigne les fichiers `.txt`
d'aujourd'hui de moins de 1000 octets.

`*` (ou une invite vide) affiche tout. **↑** rappelle les filespecs précédents.

## Une condition SQL (W)

**W** prend une condition formulée comme la clause `WHERE` d'une requête SQL, et la liste affiche alors uniquement les fichiers qui y répondent, en plus du filespec. **Ctrl+W** recherche uniquement parmi les fichiers marqués. Cela fonctionne dans le répertoire, dans Branch et dans Showall, et seulement sur ce qui est lu : un fichier que Xylem n'affiche pas ne sera jamais retourné.

```sql
ext = 'jpg' AND size > 10000000
ext IN ('md', 'txt') AND age_days < 7
name LIKE 'rapport%' AND depth >= 2
```

Chaque fichier constitue une ligne avec ces colonnes :

| Colonne | Définition |
|---|---|
| `name` | le nom complet, avec son extension |
| `ext` | l'extension, en minuscules et sans le point (`''` en l'absence d'extension) |
| `path`, `dir` | le chemin complet du fichier et de son dossier |
| `depth` | nombre de dossiers sous la racine de l'arborescence où se trouve le fichier |
| `size`, `disk` | la taille en octets et l'espace occupé sur le disque |
| `modified` | la date de dernière modification, heure locale, ex. `2026-11-15 09:30:00`: `modified >= '2026-01-01'` |
| `age_days` | jours écoulés depuis cette modification (`age_days < 7` : la dernière semaine) |
| `mode` | les permissions telles qu'écrites : `mode = 644` |
| `kind` | `file`, `link` ou `other` |
| `tagged` | si le fichier est marqué : `NOT tagged` |
| `git` | le statut Git en deux lettres (`' M'`, `'M '`, `'??'`, `'  '` si propre) ; `NULL` hors dépôt ou si Git est désactivé |
| `type` | le type de fichier comme dans Alt+F (`'PNG image, 800 x 600'`, `'Rust source'`, `'empty'`) ; lu seulement si la condition le requiert |
| `dup` | numéro du groupe de doublons (1, 2...), si Alt+D a été exécuté ; `NULL` si le fichier n'est pas un doublon |

Les opérateurs sont ceux du [filtre de tables](viewer.md#le-filtre-sql) : `=`, `<>`, `<`, `<=`, `>`, `>=`, `LIKE`, `ILIKE`, `IN`, `IS NULL`, `AND`, `OR`, `NOT` et parenthèses. Les nombres s'écrivent en entier (`10000000`, non `10e6`). Un nom de colonne inexistant est rejeté avec la liste des colonnes valides, et une condition à laquelle aucun fichier ne correspond ne modifie rien.

Le titre de la fenêtre affiche la condition après le filespec, et le message indique le nombre de fichiers correspondants. **Ctrl+T** les marque tous, permettant de les copier, déplacer, supprimer ou passer à un [Batch](execute.md). **W** avec une saisie vide réaffiche la liste complète. La première fois, **↑** affiche plusieurs exemples modifiables.

### Dans les fichiers : contains()

`contains('texte')` est vrai pour tout fichier contenant ce texte, sans tenir compte des majuscules/minuscules, comme la [recherche](search.md) de Ctrl+S (`*` dans le texte remplace toute chaîne sur la même ligne) :

```sql
ext IN ('md', 'txt') AND contains('facture')
contains('TODO') AND NOT contains('DONE')
```

`contains` lit toujours les fichiers lors de l'exécution, ce qui prend du temps : un index conserve des mots, non des fragments (pour l'index, voir `words` ci-dessous). Avant la lecture, Xylem demande confirmation en indiquant le nombre de fichiers et d'octets à lire : `No index: read the contents of 1,234 files (2.3 GB)?` (sur machine distante, il précise qu'ils seront téléchargés via SSH). Seuls les fichiers dont le résultat dépend du contenu sont lus : avec `ext = 'md' AND contains('x')`, seuls les fichiers `.md`. Le pied de page indique la progression et **Esc** arrête la lecture : la liste affiche alors les résultats trouvés jusqu'alors.

Certains fichiers sont ignorés, et le message final indique le décompte : fichiers binaires (octet zéro dans les 8 premiers Kio), liens symboliques (non suivis), fichiers dépassant 256 Mio et fichiers illisibles. Un fichier ignoré n'est jamais retourné, pas même avec `NOT contains(...)`.

Sur un point de montage cloud, `contains` est refusé : chaque fichier devrait être téléchargé. La liste conserve ce qui a été trouvé lors de la lecture : un fichier modifié ou ajouté ultérieurement ne réapparaît qu'en relançant **W**.

`contains` s'emploie seul comme condition reliée par `AND`, `OR` et `NOT` (`contains('a') = 1` est refusé), et prend un texte entre guillemets.

### Mots entiers et un index : words()

`words('facture mars')` est vrai pour un fichier contenant tous ces mots, dans n'importe quel ordre et emplacement. Un mot est une suite de lettres et chiffres, comparée sans casse ni accents : `words('creation')` trouve `Création`. Contrairement à `contains`, il ne cherche pas de sous-chaîne : `words('facture')` ne trouve pas `factures`. Un `*` placé juste après un mot accepte tout mot commençant par ce préfixe : `words('factur*')` trouve `facture` et `factures`, et `words('andre*')` trouve `André` et `Andrea`.

```sql
words('facture 2024') AND ext IN ('txt', 'md', 'csv')
words('fatur* 2024')
```

`words` peut être résolu par un **index** de la branche, évitant toute lecture directe. **Alt+N** dans l'arborescence (ou dans la fenêtre de fichiers, pour le dossier du fichier) indexe les fichiers lus de la branche après confirmation : il les lit une fois en arrière-plan, **Esc** pouvant arrêter le processus en conservant les fichiers indexés. Le relancer ne lit que les fichiers modifiés et retire ceux qui ont disparu. Sur une branche déjà indexée, Alt+N propose de réindexer ou de supprimer l'index.

|  | Sans index | Avec un index |
|---|---|---|
| Réponse | chaque fichier est lu lors de W | instantanée |
| Coût | temps à chaque requête | temps et espace lors de l'indexation |
| Recherche | fragment de texte (`contains`) ou mots et préfixes (`words`) | mots et préfixes (`words`) |
| Fichier modifié depuis | son contenu actuel | relu directement, comme sans index |
| Emplacement | tout emplacement lu | uniquement dans les branches indexées avec Alt+N |

Avant la lecture, la confirmation indique les deux données : `Index: 3,120 files up to date. No index: read the contents of 12 files (1.2 MiB)?`. Lorsque l'index répond pour tous les fichiers, aucune question n'est posée et le message indique `answered by the index`. L'index ne retourne jamais de fausse information : un fichier modifié après indexation (taille ou date différente) est relu aussitôt.

Ce que l'index ignore : fichiers binaires, fichiers de plus de 64 Mio (W les lit à la demande), liens symboliques et les dossiers et fichiers configurés dans `index_ignore` dans `config.toml` (ou dans F12, page 4, option E ; par défaut : `.git`, `target`, `node_modules`, `.ssh`, `.gnupg`, `.env`, `*.key`, `*.pem`, `id_rsa`, `id_ed25519`). Retirer les motifs de secrets copie clés privées et mots de passe dans l'index. Les fichiers distants, dans une archive ou sur un montage cloud ne sont pas indexés.

L'index est une copie des mots des fichiers conservée dans le cache de Xylem (`~/.cache/xylem/index`, dans un dossier accessible uniquement à vous ; `%LOCALAPPDATA%\xylem\cache\index` sous Windows) jusqu'à suppression via Alt+N ou effacement du dossier. Un index illisible (endommagé ou issu d'une autre version) est signalé, ses fichiers sont lus directement et Alt+N le recrée.

## La boîte de recherche (P)

**P** ouvre une boîte de recherche au-dessus de l'écran, et **Ctrl+P** l'ouvre sur les fichiers marqués. Elle trouve les fichiers selon les mots qu'ils contiennent, sans condition SQL.

Au fil de la frappe, les fichiers correspondants s'affichent immédiatement :
- Les mots correspondent à des mots entiers, et le dernier mot utilise la recherche par préfixe lors de la saisie (`facture mar` cherche `facture mar*`).
- Lorsqu'une branche dispose d'un index (**Alt+N**), les résultats des fichiers indexés apparaissent instantanément.
- S'il existe des fichiers non indexés, la barre d'état indique leur nombre et leur taille en octets. Ils ne sont lus que sur confirmation explicite (appuyez sur **Alt+R**, ou sur **Enter** en l'absence de résultat indexé).
- Chaque résultat affiche l'indicateur de marque (`*`), le nom du fichier, son dossier, sa date de modification et un extrait assaini (~80 caractères, sans caractères de contrôle ni inversions bidirectionnelles) de la première ligne correspondante.
- Touches dans la boîte de recherche :
  - **↑ / ↓**, **PgUp / PgDn**, **Ctrl+Home / Ctrl+End** : naviguent dans les résultats.
  - **Enter** : saute directement au fichier sélectionné dans la fenêtre de fichiers et ferme la boîte.
  - **Ctrl+V** : ouvre le fichier dans le visualiseur directement à la ligne de l'extrait.
  - **Ctrl+T** : bascule la marque (tag) du fichier sélectionné directement dans la boîte.
  - **Alt+R** : demande confirmation et lance la lecture des fichiers non indexés.
  - **Esc** : annule la lecture des fichiers non indexés ou ferme la boîte de recherche.

Voir aussi : [la fenêtre de fichiers](files.md), [marquage](tagging.md) (Ctrl+T
dans l'arborescence marque ce qui correspond au filespec).
