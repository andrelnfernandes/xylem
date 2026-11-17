# La visionneuse

> **Ne couvre pas encore la 0.8.0.** Les tableaux sans la limite de 50 000 lignes ne sont pour l'instant que dans la version anglaise. Les grands textes lus à mesure que l'écran avance (« Big files ») aussi.

`V` ouvre le fichier dans la visionneuse (Autoview, F7, l'affiche à côté de la
liste). Les fichiers binaires s'ouvrent en hexadécimal. Pour les fichiers qui dépassent la limite de taille,
seul le début est affiché.
Les feuilles de calcul, CSV, DBF, SQLite et Parquet s'ouvrent en tableau (voir Mode tableau).

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
casse, les accents sont aussi pris en compte : `AÇÃO` trouve `ação`) et **F4**
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

## JSON, YAML et TOML

Un `.json` (aussi `.geojson`, `.har`, `.webmanifest`, et `.jsonc` et `.json5`, avec commentaires et virgule finale), un `.yaml` ou `.yml` et un `.toml` (aussi `Cargo.lock`) s'ouvrent comme un arbre qui se replie, et le cadre indique `TREE: JSON`. Un fichier minifié sur une seule ligne se lit aussi facilement qu'un autre. Un arbre de 2 000 nœuds au plus s'ouvre entièrement déplié ; un plus grand montre son premier niveau.

| Touche | Action |
|---|---|
| ↑ ↓ PgUp PgDn Home End | déplace le curseur |
| → | déplie le nœud (encore : y entre) |
| ← | replie le nœud, ou va à celui du dessus |
| `Enter` | déplie ou replie le nœud |
| `*` | déplie tout sous le nœud |
| `k` | copie le chemin du nœud : `.servers[2].name` en JSON, `servers.2.name` en YAML et TOML |

Le pied de page montre le chemin du nœud sous le curseur. Une recherche (`F`, `B`, `/`) regarde les clés et les valeurs, y compris dans les nœuds repliés, et ouvre le chemin jusqu'à ce qu'elle trouve. `T` montre le texte à la ligne du nœud, et revient ; `H` passe par l'arbre, le texte et l'hexadécimal. `&`, `g`, `#` et Ctrl+B agissent sur les lignes du texte.

- Un nœud replié dit ce qu'il contient : `{12 keys}`, `[340 items]`.
- Un nombre est montré tel que le fichier l'écrit : `12345678901234567890` reste ainsi, et `1_000` en TOML aussi.
- Une chaîne montre un saut de ligne comme `↵`.
- Une clé qui apparaît deux fois dans un même objet est montrée deux fois, marquée `dup key`, et l'en-tête les compte. Les programmes ne s'accordent pas sur celle des deux qui l'emporte, ce qui peut cacher une valeur lors d'une relecture.
- Une ancre YAML apparaît comme `&a1` et un alias comme `*a1` : un alias n'est jamais développé, donc un fichier fait pour se multiplier (un « billion laughs ») ne coûte pas plus que son texte. Une étiquette (`!Ref`) apparaît avant sa valeur. Plusieurs documents sont placés sous une racine, comme `document 1`, `document 2`...

Un fichier invalide, ou imbriqué sur plus de 512 niveaux, s'ouvre comme texte, et l'avis dit pourquoi et à quelle ligne. L'arbre est construit à partir du fichier entier : un fichier au-delà de la limite de lecture s'ouvre comme texte.

## Notebooks Jupyter

Un `.ipynb` s'ouvre rendu, cellule par cellule, et le cadre indique `RENDERED`. En haut : le noyau, et combien de cellules il y a de chaque sorte.

- Une cellule Markdown est rendue comme un fichier Markdown.
- Une cellule de code montre `In [n]:` et son code, dans les couleurs du langage du notebook.
- Ses sorties suivent, marquées `Out [n]:` : le texte d'un flux (`stderr:` quand ce sont les erreurs), le texte d'un résultat (ou son Markdown, rendu), et une erreur avec sa trace, sans les codes ANSI de la trace.
- Une sortie image, HTML, JavaScript, SVG ou LaTeX est une ligne qui dit ce qu'elle est et sa taille, comme `[image/png, 34 KiB: not shown]`. Rien d'un notebook n'est exécuté : ni son code, ni les scripts de ses sorties, ni ses widgets.
- Une cellule raw est montrée telle quelle.

`T` montre le JSON du notebook, et `H` ses octets. Un fichier qui n'est pas un notebook que la visionneuse sait lire (JSON cassé, format inconnu) s'ouvre sur sa source, avec un avis. Le format 3 (avec worksheets) est lu aussi.

## Documents Office

Les fichiers Word (`.docx`, `.docm`, `.dotx`), texte OpenDocument (`.odt`), PowerPoint (`.pptx`, `.pptm`) et Impress (`.odp`) s'ouvrent comme un document : titres, paragraphes, listes, tableaux, gras, italique et liens, avec l'adresse réelle de chaque lien. Une présentation montre chaque diapositive sous `Slide n`, suivie des notes de l'orateur. Les notes de bas de page et les commentaires, avec leurs auteurs, viennent à la fin. Une image est une ligne, comme `[image: media/image1.png]`. Le texte d'un document n'est jamais lu comme du Markdown.

Avant le texte, un bloc dit ce que le document contient en plus, car beaucoup de cela ne se voit pas dans le programme qui l'a créé :

- qui l'a écrit, qui l'a enregistré en dernier, quand et avec quoi ;
- `Contains macros`, s'il en a. Elles ne sont jamais lues ni exécutées ;
- `Fetches from outside when it opens` : un modèle distant, un objet OLE ou un cadre qui pointent hors du fichier, avec leurs adresses. Un modèle distant est la voie par laquelle des attaques comme Follina (CVE-2022-30190) sont arrivées. Rien n'est jamais téléchargé ici ;
- les objets incorporés, par nom et taille, et le nombre d'autres liens externes ;
- les modifications suivies : le texte inséré apparaît comme `[inserted: ...]` et le texte supprimé comme `[deleted: ...]`, car ce qui a été supprimé est souvent ce qui compte ;
- le texte masqué (`[hidden: ...]`), le texte blanc et le texte d'un point (`[invisible: ...]`), et les diapositives masquées.

`H` montre les octets, et revient ; `T` dit que le document n'a pas de texte propre. Chaque partie du fichier n'est décompressée que jusqu'à une limite (64 Mio chacune, 256 Mio en tout ; 4 Mio dans l'Autoview), et l'en-tête dit quand le document a été coupé. Une partie XML avec une DTD, ou imbriquée sur plus de 256 niveaux, est refusée. Les anciens `.doc` et `.ppt` s'ouvrent en hexadécimal, comme avant.

## Exécutables

Un exécutable, une bibliothèque ou un fichier objet s'ouvre comme un rapport, quel que soit son nom : ELF (Linux, BSD), PE (`.exe`, `.dll`, `.sys` de Windows, aussi .NET) et Mach-O (macOS, aussi les binaires universels à plusieurs architectures). Il est reconnu à ses premiers octets, jamais à son nom. Le rapport indique :

- ce que c'est (exécutable, bibliothèque, objet, core dump), pour quelle machine, 32 ou 64 bits, son point d'entrée, et son interpréteur, son sous-système ou son système minimal ;
- les **protections**, chacune `yes` ou `no`, avec `⚠` là où elle manque : en ELF, PIE, une pile qui n'exécute pas de code (NX), RELRO (complet ou partiel), le canari de pile, FORTIFY et les chemins de bibliothèques (RPATH, RUNPATH : un chemin relatif ou dans `/tmp` est marqué) ; en PE, ASLR, ASLR à haute entropie, DEP, Control Flow Guard ; en Mach-O, PIE et une pile exécutable ;
- la **signature** : s'il y en a une, et sa taille. Elle n'est **pas vérifiée**, et le rapport le dit ;
- les **signes** de quelque chose de caché : une section à forte entropie (compressée ou chiffrée), le nom d'un packer (UPX...), des données après la dernière section (un overlay, où des installeurs et certains malwares gardent des données), une section à la fois inscriptible et exécutable, un point d'entrée hors de toute section exécutable ;
- l'**identité** : build ID, debug link, UUID, chemin du PDB, le compilateur, la version de Go ; la date de compilation d'un PE, que l'éditeur de liens écrit et qui peut être n'importe laquelle ;
- les bibliothèques dont il a besoin, et les listes des sections, segments, imports, exports et symboles, de 2 000 lignes au plus chacune.

Rien n'est exécuté ni chargé pour faire le rapport : pas même `ldd`, qui exécute le programme dans certains cas. `H` montre les octets, et revient. Un fichier plus grand que la limite de lecture n'est lu que là où le rapport en a besoin, 256 Mio au plus en tout ; l'Autoview fait le rapport sur le début qu'il a lu, et dit qu'il peut manquer des choses.

## Certificats et clés

Un certificat, une chaîne ou un lot de certificats, une demande de certificat (CSR), une liste de révocation (CRL), une clé privée ou publique, une clé OpenSSH, `authorized_keys` et `known_hosts` s'ouvrent comme un rapport. Le PEM est lu dans un fichier nommé pour cela (`.pem`, `.crt`, `.cer`, `.key`, `.csr`, `.crl`, `.pub`...) ou sans extension (`id_ed25519`) ; le DER dans un `.der`, `.cer`, `.crt`, `.crl` ou `.csr` qui est une valeur DER entière. Un bloc PEM dans du code source ou un document laisse ce fichier tel quel.

- **Un certificat :** son subject et son issuer (et s'il est auto-signé), son numéro de série, sa validité avec les jours restants (expiré, pas encore valide, ou expirant dans 30 jours ou moins sont marqués `⚠`), sa clé publique (RSA sous 2048 bits et DSA sont marqués), sa signature (MD5 et SHA-1 sont marqués), ses noms (un nom en punycode `xn--` est signalé), l'usage de la clé, l'usage étendu, s'il s'agit d'une CA, les adresses de sa CRL et des informations de l'autorité, une extension critique inconnue (marquée), et son empreinte SHA-256.
- **Plusieurs certificats :** le rapport dit s'ils sont dans l'ordre, chacun émis par le suivant, vérifié uniquement par leurs noms et leurs identifiants de clé. **Aucune signature n'est vérifiée ici**, et le rapport le dit.
- **Une clé privée :** seulement son type et sa taille (RSA 2048 bits, EC P-256, PKCS#8 chiffrée avec PBES2...) et, pour OpenSSH, si elle est chiffrée et son empreinte. Ses nombres ne sont jamais montrés dans le rapport. Dans l'Autoview, une clé privée montre ce rapport, jamais son texte, pour qu'elle n'apparaisse pas à l'écran simplement en passant dessus ; `T` dans la visionneuse montre le texte.
- **Clés publiques, `authorized_keys` et `known_hosts` :** une ligne par clé, avec son type et sa taille, son empreinte SHA-256 telle que l'écrit `ssh-keygen -l`, et son commentaire. Une option `command=` est marquée, car elle change ce que la clé permet ; un hôte haché apparaît comme tel.

Rien n'est téléchargé : les adresses d'une CRL ou d'un OCSP sont seulement montrées. Avant qu'un parseur ne voie un bloc, son imbrication (64 niveaux au plus) et ses longueurs sont vérifiées ; un fichier a au plus 1 000 blocs de 64 Kio chacun. PKCS#12 (`.p12`, `.pfx`) et les clés PGP ne sont pas lus.

## Journaux

Un journal s'ouvre comme texte, avec les couleurs de ses niveaux, et le pied de page indique `Log`. Un fichier est un journal par son nom (`.log`, `.out`, `.err`, un fichier tourné comme `app.log.1` ou `app.log.2.gz`, `syslog`, `messages`, `kern.log`) ou par son contenu : la plupart de ses premières lignes commencent par une date et une heure (`2026-11-16 10:00`, `[2026-11-16 ...`, ou le `Nov 16 10:00:00` de syslog).

- **Niveaux :** le premier mot de niveau dans les 120 premières colonnes d'une ligne, en majuscules ou non, lui donne sa couleur : les erreurs (`ERROR`, `ERR`, `FATAL`, `CRIT`, `PANIC`...) en rouge, les avertissements (`WARN`, `WARNING`) en jaune, `DEBUG` et `TRACE` estompés. Le `<n>` de syslog en début de ligne compte aussi, ainsi que le niveau d'un journal en JSON (`"level":"error"`). Une ligne sans niveau, comme une trace de pile, prend celui de la précédente. `C` éteint les couleurs.
- **Couleurs ANSI :** les couleurs qu'un programme a écrites dans son journal (`ESC [31m`) sont montrées comme des couleurs, et leurs séquences sont retirées du texte. Toute autre séquence est écartée, car elle pourrait déplacer le curseur ou écrire dans le presse-papiers (OSC 52), et l'en-tête les compte, comme dans `[ANSI: 3 removed]`. `H` montre toujours les octets tels qu'ils sont, et une recherche porte sur le texte sans les séquences. Dans un fichier de plus de 16 Mio, lu à travers une fenêtre, les séquences apparaissent comme `·`, comme dans tout texte, et les couleurs sont celles des niveaux.
- **Suivre :** avec la dernière ligne à l'écran (End), les nouvelles lignes arrivent à mesure que le programme les écrit, et l'écran reste à la fin ; l'en-tête indique `[FOLLOW]`. Le fichier est examiné toutes les demi-secondes, aussi sur un partage réseau. Remonter, ou chercher, arrête le suivi ; End le reprend. Avec un filtre (`&`), les nouvelles lignes passent par lui. Si le fichier a rétréci ou changé au lieu de grandir (tourné ou réécrit), l'en-tête indique `(the file changed: R reloads)`, et rien n'est lu jusqu'à R. Un journal compressé n'est pas suivi.

## Patchs

Un `.patch` ou un `.diff`, ou tout texte qui commence comme l'un d'eux (`diff --git`, ou `---`, `+++` et `@@`), s'ouvre comme texte avec les couleurs d'un patch. Le Git diff (`D`) s'affiche de la même façon. L'en-tête le résume, comme dans `[12 files, +340 −98]`, et le pied de page dit où se trouve le haut de l'écran : `3/12 src/viewer.rs @@ 2/5`.

| Touche | Action |
|---|---|
| `]` `[` | fichier suivant ou précédent |
| `}` `{` | hunk suivant ou précédent |
| `Tab` | la liste des fichiers, avec `+` et `−` pour chacun ; des lettres la filtrent, `Enter` va à l'un d'eux |

Certains changements font plus que modifier des lignes, et passent facilement inaperçus à une lecture rapide. Ils sont montrés en rouge, et l'en-tête les compte comme `to check` :

- un fichier qui devient exécutable (`100755`), un lien symbolique (`120000`) ou un sous-module (`160000`) ;
- un chemin absolu, qui remonte avec `..`, qui entre dans `.git` (un hook est du code que Git exécute), ou qui contient des caractères de contrôle.

L'en-tête compte aussi :

- `look unchanged` : une ligne retirée et une ligne ajoutée qui ne diffèrent que par ce qui ne se voit pas, comme des espaces, `\r`, des caractères invisibles ou des lettres qui se ressemblent (un `а` cyrillique pour un `a` latin). La paire est montrée en jaune ;
- `hunks with wrong counts` : un hunk dont la ligne `@@` ne correspond pas à ses lignes. L'index suit les lignes qui sont là, jamais les nombres ;
- `whitespace` : des lignes ajoutées avec une espace en fin de ligne, ou une espace avant une tabulation dans l'indentation, comme le signale `git diff --check`.

Une série de `git format-patch` montre chaque message dans la liste, par son sujet. Rien n'est jamais appliqué. Dans un fichier de plus de 16 Mio, lu à travers une fenêtre, les couleurs restent, mais il n'y a pas d'index : `]`, `}` et Tab le disent.

## Caractères de contrôle de direction

Certains caractères invisibles changent le sens d'écriture du texte et peuvent faire paraître un code différent de ce qu'il dit vraiment (Trojan Source, CVE-2021-42574). La visionneuse ne les cache jamais : chacun apparaît sous la forme d'un `·` rouge, et l'en-tête indique combien il y en a.

## Encodage du texte

Xylem trouve l'encodage seul, dans cet ordre : d'abord il décompresse le fichier, puis cherche un BOM (UTF-8, UTF-16 LE ou BE), puis de l'UTF-16 sans BOM, puis de l'UTF-8. Un fichier qui n'est rien de tout cela est un vieux texte : Xylem choisit CP850 (DOS, Europe occidentale) ou Windows-1252 en comptant les lettres accentuées. Les exports `.reg` de Windows sont en UTF-16 et s'ouvrent comme du texte normal.

**O** affiche le fichier dans l'encodage suivant : UTF-8, CP850, Windows-1252, CP437, UTF-16 LE, UTF-16 BE, puis de nouveau UTF-8. Le pied de page dit lequel, et l'en-tête montre son nom quand ce n'est pas de l'UTF-8. Si vous choisissez l'UTF-16 pour un fichier ouvert en hexadécimal, il passe en texte. Votre choix reste après avoir modifié le fichier et être revenu.

Pour toujours utiliser un seul encodage pour les vieux textes, au lieu de la devinette, réglez `legacy_charset` sur `cp850`, `cp1252` ou `cp437` (**K** à la page 2 de la [configuration](configuration.md)). La valeur par défaut est `auto`.

## Limite de taille

La visionneuse ne lit que le début d'un gros fichier, et l'en-tête indique `(showing first N MiB)`. La quantité lue dépend de l'option `viewer_max_size` (**L** à la page 2 de la [configuration](configuration.md)) : `auto`, `16`, `64`, `256` ou `1024` (Mio). Avec `auto`, la valeur par défaut, la limite est 1/32 de la RAM de la machine, jamais moins de 16 Mio ni plus de 256 Mio (16 Mio si la RAM ne peut pas être lue). L'écran de configuration affiche la valeur utilisée, par exemple `auto (256 MiB)`.

Deux cas font exception. Autoview lit toujours 1 Mio. Un fichier sur une machine distante (SFTP) est lu jusqu'à 16 Mio au plus, car Échap ne peut pas annuler cette lecture.

Une limite plus grande coûte de la mémoire et du temps : avec `1024`, un fichier texte ordinaire de cette taille occupe environ 2,5 Gio de RAM et met environ 5 s à s'ouvrir. Les lignes très courtes coûtent plus que leurs octets, car chaque ligne a sa propre gestion, et il en va de même pour les tabulations et les caractères de contrôle de direction cachés : la visionneuse garde des lignes jusqu'à environ deux fois la limite de taille en mémoire et, au-delà, l'en-tête indique `(showing first N lines)`. Un journal de lignes ordinaires (environ 100 octets) n'est jamais coupé pour cette raison.

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
| `Tab` | la liste des feuilles ou tables : tapez une partie d'un nom pour la réduire, Entrée en ouvre une et Échap revient ; en ouvrir une autre efface le filtre. Dans une base, les vues viennent après les tables, marquées `(view)` |
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

### La structure

`i` affiche, par-dessus la grille, les colonnes dans l'ordre avec ce que le fichier dit de chacune, comme `DISPLAY STRUCTURE` dans dBase et Clipper. Les flèches, PgUp, PgDn, Home et End la font défiler ; Échap, Entrée ou `i` la ferment.

| Format | Ce qui est affiché |
|---|---|
| dBase, Clipper, FoxPro | nom, type (`C` caractère, `N` numérique, `D` date, `L` logique, `M` mémo, `F` virgule flottante, `I` entier, `B` double, `Y` monétaire, `T` date et heure), largeur et décimales ; au-dessus, le nombre d'enregistrements et la date de dernière mise à jour écrits dans l'en-tête, et la longueur d'un enregistrement |
| SQLite | le type déclaré de chaque colonne, `Not null`, la valeur par défaut et la place dans la clé primaire ; au-dessus, s'il s'agit d'une table ou d'une vue, et ses index avec leurs colonnes (32 au plus). Une vue montre les colonnes qu'elle donne ; une requête complète, celles de son résultat, avec le type de chacune quand elle vient directement d'une colonne de table |
| Parquet | le type Arrow de chaque colonne et s'il accepte les nulls ; au-dessus, les lignes et groupes de lignes que déclare le fichier et le programme qui l'a écrit |
| CSV, JSON Lines, tableurs | ils ne déclarent pas de types : ce que contiennent les lignes lues dans chaque colonne (entier, décimal, logique, texte, vide) et la valeur la plus large |

Noms, types et valeurs par défaut viennent du fichier et sont nettoyés comme les cellules : les caractères de contrôle et de direction du texte s'affichent `·`. Un `.dbf` dont l'en-tête se contredit affiche `?` pour les décimales.

### Le filtre SQL

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

### Une requête complète dans une base

Dans une base SQLite, la même invite accepte une requête complète, commençant par `SELECT` ou `WITH`, avec tout ce que SQLite sait faire : `JOIN`, `GROUP BY`, `ORDER BY`, fonctions.

```
SELECT c.name, sum(o.price) FROM customers c JOIN orders o ON o.code = c.code GROUP BY c.name
```

Le cadre indique `QUERY:`, et la grille montre le résultat. Une condition filtre alors ces lignes, et **Ctrl+B** écrit tout ce que donne la requête. Laisser l'invite vide efface le filtre, puis revient à la table ; `Tab`, `[` et `]` vont aussi à une table. Une requête ne fait que lire, une à la fois : tout le reste est refusé, et une erreur laisse l'écran tel quel. Les limites ci-dessous s'appliquent : une requête lente s'arrête à 3 s et montre les lignes obtenues ; un Batch n'a pas cette limite. Xylem ne modifie pas les bases : `UPDATE`, `DELETE` et les autres reçoivent un message qui le dit. Pour en modifier une, utilisez un autre programme : `O` dans la fenêtre des fichiers l'ouvre dans celui défini par `opener`, et `'*.db' = 'sqlite3 %f'` dans `[editors]` fait que `E` l'ouvre dans `sqlite3`.

### Limites

Un tableau contient au plus 50 000 lignes et 256 colonnes, et prend au plus ce que permettent les autres limites de la visionneuse : 3 s (100 ms dans Autoview), et un classeur qui se décompresse au-delà de la limite de taille (1 Mio dans Autoview) ne s'ouvre pas en tableau. Ce qui dépasse n'apparaît pas et le filtre ne le voit pas ; l'en-tête le signale : `(showing first 50,000 rows)` ou `(stopped by the time limit: showing N rows)`. Pendant la lecture du tableau, l'écran reste libre et une note le signale ; Échap arrête la lecture et montre ce qui est déjà arrivé, `(stopped by Esc: showing N rows)`, et l'Échap suivant ferme la visionneuse. Une cellule affiche au plus 2 048 caractères, puis `…`. Les caractères de contrôle et de direction du texte s'affichent comme `·`.

Quand un classeur (XLSX, XLSB, XLS, ODS) a plus de lignes que cela, l'en-tête et la note le disent : `(the sheet has more than 50,000 rows, the most Xylem shows: open it in Excel or another spreadsheet program to see them all (O))`, c'est-à-dire : ouvrez-la dans Excel ou un autre tableur pour les voir toutes. **O** l'ouvre dans le programme que votre système utilise pour elle.

## Tableaux lus par des bibliothèques C

Les bases SQLite (`.db`, `.sqlite`, `.sqlite3`) et les fichiers Parquet sont lus par des bibliothèques écrites en C, qui n'ont pas la sûreté mémoire de Rust. Ils ne s'ouvrent donc en tableau qu'avec `V`. Autoview lit tout ce que le curseur survole et affiche ces fichiers comme un contenu ordinaire, avec la note `Parquet or SQLite: V opens the table`. N'ouvrez une base de source non fiable que si vous acceptez ce risque. Les fichiers `.ods` et `.xls` ne s'ouvrent eux aussi en tableau qu'avec `V` : leur lecteur construit d'abord la feuille entière, et cette partie ne peut pas être interrompue.

Voir aussi : [rechercher dans les fichiers marqués](search.md).
