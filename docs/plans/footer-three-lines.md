# Plano: rodapé de três linhas, sem comandos invisíveis

Origem: conversa de 2026-11-13. O autor procurou o Ctrl+N (New date dos marcados) e
não achou; o N também não aparece no menu, embora funcione. Princípio pedido: **nenhum
comando invisível**. Aprovado: rodapé de três linhas.

## O levantamento

Comparando a tabela de teclas (`DEFAULT_KEYS`, `src/keymap.rs`) com os menus, estes
comandos funcionam e não aparecem em menu nenhum:

| Tecla | Comando | Janela |
|---|---|---|
| N | New date | árvore e lista |
| Ctrl+N | New date dos marcados | lista |
| A | Available | árvore |
| ? | estatísticas | ambas |
| `\` ou `/` | Treespec | ambas |
| F5 / Ctrl+F5 | compactar o arquivo / os marcados | lista |
| F7 | Autoview | lista |
| F12 (Alt+F10) | configuração | ambas |
| Ctrl+B | Batch | lista |
| Ctrl+B, Ctrl+F11 | criar bookmark | árvore / ambas |
| Ctrl+S | Search nos marcados | lista |
| Ctrl+V | View dos marcados | lista |
| Ctrl+F7 / F8 / F9 | marcar / desmarcar / inverter o ramo | árvore |
| Alt+Q | sair mudando para a pasta | ambas |

Além disso, o menu **REMOTE** é de antes da 0.7: não mostra Delete, Makedir, Move,
Rename, Attributes nem New date, que agora funcionam numa máquina remota.

## A regra

- Um comando é **visível** se pelo menos uma das suas teclas aparece em algum menu
  da janela onde ele vale (DIR, FILE, CTRL ou ALT, na árvore ou na lista).
- Ficam de fora, numa lista explícita e comentada:
  - **navegação:** setas, PgUp/PgDn, Home/End, Tab, Espaço, Backspace, `+` e `-`
    na árvore, Esc;
  - **apelidos de um comando visível:** F2 (= F3), Del (= D), F10 (= Q), Alt+H
    (= F1), Ctrl+Insert (= Y), Ctrl+Z (= O), F11 e `'` (= Alt+B), Alt+F10 (= F12),
    Shift+Del e Ctrl+Shift+Del (a confirmação do Delete já troca entre lixeira e
    permanente).
- Um **teste** percorre `DEFAULT_KEYS` e falha se algum comando não for visível nem
  estiver na lista. Tecla nova sem menu quebra o teste.

## O desenho

Três linhas em vez de duas, à custa de uma linha da árvore.

- **Linhas 1 e 2:** os comandos de letra, em ordem alfabética pela letra destacada,
  Quit por último (como no XTree). Quando não cabem numa linha, continuam na segunda.
- **Linha seguinte:** as dicas (no CTRL e no ALT) e as teclas de função e símbolos.
- Separador entre teclas: 3 espaços, como hoje. Tudo cabe em 120 colunas; abaixo
  disso, vale o encurtamento de `docs/plans/menus-narrow.md`.

```
DIR      Available Branch Compare Delete Filespec Invert Log Makedir
         New date Oops Rename Showall Tag Untag eXecute Yank Quit
         F1 help   * log branch   ? stats   \ treespec   F3 relog   Enter files   F8 split   F9 menus

FILE     Attributes Branch Copy Delete Edit Filespec Invert Move New date
         Open Rename Showall Tag Untag View eXecute Yank Quit
         F1 help   Esc tree   ? stats   \ treespec   F3 relog   F5 archive   F7 autoview   F8 split   F9 menus

CTRL     Attributes Bookmark Copy Delete Git Move Rename Showall Tag Untag Yank        (árvore)
          on tagged files here · Showall/Tag/Untag: all logged · Yank: paths to clipboard
         Ctrl+F7/F8/F9 tag/untag/invert branch   F9 next menu

CTRL     Attributes Batch Copy Delete Git Move New date Open Rename Search Tag Untag View Yank   (lista)
          on tagged files in list · Copy/Move: no paths · Yank: paths to clipboard
         F5 archive them   F9 next menu

ALT      Bookmarks Compare Duplicates File display Help Info Kbyte Prune Quit here Release Sort Usage
          Attributes/Delete of the branch: Prune · Quit here: leaves in this folder
         X terminal   O type tagged files in terminal   F10 setup   F9 next menu

ALT      Bookmarks Copy Duplicates File display Help Info Move Quit here Sort Usage   (lista)
          Copy/Move: the tagged files, with their paths
         X terminal   O type tagged files in terminal   F5 open archive   F10 setup   F9 next menu
```

(A segunda linha do ALT é um rascunho: o texto final sai na implementação.)

- **REMOTE** e **ARCHIVE** seguem o mesmo desenho, com o que vale em cada um; o
  REMOTE passa a mostrar o que a 0.7 permite (a conferir no código de
  `src/remote`).
- **GIT**, **TERMINAL**, **USAGE**, **DUPLICATES**, **VIEW**, a configuração, os
  bookmarks e os prompts têm teclas próprias, fora de `DEFAULT_KEYS`: passam pela
  mesma auditoria à mão, e só ganham a terceira linha se precisarem.

## Como, no código

1. **Menus como dados:** cada entrada diz o comando que dispara (`Command`), além do
   texto; o desenho continua o de hoje (`menu_words`, `menu_line`, `fit_menu`). É
   isso que permite o teste sem ler a tela.
2. **O rodapé principal com altura 3** (`draw_screen`); as outras telas ficam com 2
   até a auditoria dizer o contrário. `fit_menu` vale por linha.
3. **A distribuição das palavras** nas linhas 1 e 2: em ordem, enchendo a primeira
   até a metade do total (as duas linhas ficam parecidas), Quit por último.
4. **O teste** `every_command_is_in_a_menu`, com as listas de navegação e de
   apelidos.
5. **Testes de rodapé** existentes ajustados (linhas 18/19 viram 17/18/19); os de
   largura estreita valem por linha.
6. **Manual:** a página de teclas e o `tagging.md` (menus Ctrl e Alt) nos seis
   idiomas; notas da versão.
7. **Capturas:** todas mudam (o rodapé e uma linha a menos de árvore); refeitas com
   `run.sh` e conferidas.

## O que a auditoria das outras telas achou (2026-11-13)

- **USAGE:** Ctrl+T/U (marcar e desmarcar a pasta toda), Ctrl+D (apagar os
  marcados) e Shift+Del não apareciam: ganharam a terceira linha do rodapé.
- **VIEW** (rodapé próprio, de duas linhas): Shift+M (limpar as marcas da tabela)
  não aparece. Fica para o passo seguinte, junto com uma auditoria do viewer
  inteiro (as teclas estão em `src/viewer.rs`, fora de `DEFAULT_KEYS`).
- **ARCHIVE:** o menu oferecia Log e o terminal, que um arquivo compactado recusa.
  Agora os menus saem do mesmo filtro que recusa o comando (`App::refusal`), e o
  REMOTE e o ARCHIVE deixam de ser listas à mão.

## Duas linhas quando cabe (decidido em 2026-11-13)

Numa tela larga, a terceira linha só toma espaço da árvore. O rodapé tem **duas
linhas** quando todos os menus (DIR, FILE, CTRL e ALT, na árvore e na lista)
cabem em duas naquela largura: as palavras numa, as teclas na outra, com as dicas
antes delas se couberem (são explicações; saem primeiro). Senão, três. A decisão
é uma só para a largura (`footer_height`), para a árvore não pular ao segurar
Ctrl ou ao trocar de janela. A visão Usage tem sempre três.

## Fora deste passo

- Mostrar no menu a tecla que o usuário escolheu em `[keys]` em vez da padrão.
- O menu vertical para telas muito estreitas (`docs/plans/menus-narrow.md`).
