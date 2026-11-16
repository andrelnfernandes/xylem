# Copier des fichiers d'une autre machine par SSH

Cette procédure affiche `deploy@webserver` aux côtés de cette machine et copie
trois fichiers de sauvegarde vers un répertoire local.

## Étapes

1. Appuyez sur F8. L'écran se divise en deux fenêtres, toutes deux dans le
   dossier courant. La fenêtre gauche a le focus.
2. Appuyez sur `L`, saisissez `deploy@webserver:` et appuyez sur Enter. La
   fenêtre gauche affiche désormais le dossier personnel de cet utilisateur sur
   cette machine.
3. Placez le curseur sur `backups`, appuyez sur → pour le lire et sur Enter pour
   aller à ses fichiers.
4. Appuyez sur `T` sur chaque fichier à copier.
5. Appuyez sur Ctrl+C. L'invite suggère le répertoire de l'autre fenêtre.
6. Appuyez sur Enter.

![Les six étapes : F8, Log vers deploy@webserver, les fichiers de backups, trois fichiers marqués, Ctrl+C suggérant la fenêtre locale et trois fichiers copiés](../../screenshots/howto-ssh-copy.gif)

Le pied de page affiche `Copied 3 of 3 files`.

## L'adresse

| Adresse | Ouvre |
|---|---|
| `user@host:` | le répertoire personnel |
| `user@host:/srv/www` | ce répertoire |
| `ssh://user@host:2222/srv/www` | identique, sur un autre port |
| `docker://container/path` | un conteneur en cours d'exécution, via `docker exec` |

Xylem utilise votre propre client `ssh`. Les alias d'hôtes, clés, agent SSH et
`known_hosts` de `~/.ssh/config` sont pris en compte, et rien n'est installé sur
la machine distante.

## Mots de passe

Si ssh réclame un mot de passe ou une phrase secrète (*passphrase*), la demande
apparaît dans le pied de page et la frappe reste masquée. Xylem la transmet à
ssh sans la stocker. Pour un hôte rencontré pour la première fois, le pied
affiche l'empreinte de sa clé et attend `yes`.

## Dans l'autre sens

Pour copier vers la machine distante, passez sur la fenêtre locale avec Tab,
marquez les fichiers et appuyez sur Ctrl+C : l'invite suggérera le répertoire de
la fenêtre distante. Move, Rename, Delete, Makedir, Attributes et New date
fonctionnent également du côté distant.

## Quand le résultat diffère de vos attentes

- **La fenêtre de fichiers indique `Not logged`.** Le répertoire n'a pas été lu.
  Revenez avec Esc et appuyez sur → dessus. L'étape 3 s'en charge.
- **Ctrl+C demande une destination sans rien suggérer.** L'écran n'est pas
  partagé. Saisissez un chemin local ou appuyez sur F8 d'abord.
- **Edit et eXecute ne sont pas dans le menu.** Ils ne fonctionnent pas sur une
  machine distante. Alt+X y ouvre un shell.
- **Il n'y a pas de corbeille.** La suppression sur machine distante est
  définitive, et la question le précise.

Pour revenir plus tard sans retaper l'adresse, enregistrez le répertoire comme
signet avec Ctrl+F11.

Voir aussi : [machines distantes (SSH)](remote.md),
[écran partagé](split.md), [signets](bookmarks.md).
