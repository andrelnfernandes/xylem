# La visionneuse

`V` ouvre le fichier dans la visionneuse (Autoview, F7, l'affiche à côté de la
liste). Les fichiers binaires s'ouvrent en hexadécimal. Un gros fichier sur disque n'est pas lu en entier : la visionneuse lit ce que l'écran montre à mesure qu'il avance (voir [Gros fichiers](#gros-fichiers)). Ce qui ne peut pas être lu ainsi n'affiche que son début, jusqu'à la limite de taille.
Les feuilles de calcul, CSV, DBF, SQLite et Parquet s'ouvrent en tableau (voir [Tableaux](tables.md)).

| Touche | Action |
|---|---|
| ↑ ↓ PgUp PgDn Home End | faire défiler |
| ← → | faire défiler horizontalement |
| `H` | mode suivant : texte et hexadécimal ; dans un fichier Markdown, rendu, texte et hexadécimal |
| `O` | encodage suivant (UTF-8, CP850, 1252, CP437, 1251, KOI8-R, UTF-16) |
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

**F** cherche depuis le début, **B** à rebours depuis la
fin, **/** (ou `S`) depuis la page courante et **\\** à rebours depuis celle-ci ;
**Space** poursuit dans la même direction, **+** et **-** choisissent la
direction. L'occurrence trouvée est mise en surbrillance. À l'invite de
recherche, **F2** active ou désactive la sensibilité à la casse (en ignorant la
casse, cela vaut aussi pour les lettres accentuées : `AÇÃO` trouve `ação`) et **F4**
bascule entre texte et hexadécimal (paires d'octets telles que `4d 5a`). Un `*`
au milieu du terme correspond à n'importe quelle suite de caractères sur la même
ligne (`error*disk`).

**F3** inverse le sens de la recherche, si bien que F seul fait ce que fait B : vers l'avant depuis le début ou vers l'arrière depuis la fin.

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
| `k` | copie le chemin du nœud : `.servers[2].name` en JSON, `servers.2.name` en YAML et TOML. Une clé avec une fin de ligne, un caractère de contrôle ou de direction du texte est mise entre guillemets, ces caractères échappés (`\n`, `\u202e`), pour que coller le chemin dans un shell n'en exécute jamais une ligne |

Le pied de page montre le chemin du nœud sous le curseur. Une recherche (`F`, `B`, `/`) regarde les clés et les valeurs, y compris dans les nœuds repliés, et ouvre le chemin jusqu'à ce qu'elle trouve. `T` montre le texte à la ligne du nœud, et revient ; `H` passe par l'arbre, le texte et l'hexadécimal. `&`, `g`, `#` et Ctrl+B agissent sur les lignes du texte.

- Un nœud replié dit ce qu'il contient : `{12 keys}`, `[340 items]`.
- Un nombre est montré tel que le fichier l'écrit : `12345678901234567890` reste ainsi, et `1_000` en TOML aussi.
- Une chaîne montre un saut de ligne comme `↵`.
- Une clé qui apparaît deux fois dans un même objet est montrée deux fois, marquée `dup key`, et l'en-tête les compte. Les programmes ne s'accordent pas sur celle des deux qui l'emporte, ce qui peut cacher une valeur lors d'une relecture.
- Une ancre YAML apparaît comme `&a1` et un alias comme `*a1` : un alias n'est jamais développé, donc un fichier fait pour se multiplier (un « billion laughs ») ne coûte pas plus que son texte. Une étiquette (`!Ref`) apparaît avant sa valeur. Plusieurs documents sont placés sous une racine, comme `document 1`, `document 2`...

Un fichier invalide, ou imbriqué sur plus de 512 niveaux, s'ouvre comme texte, et l'avis dit pourquoi et à quelle ligne. L'arbre est construit à partir du fichier entier : un fichier au-delà de la limite de lecture s'ouvre comme texte. L'arbre s'arrête aussi à 2 000 000 nœuds (environ 170 Mio de mémoire) ou quand sa construction prend trop de temps ; il ne contient alors que le début du fichier, et l'en-tête affiche `(tree of the start only)`. `T` montre toujours tout le texte.

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

## Livres numériques

EPUB (`.epub`, versions 2 et 3) et FictionBook (`.fb2`, et `.fb2.zip` contenant un seul livre) s'ouvrent comme un livre à lire. Les images ne sont pas affichées : chacune est une ligne, comme `[image: cover.jpg]`.

**Le livre.** Il s'ouvre sur une page de titre : le titre, l'auteur, qui l'a publié et quand, combien de chapitres et de mots il compte, et ce dont le rapport avertit, s'il y a lieu. Le texte est une colonne au milieu de la fenêtre (70 cellules par défaut), avec les paragraphes en retrait comme dans un livre imprimé et le titre de chaque chapitre centré sous un ornement. Une séparation entre scènes (`* * *`) est dessinée comme telle. Le pied de page indique le chapitre où vous êtes, où vous en êtes dans le livre, et le temps restant dans le chapitre et dans le livre.

| Touche | Action |
|---|---|
| Space, PgDn | la page suivante (PgUp la précédente) |
| `]` `[` | le chapitre suivant ou précédent |
| Tab | la table des matières : ↑↓ choisissent, des lettres réduisent la liste, Enter y va, Esc ferme ; Backspace revient ensuite là où vous étiez |
| Enter | les notes appelées sur la page, dans un cadre : ↑↓ le font défiler, toute autre touche le ferme |
| `g` | aller à une ligne, ou à une partie du livre : `40%` |
| `R` | mémoriser où le livre s'est arrêté ; `R` à nouveau oublie le livre |
| `I` | le rapport sur le fichier, et retour au livre |
| `Z` | le texte seul, sans le cadre ni le pied de page ; Esc revient |
| `/` `\` | chercher en avant ou en arrière ; `+` et `-` vont à la correspondance suivante ou précédente (ici, Space tourne la page) |
| `H` | les octets, et retour |
| Esc | fermer |

Les notes d'un livre (un `aside` EPUB de type footnote, le corps `notes` d'un FB2) ne sont pas dans le texte : chaque appel est un exposant (`¹`), et Enter montre les notes appelées sur la page. Dans le rapport, elles sont là où le livre les place.

**Mémoriser où vous vous êtes arrêté.** `R` mémorise le livre : quand vous le fermez, Xylem conserve le chapitre et la ligne où vous étiez, et la fois suivante le livre s'ouvre là, en disant `Back at chapter 7 (R forgets this book)`. Le pied de page d'un livre mémorisé commence par `[R]`. Le livre est reconnu par ce qu'il dit être (son identifiant) et sa taille, pas par son nom : un livre déplacé ou renommé est donc toujours reconnu, et un autre fichier du même nom est un autre livre. `R` à nouveau oublie le livre et supprime ce qui en était conservé. Un livre sur lequel vous n'avez jamais appuyé sur `R` ne laisse rien derrière lui.

**Le temps passé à lire** est conservé pour les livres mémorisés avec `R`, si vous l'activez (**E** à la page 5 de la [configuration](configuration.md#reading-time)). La page de titre et le rapport disent alors depuis combien de temps vous lisez le livre, en combien de sessions, jusqu'où vous êtes allé et depuis quand ; le rapport liste les sessions. Le pied de page montre le temps de cette session.

**Le rapport** (`I`) montre ce qu'est le fichier avant le texte : ce que le livre dit de lui-même (titre, auteurs, éditeur, langue, date, identifiant, sujets, série, le programme qui l'a créé), et ce qu'il contient en plus de son texte, car un livre venu de n'importe où est un fichier venu de n'importe où :

- les scripts : éléments `<script>`, `onload` et les autres attributs d'événement, liens `javascript:`, et les parties qu'un EPUB déclare scriptées. Ils ne sont jamais exécutés ; une liseuse qui exécute les scripts le ferait ;
- ce qu'une liseuse irait chercher à l'extérieur à l'ouverture du livre : images, feuilles de style, polices et cadres avec une adresse sur internet, listés. Les récupérer apprend à qui se trouve là-bas que le livre a été ouvert, et d'où. Rien n'est récupéré ici ;
- formulaires, cadres et objets incorporés, comptés, et le nombre de liens vers l'extérieur ;
- DRM : un EPUB dont les chapitres sont chiffrés dit par quoi (Adobe ADEPT, Readium LCP, Apple FairPlay) et ne peut pas être lu. Les polices obscurcies, courantes dans les livres vendus sans DRM, sont seulement comptées ;
- dans un EPUB, les parties que le paquet liste mais que le fichier n'a pas, celles qu'il a mais ne liste pas, et les adresses qui sortent du paquet ;
- dans un FB2, les fichiers incorporés (leur nombre, leurs types et leur taille), jamais décodés, et un encodage déclaré que Xylem ne connaît pas.

Viennent ensuite la table des matières, sous forme de liens (Tab et Enter les suivent), et le texte entier, avec les notes là où le livre les place.

**Réglages** (page 5 de la [configuration](configuration.md)) : la largeur de la colonne (ou la fenêtre entière), paragraphes en retrait et accolés ou séparés par une ligne vide, texte justifié, et la vitesse de lecture avec laquelle le temps restant est calculé. Avec Nerd Fonts activé (**M** à la page 2), les ornements sont des feuilles, la barre de progression est dessinée avec les glyphes de la police, et le pied de page et la table des matières reçoivent des icônes.

**Ce qui est lu.** Un FB2 est lu dans l'encodage qu'il déclare (UTF-8, UTF-16, Windows-1251, KOI8-R et les autres de `O`), et `T` montre son XML. Un EPUB est un ZIP : chaque partie n'est décompressée que jusqu'à une limite (64 Mio chacune, 256 Mio en tout ; 4 Mio dans l'Autoview, qui indique `V shows it as a book` quand un livre est plus gros), un FB2 est lu jusqu'à 64 Mio, et le rapport dit quand seul le début du livre a été lu. Aucune DTD n'est jamais lue ni récupérée, un XML avec ses propres entités est refusé, le texte n'est jamais traité comme du Markdown, et les adresses d'un livre ne sont prises que comme noms de ses propres parties, jamais comme chemins sur le disque. MOBI et AZW3 (Kindle), PDF et bandes dessinées (CBZ, CBR) ne sont pas lus ici : `O` dans la fenêtre de fichiers les ouvre dans un autre programme.

## Norton Guides

Un Norton Guide ou un guide Expert Help (`.ng`), l'aide hypertexte de DOS et de Clipper, s'ouvre comme un document. Un fichier doit avoir à la fois le nom `.ng` et la signature du guide ; tout autre fichier s'ouvre comme avant.

- **En haut :** le titre et les crédits du guide, et ce qui l'a créé.
- **Les menus :** chacun avec ses éléments, et où mène chaque élément.
- **Les entrées :** toutes, dans l'ordre du fichier, chacune sous un numéro : `§12`. Une entrée de liste montre ses lignes ; une entrée de texte montre son texte avec la mise en page du guide, puis ses liens « voir aussi ». Chaque lien dit où il mène (`→ §12`, `See also: Two() §3`), donc une recherche de `§12` vous y amène. Un lien qui ne mène nulle part indique `(broken link)`.

**Suivre les liens.** **Tab** et **Shift+Tab** choisissent le lien suivant ou précédent (à partir de l'écran, quand celui choisi en est sorti par défilement) ; **Enter** va à l'entrée vers laquelle il mène, avec le premier lien de cette entrée choisi ; **Backspace** revient là où vous étiez, sur jusqu'à 1 000 liens. Avec la souris activée, un clic sur un lien le suit, et un clic sur `Backspace` dans le pied de page revient en arrière. Dans un guide, Enter ne ferme jamais la visionneuse : c'est Esc qui le fait. Comme le guide entier est un seul document, une recherche (`/`) le parcourt toujours en entier.

Le gras, le souligné et l'inverse sortent dans les styles du thème ; les couleurs DOS d'un guide ne sont pas envoyées au terminal. Les caractères du guide sont en page de code 437, avec les glyphes que DOS dessinait pour les octets 1 à 31 (`☺ ♥ ► ▲`) ; rien dans un guide n'atteint le terminal comme caractère de contrôle.

Un guide est un réseau de décalages qui pourraient mentir : chacun est vérifié par rapport au fichier, chaque compte par rapport à son propre enregistrement, et les espaces auxquels une ligne s'étend ont une limite. Ce qui ne va pas dans un guide est listé en haut, sous « Problems in this guide », et le reste s'affiche quand même. Un guide est lu en entier, jusqu'à 64 Mio ; `H` montre les octets.

Le format est connu grâce à Dave Pearson, qui l'a décrypté dans les années 1990 et garde les Norton Guides lisibles depuis : <https://www.davep.org/norton-guides/>. Le lecteur de Xylem est son propre code, écrit à partir de sa description du format ; il n'inclut aucun code de ses lecteurs.

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
- un chemin absolu, qui remonte avec `..`, qui entre dans `.git` (un hook est du code que Git exécute), ou qui contient des caractères de contrôle, de direction du texte ou invisibles (un nom comme `invoice‮fdp.exe` cache sa vraie extension).

L'en-tête compte aussi :

- `look unchanged` : une ligne retirée et une ligne ajoutée qui ne diffèrent que par ce qui ne se voit pas, comme des espaces, `\r`, des caractères invisibles ou des lettres qui se ressemblent (un `а` cyrillique pour un `a` latin). La paire est montrée en jaune ;
- `hunks with wrong counts` : un hunk dont la ligne `@@` ne correspond pas à ses lignes. L'index suit les lignes qui sont là, jamais les nombres ;
- `whitespace` : des lignes ajoutées avec une espace en fin de ligne, ou une espace avant une tabulation dans l'indentation, comme le signale `git diff --check`.

Une série de `git format-patch` montre chaque message dans la liste, par son sujet. Rien n'est jamais appliqué. Dans un fichier de plus de 16 Mio, lu à travers une fenêtre, les couleurs restent, mais il n'y a pas d'index : `]`, `}` et Tab le disent.

## Caractères de contrôle de direction

Certains caractères invisibles changent le sens d'écriture du texte et peuvent faire paraître un code différent de ce qu'il dit vraiment (Trojan Source, CVE-2021-42574). La visionneuse ne les cache jamais : chacun apparaît sous la forme d'un `·` rouge, et l'en-tête indique combien il y en a.

## Encodage du texte

Xylem trouve l'encodage seul, dans cet ordre : d'abord il décompresse le fichier, puis cherche un BOM (UTF-8, UTF-16 LE ou BE), puis de l'UTF-16 sans BOM, puis de l'UTF-8. Un fichier qui n'est rien de tout cela est un vieux texte : Xylem choisit CP850 (DOS, Europe occidentale) ou Windows-1252 en comptant les lettres accentuées. Les exports `.reg` de Windows sont en UTF-16 et s'ouvrent comme du texte normal.

**O** affiche le fichier dans l'encodage suivant : UTF-8, CP850, Windows-1252, CP437, Windows-1251, KOI8-R, UTF-16 LE, UTF-16 BE, puis de nouveau UTF-8. Le pied de page dit lequel, et l'en-tête montre son nom quand ce n'est pas de l'UTF-8. Si vous choisissez l'UTF-16 pour un fichier ouvert en hexadécimal, il passe en texte. Votre choix reste après avoir modifié le fichier et être revenu.

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
