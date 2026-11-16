# Terminais e teclas

O Xylem recebe as teclas que o terminal envia. Alguns terminais reservam
teclas para si mesmos, e outros enviam os mesmos bytes para duas teclas
diferentes. Esta página lista esses casos e a tecla a usar como alternativa.

## Teclas retidas pelo terminal ou ambiente gráfico

| Tecla | Retida por | Use no lugar |
|---|---|---|
| F1 | GNOME Terminal (sua própria ajuda) | Alt+H |
| F11 | GNOME Terminal (tela cheia) | Alt+B, ou `'` para pular para um favorito |
| Alt+F10 | GNOME (maximiza a janela) | F12 |
| Alt+T | terminais com menu "Terminal" | associe outra tecla em `[keys.any]` |

## Ctrl+M

A maioria dos terminais envia o mesmo byte para Ctrl+M e Enter. O Xylem recebe
Enter. Para mover os arquivos marcados nesses terminais, pressione `F9` e depois
`M`, ou use Alt+M e pressione F6 no prompt para desativar a recriação dos caminhos.

Em um terminal com o protocolo de teclado do kitty, as duas teclas são
distintas e Ctrl+M move os arquivos marcados diretamente.

## O protocolo de teclado do kitty

Com esse protocolo, o terminal informa Ctrl e Alt pressionados e soltos por si
sós. O Xylem o ativa automaticamente quando o terminal confirma suporte.

| Com o protocolo | Sem o protocolo |
|---|---|
| Segurar Ctrl ou Alt exibe o menu CTRL ou ALT | F9 os exibe |
| Ctrl+M move os arquivos | Ctrl+M atua como Enter |

| Terminal | Protocolo |
|---|---|
| kitty, Ghostty, WezTerm, foot (Wayland), Alacritty recente | sim |
| iTerm2 | Ctrl+M move; segurar Ctrl ou Alt não foi verificado |
| GNOME Terminal, Ptyxis, Terminator, Tilix (baseados em VTE) | não |

## A área de transferência

`Y` e Ctrl+Y gravam na área de transferência. Em conexões SSH e dentro do tmux,
o Xylem usa a sequência OSC 52, que precisa ser aceita pelo terminal. No X11 e
no Wayland, o texto permanece na área de transferência após o Xylem ser fechado.

## Ícones e linhas de borda

Os ícones (F12, página 2) exigem uma fonte com os glifos do Nerd Fonts. Sem
eles, o terminal desenha um quadrado vazio no lugar de cada ícone. Se as
colunas ficarem desalinhadas com os ícones ativados, o terminal está tratando
caracteres de "largura ambígua" como largos; altere essa opção no terminal para
estreito (*narrow*) ou desative os ícones.

## Tamanho da tela

| Largura ou altura | Efeito |
|---|---|
| menos de 72 colunas | o nome e a versão saem do cabeçalho |
| estreita demais para uma linha de menu | as palavras do menu são abreviadas e as últimas entradas são substituídas por `…` |
| menos de 58 colunas ou 13 linhas | a tela de início não é exibida |

Veja também: [os menus](menus.md), [configuração](configuration.md),
[o terminal integrado](terminal.md).
