# Busca aproximada (|)

`|` encontra uma pasta, ou um arquivo da lista, por pedaços do nome, em
qualquer nível da árvore: `cfgxy` encontra `config/xylem`, `rdme` encontra
`README.md`. Os resultados aparecem acima do prompt enquanto você digita, o
melhor primeiro, e o cursor da janela segue o escolhido.

Para ir a um caminho que você já conhece, nível por nível, o
[Treespec](treespec.md) é mais rápido. Para escolher muitos arquivos por nome,
data ou tamanho, ou pelo que há neles, veja
[Encontrar arquivos e pastas](finding.md).

## O que ela procura

O que o `|` procura depende da janela em que ele é aberto:

- **Na árvore:** as pastas lidas, pelo caminho a partir da raiz da árvore
  (`docs plan` encontra `docs/plans`). Pastas ocultas, e o que há dentro
  delas, só são procuradas enquanto a árvore as mostra.
- **Na janela de arquivos:** os arquivos da lista como ela está, com o
  [filespec](filespec.md) e a [condição SQL](sql.md) (`W`) em vigor. No
  diretório, pelo nome; no Branch e no Showall, também pelo caminho
  (`src fzrs` encontra `src/fuzzy.rs`).

Para encontrar um arquivo em qualquer lugar da árvore, abra o Showall (`S`) e
depois o `|`.

Só o que foi lido é procurado, e nada é lido do disco nem da rede: funciona do
mesmo jeito numa [máquina remota](remote.md), dentro de um
[arquivo compactado](archives.md) e numa [montagem de nuvem](cloud.md).

## Como ela compara

- **As letras em ordem**, não necessariamente juntas: `rdme` encontra
  `README.md`.
- **Espaços separam termos**, e todo termo tem que combinar, em qualquer
  ordem: `xy docs` encontra `xylem/docs` e `docs/xylem`.
- **Maiúsculas e minúsculas** não contam, a não ser que o termo tenha uma
  maiúscula: `Rd` não encontra `readme`. Cada termo decide por si.
- **O melhor primeiro:** letras juntas contam mais que letras separadas; uma
  letra no começo de uma palavra (no início, depois de `/`, `_`, `-`, `.` ou
  um espaço, ou onde a minúscula vira maiúscula, como em `fileSpec`) conta
  mais que uma no meio; uma correspondência no nome conta mais que uma nas
  pastas acima dele. Entre iguais, o caminho mais curto vem primeiro, e depois
  a ordem da árvore ou da lista.

As letras que combinaram aparecem na cor das teclas. Um nome comprido demais
para a janela é cortado à esquerda, com `…`, para que o fim dele fique à
vista.

## Na tela

A contagem no alto à direita diz como a busca está indo:

| Ela diz | Significado |
|---|---|
| `type pieces of a name` | o prompt está vazio |
| `120 found · searching 40%` | a busca ainda está percorrendo a árvore; a lista vai se enchendo |
| `120` | terminou: todas as correspondências estão na lista |
| `500 best of 15,894` | terminou: a lista guarda as 500 melhores |
| `nothing matches` | terminou, sem nenhuma correspondência |

Uma árvore grande é procurada em etapas enquanto você digita, então os
primeiros resultados vêm na hora e o resto vem depois. Quando você acrescenta
letras no fim, a busca olha só entre o que já tinha combinado. Se a árvore
muda com o prompt aberto (uma pasta lida, liberada ou relida), a busca
recomeça.

Um `*` antes de um arquivo quer dizer que ele está marcado; depois de uma
pasta, `3*` diz quantos arquivos dela estão marcados.

## Teclas

| Tecla | O que faz |
|---|---|
| ↑ ↓ | escolhe o resultado anterior ou o próximo; o cursor da janela segue |
| PgUp PgDn, Home End | escolhe uma página acima ou abaixo, o primeiro ou o último resultado |
| Enter | fica onde o cursor está e fecha o prompt |
| Esc | volta para onde o cursor estava, e fecha o prompt |
| Ctrl+T | marca o resultado escolhido, ou o desmarca se estiver marcado, e escolhe o próximo |
| Alt+T | marca os arquivos dos resultados da lista |
| Ctrl+U | limpa o prompt |

Com o [mouse](mouse.md) ligado, um clique escolhe um resultado e um duplo
clique vai até ele.

## Marcar a partir dos resultados

**Ctrl+T** num arquivo o marca; numa pasta, marca os arquivos dela que a
janela de arquivos mostraria (com o filespec e o `W` em vigor). Se todos já
estão marcados, ele os desmarca. A escolha então passa para o próximo
resultado, de modo que Ctrl+T, Ctrl+T, Ctrl+T marca os resultados um depois
do outro.

**Alt+T** marca os arquivos de todos os resultados da lista: no máximo os 500
mostrados. Quando mais combinaram, a nota abaixo da lista diz quantos ficaram
de fora. Alt+T numa pasta que não foi lida não marca nada ali, e a nota diz
quantas foram puladas.

As marcas ficam depois que o prompt fecha, como qualquer marca: Ctrl+C,
Ctrl+M, Ctrl+D e Batch então agem sobre elas. Veja [Marcações](tagging.md).

Marcar todas as correspondências, não só as 500 da lista, é uma opção,
desligada por padrão: o Alt+T poderia então marcar milhares de arquivos que
você não viu. Leia
[Marcar todas as correspondências da busca aproximada](configuration.md#tagging-every-match-of-the-fuzzy-find)
antes de ligá-la.
