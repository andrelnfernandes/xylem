# Estudo e plano: o mouse no Xylem

Origem: conversa de 2026-11-18. O autor pediu um estudo completo sobre usar o mouse,
ligado ou desligado na tela do F12, pensando em todos os lugares onde ele ajuda
(inclusive a roda e o botão direito), e deixou as decisões comigo para aprovar depois.

**Estado: aprovado pelo autor em 2026-11-18, com todas as decisões do fim; nada
implementado ainda.** As marcas **(?)** ficaram como registro de onde havia
alternativa. Tudo na branch `mouse`, com **um PR só no fim**.

> Muda uma decisão antiga: o mouse estava no BACKLOG e no [roadmap](roadmap.md) entre
> o que ficou de fora por decisão. Saiu de lá na aprovação e entrou como frente nova.

## A ideia em uma frase

O mouse **não inventa comandos**: ele aponta. Um clique escolhe a linha (a árvore, a
lista, um item de menu) e, quando faz algo, faz pelo mesmo caminho do teclado (um
`Command` ou uma tecla): as recusas (`refusal`), as confirmações, a lixeira e o `Oops`
continuam valendo. O teclado continua sendo o jeito principal; o mouse é opcional e
**vem desligado**.

## Ligar e desligar (F12)

| Onde | O quê |
|---|---|
| `config.toml` | `[options] mouse = false` (padrão), com comentário no `--default-config` |
| F12, página 2 "Display and sorting" | `R  Mouse (click, wheel and right-click menu) .... No` |
| Quando vale | ao salvar (F3), como as outras opções; o tema é a única com prévia |

**Desligado por padrão (?)**, por três motivos: (1) com o mouse capturado, o terminal
deixa de selecionar texto com o arrastar simples; passa a ser Shift+arrastar (xterm,
GNOME Terminal, Konsole, kitty, WezTerm, Windows Terminal) ou Option+arrastar
(iTerm2), e quem copia nomes da tela estranharia; (2) o público do XTree é de
teclado; (3) não muda nada para quem atualiza. A alternativa é vir ligado, como no
mc e no htop, e anunciar a Shift+arrastar.

Uma opção só **(?)**. Velocidade da roda (3 linhas) e tempo do duplo clique (400 ms)
ficam fixos no código; viram opção se alguém pedir.

## Como o terminal manda o mouse

- **Unix:** em vez do `EnableMouseCapture` do crossterm, que liga também o `?1003`
  (todo movimento, uma enxurrada de eventos, cada um um redesenho, pesado por SSH) e o
  `?1015` (urxvt, obsoleto), o Xylem escreve só `?1000h` (cliques), `?1002h` (arrasto
  com botão apertado) e `?1006h` (SGR: colunas acima de 223, sem bytes estranhos). O
  parser do crossterm entende igual. Desligar: os mesmos com `l`.
- **Windows:** o `EnableMouseCapture` do crossterm (modo do console). Ele reporta
  movimento também; o laço descarta `Moved` sem redesenhar.
- **Sem passar o mouse por cima (hover)**: sem o `?1003` não há; não faz falta.

### O ciclo de vida, como o do teclado (`Keyboard` em `src/main.rs`)

Um `struct Mouse { on: bool }` ao lado do `Keyboard`, com:

- `sync(app)`: no começo de cada volta do laço, liga ou desliga conforme
  `app.options.mouse`. Muda na hora em que o F3 salva.
- `restore()`: antes de entregar o terminal a outro programa (`run_external`) e na
  saída.
- `resume()`: na volta de `run_external`. **Sempre reenvia**, porque o programa que
  rodou (vim, less) pode ter desligado o mouse ao sair.
- No gancho de pânico, desliga antes do ratatui restaurar o terminal; senão o shell
  fica recebendo `^[[<0;12;5M` a cada clique.

### O laço (`run` em `src/main.rs`)

Hoje só `Event::Key` passa. Entra `Event::Mouse(m) => app.handle_mouse(m, Instant::now())`.
`MouseEventKind::Moved` (só no Windows, ou de algum terminal teimoso) volta ao `poll`
sem redesenhar. Mouse com a opção desligada é ignorado também em `handle_mouse` (defesa:
um terminal que ficou com o mouse ligado por outro programa).

## O mapa de cliques (`HitMap`)

O ponto central do desenho. A tela é desenhada em `ui::draw` com `Layout`, rolagens
(`visible_window`), colunas (`draw_file_columns`) e rodapés encolhidos (`fit_menu`);
refazer essas contas no tratamento do mouse duplicaria código e sairia de sincronia.
Então **quem desenha anota**: cada função de desenho registra, no quadro que desenhou,
o que há ali.

```rust
// src/mouse.rs (novo)
pub struct HitMap { spots: Vec<(Rect, Spot)> }   // o último registrado fica por cima
pub enum Spot {
    TreeRows   { pane: usize, inner: Rect, offset: usize, len: usize },
    FileRows   { pane: usize, inner: Rect, offset: usize, len: usize },
    FileGrid   { pane: usize, inner: Rect, top: usize, cols: usize, cell_w: u16, len: usize },
    Panel      { pane: usize, window: Focus },   // borda, título, área vazia
    Scrollbar  { list: ListRef, track: Rect, len: usize, page: usize },
    Key(KeyEvent),                               // uma entrada de rodapé, uma opção de pergunta
    Rows       { list: ListRef, inner: Rect, offset: usize, len: usize }, // as outras listas
    ViewerBody { inner: Rect },
    TableCells { inner: Rect, /* colunas visíveis e larguras */ },
    HelpLink(usize),
    PromptText { area: Rect, start: usize },
    Terminal   { inner: Rect },
    ContextMenu{ inner: Rect, len: usize },
    Outside,                                     // fundo de um modal: fecha ou ignora
}
```

- Fica em `App::hits`, limpo no começo de `ui::draw`. As funções que recebem
  `&mut Pane` recebem também `&mut HitMap` (campos diferentes do `App`, o borrow
  checker aceita).
- **Vale o que o usuário viu:** o mapa é do último quadro desenhado, que é o que estava
  na tela quando ele clicou. O laço redesenha depois de cada evento, então um arrasto
  sempre usa o quadro novo.
- Coordenadas de fora da tela (terminal redimensionado entre o clique e a leitura)
  caem em nenhum `Rect` e não fazem nada.
- `ListRef` nomeia as listas que não são árvore/arquivos: Settings, Bookmarks,
  Picker, HistoryList, SearchBox, Usage, Duplicates, SheetList, Structure,
  ContextMenu.

### O rodapé vira botões, de graça

Todo rodapé passa por `fit_menu`, e as entradas já são reconhecíveis: o `key()` tem
estilo próprio e cada entrada termina num `entry_end()`/`word_end()`. Uma função só,
`footer_spots(line, y, mods)`, chamada onde hoje se faz `fit_menu` + `Paragraph`,
percorre os spans **já encolhidos** e registra cada entrada como `Spot::Key`:

- palavra de `menu_words` (`Copy`, `eXecute`, `Yes`): a letra destacada, com o
  modificador do menu em que está (Ctrl ou Alt no menu do F9/Ctrl/Alt);
- entrada de `menu_line` (`F3 save`, `Esc cancel`, `Alt+X back to Xylem`): o rótulo
  passa por `KeySpec::parse` (`src/keymap.rs`), que já entende `F3`, `Esc`,
  `ctrl+r`; acrescenta `↑` `↓` `←` `→`, e rótulos com `/` (`PgUp/PgDn`,
  `Shift+PgUp/PgDn`) viram duas metades, cada uma a sua tecla (o modificador vale
  para as duas);
- rótulo que é faixa (`A-Z`, `1-4`) ou dica sem tecla: não é clicável.

O clique numa entrada chama `handle_key` com a tecla. Assim **toda pergunta, todo
prompt e todo trabalho em andamento** ficam clicáveis sem código próprio: `Yes`/`No`
das confirmações, `Esc cancel` de cópia, Log e busca, `F3 save` do F12, `F9 next
menu`, as opções do conflito de cópia (`Yes No All Skip`).

## O que faz cada botão, em cada lugar

Convenções em todo o programa:

| Gesto | Faz |
|---|---|
| Roda | move o cursor 3 linhas **da lista sob o ponteiro** (não da que tem o foco) |
| Shift+roda, roda lateral | rola para os lados, onde há (viewer, tabela) |
| Clique | põe o cursor ali; se for outra janela ou outro lado do split, ela ganha o foco |
| Duplo clique | o "Enter" daquele lugar, nunca um programa externo |
| Botão direito | menu de contexto do item sob o ponteiro (etapa 3) |
| Botão do meio | marca ou desmarca o item, sem mover o cursor |
| Ctrl+clique | o mesmo que o botão do meio |
| Shift+clique | marca do cursor até ali (se o terminal deixar passar; muitos usam Shift para a seleção nativa) |
| Arrastar com o esquerdo | o cursor segue o ponteiro; na borda de cima ou de baixo, a lista rola |
| Clicar/arrastar a barra de rolagem | o clique no trilho pula uma página; arrastar o `┃` leva à proporção |

**Roda move o cursor, não só a vista (?).** O `visible_window` já prende a vista no
cursor; e no Xylem o cursor da árvore é o que escolhe a janela de arquivos, então rolar
a árvore mostra os arquivos de cada diretório que passa, como as setas. É o que o mc
faz. A alternativa (rolar a vista e o cursor ser puxado só quando sai da tela, como o
Ctrl+E do Vim) pede mexer no `visible_window` e deixa o cursor "preso" num canto.

**Duplo clique nunca abre com programa externo (segurança).** Todo arquivo é tratado
como hostil; um duplo clique acidental num `.desktop`, `.lnk` ou `.html` não pode ir
para o `xdg-open`/`start`. O Open (`O`) fica no teclado e no menu de contexto.

Duplo clique é detectado no Xylem (o crossterm não informa): mesmo botão, mesma
célula, até 400 ms. `handle_mouse` recebe o `Instant` de fora, para os testes.

### Janela principal (árvore e arquivos)

| Onde | Clique | Duplo clique | Roda | Outros |
|---|---|---|---|---|
| Linha da árvore | seleciona o diretório (a janela de arquivos acompanha) | Enter: vai à janela de arquivos | 3 linhas | meio/Ctrl: marca/desmarca os arquivos do diretório **(?)** |
| Marca da árvore (`+`, `?`, `─` de um aberto) | `+`: expande; `?`: Log; aberto: recolhe | — | — | — |
| Arquivo (lista ou colunas) | seleciona; se o foco estava na árvore, passa à janela de arquivos | View (o viewer interno) | 3 linhas (colunas: uma fileira) | meio/Ctrl: Tag; Shift: intervalo |
| Área vazia, borda, título | dá o foco àquela janela | — | rola aquela janela | — |
| Outro lado do split | `active` passa a ser ele, depois age como acima | | | |
| Árvore acima do terminal (Alt+X) | sai do terminal (o mesmo que Alt+X) e age | | | |
| Cabeçalho, painel de estatísticas | nada **(?)** | | | |
| Rodapé | a tecla da entrada (ver acima) | | | |

Clique fora do que existe (abaixo do último arquivo) põe o foco na janela sem mover o
cursor.

### Telas e listas que têm cursor

| Tela | Clique | Duplo clique | Roda |
|---|---|---|---|
| F12 (Settings) | seleciona **e muda** a opção, como a letra dela **(?)** (nada se grava até o F3) | — | sobe/desce a seleção |
| F12, lista de temas (F4) | escolhe e mostra a prévia | Enter | prévia de tema em tema |
| F12, campo de texto (editor…) | põe o cursor do texto ali | — | — |
| Favoritos (F11 / `''`) | seleciona | pula | 3 linhas |
| Picker de pastas (F2 no prompt) | como a árvore | escolhe (Enter) | 3 linhas |
| Histórico do prompt (↑) | escolhe a linha (Enter) | — | 3 linhas |
| Caixa de busca | seleciona o resultado | Enter | 3 linhas |
| Usage (Alt+U), Duplicados | seleciona | Enter | 3 linhas |
| Menu do F9/Ctrl/Alt | é o rodapé: a palavra é a tecla | | |
| Prompt (linha de digitação) | põe o cursor do texto na coluna | seleciona a palavra? **não** | — |
| Help (F1) | link: segue o link | — | 3 linhas |
| Splash | fecha, como a primeira tecla | | |

Com um modal aberto (pergunta, prompt, trabalho em andamento), clique fora dele e
fora do rodapé **não faz nada (?)**: não muda o cursor que o prompt vai usar. A
alternativa sedutora, clicar num diretório da árvore para preencher o destino do Copy,
já existe melhor no F2 (picker), que fica clicável.

### Viewer

| Modo | Clique | Duplo clique | Roda |
|---|---|---|---|
| Texto, Markdown, hex | nada | nada | 3 linhas; Shift: 8 colunas |
| Tabela | seleciona a célula | o Enter da tabela (o registro) | 3 linhas; Shift: uma coluna |
| Lista de planilhas, estrutura, registro | seleciona | Enter | 3 linhas |
| Autoview (F7) | a lista à esquerda como a de arquivos; o viewer à direita só rola | | |

Seleção de texto para copiar fica com o terminal (Shift+arrastar); o viewer não
copia texto por conta própria nesta fase.

### Terminal embutido (Alt+X/Alt+Z) e editor no quadro

- **O programa pediu mouse** (o `vt100` do Xylem informa
  `screen.mouse_protocol_mode() != None`: mc, vim, htop, tmux): o evento vai para ele,
  com as coordenadas relativas ao painel e na codificação que ele pediu
  (`mouse_protocol_encoding`: X10, UTF-8 ou SGR). Coordenadas presas ao painel.
- **Não pediu** (o shell no prompt): a roda percorre o histórico da tela, o mesmo que
  Shift+PgUp/PgDn; cliques não fazem nada.
- Clique na árvore acima do painel: volta ao Xylem (Alt+X).

### Menu de contexto (botão direito)

Um menu flutuante perto do ponteiro (preso dentro da tela), com borda, que se usa com
o mouse **e com o teclado** (↑↓, Enter, Esc). O botão direito primeiro põe o cursor no
item, depois abre o menu: o comando escolhido age no item do cursor, como se a tecla
tivesse sido apertada. Cada linha mostra o atalho (`keymap.key_for(cmd)`), então o menu
também ensina as teclas.

As linhas são `Command`s, filtrados pelo `refusal` (num archive não aparece Delete; no
remoto não aparece Edit), do mesmo jeito que os rodapés.

| Sob o ponteiro | Itens |
|---|---|
| Arquivo | View, Edit, Open, Info · Tag/Untag · Copy, Move, Rename, Delete, Attributes, New date · Yank path |
| Arquivo, com outros marcados | o de cima, mais um grupo "N tagged": Copy, Move, Delete, Rename (mask), View, Yank |
| Diretório na árvore | Files (Enter), Log/Relog, Expand all, Collapse, Release · Branch, Showall · Makedir, Rename, Delete, Prune, Attributes · Tag all, Untag all · Terminal here, Usage, Duplicates, Bookmark here · Yank path |
| Área vazia da lista | Filespec, Sort, File display · Tag all, Untag all, Invert |
| Viewer | Search, Go to line, Wrap, Hex, Line numbers, Close (o que valer no modo) |

Também abre com a tecla **Menu** do teclado e com **Shift+F10 (?)** (o padrão de
acessibilidade do Windows e do GTK; hoje o F10 é Quit e o Alt+F10 é Settings, Shift+F10
está livre). Isso deixa o menu testável e útil sem mouse.

Clique fora do menu só o fecha **(?)** (não passa o clique adiante; evita "fechei o
menu e sem querer mudei o cursor").

### O que fica de fora, de propósito

- **Arrastar e soltar** (arquivo sobre um diretório para copiar/mover). Um arrasto
  sem querer moveria arquivos; e copiar e mover já têm prompt, picker e histórico. Se
  um dia entrar: soltar só **abre o prompt do Copy** com o destino preenchido, nunca
  age sozinho.
- **Redimensionar** as janelas arrastando a divisa: as proporções são fixas hoje.
- **Hover** e dicas ao passar o ponteiro.
- **Seleção de texto própria** no viewer.
- Clicar no título para mudar a ordenação, no cabeçalho, no painel de estatísticas.

## Segurança

- A entrada do mouse vem do terminal, não de arquivos; os arquivos hostis não geram
  eventos. O que importa é o mouse nunca **encurtar** uma confirmação: o clique em
  `Yes` é a mesma tecla `y`, o Delete continua perguntando, o duplo clique não executa
  nem abre fora.
- Toda coordenada é testada contra um `Rect` do último quadro; índice calculado
  (`offset + linha`) é conferido contra `len` antes de usar.
- O programa do terminal embutido (até um shell remoto) só controla o mouse dentro do
  próprio painel: a codificação é a do `vt100` do Xylem, nunca repassada ao terminal de
  verdade.
- Desligar no pânico e na saída; religar depois de programa externo.

## Arquivos que mudam

| Arquivo | O quê |
|---|---|
| `src/mouse.rs` (novo) | `HitMap`, `Spot`, `footer_spots`, rótulo → tecla, duplo clique, codificação X10/UTF-8/SGR para o PTY |
| `src/main.rs` | `struct Mouse` (sync/restore/resume/pânico); `Event::Mouse` no laço; `Moved` sem redesenho |
| `src/config.rs` | `mouse: bool` em `Options` (padrão, `to_toml`, comentário do `--default-config`, teste de parse) |
| `src/settings.rs` | `Opt::Mouse`, letra `R` na página 2; `click(i)` e `wheel(n)` para a tela |
| `src/app/mouse.rs` (novo) | `App::handle_mouse`: a mesma ordem de modais do `handle_key` (`src/app/commands.rs`); seleção por índice; menu de contexto |
| `src/ui.rs` | registrar os `Spot`s em `draw_tree`, `draw_files`, `draw_file_columns`, `draw_scroll_thumb`, rodapés, listas, viewer, help; desenhar o menu de contexto |
| `src/pane.rs` | `select_row(i)` e `select_file(i)` que façam o mesmo que as setas fazem ao chegar ali (Git, autoview, janela de arquivos) |
| `src/term.rs` | `send_mouse(ev, origin)` conforme o modo pedido pelo programa |
| `src/help.rs`, `src/viewer.rs`, `src/viewer/table.rs`, `src/picker.rs`, `src/search_box.rs`, `src/input.rs` | clique/roda das telas próprias |
| `docs/manual/mouse.md` (novo), `configuration.md`, `index.md`, `releases.md` | só em inglês (as traduções depois, numa tarefa à parte) |
| `BACKLOG.md`, `docs/plans/roadmap.md` | (feito na etapa 0) marcar como feito ao terminar |

## Etapas

Todas na branch `mouse`, com **um PR só no fim** (decisão do autor). Cada etapa termina
com fmt, clippy e testes passando, para a branch poder ser revisada a qualquer momento.

### Etapa 0: preparar (este plano)

- Plano e decisões aprovados (2026-11-18). (feito)
- BACKLOG e roadmap: o mouse saiu do que ficou de fora e entrou como frente. (feito)

### Etapa 1: ligar, rolar, apontar, clicar no rodapé (~1 semana) (feita, 2026-11-18)

Como ficou, onde mudou em relação ao desenho acima:

- O mapa é preenchido por um `thread_local` em `ui.rs` (`hit`, `render_footer`), que
  vai para `App::hits` no fim de `ui::draw`: as funções de desenho não ganharam um
  `&mut HitMap` cada; só árvore e arquivos ganharam o dono (`Owner::Main(i)` ou
  `Owner::Picker(i)`).
- As teclas do rodapé são reconhecidas por uma marca invisível no estilo de
  `ui::key` (`mouse::KEY_MARK`, um modificador retirado), e não pela cor do tema:
  os testes trocam o tema ao mesmo tempo, e uma mensagem em negrito não pode virar
  tecla.
- O título de uma linha (`CONFIG`, `HELP`) não faz parte da primeira entrada; uma
  letra sozinha (`C` no viewer) mantém a maiúscula.
- Usage e Duplicados: por ora a roda vale como ↑↓ quando estão na frente; o clique
  só ativa o lado do split (as linhas próprias ficam para a etapa 2).
- Texto da opção no F12: "Mouse (wheel, clicks, footer keys; Shift+drag selects
  text)"; muda quando o menu de contexto entrar.
- O `clippy --target x86_64-pc-windows-gnu` não rodou (o target não está instalado
  nesta máquina); o código só do Windows é o `EnableMouseCapture`.


- Opção `mouse` no config e no F12; `struct Mouse` com sync/restore/resume/pânico.
- `Event::Mouse` no laço; `HitMap` e `App::handle_mouse` com a ordem dos modais.
- Roda e clique na árvore, na lista (uma coluna e colunas), no split, no autoview, no
  painel acima do terminal.
- Rodapés clicáveis (`footer_spots`): todas as perguntas e trabalhos em andamento.
- Roda no viewer (todos os modos) e no help; clique em link do help.
- Manual: `mouse.md` com a tabela dos gestos e o Shift+arrastar para copiar texto.

Testes: rótulo → tecla (`F3`, `Esc`, `Alt+X`, `PgUp/PgDn`, `↑↓`, `A-Z` não);
`footer_spots` num rodapé encolhido (a palavra cortada continua com sua letra); com
`TestBackend`, desenhar e clicar na linha N da árvore seleciona `rows[offset+N]`;
clicar no `Yes` de uma confirmação confirma; mouse com a opção desligada não faz nada;
coordenada fora da tela não faz nada. e2e: ligar no F12, salvar, conferir
`mouse_protocol_mode()` no `vt100` do teste, mandar `\x1b[<0;x;yM` e ver o cursor
mudar; abrir e fechar o editor externo e conferir que o modo voltou.

### Etapa 2: duplo clique, marcas, arrasto, listas próprias (~1 semana) (feita, 2026-11-18)

Como ficou:

- Duplo clique: mesmo botão e mesma célula até 400 ms (`mouse::DOUBLE_CLICK`); um
  terceiro clique recomeça. No rodapé, cada clique é uma tecla (não há duplo clique).
  No F12, o segundo clique de um duplo não muda a opção de volta.
- O "Enter" das listas próprias é a própria tecla Enter mandada a `handle_key`, então
  vale o que cada tela já faz; no tema (F4), o clique anda de tema em tema com ↑↓
  para a prévia acompanhar.
- Marcar com Ctrl+clique/botão do meio na árvore usa o `Tag`/`Untag` de sempre (marca
  ou desmarca os arquivos do diretório); na lista, inverte só o arquivo, sem mover o
  cursor (o `T` do teclado anda uma linha, por isso não é usado).
- Barra de rolagem: o clique acima ou abaixo do polegar é PgUp/PgDn; arrastar o
  polegar leva o cursor à mesma proporção da lista.
- Ficaram para depois (só roda, como ↑↓): a lista de planilhas, a estrutura e o
  registro do viewer, e a lista do picker de favoritos dentro de um prompt.


- Duplo clique (Enter do lugar; View no arquivo); marcadores `+`/`?`/`─` da árvore.
- Botão do meio e Ctrl+clique marcam; Shift+clique marca intervalo.
- Arrastar o cursor, com rolagem na borda; clique e arrasto da barra de rolagem.
- Clique nas listas próprias: F12 (opções e temas), favoritos, picker, histórico,
  caixa de busca, Usage, Duplicados, planilhas, estrutura; célula da tabela; cursor do
  prompt.

Testes: duplo clique com `Instant`s a 300 ms e a 500 ms; marcar com o meio não move o
cursor; Shift+clique marca exatamente o intervalo; arrasto além da borda rola; F12:
clicar numa opção muda o valor e não grava.

### Etapa 3: menu de contexto (~1 semana) (feita, 2026-11-18)

Como ficou:

- `src/app/context_menu.rs`: as linhas são `Command`s (filtrados por `refusal`) ou,
  no viewer, teclas dele (o viewer não tem `Command`s). O atalho de cada linha vem de
  `Keymap::key_in(cmd, focus)`, a tecla mais curta que de fato roda aquele comando na
  janela (respeita o `[keys]` do usuário).
- Com o menu aberto, ele recebe todas as teclas e todos os cliques: o mapa ganha um
  `Spot::Outside` sobre a tela inteira, e o clique fora só fecha.
- Shift+F10 é interceptado antes do mapa de teclas (o `KeySpec` ignora Shift, então
  viraria F10, Quit); F22 também, para terminais que mandam Shift+F10 assim.
- Pelo teclado, o menu abre ao lado da linha do cursor (`HitMap::row_of`).
- Usage e Duplicados não têm menu nesta etapa (a tecla Menu fica com eles).


- Widget `ContextMenu` (estado no `App`, desenho no `ui.rs`), aberto pelo botão
  direito, pela tecla Menu e pelo Shift+F10.
- Itens por lugar, filtrados por `refusal`; atalho de cada linha via `key_for`.

Testes: num archive não há Delete; no remoto não há Edit; com marcados aparece o
grupo; Enter num item roda o `Command` no item do cursor; Esc e clique fora fecham sem
mover nada.

### Etapa 4: terminal e editor no quadro (~3 dias) (feita, 2026-11-19)

Como ficou:

- `term::encode_mouse(ev, col, row, mode, encoding)`: botões 0/1/2, soltar 3 (no SGR,
  o próprio botão e `m`), arrasto +32, roda 64 a 67, Shift +4, Alt +8, Ctrl +16. O
  modo filtra: `Press` (X10, `?9`) só apertos e sem modificadores; `PressRelease`
  mais o soltar; `ButtonMotion` mais o arrasto; `AnyMotion` mais o `Moved`, que o
  Xylem nunca recebe (não pede o `?1003`). Coordenada presa em 223 no X10 e em 2015 no
  UTF-8.
- `Terminal::send_mouse(ev, col, row, lines)`: sem modo pedido, a roda move o
  `scrollback` (`lines` por vez) e o clique não faz nada; com modo, prende ao tamanho
  do painel, volta o `scrollback` a 0 e escreve no PTY.
- `Spot::Pty` no interior do painel (`draw_terminal_panel`, que serve ao terminal e ao
  editor no quadro). `handle_mouse` o testa antes do resto; um aperto ali abre um
  `Drag::Pty { area }`, e os arrastos e o soltar seguem para o programa mesmo fora do
  painel (presos à borda).
- O botão direito sobre o painel vai ao programa, não ao menu do Xylem.
- Fora do plano, anotado no BACKLOG para avaliar: a roda sobre less ou man (tela
  alternativa sem pedido de mouse) não faz nada; o xterm manda ↑↓ nesse caso.

Testes: as codificações SGR, padrão, UTF-8 e X10 com modos escritos num `vt100`
(clique em (5, 3) vira `\x1b[<0;6;4M`; coluna 300 presa); bash de verdade: a roda no
prompt muda o `scrollback`, e um `cat` depois de `?1000h?1006h` recebe o clique, preso
ao painel; o editor no quadro recebe clique, arrasto para fora (preso) e soltar.

- `Terminal::send_mouse` nas três codificações, coordenadas relativas e presas.
- Sem pedido do programa: roda no histórico da tela.

Testes: com o `vt100` em modo SGR (escrever `\x1b[?1000h\x1b[?1006h` no parser),
um clique em (5, 3) do painel vira `\x1b[<0;6;4M`; em X10, coluna 300 é presa; sem modo,
a roda muda o `scrollback`.

### Depois

Traduções do manual (`mouse.md` nas cinco línguas) e site, numa tarefa à parte; o
texto que anuncia entra junto com o release.

## A conferir na implementação

- O `Tag` move o cursor para baixo depois de marcar? Para o botão do meio é preciso um
  "marca sem andar".
- Qual é o "Enter" de cada lista própria (Usage, Duplicados, tabela) e se ele já está
  exposto como método ou só dentro do `*_key`.
- tmux com `mouse off` repassa o mouse que o Xylem pede? (Deve repassar; conferir.)
- Windows Terminal e conhost: o `EnableMouseCapture` desliga o QuickEdit; conferir que
  o Xylem o devolve ao sair.

## Decisões (aprovadas pelo autor em 2026-11-18)

1. **Desligado por padrão.** Alternativa: ligado, como mc e htop.
2. **Uma opção só** no F12 (`R` na página 2, "Display and sorting"). Alternativa: uma
   página "Keyboard and mouse", ou opções para a roda e o duplo clique.
3. **A roda move o cursor** (3 linhas), na lista sob o ponteiro, sem mudar o foco.
   Alternativa: rolar só a vista.
4. **Duplo clique num arquivo = View**, nunca Open. Na árvore, Enter (vai aos arquivos).
5. **Botão do meio e Ctrl+clique marcam; Shift+clique marca intervalo.**
6. **Botão direito = menu de contexto do Xylem**, com os atalhos ao lado; também na
   tecla Menu e no Shift+F10.
7. **Clique fora de um modal não faz nada**; clique fora do menu de contexto só o fecha.
8. **F12: o clique numa opção já muda o valor**, como a letra.
9. **Sem arrastar e soltar** arquivos; se vier, só abre o prompt.
10. **Unix com `?1000`/`?1002`/`?1006`** escritos à mão, sem o `?1003`; Windows com o
    crossterm.
11. **Branch `mouse`**, quatro etapas, **um PR só no fim**.
