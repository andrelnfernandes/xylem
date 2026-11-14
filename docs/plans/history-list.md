# Spec: o histórico dos prompts numa lista

Origem: conversa de 2026-11-14. O autor pediu que o ↑ num prompt mostre o histórico
numa lista, como um menu, em vez de trocar o texto da linha um item por vez. Com 100
itens guardados por prompt (`HISTORY_SIZE`), achar um lugar antigo no L era apertar ↑
dezenas de vezes.

**Estado: aprovado pelo autor em 2026-11-14.** Branch `history-list`, a partir do `main`.

- **Feito:** a lista em `LineInput` (`HistoryList`, `src/input.rs`), as teclas do app
  com a lista aberta e o Del salvo (`forget_history`, `src/app/prompts.rs`), o desenho
  (`draw_history_list`, `src/ui.rs`, por último, sobre a tela inteira), o rodapé, o
  manual em inglês (`execute.md`, seção "History", e as páginas que citavam o ↑) e as
  notas da 0.8.0.
- **Pendente:** o manual nas outras 5 línguas, com a tarefa de documentação (como as
  tabelas da 0.8.0).

## O que existe hoje

- `LineInput` (`src/input.rs`) guarda o histórico do prompt, do mais antigo ao mais
  novo. ↑ troca o texto pelo anterior, ↓ pelo seguinte, e passar do mais novo devolve o
  que estava digitado (`draft`).
- Cada tipo de prompt tem a sua chave (`PromptKind::history_key`); Log, Copy e Move
  dividem uma (`PLACES`), com a lista antiga de Copy e Move (`destination`) na frente.
- O Batch do viewer usa ↑↓ para os templates e PgUp/PgDn para o histórico, que ele
  passa ao `LineInput` como ↑↓.

## O que muda

**↑ abre uma lista** logo acima do prompt, sobre o que estiver na tela, em todos os
prompts que têm histórico (Log, Copy, Move, filespec, X, Batch, Treespec, busca, filtro
de linhas, ir para a linha...).

- O mais recente fica embaixo, junto da linha, e o cursor começa nele.
- ↑ ↓ PgUp PgDn Home End andam pela lista.
- **Enter** põe o item na linha, para editar ou confirmar com outro Enter: nunca executa
  direto, para não copiar, mover ou rodar nada sem querer.
- **Esc** fecha a lista e devolve a linha como estava.
- **Digitar filtra:** o texto da linha é o filtro (sem diferença de maiúsculas e
  minúsculas, em qualquer parte do item), e a lista mostra só o que o contém. Vale para
  o que já estava digitado antes do ↑; o texto que o prompt já traz pronto (o outro
  lado da tela no Copy e Move, o diretório no Treespec) não filtra até ser editado.
  ← → e Backspace editam a linha como sempre.
- **Del tira o item** do histórico, e isso é salvo na hora. No L, Copy e Move, sai
  também da lista antiga de Copy e Move.
- Com a lista aberta, as teclas de função do prompt (F2 árvore, F11 bookmarks, F6/F7
  do Copy, Tab do Treespec) ficam para a lista; fechada, voltam a valer.
- **↓ com a lista fechada não faz nada.** O jeito antigo, de trocar item por item, sai.
- No Batch do viewer, **PgUp** abre a lista (↑↓ continuam com os templates).
- O rodapé mostra `↑ history` nos prompts com histórico e, com a lista aberta, as
  teclas dela.

## Modelo de ameaça

O histórico vem de um arquivo no disco (`history.toml`), que pode ter sido editado à mão
ou copiado de outro lugar, e guarda nomes de pastas que vieram de fora.

1. **Nada do item chega cru ao terminal:** a lista passa pelo filtro final de `ui::draw`
   (`bidi::sanitize`), que troca controles e caracteres bidi de todo o buffer; os itens
   são cortados na largura da lista.
2. **Memória com teto:** a lista é feita do histórico já carregado (até 100 itens por
   prompt); filtrar só guarda índices.
3. **Nada é executado ao escolher:** Enter só põe o texto na linha.
4. **O que sai com Del sai do arquivo também**, pelo mesmo `save_history` de hoje.

## Testes

- `LineInput`: abrir, andar, Enter põe na linha sem confirmar, Esc devolve a linha, o
  filtro pelo que foi digitado e não pelo texto pronto, Del tira o item e o devolve em
  `take_forgotten`, ↓ fechado não faz nada, histórico vazio não abre.
- App: Del na lista do L tira o lugar do histórico salvo (também de `destination`);
  F2 com a lista aberta não abre a árvore.
- Tela: a lista aparece acima do prompt, com o item mais novo embaixo.

## Fora deste spec

- Ordenar a lista de outro jeito (por frequência) ou fixar itens.
- O histórico próprio do terminal embutido (é do shell).
