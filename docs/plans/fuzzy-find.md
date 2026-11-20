# Plano: busca aproximada ao digitar (`|`), e o zoxide como apêndice

**Proposta para a 0.9.0** (pedida em 2026-11-20). Nada aprovado para código.

## Objetivo

Achar uma pasta ou um arquivo digitando pedaços do nome, em qualquer nível da
árvore: `cfgxy` acha `config/xylem`, `rdme` acha `README.md`. O cursor vai até
o escolhido, e o que se marcar ali fica marcado na lista.

Hoje o Xylem tem três buscas, e nenhuma faz isto:

- o **Treespec** (`/`, `\`) anda pelo caminho, nível por nível, pelo começo do
  nome;
- o **Filespec** escolhe os arquivos da lista por curinga, data e tamanho;
- a **caixa de busca** (`P`) acha arquivos pelas palavras do conteúdo.

A página de comparações lista, entre o que falta do ZTreeWin, a busca ao
digitar com prompt (`|`). Esta é a versão do Xylem dela: a mesma tecla, com
busca aproximada em vez de pelo começo do nome.

Feita dentro do Xylem, sem programa externo: os nomes já estão na memória, a
tela é a do Xylem (tema, mouse, rodapé), os nomes passam pelo mesmo tratamento
de sempre, e a marcação é o `T` de sempre.

## Na tela

- **`|`** abre um prompt sobre a janela atual, com a lista dos resultados
  acima dele, do melhor para o pior, como a caixa de busca (`P`).
- **Na árvore**: as pastas logadas, pelo caminho a partir da raiz.
- **Na janela de arquivos**: os arquivos da lista atual (o diretório, o Branch
  ou o Showall, com o Filespec e o `W` que estiverem valendo), pelo nome; no
  Branch e no Showall, também pelo caminho.
- **↑ ↓** escolhem o resultado; o cursor da árvore ou da lista acompanha,
  como no Treespec.
- **Enter** confirma: o cursor fica no escolhido e o prompt fecha.
- **Esc** volta para onde estava.
- **Ctrl+T no prompt** marca ou desmarca o resultado sob o cursor sem fechar;
  **Alt+T** marca todos os resultados. Depois, Ctrl+C, Ctrl+M e o Batch agem
  nos marcados, como sempre. (Teclas a conferir no `keymap.rs`: no prompt, `T`
  é texto.)
- As letras que casaram aparecem destacadas em cada resultado.
- O rodapé diz quantos resultados há e em quantos nomes a busca procurou.

**Decidido em 2026-11-20: o que se busca depende da janela.** Na árvore, só
pastas; na janela de arquivos, só os arquivos da lista atual. Achar um arquivo
em qualquer lugar é `S` (Showall) e `|`. Uma lista que misturasse os dois daria
ao Enter dois destinos (o cursor da árvore ou a troca de janela e de lista),
deixaria sem sentido o Filespec e o `W` (que valem para a lista de arquivos) e
buscaria em muito mais nomes. Para depois da etapa 2, se fizer falta: no
Showall, o Enter também leva o cursor da árvore até a pasta do arquivo.

## Como casa

- **As letras na ordem**, não necessariamente juntas, sem diferença entre
  maiúsculas e minúsculas; com uma maiúscula na busca, passa a diferenciar
  (`Rd` não acha `readme`).
- **Espaços separam termos**: todos têm de casar, em qualquer ordem (`xy docs`
  acha `xylem/docs` e `docs/xylem`).
- **Pontuação**, do que mais conta para o que menos conta:
  - letras seguidas;
  - letra no começo do nome, depois de `/`, `_`, `-`, `.` ou de espaço, ou numa
    troca de minúscula para maiúscula (`fileSpec`);
  - o casamento no nome vale mais que nas pastas acima dele;
  - empatados, ganha o caminho mais curto e, depois, a ordem da lista.
- **Depois, se fizer falta**: `'termo` para exato, `^` para começo, `$` para
  fim, `!` para excluir. Fora da etapa 1.

Nomes que não são UTF-8 casam pela sua forma com substituição (`U+FFFD`), como
são mostrados.

## Desempenho

A árvore pode ter até 2.000.000 de nós.

- **Em segundo plano**: a busca roda em blocos, e a lista se completa enquanto
  ela anda; digitar de novo cancela a busca anterior. (Em blocos no `tick`,
  não numa thread: ver a etapa 2 em detalhe.)
- **Só os melhores**: guarda os 500 melhores resultados (um heap), não todos.
- **Busca que só cresce**: quando o texto novo é o anterior com mais letras,
  procura só entre os que já casavam.
- **Metas a medir na etapa 1**: resposta visível em até 50 ms com 100.000
  nomes; com 2.000.000, os primeiros resultados no mesmo tempo e o resto
  chegando.

**Escrever o casamento ou usar uma crate**: decidido na etapa 1, medindo.
Critérios para uma crate: licença compatível com a MIT do Xylem, sem `unsafe`
ou com pouco, e o quanto acrescenta ao binário. O casamento próprio é pequeno
(algumas centenas de linhas); a crate só vale se for claramente mais rápida
com milhões de nomes.

## Onde funciona

Em qualquer árvore logada: local, numa máquina remota (SSH), dentro de um
arquivo compactado e numa montagem do rclone. Nada é lido do disco nem da rede:
só os nomes já na memória.

## Segurança

- **Nada é lido** fora da memória; nenhum programa é executado.
- **Os nomes** aparecem pelo mesmo tratamento do resto do Xylem: controles e
  caracteres de direção do texto marcados, nunca obedecidos.
- **A busca** tem até 256 caracteres; os destaques de letras casadas são
  calculados sobre o nome tratado, para que um nome com caracteres
  invisíveis não desalinhe o destaque.

## Etapas

1. **O casamento**: as regras acima, a pontuação, os testes (casos de
   pontuação, maiúsculas, termos, nomes não UTF-8, nomes com controles), e a
   medição com 100.000 e 2.000.000 nomes; a decisão entre código próprio e
   crate.
2. **O prompt**: `|` na árvore e na janela de arquivos, a lista, o cursor
   acompanhando, Enter e Esc, em segundo plano.
3. **Marcar**: Ctrl+T e Alt+T no prompt.
4. A página do manual (em inglês), ligada no índice, em `keys` e no Treespec;
   as notas da versão; e a linha da busca ao digitar na página de comparações,
   revista (o `|` passa a existir, com busca aproximada).

**Feito na etapa 1** (2026-11-20): `src/fuzzy.rs`, com código próprio.

- A consulta: até 256 caracteres, termos separados por espaço, cada termo
  com maiúscula passa a diferenciar maiúsculas (cada um por si). `narrows`
  diz quando o texto novo só cresceu no fim, para a etapa 2 procurar só entre
  os que já casavam.
- O casamento: primeiro um teste rápido (as letras aparecem na ordem?), e só
  quem passa é pontuado. O termo é procurado primeiro no nome, e só depois no
  caminho todo. A janela é a do primeiro casamento que termina mais cedo,
  apertada a partir do fim. Pontuação: 16 por letra; bônus de começo de
  palavra (10 no início ou depois de espaço, 9 depois de `/`, 8 depois de
  pontuação, 7 na troca para maiúscula ou para dígito), contado duas vezes na
  primeira letra do termo; letras seguidas herdam o bônus do começo do trecho
  (pelo menos 4); lacuna custa 3 e depois 1 por letra; o termo no nome ganha
  mais 32.
- As posições são índices de caracteres do texto mostrado, nunca de bytes.
- `Top`: os 500 melhores num heap; empate vai para o texto mais curto e,
  depois, para a ordem da lista.

Medido (otimizado, sem LTO, caminhos sintéticos de 3 a 9 níveis):

| Nomes | `cfgxy` | `src view` | `rdme` | `zzzz` (nada) | `f` (todos) |
|---|---|---|---|---|---|
| 100.000 | 5,5 ms | 10,5 ms | 13,1 ms | 4,2 ms | 12,3 ms |
| 2.000.000 | 111 ms | 214 ms | 273 ms | 88 ms | 238 ms |

Com 100.000 nomes a meta de 50 ms sobra várias vezes. Com 2.000.000 a busca
inteira passa da meta, como o plano previa: a etapa 2 a faz em blocos numa
thread (uns 400.000 nomes cabem nos primeiros 50 ms), e a busca que só cresce
procura só entre os que já casavam.

**O caminho de cada pasta, medido** (2026-11-20, `tree_paths_timing`, mesmo
perfil). Três maneiras, numa árvore virtual montada por `Tree::from_listing`,
com a busca incluída:

| Pastas | (a) `Tree::path` por pasta | (b) um percurso, prefixo num buffer | (c) caminhos guardados: montar + buscar | Memória de (c) |
|---|---|---|---|---|
| 321.008 | 188–260 ms | 74–120 ms | 220 ms + 44–73 ms | 22 MB |
| 1.756.522 | 1.600–1.800 ms | 474–664 ms | 1.400 ms + 249–423 ms | 122 MB |

- **(a)** sobe até a raiz e monta um `PathBuf` para cada pasta: três vezes
  mais lento que (b). Fica fora.
- **(c)** busca 1,7 vez mais rápido, mas custa uns 70 bytes por pasta, leva
  1,4 s para montar com 1,7 milhão de pastas e teria de ser refeito sempre
  que a árvore muda. Fica fora: a memória é o que falta nesta máquina, e a
  busca em blocos já esconde a diferença.
- **(b)** é o escolhido: um percurso em profundidade com uma pilha
  `(pasta, tamanho do prefixo)`, que corta o buffer, acrescenta o nome e
  busca. Custa uns 0,1 µs por pasta acima da busca, não guarda nada e dá a
  ordem da árvore, que é a do desempate. Em 50 ms cabem umas 170.000 pastas.
- **A busca que só cresce** guarda os ids de todas as pastas que casaram (não
  só os 500 melhores) e monta o caminho só delas com `Tree::path`: são poucas
  quando a busca já estreitou, e, no pior caso (uma letra que casa com
  tudo), 2.000.000 de ids são 16 MB, só enquanto o prompt está aberto.

**Código próprio, sem crate**: cabe na meta com folga, tem cerca de 300 linhas
sem `unsafe` e não acrescenta dependência. As crates conhecidas não foram
medidas: a mais rápida (`nucleo-matcher`) é MPL-2.0, fora do critério de
licença, e a `fuzzy-matcher`, MIT, não tem manutenção. Se a etapa 2 mostrar
lentidão, o primeiro passo é um caminho só de bytes para nomes ASCII.

## Etapa 2 em detalhe (proposta, 2026-11-20)

### Em blocos no `tick`, não numa thread

A árvore é da `Pane`, na thread principal, e muda enquanto o prompt está
aberto (log em segundo plano, leituras de uma montagem do rclone, Relog). Uma
thread precisaria de uma cópia dos nomes (os 122 MB medidos em (c)) ou de uma
trava na árvore. Todo trabalho longo do Xylem já anda em blocos no `tick`, com
orçamento de tempo (o log, os arquivos compactados, o índice), e o laço
principal só chama o `tick` quando não há tecla esperando. A busca faz o
mesmo:

- **O estado de uma passada** guarda a pilha `(pasta, tamanho do prefixo)` e o
  buffer do percurso (b), o `Top` dos 500 melhores, os ids de todos que
  casaram e quantos nomes já viu. Cada `tick` avança até 40 ms, dos 50 que o
  laço dá, e devolve o resto para o que mais estiver andando.
- **A busca vai primeiro no `tick`**, sem retornar, como a caixa de busca: um
  log ou uma cópia em andamento continua por trás.
- **Cada tecla que muda o texto** descarta a passada e começa outra. Se o texto
  novo só cresceu (`narrows`) e a passada anterior terminou, a nova percorre
  só os ids que casaram, e não a árvore.
- **Sem espera entre teclas** (a caixa de busca espera 150 ms): a primeira
  fatia de 40 ms já mostra resultados, e a próxima tecla chega antes da fatia
  seguinte.
- **Se a árvore mudar** (`Tree::version`) durante a passada ou depois dela, a
  passada recomeça do zero, sem o atalho dos ids: uma pasta liberada (`-`)
  continua na arena, mas fora da árvore, e não pode ser achada. Numa montagem
  do rclone, que muda a cada leitura, recomeça no máximo uma vez por segundo.

### O que se busca

- **Na árvore**: todas as pastas que se alcançam pelos filhos a partir da
  raiz, menos a raiz, pelo caminho a partir dela (`docs/plans`). As ocultas só
  quando a árvore as mostra (`show_hidden`).
- **Na janela de arquivos**: os arquivos de `pane.files`, já filtrados pelo
  Filespec, pelo `W`, pelo Tags-Only e pelos ocultos.
  - No **diretório**: o nome.
  - No **Branch**: o caminho a partir da pasta do Branch.
  - No **Showall**: o caminho a partir da raiz.
  - O caminho de cada pasta é montado uma vez por passada, num mapa
    `pasta → texto` que vive só enquanto ela anda, porque a lista ordenada
    mistura arquivos de pastas diferentes.
- **Se a lista de arquivos for refeita** com o prompt aberto (Relog, mudança
  vista no disco), a passada recomeça, como na árvore. Uma `FileRef` guarda
  índices no vetor da pasta, que uma releitura muda.

### Na tela

- **O prompt** fica no rodapé, como o Treespec: `FUZZY: `.
- **A lista de resultados** ocupa a parte de baixo da janela ativa: até 12
  linhas, e nunca mais que metade da janela. A janela encolhe acima dela, para
  que o cursor que acompanha o resultado continue à vista. Na tela dividida,
  só a janela ativa encolhe.
- **Cada linha** mostra o caminho com as letras que casaram destacadas, no
  estilo de destaque que a busca já usa. Quando o caminho não cabe, é cortado
  à esquerda (`…/xylem/docs/plans`), para o nome continuar visível, e os
  destaques acompanham o corte. Os nomes passam pelo `bidi::sanitize`, como no
  resto da tela.
- **A borda da lista** diz `12 de 3.418 (buscando… 41%)` durante a passada e
  `500 melhores de 3.418` quando há mais resultados que os guardados. O
  progresso é a parte dos nomes já vista.

### Teclas e mouse

- **`|`** (comando `fuzzy_find`, nas duas janelas) abre o prompt. Ele entra no
  menu do rodapé, porque o teste `every_command_is_in_a_menu` exige, e na lista
  do F1.
- **↑ ↓, PgUp PgDn, Home End** movem a escolha na lista. As teclas de edição
  (←, →, Backspace, Ctrl+←...) ficam com o texto. Não há histórico no ↑ nesta
  etapa.
- **O cursor da janela** vai até o escolhido a cada ↑ ↓ e quando uma passada
  termina. Durante a passada a lista se reordena, mas o cursor não pula a cada
  fatia. Na árvore o movimento é o `show_node`, como no Treespec; na janela de
  arquivos, a seleção da lista.
- **Enter** confirma: o cursor fica no escolhido e o prompt fecha. Sem
  resultados, Enter não faz nada e o rodapé diz que nada casou.
- **Esc** volta à pasta de origem (a árvore) ou à seleção de origem (os
  arquivos), como o Treespec faz com `origin`. As pastas que o `show_node`
  abriu no caminho ficam abertas, como no Treespec.
- **O mouse**: um clique escolhe, um clique duplo confirma, e a roda rola a
  lista, pelo mesmo sistema de `Spot` da caixa de busca.

### Onde fica o código

- `src/fuzzy.rs`: a passada (`Pass`), que avança por orçamento sobre uma fonte
  de nomes: as pastas da árvore, os arquivos da lista ou os ids guardados.
- `src/app/`: o `PromptKind::Fuzzy { origin }`, as teclas, o `tick` e o
  Enter/Esc, perto do Treespec.
- `src/ui.rs`: a janela encolhida e a lista.
- `src/keymap.rs`, `src/input.rs`: a tecla e o comando.

### Testes

- **A passada**: dá os mesmos resultados de uma busca inteira, cortada em
  fatias de qualquer tamanho; o atalho dos ids dá os mesmos resultados que uma
  passada do zero; respeita os ocultos.
- **O app** (`src/app/tests.rs`): `|`, digitar, ↑ ↓ e o cursor indo junto,
  Enter e Esc, na árvore, no diretório, no Branch e no Showall; uma tecla no
  meio da passada recomeça a busca; um Relog durante a passada recomeça do
  zero; uma pasta liberada não aparece.
- **A tela**: o corte à esquerda com os destaques alinhados, e um nome com
  controles e caracteres de direção, marcados e com o destaque no lugar.
- **A medição**: o tempo até a primeira fatia, com 1.756.522 pastas.

### Feito na etapa 2 (2026-11-20)

Como proposto acima, com estes detalhes:

- **`fuzzy::Pass`**: a passada, com três cursores (o percurso da árvore, a
  lista de arquivos do começo ao fim, os ids que casaram antes), `step` por
  orçamento (o relógio é olhado a cada 64 itens), `narrows_to` e
  `into_narrowed`. `Rank` ganhou o `id` do que casou. `result_text` monta o
  texto de um resultado para a tela. `Tree::node_count` dá o total para o
  progresso.
- **`Pane::files_built`** sobe a cada `refresh_files`: é o que diz que a lista
  de arquivos foi refeita.
- **`src/app/fuzzy_cmd.rs`**: o `Fuzzy` (o estado do prompt) e as teclas, o
  `tick`, a volta do Esc e o clique. A escolha é guardada pelo id do
  resultado, não pela posição, porque o `show_node` lê a pasta escolhida se
  ela ainda não foi lida, isso muda a `version` da árvore e a busca recomeça;
  pelo id, a mesma pasta continua escolhida e o cursor não pula para o
  primeiro.
- **`src/ui.rs`**: `fuzzy_split`, `draw_fuzzy` e `match_spans`. O destaque é a
  cor das teclas em negrito; na linha escolhida, negrito e sublinhado, para
  não brigar com a cor da seleção.
- **Home e End** escolhem o primeiro e o último resultado, como no plano: no
  texto, o cursor anda com ← e →.

Medido com o percurso de verdade (`tree_paths_timing`, otimizado, sem LTO),
em 1.756.522 pastas, fatias de 40 ms:

| Busca | 1ª fatia vê | Passada inteira | Casaram | Uma letra a mais |
|---|---|---|---|---|
| `cfgxy` | 151.295 pastas | 502 ms, 13 fatias | 34.466 | 31 ms |
| `rdme` | 122.623 | 677 ms, 17 fatias | 635.967 | 580 ms |
| `f` | 133.375 | 543 ms, 14 fatias | 352.632 | 286 ms |
| `f d` | 125.247 | 565 ms, 15 fatias | 304.909 | 281 ms |

- **A primeira fatia** chega aos resultados em 40 ms, como a meta pedia.
- **O atalho dos ids** ajuda pouco quando muitas pastas casaram: montar o
  caminho subindo pelos pais custa mais por pasta (0,9 µs) que o percurso
  (0,35 µs). Ainda assim nunca é mais lento que uma passada nova, e a passada
  anda em fatias. Se fizer falta, o atalho pode percorrer a árvore e pular o
  que não está entre os ids.
- **Visto na tela** (tmux, 110×32, no próprio repositório): na árvore,
  `docs plan` dá `docs/plans` primeiro; no Showall, `src fzrs` dá
  `src/fuzzy.rs` e `src/app/fuzzy_cmd.rs` primeiro, com `500 best of 1,805`
  na borda.

### Para a etapa 3: um conflito de teclas

`Ctrl+T` e `Alt+T` já são "marcar todos" fora do prompt. Antes da etapa 3,
decidir se, no prompt, `Ctrl+T` marca todos os resultados (o mesmo sentido de
fora) e outra tecla marca só o escolhido (Insert ou Ctrl+Espaço), em vez do
que está em "Na tela". Há um precedente no sentido oposto: na caixa de busca
(`P`), `Ctrl+T` marca o resultado escolhido.

Marcar também muda a `version` da árvore, o que hoje faz a busca recomeçar.
A etapa 3 precisa separar as duas coisas, para que marcar não recomece uma
passada de 2.000.000 de pastas.

## Fora deste plano

- O "modo ativo" (letras que buscam sem prompt): no Xylem as letras são
  comandos, e Shift+letra já leva ao próximo nome que começa com ela.
- A sintaxe estendida (`'`, `^`, `$`, `!`) na etapa 1.
- Buscar no conteúdo: é o `P`.

## Apêndice: zoxide (opcional)

Para quem usa o zoxide no shell: o valor dele é o banco que aprendeu com o uso,
e esse banco não se recria dentro do Xylem. Por isso, aqui sim, o Xylem chama o
programa. Custa pouco e é só para quem já o tem instalado.

- **Tecla** (proposta): `Z`, livre hoje. Abre uma lista no estilo dos
  favoritos com o resultado de `zoxide query --list --score`: a pontuação e o
  caminho, filtrando enquanto se digita. Enter loga a pasta e vai até ela.
  Sem o zoxide instalado, `Z` diz que ele falta.
- **Alimentar o zoxide** (desligado por padrão): com `zoxide_add = true`, cada
  pasta em que o usuário entra pelo Xylem é registrada com `zoxide add`. Só
  pastas locais: nunca remotas, dentro de arquivos compactados ou de montagens
  do rclone, porque o banco é do shell desta máquina.
- **Segurança**: os argumentos vão para o `zoxide` como argumentos, nunca por um
  shell. Os caminhos que ele devolve passam pelas mesmas checagens do Log
  (existe, é diretório), são mostrados tratados como qualquer nome, e a saída é
  lida com limite de tamanho.
- **Onde**: só na máquina local. Numa sessão remota, `Z` diz que a lista é da
  máquina local.
- **Quando**: depois da etapa 2 da busca, se ainda fizer sentido.
