# O mouse

O Xylem é feito para o teclado, e o mouse fica desligado até você ligá-lo:
**F12**, página 2, **R** (ou `mouse = true` nas `[options]` do
[arquivo de configuração](configuration.md)), e depois **F3** para salvar. Ele
funciona na hora.

O mouse nunca tem comandos próprios. Um clique escolhe uma linha, ou vale pela
tecla escrita onde você clicou, e essa tecla faz o que sempre faz: Delete
continua perguntando, e um comando que não pode rodar ali continua sendo
recusado.

## O que ele faz

| Onde | Clique | Duplo clique | Roda |
|---|---|---|---|
| A árvore | escolhe o diretório; a janela de arquivos o acompanha | Enter: para os seus arquivos | move o cursor 3 diretórios |
| A marca antes de uma pasta | `+` a abre, `?` a lê e abre, o `─` de uma aberta a fecha | | |
| A janela de arquivos | escolhe o arquivo e passa para a janela de arquivos | View, no visualizador do próprio Xylem | move o cursor 3 arquivos (uma linha nos modos em colunas) |
| A barra de rolagem da borda direita | acima ou abaixo do marcador: uma página; arraste o marcador para ir a qualquer ponto | | |
| O outro lado de uma [tela dividida](split.md) | torna-o a janela ativa, e depois escolhe | | move o cursor daquele lado; o ativo fica |
| O [terminal integrado](terminal.md), o editor na moldura | para o programa, se ele pediu o mouse (mc, vim, htop, tmux); senão nada | | para o programa, se ele pediu; senão volta pelas linhas que saíram da tela, como Shift+PgUp |
| A árvore acima do terminal integrado | volta ao Xylem, como Alt+X | | |
| O rodapé | a tecla da entrada: `Copy` é C, `Esc cancel` é Esc, `Yes` responde à pergunta | | |
| F12 | uma opção muda, como com a sua letra (nada é salvo antes de F3); um tema da lista (F4) é pré-visualizado | o tema é escolhido | move a seleção |
| Favoritos, a caixa de busca, Usage, Duplicates | escolhe a linha | o seu Enter | 3 linhas |
| O histórico de um campo de digitação (↑) | põe essa entrada no campo | | 3 linhas |
| O texto de um campo de digitação | o cursor vai para lá | | |
| Uma tabela no visualizador | escolhe a célula | o seu Enter: o registro | 3 linhas; Shift+roda, uma coluna |
| O visualizador, a ajuda | um link da ajuda é seguido | | rola, como ↑ e ↓; Shift+roda rola o visualizador para os lados |
| Onde quer que um menu faça sentido | o botão direito o abre (abaixo) | | |

Um duplo clique nunca abre um arquivo com um programa do sistema: um arquivo
que você não criou pode ser hostil, e um clique por engano não deve executá-lo.
Open (`O`) continua lá.

Arrastar com o botão esquerdo move o cursor ao longo de uma lista; passando da
borda de cima ou de baixo, a lista rola uma linha a cada movimento.

## O menu do botão direito

O botão direito leva o cursor ao item sob o ponteiro, como um clique, e abre ao
lado do ponteiro um menu do que se pode fazer com ele. A tecla **Menu** do
teclado, ou **Shift+F10**, abre o mesmo menu para o item sob o cursor, sem
mouse.

| Sob o ponteiro | O menu |
|---|---|
| Um arquivo | View, Edit, Open with the system, Info; Tag ou Untag; Copy, Move, Rename, Delete, Attributes, New date, Yank the path |
| Um arquivo, com arquivos marcados | o mesmo, e um grupo para os marcados: Copy, Move, Rename (mask), Delete, View, Yank, Untag all |
| Um diretório da árvore | Files, Log, Relog, Expand all, Collapse, Release; Branch, Showall; Make a directory, Rename, Delete, Prune, Attributes; Tag its files, Untag; Terminal here, Usage, Duplicates, Bookmark it, Yank the path |
| A janela de arquivos, depois dos seus arquivos | Filespec, Sort, File display; Tag all, Untag all, Invert tags |
| O visualizador | Search, Next match, Go to line, e as teclas do seu modo (Wrap, Line numbers, Hex, SQL filter...), Edit, Yank the path, Close |

Cada linha mostra a sua tecla à direita, como a sua configuração a define, de
modo que o menu também é um jeito de aprender as teclas. O que não pode rodar
ali fica de fora: dentro de um arquivo compactado não há Delete, Move nem
Rename. Uma linha roda como a sua tecla rodaria: Delete continua perguntando
antes.

↑↓ escolhem, Enter executa, Esc fecha; um clique fora do menu apenas o fecha.

## Marcar

| Gesto | O que faz |
|---|---|
| Botão do meio, ou Ctrl+clique, em um arquivo | marca ou desmarca o arquivo; o cursor fica onde está |
| O mesmo em uma pasta da árvore | escolhe a pasta e marca os seus arquivos, ou os desmarca quando todos estavam marcados |
| Shift+clique em um arquivo | marca todos os arquivos do cursor até aquele, e o cursor vai para lá |

Muitos terminais guardam o Shift+clique para a sua própria seleção; neles o
botão do meio ou o Ctrl+clique marcam cada arquivo.

## O foco, os menus e as perguntas

A roda move a lista sob o ponteiro, não a que tem o foco: a janela de arquivos
pequena sob a árvore rola sem sair da árvore.

Nos menus Ctrl e Alt (mantidos pressionados, ou abertos com F4), uma palavra
vale pela sua letra com Ctrl ou Alt: um clique em `Copy` no menu Ctrl copia os
arquivos marcados.

Enquanto uma pergunta, um campo de digitação ou um trabalho em andamento está
na tela, as janelas por trás não recebem cliques: só as suas próprias entradas
no rodapé.

## Selecionar texto no terminal

Com o mouse ligado, o terminal manda os cliques para o Xylem, então arrastar
não seleciona mais texto na tela. Segure **Shift** enquanto arrasta (no xterm,
GNOME Terminal, Konsole, kitty, WezTerm, Windows Terminal), ou **Option** no
iTerm2. Para copiar o caminho de um arquivo, o Yank (`Y`) não precisa de
mouse.

## Terminais

O Xylem pede os cliques e os movimentos com um botão pressionado, no formato
SGR que todo terminal atual conhece; nunca os movimentos do ponteiro sozinhos,
que mandariam um fluxo de eventos à toa, um peso sobre SSH. Um programa no
terminal integrado ou no editor na moldura recebe o mouse no formato que pediu,
com os cantos do seu próprio painel: ele nunca chega ao resto da tela, nem ao
seu terminal. Um arrasto que sai do painel fica com o programa, preso na sua
borda. Um programa executado pelo eXecute ou pelo Edit pode desligar o mouse ao
terminar; o Xylem o liga de novo quando recebe a tela de volta.
