# Estudo: consultar o disco, por fora e por dentro dos arquivos

Origem: conversa de 2026-11-14. Um recurso necessário, para o qual não há
implementações robustas no mercado. A sugestão foi consultar os arquivos da árvore
com SQL e agir sobre o resultado; o autor perguntou se valeria para o **conteúdo** dos arquivos,
como o Google Desktop fazia, com a preocupação da performance.

**Estado: proposta para estudo. Nada aprovado, nada implementado.** As decisões em
aberto estão marcadas com **(?)**, com a recomendação, e reunidas no fim.

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

Nada foi medido. Antes de aprovar as etapas 2 e 4, medir numa árvore conhecida (o
home do autor e um projeto grande):

1. Arquivos e MB por segundo do Ctrl+S de hoje, disco frio e quente. Diz se a etapa 2
   basta para um projeto e a partir de que tamanho o índice compensa.
2. Tempo e tamanho da primeira indexação, e de uma reindexação sem mudanças.
3. Tempo de uma consulta `words` com índice, com o Xylem recém-aberto.

O que se espera, a confirmar: por fora, imediato mesmo com centenas de milhares de
arquivos; por dentro sem índice, bom num ramo e lento no disco todo; com índice,
imediato, ao custo do espaço e da primeira indexação.

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
- PDF, DOCX e outros formatos que pedem leitor novo.
- E-mail, histórico do navegador e o resto do que o Google Desktop indexava.
- Busca por significado (embeddings) e qualquer coisa que mande conteúdo para fora da
  máquina.
- Consultas que escrevem.
