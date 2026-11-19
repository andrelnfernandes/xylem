# Utiliser les fichiers marqués dans une commande du shell

Cette procédure exécute `wc -c` sur les fichiers marqués dans Xylem, depuis le
terminal intégré. Les mêmes étapes fonctionnent avec toute commande prenant des
noms de fichiers.

## Étapes

1. Dans l'arborescence, placez le curseur sur le répertoire, appuyez sur → pour
   le lire et sur Enter pour aller à ses fichiers.
2. Marquez les fichiers : `T` sur chacun, ou Ctrl+T pour toute la liste.
3. Appuyez sur Alt+X. Un terminal s'ouvre dans la partie inférieure de l'écran,
   avec le shell positionné dans le répertoire affiché au-dessus.
4. Tapez la commande et un espace, `wc -c `, puis appuyez sur Alt+O. Xylem tape
   les noms des fichiers marqués à la position du curseur, entourés de guillemets.
5. Appuyez sur Enter.

![Les cinq étapes : les fichiers de notes, Ctrl+T, Alt+X ouvrant le terminal, wc -c avec Alt+O tapant les deux noms de fichiers et la sortie obtenue](../../screenshots/howto-terminal-tagged.gif)

Alt+X revient à Xylem. Le shell continue de tourner, avec son historique,
jusqu'à `exit`.

## Ce qu'Alt+O écrit

Les noms sont relatifs au répertoire du shell et protégés par des guillemets, de
sorte que les noms comportant des espaces constituent chacun un unique argument :

```
wc -c 'ideas.md' 'todo.md'
```

Dans un Branch ou Showall, les fichiers marqués proviennent de plusieurs
dossiers ; chaque nom porte alors son chemin relatif depuis le répertoire du
shell.

## La variable

À l'entrée dans le terminal, Xylem définit également `XY_TAGGED` avec les
fichiers marqués.

| Shell | Utilisation |
|---|---|
| bash | `cp "${XY_TAGGED[@]}" ~/backup` |
| zsh, fish | `cp $XY_TAGGED ~/backup` |
| sh, dash | non définie |

La variable contient les fichiers qui étaient marqués au moment d'entrer dans le
terminal. Si vous marquez d'autres fichiers, elle s'actualisera à la prochaine
entrée.

## Une commande unique avec eXecute

`X` (eXecute) demande une ligne de commande ponctuelle et l'exécute. `%t` y
représente les fichiers marqués et `%f` le fichier sous le curseur :

```
tar czf photos.tar.gz %t
```

[eXecute](execute.md) décrit les autres variables de substitution.

## Quand le résultat diffère de vos attentes

- **La première ligne du terminal affiche ` . '…/sync.sh'`.** Xylem l'écrit
  automatiquement pour déplacer le shell dans le dossier de l'arbre et définir
  `XY_TAGGED`. L'espace initial évite l'enregistrement dans l'historique sous
  bash et zsh.
- **Alt+O n'écrit rien.** Aucun fichier n'est marqué.
- **Le shell n'a pas changé de répertoire.** Un programme tournait dans le
  terminal. Xylem ne transmet le `cd` que lorsque le shell attend à son invite.
- **F1 n'ouvre pas l'aide.** Dans le terminal, toutes les touches sont envoyées
  au shell, à l'exception de Alt+X, Alt+Z et Alt+O.

Voir aussi : [le terminal intégré](terminal.md), [eXecute](execute.md),
[Marquage et opérations groupées](tagging.md).
