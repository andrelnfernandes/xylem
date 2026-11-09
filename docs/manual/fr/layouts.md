# Layouts : fichiers à largeur fixe et multi-enregistrements

Beaucoup de fichiers échangés entre systèmes sont du texte sans noms de colonnes : chaque ligne est un enregistrement, et quels caractères contiennent quel champ est écrit dans un manuel. Les fichiers bancaires (CNAB 240), fiscaux (SPED) et l'export SDF de dBase et Clipper sont ainsi. Un layout indique à Xylem où se trouve chaque champ, et la visionneuse montre alors le fichier en tableau : un onglet par type d'enregistrement, chaque ligne commençant par son numéro de ligne dans le fichier.

## Ouvrir un fichier selon un layout

Xylem reconnaît certains fichiers à leur première ligne et les ouvre en tableau de lui-même, avec `V` et dans l'Autoview (F7). La note indique le layout utilisé : `Layout: FEBRABAN CNAB 240 — Padrão Geral  ·  K changes it`. Pour tout autre fichier, appuyez sur `K`.

| Touche | Ce qu'elle fait |
|---|---|
| `K` | la liste des layouts : tapez une partie d'un nom pour la réduire, Entrée en applique un, **F2** demande le chemin d'un fichier de layout, Échap revient. Quand un layout est utilisé, le premier élément le désactive et montre le fichier en texte |
| `[` `]` | type d'enregistrement précédent ou suivant |
| `Tab` | la liste des types d'enregistrement, avec le nombre de lignes de chacun |
| `i` | les champs de l'enregistrement : position, taille, type, description et la signification de chaque code ; au-dessus, le layout, sa source, les contrôles des totaux et leurs résultats |
| `T` | vers le texte, à la ligne de l'enregistrement sous le curseur, et retour |
| `\` ou `&` | le filtre SQL, par nom de champ : `valor_pagamento > 1000` |
| Ctrl+B | un Batch des enregistrements de l'onglet ouvert ([Batch dans la visionneuse](execute.md)) |

Les onglets suivent l'ordre où chaque type d'enregistrement apparaît pour la première fois dans le fichier : `0 - Header de Arquivo (1)`, `3A - Segmento A (420)`... Une ligne qui ne correspond à aucun enregistrement va, entière, dans l'onglet `! No record type`. La première colonne, `#line`, est la ligne dans le fichier, c'est ainsi que les validateurs des banques et du fisc signalent une erreur. Quand un champ a des codes qui ont une signification, la dernière ligne de la grille l'affiche : `Content: 2 — Retorno (Banco → Cliente)`.

Les valeurs sont lues comme le dit le layout : les codes gardent leurs zéros (`00012345`), les dates deviennent `2026-11-09`, les heures `14:30:00`, et les montants gardent leurs décimales exactes (`1234.50`) ; rien n'est arrondi. Une valeur qui n'est pas ce que dit le layout (des lettres dans un nombre, le 31 février) s'affiche telle qu'elle est écrite, et `i` les compte.

### Contrôle des totaux

Les trailers portent des totaux : les lignes d'un lot, le nombre de lots, la somme des paiements. Xylem les compte et les additionne pendant la lecture, en entiers, et indique dans `i` lesquels concordent et lesquels non, avec la ligne, ce que dit le fichier et ce qui a été compté. Quand l'un ne concorde pas, l'en-tête le signale : `1 check failed: i shows`. Un fichier lu seulement en partie (arrêté par Échap ou la limite de temps) n'a pas de contrôles.

## D'où viennent les layouts

| Où | Utilisé de lui-même ? |
|---|---|
| Le catalogue de Xylem, intégré au programme | oui, quand la première ligne correspond à son `[detect]` |
| Votre dossier : `~/.config/xylem/layouts/` (`%APPDATA%\xylem\layouts\` sous Windows), jusqu'à trois niveaux de sous-dossiers | oui, et il passe avant le catalogue |
| À côté du fichier (`K`, section `Here`), ou n'importe quel chemin (`K` puis F2) | seulement quand vous le choisissez |

Un fichier de layout dans le dossier parcouru n'est jamais lu de lui-même : un dossier téléchargé ou extrait pourrait en apporter un fait pour tromper. Quand vous le choisissez, il est lu avec la même rigueur que les données (voir Sécurité, plus bas).

Le catalogue contient, pour l'instant :

| Layout | Ce qu'il lit |
|---|---|
| FEBRABAN CNAB 240 — Padrão Geral | en-tête et trailer de fichier ; paiements par crédit en compte, chèque, OP, TED et Pix (segments A, B, C, Z) ; paiement de titres de recouvrement (J, J-52) ; recouvrement (P, Q, R à l'envoi ; T et U au retour). D'après le manuel de la FEBRABAN, version 11.0 |
| SPED — genérico | tout fichier SPED aux champs entre barres verticales (EFD ICMS/IPI, EFD-Contribuições, ECD, ECF...) : un onglet par enregistrement, l'ouverture et la clôture de chaque bloc et le bloc 9 avec ses contrôles. Les enregistrements qu'il ne déclare pas ont des champs numérotés (`CAMPO_02`, `CAMPO_03`...). Ce qui suit le `9999` dans les fichiers signés (la signature numérique) n'est pas lu |

Le dossier `layouts/` du [dépôt](https://github.com/andrelnfernandes/xylem/tree/main/layouts) liste ce qui est prévu et comment contribuer un layout, sans Rust.

### SDF à partir d'un .dbf

`COPY TO fichier SDF` dans dBase et Clipper écrit les enregistrements côte à côte, chaque champ avec sa largeur. Dans `K`, un `.dbf` à côté du fichier (ou un indiqué avec F2) donne son layout : les champs dans l'ordre, les dates en `YYYYMMDD`, les nombres avec un point. Les champs mémo ne sont pas exportés par SDF et restent de côté.

## Écrire un layout

Un layout est un fichier TOML dont le nom se termine par `.xlayout.toml`. Un positionnel :

```
schema = 1
name = "Clients de l'ancien système"
line_length = 40
[discriminator]
pos = [1, 1]
[[records]]
id = "0"
name = "En-tête"
role = "header"
fields = [
  { name = "type", pos = [1, 1], type = "code" },
  { name = "cree", pos = [2, 9], type = "date", format = "YYYYMMDD" },
  { pos = [10, 40], type = "filler" },
]
[[records]]
id = "1"
name = "Client"
fields = [
  { name = "type", pos = [1, 1], type = "code" },
  { name = "code", pos = [2, 7], type = "code" },
  { name = "nom", pos = [8, 30], type = "text" },
  { name = "solde", pos = [31, 40], type = "decimal", decimals = 2, sign = "trailing" },
]
```

Les positions commencent à 1 et incluent les deux bornes : `pos = [8, 30]` va du 8e au 30e caractère. `start = 8, len = 23` dit la même chose, et un `len` à côté de `pos` doit concorder avec lui, ce qui détecte les fautes de frappe. Avec `strict_positions` (activé par défaut), les champs d'un enregistrement doivent couvrir toute la ligne, sans trou ni chevauchement : donnez aux parties vides le type `filler`.

| Clé | Ce qu'elle indique |
|---|---|
| `schema` | toujours `1` |
| `name` | le nom affiché dans `K` et dans l'en-tête |
| `format` | `"positional"` (par défaut) ou `"delimited"` |
| `encoding` | `"auto"` (celui trouvé par la visionneuse ; `O` le change), `"utf-8"`, `"cp1252"`, `"cp850"` ou `"cp437"` |
| `offset_unit` | positionnel : `"chars"` (par défaut) ou `"bytes"` |
| `line_length`, `strict_positions` | positionnel : la longueur d'une ligne, et si les champs doivent la couvrir |
| `hide_filler` | les champs `filler` restent hors de la grille (par défaut `true`) ; `i` les liste toujours |
| `delimiter`, `outer_delimiters` | délimité : le séparateur, un caractère, et si les lignes commencent et finissent aussi par lui, comme dans SPED |
| `allow_undeclared_records` | délimité : les enregistrements que le layout ne déclare pas ont leur propre onglet |
| `decimal_sep`, `date_format` | valeurs par défaut de tous les champs : `"implied"`, `","` ou `"."` ; et le format des dates |
| `stop_after_trailer` | s'arrêter à l'enregistrement `trailer` (la suite n'est pas lue) |
| `[detect]` | comment reconnaître le fichier à sa première ligne : `line_length`, `first_line` (conditions) et `extensions` |
| `[discriminator]` | où est le code de l'enregistrement : `pos` ou, en délimité, `col` |
| `description`, `country`, `category`, `spec_source`, `spec_version`, `spec_url`, `status` | sur le layout et son manuel ; `status` vaut `"verified"` une fois vérifié avec lui |

Chaque `[[records]]` a un `id` (son code pour le discriminant ; `ids` en ajoute d'autres), un `name`, un `role` (`header`, `batch_header`, `detail`, `batch_trailer` ou `trailer`) et ses `fields`. Quand le code seul ne distingue pas les enregistrements, `match` ajoute des conditions sur la ligne, et `batch_match` des conditions sur la ligne de l'en-tête de son lot : dans le CNAB 240, le segment A d'un paiement et celui d'un débit ne diffèrent que par l'opération de l'en-tête de lot. Une condition est un endroit (`pos` ou `col`) et l'un de `value = "3"`, `in = ["1", "2"]` ou `regex = "^[0-9]+$"`. Parmi les enregistrements de même code, celui qui a le plus de conditions est essayé en premier.

| Clé du champ | Ce qu'elle indique |
|---|---|
| `name` | le nom de la colonne, aussi pour le filtre SQL |
| `pos`, `start` et `len`, ou `col` | où il se trouve |
| `type` | `text`, `code` (des chiffres qui ne sont pas une quantité : numéro fiscal, compte, code postal), `int`, `decimal`, `date`, `time` ou `filler` |
| `decimals`, `decimal_sep` | décimales, et où est la virgule : `"implied"` (le `V99` du COBOL), `","` ou `"."` |
| `sign` | `"none"`, `"leading"`, `"trailing"` ou `"overpunch"` (le signe zoné du COBOL : `{`, `A`-`I`, `}`, `J`-`R`) |
| `format` | dates : `YYYYMMDD`, `DDMMYYYY`, `YYMMDD`, `DDMMYY` ou `YYYYDDD` ; heures : `HHMMSS` ou `HHMM`. Les années à deux chiffres de 00 à 69 sont dans les années 2000 |
| `trim` | espaces retirés du texte : `"right"` (par défaut), `"left"`, `"both"` ou `"none"` |
| `desc`, `values` | la description, et ce que signifie chaque code : `values = { "0" = "Ajout", "9" = "Suppression" }` |
| `check` | ce avec quoi la valeur doit concorder, ci-dessous |

| `check` | La valeur doit être |
|---|---|
| `total_lines` | les lignes du fichier jusqu'à celle-ci |
| `record_count:1*` | les lignes des enregistrements de cet id ; `*` prend tout id qui commence ainsi, `+` en ajoute d'autres |
| `sum:3A.valor` | la somme d'un champ d'un enregistrement dans tout le fichier ; `+` en ajoute d'autres |
| `batch_lines` | les lignes depuis l'en-tête du lot jusqu'à celle-ci |
| `batch_sum:3A.valor+3J.valor` | la somme dans le lot |
| `count_by_record:REG_BLC` | les lignes dont le code est la valeur du champ indiqué (le 9900 de SPED) |
| `block_lines` | les lignes dont le code commence comme celui de celle-ci (le x990 de SPED) |

Les copybooks COBOL (`copybook = ...`), l'EBCDIC et les nombres compactés (`COMP-3`) sont prévus et pas encore lus.

## Sécurité

Les données comme un layout choisi hors de votre dossier peuvent être hostiles. Un layout est lu avec la même rigueur que les données : au plus 1 Mio, toute clé ou tout type inconnu est refusé, chaque position et chaque taille sont vérifiées, les noms ne peuvent contenir ni caractères de contrôle ni caractères de direction du texte, et enregistrements, champs, codes, conditions et expressions régulières ont des limites. Les expressions régulières s'exécutent en temps linéaire. Un layout ne peut pas lire d'autres fichiers : `copybook` est refusé pour l'instant.

Les données sont lues avec les limites du mode tableau : 3 s (100 ms dans l'Autoview), 50 000 lignes par onglet, 256 colonnes, et les lignes de plus de 512 Kio ne sont pas découpées en champs. Échap arrête la lecture. Les cellules sont rendues sûres pour le terminal comme les autres : les caractères de contrôle et de direction du texte s'affichent comme `·`.

Voir aussi : [la visionneuse](viewer.md).
