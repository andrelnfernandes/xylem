# Spec: consultar o disco, por fora e por dentro dos arquivos

Origem: conversa de 2026-11-14. Um recurso necessário, para o qual não há
implementações robustas no mercado. A sugestão foi consultar os arquivos da árvore
com SQL e agir sobre o resultado; o autor perguntou se valeria para o **conteúdo** dos arquivos,
como o Google Desktop fazia, com a preocupação da performance.

**Estado: aprovado pelo autor em 2026-11-14** ("a busca em arquivos usando SQL e
indexando com a tantivy está aprovada"). **Nada implementado.** A aprovação é do
recurso e do motor do índice; as decisões marcadas com **(?)**, reunidas no fim,
continuam em aberto até o autor confirmar cada uma, e a ordem das etapas é a deste
documento.

## Andamento

Lido por quem retoma o trabalho; atualizado ao fim de cada etapa.

- **Decidido pelo autor em 2026-11-14:**
  - o motor da etapa 3 é o **SQLite que já está no projeto**, lendo a árvore por uma
    tabela virtual (`files`) e o conteúdo por funções (`contains`, `words`): ele só
    interpreta e calcula, não guarda nada. As etapas 1 e 2 usam o mini-SQL
    (`sql_filter.rs`), mantido como subconjunto da sintaxe do SQLite;
  - a tecla é provisória: **`W`** (de WHERE) para os arquivos da lista e **`Ctrl+W`**
    para os marcados, até o autor escolher a definitiva;
  - o autor pediu o desenvolvimento sem a presença dele na madrugada de 2026-11-15,
    até onde for possível.
- **Etapa 1: feita** na branch `query` (sem commit, esperando autorização).
  - `src/model/query.rs`: `FileQuery` (a condição, pelo `sql_filter.rs`) e as colunas
    de `files`: `name`, `ext`, `path`, `dir`, `depth`, `size`, `disk`, `modified`,
    `age_days`, `mode`, `kind`, `tagged`. `git`, `type` e `dup` ficaram para depois.
  - `modified` é texto na hora local (`2026-11-15 09:30:00`), que compara como texto
    com uma data; a idade é a coluna `age_days`, em vez de `now - 30d`, que pediria
    estender o mini-SQL.
  - `Pane::query`, aplicada em `refresh_files` e `branch_matching_files` (então
    Ctrl+T na árvore também marca só o que a condição mostra). `W` e `Ctrl+W`
    abrem o prompt; o histórico começa com quatro exemplos; uma condição que não
    pega nada não muda a lista. O título da lista mostra a condição.
  - O mini-SQL não lê `10e6`: os números vão por extenso.
  - Manual: a seção "A SQL condition (W)" em `filespec.md` (só inglês).
  - Conferido no app real (build de debug): em Branch no `src` do Xylem,
    `ext = 'rs' AND size > 100000` deixou os 5 arquivos que o `find` acha, e o título
    mostrou a condição.
- **Etapa 2: feita** na branch `query` (sem commit), na madrugada de 2026-11-15, sem o
  autor.
  - O mini-SQL ganhou chamada de função (`Expr::Call`), aceita só por
    `SqlFilter::parse_with_calls`: o filtro de tabelas do viewer continua recusando.
  - `contains('texto')`: só como condição por si (sob `AND`, `OR`, `NOT`), com um
    texto entre aspas; mesmo casamento do Ctrl+S (`Pattern`, sem diferença de
    maiúsculas, `*` dentro da linha).
  - **A ordem é a performance,** por lógica de três valores (`decide`): o que o resto
    da condição decide não é lido, em `AND` e em `OR`. Não reordena nada.
  - `ContentJob` lê em passos no `tick`, como o Ctrl+S; com vários termos, lê o
    arquivo uma vez por termo. Esc para e a lista mostra o que achou.
  - **Regra 3 do autor:** antes de ler, uma confirmação diz que é sem índice, quantos
    arquivos e quantos bytes (`No index: read the contents of N files (X)?`); por SSH
    acrescenta que serão baixados. O rodapé mostra o progresso, e a mensagem final
    diz `N files read, with no index` e o que ficou de fora.
  - **Ficam de fora,** e a mensagem conta cada um: binários (byte zero nos primeiros
    8 KiB), links simbólicos, arquivos acima de 256 MiB e ilegíveis. Um arquivo de
    fora nunca é resultado, nem de `NOT contains(...)`.
  - **Decisão 4 (recomendação seguida):** recusado em montagem de nuvem; por SSH,
    confirmação (a mesma de sempre, com o aviso do download).
  - O resultado fica guardado como conjunto de caminhos (`FileQuery::hits`): um
    arquivo mudado ou novo só aparece rodando W de novo.
  - Conferido no app real (build de debug): `ext = 'rs' AND contains('LineSource')`
    em Branch no `src` perguntou "No index: read the contents of 70 files (2.3 MiB)?"
    e deixou os 3 arquivos que o `grep` acha.
- **Etapa 4: feita** na branch `query` (sem commit), na mesma madrugada, sem o autor.
  - `tantivy` 0.27 como dependência **opcional e ligada por padrão** (feature
    `index`, com `default-features = false` e `mmap`, `lz4-compression`: nenhum C
    novo; o único `zstd-sys` continua sendo o do Parquet). Compila com e sem ela.
  - **Tecla provisória: `Alt+N`** (iNdex) no ramo sob o cursor (na lista, a pasta do
    arquivo). `Alt+W` estava livre, mas era a tecla antiga do Usage, que um teste
    exige continuar livre. Num ramo com índice, Alt+N oferece "Index again", "Delete
    the index" ou "No".
  - **`words('a b')`:** todas as palavras, inteiras, sem diferença de maiúsculas nem de
    acentos. A separação de palavras é nossa (`words_of`, `WordSplitter`), a mesma ao
    indexar e ao ler um arquivo; o índice guarda as palavras já separadas e só as
    divide nos espaços (`whitespace`). Prefixo (`fatur*`): feito pelo autor em 2026-11-15 (`WordPat`, e `PhrasePrefixQuery`
    de uma palavra só no índice, que vira uma busca por faixa no dicionário e não
    pede reindexar; sem limite de expansões, para o índice nunca responder errado).
  - **Índice:** em `~/.cache/xylem/index/<FNV-1a do caminho do ramo>/`, modo 0700,
    com o caminho do ramo num arquivo `branch` e a versão em `version`. Por arquivo:
    caminho, tamanho, data e as palavras. Indexar de novo lê só o que mudou e tira o
    que sumiu; commit a cada 2.000 arquivos, e Esc guarda o que foi feito.
  - **Na consulta:** um arquivo com tamanho e data iguais aos do índice é respondido
    por ele; os outros são lidos. A pergunta diz os dois números (regra 3), e sem nada
    a ler não pergunta.
  - **Ficam fora do índice:** binários, arquivos acima de 64 MiB (W lê esses quando
    pedido), links, `.git`, `target`, `node_modules`, `.ssh`, `.gnupg`, `.env`,
    `*.key`, `*.pem`, `id_rsa`, `id_ed25519`. A lista **não é configurável ainda**.
    Remoto, arquivo compactado e nuvem: sem índice.
  - **Índice ilegível** (estragado ou de outra versão): a consulta avisa e lê os
    arquivos; Alt+N o refaz do zero. Os erros e pânicos da tantivy passam por
    `panic_guard`.
  - Um binário entra no índice marcado como tal (sem palavras): a consulta o deixa de
    fora sem lê-lo. Versão do índice: 2 (uma de outra versão é refeita).
  - O mini-SQL passou a aceitar `NOT LIKE` e `NOT ILIKE`, como o SQLite.
  - **Conferido no app real** (release e debug, HOME isolado): a árvore do Xylem (429
    arquivos logados fora de `target`) indexada em 1,5 s no release, índice de 452 KB;
    indexar de novo leu só o arquivo que mudou; `dir NOT LIKE '%/target%' AND dir NOT
    LIKE '%/.git%' AND words('memchr')` respondeu sem pergunta ("420 answered by the
    index") os mesmos 4 arquivos que o `grep -w`. O binário release foi de 17,4 MB para
    20,5 MB, e a compilação do release de cerca de 2 min para 3 min 45 s.
  - **Medido em um ramo grande** (2026-11-15, release, HOME isolado, 200.000 arquivos sintéticos em 2.000 pastas em `~/.cache/xt-dev/bigtree/`):
    - Leitura da árvore (`*`, expand all): 4,03 s, RSS 38,0 MB (pico 38,0 MB).
    - Indexação (`Alt+N`): 6,07 s para indexar os 200.000 arquivos (29,3 MiB de texto).
    - Tamanho do índice no disco: 7,5 MB.
    - Consulta `W` com `words('tantivy')` em Branch sobre os 200.000 arquivos (sem pergunta, 200.000 respondidos pelo índice): 0,58 s, 12.244 arquivos casados, delta RSS +78,9 MB (pico 158,1 MB). Como ficou bem abaixo dos limites (~1 s e ~200 MB), não exigiu otimização imediata.
    - Consulta `W` com `contains('synthetic')` sem índice: 1,10 s até a exibição da pergunta de confirmação de leitura ("No index: read the contents of 200,000 files (29.3 MiB)?").
  - Fuzz de índice corrompido: ainda não medido (só o teste de arquivos trocados por lixo).
  - Só UTF-8 é indexado direito (outra codificação vira `�` nas palavras); as células
    das tabelas (CSV, XLSX...) entram como texto cru, não pelos leitores do viewer.
  - **`index_ignore` configurável (2026-11-15):** opção `index_ignore` no `config.toml`
    e na tela de configuração (F12, página 4, E), mantendo por padrão os padrões de
    pastas e arquivos de segredos/build; aviso na tela se padrões de segredos forem
    removidos. Testes em `src/index.rs`, `src/settings.rs` e `src/app/tests.rs`.
  - **Notas de versão (2026-11-15):** adicionada seção sobre W, `contains()`, `words()`
    com prefixo e o índice em `.github/release-notes/v0.8.0.md`.
  - **Colunas `git`, `type` e `dup` (2026-11-15):**
    - `COLUMNS` e `QueryContext` ganharam suporte a `git`, `type` e `dup`.
    - `git`: status do Git em duas letras (`' M'`, `'M '`, `'??'`, `'  '` se limpo); `NULL` fora de repositório ou com Git desligado.
    - `type`: descrição do arquivo via Alt+F (`describe_file`), avaliado de forma lazy (sob `AND`, curto-circuita e não lê do disco se a condição anterior for falsa); aproveita o cache de tipos de 10.000 entradas do `Pane`.
    - `dup`: número do grupo de duplicados (1-based: 1, 2, ...), persistido em `duplicate_groups` do `Pane` mesmo após fechar a tela Alt+D com Esc; `NULL` se o arquivo não for duplicado ou Alt+D não foi rodado.
    - Manuais em inglês (`docs/manual/filespec.md`) e português (`docs/manual/pt/filespec.md`) atualizados.
    - Testes unitários em `src/model/query.rs` e no nível de app em `src/app/tests.rs` (`w_filters_by_git_type_and_dup_columns`).
- **Etapa 5: caixa de busca feita** (2026-11-15, branch `query-next`):
  - Experiência do Google Desktop sem SQL via tecla `P` (arquivos da lista) e `Ctrl+P` (arquivos marcados), implementada em `src/search_box.rs`.
  - Converte as palavras digitadas em `words('...')` com casamento de prefixo na última palavra (`fatura mar` busca `fatura mar*`).
  - Resposta imediata dos arquivos indexados pelo índice (`Alt+N`). Arquivos não indexados informam a quantidade e o tamanho em bytes e só são lidos com confirmação explícita (`Alt+R` ou `Enter` quando sem resultados indexados), lidos de forma incremental no `tick`.
  - Cada linha de resultado exibe marcação de tag (`*`), nome, pasta, data e trecho saneado da primeira linha onde as palavras aparecem (~80 caracteres, livre de controles e de overrides bidirecionais / Trojan Source).
  - Teclas na caixa: ↑↓, PgUp/PgDn, Ctrl+Home/Ctrl+End navegam na lista de resultados; Enter vai direto ao arquivo na janela de arquivos e fecha a caixa; `Ctrl+V` abre no viewer posicionado na linha do trecho; `Ctrl+T` alterna a marcação (tag) do arquivo diretamente na caixa; `Alt+R` confirma e inicia a leitura de arquivos não indexados; `Esc` cancela leitura em andamento ou fecha a caixa.
  - Manual atualizado em `docs/manual/filespec.md`, `tree.md` e `files.md`.
  - Testes unitários em `src/search_box.rs` e testes no nível de app em `src/app/tests.rs` cobrindo fluxo com índice, busca em marcados com `Ctrl+P`, leitura de arquivos sem índice com confirmação, `Ctrl+V`, `Ctrl+T`, e navegação/Esc.
- **Revisão da etapa 5 e das colunas (2026-11-15, Claude, branch `query-next`):**
  - **Corrigido:**
    - **A caixa não buscava no app real:** a busca roda no `tick`, que o laço
      principal só chama com `busy()`, e a caixa não estava nele (os testes buscavam
      na hora de digitar, por um `#[cfg(test)]`). Agora `busy()` inclui a caixa enquanto
      ela espera a pausa na digitação ou lê arquivos (`SearchBox::wants_tick`), o
      `tick` dorme até a pausa em vez de girar, e o atalho dos testes saiu.
    - **A caixa parava o log:** o `tick` retornava na caixa antes do trabalho de logar;
      agora ela roda primeiro e o resto segue.
    - **Um arquivo hostil derrubava o Xylem:** `sanitize_snippet` cortava a linha pela
      posição achada numa cópia em minúsculas (`İ` vira `i̇`, um byte a mais), no meio
      de um caractere. Agora a palavra é achada caractere a caractere, como `words()`
      as separa (`first_match`), e o trecho é calculado sob `panic_guard`.
    - **O trecho lia sem limite:** o resto de uma linha longa era descartado sem contar;
      um arquivo de 64 MiB sem quebras era lido inteiro. Agora todo byte lido conta no
      limite de 1 MiB.
    - **Remoto e arquivo compactado:** o trecho abria o caminho com `File::open`; agora
      lê pelo `content_opener` da janela, e numa máquina remota não mostra trecho (a
      tela esperaria a rede). A coluna `type` no `ContentJob` lia o caminho local; agora
      usa `Pane::type_of`, o mesmo do Alt+F (pela rede no remoto, só pelo nome num
      arquivo compactado).
    - Testes que falhavam antes: `a_snippet_is_cut_between_characters`,
      `a_snippet_reads_at_most_its_limit`,
      `the_search_box_keeps_the_loop_awake_until_it_has_searched`.
    - **Conferido no app real** (debug, HOME isolado), na árvore do Xylem: sem índice, a
      caixa diz "9 unindexed files (170.1 KiB) · Alt+R to read", o Alt+R pergunta, e com
      "y" acha `Cargo.lock` e `Cargo.toml`; depois do Alt+N, em Branch, acha 5 arquivos
      pelo índice, com o trecho e a linha (`L48: memchr = "2"`), e avisa "26,375
      unindexed files (17.6 GiB)" (o `target/`).
  - **Ainda em aberto:**
    - a coluna `type` em `refresh_files` lê o começo dos arquivos na thread da tela,
      também numa máquina remota (pela rede) e numa montagem de nuvem (um download por
      arquivo); o cache de tipos é zerado ao passar de 10.000 entradas;
    - cada pausa na digitação refaz candidatos, `Lookup` e trechos na thread da tela
      (~0,6 s por consulta em 200.000 arquivos, pela medição do passo 1);
    - `o'brien` vira `words('o''brien*')`, que o mini-SQL recusa (ele não entende `''`
      como o SQLite), e a caixa fica vazia sem dizer por quê;
    - a árvore de 200.000 arquivos das medições não existe mais em
      `~/.cache/xt-dev/bigtree/`: os números não foram reconferidos.
- **Etapa 3:** depois dela, pelo SQLite (decidido).

## A ideia

A árvore lida é uma tabela, `files`, com uma linha por arquivo. Uma consulta escolhe
linhas; o resultado vira **marcas**; e com marcas o Xylem já faz tudo: copiar, mover,
apagar, Batch, digitar no terminal (Alt+O).

```sql
ext = 'jpg' AND size > 10e6 AND modified < '2024-01-01'
ext IN ('md', 'txt') AND contains('fatura')
```

Em duas frentes, que se somam:

- **Por fora:** nome, tamanho, data, tipo. Está tudo na memória; a resposta é imediata.
- **Por dentro:** o texto dos arquivos. Sem índice no começo; com índice, por ramo,
  para a resposta na hora que o Google Desktop dava.

## Regras decididas pelo autor (2026-11-14)

1. **A busca é sempre sobre os arquivos logados que estão na lista.** Nunca aparece
   um arquivo que o Xylem não está mostrando: todo resultado pode ser marcado, visto
   e copiado ali mesmo. O índice só acelera; um arquivo que está no índice mas não na
   lista não é resultado. Para procurar num ramo, ele precisa estar logado (`*`,
   Branch), como para qualquer outra operação.
2. **Sem Ctrl, os arquivos da lista; com Ctrl, os marcados.** A mesma tecla com e sem
   Ctrl, como no resto do Xylem (e como o Ctrl+S de hoje, que procura nos marcados).
   O rodapé e o título do prompt dizem em qual dos dois a busca está sendo feita.
3. **Com índice e sem índice é dito com todas as letras.** Antes de rodar, o prompt
   mostra o que vai acontecer: quantos arquivos têm índice em dia, quantos serão
   lidos direto, e o tamanho do que será lido. Depois, o resultado diz o mesmo. O
   manual explica a diferença numa tabela:

   | | Sem índice | Com índice |
   |---|---|---|
   | Resposta | lê cada arquivo na hora | imediata |
   | Custo | tempo a cada busca | tempo e espaço ao indexar |
   | Procura | pedaço de texto (`contains`) ou palavras (`words`) | palavras (`words`) |
   | Arquivo mudado | sempre o conteúdo atual | lido direto se mais novo que o índice |
   | Onde | qualquer lugar logado | só nos ramos que o usuário mandou indexar |

Consequência da regra 1 para a etapa 5: a caixa de busca procura no que está logado,
não em todos os ramos indexados. Um ramo indexado e não logado precisa ser logado
antes; como o índice sabe quais arquivos tem, logar esse ramo pode ser oferecido ali.

## O que já existe e se aproveita

- **`src/viewer/sql_filter.rs`**: o mini-SQL do viewer. Só o predicado do `WHERE`
  (`=`, `<`, `LIKE`, `ILIKE`, `IN`, `IS NULL`, `AND`, `OR`, `NOT`), em Rust puro, sem
  dependência, com recursão limitada. Avalia sobre qualquer coisa que implemente
  `RowContext`: um arquivo da árvore pode ser uma linha. **Não tem** `SELECT`, `GROUP
  BY`, `ORDER BY`, `JOIN` nem chamada de função.
- **`src/search.rs`**: a busca do Ctrl+S. `SearchJob` anda por uma lista de arquivos com
  orçamento de tempo por passo (a tela fica livre, Esc para), e `matches_reader`
  responde sim ou não por arquivo, linha a linha.
- **O modo tabela do viewer** e o **Batch sobre registros**: para mostrar e exportar um
  resultado com colunas.
- **SQLite** (`rusqlite`, C): faz `JOIN` e `GROUP BY`, mas só roda com `V`, e o item 5
  de Segurança do backlog quer trocá-lo por Rust.
- **O cockpit** (`cockpit.md`): a busca no projeto com lista de `arquivo:linha` é a
  mesma peça vista de outro lado. Ver "Relação com o cockpit".

## As colunas de `files`

| Coluna | Vem de | Custo |
|---|---|---|
| `name`, `ext`, `path`, `dir`, `depth` | a árvore | nenhum |
| `size`, `disk`, `modified`, `mode`, `kind` | a árvore (`FileEntry`) | nenhum |
| `tagged` | a árvore | nenhum |
| `git` | o status do Git, se ligado | o que já custa hoje |
| `type` | o tipo do Alt+F (`filetype.rs`) | lê o começo do arquivo |
| `dup` | o grupo de Duplicados, se já calculado | nenhum; vazio se não foi |
| `contains('…')` | o conteúdo | lê o arquivo, ou consulta o índice |

Nomes de coluna em inglês, como os comandos. `modified` aceita data (`'2024-01-01'`) e
idade (`modified > now - 30d` **(?)** pede extensão do mini-SQL).

## Etapas

Cada uma é um PR que se sustenta sozinho; dá para parar depois de qualquer uma.

### 1. Filtro de arquivos: o `WHERE` sobre a árvore

A menor versão útil, quase sem código novo: um **filespec com predicado**. Em Branch
e Showall (ou na pasta), um prompt recebe o predicado e a lista de arquivos passa a
mostrar só o que casa. `Ctrl+T` marca tudo, como hoje. O rodapé diz quantos casaram.

- Usa `sql_filter.rs` como está, com um `RowContext` sobre `FileEntry`.
- Histórico do prompt com exemplos prontos já na primeira vez (quem não sabe SQL copia
  e muda): "maiores que 100 MB", "mexidos esta semana", "imagens sem marca".
- Erro de sintaxe mostra a posição, como no filtro do viewer.
- Já entrega "marcar por atributo", que o Alt+T do ZTreeWin fazia.

### 2. `contains('texto')`, sem índice

- O mini-SQL ganha chamada de função (só as da lista: `contains`, depois `words`).
- **A ordem é a performance:** primeiro as condições baratas, sobre todos; o conteúdo só
  dos que sobraram. `ext = 'md' AND contains('x')` abre só os `.md`. Vale para `AND`; num
  `OR` com `contains`, lê tudo, e o rodapé avisa antes de começar.
- Roda como o Ctrl+S: em segundo plano, progresso, Esc, e o resultado parcial fica.
- Limites: tamanho por arquivo, binários de fora por padrão, links simbólicos não
  seguidos. O rodapé diz o que ficou de fora, nunca mostra um resultado cortado como
  completo.
- Em nuvem (rclone) e por SSH, ler o conteúdo é baixar: pede confirmação com o total
  em bytes, ou é recusado na nuvem **(?)**.

### 3. Resultado como tabela: `SELECT`, `GROUP BY`, `JOIN`

O que chama a atenção: "quais notas desta planilha não têm o PDF na pasta?", "quanto
ocupa cada extensão?" (o "Usage por tipo" do backlog). O resultado abre no modo tabela
do viewer; uma tecla marca na árvore os arquivos das linhas; o Batch exporta.

O motor é a decisão **(?)**:

| Opção | A favor | Contra |
|---|---|---|
| SQLite em memória | pronto; `JOIN` com outro `.sqlite` de graça | é C; contraria o item 5 |
| Estender o mini-SQL (`GROUP BY`, `ORDER BY`, `JOIN` de duas tabelas) | Rust, sem dependência, limites nossos | código nosso a escrever e testar |

Atenuante do SQLite aqui: ele não leria arquivo hostil, só dados que o Xylem montou
(nomes entram como valores ligados, nunca como texto da consulta). O risco é menor que
o de abrir um `.sqlite` baixado. Mesmo assim, recomendo decidir só depois das etapas 1
e 2 em uso.

### 4. Índice por ramo

Para o conteúdo responder na hora. **Pedido pelo usuário, ramo por ramo**, como o Log:
"indexe este ramo". Nada é indexado sem pedir, e não há serviço em segundo plano
quando o Xylem está fechado.

- **Motor: `tantivy`** (ver "A tantivy, conferida").
- **Onde:** `~/.cache/xylem/index/<hash da raiz do ramo>/`, modo 0700.
- **O quê:** texto, e depois as células das tabelas que o viewer já lê (CSV, XLSX,
  Parquet, SQLite), pelos mesmos leitores e com os mesmos limites. PDF e Office com
  texto corrido ficam fora: pedem leitores novos de arquivo hostil.
- **Guardado por arquivo:** caminho, tamanho e data, para saber se mudou.
- **Atualização:** ao indexar de novo, só o que mudou (tamanho ou data), o que é novo e
  o que sumiu. Na consulta, arquivo mais novo que o índice é lido direto, como na etapa
  2: o índice velho deixa a consulta mais lenta, nunca errada.
- **Fora por padrão:** `.git`, `target`, `node_modules`, e o que costuma ter segredo
  (`.ssh`, `.gnupg`, `.env`, `*.key`, `*.pem`), com lista configurável. O índice é uma
  cópia pesquisável do conteúdo: isso vai dito no manual, com o comando de apagar.
- **Índice corrompido ou de outra versão:** erro ou pânico ao abrir descarta e avisa;
  nunca derruba o Xylem (`panic_guard.rs`).
- **Nuvem e SSH:** sem índice no começo.

### 5. A caixa de busca: a experiência do Google Desktop

Uma tecla, uma caixa, digitar e ver os resultados chegando, sem SQL. Sobre os arquivos
logados da lista (com Ctrl, os marcados), por nome e por conteúdo. Cada linha: arquivo, trecho
com o termo, data. Enter vai ao arquivo (ou abre o viewer na linha); `T` marca.

É a porta de entrada para quem não escreve consultas, e o que aparece no GIF. A lista
é a mesma do histórico dos prompts (`HistoryList`) com outra fonte.

## Subcadeia ou palavra **(?)**

Um índice de texto guarda **palavras**; a busca de hoje acha **pedaços** (`fatur` acha
`faturamento`). São respostas diferentes, e esconder isso gera resultado que muda
conforme haja índice ou não. Recomendação: duas funções, cada uma com um sentido só.

- `contains('fatur')`: pedaço de texto, exato, sempre lendo o arquivo (etapa 2).
- `words('fatura 2024')`: palavras inteiras (e prefixo com `fatur*`), pelo índice onde
  houver e lendo onde não houver, com a mesma separação de palavras nos dois caminhos.

A caixa de busca (etapa 5) usa `words`.

## Performance: o que se sabe e o que medir

Medições realizadas em 2026-11-15 (build release, máquina Linux x86_64, disco local SSD NVMe, HOME isolado):

1. **Árvore do Xylem** (429 arquivos logados fora de `target`):
   - Primeira indexação: 1,5 s, índice de 452 KB.
   - Reindexação sem mudanças: imediata (lê apenas o que mudou).
   - Consulta `words('memchr')`: imediata ("420 answered by the index").

2. **Ramo grande sintético** (`~/.cache/xt-dev/bigtree/`, 200.000 arquivos de texto em 2.000 pastas, 29,3 MiB de texto, 790 MB em disco):
   - Leitura completa da árvore (`*`, expand all): 4,03 s, RSS 38,0 MB (pico 38,0 MB).
   - Primeira indexação (`Alt+N`): 6,07 s para ler e indexar todos os 200.000 arquivos.
   - Tamanho do índice no disco: 7,5 MB.
   - Consulta `W` com `words('tantivy')` em Branch sobre os 200.000 arquivos: **0,58 s** de resposta, 12.244 arquivos casados, delta RSS de **+78,9 MB** (pico total 158,1 MB), sem pergunta (200.000 respondidos pelo índice).
   - Consulta `W` com `contains('synthetic')` sem índice: 1,10 s até abrir a pergunta de confirmação de leitura ("No index: read the contents of 200,000 files (29.3 MiB)?").

O que se conclui: mesmo em um ramo com 200.000 arquivos, o tempo de consulta pelo índice fica em ~0,58 s e o consumo de memória adicional fica em ~79 MB, bem abaixo dos limites de tolerância (~1 s e ~200 MB), confirmando que a implementação atende com folga e não exige cache ou pré-filtragem complexa no momento.

## A tantivy, conferida

Conferido em 2026-11-14, num projeto de rascunho em `~/.cache/xt-dev/tantivy-check`
(só `cargo tree`, sem compilar):

- Versão **0.27.0**, Rust mínimo 1.88.
- Com `default-features = false` e `features = ["mmap", "lz4-compression"]`: **128
  crates** na árvore (parte já está no Xylem, que tem 283), **nenhuma dependência em
  C**. O único `-sys` é `linux-raw-sys`, que são definições de chamadas do sistema em
  Rust.
- As features padrão incluem `columnar-zstd-compression`, que traz o zstd em C:
  **desligar**, como no Parquet.
- Não conferido: tempo de compilação, tamanho do binário, comportamento com índice
  corrompido (fuzz), e se a separação de palavras dela serve para português (acentos,
  `ação` = `AÇÃO`, que o `search.rs` já trata).

Fica atrás de uma feature do Cargo (`index`), para o binário sem ela continuar
pequeno e para os pacotes poderem escolher.

Por que não as alternativas: o FTS5 do SQLite funcionaria, mas é C e rodaria sem
ninguém pedir; o DuckDB é C++, bem maior, e busca textual não é o forte dele.

## Segurança

- **A consulta só lê.** Quem age são os comandos de sempre, com as confirmações de
  sempre. Não há `UPDATE` nem `DELETE` sobre `files`.
- **Nomes e conteúdo são hostis:** entram como valores, nunca como texto da consulta;
  na tela, sem caracteres de controle nem bidi, com tamanho limitado, como o resto.
- **Limites:** tamanho da consulta e profundidade (o mini-SQL já tem), número de linhas
  do resultado, bytes lidos por arquivo, tempo por passo. Passou do limite, para e diz.
- **Indexar é abrir arquivos sem o `V`.** Por isso só a pedido, só os leitores que já
  passaram pela revisão de segurança, cada arquivo com limite de tamanho e de tempo, e
  um arquivo que faz o leitor falhar é pulado e contado, sem parar a indexação.
- **O índice** fica no cache do usuário, modo 0700; apagar o índice é um comando.

## Relação com o cockpit

A "busca no projeto" do `cockpit.md` devolve **linhas** (`arquivo:linha: texto`); a
consulta devolve **arquivos**. É o mesmo motor de leitura com duas saídas. Sugestão:
fazer a busca com linhas uma vez só (em `search.rs`), e as duas telas usam: a lista de
locais do cockpit e o trecho mostrado na caixa de busca (etapa 5).

## Decisões em aberto

1. **A tecla.** Decidido: a mesma letra sem Ctrl (a lista) e com Ctrl (os marcados).
   Falta escolher a letra: `Q` é Quit e `S` é Showall (com Ctrl+S já sendo a busca nos
   marcados). Para a etapa 1, estender o Filespec (`F`), que aceitaria predicado além
   de máscara **(?)**; para a busca por conteúdo, uma letra a escolher.
2. **O motor da etapa 3:** SQLite em memória ou mini-SQL estendido. Decidir depois das
   etapas 1 e 2.
3. **`contains` e `words` separados?** Recomendo sim.
4. **Conteúdo na nuvem:** confirmar com o total em bytes, ou recusar. Recomendo recusar
   na nuvem e confirmar por SSH.
5. **O motor do índice é a tantivy:** decidido pelo autor em 2026-11-14. Em aberto só
   se entra como dependência opcional; recomendo sim, feature `index`.
6. **Onde a etapa 5 entra no roteiro:** é a mais visível, mas depende da 4.

## Fora deste estudo

- Indexador do disco inteiro em segundo plano, com o Xylem fechado.
- PDF, DOCX e outros formatos que pedem leitor novo. Os metadados de dentro dos
  arquivos (EXIF das fotos, ID3 das músicas, autor dos documentos) têm um estudo à
  parte: [query-metadata.md](query-metadata.md).
- E-mail, histórico do navegador e o resto do que o Google Desktop indexava.
- Busca por significado (embeddings) e qualquer coisa que mande conteúdo para fora da
  máquina.
- Consultas que escrevem.
