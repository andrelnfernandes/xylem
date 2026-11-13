# O terminal integrado (Alt+X)

Não existe no XTree nem no ZTreeWin: o seu shell (`$SHELL`) rodando dentro da
tela do Xylem, na parte inferior, com a árvore e os arquivos ainda visíveis
acima dele.

- **Alt+X** abre o terminal e volta ao Xylem. O shell continua rodando enquanto
  está oculto, mantendo seu histórico e o que estava na tela; `exit` o fecha.
- **Alt+Z** abre o terminal em tela cheia; lá dentro, alterna entre tela cheia
  e o painel. Alt+X depois o reabre do jeito que estava.
- **A árvore acompanha o shell:** um `cd` no terminal move o cursor da árvore
  para o mesmo diretório (Linux).
- **O shell acompanha a árvore:** quando você entra no terminal a partir de
  outro diretório, ele vai até lá. Isso só acontece quando o shell está
  aguardando no prompt; com um programa em execução (vim, less...), nada é
  digitado.
- **`$XY_TAGGED`** contém os arquivos marcados, definidos quando você entra no
  terminal. É um array: no bash, `"${XY_TAGGED[@]}"`
  (`cp "${XY_TAGGED[@]}" ~/backup`); no zsh e no fish, `$XY_TAGGED`. Com outros
  shells (sh, dash) há apenas o `cd`.
- **Alt+O** digita os arquivos marcados no cursor, entre aspas e relativos ao
  diretório do shell (`wc -l 'a.txt' 'b c.txt'`). A partir da árvore ou da
  janela de arquivos, abre o terminal e os digita.
- **Shift+PgUp/PgDn** rolam de volta pelo conteúdo que já saiu da tela.
- Ao sair do terminal, os diretórios abertos são relidos.
- **Sair do Xylem encerra o shell** e o que estiver rodando nele. Se houver um
  programa em execução lá (por exemplo, `vim`), Q e Alt+Q perguntam antes,
  mesmo com `confirm_quit` desativado (Linux e macOS).
- Programas que usam a tela inteira (vim, htop, less) funcionam dentro dele;
  todas as teclas vão para eles, exceto Alt+X, Alt+Z e Alt+O. É por isso que F1
  e Alt+H não abrem esta ajuda dentro do terminal.

## Vários terminais lado a lado (tmux)

O terminal é um shell só, mas um multiplexador como o **tmux** dentro dele dá vários,
lado a lado ou um sobre o outro: rode `tmux` e divida como de costume (`Ctrl+B %`,
`Ctrl+B "`). As teclas do tmux chegam a ele, porque o Xylem só guarda Alt+X, Alt+Z e
Alt+O; Alt+Z dá a tela inteira aos painéis, e o tmux os redistribui. Conferido com o
tmux; outros multiplexadores (GNU screen, zellij) não foram testados.

Com o tmux rodando:

- **A árvore só acompanha o shell que o Xylem abriu,** não os `cd` feitos nos painéis
  do tmux, e entrar no terminal a partir de outro diretório não muda mais o diretório
  (há um programa rodando, então nada é digitado).
- **Alt+O** digita os arquivos marcados no painel que está com o foco; o `$XY_TAGGED`
  é o de quando o tmux começou.
- **A sessão continua depois do Xylem.** Sair do Xylem pergunta antes, porque há um
  programa rodando, e encerra o shell, mas o tmux mantém a sessão em segundo plano, como
  sempre faz: `tmux attach` a traz de volta, e `tmux kill-session` (ou `exit` em todos
  os painéis) a encerra.
- **Memória:** cada painel é mais um shell, rodado pelo tmux; o Xylem continua com uma
  só tela para o terminal, com as suas 2.000 linhas de rolagem.

## Histórico próprio

Os comandos digitados nele vão para um histórico próprio do shell, separado dos
seus outros terminais: `~/.local/state/xylem/bash_history` (`zsh_history`...),
por meio de `HISTFILE`; no fish, a sessão `xylem`. Se o seu arquivo rc definir
`HISTFILE` incondicionalmente, ele prevalece. Desative na tela F12 (página 3)
ou com `terminal_history = false`.

## Teclas e cores

As teclas podem ser alteradas em `[keys.any]` (`terminal`, `terminal_full`,
`terminal_tagged`): Alt+T, por exemplo, abre o menu "Terminal" em alguns
terminais. As cores são escolhidas no F12 (página 2) ou em `terminal_theme`:
`native` (as do terminal em que o Xylem roda), `xylem` (o fundo e o texto do
tema do Xylem), `solarized`, `gruvbox`, `nord`, `dracula` ou `one_dark`.

Para entrar no terminal e mudar de diretório, o Xylem grava um pequeno script
em uma pasta própria (em `~/.cache/xylem/`, legível apenas por você) e digita
` . script` no prompt; o espaço inicial mantém a linha fora do histórico no
bash (`ignorespace`) e no zsh (`HIST_IGNORE_SPACE`). Dentro do terminal,
`XY_TERMINAL=1` fica definido, para prompts e scripts que queiram saber; o
terminal em uma [máquina remota](remote.md) também define essa variável (útil,
por exemplo, para impedir que um `~/.bashrc` inicie o tmux lá).
