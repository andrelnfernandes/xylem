# Signets

Les signets vous ramènent aux répertoires que vous utilisez souvent, à l'aide
d'une touche chacun (`0`–`9`, `a`–`z`). Ils sont conservés entre les sessions,
avec l'historique des invites.

- **Ctrl+F11** (ou Ctrl+B dans l'arborescence) enregistre le répertoire
  courant : appuyez sur la touche à lui attribuer.
- **`'`** y va directement : appuyez sur la touche du signet. À cette même
  invite, `'` ou Enter ouvre la liste et `+` enregistre le répertoire courant.
- **F11** (ou Alt+B) ouvre le gestionnaire de signets : ↑ ↓ choisissent, Enter
  (ou la touche du signet) s'y déplace, **F2** lui donne un nom, Del supprime,
  Insert ou `+` enregistre le répertoire courant, Esc ferme.
- **F11 dans les invites de [Log](treespec.md), [Copy et Move](copy-move.md)**
  ouvre la même liste pour choisir un lieu : Enter (ou la touche du signet) le
  place dans l'invite, où vous le confirmez encore ; Esc revient à ce que vous
  aviez tapé.

Un répertoire situé hors de l'arborescence s'ouvre comme une nouvelle
arborescence, comme avec [Log](treespec.md). Dans la fenêtre de fichiers, Ctrl+B
lance [Batch](execute.md) ; utilisez-y Ctrl+F11.

## Noms

Un nom dit à quoi sert un signet : « Serveur Oracle », « Conteneur Postgres
env. 1 ». Dans le gestionnaire, **F2** sur un signet demande le nom, avec
l'actuel à modifier ; un nom vide le retire. Le nom apparaît avant le lieu, et
le lieu apparaît toujours à côté, pour qu'un nom ne cache pas où mène un signet.
Un nom compte au plus 60 caractères, sans caractères de contrôle ni de sens
d'écriture.

## Machines distantes et conteneurs

Dans une fenêtre sur une [machine distante](remote.md) ou dans un conteneur,
Ctrl+F11 enregistre le dossier par l'adresse qui l'ouvre :
`ssh://oracle@db01/u01/app`, `docker://postgres-env1/var/lib/postgresql`. Y aller
connecte comme le fait l'adresse tapée dans Log, en demandant le mot de passe si
ssh en a besoin ; aucun mot de passe n'est enregistré. Si la fenêtre est déjà
sur cette machine et que le dossier est dans son arborescence, seul le curseur
se déplace. Un dossier dont le chemin contient une espace ne peut pas être
enregistré ainsi, car une adresse n'en admet pas.

Le gestionnaire marque `[not found]` un dossier local qui n'existe plus. Il ne
vérifie pas les signets distants, ni les dossiers sur un montage cloud ou un
partage réseau : cela attendrait le réseau.
