# O visualizador

`V` abre o arquivo no visualizador (o Autoview, F7, o mostra ao lado da lista).
Arquivos binários abrem em hexadecimal. Um arquivo grande no disco não é lido inteiro: o visualizador lê o que a tela mostra conforme ela anda (veja [Arquivos grandes](#arquivos-grandes)). O que não pode ser lido assim mostra só o começo, até o limite de tamanho.
Planilhas, CSV, DBF, SQLite e Parquet abrem como tabela (veja [Tabelas](tables.md)).

| Tecla | O que faz |
|---|---|
| ↑ ↓ PgUp PgDn Home End | rola verticalmente |
| ← → | rola horizontalmente |
| `H` | próximo modo: texto e hexadecimal; num arquivo Markdown, renderizado, texto e hexadecimal |
| `O` | próxima codificação (UTF-8, CP850, 1252, CP437, 1251, KOI8-R, UTF-16) |
| `C` | ativa ou desativa as cores de sintaxe |
| `W` | ativa ou desativa a quebra de linha automática |
| `#` ou `L` | ativa ou desativa os números de linha (no modo texto) |
| `g` ou `:` | pula para uma linha (um deslocamento de byte em hexadecimal, uma linha da grade no modo tabela) |
| `&` | mostra só as linhas que combinam, texto simples ou regex ([filtrar linhas](filter.md)); no modo tabela, o filtro SQL |
| Ctrl+B | uma linha de saída por registro ou linha mostrada ([Batch no visualizador](execute.md)) |
| `E` | edita o arquivo; de volta ao Xylem, o visualizador mostra o novo conteúdo na mesma posição (arquivos binários somente com um editor em [`[editors]`](execute.md)) |
| `D` | com Git ativo: mostra o que mudou desde o último commit, e volta ([Git](git.md)) |
| `T` | tabela ou texto, num arquivo que abriu como tabela; renderizado ou fonte, num arquivo Markdown |
| `U` | na visão dos arquivos marcados (Ctrl+V): desmarca o arquivo |
| `y` ou Ctrl+Insert | copia o caminho do arquivo para a área de transferência |
| Esc | fecha |

## Busca

**F** busca desde o início, **B** de trás para frente a
partir do fim, **/** (ou `S`) a partir da página atual e **\\** para trás a
partir dela; **Space** continua na mesma direção, **+** e **-** escolhem a
direção. A ocorrência é destacada. No prompt de busca, **F2** alterna a
diferenciação de maiúsculas e minúsculas (ao ignorar maiúsculas/minúsculas,
isso vale também para letras acentuadas: `AÇÃO` encontra `ação`) e **F4** alterna entre
texto e hexadecimal (pares de bytes como `4d 5a`). Um `*` no meio do termo
corresponde a qualquer trecho na mesma linha (`error*disk`).

**F3** inverte a direção da busca, e assim o F sozinho faz o que o B faz: para a frente desde o início ou para trás desde o fim.

## Cores de sintaxe

Código-fonte e scripts são coloridos em cerca de 150 linguagens (Rust, Python,
shell, C, JavaScript, SQL, YAML, Markdown...), reconhecidas pela extensão, pelo
nome (`Makefile`) ou pela primeira linha (`#!/bin/bash`). O padrão é
`syntax_highlight` (página 2 da [configuração](configuration.md)). As cores
permanecem legíveis sobre o fundo do tema atual, e apenas as primeiras 20.000
linhas são coloridas, e nenhuma linha com mais de 10.000 caracteres, para que
arquivos enormes nunca deixem o visualizador lento.

## Markdown renderizado

Arquivos `.md` e `.markdown` abrem renderizados, também no Autoview, dentro de arquivos compactados e comprimidos (`x.md.gz`), e a moldura diz `RENDERED`. `T` mostra a fonte no mesmo lugar, e volta; `H` passa por renderizado, texto e hexadecimal.

Você vê títulos, negrito, itálico, riscado, código, blocos de código com cores de sintaxe (`C` as desliga), listas aninhadas e numeradas, listas de tarefas (`[x]`), citações, linhas horizontais e tabelas. Nada é baixado, e nenhum link abre coisa alguma:

- Um link mostra o endereço real ao lado do texto, como em `seguro <https://example.com>`, para que o texto não se passe por outro endereço.
- Uma imagem é uma linha de texto: `[image] legenda <foto.png>`.
- Um diagrama Mermaid é um bloco de código com o rótulo `mermaid`.
- HTML e comentários HTML continuam visíveis como texto.
- O bloco YAML no início de um arquivo é um bloco de código com o rótulo `yaml`.

O texto sempre cabe na janela, então `W` não faz nada aqui; ← → alcançam as colunas de uma tabela larga demais. A busca olha o texto da tela, incluindo os endereços, e encontra uma frase que continua na linha seguinte. `g` e `#` contam as linhas da fonte. Ctrl+B age sobre as linhas da fonte, todas elas. `&` passa para o modo texto e filtra lá.

Um arquivo acima de 8 MiB abre na fonte, com um aviso. A renderização também para no limite de tempo (3 s, 100 ms no Autoview) ou no seu limite de memória, e o cabeçalho então diz `(rendered only the start)`. Uma tabela mostra no máximo 256 colunas. Caracteres de controle aparecem como `·`, inclusive os escritos como entidades, como `&#27;`.

O próprio Markdown esconde algumas coisas: definições de link que nada usa e células de tabela além das colunas do cabeçalho. Para ver tudo, `T` mostra a fonte. Itálico e riscado dependem do terminal e da fonte, por isso também têm cor própria.

## JSON, YAML e TOML

Um `.json` (também `.geojson`, `.har`, `.webmanifest`, e `.jsonc` e `.json5`, com comentários e vírgula sobrando), um `.yaml` ou `.yml` e um `.toml` (também o `Cargo.lock`) abrem como uma árvore que se dobra, e a moldura diz `TREE: JSON`. Um arquivo minificado numa linha só fica tão legível quanto qualquer outro. Uma árvore de até 2.000 nós abre toda aberta; uma maior mostra o primeiro nível.

| Tecla | Ação |
|---|---|
| ↑ ↓ PgUp PgDn Home End | move o cursor |
| → | abre o nó (de novo: entra nele) |
| ← | fecha o nó, ou vai ao de cima |
| `Enter` | abre ou fecha o nó |
| `*` | abre tudo abaixo do nó |
| `k` | copia o caminho do nó: `.servers[2].name` em JSON, `servers.2.name` em YAML e TOML. Uma chave com fim de linha, caractere de controle ou de direção do texto vai entre aspas, com esses caracteres escapados (`\n`, `\u202e`), para que colar o caminho num shell nunca execute uma linha dele |

O rodapé mostra o caminho do nó sob o cursor. A busca (`F`, `B`, `/`) olha chaves e valores, também dentro de nós fechados, e abre o caminho até o que achou. `T` mostra o texto na linha do nó, e volta; `H` passa por árvore, texto e hexadecimal. `&`, `g`, `#` e Ctrl+B agem sobre as linhas do texto.

- Um nó fechado diz o que tem: `{12 keys}`, `[340 items]`.
- Um número aparece como o arquivo o escreve: `12345678901234567890` continua assim, e `1_000` no TOML também.
- Uma string mostra a quebra de linha como `↵`.
- Uma chave que aparece duas vezes no mesmo objeto é mostrada duas vezes, marcada `dup key`, e o cabeçalho conta quantas são. Os programas discordam sobre qual das duas vale, o que pode esconder um valor numa revisão.
- Uma âncora de YAML aparece como `&a1` e um alias como `*a1`: o alias nunca é expandido, então um arquivo feito para se multiplicar (um "billion laughs") não custa mais que o seu texto. Uma tag (`!Ref`) aparece antes do valor. Vários documentos ficam sob uma raiz, como `document 1`, `document 2`...

Um arquivo inválido, ou aninhado mais fundo que 512 níveis, abre como texto, e o aviso diz por quê e em que linha. A árvore é montada com o arquivo inteiro: um acima do limite de leitura abre como texto. A árvore também para em 2.000.000 de nós (cerca de 170 MiB de memória) ou quando montá-la demora demais; ela então tem só o começo do arquivo, e o cabeçalho diz `(tree of the start only)`. `T` continua mostrando o texto todo.

## Notebooks do Jupyter

Um `.ipynb` abre renderizado, célula por célula, e a moldura diz `RENDERED`. No alto: o kernel e quantas células há de cada tipo.

- Uma célula Markdown é renderizada como um arquivo Markdown.
- Uma célula de código mostra `In [n]:` e o código, com as cores da linguagem do notebook.
- As saídas vêm depois, marcadas `Out [n]:`: o texto de um stream (`stderr:` quando são os erros), o texto de um resultado (ou o Markdown dele, renderizado) e um erro com o seu traceback, sem os códigos ANSI do traceback.
- Uma saída de imagem, HTML, JavaScript, SVG ou LaTeX é uma linha que diz o que é e o tamanho, como `[image/png, 34 KiB: not shown]`. Nada num notebook é executado: nem o código, nem os scripts das saídas, nem os widgets.
- Uma célula raw aparece como está.

`T` mostra o JSON do notebook, e `H` os bytes. Um arquivo que não é um notebook que o viewer consiga ler (JSON quebrado, formato desconhecido) abre na fonte, com um aviso. O formato 3 (com worksheets) também é lido.

## Documentos do Office

Arquivos do Word (`.docx`, `.docm`, `.dotx`), de texto OpenDocument (`.odt`), do PowerPoint (`.pptx`, `.pptm`) e do Impress (`.odp`) abrem como documento: títulos, parágrafos, listas, tabelas, negrito, itálico e links, com o endereço real de cada link. Uma apresentação mostra cada slide sob `Slide n`, com as notas do apresentador depois. Notas de rodapé e comentários, com os autores, vêm no fim. Uma imagem é uma linha, como `[image: media/image1.png]`. O texto de um documento nunca é lido como Markdown.

Antes do texto, um bloco diz o que o documento traz além dele, já que muito disso não aparece no programa que o fez:

- quem o escreveu, quem o salvou por último, quando e com quê;
- `Contains macros`, quando tem. Elas nunca são lidas nem executadas;
- `Fetches from outside when it opens`: um modelo remoto, um objeto OLE ou um frame que apontam para fora do arquivo, com os endereços. Um modelo remoto é como ataques como o Follina (CVE-2022-30190) chegaram. Nada é baixado aqui;
- os objetos embutidos, por nome e tamanho, e quantos outros links externos há;
- revisões: o texto inserido aparece como `[inserted: ...]` e o apagado como `[deleted: ...]`, porque o que foi apagado costuma ser o que interessa;
- texto oculto (`[hidden: ...]`), texto branco e texto de um ponto (`[invisible: ...]`), e slides ocultos.

`H` mostra os bytes, e volta; `T` diz que o documento não tem texto próprio. Cada parte do arquivo é descomprimida só até um limite (64 MiB cada, 256 MiB no total; 4 MiB no Autoview), e o cabeçalho diz quando o documento foi cortado. Uma parte XML com DTD, ou aninhada mais fundo que 256 níveis, é recusada. Os antigos `.doc` e `.ppt` abrem em hexadecimal, como antes.

## Ebooks

EPUB (`.epub`, versões 2 e 3) e FictionBook (`.fb2`, e `.fb2.zip` com um único livro dentro) abrem como um livro para ler. Imagens não são mostradas: cada uma é uma linha, como `[image: cover.jpg]`.

**O livro.** Ele abre numa folha de rosto: o título, o autor, quem o publicou e quando, quantos capítulos e palavras tem, e o que o relatório aponta, se houver algo. O texto é uma coluna no meio da janela (70 células por padrão), com os parágrafos recuados como num livro impresso e o título de cada capítulo centralizado sob um ornamento. Uma quebra entre cenas (`* * *`) é desenhada como tal. O rodapé diz o capítulo em que você está, quanto já andou no livro, e o tempo que falta no capítulo e no livro.

| Tecla | O que faz |
|---|---|
| Space, PgDn | a próxima página (PgUp a anterior) |
| `]` `[` | o próximo ou o anterior capítulo |
| Tab | o sumário: ↑↓ escolhem, letras estreitam a lista, Enter vai, Esc fecha; Backspace depois volta para onde você estava |
| Enter | as notas chamadas na página, num quadro: ↑↓ o rolam, qualquer outra tecla o fecha |
| `g` | vai para uma linha, ou para uma parte do livro: `40%` |
| `R` | lembra onde o livro parou; `R` de novo esquece o livro |
| `I` | o relatório do arquivo, e de volta ao livro |
| `Z` | só o texto, sem a moldura e o rodapé; Esc volta |
| `/` `\` | busca para a frente ou para trás; `+` e `-` vão à próxima ou à anterior ocorrência (Space aqui vira a página) |
| `H` | os bytes, e volta |
| Esc | fecha |

As notas de um livro (um `aside` de tipo footnote no EPUB, o corpo `notes` de um FB2) não ficam no texto: cada chamada é um sobrescrito (`¹`), e Enter mostra as notas chamadas na página. No relatório elas ficam onde o livro as tem.

**Lembrar onde você parou.** `R` lembra o livro: quando você o fecha, o Xylem guarda o capítulo e a linha em que você estava, e na próxima vez o livro abre ali, dizendo `Back at chapter 7 (R forgets this book)`. O rodapé de um livro lembrado começa com `[R]`. O livro é reconhecido pelo que ele diz ser (seu identificador) e pelo tamanho, não pelo nome, então um livro movido ou renomeado continua reconhecido, e outro arquivo com o mesmo nome é outro livro. `R` de novo esquece o livro e apaga o que foi guardado dele. Um livro em que você nunca apertou `R` não deixa nada para trás.

**O tempo gasto lendo** é guardado para os livros lembrados com `R`, se você o ligar (**E** na página 5 da [configuração](configuration.md#reading-time)). A folha de rosto e o relatório então dizem há quanto tempo você lê o livro, em quantas sessões, até onde chegou e desde quando; o relatório lista as sessões. O rodapé mostra o tempo desta sessão.

**O relatório** (`I`) é o que o arquivo é antes do texto: o que o livro diz de si (título, autores, editora, idioma, data, identificador, assuntos, série, o programa que o fez), e o que ele traz além do texto, já que um livro de qualquer lugar é um arquivo de qualquer lugar:

- scripts: elementos `<script>`, `onload` e os outros atributos de evento, links `javascript:`, e as partes que um EPUB declara com scripts. Eles nunca são executados; um leitor que executa scripts os executaria;
- o que um leitor buscaria fora quando o livro abre: imagens, folhas de estilo, fontes e frames com um endereço na internet, listados. Buscá-los avisa a quem está lá que o livro foi aberto, e de onde. Nada é buscado aqui;
- formulários, frames e objetos embutidos, contados, e o número de links para fora;
- DRM: um EPUB cujos capítulos estão cifrados diz com quê (Adobe ADEPT, Readium LCP, Apple FairPlay) e não pode ser lido. Fontes ofuscadas, comuns em livros vendidos sem DRM, são só contadas;
- num EPUB, as partes que o pacote lista mas o arquivo não tem, as que ele tem mas não lista, e endereços que sobem para fora do pacote;
- num FB2, os arquivos embutidos nele (quantos, de que tipos e de que tamanho), nunca decodificados, e uma codificação declarada que o Xylem não conhece.

Depois vêm o sumário, como links (Tab e Enter os seguem), e o texto inteiro, com as notas onde o livro as tem.

**Ajustes** (página 5 da [configuração](configuration.md)): a largura da coluna (ou a janela inteira), parágrafos recuados e encostados ou separados por uma linha em branco, texto justificado, e a velocidade de leitura com que o tempo que falta é contado. Com o Nerd Fonts ligado (**M** na página 2), os ornamentos são folhas, a barra de progresso é desenhada com os glifos da própria fonte, e o rodapé e o sumário ganham ícones.

**O que é lido.** Um FB2 é lido na codificação que declara (UTF-8, UTF-16, Windows-1251, KOI8-R e as outras do `O`), e `T` mostra o XML dele. Um EPUB é um ZIP: cada parte é descomprimida só até um limite (64 MiB cada, 256 MiB no total; 4 MiB no Autoview, que diz `V shows it as a book` quando um livro é maior), e um FB2 é lido até 64 MiB, e o relatório diz quando só o começo do livro foi lido. Nenhum DTD é lido ou buscado, um XML com entidades próprias é recusado, o texto nunca entra como Markdown, e os endereços de um livro são tomados só como nomes das suas próprias partes, nunca como caminhos no disco. MOBI e AZW3 (Kindle), PDF e quadrinhos (CBZ, CBR) não são lidos aqui: `O` na janela de arquivos os abre em outro programa.

## Norton Guides

Um Norton Guide ou um guia do Expert Help (`.ng`), a ajuda em hipertexto do DOS e do Clipper, abre como documento. Um arquivo precisa do nome `.ng` e da assinatura do guia; qualquer outro abre como antes.

- **No alto:** o título e os créditos do guia, e o que o fez.
- **Os menus:** cada um com seus itens, e aonde cada item leva.
- **As entradas:** todas, na ordem do arquivo, cada uma sob um número: `§12`. Uma entrada de lista mostra suas linhas; uma entrada de texto mostra o texto com o layout do guia, e depois seus links de see-also. Todo link diz aonde vai (`→ §12`, `See also: Two() §3`), então uma busca por `§12` leva até lá. Um link que não aponta para nada diz `(broken link)`.

**Seguir links.** **Tab** e **Shift+Tab** escolhem o próximo ou o anterior link (a partir da tela, quando o escolhido saiu dela); **Enter** vai à entrada a que ele leva, com o primeiro link dessa entrada escolhido; **Backspace** volta para onde você estava, por até 1.000 links. Com o mouse ligado, um clique num link o segue, e um clique em `Backspace` no rodapé volta. Num guia, Enter nunca fecha o visualizador: o Esc fecha. Como o guia inteiro é um documento só, uma busca (`/`) olha ele todo.

Negrito, sublinhado e inverso saem nos estilos do tema; as cores de DOS de um guia não são enviadas ao terminal. Os caracteres do guia são da página de código 437, com os glifos que o DOS desenhava para os bytes 1 a 31 (`☺ ♥ ► ▲`); nada num guia chega ao terminal como caractere de controle.

Um guia é uma teia de deslocamentos que podem mentir: cada um é conferido contra o arquivo, cada contagem contra o seu próprio registro, e os espaços em que uma linha se expande têm um limite. O que está errado num guia é listado no alto, em "Problems in this guide", e o resto aparece mesmo assim. Um guia é lido inteiro, até 64 MiB; `H` mostra os bytes.

O formato é conhecido graças a Dave Pearson, que o decifrou nos anos 1990 e mantém os Norton Guides legíveis desde então: <https://www.davep.org/norton-guides/>. O leitor do Xylem é código próprio, escrito a partir da descrição que ele fez do formato; não inclui código dos leitores dele.

## Executáveis

Um executável, uma biblioteca ou um arquivo objeto abre como um relatório, seja qual for o nome: ELF (Linux, BSD), PE (`.exe`, `.dll`, `.sys` do Windows, também .NET) e Mach-O (macOS, também binários universais com várias arquiteturas). Ele é reconhecido pelos primeiros bytes, nunca pelo nome. O relatório diz:

- o que é (executável, biblioteca, objeto, core dump), para que máquina, 32 ou 64 bits, o ponto de entrada, e o interpretador, o subsistema ou o sistema mínimo;
- **proteções**, cada uma `yes` ou `no`, com `⚠` onde falta: no ELF, PIE, pilha que não executa código (NX), RELRO (total ou parcial), o canário da pilha, FORTIFY e os caminhos de biblioteca (RPATH, RUNPATH: um relativo ou um em `/tmp` é marcado); no PE, ASLR, ASLR de alta entropia, DEP, Control Flow Guard; no Mach-O, PIE e pilha executável;
- **assinatura**: se tem e de que tamanho. Ela **não é verificada**, e o relatório diz isso;
- **sinais** de algo escondido: uma seção de entropia alta (empacotada ou cifrada), o nome de um empacotador (UPX...), dados depois da última seção (um overlay, onde instaladores e alguns malwares guardam dados), uma seção ao mesmo tempo gravável e executável, um ponto de entrada fora de toda seção executável;
- **identidade**: build ID, debug link, UUID, caminho do PDB, o compilador, a versão do Go; a data de compilação de um PE, que o linker escreve e pode ser qualquer uma;
- as bibliotecas de que precisa, e as listas de seções, segmentos, importações, exportações e símbolos, de até 2.000 linhas cada.

Nada é executado ou carregado para fazer o relatório: nem o `ldd`, que em alguns casos executa o programa. `H` mostra os bytes, e volta. Um arquivo maior que o limite de leitura é lido só onde o relatório precisa, 256 MiB no máximo ao todo; o Autoview faz o relatório do começo que leu, e diz que ele pode deixar coisas de fora.

## Certificados e chaves

Um certificado, uma corrente ou um conjunto deles, um pedido de certificado (CSR), uma lista de revogação (CRL), uma chave privada ou pública, uma chave OpenSSH, `authorized_keys` e `known_hosts` abrem como relatório. O PEM é lido num arquivo com nome para isso (`.pem`, `.crt`, `.cer`, `.key`, `.csr`, `.crl`, `.pub`...) ou sem extensão (`id_ed25519`); o DER num `.der`, `.cer`, `.crt`, `.crl` ou `.csr` que seja um valor DER inteiro. Um bloco PEM dentro de código-fonte ou de um documento deixa esse arquivo como está.

- **Um certificado:** o subject e o issuer (e se é autoassinado), o número de série, a validade com os dias que faltam (vencido, ainda não válido, ou vencendo em até 30 dias ficam marcados `⚠`), a chave pública (RSA abaixo de 2048 bits e DSA são marcados), a assinatura (MD5 e SHA-1 são marcados), os nomes (um nome em punycode `xn--` é apontado), o uso da chave, o uso estendido, se é uma CA, os endereços da CRL e das informações da autoridade, uma extensão crítica desconhecida (marcada), e o fingerprint SHA-256.
- **Vários certificados:** o relatório diz se estão em ordem, cada um emitido pelo seguinte, conferido só pelos nomes e pelos identificadores de chave. **Nenhuma assinatura é verificada aqui**, e o relatório diz isso.
- **Uma chave privada:** só o tipo e o tamanho (RSA 2048 bits, EC P-256, PKCS#8 cifrada com PBES2...) e, no OpenSSH, se está cifrada e o fingerprint. Os números dela nunca aparecem no relatório. No Autoview, uma chave privada mostra este relatório, nunca o texto, para que ela não apareça na tela só por passar sobre ela; `T` no viewer mostra o texto.
- **Chaves públicas, `authorized_keys` e `known_hosts`:** uma linha por chave, com o tipo e o tamanho, o fingerprint SHA-256 como o `ssh-keygen -l` o escreve, e o comentário. Uma opção `command=` é marcada, porque muda o que a chave permite; um host com hash aparece como tal.

Nada é baixado: os endereços de uma CRL ou de um OCSP só são mostrados. Antes que qualquer parser veja um bloco, o aninhamento (no máximo 64 níveis) e os tamanhos são conferidos; um arquivo tem no máximo 1.000 blocos de 64 KiB cada. PKCS#12 (`.p12`, `.pfx`) e chaves PGP não são lidos.

## Logs

Um log abre como texto, com as cores dos seus níveis, e o rodapé diz `Log`. Um arquivo é log pelo nome (`.log`, `.out`, `.err`, um rodado como `app.log.1` ou `app.log.2.gz`, `syslog`, `messages`, `kern.log`) ou pelo conteúdo: a maioria das primeiras linhas começa com data e hora (`2026-11-16 10:00`, `[2026-11-16 ...`, ou o `Nov 16 10:00:00` do syslog).

- **Níveis:** a primeira palavra de nível nas primeiras 120 colunas de uma linha, em maiúsculas ou não, dá a cor dela: erros (`ERROR`, `ERR`, `FATAL`, `CRIT`, `PANIC`...) em vermelho, avisos (`WARN`, `WARNING`) em amarelo, `DEBUG` e `TRACE` apagados. O `<n>` do syslog no começo da linha também conta, e o nível de um log em JSON também (`"level":"error"`). Uma linha sem nível, como um stack trace, fica com o da linha anterior. `C` desliga as cores.
- **Cores ANSI:** as cores que um programa gravou no log (`ESC [31m`) aparecem como cores, e as sequências saem do texto. Qualquer outra sequência é descartada, porque poderia mover o cursor ou escrever no clipboard (OSC 52), e o cabeçalho conta quantas, como em `[ANSI: 3 removed]`. `H` continua mostrando os bytes como estão, e a busca olha o texto sem as sequências. Num arquivo acima de 16 MiB, lido por uma janela, as sequências aparecem como `·`, como em qualquer texto, e as cores são as dos níveis.
- **Seguir:** com a última linha na tela (End), as linhas novas entram conforme o programa as escreve, e a tela fica no fim; o cabeçalho diz `[FOLLOW]`. O arquivo é olhado a cada meio segundo, também num compartilhamento de rede. Subir, ou buscar, para de seguir; End volta a seguir. Com um filtro (`&`), as linhas novas passam por ele. Se o arquivo encolheu ou mudou em vez de crescer (rodado ou reescrito), o cabeçalho diz `(the file changed: R reloads)`, e nada é lido até o R. Um log comprimido não é seguido.

## Patches

Um `.patch` ou `.diff`, ou qualquer texto que comece como um (`diff --git`, ou `---`, `+++` e `@@`), abre como texto com as cores de um patch. O Git diff (`D`) aparece do mesmo jeito. O cabeçalho resume, como em `[12 files, +340 −98]`, e o rodapé diz onde está o alto da tela: `3/12 src/viewer.rs @@ 2/5`.

| Tecla | Ação |
|---|---|
| `]` `[` | próximo ou anterior arquivo |
| `}` `{` | próximo ou anterior hunk |
| `Tab` | a lista dos arquivos, com `+` e `−` de cada um; letras filtram, `Enter` vai a um |

Algumas mudanças fazem mais que mudar linhas, e passam despercebidas numa leitura rápida. Elas aparecem em vermelho, e o cabeçalho as conta como `to check`:

- um arquivo que passa a ser executável (`100755`), um link simbólico (`120000`) ou um submódulo (`160000`);
- um caminho absoluto, que sobe com `..`, que entra em `.git` (um hook é código que o Git executa), ou com caracteres de controle, de direção do texto ou invisíveis (um nome como `invoice‮fdp.exe` esconde sua extensão real).

O cabeçalho também conta:

- `look unchanged`: uma linha removida e uma adicionada que só diferem no que não se vê, como espaços, `\r`, caracteres invisíveis ou letras parecidas (um `а` cirílico no lugar de um `a` latino). O par aparece em amarelo;
- `hunks with wrong counts`: um hunk cuja linha `@@` não bate com as suas linhas. O índice segue as linhas que estão lá, nunca os números;
- `whitespace`: linhas adicionadas com espaço no fim, ou com espaço antes de tab no recuo, como o `git diff --check` aponta.

Uma série do `git format-patch` mostra cada mensagem na lista, pelo assunto. Nada é aplicado. Num arquivo acima de 16 MiB, lido por uma janela, as cores ficam, mas não há índice: `]`, `}` e Tab dizem isso.

## Caracteres de controle de direção

Alguns caracteres invisíveis mudam a direção do texto e podem fazer um código parecer diferente do que ele realmente diz (Trojan Source, CVE-2021-42574). O visualizador nunca os esconde: cada um aparece como um `·` vermelho, e o cabeçalho informa quantos são.

## Codificação do texto

O Xylem descobre a codificação sozinho, nesta ordem: primeiro descomprime o arquivo, depois procura um BOM (UTF-8, UTF-16 LE ou BE), depois UTF-16 sem BOM, depois UTF-8. Um arquivo que não é nada disso é texto antigo: o Xylem escolhe CP850 (DOS, Europa Ocidental) ou Windows-1252 contando as letras acentuadas. Os arquivos `.reg` do Windows são UTF-16 e abrem como texto normal.

**O** mostra o arquivo na próxima codificação: UTF-8, CP850, Windows-1252, CP437, Windows-1251, KOI8-R, UTF-16 LE, UTF-16 BE e de volta ao UTF-8. O rodapé diz qual, e o cabeçalho mostra o nome quando não é UTF-8. Se você escolher UTF-16 num arquivo que abriu em hexadecimal, ele passa para texto. Sua escolha continua depois de editar o arquivo e voltar.

Para usar sempre uma só codificação nos textos antigos, em vez do palpite, defina `legacy_charset` como `cp850`, `cp1252` ou `cp437` (**K** na página 2 da [configuração](configuration.md)). O padrão é `auto`.

## Arquivos grandes

Com **V**, um arquivo no disco maior que 16 MiB não é carregado: o visualizador guarda as linhas perto da tela, umas 2.000, e lê as outras do arquivo conforme ela anda, sem prender a tela. Uma linha ainda não lida aparece como `…` por um instante. Assim o arquivo não tem limite de tamanho, e a memória usada não cresce com ele.

- **As linhas são contadas em segundo plano.** O rodapé mostra `Line 1 of 1,234,567+ (counting…)` e depois o total. **End** e **g** vão a qualquer linha já contada. Esc para a contagem, e o que foi contado fica.
- **A busca** (`F`, `B`, `/`, `\`, Space) percorre o arquivo em segundo plano: o rodapé mostra `Searching... line 1,200,000 (Esc stops)`, e a tela vai até a ocorrência quando ela é encontrada.
- **O filtro de linhas** (`&`) percorre o arquivo inteiro em segundo plano, e as linhas aparecem conforme são encontradas, até 1.000.000 delas. Esc o para e mantém o que foi encontrado.
- **O hexadecimal** (`H`) lê os bytes onde a tela está, e a busca dele também percorre o arquivo.
- **Outra codificação** (`O`) lê as linhas de novo; com UTF-16 elas são contadas de novo, porque uma linha termina em outro lugar.
- **Sem cores de sintaxe e sem Markdown renderizado**: os dois leem o texto desde o início.
- **Uma linha com mais de 1 MiB** é cortada na tela, terminando em `…`; o hexadecimal a mostra inteira.
- **Se o arquivo mudar** enquanto está aberto, o cabeçalho diz `(the file changed: R reloads)`, e **R** o lê de novo e volta à linha em que estava.
- **Um Batch** (`Ctrl+B`) sobre o texto ainda lê o arquivo para a memória, até o limite de tamanho abaixo, e avisa quando isso não é o arquivo inteiro.

Para chegar a uma linha, o visualizador anota onde começa cada 256ª linha, cerca de 8 bytes a cada 256 linhas, só na memória. Os caracteres de controle de direção são contados no arquivo inteiro junto com as linhas (em UTF-8), e o cabeçalho mostra quantos foram achados até ali.

Um arquivo comprimido, um item de um arquivo compactado, um arquivo numa máquina remota e o Autoview não são lidos assim: não há como pular dentro deles, então seguem o limite de tamanho abaixo.

## Limite de tamanho

Nesses casos, e nos arquivos de até 16 MiB, o visualizador lê só o início de um arquivo grande demais, e o cabeçalho diz `(showing first N MiB)`. Quanto ele lê é a opção `viewer_max_size` (**L** na página 2 da [configuração](configuration.md)): `auto`, `16`, `64`, `256` ou `1024` (MiB). Com `auto`, o padrão, o limite é 1/32 da RAM da máquina, nunca menos de 16 MiB e nunca mais de 256 MiB (são 16 MiB quando não é possível ler a RAM). A tela de configuração mostra o valor em uso, por exemplo `auto (256 MiB)`.

Dois casos não seguem essa regra. O Autoview sempre lê 1 MiB. Um arquivo em uma máquina remota (SFTP) é lido até 16 MiB, no máximo, porque o Esc não consegue cancelar essa leitura.

Um limite maior custa memória e tempo: com `1024`, um texto comprimido que descomprime para esse tamanho ocupa cerca de 2 GiB de RAM. Linhas muito curtas custam mais do que os seus bytes, porque cada linha tem o seu próprio controle, e o mesmo vale para tabulações e caracteres de controle de direção escondidos: o visualizador guarda linhas até cerca do dobro do limite de tamanho na memória e, passado isso, o cabeçalho diz `(showing first N lines)`. Um log de linhas comuns (uns 100 bytes) nunca é cortado por isso.

## Arquivos comprimidos

Arquivos terminados em `.gz`, `.xz`, `.zst` ou `.bz2` abrem descomprimidos, como texto. O Xylem olha os primeiros bytes do arquivo, não o nome, e abre só um nível: um `.gz` dentro de um `.gz` continua comprimido. O cabeçalho mostra `gzip → N bytes` (o formato e o tamanho depois de descomprimir).

A descompressão tem limites. O visualizador lê até o limite de tamanho, leva no máximo 3 s e usa no máximo 64 MiB de memória para o dicionário do xz ou a janela do zstd. No Autoview os limites são 1 MiB, 100 ms e 16 MiB. Quando o limite de saída ou o de tempo é atingido, o cabeçalho diz `(showing first N bytes decompressed)`. Uma nota explica o que aconteceu:

- `decompression stopped after 3 s` (100 ms no Autoview)
- `compressed data truncated or corrupt` (dados comprimidos truncados ou corrompidos)
- `needs more than 64 MiB of memory to decompress` (16 MiB no Autoview, com ` (V to view)` quando os 64 MiB do V podem bastar)

A nota some na primeira tecla, mas o cabeçalho guarda o problema: `(truncated or corrupt)`, `(incomplete: …)` quando o limite de memória parou a descompressão no meio, ou `(not decompressed: …)` quando nada pôde ser descomprimido (aí os bytes brutos aparecem em hexadecimal).

O Autoview mostra a nota no título. No modo hexadecimal (`H`) você vê os bytes descomprimidos. As cores seguem o nome sem a extensão de compressão: `app.rs.gz` é colorido como Rust.

Para um `.tar.gz` (ou `.tar.xz`...), a nota diz `TAR archive: Alt+F5 opens it as a folder` (o Alt+F5 abre o arquivo como uma pasta).

Para ver os bytes comprimidos, desligue `viewer_decompress` (**J** na página 2 da [configuração](configuration.md)).

## Modo tabela

Planilhas, CSV, DBF, SQLite e Parquet abrem como uma grade, com filtro SQL: veja [Tabelas](tables.md).

Veja também: [busca nos arquivos marcados](search.md).
