# Plano: refazer as capturas e mostrar o Ctrl e o Alt

Origem: conversa de 2026-11-03. As capturas e os GIFs de `docs/screenshots/` são de
2026-11-02, antes do nome e da versão no cabeçalho e de outras melhorias visuais. O
autor pediu para refazê-las, mostrar o efeito de segurar Ctrl e Alt (como num kitty)
e conferir se todos os menus aparecem certos. Aprovado.

## Por que não capturar o kitty de verdade

O desktop do autor é GNOME com Wayland: um programa não captura a tela sozinho (o
`grim` é recusado, e o portal pede aprovação a cada captura). O gerador atual
(`docs/screenshots/tools/shot.py`) emula o terminal com o pyte e desenha o PNG; como
é o Xylem que desenha tudo, a imagem é a mesma do kitty, menos a fonte. O kitty
configurado fica para o autor conferir o resultado à mão.

## O que muda no gerador

1. **O protocolo de teclado do kitty.** O Xylem só liga o modo em que Ctrl e Alt
   sozinhos trocam o menu quando o terminal responde à pergunta `CSI ? u`. Com
   `SHOT_KITTY=1`, o `shot.py` responde como um kitty; sem ela, as capturas continuam
   iguais às de um terminal comum. Passos novos: `press:ctrl`, `press:alt` (aperta e
   segura) e `release:ctrl`, `release:alt`, enviados como no kitty (`CSI 57442;5u`,
   `CSI 57442;1:3u`, e 57443 para o Alt). O pyte não conhece o liga e desliga do
   protocolo (`CSI > flags u`, `CSI < n u`) e imprimiria parte deles: o `shot.py` os
   tira antes. `SHOT_COLS` muda a largura (para a conferência em 80 colunas).
2. **Cantos arredondados** (`╭╮╰╯`, dos temas modernos) desenhados como linhas,
   como as outras bordas, em vez de letras da fonte.

## Capturas novas

- **`demo-menus-nord.gif`:** segurar Ctrl na árvore (menu CTRL), soltar, segurar Alt
  (menu ALT), soltar; o mesmo na lista de arquivos; Ctrl+G e o menu GIT.
- **Conferência dos menus** (fora do repositório, em `~/.cache/xylem-screenshots/`):
  DIR, FILE, CTRL e ALT na árvore e na lista, GIT, REMOTE, ARCHIVE, VIEW e TERMINAL,
  em 120 e em 80 colunas, para achar texto cortado ou fora do lugar.

## Ordem

1. O protocolo do kitty e os cantos no `shot.py`.
2. A conferência dos menus; o que estiver errado é corrigido no Xylem, com teste.
3. O GIF dos menus, na página inicial do site e no README.
4. Refazer todas as imagens e GIFs com `run.sh`.

## O que a conferência achou (2026-11-03) e o que foi decidido

- **Menus cortados no meio** em 120 colunas (DIR, ALT na lista, VIEW) e em quase
  todos em 80. Decisão do autor: **nunca cortar no meio**. Cada entrada de menu tem
  um marcador de fim (`entry_end`), e `fit_menu` tira entradas inteiras pelo fim
  até a linha caber; as dicas longas se dividem nos ` · `. Linha com mensagem
  acrescentada não é mexida.
- **"Sort usage (W) duplicates (Y)"** parecia um comando só: agora
  "Sort · usage (W) · duplicates (Y)".
- **Dentro de um arquivo compactado** (só leitura), os menus CTRL e ALT ofereciam
  Attributes, Delete, Move, Rename, Prune e eXecute, que ali são recusados. Decisão
  do autor: **tirar do menu** nessa situação.
