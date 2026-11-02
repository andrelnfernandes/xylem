# Plano: próximos passos e ritmo de releases

Origem: conversa de 2026-11-01, depois de conferir backlog, planos e textos.
Aprovado pelo autor: **primeiro fechar o que foi começado e não terminado, depois
seguir a lista do que é novo**, com um release por semana, passando a um a cada 15
dias quando as mudanças ficarem mais raras.

Este arquivo diz a ordem e o ritmo. O detalhe de cada frente continua no plano dela,
e o [BACKLOG](../../BACKLOG.md) continua sendo a lista completa do que falta.

## Ritmo de releases

- **Semanal** por enquanto: sai o que estiver pronto no `main`. Um release nunca
  espera um trabalho pela metade; o que não ficou pronto vai no da semana seguinte.
- **Passa a cada 15 dias** quando dois releases seguidos trouxerem só correções ou
  mudanças pequenas. Volta a ser semanal se um ciclo grande recomeçar.
- **Semana sem nada novo para o usuário não tem release.**
- **Correção de segurança não espera:** sai na hora, como `0.x.1`.
- **O `main` fica sempre pronto para lançar:** trabalho que não cabe numa semana é
  fatiado em partes que funcionam sozinhas, ou fica em branch até funcionar.

### Roteiro de cada release

Segue o que foi feito na 0.5.0:

1. `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` e `cargo test`;
   em mudanças sensíveis a `cfg`, `clippy --target x86_64-pc-windows-gnu`.
2. Num PR de versão: `Cargo.toml` e `Cargo.lock`, os exemplos de versão nos READMEs e
   no site, e as notas em `.github/release-notes/<tag>.md`, em linguagem de usuário.
3. Conferir site, READMEs, manual (6 línguas) e BACKLOG contra o que entra.
4. **Com autorização do autor:** merge e push da tag. O `release.yml` confere se a tag
   bate com o `Cargo.toml`, compila, publica no GitHub e no crates.io.
5. Conferir a página do release e instalar um dos pacotes.

O site é publicado sozinho quando muda no `main`, então texto que anuncia recurso novo
entra junto com o release, nunca antes.

## Parte 1: fechar o que está aberto

As semanas são estimativas e contam a partir da semana de 2026-11-02. Cada frente com
mais de uma semana ganha spec próprio antes de começar. O tamanho das fatias de SSH e
do viewer ainda será conferido nesses specs.

| Semana | Release | O que entra | Plano |
|---|---|---|---|
| 1 | 0.6.0 | **TAR comprimido:** listar uma vez, em segundo plano e com cancelamento, e reaproveitar a lista na cópia. Mais a branch `docs-catch-up-with-features` e o que já está no `main`: Batch, Markdown, Duplicados, leitura segura do Git | BACKLOG, Security 1 |
| 2 | 0.7.0 | **Ações na Usage (fase 3):** marcar, apagar, Prune, viewer | [usage.md](usage.md) |
| 3 | 0.8.0 | **Nuvem:** a raiz de um Drive grande sem tela em branco; Branch, Usage e Compare pedindo o ramo de uma vez ao rclone. **Segurança:** busca em entrada enorme de archive, leitores de tabela com limites e Esc, e o `.xlsx` minúsculo que derrubava o programa | [search-and-table-limits.md](search-and-table-limits.md) |
| 4 | | **SSH 3a:** camada de sistema de arquivos sob `ops`, com mkdir e renomear no remoto | BACKLOG, Features |
| 5 | | **SSH 3b:** enviar para o remoto (copiar) | |
| 6 | | **SSH 3c:** apagar e mover no remoto | |
| 7 | | **SSH 3d:** permissões e datas; Duplicados por SSH passa a fazer sentido | |
| 8 | | **SSH 4a:** Edit e Open por cópia temporária que volta ao salvar | |
| 9 | | **SSH 4b:** atualização periódica no lugar do inotify; eXecute por ssh | |
| 10 | | **SSH 4c:** Git por ssh | |
| 11 a 13 | | **Viewer em janelas (subprojeto 3):** índice de linhas em segundo plano, leitura sob demanda, Esc cancela. É o coração do viewer: fica em branch até busca, quebra de linha, "ir para a linha" e cores funcionarem | [viewers.md](viewers.md) |
| 14 | | **Logs (subprojeto 4):** follow, ir para o fim, cor por nível, ANSI só SGR | [viewers.md](viewers.md) |

Decisões já tomadas:

- **Usage, fase 3:** `D` continua abrindo Duplicados; apagar fica em Del e Shift+Del.
- **Usage, fase 4:** o treemap sai. Agrupar por tipo e exportar ficam sem data.
- **Escrita remota trata o servidor como hostil,** como já faz a leitura: cada spec
  dessa frente traz o seu modelo de ameaça.

## Parte 2: o que é novo

Depois da Parte 1, nesta ordem:

1. **JFC:** comparar o conteúdo de dois arquivos, sem depender de `diff %t` no eXecute.
2. **Salvar e recarregar o log (Zlist):** a árvore de um disco ou Drive abre na hora e é
   atualizada depois. O arquivo salvo também é lido como hostil.

## Em paralelo: divulgação

Não concorre com as frentes acima, e cada item é pequeno:

- **Um GIF curto por recurso:** Batch, SQLite, formatos, Markdown e duplicados estão no
  site, gravados por `docs/screenshots/tools/run.sh`. Falta duplicados no Drive sem
  baixar nada.
- **Onde as pessoas instalam:** AUR, Homebrew e nixpkgs depois da 0.6.0; repositório
  apt, COPR ou PPA para atualizar sozinho ([packages.md](packages.md)).
- **macOS:** reavaliar quando houver uma máquina para testar.

## Sem data

- **Cockpit de programação** ([cockpit.md](cockpit.md)): rascunho. Primeiro busca no
  projeto, lista de locais e editor na linha certa. Fica para um release depois da 0.6.0;
  a ordem em relação às frentes acima ainda será decidida.
- **Sobras de segurança que dependem do Windows:** eXecute (item 2, que também libera
  registros em `.bat`/`.cmd`) e senha do ssh (item 3). Windows não é prioridade.
- **O que os leitores de tabela ainda não limitam (item 4):** `.ods` montado inteiro e
  Parquet com listas; depende do calamine e da crate `parquet`
  ([search-and-table-limits.md](search-and-table-limits.md)).
- **SQLite e zstd em Rust (item 5):** acompanhar; depende de terceiros.
- **Pequenos, ainda não aprovados:** Enter no Graph abre o `git show`
  ([git-graph.md](git-graph.md)); `+` sem logar os arquivos das subpastas
  ([release-branch.md](release-branch.md)); Tags-Only fora do Showall
  ([showall-tagged.md](showall-tagged.md)).
- **Extras:** SDF no modo tabela; pastas inteiras nos duplicados da nuvem; o parser do
  help no `pulldown-cmark`; Parquet e XLSX como saída do Batch.

## Fora, por decisão

Conversores externos (o mais arriscado, e `O` e `X` já cobrem o uso), Xylem em janela
própria, HTML no viewer, treemap, mouse e o resto da lista do ZTreeWin que serve mais à
paridade do que ao uso (Wash, Volume, Hide, comandos de vídeo).
