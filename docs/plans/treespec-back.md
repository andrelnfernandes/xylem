# Spec: Ctrl+← e Ctrl+→ no Treespec

Origem: conversa de 2026-11-14. O `/` (Treespec) abre com o caminho atual, como deve
(decisão do autor: continua assim). O autor pediu um jeito de recuar uma pasta de cada
vez e de voltar ao que tinha recuado.

**Estado: aprovado pelo autor em 2026-11-14.** Na branch `history-list`, com a lista do
histórico ([history-list.md](history-list.md)).

## O que muda

- **Ctrl+←** tira a última pasta do caminho: `/home/andre/projetos/xylem/docs/` vira
  `/home/andre/projetos/xylem/`, e o cursor da árvore vai para `xylem`. Com um nome
  pela metade (`…/xylem/do`), tira o pedaço. Na raiz (`/`, `C:\`), não faz nada.
- O caminho fica com a barra no fim, como o prompt abre: o que se digitar em seguida
  procura dentro da pasta.
- **Ctrl+→** devolve, um passo de cada vez, o que Ctrl+← tirou, até o caminho de antes
  do primeiro Ctrl+←.
- **Digitar, apagar, Tab ou `/`** depois de recuar começa um caminho novo: Ctrl+→ não
  tem mais o que devolver.
- Recuar além da raiz da árvore vale como digitar esse caminho: Enter o abre como Log.
- O rodapé do Treespec mostra `Ctrl+←→ folder up/back`.

## Onde

- A pilha do que foi tirado fica no `App` (`treespec_back`), vazia a cada `/`.
- As teclas são tratadas em `treespec_key` (`src/app/actions.rs`), antes do
  `LineInput`, que hoje trata Ctrl+← como ←.

## Modelo de ameaça

Nada de novo lido do disco: o caminho é cortado como texto (`Path::parent`), e quem o
segue na árvore é o mesmo `treespec_follow`, com as mesmas regras de hoje.

## Testes

- Ctrl+← duas vezes e Ctrl+→ duas vezes voltam ao caminho do começo, com o cursor da
  árvore em cada pasta.
- Digitar depois de recuar esvazia a pilha.
- Na raiz, Ctrl+← não muda nada.
