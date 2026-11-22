# Teclas de função e símbolos

Estas teclas ficam na última linha dos menus DIR e FILE, ou não aparecem nos
menus porque apenas movem o cursor. Elas não se combinam com os menus do F9:
após o F9, apenas a letra seguinte aceita Ctrl ou Alt.

## Teclas de função

| Tecla | Onde | O que faz |
|---|---|---|
| F1 | ambas | Abre a ajuda sobre o que está na tela. Alt+H faz o mesmo. [Terminais e teclas](terminals.md) |
| F2 | ambas | Relê o diretório, como o F3. Num prompt que pede uma pasta, abre uma árvore para [escolhê-la](picker.md). |
| F3 | ambas | Relê o diretório do disco. |
| F5 | Arquivos | Cria um arquivo compactado com o arquivo sob o cursor. Ctrl+F5: com os arquivos marcados. Alt+F5: abre um arquivo compactado. [Arquivos compactados](archives.md) |
| F7 | Arquivos | Autoview: exibe o conteúdo do arquivo sob o cursor ao lado da lista. [A janela de arquivos](files.md) |
| F8 | ambas | Abre uma segunda janela ao lado da primeira, ou a fecha. [Tela dividida](split.md) |
| F9 | ambas | Exibe o menu CTRL, depois o menu ALT, e volta. [Os menus](menus.md) |
| F10 | ambas | Sai do programa, como `Q`. |
| F11 | ambas | Abre o gerenciador de favoritos, como Alt+B. Ctrl+F11 salva o diretório atual. [Favoritos](bookmarks.md) |
| F12 | ambas | Abre a tela de configuração. [Configuração](configuration.md) |

## Símbolos

| Tecla | Onde | O que faz |
|---|---|---|
| `*` | Árvore | Lê o diretório sob o cursor e tudo abaixo dele. Esc cancela. |
| `+` ou → | Árvore | Lê o diretório e mostra seus subdiretórios. |
| `-` | Árvore | Libera o ramo da memória. |
| ← | Árvore | Fecha o diretório, ou vai para o diretório pai. |
| `?` | ambas | Exibe as estatísticas da janela, do disco, da máquina e do Xylem. [Informações](info.md) |
| `\` ou `/` | ambas | Treespec: digite um caminho e o cursor da árvore o segue. Num Branch ou Showall, vai para o diretório do arquivo sob o cursor. [Treespec](treespec.md) |
| `\|` | ambas | Busca aproximada: digite pedaços de um nome, e o cursor vai para a melhor pasta (na árvore) ou o melhor arquivo da lista (na janela de arquivos). [Busca aproximada](fuzzy.md) |
| `'` | ambas | Pula para um favorito: pressione a tecla dele em seguida. |

## Navegação

| Tecla | Onde | O que faz |
|---|---|---|
| ↑ ↓ PgUp PgDn Home End | ambas | Movem o cursor. |
| Espaço | ambas | Desce uma linha. |
| Enter | Árvore | Vai para a janela de arquivos. |
| Enter | Arquivos | Volta para a árvore. Com `enter_views` ativado, visualiza o arquivo. |
| Esc | Arquivos | Volta para a árvore, no diretório do arquivo sob o cursor. |
| Esc | Árvore | Dentro de um arquivo compactado, fecha-o. |
| Tab | ambas | Com a tela dividida, vai para a outra janela. Caso contrário, o mesmo que Enter. |
| Backspace | ambas | Vai para o diretório pai. |
| Shift+letra, 0–9 | ambas | Vai para o próximo diretório ou arquivo cujo nome começa por esse caractere. |
| ← → | Arquivos | Rolam a página para cima ou para baixo. Nos modos de exibição em colunas, vão para o arquivo anterior e o seguinte. |

## Alterar uma tecla

`[keys.any]`, `[keys.tree]` e `[keys.files]` no `config.toml` associam uma
tecla a outro comando, ou a `none`. **K** na ajuda lista as teclas em vigor,
com essas alterações. Veja [configuração](configuration.md).

Veja também: [os menus DIR e FILE](commands.md),
[o menu Ctrl](commands-ctrl.md), [o menu Alt](commands-alt.md).
