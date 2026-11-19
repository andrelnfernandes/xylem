# Copier des fichiers de plusieurs dossiers en gardant les chemins

Cette procédure copie tous les fichiers JPG situés sous `~/projects` vers
`/mnt/backup`, chacun dans le sous-dossier correspondant. Les mêmes étapes
fonctionnent avec Move (Alt+M).

## Étapes

1. Dans l'arborescence, placez le curseur sur `projects`.
2. Appuyez sur `B`. Xylem lit la branche et la fenêtre de fichiers liste les
   fichiers de `projects` et de tous les dossiers en dessous. Sur une grande
   branche, le pied de page affiche la progression et Esc annule.
3. Appuyez sur `F`, saisissez `*.jpg` et appuyez sur Enter. La liste ne contient
   plus que les fichiers JPG.
4. Appuyez sur Ctrl+T. Tous les fichiers de la liste sont marqués (`■`).
5. Appuyez sur Alt+C. L'invite demande la destination.
6. Saisissez `/mnt/backup`. Avant d'appuyer sur Enter, lisez le mode affiché dans
   l'invite et appuyez sur **F7** jusqu'à obtenir `Relative`.
7. Appuyez sur Enter. Si `/mnt/backup` n'existe pas, Xylem demande confirmation
   avant de le créer.

![Les sept étapes : Branch sur projects, le filespec *.jpg, Ctrl+T, Alt+C, la destination avec le mode Relative et 21 fichiers copiés](../../screenshots/howto-copy-paths.gif)

Le bandeau sous l'écran indique l'étape en cours et ses touches.

## Quel mode de chemin choisir

L'étape 6 détermine le point de départ de chaque chemin sous la destination.
Pour `~/projects/photos/2025/trip/DSC_01.jpg`, avec le Branch ouvert sur
`projects` :

| Mode | Le fichier est écrit dans |
|---|---|
| Full | `/mnt/backup/home/user/projects/photos/2025/trip/DSC_01.jpg` |
| Tree | `/mnt/backup/` et le chemin depuis la racine de l'arbre Xylem |
| Current | `/mnt/backup/projects/photos/2025/trip/DSC_01.jpg` |
| Relative | `/mnt/backup/photos/2025/trip/DSC_01.jpg` |

Le mode initial de l'invite est `Full`, sauf indication contraire de
`copy_paths` dans la configuration.

## Variantes

- Sans les chemins : Ctrl+C au lieu de Alt+C place tous les JPG directement dans
  `/mnt/backup`. Deux fichiers portant le même nom entrent alors en conflit et
  Xylem demande quoi faire pour chacun.
- Par date ou taille : à l'étape 3, `*.jpg,>TODAY-30` ne retient que les JPG
  modifiés au cours des 30 derniers jours. [Filespec](filespec.md) décrit la
  syntaxe.
- Depuis tout le disque : lisez davantage avec `*`, puis appuyez sur `S`
  (Showall) à l'étape 2.
- Sélection manuelle : sautez l'étape 4 et marquez chaque fichier individuel
  avec `T`.

## Quand le résultat diffère de vos attentes

- **Alt+C lance une comparaison (Compare).** Le focus est sur l'arborescence.
  Alt+C copie avec chemins uniquement dans la fenêtre de fichiers ; appuyez sur
  Enter pour y aller.
- **Des fichiers manquent dans la liste.** Un Filespec défini plus tôt reste
  actif. Il s'affiche dans le titre de la fenêtre de fichiers ; `F`, `*`, Enter
  l'efface.
- **Les dossiers dans la destination commencent à `/home`.** Le mode était
  `Full`. Voir l'étape 6.

Ctrl+U démarque toute la liste et `B` revient aux seuls fichiers du répertoire.

Voir aussi : [Copier et Déplacer](copy-move.md),
[Marquage et opérations groupées](tagging.md),
[La fenêtre d'arborescence](tree.md).
