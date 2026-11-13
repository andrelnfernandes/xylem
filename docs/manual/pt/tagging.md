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

Como no ZTreeWin:

- **A letra sozinha** age sobre o item sob o cursor.
- **Ctrl+letra** age sobre os arquivos marcados: Ctrl+C copia, Ctrl+D apaga,
  Ctrl+T marca tudo.
- **Alt+letra** é a outra versão de um comando: na árvore, o ramo inteiro
  (Alt+P Prune, Alt+R Release, Alt+U Uso, Alt+D Duplicados, Alt+C Compare); na
  janela de arquivos, os marcados com seus caminhos (Alt+C, Alt+M); e as
  ferramentas (Alt+S Sort, Alt+F File display, Alt+B Bookmarks, Alt+H Ajuda,
  Alt+I Info, Alt+X o terminal).

**Mudou na versão 0.8.** O Alt não repete mais o Ctrl, e algumas teclas mudaram:

| Tecla | Até a 0.7 | A partir da 0.8 |
|---|---|---|
| Alt+A | permissões dos marcados | nada (Ctrl+A) |
| Alt+D | apagar os marcados | Duplicados (apagar os marcados: Ctrl+D) |
| Alt+R | renomear os marcados | Release do ramo, como `-` (renomear: Ctrl+R) |
| Alt+T | marcar tudo | nada (Ctrl+T) |
| Alt+U | desmarcar tudo | Uso (desmarcar: Ctrl+U) |
| Alt+W | Uso | nada (Alt+U) |
| Alt+Y | Duplicados | nada (Alt+D) |

As teclas antigas podem voltar em `[keys.any]` na
[configuração](configuration.md), uma ou todas. Todas, como até a 0.7:

```toml
[keys.any]
"alt+a" = "attributes_tagged"
"alt+d" = "delete_tagged"
"alt+r" = "rename_tagged"
"alt+t" = "tag_all"
"alt+u" = "untag_all"
"alt+w" = "usage"
"alt+y" = "duplicates"
```

Elas valem nas duas janelas, também o Alt+R na árvore; o Release fica no `-`.
**K** na ajuda (F1) mostra as teclas em vigor.

## Menus Ctrl e Alt (F9)

O rodapé tem três linhas numa tela comum: os comandos de letra em ordem
alfabética nas duas primeiras, as teclas de função e os símbolos na terceira.
Numa tela larga o bastante para todos os menus, tem duas: as letras numa, as
teclas na outra. Todo comando está em
um dos menus (DIR ou FILE, CTRL, ALT); dentro de um arquivo compactado ou numa
máquina remota, eles deixam de fora o que ali não funciona.

**F9** alterna
para o menu Ctrl, depois para o menu Alt e de volta. Com um desses menus
aberto, a próxima letra vale como Ctrl+letra ou Alt+letra (`F9` `M` = Ctrl+M);
`Esc` cancela. Em terminais com o protocolo de teclado do kitty, basta
**segurar Ctrl ou Alt** para ver o menu correspondente.

Soltar a tecla traz de volta o menu de sempre, como no ZTree.

O terminal precisa informar Ctrl e Alt apertados e soltos por si sós, e só o
protocolo de teclado do kitty faz isso: kitty, Ghostty, WezTerm, foot (Wayland)
e o Alacritty recente o suportam, e no Ubuntu ou no Debian a maioria está a um
pacote de distância (`sudo apt install kitty`, ou `ghostty`, `alacritty`,
`foot`). Terminais baseados em VTE (GNOME Terminal, Ptyxis, Terminator, Tilix)
ainda não o suportam, então neles os menus ficam no F9.

**Ctrl+M:** na maioria dos terminais, Ctrl+M chega igual a Enter. O Xylem ativa
o protocolo de teclado do kitty quando o terminal o suporta (kitty, WezTerm,
foot, Ghostty, Alacritty recente, iTerm2), e então Ctrl+M move os arquivos
marcados como no ZTree. Em terminais baseados em VTE (GNOME Terminal, Ptyxis,
Terminator, Tilix) use **F9 M** (ou Alt+M e F6 no prompt).
