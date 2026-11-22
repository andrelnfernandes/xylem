# Sécurité

Xylem traite comme hostile tout fichier qu'il n'a pas lui-même créé : fichiers
téléchargés, archives ouvertes, noms transmis par un serveur distant, dépôts
clonés. Le contenu ou le nom d'un fichier ne doit jamais provoquer de plantage,
de blocage, d'exécution de code, d'écriture hors destination ni d'épuisement de
mémoire. Si cela se produit, il s'agit d'une anomalie à signaler.

## Ce que fait Xylem

| Risque | Mesure |
|---|---|
| Fichier malveillant exploitant le lecteur | Texte, hexadécimal, encodages, fichiers compressés, archives, Markdown, feuilles de calcul, CSV, JSON Lines et DBF sont analysés par des parsers écrits en Rust. SQLite et Parquet sont lus par des bibliothèques C uniquement à l'ouverture par `V`, jamais dans Autoview. |
| Fichier exécutant un programme | Aucun programme externe n'est lancé sur un fichier sans action explicite avec `E`, `O` ou `X`. |
| Petit fichier s'étendant pour saturer mémoire ou disque | Chaque lecteur applique des limites de volume lu, de décompression, de temps d'exécution et de mémoire du décompresseur. Un seul niveau de compression est défait. Afficher un fichier n'écrit rien sur le disque. |
| Nom masquant sa nature réelle | Les caractères de contrôle et les contrôles bidirectionnels (Trojan Source) ne sont pas envoyés au terminal. Un nom contenant des caractères de direction masqués est affiché en rouge et `O` demande confirmation avant ouverture. |
| Archive écrivant en dehors de la destination | Les chemins contenant `..` et les chemins absolus sont rejetés, les liens symboliques dans les archives ne sont pas extraits et les bits setuid sont retirés. Les mêmes contrôles s'appliquent aux noms renvoyés par un serveur SFTP. |
| Dépôt Git exécutant ses propres commandes | L'exploration d'un dépôt n'exécute aucun hook, aucun `core.fsmonitor`, aucun diff externe ni conversion de texte, sauf si son dossier figure dans `git_trusted`. |
| Dépendance comportant une faille connue | Les dépendances sont vérifiées contre la base de données RustSec lors de l'intégration continue (CI). |

## Le réseau

Xylem ne se connecte au réseau de lui-même que pour une seule chose : si vous
avez dit oui au premier démarrage, il demande au site web une fois par jour le
numéro de la dernière version ([Nouvelles versions](configuration.md#new-versions)).
Tout le reste qui utilise le réseau est quelque chose que vous avez demandé :
une machine distante (`ssh`), le cloud (`rclone`), un conteneur (`docker`), un
partage réseau, et les programmes que vous lancez vous-même (`X`, le terminal
intégré).

## Ce que vous pouvez faire

- Désactivez la décompression dans la visionneuse (`viewer_decompress = false`,
  F12 page 2) lors de l'exploration de dossiers non sûrs avec Autoview actif, ou
  gardez Autoview fermé dans ces répertoires.
- Laissez Git désactivé (`git = false`, par défaut) dans les dépôts que vous
  n'avez pas créés vous-même.
- Répondez non à la recherche de nouvelles versions, ou désactivez-la à la
  page 3 de F12, si Xylem ne doit jamais se connecter de lui-même.
- N'exécutez pas Xylem en tant qu'administrateur (root).
- `O` transmet le fichier à une autre application. Les limites de Xylem ne
  s'appliquent pas à cette application tierce.

## Ce qui n'est pas couvert

- Les programmes démarrés par `E`, `O` et `X`.
- `rclone`, `ssh`, `docker` et `git`, exécutés par Xylem tels qu'ils sont
  installés sur le système hôte.
- Un terminal traitant lui-même incorrectement le texte qu'il reçoit.

## Signaler un problème

Signalez-le de manière confidentielle via les GitHub Security Advisories :
`https://github.com/andrelnfernandes/xylem/security/advisories/new`. N'ouvrez pas
d'issue publique à ce sujet.

Voir aussi : [la visionneuse](viewer.md), [archives](archives.md), [Git](git.md).
