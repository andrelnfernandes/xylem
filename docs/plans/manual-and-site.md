# Plano: manual e site novos

Origem: conversa de 2026-11-15. O autor quer refazer o manual e o site para a próxima
versão, mantendo o F1 que abre a ajuda do que está em uso. Branch `manual-and-site`.
**Estrutura e estilo aprovados** pelo autor em 2026-11-15, com as amostras de
`docs/plans/manual-and-site-samples/`. As seis perguntas dos menus continuam em
aberto. O andamento está na seção "Andamento", no fim.

## O que existe hoje

- **Manual:** 32 tópicos em `docs/manual/*.md` (inglês, canônico) e as traduções em
  `pt`, `es`, `fr`, `de`, `zh`. São compilados no executável (`TOPICS`, `src/help.rs`)
  e viram o F1, o `xy --man` e a parte "Manual" do site.
- **F1 contextual:** `src/app/context_help.rs` escolhe o tópico pelo nome do arquivo
  (`"copy-move"`, `"filespec"`, …) conforme o prompt, a pergunta ou a janela em uso.
- **Site:** um livro mdBook por língua (`site/book.py`, `site/build.sh`); sete páginas
  em `site/pages/{en,pt}/` e depois o manual. O menu lateral é gerado dos grupos do
  `index.md`.
- **Limites do F1:** só o subconjunto de Markdown de `TRANSLATING.md`. Linhas de
  imagem (`![...]`) são puladas no F1 e aparecem no site. HTML não é interpretado.
- **Testes:** todo tópico começa com `# Título`, todo link aponta para um tópico, o
  `index.md` cita todos, e cada tópico é diagramado de 20 a 200 colunas.

O manual atual é organizado por assunto (árvore, lista, marcas, cópia). Quem procura
"o que faz o Ctrl+R" precisa adivinhar o assunto. Não há página que apresente a tela,
os menus aparecem dentro de `tagging.md`, e não há receitas nem notas de versão.

## A estrutura proposta

A ordem abaixo é a do menu lateral do site e a do índice do F1.

| Parte | Página | Arquivo | Origem |
|---|---|---|---|
| Começo | O que é o Xylem | `index.md` | reescrito |
| | Instalar e os primeiros cinco minutos | `start.md` | novo (parte vem de `site/pages/install.md`) |
| | A tela | `screen.md` | novo |
| | Os menus e as teclas Ctrl e Alt | `menus.md` | novo (sai de `tagging.md`) |
| | Vocabulário | `glossary.md` | novo |
| Comandos | O menu DIR e o menu FILE | `commands.md` | novo |
| | O menu CTRL | `commands-ctrl.md` | novo |
| | O menu ALT | `commands-alt.md` | novo |
| | O menu GIT | `git.md` | existe |
| | Teclas de função e símbolos | `commands-keys.md` | novo |
| Assuntos | os tópicos atuais (cópia, filespec, visualizador, …) | os mesmos | revisados |
| Como fazer | uma receita por página | `howto-*.md` | novo |
| Referência | Configuração, linha de comando, terminais, segurança, ZTreeWin | vários | parte existe |
| Versões | Notas de versão | `releases.md` | novo |

### Por que as listas de comandos não substituem os tópicos

Copy aparece no menu FILE (`C`), no CTRL (Ctrl+C) e no ALT (Alt+C). Uma página por
comando por menu repetiria o destino, os caminhos e os conflitos três vezes, em seis
línguas. A proposta:

- **As páginas de comandos são tabelas na ordem do menu**, uma linha por palavra:
  a tecla, o que faz em uma frase, onde vale (árvore, lista ou as duas) e o link
  para o tópico que tem os detalhes.
- **Os tópicos por assunto continuam** com os mesmos nomes de arquivo. O F1
  contextual não muda, e os links de fora (`README`, issues) continuam valendo.
- Cada menu tem duas listas no aplicativo (árvore e lista de arquivos). A tabela
  junta as duas e a coluna "onde" diz a diferença, em vez de duas tabelas quase iguais.

### Um teste para as tabelas não envelhecerem

`src/ui.rs` já tem as listas (`DIR_WORDS`, `FILE_WORDS`, `CTRL_*_WORDS`,
`ALT_*_WORDS`, `git::MENU`). Um teste novo em `help::tests` lê `commands*.md` e falha
se uma palavra de um menu não tiver linha na página, ou se a ordem for outra. Comando
novo no menu sem linha no manual quebra o `cargo test`, como já acontece com tecla sem
menu (`docs/plans/footer-three-lines.md`).

## Cada página

**O que é o Xylem (`index.md`).** O que é (gerenciador de arquivos em modo texto, na
linha do XTreeGold e do ZTreeWin), o que quer dizer "log" em duas frases, o que se faz
com ele no dia a dia, e o que ele tem que os dois não têm: terminal embutido, SSH e
containers, rclone, pastas de rede, SQL sobre a lista de arquivos (`W`) e sobre
tabelas no visualizador, duplicados, uso do disco, Git. Depois o parágrafo de
segurança e o índice. A frase "nem no mercado" fica de fora: não há como conferir, e
a lista do que ele faz já diz o que interessa. A comparação conferível é com o
ZTreeWin, e está em `ztreewin.md`.

**Primeiros cinco minutos (`start.md`).** Instalar, `xy`, mover-se na árvore, entrar
na lista, marcar três arquivos, copiar, sair. Uma captura por passo.

**A tela (`screen.md`).** Uma captura com números e uma tabela: cabeçalho (nome,
versão, caminho, data), árvore, janela de arquivos pequena (com a árvore) e grande
(Enter), estatísticas à direita, rodapé com o menu. O que muda com F8 (duas
janelas), F7 (Autoview) e Alt+X (terminal). No F1 a imagem não aparece, então a
página também tem o esquema em texto num bloco de código.

**Os menus (`menus.md`).** A regra: letra sozinha age no item sob o cursor, Ctrl nos
marcados, Alt é a outra versão do comando. Como ler o rodapé (a letra maiúscula é a
tecla; `eXecute` é `X`). F9 e segurar a tecla. O que some dentro de um arquivo
compactado ou numa máquina remota. Ctrl+M e os terminais. A tabela "mudou na 0.8" vai
para `releases.md`.

**Vocabulário (`glossary.md`).** Log, release, branch, showall, tag, filespec,
treespec, prune, oops, autoview. Quem não veio do XTree não conhece essas palavras, e
hoje cada uma é explicada no meio de um tópico diferente.

**Comandos (`commands*.md`).** Como descrito acima. `commands-keys.md` tem F1 a F12,
`*`, `?`, `\`, Enter, Esc, Tab e a navegação, uma vez só, com a janela onde valem.

**Como fazer (`howto-*.md`).** Cada receita: o resultado em uma frase, os passos
numerados com a tecla em cada um, uma animação dos passos sendo executados (ver
"As animações das receitas"), e o link para os tópicos. Primeira leva (10):

1. Juntar arquivos de várias pastas e copiá-los mantendo os caminhos
2. Achar o que enche o disco e apagar
3. Achar duplicados e ficar com uma cópia
4. Renomear muitos arquivos com uma máscara
5. Filtrar a lista com SQL (`W`) e marcar o resultado
6. Consultar uma planilha, um CSV ou um SQLite com SQL no visualizador
7. Transformar uma tabela em script, JSON ou SQL (Batch)
8. Copiar entre esta máquina e outra por SSH
9. Abrir o Google Drive ou um S3 pelo rclone
10. Usar os arquivos marcados num comando do terminal embutido (Alt+O)

**Notas de versão (`releases.md`).** Uma seção por versão, da mais nova para a mais
antiga, a partir da 0.4.0 (as tags existentes). Em cada uma: novo, mudou (com a tecla
antiga e a nova), corrigido. Escritas a partir do `git log` entre as tags. Entram no
F1 também: é onde quem atualizou procura a tecla que mudou.

**Referência.** `configuration.md` (existe), `cli.md` (novo: as opções de
`xy --help`), `terminals.md` (novo: o que cada terminal entrega, F1, Alt+F10, Ctrl+M,
protocolo do kitty; hoje está espalhado), `security.md` (novo no manual: vem do
README), `ztreewin.md` e `windows.md` (existem).

## As animações das receitas

Pedido do autor (2026-11-15): nas receitas, um GIF com os passos sendo executados no
lugar da imagem parada, mostrando o número do passo em andamento. **Viável; protótipo
feito nesta branch.**

- **Como é gerado.** `docs/screenshots/tools/shot.py` já produz GIFs a partir de uma
  lista de teclas (os `demo-*.gif`). Ganhou a instrução `step:TECLAS`: a partir dali,
  uma faixa abaixo da tela mostra `4/7` e as teclas do passo. Foram cerca de 25
  linhas no `shot.py`, e cada receita é uma chamada `run howto-...` no `run.sh`.
- **O protótipo.** `docs/screenshots/howto-copy-paths.gif`, da receita de amostra:
  7 passos, 34 quadros, 32 segundos, 607 KB, 1104 x 816. O Xylem roda de verdade na
  árvore de exemplo; o último quadro mostra "Copied 21 of 21 files".
- **A faixa não tem palavras**, só o número e as teclas (decisão do autor,
  2026-11-15). Assim um GIF serve às seis línguas. O texto de cada passo fica na página, numerado igual.
- **No F1 a animação não aparece** (o help pula as imagens). Os passos numerados são
  o conteúdo; a animação é o complemento do site e do GitHub.
- **Tamanho.** Dez receitas dão cerca de 6 MB no repositório, a mais que os 4 MB dos
  GIFs atuais. Cada vez que são refeitas, o Git guarda outra cópia.
- **Limites.** Um GIF não pausa nem volta. Proponho no máximo 8 passos e 35 segundos
  por animação; receita maior é dividida em duas. Receitas que dependem de serviço
  externo precisam de um substituto: para SSH já existe (`fake_ssh.py`); para o
  rclone (Google Drive, S3) seria preciso escrever um, ou a receita fica com imagem
  parada. É o "quando possível".
- **Sugestão: a animação como teste da receita.** Uma instrução `expect:TEXTO` no
  `shot.py` que falha se o texto não estiver na tela (aqui, "Copied 21"). Se uma
  tecla mudar numa versão, o `run.sh` acusa a receita que deixou de valer.
- **Sugestão para depois:** no site, clicar na animação para pausar (precisa do
  primeiro quadro como PNG e de algumas linhas em `site.js`).

## O site

- Continua mdBook, um livro por língua, com `book.py` gerando o menu lateral do
  `index.md`. Não troco de ferramenta: a busca, o seletor de língua e os
  redirecionamentos já funcionam.
- **Páginas do site que ficam:** a abertura (`index.md` do site), Em ação, Telas,
  Em breve, Sobre.
- **Páginas que saem para o manual:** Instalar (vira `start.md`) e Recursos (vira a
  página "O que é"). Os endereços antigos ganham redirecionamento.
- **O menu lateral** segue as partes da tabela acima. `book.py` passa a aceitar um
  nível a mais, para "Como fazer" e "Comandos" abrirem e fecharem.
- **Capturas novas:** a tela numerada, uma por menu (DIR, FILE, CTRL, ALT, GIT) e as
  das receitas. Feitas com `docs/screenshots/tools/run.sh`, que já existe.
- **Abertura:** o texto atual usa frases de anúncio ("Your whole disk, one tree",
  "Make it yours"). Proponho manter o título e trocar os sete cartões por frases que
  dizem o que o programa faz, no mesmo tom do manual. Amostra em
  `manual-and-site-samples/site-home.md`.

## O estilo

A skill `user-guide-writing`, aplicada a tudo: cada frase diz o que acontece, quando
e o que custa. Sem "poderoso", "simples", "basta"; sem frase de fecho; sem lista de
três onde só dois itens são fatos. Verbos simples ("lê", "mostra", "apaga") no lugar
de metáforas. Mudanças de versão no formato "Mudou na versão X.Y".

Convenção de teclas, a mesma em todas as páginas: letra sozinha em código (`C`),
combinações por extenso (Ctrl+C, Alt+F5), nome do comando com a maiúscula do menu
(eXecute, iNdex).

## As perguntas do autor sobre os menus

São mudanças no aplicativo, não no manual. Cada uma é uma decisão separada; o manual
é escrito para o que for decidido.

1. **Git no menu normal?** Hoje é Ctrl+G, e a dica do próprio menu GIT diz "on this
   directory (or the tagged files)": ele não é um comando só dos marcados, então está
   fora da regra do CTRL. A letra `G` sozinha está livre nas duas janelas.
   **Recomendo** `G` no DIR e no FILE (quando a opção Git está ligada), com Ctrl+G
   mantido como apelido. Custo: no ZTreeWin `G` é Global; o Xylem já diverge no
   Ctrl+G, e isso está anotado em `ztreewin.md`.
2. **As dicas do CTRL** ("on tagged files here · Showall/Tag/Untag: all logged ·
   Yank: paths to clipboard"). A primeira parte diz o alcance e é o que o título
   "CTRL" não diz. As outras duas são exceções de comandos específicos, e são as
   primeiras a serem cortadas numa tela estreita. **Recomendo** deixar só o alcance
   ("on the tagged files of this directory" / "of this list") e levar as exceções
   para `commands-ctrl.md`, que o F1 abre com o menu CTRL na tela.
3. **As dicas do ALT** ("Compare/Duplicates/Prune/Release/Usage: the whole branch ·
   Log: … · iNdex: …"). Mesmo caso. **Recomendo** uma frase: "in the tree: the whole
   branch" / "in the file window: the tagged files, with their paths".
4. **F1, `*`, `?`, `\`, F3, Enter, F8, F9 em todos os menus?** **Não recomendo.** O
   menu CTRL aparece enquanto o Ctrl está seguro, e ali a tecla seria Ctrl+F3 ou
   Ctrl+F1, que não estão na tabela de teclas. O rodapé mostraria teclas que não
   funcionam naquele momento. No CTRL e no ALT ficam só as teclas que valem com o
   modificador, e o F9. O manual lista as teclas comuns uma vez, em `commands-keys.md`.
5. **"X terminal" e "O type tagged files in terminal" na linha das palavras?** Estão
   na linha das teclas porque nenhuma palavra do ALT tem X ou O como letra do menu.
   **Recomendo** subir os dois como palavras, `Onto terminal` e `eXec terminal`, e
   deixar a linha de baixo só para teclas de função. Os nomes são sugestão; o autor
   escolhe.
6. **F12 no menu ALT.** Pela tabela de teclas (`DEFAULT_KEYS`), F12 vale nas duas
   janelas sem modificador, e Alt+F12 não está na tabela. O menu ALT mostra "F12
   setup", então com o Alt seguro a tecla mostrada não é a que funciona. Não testei
   no terminal; a conclusão vem da leitura da tabela. **Recomendo** "F12 setup" na
   linha de teclas do DIR e do FILE, e "F10 setup" no ALT (Alt+F10 está na tabela).

## Etapas

1. **Aprovar a estrutura e o estilo** com as amostras desta branch.
2. **Decidir as seis perguntas dos menus.** O que for aprovado é feito antes do
   texto, com os testes de `ui.rs` e as capturas refeitas depois.
3. **Código do manual:** tópicos novos em `TOPICS`, o teste das tabelas de
   comandos, `context_help.rs` (F1 no menu CTRL, ALT ou GIT abre a página daquele
   menu; hoje abre a da janela), um nível a mais no menu lateral em `book.py`.
4. **Texto em inglês:** as páginas novas, depois a revisão dos 32 tópicos com a
   skill. Tópico por tópico, sem mudar nome de arquivo.
5. **Capturas** novas e as das receitas.
6. **Site:** abertura, páginas movidas, redirecionamentos.
7. **README** e `README.pt-BR.md`: apontar para as páginas novas e encurtar o que
   passou a estar no manual.
8. **Traduções** (`pt`, `es`, `fr`, `de`, `zh`) por último, de uma vez.
9. `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`,
   e `sh site/build.sh` para conferir o site.

## O que mais (sugestões)

- **Vocabulário** e **primeiros cinco minutos**, já na estrutura acima.
- **Terminais:** uma página só com o que cada terminal entrega. Hoje a explicação do
  Ctrl+M aparece em três tópicos.
- **Segurança** como página do manual, além do README.
- **Busca no F1:** o site tem busca e o F1 não. Com mais páginas, procurar uma
  palavra em todos os tópicos passa a fazer falta. Fica como item separado.
- **Página 404 e mapa do site** para os endereços que mudarem.

## Em aberto

- As seis perguntas dos menus.
- As notas de versão entram no F1 ou só no site?
- A primeira leva de receitas: as dez acima ou outras?
- `tagging.md` continua com esse nome depois que a parte dos menus sair?

## Andamento (2026-11-15)

Feito na branch, sem commit:

- **Páginas novas em inglês:** `overview`, `start`, `screen`, `menus`, `glossary`,
  `commands`, `commands-ctrl`, `commands-alt`, `commands-keys`, `cli`, `terminals`,
  `security`, `releases`, e o `index` reescrito. A página "O que é" ficou em
  `overview.md`: o `index.md` continua sendo o índice, porque o `xy --man` usa os
  parágrafos de abertura dele e o site gera o menu lateral das listas dele.
- **Receitas com animação (8):** copiar com caminhos, renomear com contador, condição
  SQL na lista, duplicados, uso do disco, SQL numa tabela, cópia por SSH, marcados no
  terminal. As animações somam 3,1 MB.
- **Código:**
  - passos numerados (`1. `) no Markdown do help (`Block::Step`), com teste;
  - teste `the_manual_lists_every_menu_word_in_order`: as tabelas de `commands*.md`
    conferidas com as listas de menus de `ui.rs`;
  - F1 com o menu CTRL ou ALT aberto pelo F9 abre a página daquele menu;
  - `shot.py`: `step:`, `expect:` e `SHOT_DUMP=1`.
- **Defeito achado pela animação e corrigido:** na vista Usage, a pergunta do Delete
  não era desenhada (o menu USAGE ficava por cima); a pergunta existia e respondia às
  teclas. Corrigido em `ui.rs`, com teste.
- **Site:** Instalar e Recursos saíram de `site/pages/` (viraram `start` e
  `overview`), com redirecionamento; cartões da abertura em inglês reescritos.
- **`tagging.md`:** a parte dos menus foi para `menus.md`; ficou um resumo com os
  links, e a âncora `#what-ctrl-and-alt-mean` continua valendo.
- **Índices traduzidos:** ganharam uma seção provisória com os links das páginas
  novas, para o teste do índice passar. Sai quando as traduções forem feitas.

Falta:

- receitas de Batch (tabela vira script) e de rclone (sem animação: não há
  substituto do rclone no gerador);
- revisar os 32 tópicos antigos com a skill;
- abertura do site em português; `README.md` e `README.pt-BR.md`;
- capturas dos menus (depois das seis decisões);
- traduções, por último.
