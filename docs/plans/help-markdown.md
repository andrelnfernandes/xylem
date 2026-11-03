# Análise: o help usar o mesmo parser de Markdown do viewer

Origem: conversa de 2026-11-02, a partir do item do BACKLOG "the help's small parser
could move to the same parser one day". **Só análise, nada decidido:** o autor pediu
para guardar e voltar ao assunto mais à frente.

## Fatos que pesam

- **O help não lê só texto embutido.** `help::topic_in` (`src/help.rs`) carrega antes
  `~/.config/xylem/manual/<língua>/<tópico>.md`, se existir. O parser do help, então,
  também lê arquivo de fora, e é o que o [plano do Markdown](markdown.md) descartou para
  arquivo hostil: quadrático (`inline()` monta o resto do texto a cada caractere) e
  recursivo.
- **Já diverge do GitHub.** `blocks()` só reconhece lista com `- `. Em
  `docs/manual/execute.md` (linha 25), a lista numerada `1.` `2.` `3.` vira um
  parágrafo só, emendado. Também não entende itálico, citação `>`, título `####`
  nem `\|` dentro de tabela.
- **A página man depende dele.** `xy --man` sai de `roff_blocks`, que usa os mesmos
  `Block` do help.
- **O renderizador do viewer** (`src/viewer/markdown.rs`) usa `pulldown-cmark` sem as
  features padrão, já é dependência do projeto, e foi feito para arquivo hostil: nunca
  segue link e sempre mostra o endereço real, mostra comentários HTML e o bloco YAML,
  tem limites de tamanho e de tempo e `panic_guard`.

## Prós de migrar

- O help mostra o que o GitHub mostra: com 6 línguas e tradutores, hoje um texto certo
  no GitHub pode sair quebrado no help sem ninguém notar (caso do `execute.md`).
- Sai o parser quadrático e recursivo, que também lê o manual customizado em
  `~/.config`. O risco é baixo (o arquivo é do usuário), mas fica sem exceção à regra
  de tratar arquivo de fora como hostil.
- Nenhuma dependência nova.
- Menos código: `blocks()`, `inline()` e `plain()`, umas 200 linhas.

## Contras

- O renderizador do viewer nunca segue link; o help precisa de links entre tópicos
  navegáveis com Enter e sem o endereço. Um modo "documento confiável" dentro do código
  de segurança abre espaço para um erro que torne navegável o link de um arquivo hostil.
- Outras escolhas do viewer também não servem ao help: mostrar comentários e YAML, os
  limites de tempo e tamanho. Cada uma pediria uma exceção.
- A página man teria de ser reescrita a partir dos eventos do `pulldown-cmark`, com mais
  casos (listas aninhadas, tabelas) e testes novos de saída.
- Risco de regressão: o help nas 6 línguas, o tópico "keys" gerado em tempo de
  execução, a ajuda de contexto, a navegação com Tab e Back entre links.
- Acoplamento: toda mudança de segurança no viewer passaria a mexer no help e na man.

## Meio-termo (recomendado)

Trocar só a leitura: o `pulldown-cmark` passa a produzir os mesmos `Block` e `Span` do
help (título, parágrafo, item, código, tabela, trechos). Ficam como estão a diagramação
do help (`layout`, `wrap`), os links entre tópicos e a página man.

- Ganha: o help lê Markdown como o GitHub, sem o parser quadrático; listas numeradas e
  o que mais faltar viram blocos que o help já sabe desenhar.
- Evita: misturar o help com o renderizador de arquivos hostis, e reescrever a man.
- Custo: um adaptador de eventos para blocos, mais testes sobre o manual inteiro nas 6
  línguas.

Junto, um teste que passa cada arquivo do manual pelo `pulldown-cmark` e falha se
aparecer algo que o help não entende.

**Alternativa mais barata:** só esse teste. Pega divergências como a lista numerada,
mas não resolve o parser quadrático lendo `~/.config`.
