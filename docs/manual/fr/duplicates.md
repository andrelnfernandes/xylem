# Doublons (Alt+D)

Les fichiers au même contenu, quels que soient leurs noms et leurs dossiers, et
l'espace gaspillé par les copies. **Alt+D** cherche dans la branche du répertoire
courant ; **D** dans la vue Utilisation cherche dans le dossier affiché.


> Jusqu'à la version 0.7, les doublons étaient Alt+Y, et Alt+D supprimait les
> fichiers marqués (désormais seulement Ctrl+D). Voir [Ctrl et Alt](tagging.md#ce-que-veulent-dire-ctrl-et-alt).

```
── 3 × 1.2 GiB (2.4 GiB to free) ──────────────────────
     1.2 GiB  2023-08-14  Backups/phone-2023.zip
 T   1.2 GiB  2023-08-14  Downloads/phone-2023.zip
 T   1.2 GiB  2023-08-14  Old/phone-2023 (1).zip
── 2 × 850.0 MiB (850.0 MiB to free) ──────────────────
     850.0 MiB  2021-12-25  Videos/christmas.mp4
 T   850.0 MiB  2022-01-03  WhatsApp/VID-20211225.mp4
```

Ce qui libère le plus vient en premier. Rien n'est marqué tant que vous ne le
demandez pas, et une copie de chaque groupe reste toujours : Xylem ne laisse pas
marquer la dernière.

## Choisir ce qui part

| Touche | Action |
|---|---|
| ↑ ↓ PgUp PgDn Home End | se déplacer |
| `T` ou Espace | marque ou démarque la copie |
| `K` | garde cette copie : marque les autres du groupe |
| `A` | dans chaque groupe, marque toutes sauf une ; `A` à nouveau change celle qui reste : la plus ancienne, la plus récente, le chemin le plus court |
| `F` | dans chaque groupe ayant une copie dans le dossier de celle-ci, garde celle-là |
| `U` | démarque tout |
| `D`, Suppr ou Ctrl+D | supprime les copies marquées, comme la commande Delete (vers la corbeille si `delete_to_trash` est activé) |
| `M` ou Ctrl+M | déplace les fichiers marqués |
| `G` | fichiers ou dossiers entiers |
| `V` ou Entrée | affiche le fichier |
| `E` | écrit la liste dans un fichier (jamais par-dessus un fichier existant) : taille, date, marque et chemin, une ligne vide entre les groupes |
| `\` | ferme la vue, avec l'arbre sur ce dossier |
| F3 | relit les dossiers et compare à nouveau |
| Échap ou Alt+D | ferme |

Avant de supprimer ou de déplacer, chaque groupe est revérifié : si une copie a
changé depuis l'analyse, ou si celle qui devait rester a disparu, rien de ce
groupe n'est touché, et le pied de page le dit.

## Dossiers entiers

Les sauvegardes répétées en entier prennent le plus de place, et une longue liste
de fichiers le cache. **G** montre les dossiers au contenu strictement identique :
mêmes noms, même contenu de chaque fichier, grand ou petit, et mêmes liens pointant
au même endroit. Seuls ceux du haut sont listés, pas les dossiers qu'ils contiennent.

Un dossier n'est jamais appelé copie si quelque chose en lui n'a pas été comparé :
un dossier illisible, un autre disque monté dedans, un fichier spécial ou un dossier
ignoré (ci-dessous). Deux clones d'un dépôt ont les mêmes fichiers, mais leur `.git`
peut contenir un travail différent.

## Comment les fichiers sont comparés

Les tailles viennent de la mémoire. Seuls les fichiers dont la taille est partagée
sont lus : d'abord les deux bouts, puis le milieu, par parties, et un groupe est
scindé dès que ses fichiers diffèrent, si bien que des fichiers différents ne sont
pas lus jusqu'au bout. La comparaison se fait par SHA-256. Cela se passe en
arrière-plan : le pied de page montre la progression et Échap annule.

Restent hors de la liste :

- les fichiers plus petits que `duplicates_min_size` (1 Kio par défaut) et les vides ;
- les liens et les fichiers spéciaux, qui ne sont pas suivis ;
- les autres liens physiques d'un même fichier : ils partagent déjà l'espace, et en
  supprimer un ne libère rien ;
- les autres disques montés dans la branche ;
- les dossiers de `duplicates_ignore` (`.git` et `node_modules` par défaut).

Un fichier modifié depuis la lecture de l'arbre, ou illisible, n'est pas comparé ;
la ligne du bas dit combien. F3 relit tout.

Sur un dossier réseau (NFS, SMB, sshfs...), le total à lire est demandé d'abord. Sur
une [machine distante](remote.md) aussi : les fichiers y sont lus, par SSH, et
réexaminés avant toute suppression, comme ici. Deux choses diffèrent : SFTP ne
donne pas d'inode, donc deux liens physiques vers un même fichier apparaissent
comme une paire (en supprimer un ne libère rien et ne perd rien) ; et les autres
disques montés à l'intérieur sont laissés de côté grâce à leur système de
fichiers, ce qui demande l'extension `statvfs` d'OpenSSH. Dans une archive,
Doublons n'est pas disponible.

## Dans le cloud (rclone)

Dans un dossier monté par `rclone mount`, lire un fichier revient à le télécharger.
Xylem demande donc à rclone les empreintes que le service conserve déjà (`rclone
lsjson --hash`) et ne télécharge rien : un Drive entier prend quelques minutes. Il
utilise l'empreinte la plus forte que donne le service (SHA-256, SHA-1, MD5...), avec
la taille, et rclone doit être dans le PATH. Sinon, ou si le distant ne donne pas
d'empreintes (un distant chiffré, par exemple), Xylem explique pourquoi et demande
avant de lire les fichiers par le montage, avec le total qu'il téléchargerait.

Les dossiers entiers ne sont comparés que sur des disques locaux.

Sur Google Drive :

- Un fichier dans deux dossiers à la fois, ou un raccourci, est un seul fichier : il
  n'est jamais proposé contre lui-même.
- Docs, Sheets et Slides n'ont pas d'empreinte : la ligne du bas dit combien ont été
  laissés de côté.
- Ce que d'autres ont partagé avec vous apparaît avec `s`, et « toutes sauf une » n'y
  touche pas : le supprimer ne libère rien de votre stockage.
- Deux fichiers au même nom dans un dossier ne peuvent pas être distingués par le
  montage ; la ligne du bas suggère `rclone dedupe`.
- Supprimer envoie à la corbeille du Drive, qui compte encore tant qu'elle n'est pas
  vidée. Xylem ne la vide pas, car cela emporterait tout ce qu'elle contient ; il
  montre la commande : `rclone cleanup gdrive:`.

L'empreinte du service est acceptée de confiance. MD5 peut être falsifié exprès : pour
des fichiers que d'autres ont pu y mettre, comparez avant de supprimer.
