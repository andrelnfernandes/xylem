# Renommer plusieurs fichiers avec un compteur

Cette procédure renomme les neuf photos de `photos/2025/trip` de `DSC_01.jpg`
… `DSC_09.jpg` en `trip_001.jpg` … `trip_009.jpg`.

## Étapes

1. Dans l'arborescence, placez le curseur sur `trip` et appuyez sur Enter pour
   aller à ses fichiers.
2. Appuyez sur Ctrl+T. Tous les fichiers de la liste sont marqués.
3. Appuyez sur Ctrl+R. L'invite demande le masque et s'ouvre avec `*.*`.
4. Appuyez sur Ctrl+U pour effacer l'invite, saisissez `trip_<001>.*` et appuyez
   sur Enter.

![Les quatre étapes : les fichiers de trip, Ctrl+T, Ctrl+R et le masque trip_<001>.* produisant trip_001.jpg à trip_009.jpg](../../screenshots/howto-rename-mask.gif)

Le pied de page affiche `Renamed 9 of 9 files`. Les fichiers sont numérotés dans
l'ordre de la liste : triez-la d'abord (Alt+S) si l'ordre est important.

## Le masque

| Élément | Signification |
|---|---|
| `trip_` | texte copié dans chaque nom |
| `<001>` | un compteur à trois chiffres : 001, 002, … |
| `.*` | l'extension de chaque fichier, inchangée |

L'étape 4 efface l'invite car la frappe s'ajouterait au `*.*` déjà présent :
`*.*trip_<001>.*` nommerait le premier fichier `DSC_01.DSC_01trip_001.jpg`.

## Autres masques

| Masque | `DSC_01.jpg` devient |
|---|---|
| `*.jpeg` | `DSC_01.jpeg` |
| `2025_*.*` | `2025_DSC_01.jpg` |
| `s/DSC/trip/` | `trip_01.jpg` |
| `*.<upper>` | `DSC_01.JPG` |

[Renommer](rename.md) détaille toutes les formes prises en charge.

## Quand le résultat diffère de vos attentes

- **Rien n'est renommé.** Le masque donnerait le même nom à deux fichiers ou
  plus. Sans compteur et sans `*`, chaque fichier recevrait le même nom.
- **Seuls certains fichiers ont été renommés.** Seuls les fichiers marqués sont
  renommés. Un Filespec peut masquer des fichiers et Ctrl+T ne marque que ce qui
  est affiché.
- **Ctrl+R ne renomme qu'un seul fichier.** La touche pressée était `R` seule,
  qui renomme le fichier sous le curseur.

Voir aussi : [Marquage et opérations groupées](tagging.md),
[Le menu Ctrl](commands-ctrl.md).
