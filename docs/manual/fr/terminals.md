# Terminaux et touches

Xylem reçoit les touches transmises par le terminal. Certains terminaux
réservent une touche pour eux-mêmes, et d'autres envoient les mêmes octets pour
deux touches distinctes. Cette page recense ces situations et les touches de
remplacement à utiliser.

## Touches retenues par le terminal ou le bureau

| Touche | Retenue par | Utiliser à la place |
|---|---|---|
| F1 | GNOME Terminal (sa propre aide) | Alt+H |
| F11 | GNOME Terminal (plein écran) | Alt+B, ou `'` pour sauter à un signet |
| Alt+F10 | GNOME (maximise la fenêtre) | F12 |
| Alt+T | terminaux avec un menu « Terminal » | associer une autre touche dans `[keys.any]` |

## Ctrl+M

La plupart des terminaux envoient le même octet pour Ctrl+M et Enter. Xylem voit
alors Enter. Pour y déplacer les fichiers marqués, appuyez sur `F4` puis sur
`M`, ou utilisez Alt+M et appuyez sur F6 à l'invite pour désactiver la
recréation des chemins.

Dans un terminal prenant en charge le protocole clavier kitty, les deux touches
sont distinctes et Ctrl+M déplace directement les fichiers marqués.

## Le protocole clavier kitty

Avec ce protocole, le terminal signale l'appui et le relâchement de Ctrl et Alt
seuls. Xylem l'active automatiquement lorsque le terminal confirme qu'il le
gère.

| Avec le protocole | Sans le protocole |
|---|---|
| Maintenir Ctrl ou Alt affiche le menu CTRL ou ALT | F4 les affiche |
| Ctrl+M déplace les fichiers | Ctrl+M équivaut à Enter |

| Terminal | Protocole |
|---|---|
| kitty, Ghostty, WezTerm, foot (Wayland), Alacritty récent | oui |
| iTerm2 | Ctrl+M déplace ; maintenir Ctrl ou Alt non vérifié |
| GNOME Terminal, Ptyxis, Terminator, Tilix (basés sur VTE) | non |

## Le presse-papiers

`Y` et Ctrl+Y écrivent dans le presse-papiers. Via SSH et dans tmux, Xylem
utilise la séquence OSC 52, que le terminal doit accepter. Sous X11 et Wayland,
le texte reste dans le presse-papiers après avoir quitté Xylem.

## Icônes et bordures

Les icônes (F12, page 2) requièrent une police contenant les glyphes Nerd Fonts.
Sans eux, le terminal affiche un carré vide pour chaque icône. Si les colonnes
se décalent avec les icônes actives, le terminal dessine les caractères de
« largeur ambiguë » comme larges ; réglez cette option du terminal sur étroit
(*narrow*) ou désactivez les icônes.

## Taille de l'écran

| Largeur ou hauteur | Effet |
|---|---|
| moins de 72 colonnes | le nom et la version quittent l'en-tête |
| trop étroit pour une ligne de menu | les mots du menu sont raccourcis, puis les dernières commandes sont remplacées par `…` |
| moins de 58 colonnes ou 13 lignes | l'écran de démarrage n'est pas affiché |

Voir aussi : [les menus](menus.md), [configuration](configuration.md),
[le terminal intégré](terminal.md).
