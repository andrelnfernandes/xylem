# Trouver ce qui remplit le disque

Cette procédure part du dossier personnel pour descendre jusqu'aux plus gros
fichiers et en supprimer un.

## Étapes

1. Dans l'arborescence, placez le curseur sur le répertoire à analyser et
   appuyez sur Alt+U. Xylem lit la branche et liste ses dossiers et fichiers
   selon l'espace occupé, les plus volumineux en tête.
2. Déplacez-vous sur un dossier (son nom commence par `/`) et appuyez sur Enter
   pour lister son contenu.
3. Appuyez à nouveau sur Enter sur le plus gros dossier et répétez jusqu'à ce
   que la liste affiche des fichiers.
4. Allez sur le fichier à supprimer et appuyez sur Del. Xylem demande
   confirmation avant suppression.
5. Appuyez sur `T` pour l'envoyer à la corbeille, puis sur `Y`.

![Les cinq étapes : Alt+U sur le dossier personnel, Enter sur projects, Enter sur backup, Del sur l'ancienne archive et T, Y l'envoyant à la corbeille](../../screenshots/howto-disk-space.gif)

Le pied de page affiche `Trashed 1 of 1 files` et les pourcentages sont
recalculés sans le fichier.

## Lire une ligne

```
     19.0 MiB ████████████████    51%   projects-2026-10.tar.gz
```

| Élément | Signification |
|---|---|
| `19.0 MiB` | l'espace occupé par l'élément sur le disque |
| la barre | l'élément comparé au plus gros élément de la liste |
| `51%` | la proportion de l'élément dans le répertoire listé |
| le nom | un dossier lorsqu'il commence par `/` |

## Touches dans la liste

| Touche | Action |
|---|---|
| Enter | entre dans le dossier ; sur un fichier, l'ouvre dans la visionneuse |
| Backspace ou ← | revient au dossier parent |
| `T` | marque le fichier, ou tous les fichiers du dossier |
| Ctrl+D | supprime les fichiers marqués |
| Alt+P | supprime le dossier avec tout son contenu, définitivement |
| `D` | cherche les doublons dans le dossier listé |
| `\` | ferme la liste et place le curseur de l'arbre sur le dossier listé |
| Esc | ferme la liste |

## Quand le résultat diffère de vos attentes

- **Une ligne commence par `>`.** Il s'agit d'un autre système de fichiers monté
  à cet endroit. Il n'est pas comptabilisé et appuyer sur Enter dessus ouvre une
  liste propre à ce disque.
- **Une ligne commence par `!` ou `.`.** Le dossier ou un élément interne n'a
  pas pu être lu, généralement par manque de permissions. L'espace réel occupé
  peut être supérieur à ce qui est affiché.
- **Les tailles diffèrent de la fenêtre de fichiers.** Cette liste affiche
  l'espace disque en blocs entiers. La touche `A` bascule vers la taille propre
  des fichiers.

## Un lecteur cloud

Les mêmes étapes fonctionnent sur un dossier Google Drive ou S3 monté par
rclone. [Stockage cloud (rclone)](cloud.md) décrit la configuration et les
particularités de ces services.

Voir aussi : [utilisation du disque](usage.md),
[trouver les doublons](howto-duplicates.md).
