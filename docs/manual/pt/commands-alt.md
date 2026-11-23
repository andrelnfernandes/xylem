# O menu Alt

Alt com uma letra é a outra versão de um comando. Na árvore, abrange todo o
ramo sob o cursor. Na janela de arquivos, Alt+C e Alt+M atuam sobre os arquivos
marcados e mantêm seus caminhos. As demais entradas abrem ferramentas que não
atuam sobre um único arquivo.

Para exibir este menu, segure Alt ou pressione F4 duas vezes. Veja [os menus](menus.md).

## Comandos

Na ordem do menu. "Árvore" e "Arquivos" indicam onde a tecla funciona.

| Tecla | Comando | Árvore | Arquivos | O que faz |
|---|---|---|---|---|
| Alt+B | Bookmarks | sim | sim | Abre o gerenciador de favoritos: ir para um diretório salvo, nomeá-lo, removê-lo. [Favoritos](bookmarks.md) |
| Alt+C | Compare | sim | | Compara todo o ramo com o da outra janela, por caminho relativo, e abre um Branch com os arquivos marcados. [Comparar](compare.md) |
| Alt+C | Copy | | sim | Copia os arquivos marcados recriando o caminho de cada arquivo sob o destino. F7 no prompt define onde o caminho começa. [Copiar e Mover](copy-move.md) |
| Alt+D | Duplicates | sim | sim | Localiza arquivos e pastas com o mesmo conteúdo no ramo. Não exclui nada sem confirmação. [Duplicados](duplicates.md) |
| Alt+F | File display | sim | sim | Altera o que cada linha da janela de arquivos exibe: nomes em colunas, tamanhos, detalhes, proprietário, tipo, nomes longos. [A janela de arquivos](files.md) |
| Alt+H | Help | sim | sim | Abre a ajuda, como faz o F1. |
| Alt+I | Info | sim | sim | Mostra tipo, tamanho, permissões, proprietário e datas do arquivo ou diretório. [Informações](info.md) |
| Alt+K | Kbyte | sim | | Exibe o tamanho, a quantidade de arquivos ou a porcentagem ao lado de cada diretório da árvore. [Informações](info.md) |
| Alt+L | Log | sim | sim | Torna o diretório sob o cursor a raiz da árvore. O que foi lido e as marcas são descartados. [Log](treespec.md) |
| Alt+M | Move | | sim | Move os arquivos marcados recriando o caminho de cada arquivo sob o destino. [Copiar e Mover](copy-move.md) |
| Alt+P | Prune | sim | | Exclui o diretório com tudo dentro dele, permanentemente, após você digitar uma confirmação. [Excluir](delete.md) |
| Alt+Q | Quit here | sim | sim | Sai e deixa o shell no diretório atual. Requer a função do shell de [Sair](quit.md). |
| Alt+R | Release | sim | | Libera o ramo da memória, como faz `-`. Suas marcas são perdidas. |
| Alt+S | Sort | sim | sim | Ordena a janela de arquivos por nome, extensão, data ou tamanho. [Ordenar](sort.md) |
| Alt+U | Usage | sim | sim | Lista as pastas e arquivos do ramo pelo espaço que ocupam, do maior para o menor. [Uso do disco](usage.md) |
| Alt+V | Views | sim | sim | Abre as visões da lista: a Varredura, que aponta o que merece um olhar, a Linha do tempo, por data, e a Tabela, a lista no visualizador de tabelas. [Visões](views.md) |
| Alt+N | iNdex | sim | sim | Indexa as palavras dos arquivos lidos do ramo, para que `words()` em uma condição SQL responda na hora. [O índice](sql.md#palavras-inteiras-prefixos-e-índice-words) |

## Outras teclas

| Tecla | Onde | O que faz |
|---|---|---|
| Alt+X | ambas | Abre o terminal integrado e volta dele. [O terminal integrado](terminal.md) |
| Alt+Z | ambas | Abre o terminal em tela cheia. |
| Alt+O | ambas | Digita os arquivos marcados na linha de comando do terminal, entre aspas. |
| Alt+F5 | Arquivos | Abre o arquivo compactado sob o cursor como uma árvore. [Arquivos compactados](archives.md) |
| Alt+F10 | ambas | Abre a configuração, como faz o F12. No GNOME, Alt+F10 maximiza a janela e não chega ao Xylem. [Configuração](configuration.md) |
| Alt+Shift+Del | ambas | Exclui os arquivos marcados permanentemente, como faz Ctrl+Shift+Del. |

## Dentro de um arquivo compactado ou máquina remota

| Onde | Não listados |
|---|---|
| Arquivo compactado | Log, Move, Prune, as teclas do terminal. Alt+C extrai os arquivos marcados com seus caminhos dentro do arquivo compactado. |
| Máquina remota | Log (Alt+L), Alt+O. Alt+X abre um shell naquela máquina. |

## Mudou na versão 0.8

Até a 0.7, Alt+A, Alt+D, Alt+R, Alt+T e Alt+U faziam o mesmo que Ctrl com
aquela letra, Alt+W era Usage e Alt+Y era Duplicates.
[Notas de versão](releases.md) traz a tabela e as linhas para o `config.toml`
que restauram as teclas antigas.

Veja também: [os menus DIR e FILE](commands.md),
[o menu Ctrl](commands-ctrl.md).
