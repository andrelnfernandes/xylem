# Os menus, Ctrl e Alt

O rodapé lista os comandos que funcionam na janela que possui o foco. Há quatro
menus. O título à esquerda indica qual deles está sendo exibido.

| Título | Exibido quando | Seus comandos atuam em |
|---|---|---|
| DIR | a árvore tem o foco | o diretório sob o cursor |
| FILE | a janela de arquivos tem o foco | o arquivo sob o cursor |
| CTRL | Ctrl é mantido pressionado, ou após F9 | os arquivos marcados |
| ALT | Alt é mantido pressionado, ou após F9 duas vezes | veja [o menu Alt](#alt-a-outra-versão-de-um-comando) |

Dentro de um arquivo compactado o título é ARCHIVE, e em uma máquina remota
REMOTE. Ambos deixam de fora os comandos que não funcionam ali.

## Como ler um menu

Cada palavra é um comando, e sua letra maiúscula é a tecla: `Copy` é `C`,
`eXecute` é `X`, `iNdex` é `N`. Nos menus CTRL e ALT a tecla é essa letra com
Ctrl ou Alt.

As palavras estão em ordem alfabética pela sua tecla, com Quit por último.
Abaixo delas há uma linha com as teclas de função e símbolos (`F3 relog`,
`? stats`).

O rodapé tem três linhas. Em uma tela larga o suficiente para caber todas as
palavras em uma linha, ele tem duas. Em uma tela estreita as palavras são
encurtadas, todas pelo mesmo número de letras, e `…` indica entradas que não
couberam.

## Ctrl: os arquivos marcados

Uma letra sozinha atua sobre um item. A mesma letra com Ctrl atua sobre os
arquivos marcados da lista:

| Tecla | Sozinha | Com Ctrl |
|---|---|---|
| `C` | copiar o arquivo | copiar os arquivos marcados |
| `D` | excluir o arquivo | excluir os arquivos marcados |
| `R` | renomear o arquivo | renomear os arquivos marcados com uma máscara |
| `T` | marcar o arquivo | marcar todos os arquivos da lista |

Na árvore, "a lista" é o diretório sob o cursor. Três comandos são mais amplos
ali: Ctrl+T e Ctrl+U marcam e desmarcam todos os arquivos lidos que atendem ao
Filespec, e Ctrl+S lista os arquivos marcados de tudo o que foi lido.

[O menu Ctrl](commands-ctrl.md) traz todos os comandos.

## Alt: a outra versão de um comando

O Alt não repete o Ctrl. O que ele faz depende da janela:

- Na árvore, o comando abrange todo o ramo. Alt+P (Prune) exclui o diretório e
  tudo dentro dele, Alt+R libera o ramo da memória, Alt+C compara o ramo com a
  outra janela.
- Na janela de arquivos, Alt+C e Alt+M copiam e movem os arquivos marcados
  recriando o caminho de cada arquivo sob o destino.
- Em ambas, o Alt abre as ferramentas que não dizem respeito a um único arquivo:
  Alt+S (Sort), Alt+F (File display), Alt+B (Bookmarks), Alt+I (Info),
  Alt+X (o terminal).

[O menu Alt](commands-alt.md) traz todos os comandos.

## Como exibir os menus Ctrl e Alt

**F9** exibe o menu CTRL, F9 novamente o menu ALT, e uma terceira vez volta ao
anterior. Enquanto um deles estiver visível, a próxima letra é recebida com
aquele modificador: `F9` `C` equivale a Ctrl+C. Esc volta sem executar nada.

No kitty, Ghostty, WezTerm, foot e Alacritty recente, segurar Ctrl ou Alt exibe
o menu, e soltar a tecla traz de volta DIR ou FILE. Esses terminais informam um
modificador pressionado isoladamente (protocolo de teclado do kitty). O GNOME
Terminal, Ptyxis, Terminator e Tilix não fazem isso, então neles os menus
alternam apenas com F9.

## Ctrl+M

A maioria dos terminais envia o mesmo byte para Ctrl+M e Enter, de modo que o
Xylem não consegue diferenciá-los. Nos terminais com o protocolo de teclado do
kitty ele consegue, e Ctrl+M move os arquivos marcados. Nos demais, pressione
`F9` `M`.

## Mudou na versão 0.8

Alt+A, Alt+D, Alt+R, Alt+T e Alt+U faziam o mesmo que Ctrl com a mesma letra.
[Notas de versão](releases.md) traz o significado antigo e o novo de cada tecla,
e as linhas para o `config.toml` que restauram os antigos.
