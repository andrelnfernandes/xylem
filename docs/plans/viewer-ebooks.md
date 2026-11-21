# Plano: EPUB e FB2 como documento no viewer

**Proposta para a 0.9.0** (pedida em 2026-11-20). Nada aprovado para código.

## Objetivo

Um ebook abre como os documentos do Office: títulos, parágrafos, listas e
tabelas, na largura da tela, com as buscas do viewer, e um quadro no topo com o
que o arquivo traz além do texto (scripts, recursos de fora, DRM, metadados).

Não é um "leitor de ebooks" para ler um livro inteiro: é **inspecionar** um
livro sem abrir o leitor. Você vê o conteúdo e o sumário, acha uma passagem e
sabe se o arquivo baixado de um site qualquer traz scripts ou rastreadores. É a
mesma postura do resto: abrir sem executar nada.

Hoje um `.epub` abre como ZIP (a lista de partes), e um `.fb2` como XML cru.

## Formatos

- **EPUB 2 e EPUB 3** (`.epub`).
- **FB2** (FictionBook: `.fb2`, e `.fb2.zip` com um único `.fb2` dentro).
- Fora: MOBI e AZW3 (formato do Kindle, documentado só por engenharia reversa,
  com compressão própria e quase sempre DRM nos livros comprados); PDF (extrair
  texto é difícil e o parser é uma grande superfície de ataque: fica com o
  leitor do sistema, `O`); CBZ/CBR (são imagens).

## O que já existe e será usado

O viewer do Office (`src/viewer/office.rs`, plano `viewer-office.md`) já resolve
as partes difíceis:

- **O `Package`**: o ZIP lido parte por parte, cada uma descomprimida até um
  limite (`Limits.part`), com um total (`Limits.total`), um teto de entradas
  (`MAX_ENTRIES`) e um prazo (`Limits.time`).
- **O XML sem DTD e sem entidades próprias** (`quick-xml`, com o teste da
  bomba de entidades), com teto de profundidade (`MAX_DEPTH`) e de eventos
  (`MAX_EVENTS`).
- **`DocEvents`** (`src/viewer/doc_events.rs`) e o `markdown::Doc`: o texto
  entra como `Text`, nunca como Markdown; layout, quebra, busca e cores vêm de
  graça.
- **O quadro no topo**, como o de macros e templates remotos do Office.
- **`data-encoding`** para o base64 do FB2, se for preciso medir uma imagem
  sem decodificá-la inteira.

## EPUB

### Como o livro é montado

1. `META-INF/container.xml` diz onde está o pacote (o OPF).
2. O OPF tem os metadados, o `manifest` (cada parte, com o tipo) e o `spine`
   (a ordem de leitura). Cada item do `spine` é um capítulo, lido na ordem.
3. Cada capítulo (XHTML) vira eventos:

| XHTML | No documento |
|---|---|
| `h1`–`h6` | título do mesmo nível (o capítulo começa num título, mesmo que o XHTML não traga um: o do sumário) |
| `p`, `div` com texto | parágrafo |
| `ul`, `ol`, `li` | lista |
| `table`, `tr`, `td`, `th` | tabela |
| `em`, `i`, `strong`, `b`, `s`, `del` | itálico, negrito, tachado |
| `blockquote` | citação |
| `pre`, `code` | bloco de código, código |
| `a href` | link, com o endereço real ao lado se for de fora |
| `br`, `hr` | quebra, linha |
| `img`, `svg`, `video`, `audio` | uma linha: `[image: images/cover.jpg]` |
| `script`, `style`, `head` | nada no texto (o `script` vai para o quadro) |

4. **O sumário** (`nav.xhtml` no EPUB 3, `toc.ncx` no EPUB 2) vira a lista de
   capítulos: uma tecla abre a lista, Enter vai ao capítulo. Os títulos de cada
   capítulo também servem para "próximo/anterior" (a tecla a decidir pelo que o
   viewer já usa para títulos no Markdown).

### Entidades do HTML

O XHTML de muitos EPUBs usa `&nbsp;`, `&mdash;`, `&eacute;` e outras que o XML
não conhece, e os EPUB 2 declaram a DTD do XHTML para defini-las. A DTD nunca é
lida: uma **tabela fixa** com as entidades nomeadas do HTML resolve, no código.
Uma entidade fora da tabela aparece como está (`&foo;`), sem erro.

### XHTML malformado

Há EPUBs com XHTML que não é XML válido (tags sem fechar). O leitor é
tolerante: não exige nomes de fechamento iguais, fecha o que ficou aberto no
fim do capítulo, e um capítulo que não dá para ler vira uma linha
"`[chapter 7: cannot be read: motivo]`" sem derrubar o livro.

### O quadro no topo

- **Metadados**: título, autores, editora, idioma, data, identificador (ISBN,
  UUID), e a ferramenta que gerou o arquivo quando dita (`meta generator`).
- **Scripts**: itens do manifest com `properties="scripted"`, elementos
  `<script>` e atributos `on*` (`onload`...) nos capítulos, com a contagem e
  onde estão. Nada é executado; o aviso diz que um leitor que executa scripts
  os rodaria.
- **Recursos de fora**: `src` ou `href` com `http://`, `https://` ou `//` em
  imagens, folhas de estilo, fontes e iframes, com os endereços. Um leitor que
  os carrega avisa quem publicou que o livro foi aberto, e de onde.
- **Formulários e iframes** (`form`, `iframe`, `object`, `embed`), contados.
- **DRM**: `META-INF/encryption.xml` cifrando capítulos (não só fontes) → o
  livro não pode ser lido; o quadro diz "protected by DRM (Adobe ADEPT,
  Readium LCP...)" pelo algoritmo declarado, e o resto do documento fica só com
  os metadados e o sumário. Fontes ofuscadas (o caso comum, que não é DRM) só
  são citadas.
- **Partes que não estão no manifest**, e itens do manifest que não existem no
  ZIP (sinal de arquivo montado à mão).

## FB2

Um único XML (`FictionBook`):

- `description/title-info`: título, autores, gênero, idioma, data, e
  `document-info` (programa que gerou, versão).
- `body` com `section` e `title` aninhados → títulos pelo nível de
  aninhamento; `p`, `empty-line`, `poem`/`stanza`/`v` (versos, um por linha),
  `cite`, `epigraph`, `table`, `emphasis`, `strong`, `strikethrough`, `code`.
- `body name="notes"` → as notas, no fim, com os links `a l:href="#n1"`
  ligando o texto à nota.
- `binary` (imagens em base64) → nunca decodificadas: uma linha com o tipo e o
  tamanho, como no Jupyter.

**Codificação**: muitos FB2 são `windows-1251` ou `KOI8-R`, declarados no
prólogo do XML. O `src/charset.rs` hoje só tem UTF-8, UTF-16, CP437, CP850 e
Windows-1252: a etapa 2 acrescenta as duas codificações cirílicas (tabelas
fixas de 128 posições, como as que já existem), sem crate nova. Codificação
desconhecida: lê como UTF-8 com substituição, e o quadro diz que a declarada
não é suportada.

**`.fb2.zip`**: um ZIP com exatamente um `.fb2` dentro, lido pelo `Package` com
os mesmos limites. Com mais de um arquivo, abre como ZIP comum.

O quadro do FB2 é menor: metadados, links de fora (`l:href` com `http`) e o
tamanho das imagens embutidas.

## Segurança

O que o Office já garante vale aqui, pelo mesmo código:

1. **ZIP com limites** por parte, total, entradas e tempo; bomba de ZIP já
   tratada.
2. **XML sem DTD e sem entidades**; a DTD do XHTML nunca é buscada nem lida
   (nada sai para a rede).
3. **Texto como texto**: nada do livro é interpretado como Markdown; controles
   e caracteres de direção do texto marcados, como em todo o viewer.
4. **Nada é executado nem carregado**: nem scripts, nem CSS, nem fontes, nem
   imagens, nem recursos de fora.
5. **Caminhos dentro do ZIP** (`../`, absolutos) no OPF ou no `spine` são
   resolvidos só como nomes de partes do próprio ZIP, nunca como caminhos do
   disco; uma referência que sai do pacote entra no quadro como suspeita.
6. **O parser corre sob o `panic_guard`**, como o do Office.

A mais: o sumário e o `spine` têm teto de itens (o mesmo `MAX_SLIDES` do Office
serve de ordem de grandeza: 10.000 capítulos), e um `spine` com o mesmo
capítulo repetido não o lê duas vezes.

## Onde entra

- No viewer (`V`) e no Autoview (F7), com o limite menor do Autoview.
- Na lista de formatos do manual (`viewer.md`, uma seção "Ebooks"), nas notas
  da versão, e no `W`: a coluna `type` passa a dizer `EPUB ebook` e `FB2
  ebook`.
- **Na Varredura** (plano `views.md`), quando ela existir: "ebooks com scripts
  ou recursos de fora" como uma categoria, pelo mesmo leitor.

## Lembrar onde parou (pedido do autor, 2026-11-20)

Uma tecla no viewer de um ebook (proposta: `P`, de posição; conferir no
`keymap.rs` na etapa 1 que está livre no viewer) liga ou desliga, **para aquele
livro**, lembrar onde a leitura parou. Desligado por padrão: nada é guardado sem
o usuário pedir.

- **Ligado**: ao fechar o viewer, a posição é guardada. Ao abrir o livro de
  novo, o viewer volta lá e diz "Back at chapter 7 (P forgets)". O rodapé
  mostra que o livro está sendo lembrado.
- **`P` de novo**: esquece aquele livro e apaga o registro dele.
- **A posição** é o capítulo e o parágrafo dentro dele, não a linha da tela:
  assim continua certa se a largura do terminal mudar ou se o texto for
  quebrado de outro jeito (`W`).
- **Qual livro é qual**: pelo identificador do livro (`dc:identifier` no EPUB,
  `id` no `document-info` do FB2) junto com o tamanho; sem identificador, pelo
  SHA-256 do arquivo. Assim o livro é reconhecido se for movido ou renomeado,
  e um arquivo diferente com o mesmo nome não herda a posição.
- **Onde fica**: `reading.toml` no diretório de estado
  (`~/.local/state/xylem/`, ao lado do `history.toml`), gravado do mesmo jeito
  atômico e com permissão `0600`.
- **Privacidade**: a lista do que alguém lê é pessoal. O arquivo guarda só um
  hash da identidade do livro e a posição: nem título, nem autor, nem caminho.
  Quem abrir o `reading.toml` vê números, não a biblioteca.
- **Limite**: no máximo 1.000 livros; passando disso, sai o lembrado há mais
  tempo.
- **Funciona** também com o livro dentro de um arquivo compactado ou numa
  máquina remota, porque a identidade sai do conteúdo; o registro fica sempre
  na máquina local.

Fica só para EPUB e FB2. Estender a outros documentos (um Markdown longo, um
texto grande) é uma decisão à parte, depois de usar.

## Etapas

1. **EPUB**: `container.xml`, OPF, `spine`, XHTML para eventos, entidades, o
   quadro (metadados, scripts, recursos de fora, DRM), o sumário como lista.
   A tecla de lembrar onde parou, com o `reading.toml`.
   Testes: EPUB 2 e 3 montados nos testes (sem baixar nada); XHTML
   malformado; entidades; um `<!ENTITY>` hostil; `encryption.xml`; um `spine`
   que aponta para fora do pacote; um capítulo gigante (corte no limite, com
   aviso).
2. **FB2**: o XML, as seções aninhadas, as notas, os versos, o base64 não
   decodificado, `windows-1251` e `KOI8-R`, o `.fb2.zip`.
3. A seção do manual (em inglês), as notas da versão, a coluna `type`.

## Etapa 1 em detalhe (proposta, 2026-11-20)

Pedido do autor em 2026-11-20: começar pelos ebooks. Ao olhar o código, três
pontos mudam o plano acima, e a etapa 1 se divide em duas.

### O que o código pede

- **A DTD do XHTML.** O `Xml` do Office recusa qualquer `<!DOCTYPE>`, e todo
  capítulo de EPUB 2 começa com um (`<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML
  1.1//EN" ...>`), como o EPUB 3 (`<!DOCTYPE html>`). Para os ebooks, um
  `DOCTYPE` **sem subconjunto interno** é aceito e ignorado (nunca lido, nunca
  buscado); com subconjunto interno (`[`, onde moram os `<!ENTITY>`), o
  capítulo é recusado como hoje. O Office continua recusando todos.
- **O sumário e os links internos** usam o que o Norton Guide já tem: o
  documento feito com `markdown::from_events_linked`, Tab e Shift+Tab escolhem
  um link, Enter vai até onde ele leva, Backspace volta. O `follows_links` do
  viewer passa a valer também para ebooks. No modo ligado os links não mostram
  o endereço: um link para fora do livro ganha o endereço escrito ao lado,
  como texto, e não é seguido.
- **A tecla de lembrar onde parou.** `P` já é "arquivo marcado anterior" no
  viewer (Ctrl+V). Proposta: **`R`** (remember), livre no viewer.

**Aprovado em 2026-11-20**, com as quatro escolhas como propostas: Space vira
página num livro (o próximo achado fica em `+` e `-`); Tab abre o sumário na
leitura; justificado desligado e parágrafos de livro por padrão; a lista de
todos os livros é uma opção à parte, desligada (`reading_titles`). O `R`
lembra onde parou, e o tempo de leitura é a opção `reading_stats`, desligada.

### 1a. O EPUB como documento

**Onde fica o código.** `src/viewer/epub.rs`, novo. O `Package` e o `Xml` saem
de `office.rs` para `src/viewer/package.rs`, compartilhados (só mudam de
lugar; o `Xml` ganha uma opção `doctype: Allow | Refuse`). `office.rs` só passa
a usá-los de lá.

**Como se reconhece.** Nome `.epub` e a parte `mimetype` com
`application/epub+zip`. Sem isso, abre como hoje (a lista de partes do ZIP).

**A montagem**, nesta ordem:

1. `META-INF/container.xml` → o caminho do OPF (o primeiro `rootfile` com
   `media-type="application/oebps-package+xml"`).
2. O OPF → metadados (`dc:title`, `dc:creator`, `dc:publisher`,
   `dc:language`, `dc:date`, `dc:identifier`, `meta name="generator"`), o
   `manifest` (id → caminho, tipo, `properties`) e o `spine` (a ordem).
3. Cada caminho é resolvido em relação à pasta do OPF, só como nome de parte
   do ZIP (o `resolve` do Office). Um que sai do pacote (`../../`, absoluto,
   `file:`) não é lido e vai para o quadro.
4. O sumário: `nav.xhtml` (o item com `properties="nav"`, a `nav
   epub:type="toc"`) no EPUB 3; senão o `toc.ncx` (o `spine toc=`). Até
   10.000 entradas, com o nível.
5. Cada capítulo do `spine`, uma vez só (um repetido é pulado), até 10.000.

**Cada capítulo vira eventos**, pela tabela do plano, com estes detalhes:

- **Começa num título que é um lugar** (`c12`, para links e para `]`/`[`): o
  primeiro `h1`–`h6` do capítulo, se vier antes de qualquer texto; senão, um
  título acrescentado com o nome do sumário, ou `Chapter 12`.
- **Um link interno** (`cap2.xhtml#s3`, `#nota1`) leva ao título com aquele
  `id`, quando é um título; senão, ao começo do capítulo. Ir até qualquer
  parágrafo pedia lugares fora dos títulos no `markdown::Doc`: fica para
  depois, se fizer falta.
- **Notas de rodapé** do EPUB 3 (`aside epub:type="footnote"`) ficam onde
  estão, como citação.
- **Entidades do HTML**: a tabela fixa das 252 do HTML 4 (`&nbsp;`,
  `&mdash;`, `&eacute;`...), que é o que os EPUB 2 usam. As outras do HTML5
  aparecem como estão (`&foo;`).
- **XHTML malformado**: `check_end_names` desligado (já é assim), o que ficou
  aberto é fechado no fim do capítulo, e um capítulo que não dá para ler vira
  `[chapter 7: cannot be read: motivo]`.

**O documento**, de cima para baixo:

1. **Ebook**: os metadados e o quadro (abaixo).
2. **Contents**: o sumário como lista aninhada, cada entrada um link para o
   seu capítulo. Sem sumário, a lista dos capítulos do `spine`, pelo título.
3. Os capítulos, na ordem do `spine`.

**O quadro.** Como no plano: scripts (manifest `scripted`, `<script>`,
atributos `on*`, com onde), recursos de fora (`src`/`href` com `http:`,
`https:` ou `//`, até 1.000 endereços), `form`/`iframe`/`object`/`embed`
contados, DRM pelo `encryption.xml` (capítulos cifrados: o documento fica só
com o quadro e o sumário; fontes ofuscadas só citadas), partes fora do
manifest e itens do manifest que não estão no ZIP, caminhos que saem do
pacote. Quando nada disso aparece, uma linha: `No scripts, nothing fetched
from outside`.

**No viewer.**

- `V` e Autoview (F7), com os limites do Office (4 MiB no Autoview; 64 MiB por
  parte e 256 MiB no todo com `V`), sob o `panic_guard`.
- `]` e `[`: próximo e anterior capítulo (como os arquivos de um patch). Tab,
  Enter e Backspace: os links, como no Norton Guide. `H` vai aos bytes.
- O rodapé diz o capítulo em que se está: `chapter 7 of 31`.

**Testes** (EPUBs montados no teste, com o `zip` de `test_docs`):

- EPUB 3 com `nav.xhtml` e EPUB 2 com `toc.ncx`: títulos, parágrafos, listas,
  tabela, ênfase, o sumário com os links, a ordem do `spine`.
- O `DOCTYPE` do XHTML 1.1 aceito; um `DOCTYPE` com `<!ENTITY>` interno
  recusado sem derrubar o livro; as entidades do HTML.
- XHTML malformado; um capítulo que não é XML; um capítulo repetido no
  `spine`; um `spine` que aponta para `../../etc/passwd`.
- O quadro: `<script>`, `onload`, imagem de `https://`, `encryption.xml` com
  capítulo cifrado e só com fonte.
- Um capítulo acima do limite: cortado, com o aviso.
- No viewer: `]`/`[`, Tab e Enter num link do sumário, Backspace.

### Feito na 1a (2026-11-20)

- **`src/viewer/package.rs`**: o `Package`, o `Xml`, `attr`, `prefixed`,
  `resolve` e os `Limits` saíram de `office.rs` sem mudar o que fazem. O
  `Xml::html` aceita um `DOCTYPE` sem subconjunto interno e recusa o que tem
  `[`; conhece as 252 entidades do HTML 4 (`package/html.rs`, tabela fixa,
  com teste) e não conta como abertos os elementos que o HTML nunca fecha
  (`<br>`, `<img>`). O OPF, o `container.xml` e o `toc.ncx` também são lidos
  assim, porque o NCX traz a DTD dele.
- **`src/viewer/epub.rs`**: `render` devolve um `Book` (o documento e os
  capítulos: lugar e título). O documento é feito com
  `markdown::from_events_linked`; os links do sumário e do texto ganham uma
  chave na leitura e são resolvidos no fim, para o título com aquele `id` ou
  para o começo do capítulo; um link para algo que não é capítulo vira texto.
- **O texto**: espaços juntados como no HTML; o parágrafo aberto só quando
  chega texto (um `<p></p>` não deixa nada); os itens de lista sem parágrafo
  próprio ficam juntos; tabelas como no Office; `pre` como bloco de código;
  imagens, `svg`, vídeo e áudio como uma linha em itálico.
- **XHTML quebrado**: cada fim de elemento fecha até o elemento de mesmo
  nome aberto, ou nada; acima de 512 abertos, o resto é lido só como texto;
  um capítulo que não é XML guarda o que já foi lido e diz que não pôde ser
  lido.
- **O quadro**: também `javascript:` em links, `@import` e `url()` de fora
  nas folhas de estilo do manifest, `remote-resources` do manifest e itens do
  `spine` que não estão no manifest.
- **No viewer**: `epub` e `chapters`; `]`/`[` vão de capítulo em capítulo;
  Tab, Enter e Backspace seguem os links, como no Norton Guide; o rodapé diz
  `chapter 7 of 31: The Storm`. Autoview e `V` com os limites do Office.
- Conferido no tmux com um EPUB gerado: o quadro, o sumário, `]`, Tab e
  Enter, e o Autoview.

### 1b. O leitor (pedido do autor, 2026-11-20)

O autor quer, neste módulo, um leitor acima da média para ebooks de texto
(sem imagens por enquanto), com enfeites a mais para quem usa Nerd Fonts. Isso
muda a postura do começo do plano ("inspecionar, não ler"): o livro abre para
**ler**, e o relatório de inspeção continua a uma tecla, sem nunca ser
escondido quando há algo a dizer.

**Duas vistas do mesmo livro:**

- **Leitura** (a que abre): a página de rosto, os capítulos com layout de
  livro, o rodapé de progresso. Se o quadro tem algo (scripts, recursos de
  fora, DRM, caminhos que saem do pacote), a página de rosto traz uma linha em
  destaque: `⚠ 3 scripts, 2 addresses outside (I shows the report)`.
- **Relatório** (`I`, de inspect; no modo de tabela `I` já é a estrutura,
  ideia parecida): o documento descrito na 1a, com os metadados, o quadro, o
  sumário e o texto sem enfeites. `I` de novo volta à leitura, no mesmo
  capítulo.

**O layout de leitura** é uma extensão do `markdown::Doc`, não um renderizador
novo: um `Style` passado ao `layout` (o padrão é o de hoje, e só o ebook usa
outro). Assim a busca, as linhas lógicas, os saltos e o `W` continuam valendo.

- **A medida**: o texto numa coluna de até 70 células, centralizada na tela
  (`reader_width`, F12; 0 = a largura toda). Numa tela estreita, a largura
  toda.
- **Parágrafos de livro**: recuo de 3 células na primeira linha e sem linha em
  branco entre eles; sem recuo o primeiro depois de um título ou de uma quebra
  de cena. Opção para parágrafos em bloco (linha em branco, sem recuo).
- **Justificado**: opção, desligada por padrão (numa fonte de largura fixa os
  espaços largos incomodam mais do que ajudam). Ligado, a última linha do
  parágrafo fica como está. Sem hifenização (pediria dicionários por língua).
- **Página de rosto**: título, autores, editora e ano, centralizados, numa
  moldura de caixa; abaixo, o sumário curto (os capítulos do primeiro nível).
- **Abertura de capítulo**: linhas em branco acima, o título centralizado e em
  negrito, um ornamento abaixo.
- **Quebra de cena**: `<hr>`, e parágrafos só com `*`, `* * *`, `***`, `#`,
  `~` ou `⁂`, viram um ornamento centralizado.
- **Citações e epígrafes** com recuo dos dois lados e em itálico; **versos**
  linha a linha, como estão; **ênfase** em itálico real (SGR 3) e negrito.
- **Notas**: a chamada vira número sobrescrito (`¹²³`); o texto das notas não
  interrompe o capítulo.

**A navegação:**

| Tecla | Na leitura |
|---|---|
| Space, PgDn / PgUp | página seguinte / anterior: a tela inteira, sem cortar linha (no resto do viewer Space é "próximo achado"; num livro vira página, e o próximo achado fica em `+` e `-`) |
| `]` `[` | capítulo seguinte / anterior |
| Tab | o sumário sobre a tela, com o capítulo atual marcado e os já lidos; letras filtram, Enter vai |
| Enter | as notas cujas chamadas estão na tela, num quadro; Esc fecha |
| `g` | vai a uma linha, ou a uma porcentagem do livro (`40%`) |
| Backspace | volta de onde um salto (sumário, `g`, link) trouxe |
| `/`, `F`, `B` | as buscas de sempre, no livro inteiro |
| `I` | leitura ou relatório |
| `R` | lembrar onde parou (1c) |
| `Z` | foco: esconde a moldura do viewer, fica só o texto e uma linha de progresso |

Os links do texto (raros em ficção fora do sumário) seguem com Tab/Enter só
no relatório, como no Norton Guide; na leitura, Tab é o sumário, como a lista
de arquivos de um patch.

**O rodapé de progresso**: `Chapter 7 of 31 · The Storm` à esquerda; à
direita a barra do livro, a porcentagem e o tempo que falta no capítulo e no
livro, pelas palavras e uma velocidade de leitura (`reading_wpm`, 250 por
padrão): `▰▰▰▰▱▱▱▱ 46% · 12 min left in chapter · 4 h 10 min in book`.

**Com Nerd Fonts** (`icons = "nerd-fonts"`), os mesmos lugares ganham
glifos; sem elas, caracteres que toda fonte tem. Nada muda de largura entre
os dois (um teste mede cada glifo em 1 célula, como em `icons.md`).

| Lugar | Sem Nerd Fonts | Com Nerd Fonts |
|---|---|---|
| Página de rosto | moldura `┌─┐` | moldura com cantos redondos `╭─╮`, ícone de livro e de autor |
| Ornamento de capítulo e quebra de cena | `*   *   *` | um floreio de glifos |
| Barra de progresso | `━━━━────` | os glifos de barra com pontas redondas |
| Rodapé | `Ch 7/31`, `12 min` | ícones de livro, relógio e marcador |
| Sumário | `▸` no atual, `✓` nos lidos | ícones de capítulo, atual e lido |
| Lembrando a posição | `[remembered]` | ícone de marcador |

Os glifos exatos se escolhem na implementação, na tela, com o autor.

**Fora, por ora**: imagens (nem a capa), CSS do livro (fontes, cores e
alinhamentos do editor), hifenização, marcadores múltiplos e grifos (o `R`
guarda uma posição só), leitura em voz alta. O layout de leitura em Markdown e
texto puro é uma decisão depois de usar nos ebooks.

### Feito na 1b (2026-11-20)

- **`markdown::BookStyle`** e `Doc::set_book`: o mesmo documento, disposto
  como livro. As linhas de texto ganharam um papel (parágrafo, primeira linha
  de parágrafo, título) e as linhas em branco entre dois parágrafos ficaram
  marcadas, para o livro tirá-las. Recuo de 3 células, títulos centralizados
  sem `#`, um ornamento sob os de capítulo (`H1`, `H2`), a regra como quebra
  de cena, e o justificado (os espaços alargados por igual, nunca a última
  linha, nem uma linha curta demais para não abrir buracos de mais de três
  células).
- **As duas vistas** saem do mesmo `epub::render`: `Book` tem `report` e
  `reading`. A de leitura começa por uma página de rosto (título, autores,
  editora · ano, o aviso, quantos capítulos e palavras), não tem as notas
  `aside`, e as chamadas de nota viram sobrescritos. Uma chamada é o link com
  `epub:type="noteref"`, ou dentro de `sup`, ou com classe de nota, ou de
  texto curto (`1`, `[12]`, `*`); o texto da nota é o do elemento com aquele
  `id` (o parágrafo inteiro, quando o `id` está numa âncora dentro dele).
  Parágrafos só com `* * *`, `***`, `#`, `⁂` viram quebra de cena.
- **`src/viewer/reader.rs`**: as opções, os glifos (com e sem Nerd Fonts, do
  mesmo tamanho, com teste), a barra, o tempo que falta e o progresso por
  palavras.
- **No viewer**: a coluna centralizada (a margem entra como espaço na
  linha); Space vira página; Tab abre o sumário (a lista das planilhas, com
  `▸` no atual e `✓` nos anteriores); Enter abre as notas chamadas na tela;
  `g 40%`; Backspace volta; `I` troca de vista no mesmo capítulo; `Z` o
  foco, e Esc sai do foco antes de fechar. O aviso do quadro também aparece
  como nota ao abrir o livro.
- **F12**: a página 5, "Reading (ebooks)", com A a D (largura, parágrafos,
  justificado, velocidade); largura e velocidade andam por valores fixos
  (0, 60, 66, 70, 80, 100; 150 a 500). E a G ficam para a 1c.
- **Diferente do plano**: a página de rosto não tem moldura (os títulos
  centralizados com o ornamento fizeram o papel dela), e não traz o sumário
  curto, só quantos capítulos e palavras: o sumário é o Tab. A posição conta
  como capítulo atual aquele cujo título está até duas linhas abaixo do topo
  (a linha em branco e o título).
- Conferido no tmux: rosto, capítulo, recuos, quebra de cena, notas, sumário,
  rodapé, foco, `I`, e com Nerd Fonts, justificado e 60 colunas.

### 1c. Lembrar onde parou (`R`)

Como no plano, com `R` no lugar de `P`. A posição é o número do capítulo e a
linha lógica dentro dele (não a linha da tela: não muda com a largura nem com
`W`). A identidade é o SHA-256 de `dc:identifier` + tamanho, ou do arquivo
inteiro (lido de novo, até o limite do `V`) quando não há identificador.
`reading.toml` no diretório de estado, gravado pelo `write_atomic` do
`config.rs` (que vai para um lugar comum), com `0600`.

Testes: ligar, fechar, abrir de novo e voltar ao capítulo; `R` de novo
esquece e apaga; o arquivo só tem hashes e números; o limite de 1.000; um
livro movido de lugar continua reconhecido; outro livro com o mesmo nome não.

### 1c, parte 2. O tempo de leitura (pedido do autor, 2026-11-20)

O autor quer saber, por livro, quanto tempo passou lendo, a velocidade, até
onde chegou. Vai junto com o `R`: **o mesmo interruptor por livro**. Um livro
lembrado guarda a posição e o tempo; um livro não lembrado não guarda nada.

**O que conta como tempo de leitura:**

- Só com o livro aberto no viewer (`V`) na leitura; o Autoview (F7), o
  relatório (`I`) e o viewer escondido atrás de outra janela não contam.
- **Parado não conta**: sem nenhuma tecla por 3 minutos (`reading_idle`, F12),
  o relógio para no momento da última tecla; a próxima tecla volta a contar.
  Assim, sair de perto com o livro aberto não infla o tempo.
- Uma **sessão** é um trecho contínuo: abre e fecha o livro, ou uma pausa
  maior que 30 minutos.

**A velocidade** (palavras por minuto) mede só a leitura de verdade: as
palavras das páginas que se passaram para a frente (Space, PgDn, ↓), divididas
pelo tempo nelas. Não contam: saltos (sumário, `]`/`[`, `g`, buscas, links),
voltar páginas, e uma página passada em menos de 5 segundos (é folhear). Com
pelo menos 10 minutos medidos, a velocidade medida substitui os 250 do
`reading_wpm` no "tempo que falta" do rodapé.

**O que se guarda** (no `reading.toml`, junto com a posição): o tempo total,
as palavras lidas e o tempo delas (a velocidade sai daí), a parte mais longe a
que se chegou (não só a posição atual: voltar ao começo não apaga o
progresso), a data da primeira e da última leitura, a data em que se chegou
ao fim, e as últimas 100 sessões (início, duração, palavras).

**Onde aparece:**

- **Na página de rosto**: `Your reading: 3 h 20 min in 6 sessions · 46% ·
  238 words/min · since 2 Nov`.
- **No relatório (`I`)**: uma seção "Your reading" com esses números, as
  sessões numa tabela, e quanto falta pela velocidade medida.
- **No rodapé**, com o livro lembrado: o tempo da sessão atual.

**Privacidade.** Hoje o plano guarda só um hash do livro: ninguém que abra o
`reading.toml` sabe o que você lê. Com o tempo de leitura, isso continua: os
números ficam presos ao hash, e você os vê ao abrir o livro. **Uma lista de
todos os livros com os seus tempos** (um "meus livros") pediria guardar os
títulos. Proposta: uma opção à parte, desligada por padrão
(`reading_titles`), que guarda título e autor junto com os números e libera
uma lista de leitura; até lá, os números só aparecem dentro de cada livro.

**A escolha no F12 (pedido do autor, 2026-11-20).** O tempo de leitura é
uma opção própria, `reading_stats`, separada do `R`:

- **Desligada** (o padrão): o `R` lembra só a posição; nada de tempo,
  velocidade ou sessões é medido nem gravado, e a página de rosto e o
  relatório não mostram "Your reading". O rodapé volta a usar os 250 do
  `reading_wpm`.
- **Ligada**: um livro lembrado com `R` guarda também o tempo, como acima.
- **Desligar depois de usar** para de medir e esconde os números, e pergunta
  se apaga os que estão guardados (`Delete the reading time of 12 books?`).
  Respondendo não, eles voltam se a opção for ligada de novo. A posição dos
  livros fica de qualquer jeito: ela é do `R`.

**Uma página nova no F12, "Reading (ebooks)"**: as quatro páginas de hoje já
estão cheias, e as opções do leitor ficam juntas:

| Letra | Opção | Padrão |
|---|---|---|
| A | Width of the text column (0: the whole screen) — `reader_width` | 70 |
| B | Paragraphs: book (indented) or block (a blank line) — `reader_paragraphs` | book |
| C | Justify the text — `reader_justify` | no |
| D | Reading speed for the time left, words per minute — `reading_wpm` | 250 |
| E | Keep statistics of reading time (with R) — `reading_stats` | no |
| F | Minutes without a key before the reading clock stops — `reading_idle` | 3 |
| G | Keep the titles of the books, for a reading list — `reading_titles` | no |

`F` e `G` só valem com `E` ligada; desligada, aparecem como `(needs E)`.
Cada uma tem a sua linha comentada no `--default-config`, e a seção do
manual (`configuration.md`) diz o que se guarda e onde, como na opção do
fuzzy.

**Testes**: o relógio com um tempo de mentira (como o `RESTART_EVERY` do
fuzzy, zero nos testes): conta com teclas, para depois do `reading_idle`,
não conta no Autoview nem no relatório; com `reading_stats` desligado nada
é medido; desligar pergunta antes de apagar; a velocidade ignora saltos, volta e
folheio; o mais longe não recua; `R` desligado não grava nada e apaga o que
havia.

### 1c em detalhe (2026-11-20, ao começar)

- **`src/reading.rs`** (novo, ao lado de `state.rs`): o `reading.toml`, lido e
  gravado inteiro; a gravação privada e atômica sai de `state::save_state`
  para uma função comum (`state::write_private`). Até 1.000 livros, sai o
  lembrado há mais tempo.
- **A identidade**: SHA-256 de `dc:identifier`, um zero e o tamanho; sem
  identificador, dos bytes do arquivo (até o limite do `V`) e do tamanho.
  Guardada em hex, sem mais nada que diga qual livro é.
- **As opções chegam antes de o livro ser lido**: o `ReaderOptions` passa a
  ir no `LoadOptions`, porque a página de rosto e o relatório são montados
  ao abrir.
- **"Your reading" na página de rosto e no relatório** é o que estava
  guardado ao abrir o livro (o documento é montado uma vez); o rodapé mostra
  ao vivo o tempo da sessão. O que muda durante a leitura aparece na próxima
  vez que o livro abrir.
- **A lista de todos os livros** (com `G` ligada) fica no relatório (`I`) de
  qualquer livro, numa seção "Your books": título, autor, tempo, até onde
  chegou e a última leitura.
- **Desligar `E`** pergunta ao gravar o F12 (F3), não na hora da tecla:
  `Delete the reading time of 12 books? (y/n)`.
- **O relógio** recebe o instante de cada tecla (nos testes, instantes
  inventados): soma o tempo desde a tecla anterior quando ele não passa do
  `reading_idle`; uma pausa maior que 30 minutos começa outra sessão.

### Feito na 1c (2026-11-20)

Como descrito acima ("1c em detalhe"), com estes detalhes:

- **`src/reading.rs`**: `Store` (até 1.000 livros), `Record`, `Session`,
  `identity`, `Clock` (o relógio recebe o instante de cada tecla: os testes
  usam instantes inventados), `summary` e as datas em UTC sem crate.
- **Nos testes**, o `reading::path()` só dá um arquivo se o teste pedir
  (`TEST_PATH`): nenhum teste grava o `reading.toml` do usuário.
- **Quando grava**: `R` liga (um registro com a posição) ou desliga (apaga o
  livro); ao fechar o viewer, o `Reader` grava a posição e o tempo, só se a
  leitura andou ou o relógio correu (o Autoview passando por um livro
  lembrado não regrava nada).
- **A velocidade** conta `Space`, `PgDn` e `↓` com ao menos 5 s desde a tecla
  anterior; com 10 minutos medidos, a velocidade medida daquele livro
  substitui o `reading_wpm` no tempo que falta do rodapé (a de quando o livro
  abriu).
- **O rodapé** de um livro lembrado começa com `[R]` (com Nerd Fonts, o
  marcador) e, com o tempo ligado, `12 min this time`.
- **F12**: E, F (1, 2, 3, 5 ou 10 minutos) e G; F e G dizem `(needs E)` com E
  desligada. Gravar com G desligada (ou com E) tira os títulos na hora; com E
  desligada, pergunta antes de apagar o tempo.
- **Um capítulo sem nome no sumário** passa a ter o do seu primeiro título,
  não o nome do arquivo.
- **Na página de rosto**, a contagem e "Your reading" ficam centralizadas.

### A ordem

1a (o livro lido: montagem, relatório, sumário), 1b (o leitor: layout,
navegação, rodapé, Nerd Fonts) e 1c (lembrar onde parou e o tempo de
leitura), um commit cada, com os testes de cada uma. As etapas 2 (FB2) e 3
(manual, notas, `type`) como no plano. O manual pode vir junto com cada etapa, se o autor preferir.

## Fora deste plano

- MOBI, AZW3, PDF, CBZ/CBR (ver "Formatos").
- Mostrar imagens.
- Converter ebooks (para texto, Markdown): o Batch da visão de texto já
  escreve linhas, se alguém precisar.
