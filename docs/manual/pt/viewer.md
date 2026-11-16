# O visualizador

`V` abre o arquivo no visualizador (o Autoview, F7, o mostra ao lado da lista).
Arquivos binários abrem em hexadecimal. Um arquivo grande no disco não é lido inteiro: o visualizador lê o que a tela mostra conforme ela anda (veja [Arquivos grandes](#arquivos-grandes)). O que não pode ser lido assim mostra só o começo, até o limite de tamanho.
Planilhas, CSV, DBF, SQLite e Parquet abrem como tabela (veja [Tabelas](tables.md)).

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

## Caracteres de controle de direção

Alguns caracteres invisíveis mudam a direção do texto e podem fazer um código parecer diferente do que ele realmente diz (Trojan Source, CVE-2021-42574). O visualizador nunca os esconde: cada um aparece como um `·` vermelho, e o cabeçalho informa quantos são.

## Codificação do texto

O Xylem descobre a codificação sozinho, nesta ordem: primeiro descomprime o arquivo, depois procura um BOM (UTF-8, UTF-16 LE ou BE), depois UTF-16 sem BOM, depois UTF-8. Um arquivo que não é nada disso é texto antigo: o Xylem escolhe CP850 (DOS, Europa Ocidental) ou Windows-1252 contando as letras acentuadas. Os arquivos `.reg` do Windows são UTF-16 e abrem como texto normal.

**O** mostra o arquivo na próxima codificação: UTF-8, CP850, Windows-1252, CP437, UTF-16 LE, UTF-16 BE e de volta ao UTF-8. O rodapé diz qual, e o cabeçalho mostra o nome quando não é UTF-8. Se você escolher UTF-16 num arquivo que abriu em hexadecimal, ele passa para texto. Sua escolha continua depois de editar o arquivo e voltar.

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
