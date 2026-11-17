# Trouver les doublons et garder une copie

Cette procédure trouve les fichiers ayant le même contenu sous un répertoire et
envoie à la corbeille toutes les copies sauf une pour chacun d'eux.

## Étapes

1. Dans l'arborescence, placez le curseur sur le répertoire et appuyez sur `*`
   pour lire sa branche.
2. Appuyez sur Alt+D. Xylem compare les fichiers et liste les groupes de
   fichiers identiques, le groupe libérant le plus d'espace en tête.
3. Appuyez sur `A`. Dans chaque groupe, toutes les copies sauf une sont marquées
   (`T`).
4. Appuyez sur `D`. Xylem demande confirmation avant suppression. Appuyez sur
   `T` pour envoyer à la corbeille, puis sur `Y`.

![Les quatre étapes : lecture de la branche, Alt+D listant trois groupes, A marquant toutes les copies sauf une et D, T, Y envoyant quatre fichiers à la corbeille](../../screenshots/howto-duplicates.gif)

Le pied de page affiche `Trashed 4 of 4 files`.

## Choisir quelle copie conserver

La touche `A` marque toutes les copies sauf une dans chaque groupe. Avant
l'étape 4, vous pouvez changer la copie conservée :

| Touche | Effet |
|---|---|
| `A` à nouveau | change la règle de conservation : la plus ancienne, la plus récente ou celle au chemin le plus court |
| `K` | conserve la copie sous le curseur et marque les autres de son groupe |
| `F` | dans chaque groupe ayant une copie dans le dossier de cette copie, conserve celle-là |
| `T` ou Espace | marque ou démarque une copie individuelle |
| `U` | démarque tout |

Xylem ne permet pas de marquer la dernière copie restante d'un groupe.

## Dossiers entiers

La touche `G` bascule la liste vers les dossiers dont l'ensemble des contenus
est identique. La ligne inférieure l'indique le cas échéant :
`1 group of folders (G)`.

## Ce que signifie la question

Lorsque `delete_to_trash` est désactivé (le réglage par défaut), l'étape 4
demande d'abord `DELETE 4 tagged files? This cannot be undone.`. Appuyer sur `T`
modifie la question en `MOVE 4 tagged files to trash?`. Appuyer ensuite sur `Y`
confirme l'action décrite.

## Quand le résultat diffère de vos attentes

- **Un fichier répété n'apparaît pas dans la liste.** Les fichiers de moins de
  1 Kio, les fichiers vides, les liens et les dossiers `.git` et `node_modules`
  sont exclus par défaut. [Doublons](duplicates.md) explique comment modifier ce
  comportement.
- **Un groupe n'a pas été supprimé.** Une copie a changé après la comparaison.
  Le groupe reste intact et le pied de page le signale. Appuyez sur F3 pour
  relancer la comparaison.

Voir aussi : [doublons](duplicates.md), [supprimer](delete.md),
[trouver ce qui remplit le disque](howto-disk-space.md).
