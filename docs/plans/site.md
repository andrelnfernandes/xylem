# Plano: o manual no site, com menu lateral, e a tela de abertura

Origem: conversa de 2026-11-02. O autor viu o site de outro projeto chamado Xylem
(`xylem.ornl.gov`, feito com mdBook) e gostou do formato: menu lateral com os capítulos,
busca, um capítulo por página. Aprovado registrar o plano. **Tela de abertura feita** (opção `splash`, **P** na página 2
do F12). **Manual no site feito** na branch `site-manual` (mdBook 0.5.4).

## O nome

- O produto passa a ser chamado de **Xylem File Manager** no manual (título do livro no
  site e abertura do `index.md`) e na tela de abertura. O resto dos textos não muda.
- **O executável continua `xy`**, decisão do autor (2026-11-02). O pacote continua
  `xylem-fm`. (Para registro: `xfm` já existe, é o X File Manager do X11.)

## A tela de abertura

Ao iniciar, o nome estilizado por uns dois segundos, como os programas de DOS da época
do XTree. A arte (fonte de blocos com sombra, 44 colunas por 6 linhas):

```
██╗  ██╗██╗   ██╗██╗     ███████╗███╗   ███╗
╚██╗██╔╝╚██╗ ██╔╝██║     ██╔════╝████╗ ████║
 ╚███╔╝  ╚████╔╝ ██║     █████╗  ██╔████╔██║
 ██╔██╗   ╚██╔╝  ██║     ██╔══╝  ██║╚██╔╝██║
██╔╝ ██╗   ██║   ███████╗███████╗██║ ╚═╝ ██║
╚═╝  ╚═╝   ╚═╝   ╚══════╝╚══════╝╚═╝     ╚═╝

              F I L E   M A N A G E R

                      v0.6.0
```

- **Cores do tema:** os blocos `█` na cor da borda ativa; a sombra (`╔═╗║╚╝`) no tom
  apagado; "F I L E   M A N A G E R" no tom do texto; a versão apagada. No `mono`, sem
  cor. Só caracteres de bloco e de desenho de caixa, que o programa já usa; nada de
  Nerd Fonts.
- **Não atrasa nada:** é desenhada por cima da tela normal enquanto o programa já lê a
  pasta inicial por baixo. Some sozinha depois de 2 segundos, ou na primeira tecla, que
  só fecha a abertura e não é repassada ao programa.
- **Quando não aparece:** com a opção `splash = false` (entrada no F12; padrão ligada),
  num terminal menor que a caixa (58 colunas por 13 linhas) e ao conectar numa máquina
  remota ou container pela linha de comando (o ssh pode pedir senha logo de cara). Os modos de
  linha de comando que não abrem a tela (`--help`, `--version`) não mudam.
  Os testes e2e desligam com a variável `XY_NO_SPLASH`; as capturas de
  `docs/screenshots/tools/run.sh`, pela opção nas configurações do `fixture.sh`.
- **Testes:** desenho com `TestBackend` (a arte centralizada, a versão, nada em terminal
  pequeno, nada com a opção desligada), a tecla que fecha sem chegar ao programa, e o
  tempo, com o relógio injetado para o teste não esperar 2 segundos.

## O nome no cabeçalho

Pedido do autor (2026-11-02), feito: `Xylem v0.6.0` sempre no cabeçalho, antes da data,
como o XTreeGold mostrava o nome no canto. "Xylem" na cor das teclas, a versão apagada.
Sai quando a tela tem menos de 72 colunas, para não tirar espaço do caminho. Sem ™: o
nome não é marca registrada do projeto, e "Xylem" é marca de outra empresa.

## O site

Antes era uma página feita à mão em `site/` (inglês e `pt-br/`), e o manual só existia no
repositório e na ajuda do F1. Primeiro o manual entrou em `/manual/` (PR #94); depois, a
pedido do autor ("tudo com menu lateral"), **o site inteiro virou o livro**.

### Estrutura

- **Um livro mdBook por língua, o site inteiro:** inglês na raiz, os outros em `/pt/`,
  `/es/`, `/fr/`, `/de/`, `/zh/`. Menu lateral, busca no navegador, ← e → entre
  páginas, tema claro e escuro, sem servidor.
- **O menu lateral** começa pelas páginas do site, em `site/pages/<lang>/` (a abertura,
  Em ação, Recursos, Telas, Instalar, Em breve, Sobre; vieram da página antiga, com o
  mesmo HTML e as mesmas classes), e segue com o manual: a parte "Manual" com o índice
  (`docs/manual/index.md`, publicado como `manual.html`) e os grupos dele (Janelas e
  vistas, Trabalhando com arquivos, …) como partes. O `SUMMARY.md` é **gerado**, para
  não haver duas listas de capítulos para manter.
- **Línguas:** as páginas do site existem em inglês e português; as outras línguas
  usam as inglesas, e os tópicos sem tradução entram em inglês, como no F1. O seletor
  de língua no topo (`site/theme/languages.js`) leva à mesma página na outra língua.
- **Endereços antigos** redirecionam: `/pt-br/` e os livros de antes, `/manual/` e
  `/manual/<lang>/` (`[output.html.redirect]` do livro inglês).
- **Imagens** do site uma vez só, em `/assets/`, fora dos livros (`ASSETS/` nas páginas
  vira o caminho certo de cada língua); as do manual vão para o livro de cada língua.
- **Cabeçalho do HTML:** descrição e Open Graph (a imagem de compartilhamento) por
  `theme/head.hbs`, gerado por língua; o ícone do Xylem como favicon.

### Os arquivos do manual não mudam por causa do site

O F1 lê os mesmos `docs/manual/**/*.md` (embutidos com `include_str!`), e eles também
geram a página do man. Então o site se adapta a eles, nunca o contrário:

- os links entre capítulos (`tree.md`, `split.md#...`) já são os que o mdBook entende;
- nada de sintaxe própria do mdBook (`{{#include}}`, admonitions) nos arquivos;
- o `TRANSLATING.md` fica fora do livro;
- se algum capítulo usar algo que o mdBook mostra mal, corrige-se no `build.sh` (cópia
  em pasta de trabalho), não no original.

### Montagem e publicação

- `site/book.py` prepara cada língua numa pasta de trabalho ao lado da pasta de saída:
  copia as páginas e os tópicos, troca só o caminho das imagens (`../screenshots/` vira
  `screenshots/`, com as imagens copiadas), gera o `SUMMARY.md` (os títulos são o `#`
  de cada arquivo), o `theme/` e o `book.toml`. `site/build.sh` roda o `mdbook build`
  de cada uma e junta em `_site/` e `_site/<lang>/`, mais `_site/assets/`.
- `pages.yml`: `cargo install mdbook --version 0.5.4 --locked` (versão fixa, com as
  dependências do Cargo.lock dele conferidas pelo checksum do crates.io; mais lento que
  baixar o binário, mas sem checksum para manter à mão), e `docs/manual/**` nos caminhos
  que disparam a publicação.
- Consequência a lembrar: o site é publicado quando o `main` muda, então o manual do site
  passa a mostrar o `main`, que pode ter recurso ainda não lançado. Decisão do autor
  (2026-11-02): **o manual do site segue o `main`**, como o repositório.

### Segurança

O site é estático e só publica o que está no repositório. O mdBook roda no CI sobre
arquivos nossos; o cuidado é com a cadeia de fornecimento: versão fixa, `--locked`, e
nenhum plugin (`mdbook-*`) de terceiros sem necessidade. O JavaScript que o
mdBook traz (busca, tema) vem do próprio mdBook; nada de scripts de CDN.

## Ordem

1. **Tela de abertura** (pequena, independe do site).
2. **Manual no site** (PR #94).
3. **O site inteiro com menu lateral** (branch `site-sidebar`). Título "Xylem File
   Manager" nas 6 línguas.
