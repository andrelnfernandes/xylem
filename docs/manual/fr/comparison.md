# Comparaisons

## Un remerciement

J'utilise une variante de XTree depuis les années 1990, et j'ai une licence
payée de ZTreeWin depuis 2013. Pour moi, ZTreeWin reste le meilleur
gestionnaire de fichiers en mode texte qui existe : rapide, prévisible,
entièrement piloté au clavier, et construit autour d'une manière de travailler
(lire l'arborescence, marquer des fichiers dans des branches entières, agir sur
les fichiers marqués) qu'aucun autre gestionnaire de fichiers n'a égalée. Kim
Henkel fait vivre la manière XTree sous Windows depuis des décennies, et Xylem
n'existerait pas sans ce travail.

Xylem n'a pas vocation à remplacer ZTreeWin. Il est né parce que je voulais la
même manière de travailler sous Linux, où ZTreeWin ne tournait pas. Depuis la
[bêta 2.9.12](https://www.ztree.com/html/beta.htm) de novembre 2026, il y tourne, sous le nom de ZTreeLin (et
sous macOS, ZTreeMac). Xylem continue comme une autre lecture de la même
manière de travailler : libre, fait d'abord pour Linux, avec ce qui est dans
[Là où Xylem va plus loin](#where-xylem-goes-further), plus bas.
Xylem se compile aussi pour Windows, parce que Rust le rend facile, mais ce
n'est pas un objectif pour moi : sous Windows, ZTreeWin vaut largement sa
licence : [ztree.com](https://www.ztree.com/), et le
[forum](https://groups.io/g/ZTreeWin) de ses utilisateurs.

La visionneuse lit les Norton Guides, les fichiers d'aide de DOS et de Clipper,
grâce à Dave Pearson, qui a décrypté leur format dans les années 1990 et les
garde lisibles depuis :
[davep.org/norton-guides](https://www.davep.org/norton-guides/).

## Autres gestionnaires de fichiers pour le terminal

Il existe beaucoup de bons gestionnaires de fichiers pour le terminal, et la
plupart des personnes qui essaient Xylem en utilisent déjà un. Voici ce qui
les distingue, aussi honnêtement que je peux le dire. Vérifié dans la
documentation de chaque projet en novembre 2026 ; ces programmes évoluent
vite, donc si quelque chose ici n'est plus à jour, ouvrez une issue.

| Programme | Disposition | Connu pour |
|---|---|---|
| ranger | trois colonnes (parent, courant, aperçu) | touches vi, aperçus par des scripts, extensible en Python |
| lf | trois colonnes, comme ranger | un seul petit binaire, rapide, des commandes shell comme extensions |
| yazi | trois colonnes | la vitesse (tout est asynchrone), l'aperçu des images, des plugins Lua avec un gestionnaire de paquets, SFTP intégré |
| superfile | plusieurs panneaux côte à côte | un look moderne, l'aperçu des images, des thèmes |
| nnn | une liste | minuscule et rapide, une sélection entre répertoires, un mode d'occupation du disque, de nombreux plugins |
| Midnight Commander (mc) | deux panneaux | le classique : son propre visualiseur et éditeur, SFTP et archives comme dossiers, les résultats de Find file dans un panneau |
| vifm | deux panneaux | les touches et commandes de vim, les résultats de `:find` comme liste de travail, une vue en arborescence |
| broot | une arborescence qui tient dans l'écran | recherche floue dans toute l'arborescence, tailles, une zone de préparation (staging) pour agir sur des fichiers de nombreux dossiers |
| far2l | deux panneaux | Far Manager pour Linux, avec ses plugins : SFTP, SMB, NFS, WebDAV et S3 (NetRocks), un éditeur hexadécimal, des scripts Python |
| ZTreeLin | une arborescence au-dessus de la liste des fichiers, écran partagé | la manière XTree de ZTreeWin sous Linux, par ses auteurs : lire l'arborescence, marquer dans des branches entières, agir sur les fichiers marqués ; un éditeur hexadécimal et les archives ; en bêta depuis novembre 2026 |

**Ce qui distingue Xylem.** L'arborescence est lue en mémoire, et une touche
affiche dans une seule liste les fichiers de toute une branche, ou de tout ce
qui a été lu (Branch, Showall), restreints par nom, date et taille (Filespec)
ou par une condition SQL (`W`), avec les octets correspondants et ceux marqués
comptés au fur et à mesure. Marquez-y ce que vous voulez et copiez, déplacez,
renommez ou supprimez le tout en une fois. Plusieurs des programmes ci-dessus
peuvent aussi rassembler dans une liste des fichiers de nombreux dossiers (le
panelize de mc, les vues personnalisées de vifm, les résultats de recherche de
yazi, la zone de préparation de broot), ou garder une sélection entre dossiers
(nnn, yazi) ; dans Xylem, comme dans ZTreeLin, cette liste est la manière de travailler en
permanence, et non le résultat d'une recherche. En plus de cela, aucun des
autres ne réunit dans un même programme :

- des tableaux avec filtre SQL pour les feuilles de calcul, CSV, SQLite (avec
  `JOIN`) et Parquet, et leurs enregistrements transformés en script, en page
  ou en base de données (Batch). Un plugin de yazi basé sur DuckDB affiche un
  aperçu et un résumé des fichiers CSV et Parquet, sans filtre, requête ni
  export ;
- des rapports sur les fichiers auxquels vous ne faites pas confiance : les
  macros, modèles distants et textes cachés des documents Office, les
  protections et les signes d'un packer dans les exécutables, les noms à
  vérifier dans un patch, sans rien exécuter de leur contenu ;
- les fichiers et dossiers en double, même sur Google Drive sans les
  télécharger ;
- un terminal intégré dont l'arborescence suit le `cd` du shell ;
- les fichiers venus de l'extérieur traités partout comme hostiles, avec une
  limite sur tout ce qui est lu ([Sécurité](security.md)).

**Là où les autres sont en avance.** L'aperçu des images (yazi, superfile,
ranger). Un système de plugins et les plugins déjà écrits pour lui (yazi, nnn,
ranger, lf, far2l) : Xylem n'en a pas, seulement eXecute et Batch. Les touches
vi dès le départ (ranger, lf, yazi, vifm), là où Xylem a ses propres commandes
d'une lettre. Des années d'usage par beaucoup de monde, et des versions macOS
et Windows testées, là où Xylem est nouveau, non testé sur Mac et
expérimental sous Windows.

## ZTreeWin

Cette partie compare les deux commande par commande, à partir du manuel officiel
de ZTreeWin v2.2 ([ztwhlp.pdf](https://www.zedtek.com/download/ztwhlp.pdf), de
2011, en particulier la « Quick Command Table » et la section 3.7 sur la
configuration). Les versions plus récentes de ZTreeWin peuvent avoir modifié
certains détails ; si quelque chose ici n'est plus à jour, ouvrez une issue.

### La même manière de travailler

Arborescence avec lecture à la demande (`+`, `*`), fenêtre de fichiers,
Filespec, Branch, Showall (aussi des seuls fichiers marqués, Ctrl+S), marquage
(T, U, I, Ctrl+T/U), Copier/Déplacer/Supprimer/Renommer avec les variantes Ctrl
(fichiers marqués) et Alt (fichiers marqués avec chemins), Créer un répertoire,
Supprimer un répertoire, Prune, Attributs, Visionneuse, Éditer, eXécuter,
Comparer (répertoire et branche), Trier, Log, écran partagé (F8 et Tab),
relecture (F3), Ouvrir (`O`), rafraîchissement automatique, navigation et
extraction d'archives, historique dans les invites, signets, aide (F1) et écran
de configuration. Les touches sont celles de ZTreeWin chaque fois que possible ;
les exceptions sont dans « Touches différentes », à la fin.

### Ce que Xylem n'a pas encore

- **Arborescence :** Global (G, et Ctrl+G pour les marqués), Graft (Alt+G), Mirror (Alt+M), libération partielle (Ctrl+Backspace), Hide (Alt+H), Wash (Alt+W), Volume
  (Ctrl+V), F5/F6 (masquer des niveaux de sous-répertoires), marquer par
  attribut (Alt+T/U), Treespec en mode édition (F5, jokers) et dossiers spéciaux
  (F6), recherche directe en mode actif (sans invite), changement de lecteur
  (`<` et `>`). La recherche directe avec invite (`|`) existe, avec une
  correspondance approximative et aussi dans la fenêtre de fichiers :
  [Recherche approximative](fuzzy.md).
- **Fichiers :** Ouvrir dans une nouvelle fenêtre de terminal (Alt+O), Batch du
  fichier courant (Alt+B), JFC (J, comparer le contenu de deux fichiers),
  sHortcut (H), Print (P), propriétaire/groupe (chown), Megaview (Ctrl+F7 dans
  la fenêtre de fichiers), fusionner les fichiers marqués entre fenêtres
  (Ctrl+F6), Partial Untag (Ctrl+F8), Tags-Only hors de Showall (Ctrl+Enter
  et Ctrl+F4, Ctrl+B dans l'arborescence).
- **Général :** macros (F12), menu d'applications (F9), Quitter vers un shell
  (Ctrl+Q), enregistrement et chargement du log (Zlist, Ctrl/Alt+Z), couleurs
  par nom et attribut (ZCOLORS.INI), options de ligne de commande de
  ZTreeWin, commandes vidéo (Alt+F6–F9).

Pour certaines, un autre programme fait l'affaire en attendant ; voir « Ce que
Xylem laisse à d'autres programmes », plus bas.

### Là où Xylem est plus simple

| Fonction | Dans ZTreeWin | Dans Xylem |
|---|---|---|
| Filespec | nom avec règles propres au nom et à l'extension, date, taille, attribut, filespec inversé (Ctrl+I), étiquettes | glob pour les noms, date et taille comme dans ZTreeWin (avec K/M/G) ; ni attributs, ni inversion, ni étiquettes |
| Visionneuse | modes Alpha, Dump, Hex, Junk et Wrap ; jeux de caractères ; édition hexadécimale ; signets ; défilement automatique | texte, hexadécimal, tableaux avec filtre SQL et Markdown rendu, avec la recherche de ZTreeWin (F, B, /, \\, Space, + et -), retour à la ligne (W), numéros de ligne (#), aller à la ligne (g / :), filtre de lignes par regex (&) ; un fichier sur disque de n'importe quelle taille, lu à mesure que l'écran avance ; pas d'édition hexadécimale, par choix (voir « Ce que Xylem laisse à d'autres programmes ») |
| Ctrl+S | texte, hexadécimal ou Unicode ; ne cherche que dans l'index des ZIP | texte ou hexadécimal |
| Attributs | attributs Windows (R, A, S, H) | permissions Unix en octal (`644`) ; pas de propriétaire/groupe. Sous Windows, R, H, S et A, comme dans ZTreeWin |
| Tri | Name, Ext, Date, Time, Length, Alpha, nuMber, Size, Value, Unsorted ; Charset ; same/diff en écran partagé | Name, Ext, Date, Size, croissant/décroissant, Path en premier ; chaque fenêtre a son propre ordre |
| Comparer | options Identical, Unique, Newer, Older, Size, Binary, Case ; filtre d'affichage | marque unique, plus récent, plus ancien, identique, différent ou unique+plus récent ; pas de comparaison binaire |
| Prune | supprime seulement les sous-répertoires ou aussi le répertoire courant ; répertoires vides uniquement ; corbeille | supprime toujours tout le répertoire, définitivement ; confirmation configurable |
| Log | plusieurs lecteurs lus, Global, changement de lecteur | une racine par fenêtre ; `L` ouvre un autre répertoire |
| eXécuter | paramètres de batch `%1`–`%8`, nouvelle fenêtre (NTW) | `sh` avec `%f`, `%t` et `%d` ; toujours dans le terminal de Xylem |
| Historique | fenêtre d'historique avec signets | ↑ à l'invite ouvre une liste (taper filtre, Del oublie), un historique par type, conservé entre les sessions |
| Configuration | 7 pages, éditeur de couleurs par élément (F4), Alt-Export | 4 pages avec les options pertinentes sous Linux ; F4 choisit parmi 12 thèmes prêts à l'emploi ; couleurs individuelles dans `config.toml` |
| Écran partagé | panneaux de statistiques optionnels de chaque côté | pas de panneau de statistiques |
| Menus | menu Ctrl/Alt en maintenant la touche | idem dans les terminaux avec le protocole kitty ; F4 dans tous les terminaux |

Options de configuration de ZTreeWin qui n'ont pas été reprises parce qu'elles
sont propres à Windows : attributs RSH, jonctions, OEM/ANSI, EBCDIC, taille de
fenêtre, raccourcis (.lnk).

### Là où Xylem va plus loin

Ce qui n'a pas d'équivalent dans le manuel de ZTreeWin v2.2 :

- **Un terminal intégré** (Alt+X) : le shell continue de tourner pendant que
  vous utilisez l'arborescence ; un `cd` dans le shell emmène l'arborescence,
  et le shell suit l'arborescence quand vous y revenez. Alt+O tape les fichiers
  marqués à l'invite. Voir [le terminal intégré](terminal.md).
- **Machines distantes via SSH** (`xy utilisateur@hote:`) : l'arborescence,
  Branch, la visionneuse et Copier fonctionnent sur une autre machine, qui a
  aussi son propre terminal. Voir [machines distantes](remote.md).
- **Stockage cloud :** Google Drive, S3 et les autres services pris en charge par rclone, montés et affichés comme une arborescence. Voir [stockage cloud](cloud.md).
- **SQL sur la liste de fichiers** (`W`) : `ext = 'jpg' AND size > 10000000` garde les fichiers qui remplissent la condition, et `contains()` et `words()` regardent à l'intérieur ; la boîte de recherche (`P`) trouve les fichiers par les mots qu'ils contiennent. Voir [trouver des fichiers](sql.md).
- **Git** (optionnel) : lettres d'état dans la liste des fichiers, un menu Git
  (Ctrl+G) et des diffs dans la visionneuse. Voir [Git](git.md).
- **Utilisation du disque** (Alt+U) : les dossiers et fichiers d'une branche
  ensemble, les plus gros d'abord, jusqu'à ce qui remplit le disque ; Alt+K
  montre dans l'arborescence la part de chaque dossier. Voir
  [utilisation du disque](usage.md).
- **Doublons** (Alt+D) : fichiers et dossiers entiers au même contenu, comparés
  par SHA-256 ou, dans un dossier monté par rclone, par les empreintes du service,
  sans rien télécharger. Une copie de chaque groupe reste toujours. Voir
  [doublons](duplicates.md).
- **Une visionneuse qui lit davantage :** coloration syntaxique, encodage du
  texte détecté seul (UTF-8, UTF-16, CP850, Windows-1252, CP437), fichiers
  `.gz`, `.xz`, `.zst` et `.bz2` affichés décompressés, et caractères cachés de
  direction du texte (Trojan Source) affichés au lieu d'être obéis. Voir
  [la visionneuse](viewer.md).
- **Tableaux et Markdown :** tableurs, CSV, JSON Lines, DBF, SQLite et Parquet
  s'ouvrent dans une grille avec filtre SQL, et les fichiers Markdown s'ouvrent
  rendus. Voir [tableaux](tables.md).
- **Des archives sans archiveur externe :** ZIP, 7z et TAR (brut, gzip, xz,
  bzip2, zstd) s'ouvrent comme des dossiers, et ZIP et TAR se créent, le tout
  par Xylem lui-même. Voir [archives](archives.md).
- **Renommage par lot** avec rechercher-remplacer (`s/foo/bar/`), compteurs et
  changements de casse. Voir [renommer](rename.md).
- **Batch sur les enregistrements :** Ctrl+B dans la visionneuse transforme les
  enregistrements d'un tableau ou les lignes d'un texte en script, JSON, CSV, XML,
  HTML, SQL ou base SQLite, avec des fichiers modèles si vous le souhaitez. Voir
  [Batch](execute.md).
- **Oops :** O (ou Ctrl+Z) remet à sa place ce qui a été envoyé en dernier
  dans la corbeille du système. Voir [supprimer](delete.md).
- **Le presse-papiers partout :** Y copie le chemin même via SSH ou dans tmux
  (OSC 52).
- **Ce qu'est chaque fichier :** un mode d'affichage (Alt+F) qui dit, dans
  l'esprit de la commande `file`, ce que contient chaque fichier, et un autre
  avec propriétaires et permissions.
- **Apparence :** 12 thèmes prêts à l'emploi et icônes Nerd Fonts en option.
  Toute touche peut être redéfinie sous `[keys]` dans `config.toml`. Voir
  [configuration](configuration.md).
- **Fichiers hostiles :** archives, noms et contenus sont traités comme
  potentiellement hostiles, avec des limites sur tout ce qui est lu et
  décompressé (voir la déclaration de sécurité dans le README).
- **Une aide en six langues** (F1, puis L), le même texte que ce manuel et la
  page de manuel.

### Ce que Xylem laisse à d'autres programmes

Certaines choses, Xylem ne les fait pas lui-même, exprès : un programme dédié
les fait mieux, et Xylem l'intègre.

- **Édition hexadécimale :** ZTreeWin édite les octets dans sa visionneuse.
  Xylem n'a pas d'éditeur hexadécimal à lui : en écrire et en maintenir un
  demanderait beaucoup de travail pour peu d'usage, et modifier des octets à la
  main est un moyen facile de corrompre un fichier. À la place, E sur un
  fichier binaire ouvre `hexedit`, `heh`, `bvi` ou `dhex` (celui qui est
  installé, ou celui que vous choisissez) dans le cadre et les couleurs de
  Xylem, ou un éditeur graphique comme ImHex ou GHex. Voir [Éditer](execute.md).
- **Éditer du texte :** votre éditeur (`$VISUAL`, `$EDITOR` ou celui défini
  dans F12), et un éditeur différent par type de fichier si vous le souhaitez
  (`[editors]`).
- **SSH :** le `ssh` du système, donc vos clés, l'agent, `~/.ssh/config` et les
  *jump hosts* fonctionnent comme partout ailleurs.
- **Stockage cloud :** Google Drive, S3 et d'autres via `rclone mount` ;
  Xylem reconnaît le montage et s'adapte. Voir [stockage cloud](cloud.md).
- **Git :** le programme `git` lui-même.
- **Ouvrir des fichiers** (O) : le programme du bureau pour chaque type
  (`xdg-open`).
- **Comparer le contenu de deux fichiers** (JFC dans ZTreeWin) : tant que ce
  n'est pas intégré, marquez les deux fichiers et lancez `diff %t` avec
  eXécuter (X), ou utilisez le terminal.
- **Imprimer** (P dans ZTreeWin) : `lp %t` avec eXécuter.
- **Autres formats d'archive** (le ZAAP de ZTreeWin) : votre archiveur, avec
  eXécuter ou dans le terminal.

### Touches différentes

Les touches suivent celles de ZTreeWin. Les exceptions :

| Fonction | ZTreeWin | Xylem |
|---|---|---|
| Aide | F1 | F1 ou Alt+H (dans GNOME Terminal, F1 ouvre l'aide du terminal) |
| Configuration | Alt+F10 | F12 ou Alt+F10 (sous GNOME, Alt+F10 maximise la fenêtre et n'atteint jamais Xylem) |
| Signets | F11 | F11, Alt+B ou apostrophe `'` (dans GNOME Terminal, F11 bascule le plein écran) |
| Déplacer les fichiers marqués | Ctrl+M | Ctrl+M dans les terminaux avec le protocole kitty ; dans les autres, `F4` `M` (ou Alt+M puis F6) |
| F2 | (rien) | relit le répertoire, comme F3 |
| F9 | menu d'applications (non implémenté) | comme F4 (les menus Ctrl/Alt), jusqu'à ce qu'il reçoive une commande à lui |
| F12 | macros (non implémenté) | configuration |
| Alt+B | Batch du fichier courant (non implémenté) | signets |
| Alt+H | Hide (non implémenté) | aide |
| Alt+O | Ouvrir dans une nouvelle fenêtre de terminal (non implémenté) | tape les fichiers marqués dans le terminal intégré |
| Alt+D | (rien) | Doublons |
| Alt+T | marquer par attribut (non implémenté) | (rien) |
| Alt+U | démarquer par attribut (non implémenté) | utilisation du disque |
| Ctrl+G | Global des fichiers marqués (non implémenté) | le menu Git (optionnel) |
| Alt+Z | Zlist (non implémenté) | le terminal intégré en plein écran |
| `/` | identique à `?` (statistiques) | Treespec, comme `\` (`?` affiche toujours les statistiques) |
| Ctrl+C, Ctrl+M dans l'arborescence | (rien) | Copier/Déplacer les fichiers marqués du répertoire courant |
| Copier le chemin dans le presse-papiers | Ctrl+Insert | Y ou Ctrl+Insert (Ctrl+Y pour les fichiers marqués ; fonctionne via OSC 52 et le presse-papiers système) |

Toutes les touches peuvent être redéfinies sous `[keys]` dans `config.toml`.

## Noms

Xylem est un projet indépendant, écrit de zéro. Il n'est ni affilié, ni
approuvé, ni lié à Zedtek, Inc. (ZTreeWin), aux propriétaires de XTree ou aux
auteurs des autres programmes cités sur cette page, et il ne contient aucun
code provenant d'eux. Leurs noms ne sont cités que pour décrire ce qui a
inspiré Xylem et la façon dont il se compare. XTree, ZTree et ZTreeWin sont
des marques de leurs propriétaires respectifs.
