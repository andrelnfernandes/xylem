# Filtrar um CSV ou uma planilha com SQL

Este procedimento abre `customers.csv` como uma tabela e mantém apenas as
linhas cujo plano seja `team` ou cujo saldo seja superior a 1000.

## Passo a passo

1. Na janela de arquivos, posicione o cursor sobre `customers.csv` e pressione
   `V`. O visualizador exibe o arquivo como uma tabela, com a primeira linha
   servindo de cabeçalho para as colunas.
2. Pressione `&`. O prompt solicita a condição.
3. Digite `plan = 'team' OR balance > 1000` e pressione Enter.

![Os três passos: V em customers.csv exibindo nove linhas, o prompt de & e a condição mantendo três linhas](../../screenshots/howto-table-sql.gif)

O rodapé exibe `Filter matched 3 rows`. As linhas preservam a numeração original
do arquivo. Pressionar `&` com o prompt vazio volta a exibir todas as linhas.

## Quais arquivos abrem em modo tabela

| Formato | Extensões |
|---|---|
| CSV, TSV | `.csv` `.tsv` `.tab` |
| JSON Lines | `.jsonl` `.ndjson` |
| Planilhas | `.xlsx` `.xls` `.xlsb` `.ods` |
| dBase | `.dbf` |
| SQLite | `.db` `.sqlite` `.sqlite3` |
| Parquet | `.parquet` |

O arquivo precisa residir neste disco local. Arquivos em máquinas remotas ou
dentro de arquivos compactados abrem em formato de texto comum.

## Como escrever a condição

| Para manter linhas onde | Condição |
|---|---|
| uma coluna seja igual a um texto | `plan = 'team'` |
| um número esteja em uma faixa | `balance >= 0 AND balance < 100` |
| um texto comece com certo prefixo | `name LIKE 'Ana%'` |
| um texto contenha um trecho, ignorando maiúsculas | `city ILIKE '%paulo%'` |
| uma célula esteja vazia | `email IS NULL` |
| uma coluna não possua nome utilizável | use sua letra correspondente: `C = 'pro'` |

Uma célula com formato numérico é tratada como número: digite `code = 7`, nunca
`code = '7'`. [O filtro SQL](tables.md#o-filtro-sql) lista todos os operadores.

## Em planilhas ou bancos de dados

- Planilhas não possuem nomes de colunas: as colunas são `A`, `B`, `C`. As
  teclas `[` e `]` navegam entre as abas anterior e seguinte, e Tab lista todas.
- Em um banco SQLite, Tab lista as tabelas e visões (*views*). O mesmo prompt
  também aceita consultas completas iniciadas por `SELECT` ou `WITH`:

```
SELECT plan, count(*), sum(balance) FROM customers GROUP BY plan
```

O Xylem opera somente em leitura no banco de dados. Comandos como `UPDATE` e
`DELETE` são recusados.

## Gravar o resultado

Ctrl+B grava as linhas filtradas em um novo arquivo: CSV, JSON, HTML, script SQL
ou tabela SQLite. [Batch](execute.md) descreve os formatos disponíveis.

## Quando não se comporta como esperado

- **`SQL error`.** A condição não pôde ser interpretada. O filtro que estava
  ativo é mantido.
- **Nenhuma linha corresponde a uma coluna visível.** Um nome de coluna
  inexistente é considerado nulo (*NULL*) sem gerar erro. Confira a grafia ou
  utilize a letra da coluna.
- **O arquivo abre em modo texto.** A extensão não consta na tabela acima, ou o
  arquivo está comprimido, remoto ou dentro de um arquivo compactado. A tecla
  `T` alterna entre o modo tabela e o modo texto quando ambos estão disponíveis.

Veja também: [o visualizador](viewer.md), [escolher arquivos com SQL](howto-sql-filter.md).
