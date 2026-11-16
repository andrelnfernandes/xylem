# O menu Ctrl

Ctrl com uma letra executa o comando nos arquivos marcados. Na janela de
arquivos, são os arquivos marcados da lista (o diretório, ou o Branch ou
Showall aberto). Na árvore, são os arquivos marcados do diretório sob o
cursor. As linhas indicadas como "tudo o que foi lido" são as exceções.

Sem nenhum arquivo marcado, esses comandos exibem `No tagged files` e não
fazem nada.

Para exibir este menu, segure Ctrl ou pressione F9. Veja [os menus](menus.md).

## Comandos

Na ordem do menu. "Árvore" e "Arquivos" indicam onde a tecla funciona.

| Tecla | Command | Árvore | Arquivos | O que faz |
|---|---|---|---|---|
| Ctrl+A | Attributes | sim | sim | Altera as permissões dos arquivos marcados. [Permissões e datas](attributes.md) |
| Ctrl+B | Bookmark | sim | | Salva o diretório sob o cursor como um favorito. Não usa as marcas. [Favoritos](bookmarks.md) |
| Ctrl+B | Batch | | sim | Grava um script ou uma lista com uma linha por arquivo marcado, a partir de uma máscara. [Batch](execute.md) |
| Ctrl+C | Copy | sim | sim | Copia os arquivos marcados para um diretório de destino, sem seus caminhos. Alt+C mantém os caminhos. [Copiar e Mover](copy-move.md) |
| Ctrl+D | Delete | sim | sim | Exclui os arquivos marcados, após confirmação. Eles vão para a lixeira quando `delete_to_trash` estiver ativo. [Excluir](delete.md) |
| Ctrl+G | Git | sim | sim | Abre o menu Git. Aparece apenas com a opção Git ativada. [Git](git.md) |
| Ctrl+M | Move | sim | sim | Move os arquivos marcados para um diretório de destino. Em terminais sem o protocolo de teclado do kitty, pressione `F9` `M`. [Copiar e Mover](copy-move.md) |
| Ctrl+N | New date | | sim | Define a data de modificação dos arquivos marcados. [Permissões e datas](attributes.md) |
| Ctrl+O | Open | | sim | Abre cada arquivo marcado no aplicativo padrão do sistema. [A janela de arquivos](files.md) |
| Ctrl+P | Phrase | sim | sim | Abre a caixa de busca sobre os arquivos marcados: lista aqueles que contêm as palavras digitadas. [A caixa de busca](filespec.md#a-caixa-de-busca-p) |
| Ctrl+R | Rename | sim | sim | Renomeia os arquivos marcados com uma máscara. [Renomear](rename.md) |
| Ctrl+S | Showall | sim | | Lista os arquivos marcados de tudo o que foi lido. [A janela da árvore](tree.md) |
| Ctrl+S | Search | | sim | Procura um texto nos arquivos marcados e desmarca os que não o contêm. [Busca nos arquivos marcados](search.md) |
| Ctrl+T | Tag | sim | sim | Arquivos: marca toda a lista. Árvore: marca tudo o que foi lido e atende ao Filespec. [Marcações](tagging.md) |
| Ctrl+U | Untag | sim | sim | Arquivos: desmarca toda a lista. Árvore: desmarca tudo o que foi lido e atende ao Filespec. |
| Ctrl+V | View | | sim | Abre os arquivos marcados no visualizador, um após o outro; `N` e `P` alternam entre os arquivos. [Busca nos arquivos marcados](search.md) |
| Ctrl+W | Where | sim | sim | Mantém na lista os arquivos marcados que atendem a uma condição SQL. [Uma condição SQL](filespec.md#uma-condicao-sql-w) |
| Ctrl+Y | Yank | sim | sim | Copia os caminhos dos arquivos marcados para a área de transferência. |

## Teclas de função

| Tecla | Onde | O que faz |
|---|---|---|
| Ctrl+F5 | Arquivos | Coloca os arquivos marcados em um novo arquivo compactado. [Arquivos compactados](archives.md) |
| Ctrl+F7 | Árvore | Marca os arquivos lidos do ramo sob o cursor. |
| Ctrl+F8 | Árvore | Desmarca-os. |
| Ctrl+F9 | Árvore | Inverte suas marcas. |
| Ctrl+F11 | ambas | Salva um favorito, como Ctrl+B faz na árvore. |
| Ctrl+Shift+Del | ambas | Exclui os arquivos marcados permanentemente, sem lixeira. |
| Ctrl+Z | ambas | Oops: restaura o último item enviado para a lixeira. |

## Dentro de um arquivo compactado ou máquina remota

O menu omite o que não pode ser executado ali.

| Onde | Não listados |
|---|---|
| Arquivo compactado | Attributes, Delete, Git, Move, New date, Rename, Ctrl+F5. Copy extrai. |
| Máquina remota | Batch, Git, Ctrl+F5. |
