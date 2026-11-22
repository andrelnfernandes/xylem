# Plano: Norton Guides (`.ng`) no viewer

**Feito na 0.9.0** (pedido em 2026-11-20; PR #128).

## Objetivo

Os guias Norton Guides (`.ng`) e Expert Help são a documentação de uma época:
as bibliotecas Clipper, o próprio Clipper, linguagens e ferramentas de DOS. Quem
os guarda costuma ser o mesmo público que veio do XTree. Hoje um `.ng` abre em
hex no Xylem.

O viewer passa a abri-los como o guia que são: o menu do topo, as listas de
tópicos, o texto de cada tópico com "see also", e a navegação entre eles, com
as buscas do viewer.

## Agradecimento e referências

O formato só é conhecido hoje porque **Dave Pearson** o decifrou nos anos 90 e
vem mantendo os Norton Guides legíveis desde então, com leitores e conversores
para cada época. O site dele reúne tudo: <https://www.davep.org/norton-guides/>.

Referências:

- as notas do formato: <https://www.davep.org/norton-guides/file-format>
  (incompletas, segundo o próprio autor);
- `ngdb.py`, a biblioteca em Python (<https://github.com/davep/ngdb.py>,
  GPL-3.0): a referência mais completa e atual;
- `ng2html`, o conversor para HTML em C (<https://github.com/davep/ng2html>,
  GPL-2.0), dos anos 90;
- `aging`, o leitor para o terminal (<https://github.com/davep/aging>, GPLv3+),
  feito com a `ngdb.py`.

**O agradecimento vai para o produto**, não só para este plano:

- a seção "Norton Guides" do manual (`viewer.md`) termina com este parágrafo
  (em inglês, como o manual):

  > The format is known thanks to Dave Pearson, who worked it out in the
  > 1990s and has kept Norton Guides readable ever since:
  > <https://www.davep.org/norton-guides/>. Xylem's reader is its own code,
  > written from his description of the format; it includes no code from his
  > readers.

- a página de comparações (`comparison.md`, "A thank-you") ganha uma linha
  sobre ele, porque é onde o manual agradece a quem o Xylem deve algo; ali,
  só o agradecimento, sem a frase sobre o código;
- as notas da versão que trouxer o formato também citam o nome e o site;
- o comentário do topo do módulo Rust cita as mesmas fontes e repete que o
  código é próprio, escrito a partir da descrição do formato.

A frase sobre o código não é exigência legal: está ali porque o agradecimento
cita três projetos GPL num programa MIT, e quem lê poderia supor que o código
deles veio junto.

Nas traduções, o nome e o link ficam iguais.

**Licença**: os três projetos são GPL, e o Xylem é MIT. O leitor do Xylem é
código próprio, escrito a partir da descrição do formato. Os testes usam guias
montados pelo próprio teste, nunca os guias de exemplo desses projetos.

## O formato

A estrutura, a partir das notas do formato, com o que elas não dizem
completado na seção seguinte:

- **Números** em little-endian: palavras de 2 bytes, longos de 4.
- **Cabeçalho**, sem cifra, 378 bytes: a assinatura (`NG` para Norton Guide,
  `EH` para Expert Help), 4 bytes desconhecidos, a quantidade de menus
  (palavra), o título (40 bytes) e cinco linhas de créditos (66 bytes cada),
  terminados em NUL só quando são mais curtos que o campo.
- **Tudo depois do cabeçalho** é cifrado com XOR `0x1A` em cada byte, inclusive
  os números.
- **Cada registro** começa com o tipo (palavra) e um tamanho (palavra), seguidos
  de 22 bytes fixos; o tamanho conta o que vem depois desses 26 bytes. Pular
  um registro é avançar 26 + tamanho. Tipos: 0 entrada curta, 1 entrada longa,
  2 menu; qualquer outro encerra a leitura dos menus.
- **Os 22 bytes fixos de uma entrada**: a quantidade de linhas, a marca de "see
  also", a linha da origem, o deslocamento da origem (longo), o menu e a opção
  que levam a ela, e os deslocamentos da anterior e da próxima (longos). O valor
  `0xFFFF` numa palavra e `0xFFFFFFFF` num deslocamento querem dizer "nenhum".
- **Menus**: vêm logo depois do cabeçalho, um após o outro. A quantidade de
  opções é a palavra lida **menos um**; depois, 20 bytes desconhecidos, um
  deslocamento por opção, `(opções + 1) × 8` bytes desconhecidos, o título, as
  opções (cada uma terminada em NUL) e um byte desconhecido. A primeira entrada
  do guia começa onde o último menu termina.
- **Entrada curta**: para cada linha, **6 bytes** (2 desconhecidos e o
  deslocamento da entrada para onde a linha leva), e depois as linhas.
- **Entrada longa**: as linhas; se a marca de "see also" não for zero, a
  quantidade de "see also" (palavra), um deslocamento para cada um e os textos.
  O manual do compilador do Expert Help limita a 20 "see also"; acima disso,
  o guia está corrompido.
- **Texto**: linhas terminadas em NUL. `0xFF` seguido de um byte `n` vira `n`
  espaços; `0xFF 0xFF` vira um espaço; `0xFF` no fim da linha, um espaço. Nos
  títulos e opções de menu e nos "see also", `0xFF` é só um espaço (alguns
  autores o usaram como espaço que não quebra).
- **Formatação** dentro do texto, com `^`: `^B` liga e desliga negrito, `^U`
  sublinhado, `^R` vídeo reverso, `^N` volta ao normal, `^A` seguido de dois
  dígitos hex é uma cor de DOS (frente nos 4 bits baixos, fundo nos altos; o
  mesmo atributo de novo volta ao normal), `^C` seguido de dois dígitos hex é
  um caractere pelo código, `^^` é um `^`. Maiúsculas e minúsculas valem igual.
  Um `^A` ou `^C` sem hex válido aparece como texto; um `^` seguido de outra
  letra some, e a letra aparece; um `^` no fim da linha é ignorado.
- **Caracteres**: CP437, **incluindo os glifos de 1 a 31** (☺ ☻ ♥ ♦ ... ▲ ▼) e o
  127 (⌂), que os guias usam como desenho; o 0 vira espaço. O
  `src/charset.rs` hoje só traduz de 128 a 255: a tabela dos 32 primeiros
  entra nesta etapa, só para este formato (no texto comum eles continuam sendo
  caracteres de controle, mostrados como `·`).

## Casos que a página do formato não cobre

A página do formato é incompleta, como o próprio autor avisa. Os fatos abaixo
completam a estrutura, e onde há mais de uma leitura possível, fica a escolha
do Xylem.

**Estrutura**

- **Ordem dos bytes**: little-endian (arquivos de DOS, em x86).
- **Cabeçalho**: sem cifra; só o que vem depois dele é cifrado.
- **"Length" do registro**: conta o que vem depois dos 26 bytes fixos; pular um
  registro é avançar 26 + tamanho.
- **Entrada curta**: cada linha tem 6 bytes antes do texto (2 desconhecidos e o
  deslocamento), não os 4 de um `long` só que a página sugere. Os 8 bytes
  "unknown" do cabeçalho da entrada ficam no lugar de "anterior" e "próxima"
  da entrada longa, e numa entrada curta não se usam.
- **Contagem de opções do menu**: a palavra gravada, menos um, dá a quantidade
  de ponteiros e de opções; os bytes desconhecidos são `8 ×` a palavra gravada.
  **Hipótese**: a palavra gravada conta o título do menu como um item, que tem
  os seus 8 bytes, mas não ponteiro. A conferir nos guias reais.
- **Depois do último texto do menu** há um byte desconhecido, que é pulado.
- **Fim dos menus**: o cabeçalho diz quantos são, mas o leitor não confia só
  nisso. Para no primeiro registro que não seja menu, ou na quantidade do
  cabeçalho, o que vier antes.
- **"See also"**: no máximo 20 (o limite do manual do compilador do Expert
  Help).

**Texto**

- **`0xFF` seguido de `n`**: `n` espaços, com `n` lido **sem sinal** (de 0 a
  255); `n = 0` não acrescenta nada.
- **`0xFF 0xFF`**: um espaço.
- **`0xFF` no fim da linha**: um espaço, e a leitura para no fim da linha.
- **`0xFF` em títulos, opções de menu e "see also"**: um espaço comum (alguns
  autores o usaram como espaço que não quebra).
- **Tamanho de uma linha**: até 1.024 bytes antes de expandir. Sem NUL dentro
  do limite, a linha acaba no limite, e o resto do registro continua a ser lido
  pelo tamanho do registro, não pelo texto.
- **Títulos e opções**: as notas falam em 40 e 50 caracteres, mas há guias com
  mais; o leitor aceita até 128 e corta o resto com aviso.

**Formatação**

- **Os códigos `^`** (`^A ^B ^C ^N ^R ^U ^^`) não estão nas notas do formato,
  que tratam só da estrutura: são, provavelmente, a marcação da fonte do guia,
  mantida no texto compilado.
- **`^B`, `^U` e `^R`**: cada um liga e desliga sozinho, e podem se somar
  (negrito sublinhado); `^N` desliga todos.
- **`^A` com o mesmo atributo de novo** volta ao normal. A conferir nos guias
  reais.
- **Caracteres**: CP437 com os glifos de 1 a 31 e o 127, que os guias usam
  para desenhar molduras e setas; o 0 vira espaço.

**Guias corrompidos**

- **Mais de 20 "see also"**: a lista é dada como corrompida e não é lida; o
  texto da entrada aparece, com o aviso.
- **Contagem de opções do menu igual a zero**: o "menos um" não pode virar um
  número enorme; o menu fica vazio, com o aviso.

### Ainda em aberto

- Os campos desconhecidos (4 bytes do cabeçalho, 20 e os `8 ×` contagem dos
  menus, 2 por linha das entradas curtas): ignorados.
- O `^A` repetido voltando ao normal, a confirmar vendo um guia que use cores.

**Conferido em 2026-11-20, com os 42 guias da coleção pública** (etapa 1):
todos abrem sem nenhum problema apontado, e nenhum dos cerca de 30 mil links
(opções de menu, linhas de entradas curtas e "see also") aponta para fora de
uma entrada. Isso confirma a contagem do menu "menos um" e o pulo de 26 +
tamanho bytes também nos menus. O maior guia (6,2 MB, 9.352 entradas) abre em
0,56 s no build de release, com 167 MB de memória; um guia típico, em menos de
0,1 s.

**Feito na etapa 1**: o guia inteiro vira um documento só, com as entradas
numeradas (`§12`) e cada link dizendo para onde leva (`→ §12`); a busca do
viewer serve de navegação.

**Feito na etapa 1b** (decisão do autor: navegar, sem digitar `§`): o guia
continua um documento só, para que a busca cubra o guia inteiro, e os links
passam a ser seguidos. Tab e Shift+Tab escolhem o link (a partir da tela,
quando o escolhido saiu dela), Enter vai à entrada com o primeiro link dela
escolhido, Backspace volta (até 1.000 passos); um clique segue o link, e o
`Backspace` do rodapé, clicável, volta, porque o terminal não informa o botão
"voltar" do mouse. No guia, Enter nunca fecha o viewer. Por dentro: o `Doc`
ganhou saltos (`Jump`: linha, caracteres, destino) e lugares (títulos com id),
só em documentos feitos por `from_events_linked`; um arquivo Markdown nunca
os tem, e um link dele continua mostrando o endereço real. O link mantém a
aparência do texto do guia e só o escolhido aparece em destaque. Um link
quebrado em duas linhas da tela só é clicável na primeira.

Guias reais para conferir: a coleção pública de Dave Pearson
(<https://github.com/davep/norton-guides>, com guias de domínio público, de
licença permissiva ou publicados com permissão), lida da cópia local do autor,
nunca copiada para o repositório do Xylem. Os testes automáticos continuam com
guias montados pelo próprio teste.

## Como aparece

- **O quadro do topo**: o título e os créditos do guia, e se é Norton Guide ou
  Expert Help.
- **Os menus**: uma barra com os títulos dos menus, como no guia original; a
  letra ou ← → escolhe o menu, ↑ ↓ a opção, Enter abre.
- **Uma entrada curta**: a lista de linhas, cada uma um link; Enter segue.
- **Uma entrada longa**: o texto, com negrito, sublinhado e reverso pelos
  estilos do tema; no fim, os "see also" como links. Uma tecla vai para a
  "próxima" e a "anterior" do guia (as do próprio formato).
- **Voltar**: uma pilha de onde se veio. Backspace volta um passo, e Esc fecha
  o guia. Nunca há um laço sem saída: a pilha tem teto, e o mais antigo sai.
- **Buscar**: a busca do viewer (`F`, `/`, `B`...) vale dentro da entrada
  aberta. Uma busca no guia inteiro (todas as entradas longas) fica para a
  etapa 2, com progresso e Esc.

Decidido na etapa 1: em vez da barra de menus e de uma entrada por vez, o
guia é um documento só, com links internos próprios do viewer, com as teclas
do F1 (Tab, Shift+Tab, Enter, Backspace). Assim a busca do viewer já cobre o
guia inteiro, e a etapa 2 fica quase toda feita (ver "Feito na etapa 1b").

## Reconhecimento

Pela extensão `.ng` **e** pela assinatura (`NG` ou `EH` nos dois primeiros
bytes). Só a assinatura, sem a extensão, não basta: dois bytes casam por acaso
com muitos arquivos. Um `.ng` sem a assinatura abre como hoje.

## Segurança

O guia é uma rede de ponteiros vinda de fora, e cada um deles pode mentir.

1. **Todo deslocamento é conferido** contra o tamanho do arquivo antes de ser
   lido; um ponteiro que aponta para fora, para o cabeçalho ou para o meio de
   outro registro de tipo desconhecido vira um link morto, marcado, e a
   navegação continua.
2. **Toda contagem é conferida** contra o tamanho do próprio registro: a
   quantidade de linhas, de opções e de "see also" não pode exigir mais bytes do
   que o registro tem. Um registro que mente é mostrado até onde dá, com o aviso.
3. **A expansão de espaços tem teto** por linha (por exemplo, 4.096
   caracteres, a partir de no máximo 1.024 bytes lidos) e por entrada: dez bytes
   não viram um gigabyte. Os "see also" têm
   o teto de 20, e as opções de menu, o de 128 caracteres.
4. **Ciclos**: a navegação é feita pelo usuário, um passo de cada vez, então um
   ciclo só cansa. A busca no guia inteiro (etapa 2) e qualquer índice visitam
   cada deslocamento uma vez, com um conjunto de vistos.
5. **Limites gerais**: o arquivo inteiro até o limite de leitura do viewer (um
   guia típico tem de dezenas de KiB a poucos MiB); no máximo 100.000 registros;
   e o prazo de leitura do viewer.
6. **Texto como texto**: os códigos `^B`, `^U`, `^R`, `^N` e `^A` viram
   estilos do tema (a cor de DOS do `^A` é traduzida para a paleta do tema, não
   enviada crua); os bytes de 1 a 31 viram os glifos do CP437, que são
   caracteres comuns; o `^C` passa pela mesma tabela. Nenhum byte do guia chega
   ao terminal como controle.
7. **O parser corre sob o `panic_guard`**, como os do Office e dos ebooks.

Nada no formato executa código, e o Xylem não acrescenta nada que execute.

## Onde entra

- No viewer (`V`) e no Autoview (F7: o quadro do topo e o primeiro menu).
- Na coluna `type` do `W`: `Norton Guide` e `Expert Help guide`.
- Na seção de formatos do manual (`viewer.md`, "Norton Guides"), nas notas da
  versão.

## Etapas

1. **O guia**: o cabeçalho, os três registros, a cifra, os espaços, os
   códigos de formatação, os menus, as entradas, os links e o voltar. Testes com
   guias montados no teste: um guia mínimo; ponteiros para fora; contagens que
   mentem; uma bomba de espaços; um ciclo; um arquivo cortado no meio; um
   `.ng` sem a assinatura; mais de 20 "see also"; uma quantidade de opções de
   menu igual a zero (que, menos um, não pode virar um número enorme); os glifos
   de 1 a 31; `0xFF 0xFF` e `0xFF` no fim da linha; `^A` e `^C` sem hex.
   Depois, a conferência com os guias da coleção pública: todos abrem, os menus
   e os "see also" levam a entradas válidas, e as duas hipóteses (contagem do
   menu, `^A` repetido) ficam confirmadas ou corrigidas neste plano.
2. **A busca no guia inteiro**, com progresso, Esc e o conjunto de vistos.
3. A seção do manual (em inglês) com o agradecimento a Dave Pearson e o link
   para o site dele, a linha em "A thank-you" da página de comparações, as notas
   da versão, a coluna `type`.

## Fora deste plano

- Criar ou editar guias (o compilador `ngc` e as fontes `.ngo`).
- Outros formatos de ajuda da mesma época (`.hlp` do Windows, `.inf` do OS/2):
  outro plano, se houver pedido.
- Exportar um guia inteiro para HTML ou Markdown: talvez pelo Batch, depois.
