# Vues (Alt+V)

**Alt+V** ouvre les vues de la liste : d'autres façons de regarder les fichiers
de la fenêtre de fichiers, pour une question à laquelle l'arborescence et la
liste répondent mal. Un petit menu demande laquelle, chacune avec sa lettre :

| Touche | Vue | Ce à quoi elle répond |
|---|---|---|
| `S` | [Scan](#scan) | Qu'est-ce qui, dans ces fichiers, mérite un coup d'œil ? |
| `T` | [Timeline](#timeline) | Qu'est-ce qui a changé, et quand ? |
| `L` | [Table](#table) | Quels fichiers remplissent cette condition, vus comme un tableau ? |

Une vue porte sur **la liste courante** : le répertoire, le Branch ou le
Showall, avec le filespec et la condition `W` en vigueur. Elle ne fait que lire :
rien dans une vue ne supprime, ne déplace ni ne modifie un fichier. Ce que vous y
choisissez revient à la liste classique sous forme de filtre, et de là Ctrl+T,
Ctrl+C ou Batch agissent comme toujours. **W sans texte** retire ce filtre,
comme il le fait avec une condition.

Chaque vue peut aussi avoir sa propre touche dans `[keys]`
([configuration](configuration.md)), sans touche par défaut : `view_scan`,
`view_timeline`, `view_table`.

## Scan

Les rapports que la [visionneuse](viewer.md) fait des fichiers potentiellement
hostiles (exécutables, documents Office, patchs), un fichier à la fois avec `V`,
appliqués à une liste entière : un ZIP téléchargé, un dépôt cloné, une clé USB.
**Alt+V, S** (d'abord Branch, avec `B`, pour inclure les dossiers en dessous).

```
      1  Names with control or text-direction characters
      1  Names that deceive (fatura.pdf.exe, spaces, a trailing dot)
      3  setuid, setgid, writable by anyone
      2  Links that leave the folder, or point to nothing
      1  Executables: packed, without PIE or NX, writable and executable
      1  Office: macros, remote template, embedded objects, hidden text
      0  Patches: dangerous paths, hidden characters, hidden changes
      0  Not analysed: over a limit, or the reader could not take it
```

| Catégorie | Ce qu'elle examine |
|---|---|
| Noms avec des caractères de contrôle ou de direction du texte | Le nom de chaque fichier et dossier : caractères de contrôle, ceux qui inversent le texte (un nom affiché `exe.pdf` qui est `fdp.exe`), octets qui ne sont pas de l'UTF-8 |
| Noms trompeurs | L'extension d'un document avant une extension qui s'exécute (`fatura.pdf.exe`, `photo.jpg.scr`), des espaces avant l'extension, un nom qui finit par une espace ou un point |
| setuid, setgid, modifiable par tous | Le mode de chaque fichier, et de chaque dossier : un dossier où tout le monde peut écrire ne compte que sans le sticky bit, que `/tmp` possède |
| Liens qui sortent du dossier, ou ne mènent nulle part | Où pointe chaque lien, d'après son texte : le lien n'est jamais suivi pour lire ce qui s'y trouve |
| Exécutables | ELF, PE et Mach-O, reconnus à leurs premiers octets et non au nom : pas de PIE, pas de NX ou DEP, pas d'ASLR, une pile exécutable, des sections compressées (UPX et autres) ou à la fois modifiables et exécutables, des données après la dernière section |
| Office | Word, PowerPoint et OpenDocument : macros, ce que le document va chercher à l'extérieur à son ouverture (un modèle distant), objets incorporés, texte masqué et diapositives masquées |
| Patchs | `.patch` et `.diff` : fichiers qui sortent avec `..`, sont absolus, entrent dans `.git` ou deviennent exécutables ; noms avec des caractères cachés ; blocs dont les comptes ne correspondent pas ; lignes modifiées uniquement dans ce qui ne se voit pas |
| Non analysés | Les fichiers que les lecteurs n'ont pas pu prendre, chacun avec la raison : au-delà d'une limite, un paquet abîmé, un lecteur en échec |

Les noms et les modes des fichiers sont déjà en mémoire, et s'affichent
aussitôt. Les modes des dossiers, les liens et les contenus interrogent le
disque, en arrière-plan : l'écran se remplit à mesure qu'ils arrivent, dit
combien il en reste, et **Esc** l'arrête.

Les contenus passent par les mêmes lecteurs que les rapports de la
[visionneuse](viewer.md) (`V`), avec les mêmes limites, un fichier à la fois. Un
fichier au-delà d'une limite, ou qu'un lecteur n'a pas pu prendre, va dans
**Non analysés** avec la raison : il ne disparaît jamais en silence. Au plus
`scan_max_files` fichiers (10000) ont leur contenu lu ; au-delà, ils sont listés
comme non analysés ([configuration](configuration.md)).

| Touche | Action |
|---|---|
| ↑ ↓ | se déplacer |
| Enter | les fichiers de la catégorie, chacun avec la raison de sa présence |
| Enter, sur un fichier | retour à la fenêtre de fichiers, qui ne garde que les fichiers de cette catégorie, avec le curseur sur celui-ci |
| Enter, sur un dossier | le dossier dans l'arborescence |
| `V`, sur un fichier | l'afficher, avec son rapport ; Esc revient au Scan |
| `R` | scanner à nouveau |
| Esc | retour aux catégories, puis à la liste |

Esc sur la liste garde le Scan : Alt+V, S sur la même liste l'ouvre tel qu'il
était, sans relire. `R` relit.

### Où il ne regarde pas

- **Disques Windows et FAT, exFAT, NTFS, SMB, montages rclone.** Là, le pilote
  donne le même mode à tous les fichiers, donc les modes ne disent rien, et le
  Scan ne les compte pas : la catégorie le dit à la place. Sous WSL, il s'agit
  de `/mnt/c` et des autres disques Windows.
- **Dans une archive** (Alt+F5), les modes des fichiers sont ceux que l'archive
  enregistre, et ils comptent : un fichier setuid dans un TAR mérite d'être
  connu. Chaque membre est extrait avec une limite (64 Mio) et lu comme sur le
  disque. Les dossiers et les liens ne sont pas examinés. Dans un TAR, un membre
  situé loin dans l'archive n'est pas extrait, et est listé comme non analysé.
- **Sur un montage cloud** (rclone), les contenus ne sont pas lus : chaque
  fichier serait téléchargé. Les noms et le reste le sont.
- **Sur une machine distante**, seuls les noms et les modes des fichiers sont
  examinés, pour l'instant : lire les contenus ferait passer chaque fichier par
  SSH.

Le Scan n'est pas un antivirus. Il montre ce qui mérite un coup d'œil, et ne dit
pas que le reste est sûr.

## Timeline

Les fichiers de la liste dans un histogramme selon leur date de modification :
quand un dossier a été actif, ce qui est arrivé la semaine dernière, quelles
photos viennent de ce voyage. Elle ne lit rien sur le disque : seulement les
dates déjà en mémoire.

```
 2026-11-18: 312 files, 1.4 GiB

                                    █
                                    █▆
 ▁▁▂▁▁▁▁▁▃▁▁▂▁▁▁▁▁                  ██▁
                                    ▲
 11-01     11-05     11-09     11-13     11-17
```

L'échelle est choisie d'après les dates : heures, jours, semaines, mois ou
années. Les dates d'avant 1980 et celles du futur vont dans une barre à part à
chaque extrémité, parce que des dates fausses ou hostiles existent et
aplatiraient sinon le graphique. Les fichiers sans date sont comptés en dessous.

| Touche | Action |
|---|---|
| ← → PgUp PgDn Home End | aller de barre en barre ; la ligne du haut indique les fichiers et les octets de la barre |
| Shift+← → | choisir une plage de barres |
| `+` `-` | une échelle plus fine ou plus grossière (`+` des jours aux heures) |
| `B` | la hauteur des barres selon les octets au lieu des fichiers |
| Enter | retour à la fenêtre de fichiers avec seulement les fichiers de la barre ou de la plage |
| Esc | retour, sans rien changer |

Enter transforme le choix en une [condition W](sql.md) sur `modified`, comme
`modified >= '2026-11-18 00:00:00' AND modified < '2026-11-19 00:00:00'`,
jointe par AND à la condition déjà en vigueur. Le titre de la fenêtre de
fichiers l'affiche, et **W sans texte** la retire.

## Table

La liste s'ouvre dans la [visionneuse de tableaux](tables.md), un fichier par
ligne, avec les colonnes de la [condition W](sql.md) : `name`, `ext`, `dir`,
`size`, `disk`, `modified`, `age_days`, `mode`, `kind` et `tagged`, et `git` et
`dup` quand un fichier de la liste a un état Git ou fait partie d'un groupe de
[doublons](duplicates.md). Les valeurs sont celles que voit `W`, donc une
condition qui fonctionne dans l'une fonctionne dans l'autre.

La liste y a ce qu'ont les tableaux : le filtre SQL (`\` ou `&`),
l'enregistrement sous forme de fiche (Enter), la copie d'un enregistrement, les
marques (`m`) et le [Batch](execute.md) (Ctrl+B), une ligne de sortie par ligne
du tableau. Une liste de millions de fichiers s'ouvre aussitôt : le tableau est
lu par fenêtres, comme un gros fichier.

| Touche | Action |
|---|---|
| `\` ou `&` | le filtre SQL du tableau |
| `s` | trie par la colonne du curseur ; `s` à nouveau, dans l'autre sens ; le filtre reste actif |
| `W` | retour à la fenêtre de fichiers, avec le filtre comme condition W, joint par AND à celle qui était en vigueur |
| Esc | retour, sans rien changer |

Le tableau est la liste telle qu'elle était à son ouverture : un fichier marqué
ou modifié après cela y apparaît la prochaine fois qu'il s'ouvre. `type`, qui lit
chaque fichier, n'est pas une colonne ici ; `W` l'a toujours.
