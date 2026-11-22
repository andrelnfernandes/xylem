# Configuration

## L'écran de configuration (F12)

**F12** (ou Alt+F10 ; sous GNOME, Alt+F10 maximise
généralement la fenêtre et n'atteint jamais Xylem) ouvre cinq pages :

| Page | Options |
|---|---|
| 1. Programmes et formats | éditeur, ouvrir l'éditeur dans le cadre de Xylem, thème Xylem dans l'éditeur, éditer les fichiers binaires avec E, éditeur binaire, langue de l'aide, programme d'ouverture, format de date, format de nombres, commandes Git, dossiers Git de confiance |
| 2. Affichage et tri | fichiers cachés, ordre et sens de tri initiaux, action d'Enter dans la fenêtre de fichiers, mode d'affichage initial, coloration syntaxique, retour à la ligne, numéros de ligne, fichiers compressés, encodage des vieux textes, limite de taille de ce que lit la visionneuse, icônes et glyphes (Nerd Fonts), noms de fichiers colorés par type, bordures des fenêtres, couleurs du terminal intégré, secondes dans l'horloge, [la souris](mouse.md) |
| 3. Confirmations et historique | confirmer avant de quitter, supprimer vers la corbeille, confirmation de Prune, entrées avec `@` non enregistrées, historique propre au terminal intégré, [Alt+T dans la recherche approximative marque toutes les correspondances](#tagging-every-match-of-the-fuzzy-find), [chercher une nouvelle version](#new-versions) |
| 4. Copie, comparaison et rafraîchissement | ignorer 2 s, ignorer 1 h, rafraîchissement automatique, point de départ des chemins Alt+C/Alt+M, fichiers et dossiers laissés hors de l'index |
| 5. Lecture (livres numériques) | la largeur de la colonne de texte, paragraphes comme dans un livre ou en blocs, texte justifié, la vitesse de lecture pour le temps restant, [le temps passé à lire](#reading-time), les minutes sans touche avant que son horloge s'arrête, les titres conservés pour une liste des livres |

La limite de taille de la visionneuse (**L** à la page 2) coûte de la mémoire et du temps : avec `1024` Mio, ouvrir un fichier texte ordinaire de cette taille occupe environ 2,5 Gio de RAM et prend environ 5 s, et les fichiers aux lignes très courtes coûtent plus par octet (voir [la visionneuse](viewer.md)).

Les icônes et glyphes (**M** à la page 2) nécessitent une police intégrant le patch Nerd Fonts. Aucune police spécifique n'est imposée : n'importe quelle police à chasse fixe convient (comme Fira Code, JetBrains Mono, Hack, Meslo ou Cascadia Code), tant que vous utilisez son édition « Nerd Font ». Nerd Fonts injecte des milliers de glyphes standards dans la zone privée d'Unicode. Une police classique sans ce patch (même la Fira Code originale) ne contient pas ces icônes et affichera des carrés vides (`□`). Pour les utiliser :
- Téléchargez la Nerd Font de votre choix sur nerdfonts.com ou installez un paquet Nerd Fonts via le gestionnaire de paquets de votre distribution.
- Sélectionnez cette police dans les réglages de votre émulateur de terminal.

Si les colonnes se décalent avec les icônes, le terminal affiche probablement les caractères de « largeur ambiguë » en double largeur (un réglage courant avec les polices chinoises, japonaises et coréennes), alors que Xylem compte chaque icône comme une cellule. Réglez cette option sur étroit, ou désactivez les icônes.

Les noms colorés par type (**N** à la page 2) donnent aux archives, images, audio et vidéo, documents, code et données une couleur du thème chacun, d'après le nom seul (rien n'est lu). Désactivé, la liste garde l'aspect classique. Les bordures des fenêtres (**O**) sont arrondies dans les thèmes nord, gruvbox, slate, graphite, daylight et vivid, droites dans les autres ; `plain` ou `rounded` les fixe dans tous les thèmes. L'écran d'accueil (**P**) affiche le nom en lettres de bloc pendant deux secondes à l'ouverture de Xylem ; n'importe quelle touche le ferme (cette touche ne fait rien d'autre), et le dossier de départ est lu pendant ce temps. Il n'apparaît pas lors d'une connexion à une machine distante ou à un conteneur depuis la ligne de commande, ni dans un terminal de moins de 58 sur 13.

La lettre de l'option (ou Enter) modifie la valeur, PgUp/PgDn ou 1–5 changent de
page, **F4** choisit le thème dans une liste (avec aperçu immédiat), **F3**
enregistre et applique, **Esc** annule (en demandant confirmation s'il y a des
modifications).

### Marquer toutes les correspondances de la recherche approximative

Dans la recherche approximative (`|`), **Alt+T** marque les résultats de la
liste. La liste garde les 500 meilleures correspondances, et une recherche peut
en trouver bien plus : `f` dans une grande arborescence peut correspondre à cent
mille fichiers. Par défaut, Alt+T ne marque que les 500 de la liste, et quand il
y en avait plus, la ligne sous la liste le dit et nomme cette option.

Avec **Alt+T dans la recherche approximative marque toutes les correspondances**
activé (**F** à la page 3, `fuzzy_tag_all = true` dans le fichier), Alt+T marque
toutes les correspondances, y compris celles que vous n'avez jamais vues. La
ligne sous la liste dit alors toujours combien de correspondances n'y figuraient
pas.

C'est désactivé par défaut, car les marques s'appliquent à ce que vous faites
ensuite. Ctrl+C, Ctrl+M, Ctrl+D et le Batch agissent sur tous les fichiers
marqués, pas seulement sur ceux que vous avez vus. Des milliers de fichiers
marqués par une recherche à peine regardée, puis supprimés, sont des milliers de
fichiers perdus. N'activez l'option que si vous vérifiez ce qui est marqué avant
d'agir :

- le panneau des statistiques indique combien de fichiers sont marqués et leur
  taille ;
- **Ctrl+S** dans l'arborescence liste les fichiers marqués de toute
  l'arborescence (un Showall des marqués) ;
- **Ctrl+V** dans la fenêtre de fichiers les affiche l'un après l'autre.

### Nouvelles versions

Au premier démarrage de Xylem, une fois l'écran d'accueil disparu, il demande :
*Look for a new version of Xylem once a day?* **Y** répond oui, **N** non, et la
réponse est conservée dans le fichier de configuration (`check_updates`).
**Esc** ne conserve rien, et la question revient la fois suivante. **G** à la
page 3 change la réponse plus tard.

Une fois activé, Xylem demande une fois par jour au site web
(`https://andrelnfernandes.github.io/xylem/latest.txt`) une seule ligne : le
numéro de la dernière version. Il le fait avec le `curl` du système, en
arrière-plan, en HTTPS uniquement et en cinq secondes au plus ; sans réseau, ou
sans `curl`, rien n'est dit et il réessaie le lendemain. La requête ne contient
rien de plus que `xylem/` et la version que vous utilisez ; le site voit votre
adresse IP, comme pour toute visite. Rien n'est téléchargé ni installé : quand
une version plus récente existe, la ligne du haut indique `0.10.0 is out` à
côté de l'horloge, et un message donne l'adresse des versions. Mettez à jour
comme vous avez installé : le gestionnaire de paquets pour un `.deb` ou un
`.rpm`, une nouvelle archive sinon.

La réponse du site est traitée comme hostile : tout ce qui n'est pas trois
nombres, comme la page du Wi-Fi d'un hôtel, est ignoré. La date de la dernière
vérification et la version trouvée sont conservées dans le dossier d'état
(`~/.local/state/xylem/update-check`).

`XYLEM_CHECK_UPDATES=0` (ou `1`) dans l'environnement répond sans demander et
l'emporte sur le fichier, pour les installations sur de nombreuses machines. Une
compilation sans la fonctionnalité `update-check` (`cargo build
--no-default-features --features index`), comme une distribution peut en faire,
n'a ni question, ni vérification, ni option.

### Temps de lecture

La visionneuse lit les [livres numériques](viewer.md#ebooks) EPUB et FB2. La
page 5 règle la mise en page du texte, et si Xylem conserve le temps que vous
passez à lire.

**Conserver le temps passé à lire** (**E**, `reading_stats = true` dans le
fichier) est désactivé par défaut. Activé, Xylem conserve, pour chaque livre
mémorisé avec `R` dans la visionneuse :

- le temps de lecture, et les sessions : une pause de plus de 30 minutes en
  commence une nouvelle ;
- le point le plus loin atteint, le premier et le dernier jour de lecture, et si
  le livre a été terminé ;
- la vitesse de lecture, en mots par minute. Seules les pages lues en avançant
  comptent, et seulement quand au moins 5 secondes y ont été passées, donc les
  sauts, les retours en arrière et le survol ne comptent pas. Après 10 minutes
  mesurées, le pied de page calcule le temps restant avec la vitesse de ce
  livre au lieu de celle de cette page (**D**).

L'horloge ne tourne que pendant que vous lisez : d'une touche à la suivante, si
pas plus de **F** minutes (3 par défaut) se sont écoulées entre elles. Une pause
plus longue l'arrête, et cette pause ne compte pas. L'Autoview (F7) et le
rapport (`I`) ne comptent jamais.

Tout reste sur votre ordinateur, dans `~/.local/state/xylem/reading.toml` (ou
`$XDG_STATE_HOME/xylem/`), lisible par vous seul. Un livre est conservé sous la
forme d'une empreinte de ce qu'il dit être et de sa taille, et de nombres : ni
son nom, ni son emplacement, ni son titre. Avec **G** activé
(`reading_titles = true`), le titre et l'auteur sont aussi conservés, et le
rapport de n'importe quel livre liste alors tous les livres lus, sous « Your
books ».

Désactiver **G** et enregistrer (F3) supprime aussitôt les titres conservés.
Désactiver **E** et enregistrer demande s'il faut supprimer le temps de lecture
de tous les livres ; sinon, il reste dans le fichier et n'est pas affiché. Dans
les deux cas, les positions mémorisées avec `R` restent ; `R` sur un livre
l'oublie.

## Le fichier de configuration

Tout est enregistré dans `~/.config/xylem/config.toml` (ou
`$XDG_CONFIG_HOME/xylem/` ; `%APPDATA%\xylem\` sous Windows), qui peut aussi
être édité à la main : l'écran de configuration conserve vos commentaires. Pour
partir d'un fichier commenté contenant toutes les valeurs par défaut :

```bash
mkdir -p ~/.config/xylem && xy --default-config > ~/.config/xylem/config.toml
```

- `theme` : `classic` (blanc et jaune sur bleu), `dark`, `light`, `mono` (les couleurs du
  terminal), `norton`, `solarized`, `gruvbox`, `nord`, `slate` (dans le style de
  Windows Terminal), `graphite` (macOS en mode sombre), `daylight` (macOS en
  mode clair) ou `vivid` (couleurs vives, pour les présentations et captures
  d'écran). F4 les affiche tous avec un aperçu.
- `[options]` : les options de l'écran de configuration, et quelques-unes qui
  n'existent que dans le fichier : `scan_max_files` (10000), le nombre maximal
  de fichiers dont le [Scan](views.md#scan) lit le contenu.
- `[colors]` : couleurs individuelles surchargeant le thème, sous forme de noms
  (`"yellow"`), de codes `"#rrggbb"` ou d'indices de 0 à 255.
- `[keys.any]`, `[keys.tree]`, `[keys.files]` : redéfinissent les touches dans
  les deux fenêtres ou dans une seule (qui a priorité). Par ex. `"f3" = "view"`,
  `"ctrl+r" = "relog"` ; `"none"` désactive une touche. La liste des touches de
  l'aide (**K**) affiche les touches en vigueur, avec vos modifications.
- `[editors]` : l'éditeur pour chaque type de fichier
  (`"*.png *.jpg" = "gimp %f"`, `binary = "hexedit"`) ; voir
  [Éditer](execute.md).

Une erreur dans le fichier n'empêche pas Xylem de démarrer : il utilise les
valeurs par défaut et affiche l'erreur dans le pied de page, et l'écran de
configuration refusera d'enregistrer tant que le fichier n'aura pas été corrigé,
afin de ne pas écraser ce que vous avez écrit.

Xylem s'appelait xt jusqu'à la version 0.1. Au premier lancement, il déplace la
configuration, l'historique et les signets de xt vers ses propres dossiers.
