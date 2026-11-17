# O visualizador

> **Ainda não cobre a 0.8.0.** As tabelas sem o limite de 50.000 linhas estão só na versão em inglês por enquanto. Os textos grandes lidos conforme a tela anda ("Big files") também.

`V` abre o arquivo no visualizador (o Autoview, F7, o mostra ao lado da lista).
Arquivos binários abrem em hexadecimal. Arquivos acima do limite de tamanho mostram
apenas o início.
Planilhas, CSV, DBF, SQLite e Parquet abrem como tabela (veja Modo tabela).

| Tecla | O que faz |
|---|---|
| ↑ ↓ PgUp PgDn Home End | rola verticalmente |
| ← → | rola horizontalmente |
| `H` | próximo modo: texto e hexadecimal; num arquivo Markdown, renderizado, texto e hexadecimal |
| `O` | próxima codificação (UTF-8, CP850, 1252, CP437, UTF-16) |
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

Como no ZTreeWin: **F** busca desde o início, **B** de trás para frente a
partir do fim, **/** (ou `S`) a partir da página atual e **\\** para trás a
partir dela; **Space** continua na mesma direção, **+** e **-** escolhem a
direção. A ocorrência é destacada. No prompt de busca, **F2** alterna a
diferenciação de maiúsculas e minúsculas (ao ignorar maiúsculas/minúsculas, os
acentos também são considerados: `AÇÃO` encontra `ação`) e **F4** alterna entre
texto e hexadecimal (pares de bytes como `4d 5a`). Um `*` no meio do termo
corresponde a qualquer trecho na mesma linha (`error*disk`).

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
| `k` | copia o caminho do nó: `.servers[2].name` em JSON, `servers.2.name` em YAML e TOML |

O rodapé mostra o caminho do nó sob o cursor. A busca (`F`, `B`, `/`) olha chaves e valores, também dentro de nós fechados, e abre o caminho até o que achou. `T` mostra o texto na linha do nó, e volta; `H` passa por árvore, texto e hexadecimal. `&`, `g`, `#` e Ctrl+B agem sobre as linhas do texto.

- Um nó fechado diz o que tem: `{12 keys}`, `[340 items]`.
- Um número aparece como o arquivo o escreve: `12345678901234567890` continua assim, e `1_000` no TOML também.
- Uma string mostra a quebra de linha como `↵`.
- Uma chave que aparece duas vezes no mesmo objeto é mostrada duas vezes, marcada `dup key`, e o cabeçalho conta quantas são. Os programas discordam sobre qual das duas vale, o que pode esconder um valor numa revisão.
- Uma âncora de YAML aparece como `&a1` e um alias como `*a1`: o alias nunca é expandido, então um arquivo feito para se multiplicar (um "billion laughs") não custa mais que o seu texto. Uma tag (`!Ref`) aparece antes do valor. Vários documentos ficam sob uma raiz, como `document 1`, `document 2`...

Um arquivo inválido, ou aninhado mais fundo que 512 níveis, abre como texto, e o aviso diz por quê e em que linha. A árvore é montada com o arquivo inteiro: um acima do limite de leitura abre como texto.

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
- um caminho absoluto, que sobe com `..`, que entra em `.git` (um hook é código que o Git executa), ou com caracteres de controle.

O cabeçalho também conta:

- `look unchanged`: uma linha removida e uma adicionada que só diferem no que não se vê, como espaços, `\r`, caracteres invisíveis ou letras parecidas (um `а` cirílico no lugar de um `a` latino). O par aparece em amarelo;
- `hunks with wrong counts`: um hunk cuja linha `@@` não bate com as suas linhas. O índice segue as linhas que estão lá, nunca os números;
- `whitespace`: linhas adicionadas com espaço no fim, ou com espaço antes de tab no recuo, como o `git diff --check` aponta.

Uma série do `git format-patch` mostra cada mensagem na lista, pelo assunto. Nada é aplicado. Num arquivo acima de 16 MiB, lido por uma janela, as cores ficam, mas não há índice: `]`, `}` e Tab dizem isso.

## Caracteres de controle de direção

Alguns caracteres invisíveis mudam a direção do texto e podem fazer um código parecer diferente do que ele realmente diz (Trojan Source, CVE-2021-42574). O visualizador nunca os esconde: cada um aparece como um `·` vermelho, e o cabeçalho informa quantos são.

## Codificação do texto

O Xylem descobre a codificação sozinho, nesta ordem: primeiro descomprime o arquivo, depois procura um BOM (UTF-8, UTF-16 LE ou BE), depois UTF-16 sem BOM, depois UTF-8. Um arquivo que não é nada disso é texto antigo: o Xylem escolhe CP850 (DOS, Europa Ocidental) ou Windows-1252 contando as letras acentuadas. Os arquivos `.reg` do Windows são UTF-16 e abrem como texto normal.

**O** mostra o arquivo na próxima codificação: UTF-8, CP850, Windows-1252, CP437, UTF-16 LE, UTF-16 BE e de volta ao UTF-8. O rodapé diz qual, e o cabeçalho mostra o nome quando não é UTF-8. Se você escolher UTF-16 num arquivo que abriu em hexadecimal, ele passa para texto. Sua escolha continua depois de editar o arquivo e voltar.

Para usar sempre uma só codificação nos textos antigos, em vez do palpite, defina `legacy_charset` como `cp850`, `cp1252` ou `cp437` (**K** na página 2 da [configuração](configuration.md)). O padrão é `auto`.

## Limite de tamanho

O visualizador lê só o início de um arquivo grande, e o cabeçalho diz `(showing first N MiB)`. Quanto ele lê é a opção `viewer_max_size` (**L** na página 2 da [configuração](configuration.md)): `auto`, `16`, `64`, `256` ou `1024` (MiB). Com `auto`, o padrão, o limite é 1/32 da RAM da máquina, nunca menos de 16 MiB e nunca mais de 256 MiB (são 16 MiB quando não é possível ler a RAM). A tela de configuração mostra o valor em uso, por exemplo `auto (256 MiB)`.

Dois casos não seguem essa regra. O Autoview sempre lê 1 MiB. Um arquivo em uma máquina remota (SFTP) é lido até 16 MiB, no máximo, porque o Esc não consegue cancelar essa leitura.

Um limite maior custa memória e tempo: com `1024`, um arquivo de texto comum desse tamanho ocupa cerca de 2,5 GiB de RAM e leva uns 5 s para abrir. Linhas muito curtas custam mais do que os seus bytes, porque cada linha tem o seu próprio controle, e o mesmo vale para tabulações e caracteres de controle de direção escondidos: o visualizador guarda linhas até cerca do dobro do limite de tamanho na memória e, passado isso, o cabeçalho diz `(showing first N lines)`. Um log de linhas comuns (uns 100 bytes) nunca é cortado por isso.

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

Alguns arquivos abrem como uma grade em vez de texto. O Xylem decide pela extensão, e só para arquivos sem compressão neste disco: um arquivo de máquina remota ou de dentro de um pacote aparece como conteúdo comum, com a nota `Tables are shown only for files on this disk`.

| Formato | Extensões | Nomes das colunas |
|---|---|---|
| CSV, TSV | `.csv` `.tsv` `.tab` | a primeira linha, quando pelo menos metade dela não é número; senão A, B, C... |
| JSON Lines | `.jsonl` `.ndjson` | as chaves dos objetos |
| Planilhas | `.xlsx` `.xls` `.xlsb` `.ods` | A, B, C... (a primeira linha é uma linha como as outras) |
| dBase, Clipper, FoxPro | `.dbf`, com seu arquivo memo `.dbt` | os nomes dos campos |
| SQLite | `.db` `.sqlite` `.sqlite3` | as colunas da tabela |
| Parquet | `.parquet` | as colunas do arquivo |

O separador do CSV (vírgula, ponto e vírgula, tabulação ou barra vertical) é detectado pelo início do arquivo.

| Tecla | O que faz |
|---|---|
| ↑ ↓ | linha anterior ou seguinte |
| ← → | coluna anterior ou seguinte |
| PgUp PgDn | uma página de linhas para cima ou para baixo |
| Home End | primeira ou última coluna |
| Ctrl+Home Ctrl+End | primeira ou última linha; sob um filtro, das linhas que ele mantém |
| `[` `]` | aba anterior ou seguinte de uma planilha, ou tabela de um banco SQLite, dando a volta; isso limpa o filtro |
| `Tab` | a lista de abas ou tabelas: digite parte de um nome para reduzi-la, Enter abre uma e Esc volta; abrir outra limpa o filtro. Num banco, as views vêm depois das tabelas, marcadas com `(view)` |
| `g` ou `:` | pula para uma linha pelo número dela no arquivo; se o filtro a escondeu, para a próxima que ele manteve |
| `\` ou `&` | o filtro SQL |
| `m` ou Insert | marca ou desmarca o registro sob o cursor e vai para o próximo; o **Ctrl+B** passa a gravar só os marcados |
| `M` | limpa as marcas |
| `i` | a estrutura da tabela: o que o arquivo diz de cada coluna (veja abaixo) |
| Enter | o registro sob o cursor em forma de ficha, um campo por linha: ↑ ↓ escolhem o campo, ← → vão ao registro anterior ou seguinte, `c` copia o registro inteiro (`NOME : valor`, uma linha por campo) e `y` o valor do campo, para colar num editor ou terminal; Esc volta. O que se copia é o que a tela mostra: caracteres de controle e de direção do texto, quebras de linha inclusive, viram `·`, e colar num terminal não executa nada. Fora da tabela, Enter fecha o visualizador |
| `T` | vai para o texto do arquivo e volta |
| `H` | da tabela para o texto, depois hexadecimal, depois de volta à tabela |

A moldura mostra `TABLE [2/5: nome]` quando há mais de uma aba ou tabela, e a linha de baixo mostra onde está o cursor: `Cell B7 (Row 7 of 120)`. Com filtro, as linhas mantêm o número que têm no arquivo, e essa linha diz também qual é a ocorrência: `Cell A13 (Row 13 of 31, match 2 of 8)`. A última linha da grade traz o conteúdo inteiro da célula e o filtro em uso. A busca (`F`, `B`, `/`) não olha dentro da grade e nada encontra ali; ela funciona no modo texto.

As marcas continuam sob um filtro, e o Batch grava o que está marcado e o filtro mantém. Elas pertencem à aba ou tabela aberta e se perdem ao trocá-la. Se o arquivo mudar depois da marcação, o Batch recusa, pois outros registros podem estar naquelas posições.

### A estrutura

`i` mostra, sobre a grade, as colunas em ordem com o que o arquivo diz de cada uma, como o `DISPLAY STRUCTURE` do dBase e do Clipper. As setas, PgUp, PgDn, Home e End rolam a lista; Esc, Enter ou `i` fecham.

| Formato | O que mostra |
|---|---|
| dBase, Clipper, FoxPro | nome, tipo (`C` caractere, `N` numérico, `D` data, `L` lógico, `M` memo, `F` ponto flutuante, `I` inteiro, `B` double, `Y` moeda, `T` data e hora), tamanho e decimais; acima, o número de registros e a data da última atualização escritos no cabeçalho, e o tamanho de um registro |
| SQLite | o tipo declarado de cada coluna, `Not null`, o valor padrão e a posição na chave primária; acima, se é tabela ou visão, e os índices com suas colunas (no máximo 32). Uma visão mostra as colunas que dá; uma query inteira, as do resultado, com o tipo de cada uma quando vem direto de uma coluna de tabela |
| Parquet | o tipo Arrow de cada coluna e se aceita nulos; acima, as linhas e os grupos de linhas que o arquivo declara e o programa que o gravou |
| CSV, JSON Lines, planilhas | não declaram tipos: o que as linhas lidas têm em cada coluna (inteiro, decimal, lógico, texto, vazio) e o valor mais largo |

Nomes, tipos e valores padrão vêm do arquivo e passam pela mesma limpeza das células: caracteres de controle e de direção do texto aparecem como `·`. Um `.dbf` cujo cabeçalho se contradiz mostra `?` nos decimais.

### O filtro SQL

`\` ou `&` pede uma condição, como o `WHERE` do SQL (a palavra `WHERE` é opcional), e mostra só as linhas que a atendem. O prompt começa com o filtro atual; deixe-o vazio para ver todas as linhas de novo. Uma nota diz `Filter matched N rows` ou `SQL error: ...`; depois de um erro, o filtro anterior continua valendo.

```
age >= 30 AND name LIKE 'Ali%'
WHERE status IN ('open', 'late') OR total > 1000.50
NOT (city = 'Rio' OR city IS NULL)
B ILIKE '%silva%'
```

| Operador | O que faz |
|---|---|
| `=` `!=` `<>` `<` `<=` `>` `>=` | comparam |
| `AND` `OR` `NOT` e parênteses | combinam condições (`AND` vem antes de `OR`) |
| `LIKE` `ILIKE` | combinam com um padrão, `%` para qualquer texto e `_` para um caractere; `ILIKE` ignora maiúsculas e minúsculas |
| `IN (...)` `NOT IN (...)` | está, ou não está, na lista |
| `IS NULL` `IS NOT NULL` | a célula está, ou não está, vazia |

As palavras-chave podem estar em maiúsculas ou minúsculas.

- **Colunas:** o nome da coluna, em maiúsculas ou minúsculas e sem aspas, ou a sua letra, de `A` a `Z`. A letra é o jeito de chegar a um nome com espaços ou símbolos. Uma coluna que não existe conta como vazia, sem erro.
- **Valores:** texto entre aspas simples ou duplas (`\'` para uma aspa dentro dele), números, `TRUE`, `FALSE` e `NULL`. O que está entre aspas é sempre texto, nunca coluna.
- **Células:** o tipo vem da aparência da célula. Vazia é `NULL`, `true` e `false` são booleanos, números são números e o resto é texto. Por isso escreva `code = 7`, não `code = '7'`: texto e número nunca são iguais, e uma célula com `007` é o número 7. Comparações de texto distinguem maiúsculas de minúsculas.
- **Tamanho:** um filtro tem no máximo 1.024 bytes e 128 termos.

### Uma query inteira num banco

Num banco SQLite, o mesmo prompt aceita uma query inteira, começando com `SELECT` ou `WITH`, com tudo o que o SQLite faz: `JOIN`, `GROUP BY`, `ORDER BY`, funções.

```
SELECT c.name, sum(o.price) FROM customers c JOIN orders o ON o.code = c.code GROUP BY c.name
```

A moldura diz `QUERY:`, e a grade mostra o resultado. Uma condição filtra então essas linhas, e **Ctrl+B** grava tudo o que a query dá. Deixar o prompt vazio limpa o filtro e depois volta à tabela; `Tab`, `[` e `]` também vão para uma tabela. A query só lê, uma de cada vez: qualquer outra coisa é recusada, e um erro deixa a tela como estava. Os limites abaixo valem, então uma query lenta para em 3 s e mostra as linhas que obteve; o Batch não tem esse limite. O Xylem não altera bancos: `UPDATE`, `DELETE` e parecidos recebem uma mensagem dizendo isso. Para alterar um banco, use outro programa: `O` na janela de arquivos o abre no definido em `opener`, e `'*.db' = 'sqlite3 %f'` em `[editors]` faz o `E` abri-lo no `sqlite3`.

### Limites

Uma tabela guarda no máximo 50.000 linhas e 256 colunas, e leva no máximo o que os outros limites do visualizador permitem: 3 s (100 ms no Autoview), e uma planilha que desempacota para mais que o limite de tamanho (1 MiB no Autoview) não abre como tabela. O que passa disso não aparece e o filtro não vê; o cabeçalho avisa: `(showing first 50,000 rows)` ou `(stopped by the time limit: showing N rows)`. Enquanto a tabela é lida, a tela fica livre e uma nota avisa; Esc para a leitura e mostra o que já veio, `(stopped by Esc: showing N rows)`, e o Esc seguinte fecha o visualizador. Uma célula mostra no máximo 2.048 caracteres e depois `…`. Caracteres de controle e de direção do texto aparecem como `·`.

Quando uma planilha (XLSX, XLSB, XLS, ODS) tem mais linhas que isso, o cabeçalho e a nota dizem isso: `(the sheet has more than 50,000 rows, the most Xylem shows: open it in Excel or another spreadsheet program to see them all (O))`, ou seja, abra-a no Excel ou em outro programa de planilhas para ver todas. **O** a abre no programa que o seu sistema usa para ela.

## Tabelas em bibliotecas C

Bancos SQLite (`.db`, `.sqlite`, `.sqlite3`) e arquivos Parquet são lidos por bibliotecas escritas em C, que não têm a segurança de memória do Rust. Por isso, eles só abrem como tabela com `V`. O Autoview lê tudo por onde o cursor passa e mostra esses arquivos como conteúdo comum, com a nota `Parquet or SQLite: V opens the table`. Abra um banco de origem não confiável só se aceitar esse risco. Arquivos `.ods` e `.xls` também só abrem como tabela com `V`: o leitor deles monta a planilha inteira primeiro, e essa parte não pode ser interrompida.

Veja também: [busca nos arquivos marcados](search.md).
