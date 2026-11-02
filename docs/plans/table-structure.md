# Plano: fim e começo da tabela, e a estrutura das colunas

Origem: pedido de 2026-11-02, notado em um `.dbf` aberto com `V`. Estado: implementado
na branch `table-structure`.

## 1. O que falta

1. No modo tabela não há tecla para o último registro (nem para o primeiro). `Home` e
   `End` já vão à primeira e à última coluna, e `g` pede um número.
2. Não há como ver a estrutura da tabela, o `DISPLAY STRUCTURE` do dBase/Clipper:
   nome, tipo, tamanho e decimais de cada campo.

## 2. Teclas

Teclas do viewer em uso no modo tabela: setas, PgUp/PgDn, Home/End (colunas),
Alt+Home/Alt+End (primeiro e último arquivo marcado, em Ctrl+V), `m`/Insert, `M`,
Ctrl+B, `q`/`v`/Esc/Enter, `w`, `#`/`l`, `g`/`:`, `t`, `&`, `\`, Tab, `[`, `]`, `h`,
`o`, `c`, `f`, `b`, `/`, `s`, `+`, `-`, Espaço, `n`, `p`, `u`, `e`, `d`, `y`,
Ctrl+Insert, F9. Livres: `a`, `i`, `j`, `k`, `r`, `x`, `z`.

- **Ctrl+Home / Ctrl+End:** primeiro / último registro, na mesma coluna. No modo texto
  Home/End vão ao começo e ao fim do arquivo; na tabela Home/End já são das colunas,
  e Ctrl+Home/Ctrl+End é a convenção das planilhas para ir às pontas. Sob um filtro,
  o primeiro e o último dos registros mantidos. Em terminais que não mandam Ctrl+Home,
  `g` continua servindo.
- **`i`:** a estrutura, numa janela sobre a grade (como a lista de abas do Tab). Setas,
  PgUp/PgDn e Home/End rolam; Esc, Enter ou `i` de novo fecham.
- Correção de passagem: hoje a tabela toma Home/End com qualquer modificador, e
  Alt+Home/Alt+End não chegam aos arquivos marcados. Com Alt, voltam a chegar.
- Correção achada ao testar no arquivo real: o viewer passava à grade a altura toda
  da moldura, mas a grade reserva 2 linhas (nomes das colunas e conteúdo da célula).
  ↓, `m` e `g` deixavam o cursor até 2 linhas abaixo da tela, e Ctrl+End mostraria a
  última página sem a última linha. `Viewer::grid_rows()` e `TABLE_CHROME` dão a
  mesma conta ao viewer, ao `g` e ao desenho.

## 3. O que cada formato sabe das colunas

O leitor (`records.rs`) entrega ao `RecordSink`, junto com os nomes, uma `Structure`:
os títulos das colunas da janela, uma linha por coluna e algumas linhas sobre o todo.
O método tem implementação vazia por padrão, e o Batch não o usa.

| Formato | Colunas da janela | Sobre o todo |
|---|---|---|
| DBF | Nome, Tipo (`C Character`, `N Numeric`, `D Date`, `L Logical`, `M Memo`, `F`, `Y`, `T`, `I`, `B`), Tamanho, Decimais | registros que o cabeçalho declara, última atualização, tamanho do registro |
| SQLite (tabela ou visão) | Nome, Tipo declarado, Not null, Default, Chave primária (`pragma_table_info`); numa visão, as colunas que ela dá | tabela ou visão, e os índices com suas colunas (`pragma_index_list` e `pragma_index_info`, no máximo 32) |
| SQLite (consulta inteira) | Nome e tipo declarado das colunas do resultado (`sqlite3_column_decltype`, feature `column_decltype` do `rusqlite`); `(expression)` para o que não vem direto de uma coluna | — |
| Parquet | Nome, Tipo Arrow, Aceita nulo | linhas e grupos de linhas declarados, programa que gravou (`created_by`) |
| CSV, JSON Lines, planilhas | Nome, Valores (o que se viu nas linhas lidas: número, texto, lógico, vazio), Mais largo | um aviso de que o formato não declara tipos |

Os decimais do DBF não são públicos no crate `dbase`: são lidos dos descritores de
campo do próprio arquivo (byte 17 de cada descritor de 32 bytes a partir do byte 32),
no máximo os 256 primeiros, e só quando nome, tipo e tamanho batem com o que o crate
leu; senão aparece `?`.

## 4. Segurança

- A estrutura vem de arquivo hostil. Nomes, tipos declarados (SQLite aceita qualquer
  texto), defaults, tipos Arrow com nomes de fuso e `created_by` passam por
  `TableGrid::sanitize_cell` no `GridSink`: controle, bidi e separadores de linha viram
  `·`, e cada texto tem no máximo 2.048 caracteres. A janela ainda corta cada coluna à
  largura da tela.
- Limites: no máximo `MAX_COLS` (256) linhas na estrutura, como na grade. A leitura dos
  decimais lê no máximo 32 + 256 × 32 bytes.
- SQLite: o nome da tabela vai como parâmetro de `pragma_table_info(?1)`, nunca como
  texto de SQL; a conexão continua em modo somente leitura, defensivo, com
  `query_only` e `trusted_schema = OFF`. Um erro aí não impede a tabela de abrir: a
  janela cai no que se viu nas linhas.
- A estrutura sem tipos declarados é calculada só quando `i` é pressionado, sobre as
  linhas já carregadas (no máximo 50.000 × 256 células).

## 5. Rodapé, ajuda e testes

- Rodapé do modo tabela: `i structure` na primeira linha, e `Ctrl+Home Ctrl+End first,
  last row` na segunda, junto de `Row N of M`. Com a janela aberta: `COLUMNS ↑↓ PgUp
  PgDn scroll  Esc back`.
- `docs/manual/viewer.md` e as 5 traduções (de, es, fr, pt, zh): as duas teclas na
  tabela do modo tabela, e um parágrafo sobre a estrutura de cada formato.
- Pedido do autor (2026-11-02): as duas teclas valem no SQLite como requisito, em
  tabelas, visões e consultas, com testes próprios.
- Testes: Ctrl+End/Ctrl+Home com e sem filtro, e numa tabela e numa consulta SQLite;
  os índices (com expressão, `UNIQUE` e o corte em 32); Alt+Home na tabela vira `Action::First`;
  `i` abre e fecha a janela; a estrutura de um DBF (tipos, tamanhos, decimais, total),
  de um SQLite (tipo, not null, chave), de um Parquet e de um CSV; um nome hostil com
  escape e bidi chega limpo.
