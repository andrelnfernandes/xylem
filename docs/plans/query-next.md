# Roteiro: os próximos passos da consulta ao disco

Origem: conversa de 2026-11-15. O autor vai fazer estes passos com outro assistente
(Gemini) e depois pedir uma revisão. Este documento é o briefing: lê-se de ponta a ponta
antes de começar.

**Estado: roteiro aprovado pelo autor para execução.** O desenho de fundo está em
[query.md](query.md) (seção "Andamento" e "Regras decididas pelo autor"); o que está
aqui não muda nenhuma decisão de lá.

## Antes de começar

### O que já existe (no `main`, PRs #118 e o da branch `words-prefix`)

- **`W` / `Ctrl+W`:** uma condição SQL (o `WHERE`) sobre os arquivos da lista, ou só
  sobre os marcados. Código em `src/model/query.rs` (`FileQuery`, colunas em
  `COLUMNS`, `Row` como `RowContext`) e `src/pane.rs` (`query`, `query_takes`,
  `query_candidates`). O prompt e a mensagem em `src/app/actions.rs` (`set_query`,
  `start_content`, `finish_content`).
- **`contains('x')`:** lê os arquivos (`ContentJob` em `query.rs`), em passos no
  `tick`, com pergunta antes, progresso no rodapé (`src/ui.rs`) e Esc.
- **`words('a b*')`:** palavras inteiras e prefixos, sem maiúsculas nem acentos
  (`words_of`, `word_pats`, `WordSplitter`, `scan_words`).
- **O índice:** `src/index.rs` (tantivy, feature `index`), `Alt+N` no ramo, `Lookup`
  responde por arquivo (`Known::Terms` / `Known::Binary`), `IndexJob` indexa em outra
  thread.
- **O mini-SQL:** `src/viewer/sql_filter.rs`; `Expr::Call` só com
  `SqlFilter::parse_with_calls`.
- **Manual:** `docs/manual/filespec.md` (inglês, a referência) e
  `docs/manual/pt/filespec.md`; as linhas de `W` e `Alt+N` em `tree.md`.

### Regras do projeto (do `AGENTS.md`, valem para cada passo)

1. Ao fim de **cada** passo, os três precisam passar:
   - `cargo fmt --check`
   - `cargo clippy --all-targets -- -D warnings`, e também com
     `--no-default-features` (o Xylem compila sem a tantivy)
   - `cargo test`
2. **Segurança primeiro:** todo arquivo de fora é hostil. Limite de tamanho, de tempo
   e de memória em toda leitura; nada de `unwrap` em dado vindo de arquivo; texto de
   arquivo nunca vai para a tela sem passar pelo saneamento (sem controles nem bidi).
3. **Temporários só em `~/.cache/xt-dev/`,** nunca em `/tmp`. Os testes usam
   `crate::testing::tempdir()`; os do índice também
   `crate::index::TEST_BASE.with_borrow_mut(...)`, para não escrever no cache de
   verdade.
4. **Commit e push só com autorização explícita do autor.** Trabalhar numa branch nova
   a partir do `main` (sugestão: `query-next`).
5. **Planos em português** em `docs/plans/`; **manual em inglês primeiro** em
   `docs/manual/`; as outras línguas só no fim (passo 6) e anotadas no `BACKLOG.md`
   enquanto não forem feitas.
6. **Estilo:** comentários e mensagens em inglês simples, frases curtas, como o resto
   do código. Mensagens de commit em minúsculas, frase descritiva em inglês.
7. Ao fim de cada passo, atualizar a seção "Andamento" de `docs/plans/query.md`.

### Teclas

- `W`, `Ctrl+W` e `Alt+N` são provisórias; não trocar sem o autor pedir.
- **Não usar:** `Alt+W` e `Alt+Y` (teclas antigas do Usage e do Duplicates), `Alt+A`
  e `Alt+T` (proibidas pelo teste `alt_is_not_a_copy_of_ctrl`, como no ZTreeWin),
  `Ctrl+H`, `Ctrl+J` e `Ctrl+M` (o terminal as confunde com Backspace e Enter),
  `Ctrl+Z` (Oops).
- Toda tecla nova entra em `COMMANDS` e `DEFAULT_KEYS` (`src/keymap.rs`), no rodapé
  (`src/ui.rs`, listas `*_WORDS` e as dicas) e no manual; os testes do `keymap.rs`
  conferem conflitos.

## Os passos, nesta ordem

### Passo 1: medir um ramo grande

**Por quê:** é o maior risco do que já está no `main`. A cada consulta com `words()`,
`index::imp::stamps` lê **todos** os documentos do índice para saber tamanho e data de
cada arquivo. Num ramo de centenas de milhares de arquivos isso pode levar segundos e
muita memória.

**O que fazer:**

1. Gerar em `~/.cache/xt-dev/bigtree/` uma árvore sintética: 200.000 arquivos de texto
   pequenos (alguns KB, palavras variadas), em 2.000 pastas. Um script Python basta.
2. Medir com o binário release (`cargo build --release`), com `HOME` isolado em
   `~/.cache/xt-dev/mem/home` (para o índice não ir para o cache de verdade):
   - tempo e memória (RSS de pico, `/proc/<pid>/status`) para logar a árvore (`*`);
   - tempo e tamanho do índice no Alt+N;
   - tempo e memória de `W` com `words('alguma palavra')` em Branch, com o índice em
     dia (não deve perguntar nada);
   - o mesmo com `contains('x')` sem índice (só o tempo até a pergunta).
   Um script em `~/.cache/xt-dev/mem/` que dirige o `xy` por um pseudo-terminal já
   existe (`drive.py`, `idx.py`); pode ser reaproveitado.
3. **Se a consulta com índice passar de ~1 s ou de ~200 MB a mais,** otimizar:
   - guardar o `Lookup` de cada índice entre consultas, refeito só quando o índice
     mudar (a tantivy diz a geração do `Searcher`), ou
   - em vez de ler todos os documentos, perguntar ao índice só pelos caminhos dos
     candidatos (o campo `path` é `STRING`, dá para buscar o termo exato).
   A resposta tem de continuar a mesma: um arquivo com outro tamanho ou data é lido de
   novo.
4. **Registrar os números** na seção "Performance" de `query.md`.

**Pronto quando:** os números estão no plano e, se houve otimização, os testes do
índice continuam passando e há um teste novo para ela.

### Passo 2: as notas da versão

- Ver com `git tag` se a 0.8.0 já foi lançada. Se não, acrescentar em
  `.github/release-notes/v0.8.0.md`; se sim, criar `v0.9.0.md` no mesmo formato.
- Escrever, para quem usa: W e Ctrl+W, `contains()`, `words()` com prefixo, o índice
  com Alt+N (o que ele guarda, onde, como apagar), e que as teclas são provisórias.
  Curto, com um ou dois exemplos de condição.

**Pronto quando:** o texto segue o formato das notas anteriores.

### Passo 3: a lista do que o índice deixa de fora, configurável

- Hoje é fixa em `src/index.rs` (`LEFT_OUT_DIRS`, `LEFT_OUT_FILES`).
- Criar a opção `index_ignore` no `config.toml`, como `duplicates_ignore`
  (`src/config.rs`): o valor padrão é a lista de hoje; o usuário pode tirar e pôr.
- Mostrar a opção na tela de configuração (F12), como as outras listas.
- **Cuidado:** os padrões de segredo (`.ssh`, `.gnupg`, `.env`, `*.key`, `*.pem`,
  `id_rsa`, `id_ed25519`) ficam no valor padrão; o manual avisa que tirá-los copia
  segredos para o índice.
- Teste: um ramo com um arquivo ignorado pela configuração não entra no índice.

**Pronto quando:** a opção existe, está no manual (`configuration.md` e a seção do
índice em `filespec.md`) e tem teste.

### Passo 4: as colunas `git`, `type` e `dup`

- `type`: o tipo que o Alt+F mostra (`src/filetype.rs`). Lê o começo do arquivo: tratar
  como o `contains`, ou seja, só quando a condição precisa (cabe no `decide` de três
  valores, como uma função que lê).
- `git`: o status do Git do arquivo, quando o Git está ligado para aquela pasta (ver
  `src/git.rs`); `NULL` fora de repositório.
- `dup`: o número do grupo de duplicados, se o Alt+D já calculou; `NULL` se não.
- Cada uma entra em `COLUMNS`, no `RowContext` de `Row` e na tabela do manual.

**Pronto quando:** as três têm teste e estão no manual. Se alguma ficar complicada
demais, parar nela e anotar o porquê no plano.

### Passo 5: a caixa de busca (etapa 5 do `query.md`)

A experiência do Google Desktop: uma tecla, uma caixa, digitar e ver os resultados
chegando, **sem SQL.**

- **Tecla provisória:** `P` para os arquivos da lista e `Ctrl+P` para os marcados
  (as duas estão livres; conferir com os testes do `keymap.rs`).
- **O que procura:** o texto digitado vira `words('...')` (com prefixo na última
  palavra enquanto se digita: `fatura mar` procura `fatura mar*`), sobre os arquivos
  logados da lista (regra 1 do autor em `query.md`).
- **Resultados enquanto digita:** cada tecla recomeça a busca (com um pequeno atraso,
  ~150 ms, para não buscar a cada letra). Primeiro o que o índice responde, na hora;
  depois, se houver arquivos sem índice, **só com confirmação** (regra 3: dizer com
  todas as letras o que tem índice e o que será lido).
- **Cada linha:** o nome do arquivo, a pasta, a data e **um trecho** da primeira linha
  onde as palavras aparecem (até ~80 caracteres, saneado: sem controles nem bidi).
  O trecho exige ler o arquivo; ler só os resultados visíveis, com limite de tamanho
  e tempo.
- **Teclas na caixa:** ↑↓ andam, Enter vai ao arquivo na lista (e fecha a caixa),
  `Ctrl+V` (ou outra a escolher) abre no viewer na linha do trecho, `T` marca o
  arquivo, Esc fecha (e para uma leitura em andamento).
- **Reaproveitar:** a lista do histórico dos prompts (`HistoryList`, em `src/input.rs`)
  e o desenho dela na tela; o `ContentJob` e o `Lookup` para a busca.
- **Testes:** no nível do app (`src/app/tests.rs`), como
  `alt_n_indexes_the_branch_and_w_uses_it`: indexar, abrir a caixa, digitar, conferir
  os resultados, Enter, `T`.
- **Manual:** uma seção nova em `filespec.md` (ou uma página `search-box.md` ligada do
  `index.md` e do `help.rs`), e as linhas de `P` em `tree.md` e `files.md`.

**Pronto quando:** a caixa funciona com índice e sem índice, os testes passam e o
manual em inglês explica.

### Passo 6: as traduções (por último)

- As seções do W, `contains`, `words`, Alt+N (e, se feitos, da caixa de busca e do
  `index_ignore`) em `es`, `fr`, `de` e `zh`, em `docs/manual/<língua>/filespec.md` e
  `tree.md`. O português já tem as três primeiras.
- Tirar do `BACKLOG.md` ("Translations behind English") o que ficou em dia.

## Fora deste roteiro

- A etapa 3 (`SELECT`, `GROUP BY`, `JOIN` pelo SQLite): depende de o autor ver as
  outras em uso.
- Os metadados de dentro dos arquivos e os leitores à parte
  ([query-metadata.md](query-metadata.md)): só estudo.
- Trocar as teclas provisórias.

## Como a revisão vai ser feita na volta

1. Ler o diff inteiro de cada passo contra o `main`.
2. Rodar os três comandos do `AGENTS.md`, com e sem `--no-default-features`, e o
   clippy para o alvo Windows (`--target x86_64-pc-windows-gnu`).
3. Conferir no app real, por um pseudo-terminal, com `HOME` isolado: os números do
   passo 1, a caixa de busca do passo 5, e que nada novo escreve fora de
   `~/.cache/xylem` e `~/.cache/xt-dev`.
4. Procurar especialmente: leitura sem limite, `unwrap` sobre dado de arquivo, texto
   de arquivo sem saneamento na tela, teste que escreve no cache de verdade, resposta
   do índice que poderia ser diferente da leitura direta.
5. Conferir que a seção "Andamento" do `query.md` diz o que foi feito, o que ficou de
   fora e o que não foi verificado.
