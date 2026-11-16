# Choisir des fichiers avec une condition SQL

Cette procédure liste les fichiers JPG de plus de 1 Mo situés sous `~/projects`
et les marque, prêts pour Copier, Déplacer ou Supprimer.

## Étapes

1. Dans l'arborescence, placez le curseur sur `projects` et appuyez sur `B`. La
   fenêtre de fichiers liste les fichiers de toute la branche.
2. Appuyez sur `W`. L'invite demande une condition.
3. Saisissez `ext = 'jpg' AND size > 1000000` et appuyez sur Enter. La liste
   conserve les fichiers répondant à la condition et le pied indique leur nombre.
4. Appuyez sur Ctrl+T. Les fichiers de la liste sont marqués.

![Les quatre étapes : Branch sur projects, W, la condition saisie et les 12 fichiers retenus marqués avec Ctrl+T](../../screenshots/howto-sql-filter.gif)

Le titre de la fenêtre de fichiers affiche la condition : `Branch: * where ext =
'jpg' AND size > 1000000 (12)`. Appuyer sur `W` avec une invite vide réaffiche
la liste complète ; les marques sont conservées.

## Écrire la condition

La condition correspond à ce qui suit `WHERE` dans une requête SQL. Le texte se
place entre guillemets simples et les nombres s'écrivent entiers.

| Pour conserver | Condition |
|---|---|
| fichiers modifiés la semaine dernière | `age_days < 7` |
| fichiers Markdown et texte | `ext IN ('md', 'txt')` |
| noms commençant par `report` | `name LIKE 'report%'` |
| fichiers pas encore marqués | `NOT tagged` |
| fichiers contenant un texte | `contains('invoice')` |

[Une condition SQL](filespec.md#une-condition-sql-w) liste toutes les colonnes.

## Quand le résultat diffère de vos attentes

- **`No column "jpg"`.** Le texte a été saisi sans guillemets. `ext = jpg`
  compare deux colonnes entre elles ; `ext = 'jpg'` compare avec la chaîne.
- **Des fichiers attendus manquent.** `W` n'examine que les fichiers lus en
  mémoire. Ouvrez d'abord le Branch ou lisez d'autres dossiers avec `*` et ouvrez
  le Showall (`S`).
- **Rien ne change.** Aucun fichier ne répond à la condition et la liste reste
  telle quelle.
- **`contains` demande confirmation avant exécution.** Il lit chaque fichier à
  vérifier et la question précise le nombre de fichiers et d'octets concernés.

Voir aussi : [Filespec](filespec.md),
[Marquage et opérations groupées](tagging.md),
[copier des fichiers en gardant les chemins](howto-copy-with-paths.md).
