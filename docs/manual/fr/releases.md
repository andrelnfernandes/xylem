# Notes de version

La version la plus récente en premier. Chaque section détaille les nouveautés,
les changements pour les utilisateurs de la version précédente et les touches
modifiées. Les notes complètes de chaque version sont disponibles sur
`https://github.com/andrelnfernandes/xylem/releases`.

## 0.9.0

**Nouveautés.**

- Les Norton Guides (`.ng`), les fichiers d'aide de DOS et de Clipper,
  s'ouvrent dans la visionneuse comme un document : les menus, puis chaque
  entrée numérotée (`§12`), avec ses liens et ses « voir aussi » renvoyant à
  ces numéros. Tab choisit un lien, Enter ou un clic le suit, Backspace revient
  en arrière. [La visionneuse](viewer.md)
- Recherche approximative (`|`) : tapez des morceaux d'un nom (`cfgxy` trouve
  `config/xylem`) et le curseur va au meilleur dossier, dans l'arborescence, ou
  fichier de la liste, dans la fenêtre de fichiers. Les résultats s'affichent à
  mesure que vous tapez, le meilleur en premier, même dans une arborescence de
  millions de dossiers. Ctrl+T marque le résultat choisi et Alt+T la liste.
  [Recherche approximative](fuzzy.md)
- [Trouver des fichiers et des dossiers](finding.md) : une page qui dit quelle
  recherche répond à quelle question.
- Livres numériques : EPUB et FictionBook (`.fb2`, `.fb2.zip`) s'ouvrent comme
  un livre à lire, dans une colonne aux paragraphes de livre imprimé : une page
  de titre, Space pour la page suivante, `]` `[` pour les chapitres, Tab pour la
  table des matières, Enter pour les notes appelées sur la page, `g 40%` pour
  une partie du livre, et le temps restant du chapitre et du livre. `I` montre
  le rapport sur le fichier : ce que le livre dit de lui-même, et s'il contient
  des scripts, des choses récupérées à l'extérieur ou des DRM. [La
  visionneuse](viewer.md#ebooks)
- `R` dans un livre mémorise où vous vous êtes arrêté, et le livre s'ouvre là
  la fois suivante. Si vous l'activez (page 5 de F12), Xylem conserve aussi le
  temps passé à lire chaque livre mémorisé, vos sessions et votre vitesse de
  lecture, uniquement sur votre ordinateur. [Temps de
  lecture](configuration.md#reading-time)
- F1 ouvre l'aide à la section de ce que vous faites : dans le Batch de la
  visionneuse (Ctrl+B), à « Batch dans la visionneuse », pas en haut de la page.
  Les liens vers une section d'une page de l'aide y mènent désormais.
- Xylem peut savoir qu'une nouvelle version est sortie. Au premier démarrage,
  il demande s'il peut chercher une fois par jour ; avec un oui, il ne récupère
  du site web que le numéro de la dernière version, et la ligne du haut indique
  quand une plus récente est sortie. Rien n'est téléchargé ni installé.
  [Nouvelles versions](configuration.md#new-versions)
- Vues (Alt+V) : d'autres façons de regarder la liste. Le Scan (`S`) montre ce
  qui, dans un dossier, mérite un coup d'œil avant de l'ouvrir : noms avec des
  caractères cachés ou une double extension (`fatura.pdf.exe`), fichiers setuid
  et ce que n'importe qui peut écrire, liens qui sortent du dossier ou ne
  mènent nulle part, et, lus avec les lecteurs et les limites de la visionneuse,
  exécutables sans PIE ni NX ou compressés, documents Office avec macros ou
  modèle distant, patchs qui sortent avec `..`. Enter sur une catégorie ramène
  ses fichiers dans la liste, pour Ctrl+T et le reste. La Timeline (`T`) montre
  les fichiers de la liste dans un histogramme par date, par heures, jours,
  semaines, mois ou années ; Enter sur une barre, ou une plage, ne garde que ses
  fichiers, comme une condition `W`. La Table (`L`), triée par n'importe quelle colonne avec `s`, ouvre la liste dans la
  visionneuse de tableaux, un fichier par ligne avec les colonnes de `W`, même
  une liste de millions de fichiers ; `W` y renvoie son filtre SQL à la liste.
  [Vues](views.md)

**Changements.**

- `O` dans la visionneuse passe par deux encodages de plus, Windows-1251 et
  KOI8-R (cyrillique), avant l'UTF-16.
- L'affichage Type (Alt+F) indique `EPUB ebook (ZIP)`, `FB2 ebook` et
  `FB2 ebook (ZIP)`.

**Corrections.**

- Avec Git activé, un grand dépôt sur un système de fichiers lent (`/mnt/c`
  depuis WSL, un partage réseau) figeait l'écran pendant que `git status`
  tournait, pendant des minutes. L'état est désormais lu en arrière-plan, et un
  dépôt lent est relu moins souvent. [Git](git.md)
- La bibliothèque que Xylem utilise pour les appels système (rustix 1.1.6) est
  mise à jour pour un avis concernant les attributs étendus
  ([RUSTSEC-2026-0486](https://rustsec.org/advisories/RUSTSEC-2026-0486)).

## 0.8.0

**Changement : certaines touches Alt.** Alt ne répète plus Ctrl. Il devient
l'autre version d'une commande.

| Touche | Jusqu'à 0.7 | À partir de 0.8 |
|---|---|---|
| Alt+A | Attributes des fichiers marqués | rien (Ctrl+A) |
| Alt+D | supprime les fichiers marqués | Duplicates (Ctrl+D les supprime) |
| Alt+R | renomme les fichiers marqués | Release de la branche, comme `-` (Ctrl+R les renomme) |
| Alt+T | marque tout | rien (Ctrl+T) |
| Alt+U | démarque tout | Usage (Ctrl+U démarque) |
| Alt+W | Usage | rien (Alt+U) |
| Alt+Y | Duplicates | rien (Alt+D) |

Pour conserver les anciennes touches, ajoutez-les dans `config.toml`, une à une
ou toutes :

```toml
[keys.any]
"alt+a" = "attributes_tagged"
"alt+d" = "delete_tagged"
"alt+r" = "rename_tagged"
"alt+t" = "tag_all"
"alt+u" = "untag_all"
"alt+w" = "usage"
"alt+y" = "duplicates"
```

**Changement : le pied de page.** Chaque commande fait désormais partie d'un
menu et le pied de page comporte trois lignes, ou deux sur un écran assez large
pour afficher chaque menu sur une seule ligne. Sur un écran étroit, les mots
sont raccourcis avant qu'une commande ne soit omise. Dans une archive ou sur une
machine distante, les menus ne listent que ce qui y fonctionne.
[Les menus](menus.md)

**Changement : invites.** ↑ ouvre l'historique d'une invite sous forme de liste ;
↓ ne fait plus défiler les entrées une par une. `B` dans un Branch ou Showall
revient à la liste propre du répertoire. La fenêtre de fichiers réduite occupe un
quart de la hauteur de l'écran.

**Nouveautés.**

- `W` conserve dans la liste les fichiers répondant à une condition SQL ; Ctrl+W
  filtre parmi les fichiers marqués. `contains('texte')` lit le contenu des
  fichiers et `words('texte')` consulte l'index construit avec Alt+N.
  [Une condition SQL](sql.md#une-condition-sql-w)
- `P` ouvre une boîte de recherche listant les fichiers selon les mots saisis.
  [La boîte de recherche](sql.md#la-boîte-de-recherche-p)
- Les tableaux CSV, TSV, JSON Lines, DBF et SQLite n'ont plus de limite de
  lignes dans la visionneuse. Les feuilles de calcul conservent la limite de
  50 000 lignes. [Tableaux](tables.md)
- Dans un tableau, Enter montre l'enregistrement en fiche, et `c` le copie ;
  `c` copie aussi la structure affichée par `i`. [Tableaux](tables.md)
- Un texte de plus de 16 Mio est lu à travers une fenêtre, sans limite de
  taille. [Gros fichiers](viewer.md#gros-fichiers)
- Nouvelles visionneuses : JSON, YAML et TOML en arbre repliable ; journaux
  aux couleurs de leurs niveaux, et suivi ; patchs avec un index des fichiers
  et des hunks et ce qu'il faut vérifier ; notebooks Jupyter ; documents
  Office, avec leurs macros, modèles distants et texte masqué ; exécutables
  (ELF, PE, Mach-O) et certificats et clés sous forme de rapports.
  [La visionneuse](viewer.md)
- Il se compile sur macOS (pas encore testé sur un Mac).
- Alt+L fait du répertoire sous le curseur la racine de l'arborescence.
- Dans Treespec, Ctrl+← retire le dernier dossier du chemin et Ctrl+→ le
  restaure.
- `clock_seconds` affiche les secondes sur l'horloge de l'en-tête.
- La souris, désactivée par défaut (F12, page 2, R) : la molette, les clics et
  doubles clics sur l'arborescence, les listes et les touches du bas de
  l'écran, le marquage avec le bouton du milieu, un menu sur le bouton droit
  (aussi la touche Menu et Shift+F10), et la souris transmise aux programmes du
  terminal intégré et de l'éditeur dans le cadre.
  [La souris](mouse.md)
- Ce manuel : les pages sur l'écran, les menus, les commandes de chaque menu, le
  vocabulaire et les recettes pas à pas.

## 0.7.0

**Changement : lecture (Logging).** Un répertoire n'est lu que sur demande
explicite. Déplacer le curseur sur un répertoire non lu affiche `Not logged` ;
`+` ou → le lit. Auparavant, passer dessus le lisait automatiquement.
[La fenêtre d'arborescence](tree.md)

**Changement : invites.** Log, ainsi que Copy et Move sans écran partagé,
s'ouvrent vides. ↑ liste les emplacements utilisés auparavant.

**Nouveautés.**

- Les machines distantes ne sont plus en lecture seule : Copy vers elles, Move,
  Rename, Makedir, Delete, Prune, Attributes, New date et Duplicates y
  fonctionnent. [Machines distantes (SSH)](remote.md)
- `docker://container/path` ouvre un conteneur via `docker exec`.
- Les signets peuvent être nommés et un signet sur machine distante conserve son
  adresse de connexion. F11 dans les invites Log, Copy et Move insère un signet
  dans l'invite. [Signets](bookmarks.md)
- Les dossiers montés par rclone et les partages réseau sont lus en arrière-plan ;
  l'arborescence s'affiche immédiatement.
  [Stockage cloud (rclone)](cloud.md)
- La vue Utilisation permet de marquer, supprimer, élaguer (Prune) et visualiser
  directement depuis son interface.
- Écran de démarrage, nom et version dans l'en-tête, noms de fichiers colorés
  selon le type et bordures arrondies. [Configuration](configuration.md)

## 0.6.1

Mise à jour du lecteur de feuilles de calcul (calamine 0.38) suite à l'avis de
sécurité RUSTSEC-2026-0436.

## 0.6.0

**Nouveautés.**

- La visionneuse affiche feuilles de calcul, CSV, JSON Lines, DBF, SQLite et
  Parquet comme tableaux avec filtre SQL. Une base SQLite accepte en plus une
  requête complète en lecture seule. [La visionneuse](viewer.md)
- Ctrl+B dans la visionneuse écrit un fichier depuis les enregistrements d'un
  tableau ou les lignes d'un texte : JSON, CSV, XML, HTML, gabarit ou table
  SQLite. [Batch](execute.md)
- Duplicates trouve les fichiers et dossiers identiques, sur disque local et via
  les empreintes fournies par rclone. La touche était Alt+Y ; elle devient Alt+D
  dès la 0.8. [Doublons](duplicates.md)
- Les fichiers Markdown s'ouvrent rendus dans la visionneuse. `T` affiche la
  source.
- La visionneuse filtre les lignes par expression régulière (`&`).
  [Filtrer les lignes](filter.md)
- `*` sur un dossier monté par rclone demande toute la branche en une seule
  requête.
- L'exploration d'un dépôt Git n'exécute rien défini par le dépôt, sauf si son
  dossier figure dans `git_trusted`. [Git](git.md)

## 0.5.0

**Changement.** `-` dans l'arborescence libère la branche de la mémoire ; ← ne
fait que la replier. `E` ouvre l'éditeur dans le cadre de Xylem.

**Nouveautés.**

- Mesures de sécurité pour fichiers hostiles : limites de décompression,
  contrôles des chemins extraits, blocage des caractères de contrôle vers le
  terminal. [Sécurité](security.md)
- La visionneuse replie les lignes (`W`), les numérote (`#`), saute à une ligne
  (`g`), affiche les fichiers `.gz`, `.xz`, `.zst` et `.bz2` décompressés et
  reconnaît CP437, CP850, Windows-1252 et UTF-16.
- Aide en six langues ; `L` dans l'aide permet d'en changer.
- Ctrl+S dans l'arborescence liste uniquement les fichiers marqués de tout ce qui
  a été lu.
- Graph dans le menu Git.
- Icônes Nerd Fonts, désactivées par défaut.
- Paquets `.deb` et `.rpm`.

## 0.4.0

**Nouveautés.**

- Vue Utilisation : dossiers et fichiers d'une branche ordonnés selon l'espace
  occupé. La touche était Alt+W ; elle devient Alt+U dès la 0.8.
  [Utilisation du disque](usage.md)
- Alt+K Percent affiche la part de chaque répertoire par rapport à son parent.
- Machines distantes via SSH, en lecture seule dans cette version.
  [Machines distantes (SSH)](remote.md)
- Google Drive, S3 et autres stockages via rclone.
  [Stockage cloud (rclone)](cloud.md)
- Thèmes slate, graphite, daylight et vivid.
- `[editors]` dans `config.toml` choisit l'éditeur par nom de fichier et `E`
  refuse les binaires sans éditeur configuré.
- `xy --man` affiche la page de manuel.

## 0.3.0 et versions antérieures

- 0.3.0 : aide sur F1 et page de manuel ; archives 7z, tar.xz, tar.bz2 et
  tar.zst ; modes d'affichage par propriétaire et type (Alt+F) ; affichage des
  deux fichiers lors d'un conflit de copie.
- 0.2.0 : le programme passe de xt à Xylem, et la commande à `xy`. Corbeille et
  Oops, renommage par lot avec masque et presse-papiers.
- 0.1.0 : première version, publiée sous le nom xt.
