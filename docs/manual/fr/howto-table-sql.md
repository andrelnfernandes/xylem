# Filtrer un CSV ou un tableur avec SQL

Cette procédure ouvre `customers.csv` comme un tableau et retient les lignes dont
le forfait est `team` ou dont le solde dépasse 1000.

## Étapes

1. Dans la fenêtre de fichiers, placez le curseur sur `customers.csv` et appuyez
   sur `V`. La visionneuse affiche le fichier comme un tableau, la première
   ligne faisant office de noms de colonnes.
2. Appuyez sur `&`. L'invite demande une condition.
3. Saisissez `plan = 'team' OR balance > 1000` et appuyez sur Enter.

![Les trois étapes : V sur customers.csv montrant neuf lignes, l'invite & et la condition retenant trois lignes](../../screenshots/howto-table-sql.gif)

Le pied de page affiche `Filter matched 3 rows`. Les lignes conservent leurs
numéros dans le fichier d'origine. Appuyer sur `&` avec une invite vide réaffiche
toutes les lignes.

## Quels fichiers s'ouvrent en mode tableau

| Format | Extensions |
|---|---|
| CSV, TSV | `.csv` `.tsv` `.tab` |
| JSON Lines | `.jsonl` `.ndjson` |
| Feuilles de calcul | `.xlsx` `.xls` `.xlsb` `.ods` |
| dBase | `.dbf` |
| SQLite | `.db` `.sqlite` `.sqlite3` |
| Parquet | `.parquet` |

Le fichier doit résider sur le disque local. Un fichier sur machine distante ou
dans une archive s'ouvre en mode texte brut.

## Écrire la condition

| Pour retenir les lignes où | Condition |
|---|---|
| une colonne égale un texte | `plan = 'team'` |
| un nombre est dans un intervalle | `balance >= 0 AND balance < 100` |
| un texte commence par un préfixe | `name LIKE 'Ana%'` |
| un texte contient un fragment, sans distinction de casse | `city ILIKE '%paulo%'` |
| une cellule est vide | `email IS NULL` |
| une colonne n'a pas de nom exploitable | utiliser sa lettre : `C = 'pro'` |

Une cellule ressemblant à un nombre est traitée comme un nombre : écrivez
`code = 7`, pas `code = '7'`. [Le filtre SQL](tables.md#le-filtre-sql) détaille
les opérateurs.

## Dans un tableur ou une base de données

- Une feuille de calcul n'a pas de noms de colonnes : les colonnes sont `A`, `B`,
  `C`. Les touches `[` et `]` naviguent entre les feuilles précédente et suivante
  et Tab les liste.
- Dans une base SQLite, Tab liste les tables et vues. La même invite accepte
  aussi une requête complète commençant par `SELECT` ou `WITH` :

```
SELECT plan, count(*), sum(balance) FROM customers GROUP BY plan
```

Xylem fonctionne uniquement en lecture seule. `UPDATE`, `DELETE` et assimilés
sont refusés.

## Enregistrer le résultat

Ctrl+B écrit les lignes retenues par le filtre dans un fichier : CSV, JSON, HTML,
script SQL ou table SQLite. [Batch](execute.md) décrit les formats disponibles.

## Quand le résultat diffère de vos attentes

- **`SQL error`.** La condition n'a pas pu être interprétée. Le filtre
  précédemment actif est maintenu.
- **Aucune ligne ne correspond à une colonne visible.** Un nom de colonne
  inexistant compte comme vide (*NULL*) sans générer d'erreur. Vérifiez
  l'orthographe ou utilisez la lettre de colonne.
- **Le fichier s'ouvre en texte.** Son extension ne figure pas dans le tableau
  ci-dessus, ou le fichier est compressé, distant ou dans une archive. `T`
  bascule entre tableau et texte lorsque les deux existent.

Voir aussi : [la visionneuse](viewer.md),
[choisir des fichiers avec une condition SQL](howto-sql-filter.md).
