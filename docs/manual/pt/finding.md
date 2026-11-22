# Encontrar arquivos e pastas

O Xylem tem várias formas de encontrar coisas. Cada uma responde a uma
pergunta diferente, e elas se dividem em três grupos: ir a um lugar, filtrar a
lista e olhar dentro dos arquivos.

| Você quer | Tecla | O que você obtém |
|---|---|---|
| *Aquela* pasta, por pedaços do nome | `\|` na árvore | o cursor da árvore vai até ela ([Busca aproximada](fuzzy.md)) |
| *Aquele* arquivo da lista, por pedaços do nome | `\|` na janela de arquivos | o cursor de arquivos vai até ele ([Busca aproximada](fuzzy.md)) |
| Um caminho que você conhece, nível por nível | `\` ou `/` | o cursor da árvore segue o que você digita ([Treespec](treespec.md)) |
| O próximo nome que começa com uma letra | Shift+letra | o cursor vai até ele ([A janela da árvore](tree.md)) |
| Os arquivos com este nome, data ou tamanho | `F` | a lista mostra só eles, até ser mudada ([Filespec](filespec.md)) |
| Os arquivos que atendem a uma condição | `W` | a lista mostra só eles, até ser mudada ([SQL](sql.md)) |
| Os arquivos com estas palavras | `P` | uma lista de resultados, o melhor primeiro ([A caixa de busca](sql.md#the-search-box-p)) |
| Os arquivos marcados com este texto | Ctrl+S | os outros são desmarcados ([Busca nos arquivos marcados](search.md)) |

## Ir a um lugar

A **busca aproximada** (`|`), o **Treespec** (`\`, `/`) e **Shift+letra**
movem o cursor, e não mudam mais nada. Use `|` quando você lembra um pedaço do
nome mas não onde ele está; o Treespec quando sabe o caminho; Shift+letra para
pular dentro da pasta em que está.

## Filtrar a lista

O **Filespec** (`F`) e o **W** decidem quais arquivos a janela de arquivos
mostra, e o filtro fica até você mudá-lo: o título da janela o mostra. `F` é
para nomes, datas e tamanhos; `W` para tudo o que o Xylem sabe de um arquivo,
inclusive o conteúdo (`contains`, `words`). Eles valem juntos, e o Ctrl+T
então marca todos os arquivos que passaram.

Para olhar a árvore inteira, abra antes o Branch (`B`) ou o Showall (`S`): os
filtros, e o `|`, então trabalham sobre todos os arquivos lidos dele.

## Olhar dentro dos arquivos

A **caixa de busca** (`P`) encontra arquivos pelas palavras que contêm,
enquanto você digita, e é instantânea num ramo indexado com Alt+N. O
**Ctrl+S** procura um trecho de texto nos arquivos marcados e desmarca os que
não o têm, e o Ctrl+V depois mostra cada ocorrência. No `W`,
`contains('texto')` e `words('a b')` fazem o mesmo como parte de uma condição.

## Marcar o que foi encontrado

Cada uma dessas formas termina em marcas quando você quer agir sobre muitos
arquivos:

- `|` e `P`: Ctrl+T marca o resultado escolhido; no `|`, Alt+T marca a lista.
- `F` e `W`: Ctrl+T na árvore marca todos os arquivos que passaram.
- O Ctrl+S deixa marcados só os arquivos que têm o texto.

Depois, Ctrl+C, Ctrl+M, Ctrl+D e Batch agem sobre os arquivos marcados. Veja
[Marcações](tagging.md).
