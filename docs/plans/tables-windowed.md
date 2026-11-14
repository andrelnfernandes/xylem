# Spec: tabelas sem o limite de 50.000 linhas

Origem: conversa de 2026-11-13. O autor abriu arquivos SQLite e CSV com mais de 50.000
linhas e não conseguiu ver tudo. Ele notou que isso se encaixa no viewer em janelas, como
o `less` (subprojeto 3 de [viewers.md](viewers.md)), e abre caminho para os logs
(subprojeto 4). Pedido do autor: entrar na **0.8.0**, prevista para uns 5 dias depois.

**Estado: aprovado pelo autor em 2026-11-13, com as recomendações abaixo ("Decisões"),
para a 0.8.0.** Em implementação na branch `tables-windowed`:

- **Feitas:** fatia 4 (2951f8e), fatia 1 (5277b52), fatia 2 (2c47cff) e fatia 3. A
  janela está em `src/viewer/table_window.rs`: `Window`, as threads de leitura, contagem
  e filtro, e o trait `RowSource` em `records.rs`, com `SqliteRows`, `TextRows` (CSV,
  TSV, JSONL) e `DbfRows` como fontes. A grade (`table.rs`) usa `row`, `file_row`,
  `known_rows` e `total_text`, e o viewer chama `poll_window`, `pause_window` e
  `resume_window`.
- **Fatia 2, como ficou:**
  - `RowIndex` (em `records.rs`) guarda o byte de uma linha a cada 256, compartilhado por
    todas as threads da janela; quem o preenche é a contagem, que retoma da última marca
    se for parada. Cada marca ocupa 8 bytes (só o byte; o número da linha é o da marca),
    não os 24 do desenho.
  - A contagem diz o progresso (`RowSource::count` recebe um `Progress`), então o total
    cresce na tela enquanto conta; no JSONL ela também acha colunas novas depois das
    primeiras 2.000 linhas, e a grade as ganha.
  - CSV: um registro com mais de 16 MiB para a leitura (`MAX_RECORD_BYTES`, pelo
    `Fenced` sob o leitor do `csv`); bytes que não são UTF-8 viram `�` em vez de parar a
    leitura (antes parava).
  - JSONL: um registro é um objeto com ao menos uma chave, seja qual for o lugar de onde
    se lê (`{}` deixou de ser linha de nulos); uma linha com mais de 512 KiB é pulada
    inteira (antes era lida em pedaços), e uma linha que não é UTF-8 é pulada (antes
    parava a leitura).
  - O arquivo que mudou: tamanho e data guardados antes de ler as primeiras linhas e
    conferidos antes e depois de cada leitura. Ao mudar, a janela para, o cabeçalho diz
    `(the file changed: R reloads)`, o rodapé mostra `R reload`, e **R** relê.
  - Leitura adiada do texto: com `V`, um arquivo candidato a tabela lê só 64 KiB
    (`TABLE_PEEK`); o resto é lido (`fill_text`) ao ir para T ou H, se a tabela não abrir,
    ou ao voltar ao texto num reload. Na tabela, o cabeçalho não diz mais `(showing first
    N MiB)` nem os controles bidi do texto.
- **Fatia 3, como ficou:** `DbfRows` em `records.rs`, com o mesmo `RowIndex`, mas as
  marcas são números de registro, não bytes: um registro apagado não é linha, então a
  linha N não é o registro N. A contagem só lê a marca de apagado de cada registro, numa
  passada; a leitura vai à marca pelo `seek` do `dbase` e passa por no máximo 255 linhas.
  - O `dbase` 0.8 não para depois de um `seek`: o iterador conta desde 0 até
    `num_records` e leria além do fim. Por isso o arquivo fica sob um `DbfFile`, que
    termina onde o cabeçalho diz que os registros terminam (e para com o tempo e o Esc).
  - O leitor é montado pelo `ReaderBuilder` sobre esse arquivo, também para as primeiras
    linhas (`read_dbf`), e o arquivo de memo é escolhido como no `from_path` do crate
    (`.fpt` nas versões FoxPro, `.dbt` nas outras), já que ele não expõe essa escolha.
  - Um arquivo mais curto que o cabeçalho conta só os registros inteiros.
- **Fechamento:** as notas da 0.8.0 (`.github/release-notes/v0.8.0.md`), o BACKLOG e
  o `viewers.md` atualizados. Os testes que faltavam (o teto de 1.000.000 de resultados
  e o Esc na contagem e no filtro de CSV, JSONL e DBF) estão escritos. Conferido no app
  real (build de debug), com um CSV de 108 MB (4 milhões de linhas) e um banco de 333
  MB: a tabela abre em 0,14 s e 0,08 s (o banco levava 16 s), Ctrl+End, `g`, o filtro
  e o Esc acertam; T lê o resto do texto em 5,8 s. A nota `Filtering the whole
  table...` passou a dar lugar a `Filter matched N rows` quando a varredura acaba.
  Fica para depois: o manual nas outras 5 línguas, com a tarefa de documentação
  (BACKLOG).
- **Manual:** durante o desenvolvimento, só o inglês (`docs/manual/viewer.md`, seção
  "Big tables"); as traduções ficam pendentes no BACKLOG.

## O que existe hoje

- **Um só lugar guarda as linhas:** `GridSink` em `src/viewer/tabular_loader.rs` recebe
  as linhas que cada leitor de `src/viewer/records.rs` entrega e para em
  `MAX_TABLE_ROWS = 50_000`. O cabeçalho diz `(showing first 50,000 rows)`.
- **O que depende de ter as linhas na memória** (`TableGrid`, `src/viewer/table.rs`):
  - o filtro SQL (`\` e `&`, `apply_filter`) percorre `rows`, então só vê as 50.000;
  - Ctrl+End, `g` (ir para a linha) e o número da linha (`cursor_file_row`) só vão até a
    última guardada;
  - a largura das colunas vem das primeiras 200 linhas (`recalculate_widths`).
- **O que já não depende:**
  - o **Batch** (Ctrl+B) relê o arquivo pelo leitor, sem teto de linhas, e escreve só os
    registros marcados (`m`), pelo número da linha (teste
    `marked_records_are_the_only_ones_written_past_the_limit_of_the_screen`);
  - as **marcas** são números de linha (`BTreeSet<usize>`), não referências à memória.
- **Leitores que sabem pular:**
  - o `csv` 1.4 tem `Reader::position()` e `Reader::seek(Position)`;
  - o `dbase` 0.8 tem `Reader::seek(índice)`, porque os registros têm tamanho fixo;
  - o SQLite tem `LIMIT` e `OFFSET`.
- **A tabela é lida em outra thread** (`read_table_in_background`), uma de cada vez
  (`reader_is_free`), com tempo, Esc e o aviso `TABLE_BUSY`.

## O que muda

A grade deixa de guardar a tabela inteira e passa a guardar uma **janela**: as linhas
perto do cursor. As outras são pedidas ao arquivo conforme o cursor anda, numa thread, sem
segurar a tela.

### Para quem usa

- **A tabela abre na hora,** com as primeiras linhas, como hoje.
- **O total aparece enquanto é contado:** o cabeçalho mostra `1,234,567+ rows
  (counting…)` e, ao terminar, `8,412,903 rows`. Esc para a contagem; o que já foi contado
  continua valendo.
- **Ctrl+End e `g`** chegam à última linha. Enquanto a contagem não termina, Ctrl+End vai
  à última já contada e `g` além dela diz que a contagem ainda não chegou lá.
- **Andar pela tabela** nunca espera o disco: uma linha ainda não lida aparece como
  `…` por um instante e se preenche quando chega.
- **O filtro varre a tabela inteira** em segundo plano: o rodapé mostra
  `1,203 matches so far (row 2,300,000 of 8,412,903)`, os resultados aparecem conforme
  são achados, e Esc para a varredura.
- **O Batch, as marcas e `i` (a estrutura)** continuam como estão.

### Por formato

| Formato | Como chega à linha N | Na 0.8.0 |
|---|---|---|
| **SQLite** (tabelas e views) | `SELECT … LIMIT ? OFFSET ?`; o total por `count(*)` | sim |
| **SQLite** (query digitada) | `SELECT * FROM (query) LIMIT ? OFFSET ?` | sim |
| **CSV e TSV** | índice em segundo plano (abaixo) | sim |
| **JSON Lines** | o mesmo índice, por linha | sim |
| **DBF** | `seek` do `dbase` (registros de tamanho fixo) | sim |
| **Parquet** | grupos de linhas do próprio formato | não: fica para a 0.9 |
| **Planilhas** (XLSX, XLSB, XLS, ODS) | não dá: XML comprimido, lido do começo | não: o limite continua, com o aviso abaixo |
| **Comprimidos** (`.csv.gz`) | não dá: não se pula dentro deles | não: como hoje |

### O índice de CSV, TSV e JSON Lines

- Uma passada em segundo plano lê o arquivo do começo ao fim e **anota a posição de uma
  linha a cada 256**: o byte e o número da linha, a `Position` do `csv`.
- Para chegar à linha N, o Xylem pula para a anotada mais próxima antes dela e lê no
  máximo 255 linhas. Um registro CSV com quebra de linha dentro de aspas conta como um, e
  o índice o respeita, porque quem anota é o próprio leitor de CSV.
- **Memória:** 24 bytes a cada 256 linhas; 100 milhões de linhas ocupam uns 9 MiB.
- **É a peça que o viewer de texto e os logs vão reaproveitar:** a mesma passada, contando
  linhas de texto, e, no follow, continuando a partir do fim quando o arquivo cresce.

### A janela na memória

- No máximo **2.000 linhas** e **32 MiB** de texto de células, o que vier primeiro; as
  mais longe do cursor saem. O teto em bytes importa porque uma linha pode ter 256 colunas
  de 2.048 caracteres.
- A largura das colunas continua vindo das primeiras 200 linhas.

### O filtro numa tabela enorme

- A varredura passa por todas as linhas, pelo mesmo leitor, e guarda **os números das
  linhas que atendem**, até **1.000.000** de resultados (8 MiB). Passou disso, ela para e o
  rodapé diz `first 1,000,000 matches`.
- A lista filtrada também é uma janela: as linhas dela são lidas pelo número, como as
  outras.
- **SQLite: em aberto** (ver "Para decidir").

### Planilhas: o limite continua, com um aviso claro

Pedido do autor: como uma planilha não pode ser lida em partes, ela continua com o limite
de 50.000 linhas. Ao passar disso, o cabeçalho e a nota dizem:

`(the sheet has more than 50,000 rows, the most Xylem shows: open it in Excel or another
spreadsheet program to see them all (O))`

**O** abre a planilha no programa do sistema. O manual (`viewer.md`, 6 línguas) diz o
mesmo.

### O arquivo mudou

- Ao abrir, o Xylem guarda o tamanho e a data do arquivo. Se mudarem, o índice e a
  janela deixam de valer: o rodapé diz `the file changed: R reloads`, e nada é lido de
  posições velhas.
- Acompanhar um arquivo que cresce é o follow dos logs, com spec próprio.

### Onde não muda

- **Autoview (F7)** continua como hoje: as primeiras linhas e o aviso do corte, sem índice
  nem contagem. Ele roda sobre tudo o que o cursor passa, e não pode deixar trabalho em
  segundo plano a cada arquivo.

## Modelo de ameaça

Todo arquivo é hostil (princípios de [viewers.md](viewers.md)). O que muda é que o arquivo
passa a ser lido por mais tempo, em partes, depois de aberto.

1. **A tela nunca espera o arquivo.** Índice, contagem, janela e filtro rodam fora da
   thread da tela, com Esc. Uma leitura de cada vez, como hoje (`reader_is_free`).
2. **Memória com teto em tudo:** janela (2.000 linhas e 32 MiB), índice (24 bytes a cada
   256 linhas, até 4 milhões de entradas, ou cerca de 1 bilhão de linhas; passou disso, a
   contagem para e diz), e resultados do filtro (1.000.000). Uma célula continua cortada
   em 2.048 caracteres; um registro enorme (um CSV sem quebra de linha, por exemplo)
   ganha um teto próprio de bytes, a conferir no leitor de cada formato.
3. **Bomba de linhas:** um arquivo de bilhões de linhas vazias só faz a contagem demorar,
   com a tela livre e Esc, e o teto do índice para tudo.
4. **Arquivo trocado no meio:** entre a passada do índice e a leitura de uma janela, o
   arquivo pode mudar ou ser substituído. Tamanho e data são conferidos antes de cada
   leitura. Uma posição além do fim, ou que não começa um registro, vira `the file
   changed`, nunca uma linha inventada ou um `panic`.
5. **SQLite** continua só com `V`, em modo defensivo, `query_only` e com o tempo limitado
   pelo `progress_handler`, agora também em cada janela e na contagem. A query digitada é
   envolvida em `SELECT * FROM (…) LIMIT ? OFFSET ?`, e os números vão como parâmetros,
   nunca no texto.
6. **Nada é escrito:** o índice fica só na memória e some ao fechar o viewer.
7. **A tela também é vetor:** células e nomes continuam passando por `sanitize_cell` e
   pelo filtro final de `ui::draw`.

## Fatias

Na branch até as fatias 1 e 2 funcionarem juntas. Se não ficarem prontas para a 0.8.0, a
0.8.0 sai sem elas, como diz o [roteiro](roadmap.md): um release não espera trabalho pela
metade.

1. **A grade em janela e o SQLite:** a fonte de linhas por trás da grade, a janela com os
   tetos, a contagem, Ctrl+End e `g`, o filtro em segundo plano, e o SQLite (tabelas,
   views e query).
2. **CSV, TSV e JSON Lines** com o índice em segundo plano, e o arquivo que mudou.
   Também (pedido do autor em 2026-11-13): **abrir a tabela sem ler antes o arquivo
   para o texto e o hexa.** Um banco de 300 MB levou 16 s para abrir (build de debug),
   porque o viewer lia antes até o limite de tamanho (238 MiB) para os modos texto e
   hexa. Com `V`, um arquivo que abre como tabela lê só o começo (64 KiB); o resto é
   lido quando se vai para o texto ou o hexa (T, H), ou se a tabela não abrir. Um
   arquivo comprimido continua lido por inteiro, porque não abre como tabela. Na tabela,
   o cabeçalho deixa de dizer `(showing first N MiB)` e o número de controles bidi
   do texto, que são do texto e não da grade.
3. **DBF** pelo `seek`.
4. **O aviso das planilhas.** Já está escrito, numa branch local (`spreadsheet-row-limit`,
   sem commit): parado quando o autor pediu para planejar antes.

Ao fim: `docs/manual/viewer.md` (6 línguas), as notas da 0.8.0, o BACKLOG e o
[viewers.md](viewers.md) (subprojeto 3 começado pelas tabelas).

## Testes

- Grade em janela: andar, PgDn, Ctrl+End e `g` numa tabela de 200.000 linhas, sem nunca
  guardar mais que a janela; teto de bytes com linhas largas.
- SQLite: tabela, view e query de 200.000 linhas; a última linha é a certa; a contagem
  com Esc; query que demora para no tempo.
- CSV: registros com quebra de linha entre aspas antes e depois de uma posição anotada;
  TSV; BOM; JSONL com linhas inválidas no meio.
- Filtro: resultados além da linha 50.000; o teto de 1.000.000; Esc no meio.
- Arquivo que mudou: encurtado, trocado por outro e aumentado depois do índice; nenhuma
  linha errada aparece.
- Hostis (proptest): bytes aleatórios e posições do índice apontando para o meio de um
  registro não causam `panic`.
- Planilha: os dois testes já escritos (um `.xlsx` real de 50.002 linhas).
- Autoview: continua lendo só as primeiras linhas.

## Fora deste spec

- **Parquet em janelas:** 0.9.
- **O viewer de texto em janelas e os logs (follow, ir para o fim, cor por nível):** spec
  próprio, reaproveitando o índice daqui.
- **Ordenar uma tabela pela coluna:** não existe hoje, e numa tabela enorme exigiria
  outra estrutura.
- **Tabelas dentro de archives ou em máquinas remotas:** já não abrem como tabela hoje.

## Decisões (2026-11-13)

1. **O filtro numa tabela SQLite enorme:**
   - (a) **varrer em segundo plano, como nos outros formatos** (escolhido): o mesmo
     comportamento e a mesma sintaxe em todo lugar. Para velocidade, a query inteira
     (`SELECT … WHERE …`) já roda no próprio SQLite;
   - (b, descartado) mandar a condição para o SQLite como `WHERE`: rápido e usa os índices do banco,
     mas a sintaxe do filtro do Xylem não é a do SQLite, e alguns resultados poderiam
     diferir entre um CSV e um SQLite com os mesmos dados.
2. **Os tetos:** 2.000 linhas e 32 MiB na janela, 1.000.000 de resultados no filtro, e o
   índice até cerca de 1 bilhão de linhas (aprovados como propostos).
3. **O escopo da 0.8.0:** SQLite, CSV, TSV, JSON Lines e DBF; Parquet na 0.9.
4. **Sem PR só para o spec:** ele vai na mesma branch e no mesmo PR da implementação.
