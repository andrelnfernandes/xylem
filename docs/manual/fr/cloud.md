# Stockage cloud (rclone)

Xylem n'a pas de client Google Drive ou S3 intégré, mais il n'en a pas besoin :
[rclone](https://rclone.org) les monte (ainsi qu'environ 70 autres services :
OneDrive, Dropbox, Backblaze B2, Cloudflare R2, MinIO...) comme un dossier, et
Xylem travaille dans ce dossier comme dans n'importe quel autre. Tout y
fonctionne : lecture, Branch, marquage, visionneuse, Copier et Déplacer dans les
deux sens, Renommer, Créer un répertoire, Supprimer.

## Pourquoi rclone

Nous avons choisi de nous appuyer sur rclone plutôt que de dialoguer directement
avec Google Drive et Amazon S3. Écrire notre propre code pour chaque service
ajouterait autant de sources d'erreurs, et Google Drive imposerait en outre les
règles de Google : une application qui lit tous les fichiers d'un utilisateur
doit être vérifiée par Google et passer un audit de sécurité annuel, sans quoi
elle affiche un avertissement « application non vérifiée » et reste limitée à
quelques utilisateurs. Pour S3, la bibliothèque officielle est très volumineuse
et ajouterait beaucoup de complexité, ce que Xylem évite par principe chaque
fois que possible. rclone fait déjà tout cela très bien, pour de nombreux
services à la fois.

## Configuration initiale (une seule fois)

Installez rclone (`apt install rclone`, `dnf install rclone`, `brew install
rclone`, ou depuis [rclone.org](https://rclone.org/downloads/)) et créez un
remote avec `rclone config`. Vous lui donnez un nom, qui est ce qui précède les
deux-points ci-dessous.

- **Google Drive :** type de stockage `drive` ; les valeurs par défaut
  conviennent. rclone ouvre le navigateur pour vous permettre de vous connecter
  à Google et d'autoriser l'accès. Sur une machine sans navigateur (via SSH),
  répondez *no* à la question sur le navigateur : rclone affiche une commande à
  lancer avec `rclone authorize` sur une machine qui en possède un.
- **Amazon S3 :** type de stockage `s3`, fournisseur `AWS`, puis soit la clé
  d'accès et le secret, soit *env_auth* pour utiliser ce dont se servent déjà
  les outils AWS (`~/.aws/credentials`, variables d'environnement, rôle de
  l'instance). Les autres services compatibles S3 (R2, MinIO, Wasabi...)
  figurent parmi les fournisseurs de la même liste.

`rclone lsd gdrive:` (ou `rclone lsd s3:`) liste les dossiers de premier niveau
(les buckets, sur S3) : si cela fonctionne, le remote est prêt.

## Monter et ouvrir

Sous Linux, avec un remote nommé `gdrive` :

```bash
mkdir -p ~/gdrive
rclone mount gdrive: ~/gdrive --vfs-cache-mode writes --daemon
xy ~/gdrive
```

Pour S3, faites de même avec `s3:` (tous les buckets) ou `s3:mon-bucket` (un
seul). `fusermount -u ~/gdrive` démonte le dossier.

- `--vfs-cache-mode writes` garde une copie locale des fichiers en cours
  d'écriture, ce dont ont besoin les éditeurs (`E`) et certains programmes.
- `--dir-cache-time 1h` conserve la liste des répertoires plus longtemps que les
  5 minutes par défaut : relire (F3) est plus rapide, mais les modifications
  effectuées ailleurs mettent plus de temps à apparaître.

Sous **macOS**, `rclone mount` nécessite [macFUSE](https://osxfuse.github.io) ;
`rclone nfsmount gdrive: ~/gdrive` ne demande rien de plus. Sous **Windows**, il
nécessite [WinFsp](https://winfsp.dev) et monte le stockage sur une lettre de
lecteur : `rclone mount gdrive: X:`, puis `xy X:\`.

Pour garder le dossier sous la main, enregistrez-le comme [signet](bookmarks.md)
(Alt+B).

Sous Linux, Xylem reconnaît les dossiers montés par rclone : le panneau DISK
affiche le service (Google Drive, Amazon S3, OneDrive...) et, en dessous, le
remote (`rclone gdrive:`). Info (Alt+I) et Avail (`A`) les affichent aussi. Le
service est lu dans le fichier de configuration de rclone
(`~/.config/rclone/rclone.conf`, ou `$RCLONE_CONFIG`) ; lorsqu'il est chiffré,
le panneau indique simplement rclone.

## Ce qui change

**Attendez-vous à des lenteurs par moments.** Tout passe par rclone et par le réseau du service :
cela peut donc être bien plus lent qu'un disque local, de par la nature même de cette intégration.
Afficher un fichier, ouvrir une archive ou lire un grand dossier peut prendre du temps, et l'écran
peut ne pas répondre tant que le service n'a pas répondu : c'est en général de l'attente, pas un
plantage. Sur Google Drive, il vaut la peine de créer votre propre `client_id` : voir la section
sur l'erreur 403 plus bas.

- **Tout passe par le réseau.** Chaque lecture de répertoire est une requête, et
  un service comme Google Drive les refuse quand elles sont trop nombreuses.
  Aussi, sur un dossier monté, `*`, Branch (`B`), la vue Usage et Compare sur une branche demandent à rclone
  toute la branche en une seule liste, bien plus rapide, sauf si elle est déjà
  entièrement lue ; Échap laisse l'arbre tel quel. Si rclone échoue ou montre
  d'autres dossiers que le montage, ils lisent dossier par dossier. Seul `*`
  déplie l'arbre.
- **L'écran n'attend pas.** L'arbre s'affiche aussitôt et se remplit à mesure que
  les dossiers arrivent, aussi en ouvrant un dossier (→ ou `+`) ou en y amenant le
  curseur ; la fenêtre de fichiers indique `reading...` entre-temps. F3 et l'accès
  à un chemin saisi attendent toujours leur lecture.
- **Les modifications faites ailleurs** (dans le navigateur, sur le téléphone)
  n'apparaissent qu'après F3, et une fois le cache de répertoires expiré
  (`--dir-cache-time`).
- **Supprimer :** répondez `P` (définitivement) plutôt que `T` : sous Linux, la
  corbeille d'un dossier monté est un dossier `.Trash-` (suivi de votre numéro
  d'utilisateur) à sa racine, de sorte que le fichier resterait sur le remote et
  continuerait d'occuper de l'espace. Google Drive place de toute façon ce que
  rclone supprime dans la propre corbeille du Drive ; sur S3, un fichier
  supprimé disparaît, sauf si le bucket utilise le versionnement.
- **Permissions et propriétaires** ne sont pas conservés : le montage affiche
  les mêmes pour tous les fichiers, et Attributs ne peut pas les modifier. Les
  dates sont conservées.
- **Git :** la liste de fichiers n'affiche pas les lettres d'état sur un dossier monté, car
  `git status` lit tout le dépôt par le réseau. Ctrl+G y fonctionne toujours.
- **Google Docs, Sheets et Slides** ne sont pas de vrais fichiers : rclone les
  présente exportés (`.docx`, `.xlsx`, `.pptx`), avec une taille de 0.
  `--drive-skip-gdocs` les masque.
- **S3 n'a pas de vrais répertoires**, seulement des noms contenant `/` : un
  répertoire vide peut disparaître une fois démonté, et renommer ou déplacer un
  répertoire copie puis supprime chacun des fichiers qu'il contient.

## Quand le service refuse : erreur 403

Google Drive limite le nombre de requêtes acceptées par minute. Sur un Drive
encombré de petits fichiers et dossiers (sauvegardes de projets, `node_modules`,
extensions d'éditeur), **les erreurs 403 sont courantes** : `Quota exceeded`,
`rateLimitExceeded`. rclone attend et réessaie, donc tout ralentit, et il peut
finir par abandonner. Tout gestionnaire de fichiers qui lit dossier par dossier à
travers le montage se heurte à la même limite.

Ce que fait Xylem : `*` affiche le nombre de requêtes refusées et, après 100,
demande s'il faut continuer d'essayer ou arrêter ; arrêter laisse l'arbre tel
quel. Si rclone abandonne, Xylem le signale et ne passe pas de lui-même à la
lecture dossier par dossier, qui ne serait que davantage refusée.

Ce qui aide, à commencer par le premier point :

- **Lire par dossiers.** Au lieu de `*` à la racine, utilisez `*` sur des dossiers
  plus petits, ou ouvrez-les avec → ou `+`. Sur un Drive encombré, c'est souvent la
  meilleure voie.
- **Votre propre `client_id`.** Si vous n'en avez pas créé, rclone utilise un client
  partagé avec ses autres utilisateurs, et le quota aussi. Créer le vôtre est
  gratuit : le [guide de rclone](https://rclone.org/drive/#making-your-own-client-id)
  l'explique. Ensuite, autorisez de nouveau avec `rclone config`. Les limites de
  votre compte restent, mais l'usage des autres ne compte plus contre vous.
- **Garder les listes plus longtemps :** `--dir-cache-time 1h` sur `rclone mount`.
- **Supprimer ce qui peut être recréé.** Moins de dossiers, c'est moins de requêtes,
  quel que soit le programme.

## Trouver ce qui remplit votre Google Drive

Lorsque le Drive réclame sans cesse un forfait de stockage supérieur, Xylem peut
montrer où va l'espace avant que vous ne payiez davantage. Montez le Drive,
allez à sa racine, lisez-le entièrement avec `*` (sur un grand Drive, cela prend
un moment) et choisissez Alt+K → **Percent** ([Tailles des
répertoires](info.md)) : chaque dossier indique quelle part de son dossier
parent il occupe, avec une barre proportionnelle au plus grand des dossiers de
même niveau. Suivez les plus grandes barres jusqu'aux anciennes sauvegardes,
vidéos et doublons.

La [vue d'utilisation du disque](usage.md) (Alt+W, à la racine du Drive) va plus
loin : fichiers et dossiers sont listés ensemble, les plus grands en premier, si
bien qu'un fichier énorme isolé ressort aussi bien qu'un dossier plein. Et
Branch (`B`) liste tous les fichiers d'un dossier : [triés](sort.md) par taille
décroissante (Alt+S, puis `O` pour décroissant et `S` pour Taille), les plus
volumineux arrivent en tête.

Certains éléments comptent dans votre forfait de stockage sans apparaître dans
le montage :

- **La corbeille du Drive** occupe de l'espace tant qu'elle n'est pas vidée (ou
  pendant 30 jours). `rclone about gdrive:` indique l'espace total, utilisé et
  dans la corbeille ; monter avec `--drive-trashed-only` n'affiche que le
  contenu de la corbeille.
- **Gmail et Google Photos** partagent le même espace de stockage, alors que
  seuls les fichiers du Drive figurent dans le montage. Si `rclone about`
  indique beaucoup plus d'espace utilisé que le total du montage, le reste se
  trouve probablement là.
- **Google Docs, Sheets et Slides** s'affichent avec une taille de 0, mais les
  documents récents comptent dans le quota de stockage.
- Les fichiers **partagés avec vous** ne comptent pas : ils sont décomptés du
  stockage de leur propriétaire.
