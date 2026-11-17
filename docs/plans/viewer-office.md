# Plano: DOCX, ODT e PPTX como documento no viewer

Origem: conversa de 2026-11-16 (veja [viewer-logs.md](viewer-logs.md)).
**Spec proposto, ainda não aprovado.**

## Objetivo

Um documento de escritório abre como o Markdown renderizado: títulos,
parágrafos, listas e tabelas, na largura da tela, com busca. Também mostra o
que um documento pode esconder de quem o abre: macros, conteúdo oculto, revisões
e links externos.

Hoje esses arquivos abrem em hex, porque são ZIP. As planilhas (XLSX, ODS)
continuam como tabela, sem mudança.

## Formatos

- **Word:** `.docx`, `.docm` (com macros), `.dotx` e `.dotm`.
- **OpenDocument:** `.odt` e `.ott`.
- **PowerPoint:** `.pptx`, `.pptm` e `.potx`. **Impress:** `.odp`.
- Fora: `.doc` e `.ppt` antigos (formato OLE binário, outro parser inteiro), e
  `.rtf`. Ficam para depois, se houver pedido.

## Como o documento é montado

- **Reaproveita o `markdown::Doc`** sem passar por texto Markdown. O leitor do
  documento gera os mesmos eventos do `pulldown-cmark` (`Start(Heading)`,
  `Text`, `End`, `Start(Table)` …) e os entrega ao `Builder` de
  `markdown.rs`, que passa a ter uma entrada pública para eventos.
  - Por isso o texto do documento **nunca é interpretado como Markdown**: um
    `**` ou um `[a](b)` dentro do DOCX aparece literal, e não há escape a acertar.
  - Layout, quebra na largura, busca, cores e o `Left`/`Right` vêm de graça.
- O que vira o quê:
  - Word: estilos `Heading1`…`Heading6` e `Title` → títulos; `w:numPr` →
    item de lista (numerado ou não, pelo `numbering.xml`); `w:tbl` → tabela;
    negrito, itálico e tachado; `w:br` → quebra; `w:tab` → espaço.
  - ODT: `text:h` com `outline-level` → título; `text:list` → lista;
    `table:table` → tabela.
  - PPTX e ODP: cada slide vira `## Slide n`, com o título primeiro e as caixas
    de texto na ordem do arquivo. As notas do apresentador vêm depois, marcadas
    `Notes`.
  - Imagem: `[image: nome.png]`. Nada é desenhado.
  - Link: o texto e o endereço real, inteiro, como no Markdown.
  - Notas de rodapé e comentários: no fim, numerados, com o autor do comentário.
- **Sem modo fonte:** `T` diz que o documento não tem fonte de texto, e `H` faz
  Documento → Hex.

## O que fica à vista (cabeçalho do documento)

Antes do texto, um bloco com o que o documento traz além dele:

- **Metadados** (`docProps/core.xml`, `meta.xml`): autor, quem salvou por
  último, datas de criação e de modificação, revisão e aplicativo. Isso costuma
  vazar sem que o autor saiba.
- **Macros:** se o pacote tem `vbaProject.bin` (ou `Basic/` no ODF), o
  cabeçalho diz `contains macros`, em destaque. Nada é lido nem executado.
- **Relações externas** (`TargetMode="External"` nos `.rels`): todas listadas
  com o endereço. Um modelo remoto (`attachedTemplate`) e um objeto OLE externo
  ficam em destaque, porque foi por aí que o CVE-2022-30190 (Follina) e outros
  ataques de modelo remoto chegaram.
- **Objetos embutidos** (`word/embeddings/`): nome e tamanho de cada um.
- **Revisões** (`w:ins`, `w:del`): o texto inserido aparece sublinhado e o
  removido tachado, os dois marcados com o autor. O cabeçalho conta quantos
  são. Não há "aceitar tudo": o texto apagado é justamente o que interessa ver.
- **Texto oculto** (`w:vanish`, `text:display="none"`) e **texto invisível**
  (cor igual ao fundo ou fonte de tamanho 1): aparece com a marca `hidden`, e o
  cabeçalho conta quantos são.

## Leitura do pacote

- `zip` e `quick-xml`, que já estão no projeto (pelas planilhas). **Nenhuma
  dependência nova.**
- Só as partes que interessam são abertas, pelo nome, a partir do
  `[Content_Types].xml` (OOXML) ou do `META-INF/manifest.xml` (ODF). Nada é
  extraído para o disco.

## Segurança

- **Bomba de ZIP:** cada parte é descomprimida com `take` até o teto: 64 MiB
  por parte e 256 MiB no total (`V`), ou 4 MiB no Autoview. Acima disso, a
  parte é cortada, e a nota diz isso. O tamanho declarado no diretório do ZIP
  não é usado para reservar memória.
- **Entradas:** no máximo 10.000 entradas no pacote. Nomes com `..`, absolutos
  ou repetidos são ignorados e contados na nota.
- **XML:** `quick-xml` lê sem DTD e sem expandir entidades definidas no
  arquivo, então não há XXE nem *billion laughs*. Um `<!DOCTYPE` faz a parte
  ser recusada. Só as cinco entidades do XML e as referências numéricas valem.
- **Profundidade de XML:** no máximo 256 níveis abertos. Tabela dentro de
  tabela é achatada a partir do terceiro nível.
- **Tempo:** o mesmo prazo do Markdown, conferido a cada 4.096 eventos de XML.
  Ao estourar, mostra o que tiver, marcado como cortado.
- **O que chega à tela:** todo texto passa por `Line::push` **depois** de
  decodificar as entidades, porque `&#27;` e `&#x202E;` só viram caractere ali.
- **Pânico** vira hex, por `panic_guard::catch`.
- **Autoview:** só se o arquivo couber em 4 MiB, porque o diretório do ZIP fica
  no fim e o arquivo precisa ser lido inteiro. Acima disso, a nota diz `V to
  view`.
- proptest: ZIPs montados com partes aleatórias, XML aleatório e aninhamento
  fundo nunca dão pânico, nunca passam dos tetos e nunca levam controle à
  tela.

## Etapas (um PR cada)

1. **Entrada de eventos no `markdown::Builder`**, sem mudar nada no Markdown
   (testes atuais iguais).
2. **DOCX:** texto, títulos, listas, tabelas, links, imagens, e o cabeçalho
   com metadados, macros, relações externas, revisões e oculto.
3. **ODT.**
4. **PPTX e ODP,** por slide, com as notas.

## Questões abertas

| # | Questão | Proposta |
|---|---|---|
| ?1 | Revisões: mostrar o texto como ficou, ou com inserções e remoções marcadas? | marcadas, sempre |
| ?2 | Texto oculto: mostrar marcado ou só contar no cabeçalho? | mostrar marcado |
| ?3 | Os metadados abrem visíveis ou recolhidos numa linha? | visíveis, num bloco curto |
| ?4 | `.doc` e `.ppt` antigos entram? | não agora |

## Para depois

- `.doc`, `.ppt` e `.rtf`.
- EPUB, que também é ZIP, mas o conteúdo é XHTML (o HTML no viewer está fora
  por decisão).
- Extrair um objeto embutido para uma pasta, com uma ação explícita.
