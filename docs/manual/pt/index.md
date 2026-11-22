# Manual do Xylem

O Xylem é um gerenciador de arquivos para o terminal, guiado pelo teclado: uma
árvore de diretórios no topo, os arquivos do diretório
atual abaixo e estatísticas à direita. Ele lê (*log*) diretórios para a
memória, permitindo que uma única lista reúna arquivos de todo um ramo e um
único comando copie, mova, exclua, compare ou renomeie todos os arquivos
marcados nele.

O Xylem trata arquivos que não criou como hostis: o conteúdo ou o nome de um
arquivo nunca deve fazê-lo quebrar, travar, executar código, gravar fora do
destino ou esgotar a memória. [Segurança](security.md) lista as medidas
adotadas e o que não está coberto.

**F1** (ou **Alt+H**, para terminais que reservam o F1 para si) abre a ajuda
sobre o que está na tela: dentro de um prompt, o tópico daquele prompt; na
árvore ou na janela de arquivos, o tópico daquela janela.

| Tecla na ajuda | O que faz |
|---|---|
| Tab, Shift+Tab | seleciona o link seguinte ou o anterior |
| Enter | segue o link |
| Backspace | volta |
| `I` | este índice |
| `K` | as teclas em vigor |
| `L` | alterna o idioma |
| Esc | fecha |

As versões deste manual em espanhol, francês, alemão e chinês não foram
revisadas por falantes nativos. Correções:
`https://github.com/andrelnfernandes/xylem/issues`.

## Comece por aqui

- [O que é o Xylem](overview.md): leitura, marcas e o que os comandos cobrem
- [Instalar e os primeiros cinco minutos](start.md)
- [A tela](screen.md): a árvore, a janela de arquivos, as estatísticas, o rodapé
- [Os menus, Ctrl e Alt](menus.md): uma letra sozinha, com Ctrl, com Alt, e F9
- [O mouse](mouse.md): ligá-lo no F12, a roda, os cliques e o rodapé
- [Vocabulário](glossary.md): log, branch, tag, filespec

## Comandos

- [Os menus DIR e FILE](commands.md): as letras sozinhas
- [O menu Ctrl](commands-ctrl.md): os arquivos marcados
- [O menu Alt](commands-alt.md): o ramo, os caminhos e as ferramentas
- [Git](git.md): o menu Git (Ctrl+G)
- [Teclas de função e símbolos](commands-keys.md)
- [As teclas em vigor](keys.md): com as alterações da configuração

## Janelas e visualizações

- [A janela da árvore](tree.md): navegação, leitura (Log), Branch e Showall
- [A janela de arquivos](files.md): modos de exibição, Autoview, abrir arquivos
- [Tela dividida](split.md): duas janelas lado a lado
- [O visualizador](viewer.md): texto, hexadecimal e tabelas, busca, cores de sintaxe
- [Filtrar linhas](filter.md): `&` no visualizador, com um guia de expressões regulares
- [Busca nos arquivos marcados](search.md): Ctrl+S e Ctrl+V
- [Informações](info.md): Info, Avail, estatísticas e tamanhos de diretórios
- [Uso do disco](usage.md): para onde vai o espaço (Alt+U)
- [Duplicados](duplicates.md): arquivos e pastas com o mesmo conteúdo (Alt+D)
- [Visões](views.md): a lista vista de outro jeito: a Varredura, a Linha do tempo e a Tabela (Alt+V)

## Trabalhando com arquivos

- [Encontrar arquivos e pastas](finding.md): qual busca responde a qual pergunta
- [Marcações e operações em grupo](tagging.md)
- [Copiar e Mover](copy-move.md), [escolher uma pasta](picker.md)
- [Excluir, lixeira e Prune](delete.md)
- [Renomear e renomeação em lote](rename.md)
- [Permissões e datas](attributes.md)
- [Filespec: o filtro de arquivos](filespec.md)
- [Treespec e Log](treespec.md)
- [Busca aproximada](fuzzy.md): uma pasta ou um arquivo por pedaços do nome (`|`)
- [Comparar](compare.md) e [Ordenar](sort.md)
- [Edit, eXecute e Batch](execute.md)
- [Arquivos compactados](archives.md): ZIP, 7z e TAR

## Mais ferramentas

- [O terminal integrado](terminal.md)
- [Máquinas remotas (SSH)](remote.md): outra máquina ou container como uma árvore
- [Armazenamento na nuvem (rclone)](cloud.md): Google Drive, S3 e outros
- [Encontrar arquivos com SQL](sql.md): uma condição sobre os arquivos e o conteúdo deles (W), e as palavras que contêm (P)
- [Tabelas](tables.md): planilhas, CSV, dBase, SQLite e Parquet como uma grade, com filtro SQL e cada registro como ficha
- [Favoritos](bookmarks.md)

## Como fazer

- [Copiar arquivos de várias pastas mantendo os caminhos](howto-copy-with-paths.md)
- [Renomear muitos arquivos com um contador](howto-rename-mask.md)
- [Escolher arquivos com uma condição SQL](howto-sql-filter.md)
- [Achar duplicados e ficar com uma cópia](howto-duplicates.md)
- [Achar o que enche o disco](howto-disk-space.md)
- [Filtrar um CSV ou uma planilha com SQL](howto-table-sql.md)
- [Copiar arquivos de outra máquina por SSH](howto-ssh-copy.md)
- [Usar os arquivos marcados num comando do shell](howto-terminal-tagged.md)

## Referência

- [Configuração](configuration.md): a tela F12, temas, teclas
- [A linha de comando](cli.md)
- [Terminais e teclas](terminals.md): F1, Ctrl+M, o protocolo de teclado do kitty
- [Segurança](security.md)
- [Sair e mudar o diretório do shell](quit.md)
- [Windows](windows.md)
- [Comparações](comparison.md): outros gerenciadores de arquivos, e um agradecimento
- [Notas de versão](releases.md)
