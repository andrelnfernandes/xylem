# Os menus DIR e FILE

Uma letra sozinha executa o comando no item sob o cursor: o diretório na
árvore (o menu DIR), o arquivo na janela de arquivos (o menu FILE).

Algumas letras representam comandos diferentes em cada janela. `C` compara na
árvore e copia na janela de arquivos; `M` cria um diretório na árvore e move um
arquivo na janela de arquivos.

## Comandos

Na ordem dos menus. "Árvore" e "Arquivos" indicam onde a tecla funciona.

| Tecla | Comando | Árvore | Arquivos | O que faz |
|---|---|---|---|---|
| `A` | Available | sim | | Lista os sistemas de arquivos montados com espaço total, usado e livre. [Informações](info.md) |
| `A` | Attributes | | sim | Altera as permissões do arquivo, em octal. [Permissões e datas](attributes.md) |
| `B` | Branch | sim | sim | Lista os arquivos do diretório e de tudo abaixo dele, lendo o ramo primeiro. Em Branch ou Showall, volta aos arquivos apenas do diretório. |
| `C` | Compare | sim | | Com a tela dividida, compara o diretório com o da outra janela e marca os arquivos que diferem pelo critério escolhido. [Comparar](compare.md) |
| `C` | Copy | | sim | Copia o arquivo. O prompt pergunta o destino. [Copiar e Mover](copy-move.md) |
| `D` | Delete | sim | sim | Exclui o arquivo, após confirmação. Na árvore, exclui o diretório se estiver vazio. [Excluir](delete.md) |
| `E` | Edit | | sim | Abre o arquivo no seu editor. [Edit](execute.md) |
| `F` | Filespec | sim | sim | Define quais arquivos a janela exibe, por nome, data e tamanho. [Filespec](filespec.md) |
| `I` | Invert | sim | sim | Marca os desmarcados e desmarca os marcados: do diretório na árvore, da lista na janela de arquivos. |
| `L` | Log | sim | | Abre outro diretório, ou máquina remota, como a árvore desta janela. [Log](treespec.md) |
| `M` | Makedir | sim | | Cria um subdiretório no diretório sob o cursor. |
| `M` | Move | | sim | Move o arquivo. [Copiar e Mover](copy-move.md) |
| `N` | New date | sim | sim | Define a data de modificação do arquivo ou diretório. [Permissões e datas](attributes.md) |
| `O` | Oops | sim | | Restaura o último item enviado para a lixeira. Na janela de arquivos, use Ctrl+Z. [Excluir](delete.md) |
| `O` | Open | | sim | Abre o arquivo no aplicativo padrão do sistema. [A janela de arquivos](files.md) |
| `P` | Phrase | sim | sim | Abre a caixa de busca: lista os arquivos lidos que contêm as palavras digitadas. [A caixa de busca](sql.md#a-caixa-de-busca-p) |
| `R` | Rename | sim | sim | Renomeia o arquivo ou o diretório. [Renomear](rename.md) |
| `S` | Showall | sim | sim | Lista todos os arquivos lidos da janela. |
| `T` | Tag | sim | sim | Arquivos: marca o arquivo e desce uma linha. Árvore: marca os arquivos do diretório. [Marcações](tagging.md) |
| `U` | Untag | sim | sim | Arquivos: desmarca o arquivo e desce uma linha. Árvore: desmarca os arquivos do diretório. |
| `V` | View | | sim | Abre o arquivo no visualizador: texto, hexadecimal ou tabela. [O visualizador](viewer.md) |
| `W` | Where | sim | sim | Mantém na lista os arquivos que atendem a uma condição SQL. [Uma condição SQL](sql.md#uma-condição-sql-w) |
| `X` | eXecute | sim | sim | Executa um comando do shell. `%f` representa o arquivo e `%t` os arquivos marcados. [eXecute](execute.md) |
| `Y` | Yank | sim | sim | Copia o caminho do arquivo ou diretório para a área de transferência. |
| `Q` | Quit | sim | sim | Sai do programa, após confirmação. [Sair](quit.md) |

As teclas na última linha do menu (F1, `*`, `?`, `\`, F3, Enter, F5, F7, F8, F4)
estão em [teclas de função e símbolos](commands-keys.md).

## Teclas que não estão no menu

Estas são teclas alternativas para um comando da tabela.

| Tecla | Equivalente a |
|---|---|
| Del | `D` |
| Shift+Del | `D`, excluindo em definitivo sem lixeira |
| F10 | `Q` |
| Ctrl+Insert | `Y` |
| Ctrl+Z | `O` (Oops), em ambas as janelas |

## Dentro de um arquivo compactado ou máquina remota

O título do menu é ARCHIVE ou REMOTE, e os comandos que não podem rodar ali
não são listados.

| Onde | Não listados |
|---|---|
| Arquivo compactado | Delete, Edit, Log, Makedir, Move, New date, Oops, Rename, Attributes, eXecute. Copy extrai. |
| Máquina remota | Edit, Oops, eXecute. Não há lixeira em máquina remota. |

Veja [arquivos compactados](archives.md) e [máquinas remotas](remote.md).

Veja também: [o menu Ctrl](commands-ctrl.md), [o menu Alt](commands-alt.md),
[os menus](menus.md).
