# Touches de fonction et symboles

Ces touches figurent sur la dernière ligne des menus DIR et FILE, ou n'apparaissent
pas dans les menus parce qu'elles ne font que déplacer le curseur. Elles ne se
combinent pas avec les menus de F9 : après F9, seule la lettre suivante prend
Ctrl ou Alt.

## Touches de fonction

| Touche | Où | Action |
|---|---|---|
| F1 | les deux | Ouvre l'aide sur ce qui est à l'écran. Alt+H fait de même. [Terminaux et touches](terminals.md) |
| F2 | les deux | Relit le répertoire, comme F3. Dans une invite demandant un dossier, ouvre une arborescence pour [le choisir](picker.md). |
| F3 | les deux | Relit le répertoire depuis le disque. |
| F5 | Fichiers | Crée une archive contenant le fichier. Ctrl+F5 : avec les fichiers marqués. Alt+F5 : ouvre une archive. [Archives](archives.md) |
| F7 | Fichiers | Autoview : affiche le contenu du fichier sous le curseur à côté de la liste. [La fenêtre de fichiers](files.md) |
| F8 | les deux | Ouvre une seconde fenêtre à côté de la première, ou la ferme. [Écran partagé](split.md) |
| F9 | les deux | Affiche le menu CTRL, puis le menu ALT, puis revient au menu normal. [Les menus](menus.md) |
| F10 | les deux | Quitte l'application, comme `Q`. |
| F11 | les deux | Ouvre le gestionnaire de signets, comme Alt+B. Ctrl+F11 enregistre le répertoire courant. [Signets](bookmarks.md) |
| F12 | les deux | Ouvre la configuration. [Configuration](configuration.md) |

## Symboles

| Touche | Où | Action |
|---|---|---|
| `*` | Arborescence | Lit le répertoire sous le curseur et tout ce qui est en dessous. Esc annule. |
| `+` ou → | Arborescence | Lit le répertoire et affiche ses sous-répertoires. |
| `-` | Arborescence | Libère la branche de la mémoire. |
| ← | Arborescence | Replie le répertoire, ou va au répertoire parent. |
| `?` | les deux | Affiche les statistiques de la fenêtre, du disque, de la machine et de Xylem. [Informations](info.md) |
| `\` ou `/` | les deux | Treespec : tapez un chemin et le curseur de l'arbre le suit. Dans un Branch ou Showall, va au répertoire du fichier sous le curseur. [Treespec et Log](treespec.md) |
| `'` | les deux | Saute à un signet : appuyez ensuite sur sa touche. |

## Se déplacer

| Touche | Où | Action |
|---|---|---|
| ↑ ↓ PgUp PgDn Home End | les deux | Déplacent le curseur. |
| Espace | les deux | Descend d'une ligne. |
| Enter | Arborescence | Va à la fenêtre de fichiers. |
| Enter | Fichiers | Revient à l'arborescence. Avec `enter_views` actif, affiche le fichier. |
| Esc | Fichiers | Revient à l'arborescence, sur le répertoire du fichier sous le curseur. |
| Esc | Arborescence | Dans une archive, la ferme. |
| Tab | les deux | En écran partagé, passe à l'autre fenêtre. Sinon, identique à Enter. |
| Backspace | les deux | Va au répertoire parent. |
| Shift+lettre, 0–9 | les deux | Va au répertoire ou fichier suivant dont le nom commence par ce caractère. |
| ← → | Fichiers | Page précédente et suivante. En mode colonnes, fichier précédent et suivant. |

## Modifier une touche

`[keys.any]`, `[keys.tree]` et `[keys.files]` dans `config.toml` associent une
touche à une autre commande, ou à `none`. **K** dans l'aide liste les touches en
vigueur avec ces changements. Voir [configuration](configuration.md).

Voir aussi : [les menus DIR et FILE](commands.md),
[le menu Ctrl](commands-ctrl.md), [le menu Alt](commands-alt.md).
