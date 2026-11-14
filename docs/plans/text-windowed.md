# Spec: o texto do viewer em janelas

Origem: conversa de 2026-11-14. A medição de memória daquele dia mostrou que um texto
de 234 MB ocupa 475 MB no viewer (os bytes lidos mais as linhas prontas), depois de já
corrigidos o pico e a memória retida (`Lines`, PR #116). O autor pediu as janelas para o
texto: é o que falta do subprojeto 3 de [viewers.md](viewers.md), começado pelas tabelas
([tables-windowed.md](tables-windowed.md)).

**Estado: aprovado pelo autor em 2026-11-14, com as recomendações abaixo ("Decisões").**
Em implementação na branch `text-windowed`. As decisões em
aberto estavam marcadas com **(?)** e estão reunidas no fim; valem as recomendações.

## Como ficou (2026-11-14)

As cinco fatias estão na branch `text-windowed`, num só conjunto de mudanças.

- **`src/viewer/text_window.rs`:** `TextWindow` (a janela), `Text` (o texto do viewer:
  `Whole(Lines)` na memória ou `Window`), o índice (`LineIndex`, uma marca a cada 256
  linhas), e as threads de contagem, leitura, busca e filtro. `Viewer::open` abre em
  janela todo arquivo acima de 16 MiB com `V` que não seja comprimido
  (`open_window`); o resto do viewer continua indexando `self.lines[i]`, que numa
  janela devolve `…` para a linha ainda não lida.
- **A linha é escrita por `put_line`,** tirado de `text_lines`: o mesmo código faz a
  linha de um arquivo pequeno e a de uma janela.
- **A janela guarda as linhas pelo número,** não um intervalo: com o filtro, as linhas
  da tela estão longe umas das outras no arquivo. São até 1.000 linhas para cada lado
  do topo da tela; as da tela vêm sempre, as outras enquanto couberem em 32 MiB.
- **Espera curta:** ao pedir linhas que a tela precisa, a tela espera 30 ms pela
  resposta antes de mostrar `…`, o que em disco local evita o piscar.
- **A contagem começa quando o texto é mostrado,** não ao abrir: um CSV grande visto
  só como tabela, ou um binário visto só em hex, nunca é contado.
- **Busca:** o achado além das linhas já contadas espera a contagem chegar lá (e a
  retoma, se o Esc a tinha parado). Para trás, lê um bloco do índice por vez e guarda
  só o último achado do bloco.
- **Filtro:** as linhas aparecem conforme são achadas e o topo volta para o começo; um
  filtro que não acha nada sai sozinho, com o aviso.
- **Esc** para, nesta ordem, a busca, o filtro e a contagem; o seguinte fecha o viewer.
- **Reload (R ou depois de editar):** volta à linha em que estava quando a contagem
  chega lá.
- **Medido** no app real (release, glibc), com o texto de 234 MB e 2,5 milhões de
  linhas: abre na hora e ocupa **15 MB** (eram 475 MB); a contagem leva 0,08 s de CPU;
  End, `g`, a busca até a última linha, o filtro e o hex no fim do arquivo acertam.
- **Ficou para depois** (no BACKLOG): Batch sem limite sobre esses textos; no Ctrl+V, a
  busca que não acha nada num arquivo grande não segue para o próximo arquivo marcado
  (a resposta chega depois); a porcentagem do rodapé, enquanto conta, é pelas linhas
  já contadas, não pelos bytes; em UTF-16 os controles bidi são contados só nas
  linhas da janela.
- **Manual:** só o inglês (`docs/manual/viewer.md`, "Big files"); as traduções ficam
  pendentes no BACKLOG, com a nota no topo de cada página.

## O que existe hoje

- **O arquivo inteiro vai para a memória,** até o limite (`viewer_max_size`; no
  automático, RAM/32 entre 16 e 256 MiB), e fica duas vezes: `data: Vec<u8>` (hex, troca
  de codificação, Batch) e `lines: Lines` (texto pronto para a tela).
- **Acima do limite, o resto não se vê:** o cabeçalho diz `(showing first N MiB)`.
- **Tudo percorre `lines` na thread da tela:** busca (`find_text`), filtro de linhas
  (`&`), ir para a linha, quebra de linha, cores (`Highlighter`, sequencial e só até
  20.000 linhas) e a contagem de controles bidi.
- **O que as tabelas já deixaram pronto** (`table_window.rs`, `records.rs`): a janela
  com tetos (2.000 linhas, 32 MiB), as threads de leitura, contagem e filtro com Esc, o
  `RowIndex` (o byte de uma linha a cada 256, 8 bytes por marca, no máximo 4 milhões de
  marcas) e a conferência de tamanho e data do arquivo antes e depois de cada leitura.

## O que muda

Um texto grande deixa de ser carregado: o viewer guarda só as **linhas perto da tela** e
pede as outras ao arquivo conforme se anda, numa thread. Um índice de linhas é montado em
segundo plano.

### Quais arquivos

- **Em janelas:** arquivo local, no disco, sem compressão, maior que **16 MiB (?1)**.
  Sem limite de tamanho.
- **Como hoje, na memória:** arquivos até 16 MiB (com cores, Markdown e tudo o mais sem
  mudança), comprimidos (`.gz`, `.xz`, `.zst`, `.bz2`), entradas de ZIP/TAR, arquivos
  remotos (SSH, contêiner) e as telas internas (ajuda, Git, info). Para esses o
  `viewer_max_size` continua valendo, porque não dá para pular dentro deles.
- **Autoview** não muda: lê só o começo, com o limite dele.

### Para quem usa

- **Abre na hora,** com as primeiras linhas, seja qual for o tamanho.
- **O total aparece enquanto é contado:** `Line 1 of 1,234,567+ (counting…)` e, ao fim,
  o total. Esc para a contagem; o que já foi contado vale. A porcentagem, enquanto conta,
  é pelos bytes.
- **Andar** nunca espera o disco: uma linha ainda não lida aparece como `…` por um
  instante.
- **`g` e End** chegam a qualquer linha já contada; além dela, `g` diz que a contagem
  ainda não chegou lá e **End vai à última contada (?3)**.
- **Busca (`F`, `/`, `\`)** roda em segundo plano a partir da tela: o rodapé mostra
  `Searching… line 1,200,000 (Esc stops)` e a tela pula para o achado.
- **Filtro de linhas (`&`)** varre o arquivo inteiro em segundo plano; as linhas
  aparecem conforme são achadas, até 1.000.000 (o teto das tabelas), e Esc para.
- **Hex (`H`)** também em janelas: blocos lidos por posição, sem limite.
- **Codificação (`O`)** decodifica de novo só a janela. O índice vale para UTF-8, CP437,
  CP850 e 1252 (a quebra de linha é o mesmo byte); para UTF-16 ele é refeito.
- **Quebra de linha (`W`) e números (`#`)** como hoje.
- **Cores: desligadas nesses arquivos (?2).** O `syntect` precisa ler do começo e hoje já
  para em 20.000 linhas, que um arquivo de mais de 16 MiB quase sempre passa.
- **O arquivo mudou:** como nas tabelas, a janela para, o cabeçalho diz
  `(the file changed: R reloads)` e **R** relê. Acompanhar um log que cresce é o
  subprojeto 4.
- **Batch (Ctrl+B)** sobre o texto: **lê até o limite, como hoje, dizendo que cortou
  (?4)**.
- **T num CSV grande** (texto de um arquivo que abre como tabela) passa a usar a janela,
  em vez de ler o arquivo até o limite.

### Como

- **`text_window.rs`**, ao lado de `table_window.rs`:
  - `LineIndex`: o byte onde começa uma linha a cada 256. A contagem procura `\n` com
    `memchr` em blocos (em UTF-16, de dois em dois bytes), retoma da última marca se for
    parada, e para no teto de marcas (cerca de um bilhão de linhas), dizendo isso.
  - a janela: linhas decodificadas perto da tela, pelos mesmos tetos das tabelas, feitas
    pelo `text_lines` de hoje (tabs, controles e bidi trocados por `·`), então a tela
    não ganha nenhum caminho novo para bytes hostis;
  - as threads de leitura, busca e filtro, uma de cada por viewer, com Esc.
- **`Viewer`** passa a ter duas fontes de texto: `Lines` (na memória, como hoje) ou a
  janela. O que hoje indexa `self.lines` pede a linha à fonte.
- **Codificação:** detectada no primeiro 1 MiB; bytes inválidos adiante viram `�`.
- **Controles bidi:** a contagem do arquivo inteiro sai da passada da contagem (em
  UTF-8, são sequências de bytes fixas), e o cabeçalho mostra o que já foi achado. A
  troca por `·` na tela continua por linha.

### Segurança

- **Uma linha sem fim:** uma linha com mais de **1 MiB é mostrada cortada (?5)**, com a
  marca de corte; o hex mostra tudo. Hoje ela é lida inteira até o orçamento de linhas.
- **Memória limitada pelo desenho:** a janela (32 MiB), o índice (32 MiB), os achados do
  filtro (8 MiB) e um bloco de leitura. Nada cresce com o arquivo.
- **Nenhum parser novo:** só a procura de `\n`, a decodificação e o `text_lines`, que já
  existem e já têm fuzz.
- **O arquivo que muda sob a leitura** é conferido por tamanho e data, como nas tabelas.

## Fatias

1. **A fonte, o índice e a janela no modo texto:** abrir, andar, PgUp/PgDn, Home/End,
   `g`, quebra de linha, números, codificação, o total sendo contado, o arquivo que mudou
   e R.
2. **Hex em janelas.**
3. **Busca em segundo plano** (texto e hex).
4. **Filtro de linhas em segundo plano.**
5. **Fechamento:** Batch, T no CSV grande, o manual em inglês, o BACKLOG, o
   `viewers.md`, as notas da versão, e a medição de memória (meta: o texto de 234 MB em
   menos de 50 MB, contra 475 MB hoje).

As fatias ficam na branch `text-windowed` e vão num PR só: a fatia 1 sozinha tiraria a
busca dos arquivos grandes.

**Tamanho:** parecido com o das tabelas em janelas (PR #113: 3.363 linhas, cerca de um
dia de sessões).

## Decisões em aberto

| # | Decisão | Recomendação |
|---|---|---|
| ?1 | A partir de que tamanho o texto é em janelas | 16 MiB, fixo (o piso do limite automático) |
| ?2 | Cores nos arquivos em janelas | desligadas; `C` diz o porquê |
| ?3 | End antes de a contagem terminar | vai à última linha contada, como nas tabelas; ler o fim de trás para a frente fica para os logs (subprojeto 4) |
| ?4 | Batch sobre um texto em janelas | lê até o limite, como hoje; Batch sem limite fica para depois |
| ?5 | Linha com mais de 1 MiB | cortada na tela, com a marca; o hex mostra tudo |
| ?6 | Em que versão entra | 0.9.0, junto com o Parquet em janelas |
