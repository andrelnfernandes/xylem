# Marcações e operações em grupo

Os arquivos marcados (indicados com `■`) são o alvo das operações em grupo: os
arquivos marcados **da lista atual** (o diretório, ou o Branch ou Showall
aberto).

## Marcar arquivos

| Tecla | Árvore | Janela de arquivos |
|---|---|---|
| `T` / `U` | marca / desmarca os arquivos do diretório | marca / desmarca o arquivo e desce |
| Ctrl+T / Ctrl+U | marca / desmarca tudo o que foi lido (que corresponda ao Filespec) | marca / desmarca toda a lista |
| Ctrl+S | Showall só dos arquivos marcados | |
| `I` | inverte as marcações do diretório | inverte as marcações da lista |
| Ctrl+F7 / F8 / F9 | marca / desmarca / inverte os arquivos lidos do ramo | |

## Operações nos arquivos marcados

| Tecla | O que faz |
|---|---|
| Ctrl+C / Ctrl+M | [copia / move](copy-move.md) os arquivos para o destino |
| Alt+C / Alt+M | copia / move os arquivos, **recriando seus caminhos** |
| Ctrl+D | [exclui](delete.md) os arquivos |
| Ctrl+A | altera suas [permissões](attributes.md) |
| Ctrl+N | altera sua [data](attributes.md) |
| Ctrl+R | [renomeia](rename.md) os arquivos com uma máscara |
| Ctrl+B | [Batch](execute.md): um script ou lista com uma linha por arquivo |
| Ctrl+F5 | coloca-os em um [arquivo compactado](archives.md) |
| Ctrl+S / Ctrl+V | [busca texto neles / visualiza-os](search.md) |
| Ctrl+O | abre-os no programa padrão |
| Ctrl+Y | copia seus caminhos para a área de transferência |
| Alt+O | digita-os no [terminal integrado](terminal.md) |

Em eXecute, `%t` representa os arquivos marcados ([eXecute](execute.md)).

## O que Ctrl e Alt querem dizer

- Uma letra sozinha age sobre o item sob o cursor.
- Ctrl+letra age sobre os arquivos marcados: Ctrl+C copia, Ctrl+D exclui.
- Alt+letra é a outra versão de um comando: o ramo inteiro na árvore, os
  arquivos marcados com seus caminhos na janela de arquivos.

[Os menus](menus.md) explica a regra e como exibir os menus Ctrl e Alt com
F4. [O menu Ctrl](commands-ctrl.md) e [o menu Alt](commands-alt.md) listam cada
comando.

**Mudou na versão 0.8.** O Alt não repete mais o Ctrl: Alt+A, Alt+D, Alt+R,
Alt+T, Alt+U, Alt+W e Alt+Y mudaram. [Notas de versão](releases.md) traz o
significado antigo e o novo de cada tecla, e as linhas para o `config.toml` que
restauram os antigos.
