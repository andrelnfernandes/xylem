# Vocabulário

As palavras abaixo vêm do XTree e do ZTreeWin. Os menus e este manual as usam
com estes significados.

| Termo | Significado |
|---|---|
| Log | Ler um diretório para a memória: seus arquivos e os nomes de seus subdiretórios. `→` ou `+` lê um diretório e `*` um ramo inteiro. `L` (Log) também é o comando que abre outro diretório como a árvore. [A janela da árvore](tree.md) |
| Logged | Lido para a memória. Apenas arquivos lidos aparecem em listas, são contados nas estatísticas e podem ser marcados. |
| Not logged | Um diretório cujo nome é conhecido e cujo conteúdo não foi lido. A árvore o indica com `?`. |
| Release | Liberar um ramo da memória com `-`. Seus arquivos saem das listas e perdem as marcas. Nada é alterado no disco. |
| Relog | Reler um diretório do disco com F3. |
| Branch | Um diretório e tudo o que está abaixo dele. `B` lista os arquivos do ramo em uma única lista. |
| Showall | A lista de todos os arquivos lidos na janela, aberta com `S`. |
| Tag | Uma marca em um arquivo (`■`). Os comandos Ctrl atuam sobre os arquivos marcados. As marcas ficam na memória e são perdidas no Release ou ao sair. [Marcações](tagging.md) |
| Filespec | O filtro de quais arquivos a janela exibe, por nome, data e tamanho. Definido com `F`. [Filespec](filespec.md) |
| Treespec | Um caminho digitado para mover o cursor da árvore. Aberto com `\` ou `/`. [Treespec](treespec.md) |
| Janela | Uma árvore com sua janela de arquivos. F8 abre uma segunda ao lado. [Tela dividida](split.md) |
| Raiz | O diretório no topo da árvore. `L` a altera, e Alt+L torna o diretório sob o cursor a raiz. |
| Matching | Os arquivos lidos que passam pelo Filespec. |
| Máscara | Um padrão que constrói um nome ou uma linha por arquivo: `*.bak`, `foto_<001>.*`. Usado por Rename e Batch. [Renomear](rename.md) |
| Batch | Ctrl+B: grava um arquivo com uma linha por arquivo marcado, ou por registro de uma tabela no visualizador. [Batch](execute.md) |
| Prune | Alt+P: exclui um diretório com tudo o que está dentro dele. [Excluir](delete.md) |
| Oops | Restaura o último item enviado para a lixeira. `O` na árvore, Ctrl+Z nas duas janelas. |
| Yank | `Y`: copia um caminho para a área de transferência. |
| Autoview | F7: a janela de arquivos com o conteúdo do arquivo sob o cursor ao lado da lista. |
| eXecute | `X`: executa um comando do shell, com `%f` para o arquivo e `%t` para os arquivos marcados. [eXecute](execute.md) |
| Avail | `A` na árvore: os sistemas de arquivos montados com seu espaço livre. |

Veja também: [o que é o Xylem](overview.md), [os menus](menus.md).
