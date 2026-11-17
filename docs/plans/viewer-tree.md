# Plano: JSON, YAML e TOML como árvore no viewer

Origem: conversa de 2026-11-16 (veja [viewer-logs.md](viewer-logs.md)).
**Spec proposto, ainda não aprovado.**

## Objetivo

Um arquivo de dados estruturados abre como uma árvore que se dobra: as chaves e
os valores um por linha, com recuo, o caminho do nó atual no rodapé e a busca
que abre o caminho até o que achou. Um JSON minificado de uma linha só fica
legível sem programa externo.

Hoje esses arquivos abrem em texto, com a cor do `syntect`. O JSONL continua
como tabela, sem mudança.

## Formatos

- **JSON:** `.json`, `.geojson`, `.har`, `.webmanifest`, `.jsonc` e `.json5`
  (estes dois aceitam comentários e vírgula sobrando; o JSON5 completo fica para
  depois).
- **YAML:** `.yaml` e `.yml`, inclusive com vários documentos (`---`).
- **TOML:** `.toml`, inclusive `Cargo.lock` e `pyproject.toml`.
- Só pela extensão, também dentro de arquivo comprimido (`a.json.gz`), como o
  Markdown faz hoje.

## Modo e teclas

- Um modo novo, `ViewMode::Tree`. `H` faz Árvore → Texto → Hex, e `T` alterna
  árvore e fonte mantendo o lugar, como no Markdown.
- Ao abrir, a árvore vem toda aberta se tiver até 2.000 nós. Com mais, só os dois
  primeiros níveis (?1).
- Teclas da árvore:
  - **Right** abre o nó. **Left** fecha o nó ou, se já está fechado, vai ao pai,
    como na árvore de pastas.
  - **Enter** abre ou fecha. Hoje o Enter fecha o viewer, mas na tabela ele já
    abre o registro, e aqui seguiria a tabela.
  - **`*`** abre tudo abaixo do nó, com o mesmo teto de nós da abertura.
  - **`k`** copia o caminho do nó (`.servers[2].name` em JSON, `servers.2.name`
    em YAML e TOML) (?2).
- O rodapé mostra o caminho do nó atual. A margem mostra a linha do nó na
  fonte, como o `#` do Markdown.
- **Busca** (`F`, `B`, `/`): procura em chaves e valores, inclusive dentro de
  nós fechados, e abre o caminho até o que achou.
- **`&`, `g` e Ctrl+B** passam para o texto, como no Markdown, porque trabalham
  com linhas.

## Como cada linha aparece

- `chave: valor` para escalares. Um nó fechado mostra o resumo: `{12 keys}`,
  `[340 items]`.
- Texto de string: já sem escape (`\n` vira `↵`, `\t` vira espaço), cortado na
  largura da tela, com `…`. O valor inteiro aparece numa linha só ao andar para
  a direita, como no Markdown.
- **Número como está na fonte**, sem passar por `f64`: `12345678901234567890`
  não vira `1.2345678901234567e19`.
- Cores por tipo (string, número, booleano, nulo, chave) e o tipo de TOML que o
  YAML não tem (data e hora). `c` desliga as cores.
- YAML: âncora e alias aparecem como estão (`&base`, `*base`) e o alias **nunca
  é expandido** (veja Segurança). Uma tag (`!!binary`, `!Ref`) aparece antes do
  valor.
- **Chave repetida** num mesmo objeto aparece duas vezes, marcada `dup key`, e o
  cabeçalho conta quantas. Parsers diferentes ficam com a primeira ou com a
  última, e essa diferença já foi usada para esconder valores (o
  CVE-2017-12635 do CouchDB, por exemplo). Aqui as duas ficam à vista.

## Parsers

| Formato | Leitor | Por quê |
|---|---|---|
| JSON | próprio, em `src/viewer/tree/json.rs`: tokenizador sem recursão, com a posição de cada nó | o `serde_json` não dá a posição na fonte (para `T` e a margem) e converte números; um tokenizador de JSON é pequeno e fácil de cobrir com proptest |
| YAML | `saphyr-parser` (só o parser de eventos, sem montar documento) | Rust puro, ativo, dá a posição (`Marker`) de cada evento; o `serde_yaml` foi arquivado e o `serde_yml` tem aviso de segurança (RUSTSEC-2025-0068) |
| TOML | `toml_edit`, que já é dependência | guarda a posição (`span`) de cada item |

Nova dependência: só o `saphyr-parser`, que precisa de revisão (princípio 7 de
[viewers.md](viewers.md)), inclusive do `unsafe`, se houver.

## Estrutura

- Os nós ficam num `Vec` plano, sem `String` por nó: tipo, faixa da chave e do
  valor na fonte (bytes), pai, primeiro filho, próximo irmão, número de filhos e
  linha na fonte. São cerca de 40 bytes por nó, e o texto é lido da própria
  fonte quando a linha é desenhada.
- As linhas visíveis são calculadas a partir de quais nós estão abertos. Abrir
  ou fechar recalcula só o trecho abaixo do nó.
- Nada é recursivo: a descida é uma pilha num `Vec`, limitada pela profundidade.

## Segurança

- **Bomba de aliases do YAML (*billion laughs*):** nenhum alias é expandido, e
  o custo fica linear no tamanho do arquivo. É a razão de usar o parser de
  eventos, e não um que monta o documento.
- **Profundidade:** no máximo 512 níveis. Acima disso, o nó mostra `…too deep`
  e o resto daquele ramo não entra. Isso vale para os três parsers. O
  `toml_edit` e o `saphyr-parser` são recursivos por dentro, então a entrada
  com aninhamento passa antes por uma contagem de `[`, `{` e de recuo, sem
  recursão, que recusa a árvore e deixa o texto se a profundidade passar do
  limite.
- **Tamanho:** a árvore só é montada até 64 MiB de fonte (`V`) ou 1 MiB
  (Autoview). Acima disso, o arquivo abre em texto, com a nota `Too large for
  the tree: showing the text`. O teto de nós é 16 por KiB de fonte.
- **Tempo:** o mesmo prazo do Markdown (3 s no `V`, 100 ms no Autoview),
  conferido a cada 4.096 eventos. Ao estourar, aparece o que foi lido e a nota
  diz que a árvore está cortada.
- **Pânico** vira modo texto, por `panic_guard::catch`.
- **O que chega à tela:** todo texto de chave e de valor passa pela mesma
  limpeza do texto (controle vira `·`, bidi marcado e contado) **depois** de
  tirar o escape, porque `\u001b` e `‮` só viram caractere ali. Um escape
  inválido (`\ud800` sozinho) aparece literal e marcado.
- **Autoview:** os três parsers são Rust e podem ser interrompidos, então valem
  no Autoview, com os limites de lá.
- proptest: bytes aleatórios e entradas montadas (aninhamento fundo, aliases em
  cadeia, chaves repetidas, escapes inválidos) nunca dão pânico, nunca passam de
  512 níveis e nunca levam controle à tela.

## Etapas (um PR cada)

1. **O modo `Tree` e o JSON:** estrutura de nós, teclas, busca, `T`, `k`, chave
   repetida e limites.
2. **TOML,** com `toml_edit`.
3. **YAML,** com `saphyr-parser` e vários documentos.
4. **JSONC e JSON5 básico** (comentários e vírgula sobrando).

## Questões abertas

| # | Questão | Proposta |
|---|---|---|
| ?1 | Como a árvore abre | toda aberta até 2.000 nós; acima, dois níveis |
| ?2 | Tecla para copiar o caminho do nó | `k` (está livre; `y` já copia o caminho do arquivo) |
| ?3 | YAML e TOML abrem na árvore ou no texto? Arquivos de configuração são lidos como texto com frequência | todos abrem na árvore, como o Markdown abre renderizado; `T` volta ao texto, e a escolha é lembrada por extensão na sessão |
| ?4 | Enter na árvore abre e fecha o nó, em vez de fechar o viewer | sim, como na tabela |

## Para depois

- Um array de objetos parecidos aberto como tabela (com SQL e Batch), pela
  mesma leitura do JSONL.
- JSON5 completo (chaves sem aspas, strings com aspas simples, hexadecimal).
- Filtrar a árvore por caminho (`.servers[*].name`), à maneira do `jq`, sem
  executar nada.
- Notebooks do Jupyter (`.ipynb`), que são JSON, como documento (item 7 da
  conversa de 2026-11-16).
