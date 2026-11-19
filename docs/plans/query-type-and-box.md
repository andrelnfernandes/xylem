# Roteiro: a coluna `type` sem ler na tela, e a caixa de busca rápida

Origem: conversa de 2026-11-15, depois da revisão da branch `query-next` (PR #119).
O autor vai fazer este trabalho com outro assistente (Gemini) e depois pedir uma
revisão. Este documento é o briefing: lê-se inteiro antes de começar.

**Estado: roteiro aprovado pelo autor para execução.** Ele não muda nenhuma decisão de
[query.md](query.md); só corrige dois pontos que a revisão deixou em aberto (seção
"Andamento", "Revisão da etapa 5 e das colunas", item "Ainda em aberto").

## Regras (do `AGENTS.md`, para cada passo)

1. Ao fim de cada passo, os três passam:
   - `cargo fmt --check`
   - `cargo clippy --all-targets -- -D warnings`, também com `--no-default-features`
     e com `--target x86_64-pc-windows-gnu`
   - `cargo test`
2. **Segurança primeiro:** todo arquivo é hostil. Toda leitura tem limite de bytes e
   de tempo; nada de `unwrap` em dado de arquivo; texto de arquivo só vai para a tela
   saneado.
3. **A tela nunca espera o disco, a rede ou uma leitura grande.** Este roteiro existe
   por causa desta regra.
4. Temporários só em `~/.cache/xt-dev/`. Testes com `crate::testing::tempdir()` e, os
   do índice, `crate::index::TEST_BASE`.
5. Branch nova a partir do `main` (sugestão: `query-type-box`). **Commit e push só com
   autorização do autor.**
6. Manual em inglês primeiro (`docs/manual/filespec.md`); as outras cinco línguas no
   fim, como as que o Gemini já fez.
7. **Todo teste de correção precisa falhar no código de antes.** Rodar o teste novo
   contra o `main` antes de corrigir, e anotar no plano que falhou.
8. **Os testes não podem ter atalho que o app não tem.** A caixa já teve um
   `#[cfg(test)]` que buscava na hora de digitar e escondia que, no app, ela nunca
   buscava. Teste de app conduz como o laço principal: `while app.busy() { app.tick(..) }`.
9. Ao fim de cada passo, atualizar a seção "Andamento" de `query.md`.

## Parte A: a coluna `type`

### O problema

`type` é o que o Alt+F mostra (`src/filetype.rs`), e descobrir isso lê o começo do
arquivo (`HEAD_BYTES`). Hoje:

- **Na lista:** `Pane::query_takes` (`src/pane.rs`) avalia a condição em
  `refresh_files` e em `branch_matching_files` com um `lazy_type` que chama
  `describe_file`. Isso **lê arquivos na thread da tela**, a cada atualização da
  lista: numa máquina remota pela rede, numa montagem de nuvem com um download por
  arquivo.
- **O cache de tipos** (`Pane::types`) é zerado ao passar de 10.000 entradas: numa
  lista grande, tudo é relido.
- **Um resto perigoso:** em `RowContext for Row` (`src/model/query.rs`, braço
  `"type"`), sem `lazy_type` o código lê o caminho **do disco local** com
  `filetype::describe(.., true)`, mesmo numa janela remota.

### A solução: `type` é como o `contains`

Uma condição com `type` passa pelo mesmo caminho do `contains`: pergunta antes, lê em
passos no `tick`, Esc para, recusada na nuvem, avisa o download por SSH. A lista nunca
lê.

1. **`FileQuery` sabe que usa `type`:** um `uses_type()` como o `uses_git()`, e
   `reads_contents()` passa a ser `!terms.is_empty() || uses_type()`. Assim
   `set_query` (`src/app/actions.rs`) manda a condição para `start_content`.
2. **Três valores para o `type`:** em `decide` (`query.rs`), um nó que não é
   `AND`/`OR`/`NOT` e que lê a coluna `type` vale `None` (não sei) enquanto o tipo
   do arquivo não foi lido. Uma função `expr_uses(expr, "type")` sobre o
   `columns_in` resolve. Com isso, `ext = 'jpg' AND type LIKE '%PNG%'` só lê os
   `.jpg`, como já acontece com o `contains`.
3. **O `Row` deixa de ler:** o braço `"type"` só devolve o tipo já conhecido (um campo
   `known_type: Option<&str>` no `QueryContext`, no lugar do `lazy_type`); sem ele,
   `None`. Some a leitura do disco local. O `lazy_type` sai do `QueryContext`.
4. **O `ContentJob` lê o tipo como um termo a mais:** o `Item` ganha
   `type_name: Option<String>`; em `open_next`, se a condição usa `type` e ele não é
   conhecido, o primeiro passo é `self.type_of(&path, &file)` (o `TypeOf` que a janela
   já fornece, `Pane::type_of`), e depois `decide` de novo. Se isso basta, o arquivo
   nem é aberto para os termos. Um arquivo de cada vez por passo, dentro do orçamento.
5. **A pergunta diz o que vai ler:** com só `type`, "Read the start of 1,234 files to
   tell their type?"; com termos também, a pergunta de hoje. Os bytes: a soma de
   `min(tamanho, HEAD_BYTES)` para o `type`.
6. **A lista usa o resultado guardado** (`FileQuery::hits`), como com o `contains`:
   `query_takes` não chama nada que leia. Remover o `lazy_type` de `query_takes`.
7. **O índice não guarda o tipo** por enquanto: os arquivos são lidos. (Guardar o tipo
   no índice fica anotado no plano, não é deste roteiro.)
8. **O cache de 10.000 do Alt+F** fica como está: ele é só do Alt+F agora. Anotar no
   plano.

### Testes da Parte A

- **Unidade, `query.rs`:**
  - `decide` com `type = 'x'` e tipo desconhecido dá `None`; com `size > 0 OR type =
    'x'` e `size > 0` verdadeiro dá `Some(true)` sem tipo.
  - um `ContentJob` com `ext = 'jpg' AND type LIKE '%PNG%'` sobre um `.jpg`, um `.png`
    e um `.txt` chama o `TypeOf` **só para o `.jpg`** (um `TypeOf` que conta as
    chamadas).
- **App, `src/app/tests.rs`:**
  - `W` com `type LIKE '%...%'` **pergunta antes** (o `confirm` existe) e, antes do
    "y", nada foi lido: um `TypeOf` ou `Opener` que conta mostra zero. Depois do "y",
    conduzido por `busy()`/`tick`, a lista tem o arquivo certo.
  - `refresh_files` com uma condição de `type` já aplicada não lê nada (o mesmo
    contador continua igual depois de um `refresh_files`).
  - Numa montagem de nuvem, recusado com mensagem. Os testes simulam nuvem com o
    `always_slow` da árvore (`src/model/tree.rs`, "In tests, a folder of the disk
    behaves as a cloud mount"); ver como os testes de `cloud_log` o ligam.
- O teste `w_filters_by_git_type_and_dup_columns` (do Gemini) precisa passar a
  conduzir pela pergunta.

### Manual da Parte A

Em `filespec.md`, na tabela de colunas: `type` lê o começo do arquivo, por isso
pergunta antes, como o `contains`, e não funciona em montagem de nuvem.

## Parte B: a caixa de busca rápida

### O problema

Em `SearchBox::update_search` (`src/search_box.rs`), a cada pausa na digitação, tudo
é refeito **na thread da tela**:

1. `pane.query_candidates` duas vezes (uma para o `ContentJob`, outra para o mapa
   dos resultados): anda a árvore inteira, junta caminhos, procura Git e duplicados e
   clona cada `FileEntry`;
2. `index::lookup`: abre cada índice e **lê todos os documentos** (`stamps`) para
   saber tamanho e data de cada arquivo, e só então busca os termos;
3. `ContentJob::new` sobre todos os candidatos;
4. `ensure_snippets`: até ~10 arquivos lidos, cada um até 1 MiB.

Pela medição do passo 1 de [query-next.md](query-next.md), só o `lookup` leva ~0,6 s
com 200.000 arquivos. A tela trava a cada pausa.

### A solução

1. **Os candidatos uma vez por abertura da caixa.** Guardar no `SearchBox` os
   candidatos e um mapa caminho → índice no vetor, montados ao abrir (ou na primeira
   busca), e refeitos só quando a árvore mudar (`pane.tree.version()`, que já existe)
   ou o `tagged` mudar. Nada de segunda chamada a `query_candidates`; o
   `step_unindexed` usa o mesmo mapa.
2. **Separar no índice o que muda por tecla do que não muda** (`src/index.rs`):
   - `Stamps` (caminho → tamanho e data, e os binários): carregado uma vez e guardado
     no `SearchBox`; recarregado quando um Alt+N termina (o `finish_index` em
     `actions.rs` pode limpar o cache da caixa) ou quando a geração do índice mudar.
   - a busca dos termos (`TermQuery` / `PhrasePrefixQuery`): por tecla. É rápida.
   - Manter `index::lookup` funcionando para o W (que roda uma vez), por cima das duas
     partes novas.
   - Mesma resposta de antes: arquivo com outro tamanho ou data é lido; o índice nunca
     responde errado.
3. **Os trechos fora do caminho da tecla:** `ensure_snippets` passa a calcular no
   máximo um ou dois trechos por `tick` (orçamento de tempo e um teto de bytes por
   passada, por exemplo 2 MiB), e a linha mostra `…` até o trecho chegar. A caixa
   continua em `busy()` enquanto houver trecho visível por calcular
   (`SearchBox::wants_tick`).
4. **Se ainda passar da meta (abaixo), refinar a partir do resultado anterior:** quando
   o texto novo só acrescenta letras à última palavra ou acrescenta uma palavra, os
   resultados novos estão contidos nos anteriores (é um `AND` mais estreito). Filtrar
   os anteriores em vez de buscar tudo. Só se a medição pedir.
5. **Não mudar:** o que a caixa procura, o debounce de 150 ms, as teclas, a pergunta
   antes de ler o que não tem índice.

### Metas e medição da Parte B

- Gerar a árvore sintética de 200.000 arquivos em `~/.cache/xt-dev/bigtree/` com um
  script **guardado** em `~/.cache/xt-dev/bigtree-gen.py` (a da medição anterior foi
  apagada e os números não puderam ser reconferidos). Deixar a árvore lá.
- Medir com o binário release, por um pseudo-terminal, com `HOME` isolado
  (`~/.cache/xt-dev/mem/home`; há `drive.py`, `box2.py` e `idx.py` em
  `~/.cache/xt-dev/mem/` para reaproveitar). Para enviar teclas com Alt ou com
  Kitty, ver como esses scripts mandam `*` (`\x1b[42u`) e Alt+N (`\x1b[110;3u`).
- **Metas, com o índice em dia e 200.000 arquivos:**
  - da pausa na digitação até os resultados na tela: até **150 ms**;
  - nenhuma tecla esperando mais de **50 ms** para aparecer na caixa;
  - memória: até **100 MB** a mais que o Xylem sem a caixa aberta.
- Registrar antes e depois, com os mesmos passos, na seção "Performance" de
  `query.md`.

### Testes da Parte B

- **Unidade, `search_box.rs` e `index.rs`:**
  - duas buscas seguidas com o mesmo índice carregam os `Stamps` **uma vez** (um
    contador `#[cfg(test)]` no `index.rs`, ou uma função que diga se veio do cache);
  - depois de um Alt+N (ou de limpar o cache), carrega de novo e acha o arquivo novo;
  - os resultados são os mesmos que o `index::lookup` daria (comparar os dois).
- **App:** um teste conduzido por `busy()`/`tick` que digita, confere resultados e
  trechos (que chegam em ticks seguintes), muda um arquivo (`rescan_everywhere`) e vê
  o resultado mudar, como o `alt_n_indexes_the_branch_and_w_uses_it`.

## Ordem

1. Parte A, com os testes que falham antes, e o manual em inglês.
2. A medição "antes" da Parte B.
3. Parte B, itens 1 a 3; medir; item 4 só se a meta não for alcançada.
4. As traduções do que mudou no manual.
5. "Andamento" do `query.md` em dia: o que foi feito, os números, o que ficou de fora.

## Fora deste roteiro

- A aspa no texto da caixa (`o'brien`): o mini-SQL não entende `''`. Pequeno, mas
  separado.
- As teclas provisórias.
- Guardar o tipo no índice.

## Como a revisão vai ser feita na volta

1. Ler o diff contra o `main`.
2. Rodar as validações, com e sem `--no-default-features`, e para Windows.
3. Conferir no app real, com `HOME` isolado, na árvore de 200.000 arquivos: os tempos
   da caixa e que um `W` com `type` pergunta antes de ler.
4. Procurar especialmente: qualquer leitura de arquivo que sobrou na thread da tela
   (`refresh_files`, `query_takes`, `update_search`), o cache que serve resposta velha
   depois de um Alt+N ou de um arquivo alterado, e teste com atalho que o app não tem.
