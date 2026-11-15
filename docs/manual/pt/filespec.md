# Filespec: o filtro de arquivos

`F` define quais arquivos aparecem na janela de arquivos (cada janela tem o seu
próprio filtro). O filespec atual aparece no título da janela. As
especificações são separadas por espaços, `;` ou `,`, de três tipos, como no
ZTreeWin:

- **nome:** um padrão glob (`*.rs`, `a?c*`, `[abc]*`), sem diferenciar
  maiúsculas e minúsculas; entre aspas, um nome pode conter espaços
  (`"meu arquivo*"`);
- **data:** `<`, `=`, `>`, `<=`, `>=` ou `<>` e uma data na ordem do formato
  configurado, com `/`, `.` ou `-` e ano de 2 ou 4 dígitos (`>01/05/2026`;
  `2026-05-01` funciona sempre), ou `TODAY` e `TODAY-n` (`>TODAY-7`: os últimos
  7 dias);
- **tamanho:** o mesmo com um `s` após o operador (`>=s200000`); `K`, `M` e `G`
  também funcionam (`>s100M`).

Um `-` no início nega a condição (`-*.bak`; `-<s1K` é o mesmo que `>=s1K`).
Dentro de cada tipo, especificações positivas são combinadas com OU (OR),
exceto `>a` seguido imediatamente de `<b`, que representa o intervalo entre os
dois (`>=01/05/2026,<=31/05/2026`: maio); `<>` exclui. Entre tipos diferentes,
aplica-se E (AND): `*.txt,=TODAY,<s1000` são os arquivos `.txt` de hoje menores
que 1000 bytes.

`*` (ou um prompt vazio) mostra tudo. **↑** traz filespecs anteriores ([histórico](execute.md#history)).

## Uma condição SQL (W)

**W** recebe uma condição escrita como o `WHERE` de uma consulta SQL, e a lista passa a mostrar apenas os arquivos que a satisfazem, além do filespec. **Ctrl+W** olha apenas para os arquivos marcados. Funciona no diretório, em Branch e em Showall, e somente sobre o que já foi lido (*logged*): um arquivo que o Xylem não está mostrando nunca é resultado.

```sql
ext = 'jpg' AND size > 10000000
ext IN ('md', 'txt') AND age_days < 7
name LIKE 'relatorio%' AND depth >= 2
```

Cada arquivo é uma linha com estas colunas:

| Coluna | O que é |
|---|---|
| `name` | o nome, com a extensão |
| `ext` | a extensão, em minúsculas e sem o ponto (`''` quando não há) |
| `path`, `dir` | o caminho completo do arquivo e o da sua pasta |
| `depth` | quantas pastas abaixo da raiz da árvore fica a pasta do arquivo |
| `size`, `disk` | o tamanho em bytes e o espaço que ocupa no disco |
| `modified` | a data da última alteração, na hora local, como `2026-11-15 09:30:00`: `modified >= '2026-01-01'` |
| `age_days` | dias desde essa alteração (`age_days < 7`: a última semana) |
| `mode` | as permissões como são escritas: `mode = 644` |
| `kind` | `file`, `link` ou `other` |
| `tagged` | se o arquivo está marcado: `NOT tagged` |

Os operadores são os do [filtro de tabelas](viewer.md#the-sql-filter): `=`, `<>`, `<`, `<=`, `>`, `>=`, `LIKE`, `ILIKE`, `IN`, `IS NULL`, `AND`, `OR`, `NOT` e parênteses. Números são escritos por extenso (`10000000`, não `10e6`). Um nome de coluna inexistente é recusado mostrando a lista das colunas válidas, e uma condição que nenhum arquivo satisfaz não altera a lista.

O título da janela de arquivos mostra a condição após o filespec, e a mensagem informa quantos arquivos casaram. **Ctrl+T** marca todos eles, e a partir daí podem ser copiados, movidos, excluídos ou enviados para um [Batch](execute.md). **W** com o prompt vazio volta a mostrar toda a lista. Na primeira vez, **↑** traz alguns exemplos prontos para editar.

### Dentro dos arquivos: contains()

`contains('texto')` é verdadeiro para um arquivo que contenha o texto, sem diferenciar maiúsculas e minúsculas, assim como a [busca](search.md) do Ctrl+S o encontra (`*` dentro do texto vale por qualquer trecho na mesma linha):

```sql
ext IN ('md', 'txt') AND contains('fatura')
contains('TODO') AND NOT contains('DONE')
```

`contains` sempre lê os arquivos quando a condição roda, o que leva tempo: um índice guarda palavras, não pedaços no meio delas (para usar o índice, veja `words` abaixo). Antes de ler, o Xylem pergunta, informando quantos arquivos e quantos bytes serão lidos: `No index: read the contents of 1,234 files (2.3 GB)?` (em máquina remota, avisa também que serão baixados via SSH). Apenas os arquivos cuja resposta depende do conteúdo são lidos: com `ext = 'md' AND contains('x')`, somente os arquivos `.md`. O rodapé mostra o andamento da leitura e **Esc** a interrompe: a lista passa a mostrar o que foi encontrado até ali, e a mensagem informa que parou.

Alguns arquivos ficam de fora da leitura, e a mensagem ao final informa quantos de cada tipo foram ignorados: arquivos binários (um byte zero nos primeiros 8 KiB), links simbólicos (não são seguidos), arquivos acima de 256 MiB e os que não puderam ser lidos. Um arquivo deixado de fora nunca entra no resultado, nem mesmo em `NOT contains(...)`.

Em montagem de nuvem, `contains` é recusado: cada arquivo precisaria ser baixado. A lista mantém o que foi encontrado no momento da leitura: um arquivo alterado ou criado depois só aparece rodando **W** novamente.

`contains` entra sozinho como condição própria ligada por `AND`, `OR` e `NOT` (`contains('a') = 1` é recusado), e recebe um texto entre aspas.

### Palavras inteiras, prefixos e índice: words()

`words('fatura marco')` é verdadeiro para um arquivo que contenha todas essas palavras, em qualquer ordem e em qualquer posição. Uma palavra é uma sequência de letras e dígitos, comparada sem diferenciar maiúsculas de minúsculas nem acentos: `words('acao')` encontra `Ação`. Ao contrário de `contains`, ela busca palavras inteiras (`words('fatura')` não encontra `faturas`), a menos que você termine a palavra com `*` para buscar por prefixo: `words('fatur*')` (ou `words('andre*')`) encontra qualquer palavra que comece por `fatur` (ou `andre`).

```sql
words('fatura 2024') AND ext IN ('txt', 'md', 'csv')
words('andre*')
```

`words` pode ser respondido por um **índice** do ramo, sem precisar ler os arquivos na hora. **Alt+N** na árvore (ou na janela de arquivos, para a pasta do arquivo) indexa os arquivos lidos do ramo, após confirmação: ele os lê uma vez, em segundo plano, podendo parar com **Esc** e mantendo o que já foi feito. Indexar novamente lê apenas os arquivos que mudaram e remove os que saíram. Em um ramo que já possui índice, Alt+N oferece atualizar o índice ou apagá-lo.

|  | Sem índice | Com índice |
|---|---|---|
| Resposta | lê cada arquivo quando W roda | imediata |
| Custo | tempo a cada condição | tempo e espaço ao indexar |
| Procura | trecho de texto (`contains`) ou palavras/prefixos (`words`) | palavras e prefixos (`words`) |
| Arquivo alterado depois | o conteúdo atual | lido novamente na hora, como sem índice |
| Onde | qualquer lugar lido (*logged*) | apenas nos ramos indexados com Alt+N |

Antes de ler, a confirmação informa ambos: `Index: 3,120 files up to date. No index: read the contents of 12 files (1.2 MiB)?`. Quando o índice responde por todos os arquivos, não há pergunta, e a mensagem informa `answered by the index`. O índice nunca produz resposta desatualizada: um arquivo alterado após ter sido indexado (outro tamanho ou data) é lido novamente na hora.

O que fica fora do índice: arquivos binários, arquivos acima de 64 MiB (W os lê quando necessário), links simbólicos, e estas pastas e arquivos, que são grandes e mudam o tempo todo ou guardam segredos: `.git`, `target`, `node_modules`, `.ssh`, `.gnupg`, `.env`, `*.key`, `*.pem`, `id_rsa`, `id_ed25519`. Arquivos em máquina remota, dentro de pacote compactado ou em montagem de nuvem não são indexados.

O índice é uma cópia das palavras dos arquivos, mantida no cache do Xylem (`~/.cache/xylem/index`, em uma pasta que só o seu usuário pode ler; `%LOCALAPPDATA%\xylem\cache\index` no Windows) até que você o apague com Alt+N ou exclua essa pasta. Um índice ilegível (corrompido ou de outra versão do Xylem) é informado na confirmação, seus arquivos são lidos normalmente, e Alt+N o reconstrói.

W é o primeiro passo de consulta ao disco: resultados em formato de tabela virão depois.

Veja também: [a janela de arquivos](files.md), [marcações](tagging.md) (Ctrl+T
na árvore marca o que corresponde ao filespec).
