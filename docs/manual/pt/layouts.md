# Layouts: arquivos de largura fixa e multi-registro

Muitos arquivos trocados entre sistemas são texto sem nomes de coluna: cada linha é um registro, e quais caracteres guardam qual campo está escrito num manual. Arquivos de banco (CNAB 240), fiscais (SPED) e a exportação SDF do dBase e do Clipper são assim. Um layout diz ao Xylem onde está cada campo, e o visualizador mostra o arquivo como tabela: uma aba por tipo de registro, cada linha começando pela sua linha no arquivo.

## Abrir um arquivo por um layout

O Xylem reconhece alguns arquivos pela primeira linha e os abre como tabela sozinho, com `V` e no Autoview (F7). A nota diz qual layout usou: `Layout: FEBRABAN CNAB 240 — Padrão Geral  ·  K changes it`. Para qualquer outro arquivo, aperte `K`.

| Tecla | O que faz |
|---|---|
| `K` | a lista de layouts: digite parte de um nome para filtrá-la, Enter aplica um, **F2** pede o caminho de um arquivo de layout, Esc volta. Com um layout em uso, o primeiro item o desliga e mostra o arquivo como texto |
| `[` `]` | tipo de registro anterior ou seguinte |
| `Tab` | a lista dos tipos de registro, com quantas linhas cada um tem |
| `i` | os campos do registro: posição, tamanho, tipo, descrição e o significado de cada código; acima deles, o layout, a sua fonte, as conferências dos totais e os resultados |
| `T` | para o texto, na linha do registro sob o cursor, e de volta |
| `\` ou `&` | o filtro SQL, pelo nome do campo: `valor_pagamento > 1000` |
| Ctrl+B | um Batch dos registros da aba aberta ([Batch no visualizador](execute.md)) |

As abas vêm na ordem em que cada tipo de registro aparece pela primeira vez no arquivo: `0 - Header de Arquivo (1)`, `3A - Segmento A (420)`... Uma linha que não corresponde a nenhum registro vai inteira para a aba `! No record type`. A primeira coluna, `#line`, é a linha no arquivo, que é como os validadores dos bancos e do fisco apontam um erro. Quando um campo tem códigos com significado, a última linha da grade o mostra: `Content: 2 — Retorno (Banco → Cliente)`.

Os valores são lidos como o layout diz: códigos mantêm os zeros (`00012345`), datas viram `2026-11-09`, horas `14:30:00`, e valores mantêm as casas decimais exatas (`1234.50`); nada é arredondado. Um valor que não é o que o layout diz (letras num número, 31 de fevereiro) aparece como está escrito, e o `i` os conta.

### Conferência dos totais

Os trailers trazem totais: as linhas de um lote, o número de lotes, a soma dos pagamentos. O Xylem conta e soma durante a leitura, em inteiros, e diz no `i` quais conferem e quais não, com a linha, o que o arquivo diz e o que foi contado. Quando algum não confere, o cabeçalho avisa: `1 check failed: i shows`. Um arquivo lido só em parte (parado pelo Esc ou pelo tempo limite) não tem conferência.

## De onde vêm os layouts

| Onde | Usado sozinho? |
|---|---|
| O catálogo do Xylem, embutido no programa | sim, quando a primeira linha corresponde ao seu `[detect]` |
| A sua pasta: `~/.config/xylem/layouts/` (`%APPDATA%\xylem\layouts\` no Windows), até três níveis de subpastas | sim, e vem antes do catálogo |
| Ao lado do arquivo (`K`, seção `Here`), ou qualquer caminho (`K` e depois F2) | só quando você escolhe |

Um arquivo de layout na pasta que está sendo vista nunca é lido sozinho: uma pasta baixada ou extraída poderia trazer um feito para enganar. Quando você o escolhe, ele é lido com o mesmo rigor dos dados (veja Segurança, abaixo).

O catálogo tem, por enquanto:

| Layout | O que lê |
|---|---|
| FEBRABAN CNAB 240 — Padrão Geral | header e trailer de arquivo; pagamentos por crédito em conta, cheque, OP, TED e Pix (segmentos A, B, C, Z); pagamento de títulos de cobrança (J, J-52); cobrança (P, Q, R na remessa; T e U no retorno). Do manual da FEBRABAN, versão 11.0 |
| SPED — genérico | qualquer arquivo SPED de campos entre barras verticais (EFD ICMS/IPI, EFD-Contribuições, ECD, ECF...): uma aba por registro, a abertura e o encerramento de cada bloco e o bloco 9 com as suas conferências. Os registros que ele não declara têm os campos numerados (`CAMPO_02`, `CAMPO_03`...). O que vem depois do `9999` nos arquivos assinados (a assinatura digital) não é lido |

A pasta `layouts/` do [repositório](https://github.com/andrelnfernandes/xylem/tree/main/layouts) lista o que está planejado e como contribuir com um layout, sem precisar de Rust.

### SDF a partir de um .dbf

`COPY TO arquivo SDF` no dBase e no Clipper grava os registros lado a lado, cada campo com a sua largura. No `K`, um `.dbf` ao lado do arquivo (ou um indicado com F2) dá o seu layout: os campos em ordem, datas como `YYYYMMDD`, números com ponto. Campos memo não são exportados pelo SDF e ficam de fora.

## Como escrever um layout

Um layout é um arquivo TOML cujo nome termina em `.xlayout.toml`. Um posicional:

```
schema = 1
name = "Clientes do sistema antigo"
line_length = 40
[discriminator]
pos = [1, 1]
[[records]]
id = "0"
name = "Header"
role = "header"
fields = [
  { name = "tipo", pos = [1, 1], type = "code" },
  { name = "criado", pos = [2, 9], type = "date", format = "YYYYMMDD" },
  { pos = [10, 40], type = "filler" },
]
[[records]]
id = "1"
name = "Cliente"
fields = [
  { name = "tipo", pos = [1, 1], type = "code" },
  { name = "codigo", pos = [2, 7], type = "code" },
  { name = "nome", pos = [8, 30], type = "text" },
  { name = "saldo", pos = [31, 40], type = "decimal", decimals = 2, sign = "trailing" },
]
```

As posições começam em 1 e incluem as duas pontas: `pos = [8, 30]` vai do 8º ao 30º caractere. `start = 8, len = 23` diz o mesmo, e um `len` ao lado de `pos` precisa concordar com ele, o que pega erros de digitação. Com `strict_positions` (ligado por padrão), os campos de um registro precisam cobrir a linha inteira, sem buraco nem sobreposição: dê às partes vazias o tipo `filler`.

| Chave | O que diz |
|---|---|
| `schema` | sempre `1` |
| `name` | o nome mostrado no `K` e no cabeçalho |
| `format` | `"positional"` (o padrão) ou `"delimited"` |
| `encoding` | `"auto"` (a que o visualizador encontrou; o `O` a troca), `"utf-8"`, `"cp1252"`, `"cp850"` ou `"cp437"` |
| `offset_unit` | posicional: `"chars"` (o padrão) ou `"bytes"` |
| `line_length`, `strict_positions` | posicional: o tamanho de uma linha, e se os campos precisam cobri-la |
| `hide_filler` | campos `filler` ficam fora da grade (padrão `true`); o `i` sempre os lista |
| `delimiter`, `outer_delimiters` | delimitado: o separador, um caractere, e se as linhas também começam e terminam com ele, como no SPED |
| `allow_undeclared_records` | delimitado: registros que o layout não declara ganham uma aba própria |
| `decimal_sep`, `date_format` | padrões para todos os campos: `"implied"`, `","` ou `"."`; e o formato das datas |
| `stop_after_trailer` | parar no registro `trailer` (o que vem depois não é lido) |
| `[detect]` | como reconhecer o arquivo pela primeira linha: `line_length`, `first_line` (condições) e `extensions` |
| `[discriminator]` | onde está o código do registro: `pos` ou, delimitado, `col` |
| `description`, `country`, `category`, `spec_source`, `spec_version`, `spec_url`, `status` | sobre o layout e o seu manual; `status` é `"verified"` depois de conferido com ele |

Cada `[[records]]` tem um `id` (o seu código para o discriminador; `ids` acrescenta outros), um `name`, um `role` (`header`, `batch_header`, `detail`, `batch_trailer` ou `trailer`) e os seus `fields`. Quando só o código não separa os registros, `match` acrescenta condições sobre a linha, e `batch_match` condições sobre a linha do header do seu lote: no CNAB 240, o segmento A de um pagamento e o de um débito só diferem pela operação no header de lote. Uma condição é um lugar (`pos` ou `col`) e um de `value = "3"`, `in = ["1", "2"]` ou `regex = "^[0-9]+$"`. Entre registros com o mesmo código, o que tem mais condições é tentado primeiro.

| Chave do campo | O que diz |
|---|---|
| `name` | o nome da coluna, também para o filtro SQL |
| `pos`, `start` e `len`, ou `col` | onde ele está |
| `type` | `text`, `code` (dígitos que não são quantidade: CPF, conta, CEP), `int`, `decimal`, `date`, `time` ou `filler` |
| `decimals`, `decimal_sep` | casas decimais, e onde está a vírgula: `"implied"` (o `V99` do COBOL), `","` ou `"."` |
| `sign` | `"none"`, `"leading"`, `"trailing"` ou `"overpunch"` (o sinal zonado do COBOL: `{`, `A`-`I`, `}`, `J`-`R`) |
| `format` | datas: `YYYYMMDD`, `DDMMYYYY`, `YYMMDD`, `DDMMYY` ou `YYYYDDD`; horas: `HHMMSS` ou `HHMM`. Anos de dois dígitos de 00 a 69 são dos anos 2000 |
| `trim` | espaços tirados do texto: `"right"` (o padrão), `"left"`, `"both"` ou `"none"` |
| `desc`, `values` | a descrição, e o que cada código significa: `values = { "0" = "Inclusão", "9" = "Exclusão" }` |
| `check` | com o que o valor precisa concordar, abaixo |

| `check` | O valor precisa ser |
|---|---|
| `total_lines` | as linhas do arquivo até esta |
| `record_count:1*` | as linhas dos registros com este id; `*` pega todo id que começa assim, `+` acrescenta outros |
| `sum:3A.valor` | a soma de um campo de um registro no arquivo inteiro; `+` acrescenta outros |
| `batch_lines` | as linhas do header do lote até esta |
| `batch_sum:3A.valor+3J.valor` | a soma no lote |
| `count_by_record:REG_BLC` | as linhas cujo código é o valor do campo indicado (o 9900 do SPED) |
| `block_lines` | as linhas cujo código começa como o desta (o x990 do SPED) |

Copybooks COBOL (`copybook = ...`), EBCDIC e números compactados (`COMP-3`) estão planejados e ainda não são lidos.

## Segurança

Tanto os dados quanto um layout escolhido fora da sua pasta podem ser hostis. Um layout é lido com o mesmo rigor dos dados: no máximo 1 MiB, qualquer chave ou tipo desconhecido é recusado, toda posição e todo tamanho são conferidos, nomes não podem ter caracteres de controle nem de direção do texto, e registros, campos, códigos, condições e expressões regulares têm limites. Expressões regulares rodam em tempo linear. Um layout não pode ler outros arquivos: `copybook` é recusado por enquanto.

Os dados são lidos com os limites do modo tabela: 3 s (100 ms no Autoview), 50.000 linhas por aba, 256 colunas, e linhas com mais de 512 KiB não são divididas em campos. O Esc para a leitura. As células são tornadas seguras para o terminal como quaisquer outras: caracteres de controle e de direção do texto aparecem como `·`.

Veja também: [o visualizador](viewer.md).
