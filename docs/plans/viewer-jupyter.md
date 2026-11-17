# Plano: notebooks do Jupyter no viewer

Origem: conversa de 2026-11-16 (veja [viewer-logs.md](viewer-logs.md)).
**Spec proposto, ainda não aprovado.**

## Objetivo

Um `.ipynb` abre como documento, célula por célula: o Markdown renderizado, o
código com a cor da linguagem e as saídas em texto. Hoje ele abre como um JSON
enorme numa linha só, que é ilegível, e por isso o notebook quase nunca é lido
fora do Jupyter.

## Como o documento é montado

- Pelo `markdown::Builder` com eventos (etapa 1 de
  [viewer-office.md](viewer-office.md)). Nenhuma dependência nova: o JSON é lido
  com o `serde_json`, que já está no projeto.
- **Cabeçalho:** kernel, linguagem, versão do `nbformat` e o número de células
  de cada tipo.
- **Célula Markdown:** o texto passa pelo mesmo parser do Markdown e entra no
  documento. Aqui interpretar é o certo, porque a célula é Markdown. Os
  anexos da célula (`attachments`) aparecem como `[image: nome]`.
- **Célula de código:** um bloco de código com a cor da linguagem do notebook
  (`metadata.language_info.name`, ou a do kernel), e o rótulo `In [n]` com o
  contador de execução (ou `In [ ]` se não foi executada).
- **Saídas**, depois do código, marcadas `Out [n]`:
  - `stream`: o texto. O `stderr` vem marcado.
  - `execute_result` e `display_data`: o `text/plain`. O `text/markdown`, se
    houver, é renderizado no lugar do texto puro.
  - Imagem: `[image/png, 34 KiB]`. **HTML, JavaScript, SVG, LaTeX e widgets:**
    `[text/html, 12 KiB]`, nunca desenhados nem executados.
  - `error`: o nome e a mensagem, e o traceback em seguida. O traceback vem com
    ANSI, que passa pelo mesmo filtro de [viewer-logs.md](viewer-logs.md): só a
    cor fica.
- **Célula raw:** o texto literal, marcado `raw`.
- `T` mostra a fonte em JSON (e a árvore de [viewer-tree.md](viewer-tree.md),
  quando houver), `H` o hex. A busca procura no documento, inclusive nas
  saídas.

## Segurança

- **Nada é executado:** nem o código, nem o JavaScript das saídas, nem os
  widgets. É o mesmo princípio do "notebook não confiável" do próprio Jupyter,
  levado ao extremo: aqui nenhum notebook é confiável.
- **Tamanho:** até 64 MiB no `V` e 1 MiB no Autoview. As saídas com imagens em
  base64 é que costumam deixar o arquivo grande. O JSON é lido sem montar os
  valores das imagens: um `Visitor` do `serde` pula as chaves de mídia binária
  e guarda só o tamanho.
- **Profundidade:** o `serde_json` tem limite de recursão (128). Um notebook
  mais fundo do que isso abre como texto, com a nota.
- **Tetos:** 100.000 células e 1 MiB de texto por saída. Acima disso, corta e
  diz.
- **Tempo:** o prazo do Markdown (3 s no `V`, 100 ms no Autoview), conferido
  entre as células e dentro do parser de Markdown de cada uma.
- **O que chega à tela:** tudo passa por `Line::push` depois do JSON e do
  Markdown, com a contagem bidi. É o mesmo caminho do Markdown de hoje.
- **Pânico** vira texto, por `panic_guard::catch`.
- proptest: notebooks montados com células e saídas aleatórias, tipos trocados
  (um número onde se espera texto) e JSON quebrado nunca dão pânico.
- **Formato antigo:** o `nbformat` 3 (com `worksheets`) é lido. Versões mais
  antigas ou desconhecidas abrem como texto, com a nota.

## Etapas

Um PR só, depois da etapa 1 de [viewer-office.md](viewer-office.md) (a entrada
de eventos no `Builder`) e da etapa 3 de [viewer-logs.md](viewer-logs.md) (o
filtro de ANSI), que ele reaproveita.

## Questões abertas

| # | Questão | Proposta |
|---|---|---|
| ?1 | Mostrar as saídas, ou só o código e o Markdown? | mostrar sempre; uma tecla para escondê-las fica para depois, se houver pedido |
| ?2 | `text/markdown` das saídas é renderizado? | sim, como as células Markdown |

## Para depois

- Esconder e mostrar as saídas com uma tecla.
- Os metadados de cada célula (tags, `collapsed`).
- Outros formatos de notebook (R Markdown, Quarto `.qmd`), que já são quase
  Markdown.
