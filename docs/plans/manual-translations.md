# Plano: traduzir o manual novo (para executar com o Gemini)

Origem: conversa de 2026-11-15. O manual em inglês foi refeito na branch
`manual-and-site` (ver `docs/plans/manual-and-site.md`). Falta traduzir para
`pt`, `es`, `fr`, `de` e `zh`. Este plano é para outro assistente executar sem o
contexto da conversa.

## Regras do projeto (valem para tudo)

- Trabalhar na branch `manual-and-site`. **Não fazer commit nem push**: só o autor
  autoriza.
- Arquivos temporários só em `~/.cache/xt-dev/`, nunca em `/tmp`.
- Antes de entregar, têm de passar: `cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings` e `cargo test`.
- Ler `docs/manual/TRANSLATING.md` antes de começar.

## O que traduzir

O inglês em `docs/manual/*.md` é o texto canônico. Cada tradução vai em
`docs/manual/<lang>/<mesmo-nome>.md`.

**1. Páginas novas (21), que não existem em nenhuma língua:**

`overview`, `start`, `screen`, `menus`, `glossary`, `commands`, `commands-ctrl`,
`commands-alt`, `commands-keys`, `cli`, `terminals`, `security`, `releases`,
`howto-copy-with-paths`, `howto-rename-mask`, `howto-sql-filter`,
`howto-duplicates`, `howto-disk-space`, `howto-table-sql`, `howto-ssh-copy`,
`howto-terminal-tagged`.

**2. `index.md`, reescrito.** Traduzir de novo a partir do inglês, com as mesmas
seções e a mesma ordem. Os índices traduzidos têm hoje, no fim, uma seção
provisória ("Páginas novas (ainda em inglês)" e equivalentes) que **sai** quando o
índice for refeito.

**3. Tópicos com trechos alterados (9).** Só o trecho mudou; o resto da tradução
existente fica. Para ver o que mudou em cada um:

```bash
git diff main -- docs/manual/cloud.md docs/manual/duplicates.md docs/manual/execute.md docs/manual/filespec.md docs/manual/git.md docs/manual/tagging.md docs/manual/terminal.md docs/manual/treespec.md docs/manual/usage.md
```

Em `tagging.md`, as seções "What Ctrl and Alt mean" e "Ctrl and Alt menus (F9)"
viraram uma seção curta com links para `menus.md`, `commands-ctrl.md`,
`commands-alt.md` e `releases.md`. Manter o título da primeira seção como está em
cada língua: outros arquivos apontam para a âncora dele (por exemplo
`README.pt-BR.md` aponta para `pt/tagging.md#o-que-ctrl-e-alt-querem-dizer`).

**4. Site em português:** `site/pages/pt/index.md`. Trocar o parágrafo de abertura
e o bloco de cartões pelos de `site/pages/en/index.md`, traduzidos, mantendo o HTML
e as classes. As outras línguas usam as páginas inglesas do site.

**5. `README.pt-BR.md`:** tirar a frase "(as páginas novas ainda estão em inglês)"
da seção Documentação e apontar os links dela para `docs/manual/pt/`.

## Como traduzir

- **Nomes de arquivo e destinos de link não mudam.** Traduz-se o rótulo:
  `[A tela](screen.md)`. Âncoras como `filespec.md#a-sql-condition-w` apontam para
  o título **na língua da página**: conferir o título traduzido em
  `<lang>/filespec.md` e usar a âncora dele.
- **Primeira linha:** `# Título traduzido`.
- **Não traduzir:** teclas (`Ctrl+C`, `Alt+F5`, `Enter`, `Esc`, `Del`), nomes de
  comandos como aparecem no menu (Copy, Move, Branch, Showall, Filespec, Treespec,
  eXecute, iNdex, Prune, Oops, Yank, Kbyte, New date), títulos de menu (DIR, FILE,
  CTRL, ALT, GIT, ARCHIVE, REMOTE), mensagens do programa entre crases
  (`Not logged`, `Copied 21 of 21 files`, `No tagged files`), nomes de opções
  (`delete_to_trash`), blocos de código e condições SQL.
- **Nas tabelas de `commands*.md`,** a coluna "Command" fica em inglês. As colunas
  "Tree" e "Files" podem ter o cabeçalho traduzido; o `yes` das células pode ser
  traduzido.
- **Imagens:** manter a linha `![...](../screenshots/arquivo)` com o mesmo arquivo
  e traduzir o texto alternativo. Atenção ao caminho: dentro de
  `docs/manual/<lang>/` as traduções existentes usam `../../screenshots/`. Conferir
  em `docs/manual/pt/remote.md` e seguir o mesmo.
- **Vocabulário:** usar os termos que as traduções existentes já usam. Conferir em
  `<lang>/tree.md`, `<lang>/tagging.md` e `<lang>/files.md` como foram traduzidos
  log, tag, branch, tree, file window, prompt, footer. `glossary.md` tem de
  concordar com eles.
- **Estilo:** o do inglês. Frases que dizem o que acontece, quando e o que custa.
  Sem adjetivos de propaganda ("poderoso", "simples", "basta"), sem frase de fecho
  no fim da seção, sem "vale notar que". Não acrescentar nem tirar informação.
- **Subconjunto de Markdown** (o F1 só entende isto): `#`, `##`, `###`, parágrafos,
  listas com `- `, listas numeradas `1. `, tabelas com `|`, blocos de código com
  três crases, `` `código` ``, `**negrito**` e `[rótulo](topico.md)`. Sem HTML, sem
  `>` de citação, sem itálico.

## Ordem

1. `pt` inteiro (o autor lê português e revisa primeiro).
2. Rodar as validações abaixo.
3. `es`, `fr`, `de`, `zh`, uma língua de cada vez, validando a cada uma.
4. Por último, o site em português e o `README.pt-BR.md`.

Em cada língua, começar por `glossary.md` e `menus.md`, que fixam os termos, depois
`index.md`, depois o resto.

## Validação

```bash
cargo test --bins help::
```

Confere, para todas as línguas: título na primeira linha, links para tópicos que
existem, o índice citando todos os tópicos, e a diagramação de 20 a 200 colunas.

```bash
cargo fmt --check
```

```bash
cargo clippy --all-targets -- -D warnings
```

```bash
cargo test
```

```bash
sh site/build.sh ~/.cache/xt-dev/site/_site
```

O site tem de construir sem avisos de link quebrado. Para olhar:
`python3 -m http.server -d ~/.cache/xt-dev/site/_site 8765`.

Conferência à mão, por língua: abrir `xy`, F1, `L` até a língua, e percorrer o
índice. Nenhuma página nova deve aparecer em inglês.

## Pronto quando

- As 21 páginas novas e o `index.md` existem nas cinco línguas.
- Os nove tópicos alterados têm o trecho novo nas cinco línguas.
- A seção provisória saiu dos cinco índices.
- As validações passam e o site constrói.
- Nada foi commitado; o autor revisa e decide.

## Fora deste plano

- As duas receitas que faltam (Batch e rclone) e as capturas dos menus: quando
  forem escritas em inglês, entram numa segunda rodada de tradução.
- As seis decisões sobre os menus (`docs/plans/manual-and-site.md`): se mudarem o
  aplicativo, `menus.md` e `commands*.md` mudam em inglês e nas traduções.
