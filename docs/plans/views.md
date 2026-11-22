# Plano: visões fora do visual clássico (Linha do tempo, Varredura, Tabela)

**Feito na 0.9.0** (pedida em 2026-11-20; aprovada em 2026-11-21: as quatro etapas entram na 0.9.0, e a tecla é **Alt+V**). Etapas 1 a 3 no PR #133; a 4, a Tabela, no branch `views-table`.

Origem: o autor propôs romper com o visual clássico em situações específicas,
por um comando. Critério combinado: só vale uma tela nova que responda a uma
pergunta que a árvore e a lista não respondem bem; o visual clássico continua o
padrão, e cada visão é ida e volta. As três visões saem do que o Xylem já tem na
memória (tamanho, data, tipo, permissões, status do Git) ou dos relatórios que
já existem por arquivo.

Descartados na conversa: treemap (ilegível em texto; o Usage responde melhor),
imagens (um protocolo por terminal, e um decodificador alimentado com arquivos
hostis), grafo do Git (fora do papel do Xylem), modo foco (trivial e sem valor
para quem chega).

## A tecla: um seletor de visões

Uma tecla abre um seletor no estilo dos favoritos: **Timeline**, **Scan**,
**Table**, cada um com a sua letra. A visão vale para **a lista atual**: o
diretório, o Branch, o Showall, com o Filespec e o `W` que estiverem valendo.
Esc volta ao visual clássico, na mesma posição.

**A tecla é Alt+V** (decidido pelo autor em 2026-11-21): V de visões, ao lado
do V de View, e o K fica livre. G, H e J estão reservadas para comandos que
ainda faltam (Global, sHortcut, JFC), Alt+G para o Graft e Z para o zoxide.
Cada visão também pode ter tecla própria em `[keys]`, sem padrão.

As três visões são só leitura: nada nelas apaga, move ou muda arquivos. Quando
uma visão leva a uma escolha (uma barra, uma categoria, uma linha), a escolha
volta para a lista clássica como filtro, e de lá seguem Ctrl+T, Ctrl+C, Batch
como sempre.

## 1. Varredura (Scan): o que há de perigoso nesta pasta

Os relatórios de arquivos não confiáveis, hoje um arquivo por vez com `V`,
aplicados a uma lista inteira: um ZIP baixado, um repositório clonado, um
pendrive.

### O que mostra

Uma tela por categoria, com a contagem, e Enter abre a lista dos arquivos da
categoria (e `V` em cada um mostra o relatório de sempre):

| Categoria | De onde vem | Lê o conteúdo? |
|---|---|---|
| Nomes com caracteres de direção do texto ou de controle | `bidi.rs`, o nome | não |
| Nomes que enganam: extensão dupla (`fatura.pdf.exe`), espaços antes da extensão | o nome | não |
| setuid e setgid; arquivos e pastas que qualquer um pode escrever | o modo | não |
| Links simbólicos que saem da pasta varrida, ou apontam para o que não existe | o destino do link, sem segui-lo | não |
| Executáveis: empacotados (UPX e outros), sem PIE, sem NX, com seções graváveis e executáveis | `viewer/binaries.rs` | sim, o cabeçalho, com limite |
| Documentos do Office com macro, template remoto, objetos embutidos ou texto oculto | `viewer/office.rs` | sim, com os limites de hoje |
| Patches que tocam caminhos perigosos ou nomes com caracteres escondidos | `viewer/patch.rs` | sim, com limite |
| Scripts de instalação em pacotes, quando os formatos novos existirem | plano de formatos | sim |

### Como roda

- **Primeiro o que não lê nada** (nome, modo, link): aparece na hora, porque já
  está na memória.
- **Depois o que lê conteúdo**, em segundo plano, com progresso e Esc para
  parar. A tela se completa enquanto lê, e diz quantos arquivos faltam.
- **Limites**: o mesmo limite de leitura de cada relatório, um teto de arquivos
  lidos (configurável, padrão a decidir medindo), e um tempo por arquivo. Um
  arquivo que estoura o limite entra em "não analisado", com o motivo: nunca
  some em silêncio.
- **Numa montagem do rclone**, a parte que lê conteúdo fica desligada, como o
  `contains` do `W`: cada arquivo seria baixado. A tela diz isso e mostra só o
  que vem do nome e do modo. O mesmo numa máquina remota, até medir o custo pelo
  SSH.
- **Dentro de um arquivo compactado**: a varredura roda sobre a lista do
  arquivo, lendo cada membro com os limites de descompactação de hoje (bomba de
  ZIP já tratada).

### Segurança

A varredura é a função que mais lê arquivos hostis de uma vez. Ela não traz
parser novo: chama os mesmos que o `V` já usa, com os mesmos limites, um
arquivo por vez. Um parser que falha num arquivo marca aquele arquivo como "não
analisado" e segue (o `panic_guard` já cobre o viewer: conferir que cobre aqui).

A varredura não é antivírus e a tela diz isso: aponta o que merece um olhar,
não garante que o resto é seguro.

## 2. Linha do tempo (Timeline): o que mudou, e quando

### O que mostra

Os arquivos da lista num histograma pela data de modificação. A escala se
escolhe sozinha pelo intervalo das datas (horas, dias, semanas, meses, anos), e
`+`/`-` mudam a escala. Cada barra mostra, ao lado do cursor, quantos arquivos e
quantos bytes.

```
 nov 2026  ▁▁▂▁▁▁▁▁▃▁▁▂▁▁▁▁▁█▆▁   18/11: 312 files, 1.4 GiB
           1        10        20
```

- `B` alterna entre contar arquivos e somar bytes.
- Enter numa barra volta à lista com o período como condição. Um intervalo
  (Shift+← →) também.
- As datas no futuro e as anteriores a 1980 ficam numa barra à parte em cada
  ponta, porque datas hostis ou erradas existem e não podem achatar o gráfico.

### Como a escolha volta para a lista

Como uma condição do `W` sobre `modified` (`modified >= '2026-11-18' AND
modified < '2026-11-19'`), somada com AND à condição que já estiver valendo. O
título da janela mostra a condição, e `W` vazio a desfaz, como hoje. Não lê
nada do disco: só as datas já na memória.

## 3. Tabela (Table): a lista de arquivos no viewer de tabelas

### O que mostra

A lista atual aberta no viewer de tabelas, com as colunas que o `W` já conhece:
`name`, `ext`, `dir`, `size`, `disk`, `modified`, `age_days`, `mode`, `kind`,
`tagged`, `git`, `dup` (e `type` só se pedido, porque lê o começo de cada
arquivo).

Ganha o que as tabelas já têm: ordenar por qualquer coluna, o filtro SQL, o
registro como formulário, copiar um registro (`c`) e o Batch de uma linha por
registro (com uma máscara, um CSV, um script ou uma página).

### Como a escolha volta para a lista

- `T` numa linha marca o arquivo na lista clássica; as marcas aparecem como a
  coluna `tagged`.
- Esc volta para a lista; o filtro SQL usado na tabela pode virar o `W` da
  lista (uma pergunta, ou uma tecla própria), para os comandos que agem nos
  arquivos.

### A medir antes

Com 2.000.000 de nós (o teto da árvore), a tabela não pode copiar a lista
inteira para strings: as linhas têm de ser lidas da memória da árvore sob
demanda, como o viewer já faz com tabelas grandes por janela. Medir memória e
tempo de abertura com a árvore de teste maior antes de prometer a visão para
Showall de um disco inteiro.

## Etapas

Cada uma com a página do manual em inglês, as notas da versão e os testes; um
GIF quando valer.

1. **O seletor e a Varredura**, a parte sem leitura (nomes, modos, links).
2. **A Varredura com leitura**: executáveis, Office, patches, em segundo plano
   e com limites. Testes com os arquivos hostis do `fixture.sh` (os mesmos que
   o GIF dos arquivos não confiáveis vai usar).
3. **A Linha do tempo.**
4. **A Tabela**, depois da medição.

## Fora deste plano

- Agir em arquivos de dentro das visões (apagar, mover): a escolha sempre volta
  para a lista clássica.
- Salvar uma varredura como relatório (HTML, JSON): talvez depois, pelo Batch da
  tabela de resultados.
- Outras visões (composição por tipo): pode entrar como uma linha no topo do
  Usage, num plano próprio.

## Andamento (2026-11-21, para retomar)

**Decidido pelo autor:** a tecla é **Alt+V**, e as quatro etapas entram na 0.9.0.
O trabalho fica no branch `views`, criado a partir da `main` em `58620a6`.

**Conferido no código** (nada escrito ainda):

- **A tecla:** `alt+v` está livre no keymap padrão (`src/keymap.rs`, tabela
  perto da linha 430). Também estão livres `k`, `alt+a`, `alt+e`, `alt+j`,
  `alt+t`, `alt+w` e `alt+y`.
- **A lista atual:** `Pane::refresh_files` (`src/pane.rs`, perto da linha
  1048) monta `pane.files: Vec<FileRef>` a partir de `FileView` (Dir, Branch,
  Showall), aplicando o `filespec`, o `query` (o `W`, `FileQuery` em
  `src/model/query.rs`) e o `tags_only`.
- **O que cada arquivo tem na memória:** `FileEntry` (`src/model/tree.rs`) tem
  `name`, `size`, `disk`, `link` (inode), `modified`, `mode` e
  `kind: File | Symlink | Other`. O destino do link não fica guardado:
  para a Varredura, ler com `read_link`, que não segue o link, só em pasta
  local.
- **Nomes:** `bidi::has_bidi` está em `src/bidi.rs`.
- **Confirmações:** `ask_options` e `ConfirmKind` estão em
  `src/app/prompts.rs`, e a resposta é tratada em `confirmed()`.
- **Outras visões que já ocupam a tela:** `pane.usage` (Alt+U) e
  `pane.duplicates` (Alt+D). Um seletor sobreposto no estilo dos favoritos é o
  `bookmarks_view`, no App.

**Desenho da etapa 1** (aprovado pelo autor em 2026-11-21, com o aviso nos
sistemas de arquivos do Windows; o autor pediu as quatro etapas em seguida, e
testa na volta):

- **O seletor:** Alt+V abre uma caixa no estilo dos favoritos com **S**can,
  **T**imeline e ta**B**le, que também se escolhem com ↑↓ e Enter. Uma visão
  que ainda não existe não aparece.
- **A Varredura:** uma caixa sobreposta no App (`scan: Option<ScanView>`), e
  não uma visão do Pane como o Usage. Ela lista as categorias com as
  contagens. Na etapa 2, a mesma caixa mostra o progresso da leitura.
- **Como a escolha volta para a lista:** um filtro novo no Pane,
  `pick: Option<Pick { label, paths: Arc<HashSet<PathBuf>> }>`, aplicado em
  `refresh_files` junto com o `filespec` e o `W`. O título da lista mostra o
  rótulo, e um `W` vazio desfaz o `pick` também. A Linha do tempo (etapa 3)
  usa o `W` sobre `modified`, como o plano diz.
- **As categorias da etapa 1**, todas sem ler conteúdo:
  1. Nomes com caracteres de controle ou de direção do texto
     (`bidi::has_bidi`, `char::is_control`).
  2. Nomes que enganam:
     - uma extensão de documento seguida de uma executável
       (`fatura.pdf.exe`; executáveis: exe, scr, bat, cmd, com, pif, js, vbs,
       jar, msi, ps1, lnk, hta);
     - espaços antes do ponto da extensão;
     - um nome que termina em espaço ou ponto.
  3. setuid e setgid; arquivos e pastas graváveis por qualquer um (numa pasta,
     só sem o sticky bit). Nos sistemas de arquivos em que os modos são
     inventados (drvfs e 9p do WSL, vfat, exfat, ntfs, fuseblk), esta categoria
     não conta nada: a tela mostra no lugar um aviso de que as permissões não
     valem ali.
  4. Links que saem da raiz varrida (o diretório, o Branch ou a raiz do
     Showall), resolvidos sem seguir o link, ou que apontam para o que não
     existe. Só em pasta local: dentro de um arquivo compactado e na máquina
     remota, a categoria diz que não foi vista.
- **Commits:** um commit por etapa no branch `views`, e um PR no fim, se o
  autor concordar.

## Andamento (2026-11-21, tarde): etapas 1 a 3 prontas (PR #133)

Feitas no branch `views`, num commit só (as etapas mexem nos mesmos
arquivos), autorizado pelo autor, e no PR #133. Passaram
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` e
`cargo test` ao fim de cada etapa.

**Para a etapa 4:** o autor vai retomá-la noutra máquina, com mais memória,
começando pela medição.

- **Etapa 1** (`src/scan.rs`, `src/app/views_cmd.rs`, `draw_scan` em
  `src/ui.rs`): Alt+V abre o seletor (o menu de contexto, com
  `letters: true`: a letra roda a linha). A Varredura é uma tela cheia, como
  os favoritos. Nomes e modos dos arquivos saem da memória; modos das pastas e
  links, numa thread (`scan::Job`). Enter numa categoria lista os achados com
  o motivo; Enter num arquivo volta à lista com `Pane::pick` (os caminhos da
  categoria), cursor nele; numa pasta, vai para ela na árvore. `V` abre o
  relatório, e Esc volta à Varredura. Esc guarda a Varredura (`scan_back`):
  Alt+V, S na mesma lista a reabre; `R` varre de novo. O aviso dos sistemas
  de arquivos do Windows está em `scan::modes_are_made_up` (9p, drvfs, vfat,
  exfat, ntfs, ntfs3, fuseblk, cifs, smb3, fuse.rclone).
- **Etapa 2**: a mesma thread lê o conteúdo com os leitores do viewer.
  `binaries::warnings_file` / `warnings` (sem PIE, sem NX/DEP, sem ASLR, pilha
  executável e os sinais: empacotado, W+X, dados depois da última seção);
  `office::warnings` (o `render` foi dividido em `read` + layout, sem mudar o
  que o viewer mostra); `patch::warnings`. Cada arquivo sob `panic_guard`;
  o que falha ou passa do teto vai para "Not analysed" com o motivo. Teto:
  `scan_max_files` (10000, só no arquivo de configuração), contando os
  arquivos lidos, não os 64 bytes de cada um que dizem se é binário. Dentro de
  um arquivo compactado, cada membro é extraído com limite de 64 MiB pelo
  `archive::Reader`, aberto uma vez na thread. Conteúdo desligado na máquina
  remota e no rclone, com a nota na categoria.
- **Etapa 3** (`src/timeline.rs`, `draw_timeline`): histograma das datas da
  lista, escala automática (horas até 2 dias, dias até 90, semanas até 1 ano,
  meses até 10, anos), `+`/`-`, `B`, Shift+← → para um intervalo, barras à
  parte para antes de 1980 e para o futuro (mais de um dia à frente). Enter
  vira `W` sobre `modified`, com AND à condição que já havia, entre
  parênteses. Até 200.000 barras por escala.

**Traduções pendentes** (só o inglês foi escrito): `views.md` (página nova),
`releases.md` (0.9.0, Views), `commands-alt.md` (linha Alt+V),
`configuration.md` (`scan_max_files`). Os índices em pt, es, de, fr e zh
ganharam só a linha do link, porque o teste `index_links_to_every_topic`
exige.

**Etapa 4 (Tabela): não começada.** O plano manda medir antes, e a medição
com 2.000.000 de nós pesa na memória do WSL; ficou para combinar com o
autor. O caminho visto no código: `viewer::records::RowSource` (lido por
`table_window::Window` através de um `Opener` `Send + Sync`) pode ser
implementado sobre um retrato compacto da lista (`Arc<Vec<linha>>`, com o
índice da pasta em vez do caminho), sem gerar texto: as células só viram
texto quando a janela as pede. A medir: memória e tempo do retrato com a
árvore de teste maior. Ainda a decidir: como `T` na tabela marca o arquivo na
lista (uma ação nova do viewer para o App) e como o filtro SQL da tabela vira
o `W` (os nomes das colunas iguais aos do `W`; o dialeto do `sql_filter` e o
do `FileQuery` precisam ser conferidos).

## Andamento (2026-11-22): etapa 4, a Tabela

Feita no branch `views-table`, para entrar na 0.9.0 (pedido do autor no dia do
release: "primeiro vamos terminar o que falta").

- **A medição** (pedida acima), com `cargo test --release -- --ignored --nocapture
  list_of_two_million`, numa máquina de 14 GB: **2.000.000 de arquivos chegam à tela
  em 150 ms, com 278 MiB a mais** (146 bytes por arquivo, quase o mesmo que a árvore
  já gasta com eles). Fica como está: o retrato compacto descrito abaixo pouparia uns
  30%, ao custo de reescrever os valores do `W`.
- **Como ficou:** `src/viewer/list_rows.rs`. `ListSnapshot` guarda cada arquivo como
  `ListFile` (o `FileEntry`, o número da pasta, o Git e o grupo de duplicados), e as
  pastas uma vez cada. `ListRows` é o `RowSource` da janela de tabelas, e as células
  só são feitas quando a janela pede a linha, por `query::file_value`, a mesma função
  do `W`. Por isso o filtro da tabela e o `W` concordam.
- **As colunas:** as do `W` menos `type` (lê cada arquivo), `path` e `depth` (`dir` já
  diz). `git` e `dup` só aparecem quando algum arquivo tem um valor.
- **A tecla é `L`** no seletor (T já era a Linha do tempo), e `view_table` em `[keys]`.
- **De volta à lista: decidido assim, sem consulta, para caber no dia.** `W` na
  tabela leva o filtro SQL dela para a lista como condição do `W`, com AND à que
  havia, como a Linha do tempo faz. Sem filtro, uma nota diz como fazer um.
  **`T` para marcar na tabela ficou de fora:** as marcas da tabela (`m`) continuam
  servindo ao Batch, e marcar arquivos se faz na lista, depois do `W`, com Ctrl+T.
- **O Batch** (Ctrl+B) lê as linhas pela mesma fonte: `batch::Source::Table` ganhou
  `rows`, que, quando existe, substitui a leitura do arquivo.
- **Não há ordenação na tabela:** a grade não ordena por coluna (nenhuma tabela
  ordena hoje). A tabela segue a ordem da lista (Alt+S antes de abrir).
- **Testes:** `list_rows` (valores, colunas, contagem), `views_tests` (abrir, filtrar,
  `W` de volta) e `batch` (Batch sobre a lista).
