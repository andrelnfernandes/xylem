# Tabelas: planilhas, CSV e bancos de dados

Alguns arquivos abrem como uma grade em vez de texto. O Xylem decide pela extensão, e só para arquivos sem compressão neste disco: um arquivo de máquina remota ou de dentro de um arquivo compactado aparece como conteúdo comum, com a nota `Tables are shown only for files on this disk`.

| Formato | Extensões | Nomes das colunas |
|---|---|---|
| CSV, TSV | `.csv` `.tsv` `.tab` | a primeira linha, quando pelo menos metade dela não é número; senão A, B, C... |
| JSON Lines | `.jsonl` `.ndjson` | as chaves dos objetos |
| Planilhas | `.xlsx` `.xls` `.xlsb` `.ods` | A, B, C... (a primeira linha é uma linha como as outras) |
| dBase, Clipper, FoxPro | `.dbf`, com seu arquivo memo `.dbt` | os nomes dos campos |
| SQLite | `.db` `.sqlite` `.sqlite3` | as colunas da tabela |
| Parquet | `.parquet` | as colunas do arquivo |

O separador do CSV (vírgula, ponto e vírgula, tabulação ou barra vertical) é detectado pelo início do arquivo.

| Tecla | O que faz |
|---|---|
| ↑ ↓ | linha anterior ou seguinte |
| ← → | coluna anterior ou seguinte |
| PgUp PgDn | uma página de linhas para cima ou para baixo |
| Home End | primeira ou última coluna |
| Ctrl+Home Ctrl+End | primeira ou última linha; sob um filtro, das linhas que ele mantém |
| `[` `]` | aba anterior ou seguinte de uma planilha, ou tabela de um banco SQLite, dando a volta; isso limpa o filtro |
| `Tab` | a lista de abas ou tabelas: digite parte de um nome para reduzi-la, Enter abre uma e Esc volta; abrir outra limpa o filtro. Num banco, as views vêm depois das tabelas, marcadas com `(view)` |
| `g` ou `:` | pula para uma linha pelo número dela no arquivo; se o filtro a escondeu, para a próxima que ele manteve |
| `\` ou `&` | o filtro SQL |
| `m` ou Insert | marca ou desmarca o registro sob o cursor e vai para o próximo; o **Ctrl+B** passa a gravar só os marcados |
| `M` | limpa as marcas |
| `i` | a estrutura da tabela: o que o arquivo diz de cada coluna (veja abaixo) |
| Enter | o registro sob o cursor em forma de ficha, um campo por linha: ↑ ↓ escolhem o campo, ← → vão ao registro anterior ou seguinte, `c` copia o registro inteiro (`NOME : valor`, uma linha por campo) e `y` o valor do campo, para colar num editor ou terminal; Esc volta. O que se copia é o que a tela mostra: caracteres de controle e de direção do texto, quebras de linha inclusive, viram `·`, e colar num terminal não executa nada. Fora da tabela, Enter fecha o visualizador |
| `T` | vai para o texto do arquivo e volta |
| `H` | da tabela para o texto, depois hexadecimal, depois de volta à tabela |

A moldura mostra `TABLE [2/5: nome]` quando há mais de uma aba ou tabela, e a linha de baixo mostra onde está o cursor: `Cell B7 (Row 7 of 120)`. Com filtro, as linhas mantêm o número que têm no arquivo, e essa linha diz também qual é a ocorrência: `Cell A13 (Row 13 of 31, match 2 of 8)`. A última linha da grade traz o conteúdo inteiro da célula e o filtro em uso. A busca (`F`, `B`, `/`) não olha dentro da grade e nada encontra ali; ela funciona no modo texto.

As marcas continuam sob um filtro, e o Batch grava o que está marcado e o filtro mantém. Elas pertencem à aba ou tabela aberta e se perdem ao trocá-la. Se o arquivo mudar depois da marcação, o Batch recusa, pois outros registros podem estar naquelas posições.

## A estrutura

`i` mostra, sobre a grade, as colunas em ordem com o que o arquivo diz de cada uma, como o `DISPLAY STRUCTURE` do dBase e do Clipper. As setas, PgUp, PgDn, Home e End rolam a lista; Esc, Enter ou `i` fecham.

| Formato | O que mostra |
|---|---|
| dBase, Clipper, FoxPro | nome, tipo (`C` caractere, `N` numérico, `D` data, `L` lógico, `M` memo, `F` ponto flutuante, `I` inteiro, `B` double, `Y` moeda, `T` data e hora), tamanho e decimais; acima, o número de registros e a data da última atualização escritos no cabeçalho, e o tamanho de um registro |
| SQLite | o tipo declarado de cada coluna, `Not null`, o valor padrão e a posição na chave primária; acima, se é tabela ou visão, e os índices com suas colunas (no máximo 32). Uma visão mostra as colunas que dá; uma query inteira, as do resultado, com o tipo de cada uma quando vem direto de uma coluna de tabela |
| Parquet | o tipo Arrow de cada coluna e se aceita nulos; acima, as linhas e os grupos de linhas que o arquivo declara e o programa que o gravou |
| CSV, JSON Lines, planilhas | não declaram tipos: o que as linhas lidas têm em cada coluna (inteiro, decimal, lógico, texto, vazio) e o valor mais largo |

Nomes, tipos e valores padrão vêm do arquivo e passam pela mesma limpeza das células: caracteres de controle e de direção do texto aparecem como `·`. Um `.dbf` cujo cabeçalho se contradiz mostra `?` nos decimais.

## O filtro SQL

`\` ou `&` pede uma condição, como o `WHERE` do SQL (a palavra `WHERE` é opcional), e mostra só as linhas que a atendem. O prompt começa com o filtro atual; deixe-o vazio para ver todas as linhas de novo. Uma nota diz `Filter matched N rows` ou `SQL error: ...`; depois de um erro, o filtro anterior continua valendo.

```
age >= 30 AND name LIKE 'Ali%'
WHERE status IN ('open', 'late') OR total > 1000.50
NOT (city = 'Rio' OR city IS NULL)
B ILIKE '%silva%'
```

| Operador | O que faz |
|---|---|
| `=` `!=` `<>` `<` `<=` `>` `>=` | comparam |
| `AND` `OR` `NOT` e parênteses | combinam condições (`AND` vem antes de `OR`) |
| `LIKE` `ILIKE` | combinam com um padrão, `%` para qualquer texto e `_` para um caractere; `ILIKE` ignora maiúsculas e minúsculas |
| `IN (...)` `NOT IN (...)` | está, ou não está, na lista |
| `IS NULL` `IS NOT NULL` | a célula está, ou não está, vazia |

As palavras-chave podem estar em maiúsculas ou minúsculas.

- **Colunas:** o nome da coluna, em maiúsculas ou minúsculas e sem aspas, ou a sua letra, de `A` a `Z`. A letra é o jeito de chegar a um nome com espaços ou símbolos. Uma coluna que não existe conta como vazia, sem erro.
- **Valores:** texto entre aspas simples ou duplas (`\'` para uma aspa dentro dele), números, `TRUE`, `FALSE` e `NULL`. O que está entre aspas é sempre texto, nunca coluna.
- **Células:** o tipo vem da aparência da célula. Vazia é `NULL`, `true` e `false` são booleanos, números são números e o resto é texto. Por isso escreva `code = 7`, não `code = '7'`: texto e número nunca são iguais, e uma célula com `007` é o número 7. Comparações de texto distinguem maiúsculas de minúsculas.
- **Tamanho:** um filtro tem no máximo 1.024 bytes e 128 termos.

## Uma query inteira num banco

Num banco SQLite, o mesmo prompt aceita uma query inteira, começando com `SELECT` ou `WITH`, com tudo o que o SQLite faz: `JOIN`, `GROUP BY`, `ORDER BY`, funções.

```
SELECT c.name, sum(o.price) FROM customers c JOIN orders o ON o.code = c.code GROUP BY c.name
```

A moldura diz `QUERY:`, e a grade mostra o resultado. Uma condição filtra então essas linhas, e **Ctrl+B** grava tudo o que a query dá. Deixar o prompt vazio limpa o filtro e depois volta à tabela; `Tab`, `[` e `]` também vão para uma tabela. A query só lê, uma de cada vez: qualquer outra coisa é recusada, e um erro deixa a tela como estava. Os limites abaixo valem, então uma query lenta para em 3 s e mostra as linhas que obteve; o Batch não tem esse limite. O Xylem não altera bancos: `UPDATE`, `DELETE` e parecidos recebem uma mensagem dizendo isso. Para alterar um banco, use outro programa: `O` na janela de arquivos o abre no definido em `opener`, e `'*.db' = 'sqlite3 %f'` em `[editors]` faz o `E` abri-lo no `sqlite3`.

## Tabelas grandes

Com **V**, estas tabelas não têm limite de linhas: um arquivo CSV ou TSV, um arquivo JSON Lines, um arquivo dBase (`.dbf`) e uma tabela ou view de um banco SQLite, ou uma query digitada nele. A grade guarda as linhas perto do cursor, no máximo 2.000 delas e 32 MiB de texto, e lê as outras do arquivo conforme o cursor anda, sem prender a tela: uma linha ainda não lida aparece como `…` por um instante.

- **O total** aparece enquanto é contado: `Row 1 of 2,000+ (counting…)`, e depois `Row 1 of 8,412,903`. **Ctrl+End** vai para a última linha e `g` para qualquer uma; antes de a contagem terminar, `g` além dela avisa que a contagem ainda não chegou lá.
- **O filtro** (`&`) percorre a tabela inteira, em segundo plano. A linha abaixo da grade diz até onde ele foi, `1,203 matches so far, row 2,300,000 of 8,412,903`, e as linhas aparecem conforme são encontradas. Ele guarda no máximo 1.000.000 de ocorrências, e então diz `first 1,000,000 matches`.
- **Esc** para a contagem ou o filtro, mantendo o que já encontraram; o Esc seguinte fecha o visualizador.
- A largura das colunas vem das primeiras linhas, e uma coluna se alarga quando uma linha mais larga aparece na tela. Um arquivo JSON Lines também pode ganhar colunas durante a contagem, quando uma chave aparece pela primeira vez mais adiante.
- **CSV, TSV e JSON Lines** são lidos uma vez do início ao fim durante a contagem, e o Xylem anota onde começa cada 256ª linha, de modo que qualquer linha é alcançada lendo no máximo 255 outras. Essa anotação fica só na memória, cerca de 8 bytes a cada 256 linhas, e some quando o visualizador fecha. Um registro CSV com quebra de linha dentro de aspas continua sendo uma linha. Um registro CSV com mais de 16 MiB para a leitura, e uma linha de JSON Lines com mais de 512 KiB é pulada.
- **Os registros dBase** têm todos o mesmo tamanho, então o Xylem vai direto a qualquer um. A contagem olha só se cada registro foi excluído, porque um registro excluído não é uma linha: o Xylem anota onde está cada 256ª linha, do mesmo jeito.
- **Quando o arquivo muda** enquanto está aberto (cresce, diminui ou é substituído), o que o Xylem anotou dele deixa de valer: nada mais é lido dele, e o cabeçalho diz `(the file changed: R reloads)`. **R** o lê de novo.
- **O texto e o hexadecimal** de um arquivo que abre como tabela só são lidos quando você vai até eles (**T**, **H**): até lá o Xylem lê só o início do arquivo, e assim uma tabela grande abre na hora.

O Autoview (F7) não abre bancos, e lê só as primeiras linhas das outras tabelas. Os outros formatos ficam com o limite abaixo por enquanto.

## Limites

Fora as tabelas grandes acima, uma tabela guarda no máximo 50.000 linhas e 256 colunas, e leva no máximo o que os outros limites do visualizador permitem: 3 s (100 ms no Autoview), e uma planilha que desempacota para mais que o limite de tamanho (1 MiB no Autoview) não abre como tabela. O que passa disso não aparece e o filtro não vê; o cabeçalho avisa: `(showing first 50,000 rows)` ou `(stopped by the time limit: showing N rows)`. Enquanto a tabela é lida, a tela fica livre e uma nota avisa; Esc para a leitura e mostra o que já veio, `(stopped by Esc: showing N rows)`, e o Esc seguinte fecha o visualizador. Uma célula mostra no máximo 2.048 caracteres e depois `…`. Caracteres de controle e de direção do texto aparecem como `·`.

Quando uma planilha (XLSX, XLSB, XLS, ODS) tem mais linhas que isso, o cabeçalho e a nota dizem isso: `(the sheet has more than 50,000 rows, the most Xylem shows: open it in Excel or another spreadsheet program to see them all (O))`, ou seja, abra-a no Excel ou em outro programa de planilhas para ver todas. **O** a abre no programa que o seu sistema usa para ela.

## Tabelas em bibliotecas C

Bancos SQLite (`.db`, `.sqlite`, `.sqlite3`) e arquivos Parquet são lidos por bibliotecas escritas em C, que não têm a segurança de memória do Rust. Por isso, eles só abrem como tabela com `V`. O Autoview lê tudo por onde o cursor passa e mostra esses arquivos como conteúdo comum, com a nota `Parquet or SQLite: V opens the table`. Abra um banco de origem não confiável só se aceitar esse risco. Arquivos `.ods` e `.xls` também só abrem como tabela com `V`: o leitor deles monta a planilha inteira primeiro, e essa parte não pode ser interrompida.

Veja também: [o visualizador](viewer.md).
