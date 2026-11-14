# Estudo: o Xylem para quem usa Vim

Origem: conversa de 2026-11-14. O autor pediu para imaginar como seria o Xylem
adaptado para usuários de Vim e registrar num plano para estudar.

**Estado: proposta para estudo. Nada aprovado, nada implementado.** As decisões em
aberto estão marcadas com **(?)**, com a recomendação, e reunidas no fim.

## A ideia em uma frase

O Xylem já vive num "modo Normal" permanente: nas listas ninguém digita texto, cada
letra é um comando. Adaptar para Vim não é criar modos, é **trocar o vocabulário**:
sai a letra mnemônica do XTree (`C`opy, `D`elete, `T`ag, com Ctrl para os marcados) e
entra a gramática do Vim (movimento, seleção visual, `yy` `dd` `p`, prefixos `g` e
`z`, linha de comando `:`). Os comandos (`Command`, `src/input.rs`) são os mesmos; muda
a tabela de teclas e o rodapé.

## O que já é Vim hoje

- `/` abre o Treespec (o comentário em `src/keymap.rs` já cita o vim).
- `'` + letra pula para um favorito: é o `'a` das marcas do Vim.
- `Y` é "Yank" no rodapé (copia o caminho para a área de transferência).
- No viewer: `/` busca, `n` próxima ocorrência, `g` e `:` vão para a linha.
- `q` sai. O editor (`E`) recebe o tema do Xylem no vim/nvim.
- **Letras livres** nas duas janelas: `g h j k p w z`. Na lista, também `l`; na árvore,
  também `e` e `v`. Ou seja, `h j k` e os prefixos `g` e `z` não tiram nada de ninguém.

## Três níveis

| Nível | O que é | Custo | Para quem |
|---|---|---|---|
| 0 | O `[keys]` do `config.toml`, como está | nenhum (só documentar) | quem quer só `j`/`k` |
| 1 | Movimento Vim sempre ligado, onde a tecla está livre | pequeno | todos; não muda nada do XTree |
| 2 | Conjunto de teclas `vim`, escolhido na configuração | grande | quem pensa em Vim |

### Nível 0: o que dá para fazer hoje

```toml
[keys.any]
j = "down"
k = "up"
g = "home"

[keys.tree]
h = "collapse"

[keys.files]
h = "left"
l = "right"
```

Limites: uma tecla, um comando (não há `gg`, `dd`, `zo`); sem contagem (`5j`); a
tabela não distingue `g` de `G` (as letras são normalizadas para minúsculas, só o
Delete conta o Shift); e o viewer, os menus de escolha e a lista do histórico têm as
teclas fixas no código.

### Nível 1: movimento Vim para todos

Só teclas livres, ligadas por padrão, sem opção nova:

| Tecla | Faz | Observação |
|---|---|---|
| `j` `k` | desce, sobe | em toda lista: árvore, arquivos, viewer, histórico, menus de escolha |
| `h` | árvore: recolhe / vai ao pai; lista: coluna à esquerda | o mesmo que `←` |
| `l` | lista: coluna à direita | na árvore `L` continua sendo Log |
| `gg` `G` | primeira e última linha | pede sequência de teclas e maiúscula distinta |
| `Ctrl+E` `Ctrl+Y`… | **não** | `Ctrl+Y` já é Yank dos marcados |

`Ctrl+D`/`Ctrl+U` (meia página) e `Ctrl+F`/`Ctrl+B` ficam de fora neste nível: são
Delete e Untag dos marcados, e Batch. No viewer, `h` (hex) e `l` (números de linha)
estão ocupados; lá entram só `j` `k` `gg` `G`.

### Nível 2: o conjunto `vim`

`[options] keys = "vim"` (o padrão continua `"xtree"`). O `[keys]` do usuário se
aplica por cima do conjunto escolhido. **Regra de convivência:** o conjunto `vim` só
troca as letras sozinhas e o Ctrl+letra. As teclas de função (F1 a F12) e o Alt+letra
(as ferramentas: Sort, Usage, Duplicates, terminal…) valem iguais nos dois conjuntos,
então o manual e as capturas continuam servindo.

## A gramática do conjunto `vim`

### Movimento

| Tecla | Faz |
|---|---|
| `h j k l` | as setas, em todo lugar (na árvore, `h` recolhe e `l` expande) |
| `gg` `G` | primeira, última |
| `Ctrl+D` `Ctrl+U` | meia página |
| `Ctrl+F` `Ctrl+B` | página |
| `5j`, `12G` | contagem; `12G` vai para a linha 12 |
| `/` `n` `N` | Treespec e as ocorrências seguinte e anterior (a conferir: o Treespec hoje tem "seguinte"?) |
| `m`+letra, `'`+letra | grava e pula para o favorito (o `'` já existe); `:marks` abre a lista (F11) |

### A árvore são dobras

O ponto em que o Vim cai melhor: expandir e recolher diretórios é o `z` das dobras.

| Tecla | Vim | Xylem |
|---|---|---|
| `zo` | abre a dobra | Expand (lê e expande) |
| `zc` | fecha | Collapse |
| `za` | alterna | expande ou recolhe |
| `zR` | abre todas | Expand all (`*`) |
| `zM` | fecha todas | recolhe o ramo inteiro |
| `zd` | apaga a dobra | Release (tira o ramo da memória; não apaga nada no disco) |

### Marcar é o modo Visual

| Tecla | Faz | Hoje |
|---|---|---|
| `Espaço` ou `t` | marca ou desmarca o item e desce | `T` / `U` |
| `V` | começa um intervalo: `j`/`k` estendem, `Esc` termina e **mantém** as marcas | não existe |
| `ggVG` | marca tudo | `Ctrl+T` |
| `~` | inverte as marcas (no Vim, inverte maiúsculas) **(?)** | `I` |
| `:untag`, `:tag *.rs` | desmarca tudo; marca por filespec | `Ctrl+U` |

O intervalo com `V` é uma função nova de verdade (hoje não há como marcar "daqui até
ali" sem apertar `T` em cada um) e poderia entrar também no conjunto `xtree`.

**Sobre o que o verbo age (?):** no XTree a letra sozinha é o item do cursor e o Ctrl
são os marcados; nunca há dúvida. No Vim o Ctrl está ocupado com movimento. Como no
vifm e no ranger: **se há marcados na lista, o verbo age neles; senão, no item do
cursor**. O risco é apagar 40 arquivos achando que era um; a defesa é o que já existe
(a confirmação diz quantos, o Delete vai para a lixeira, `u` desfaz) mais o rodapé
mostrando o alvo antes da tecla (`dd: 40 marcados`).

### Verbos

| Tecla | Faz | Hoje |
|---|---|---|
| `yy` | guarda os arquivos para copiar | — |
| `dd` | guarda para mover (recorta; nada acontece até o `p`) **(?)** | — |
| `p` | põe no diretório atual o que foi guardado, com a confirmação de sempre | — |
| `D` | Delete (lixeira) | `D` |
| `u` | desfaz: Oops, devolve da lixeira | `Ctrl+Z` |
| `cw` ou `cc` | Rename (com marcados, a máscara) | `R` / `Ctrl+R` |
| `yp` | Yank do caminho para a área de transferência | `Y` |
| `gx` | Open com o programa do sistema | `O` |
| `e` | Edit | `E` |
| `Enter` | na lista, View (a opção `enter_views` ligada); `Esc` volta à árvore | `V` |
| `Ctrl+G` | Info (no Vim, mostra dados do arquivo) | `Alt+I` |
| `Ctrl+L` | Relog (no Vim, redesenha) | `F3` |
| `!` | eXecute: abre o prompt de comando | `X` |
| `ZZ` | sai ficando no diretório (`quit_cd`) | `Alt+Q` |
| `ZQ` | sai | `Q` |

`yy`/`dd`/`p` são **um jeito novo de copiar e mover**: hoje o `C` pergunta o destino
num prompt; no Vim a pessoa guarda, anda até o destino e põe. Pede uma lista interna
de "guardados" (caminhos + copiar ou mover), visível no rodapé (`3 guardados para
copiar`). O prompt de destino, com histórico e favoritos, continua em `:copy` e
`:move`. Com o split, `p` no outro painel é o caso de uso natural.

`dd` como "recortar" segue o ranger; no vifm `dd` apaga. **(?)** Recomendo recortar:
errar um `dd` não custa nada.

Operador + movimento (`d3j`, `yG`) fica para um segundo passo: `yy`/`dd` com contagem
(`3dd`) e o `V` cobrem quase tudo.

### Janelas

| Tecla | Faz | Hoje |
|---|---|---|
| `Ctrl+W s` / `Ctrl+W v` | divide a tela | `F8` |
| `Ctrl+W w`, `Ctrl+W h/l` | troca de painel | `Tab` |
| `Ctrl+W j/k` | troca entre árvore e lista | `Enter` |
| `Ctrl+W o` | volta a um painel só | `F8` |

### A linha de comando `:`

Um prompt com o histórico e a lista que já existem (`LineInput`), que aceita o nome do
comando da tabela (`COMMANDS`) e alguns apelidos do Vim. É também a saída para tudo o
que não ganhou tecla, e serviria ao conjunto `xtree` como "paleta de comandos".

| Comando | Faz | Hoje |
|---|---|---|
| `:q`, `:q!` | sai | `Q` |
| `:e caminho`, `:cd caminho` | Log | `L` |
| `:e!` | Relog | `F3` |
| `:sort nome` | Sort | `Alt+S` |
| `:filter *.rs` | Filespec | `F` |
| `:split`, `:only` | divide, um painel | `F8` |
| `:!cmd` | eXecute, com `%f` e `%t` como hoje | `X` |
| `:terminal` | terminal embutido | `Alt+X` |
| `:grep texto` | Search nos marcados | `Ctrl+S` |
| `:mkdir`, `:chmod`, `:touch` | Make dir, Attributes, New date | `M`, `A`, `N` |
| `:branch`, `:showall` | as vistas | `B`, `S` |
| `:usage`, `:duplicates`, `:compare`, `:batch`, `:git` | as ferramentas | Alt e Ctrl |
| `:set theme=nord`, `:set` | uma opção; sem argumento, a tela de configuração | `F12` |
| `:help`, `:marks` | ajuda, favoritos | `F1`, `F11` |
| `:42` | linha 42 | — |

`Tab` completa o nome do comando. Sem abreviações livres do Vim (`:so`, `:sp`): só os
apelidos da tabela, para a ajuda poder listar todos.

### O viewer

Já é parecido com o `less`. No conjunto `vim`: `j k` rolam, `h l` deslocam para os
lados (e andam pelas células na tabela), `gg` `G`, `Ctrl+D/U/F/B`, `0` e `$` para o
começo e o fim da linha, `/` e `?` buscam para frente e para trás, `n` `N`. O que
perde a letra vai para o `:set`: `:set hex` (hoje `H`), `:set nu` (`L`), `:set wrap`
(`W`). `:42` já funciona.

### Os prompts

Ficam como estão, que é o "modo de inserção": digitar, `Esc` cancela, `Enter`
confirma. Entram só `Ctrl+W` (apaga a palavra) e `Ctrl+U` (apaga a linha), que quem
usa Vim ou shell espera. Edição modal dentro do prompt (`Esc` e depois `b`, `cw`):
**não**; o `Esc` precisa continuar cancelando.

## O rodapé

O rodapé de hoje destaca a letra de cada palavra e troca de menu quando se segura Ctrl
ou Alt. No conjunto `vim` a mesma ideia serve aos prefixos: **apertar `g`, `z`, `d`,
`y` ou `Ctrl+W` troca o rodapé pelo menu do prefixo**, como hoje acontece ao segurar o
Ctrl. Não há tempo de espera: o prefixo fica pendente até a próxima tecla, e `Esc`
desiste.

```
ARQUIVOS  hjkl mover  t marcar  V intervalo  yy copiar  dd recortar  p pôr  D apagar  cw renomear
          Enter ver   e editar  gx abrir     u desfazer  ! comando   / buscar  : comando   g… z…

z         o expandir   c recolher   a alternar   R expandir tudo   M recolher tudo   d liberar

d         d recortar: 3 marcados (fotos/a.jpg, fotos/b.jpg, …)                     Esc desiste

:sort n▌
          name   date   size   extension                                          Tab completa
```

Segurar o Alt continua mostrando o menu ALT de hoje, igual.

## O que muda no código

- **`src/keymap.rs`**
  - Sequências: `gg`, `dd`, `zo`, `Ctrl+W v`. A tabela vira uma árvore de prefixos e o
    app guarda o prefixo pendente.
  - Maiúsculas distintas (`G`, `V`, `D`, `ZZ`) só no conjunto `vim`; o `xtree`
    continua sem diferenciar, como o XTree.
  - Contagem: dígitos antes do comando, com teto (ver Segurança).
  - Dois conjuntos de padrões (`DEFAULT_KEYS` e um `VIM_KEYS`) e a escolha em
    `Options` (`src/config.rs`), com a tela de configuração (`src/settings.rs`).
- **`src/input.rs`, `src/app/`**: comandos novos: guardar para copiar/mover e pôr,
  intervalo visual, alternar marca, recolher tudo, alternar dobra, a linha `:`.
- **`src/ui.rs`**: as listas de palavras do rodapé (`TREE_WORDS`, `FILE_WORDS`,
  `CTRL_*`) são fixas e pensadas para "uma letra por palavra". O conjunto `vim` pede
  listas próprias no formato `tecla palavra` e os menus de prefixo. É a parte mais
  trabalhosa, junto com os testes de largura (`menus-narrow.md`).
- **`src/viewer.rs`**: as teclas estão num `match` fixo; precisam passar por uma
  tabela para ter dois conjuntos.
- **Ajuda (F1)**: sai da tabela de teclas, então acompanha; precisa saber mostrar
  sequências.
- **Testes**: o teste "todo comando está num menu" passa a rodar para os dois
  conjuntos; cada sequência; o `xtree` sem nenhuma mudança de comportamento.
- **Manual**: uma página nova, `vim.md`, com a tabela "XTree → Vim" (como a
  `ztreewin.md`), nas seis línguas, por último. As outras páginas continuam
  descrevendo o conjunto `xtree`.

## Segurança

Nenhum leitor novo de arquivo externo. O que entra é texto digitado pelo usuário:

- A linha `:` só aceita nomes da tabela de comandos; o que não reconhece é recusado
  com mensagem, nunca passado ao shell. `:!` é o mesmo caminho do `X`, com o mesmo
  escape de `%f` e `%t`.
- Contagem com teto (por exemplo 9999) e aritmética saturada: `99999999999j` não pode
  estourar nem travar.
- `p` usa as mesmas rotinas de Copy e Move (`src/ops`), com as mesmas confirmações e
  os mesmos limites; a lista de guardados é revalidada na hora do `p` (o arquivo pode
  ter mudado ou sumido).
- `keys = "..."` no `config.toml` com valor desconhecido é erro de configuração, como
  as outras opções.

## Etapas sugeridas

Cada uma é um PR que se sustenta sozinho; dá para parar depois de qualquer uma.

1. **Nível 1**: `j k h l` nas listas e no viewer. Pequeno, sem opção nova.
2. **Sequências e maiúsculas na tabela de teclas**, com o menu de prefixo no rodapé;
   `gg`/`G` para todos. É a base de tudo o que vem depois.
3. **Intervalo visual** (`V`) e **guardar/pôr** (`yy` `dd` `p`) como comandos, ainda
   sem tecla no `xtree` (ou com teclas livres **(?)**).
4. **A linha `:`**, útil também no `xtree`.
5. **O conjunto `vim`**: a tabela, o rodapé, o viewer, a opção, a página do manual.
6. Depois, se fizer falta: operador + movimento, `?` para trás no Treespec.

## Decisões em aberto

1. **Vale o nível 2, ou os níveis 0 e 1 bastam?** Recomendo fazer as etapas 1 a 4,
   que melhoram o Xylem para todos, e decidir a 5 depois de usá-las.
2. **Com marcados, o verbo age neles sem Ctrl?** Recomendo sim, só no conjunto `vim`,
   com o alvo no rodapé.
3. **`dd` recorta ou apaga?** Recomendo recortar, e `D` apaga.
4. **`V` e `yy`/`p` também no `xtree`?** Recomendo o `V` sim (função que falta); o
   guardar/pôr só pela linha `:` ou por tecla configurada.
5. **A linha `:` aceita os nomes em inglês da tabela (`make_dir`) ou nomes próprios
   (`mkdir`)?** Recomendo os dois: o nome da tabela sempre vale, os apelidos do Vim
   por cima.
6. **`~` para inverter as marcas**, ou só `:invert`?

## Fora deste estudo

- Registradores nomeados (`"ayy`), macros (`q`), repetir (`.`): o Batch já cobre o
  "fazer o mesmo em muitos".
- Edição modal dentro dos prompts.
- Abas (`gt`): o Xylem tem dois painéis, não abas.
- Mudar as teclas do terminal embutido: lá tudo vai para o shell.
