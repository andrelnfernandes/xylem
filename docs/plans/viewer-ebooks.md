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

## Fora deste plano

- MOBI, AZW3, PDF, CBZ/CBR (ver "Formatos").
- Mostrar imagens.
- Converter ebooks (para texto, Markdown): o Batch da visão de texto já
  escreve linhas, se alguém precisar.
