# Notas de versão

A versão mais recente primeiro. Cada seção lista o que há de novo, o que mudou
para quem usava a versão anterior e as teclas que mudaram. As notas completas
de cada versão estão em `https://github.com/andrelnfernandes/xylem/releases`.

## 0.9.0

**Novidades.**

- Os Norton Guides (`.ng`), os arquivos de ajuda do DOS e do Clipper, abrem
  no visualizador como documento: os menus, depois cada entrada numerada
  (`§12`), com seus links e seu see-also apontando para esses números. Tab
  escolhe um link, Enter ou um clique o segue, Backspace volta.
  [O visualizador](viewer.md)
- Busca aproximada (`|`): digite pedaços de um nome (`cfgxy` encontra
  `config/xylem`) e o cursor vai para a melhor pasta, na árvore, ou o melhor
  arquivo da lista, na janela de arquivos. Os resultados aparecem enquanto
  você digita, o melhor primeiro, mesmo numa árvore de milhões de pastas.
  Ctrl+T marca o resultado escolhido e Alt+T a lista.
  [Busca aproximada](fuzzy.md)
- [Encontrar arquivos e pastas](finding.md): uma página que diz qual busca
  responde a qual pergunta.
- Ebooks: EPUB e FictionBook (`.fb2`, `.fb2.zip`) abrem como um livro para
  ler, numa coluna com os parágrafos de um livro impresso: uma folha de rosto,
  Space para a próxima página, `]` `[` para os capítulos, Tab para o sumário,
  Enter para as notas chamadas na página, `g 40%` para uma parte do livro, e
  quanto falta do capítulo e do livro. `I` mostra o relatório do arquivo: o
  que o livro diz de si, e se ele traz scripts, coisas buscadas de fora ou
  DRM. [O visualizador](viewer.md#ebooks)
- `R` num livro lembra onde você parou, e o livro abre ali na próxima vez. Se
  você ligar (página 5 do F12), o Xylem também guarda o tempo que você passou
  lendo cada livro lembrado, as suas sessões e a sua velocidade de leitura,
  só no seu computador. [Tempo de leitura](configuration.md#reading-time)
- O F1 abre a ajuda na seção do que você está fazendo: no Batch do
  visualizador (Ctrl+B), em "Batch no visualizador", não no topo da página.
  Links para uma seção de uma página da ajuda agora levam até ela.
- O Xylem pode avisar que saiu uma versão nova. Na primeira vez que abre, ele
  pergunta se pode procurar uma vez por dia; com um sim, busca no site só o
  número da versão mais recente, e a linha do topo diz quando sai uma mais
  nova. Nada é baixado ou instalado. [Versões
  novas](configuration.md#new-versions)
- Visões (Alt+V): outras formas de olhar a lista. A Varredura (`S`) aponta o
  que numa pasta merece um olhar antes de você abri-la: nomes com caracteres
  ocultos ou extensão dupla (`fatura.pdf.exe`), arquivos setuid e o que
  qualquer um pode gravar, links que saem da pasta ou apontam para nada, e,
  lidos com os próprios leitores e limites do visualizador, executáveis sem
  PIE ou NX ou empacotados, documentos do Office com macros ou modelo remoto,
  patches que sobem para fora com `..`. Enter numa categoria traz os arquivos
  dela de volta para a lista, para o Ctrl+T e o resto. A Linha do tempo (`T`)
  mostra os arquivos da lista num histograma por data, por horas, dias,
  semanas, meses ou anos; Enter numa barra, ou num intervalo, deixa só os
  arquivos dela, como uma condição `W`. A Tabela (`L`) abre a lista no
  visualizador de tabelas, um arquivo por linha com as colunas do `W`, mesmo
  uma lista de milhões de arquivos; `W` ali leva o filtro SQL dela de volta
  para a lista. [Visões](views.md)

**Mudanças.**

- `O` no visualizador passa por mais duas codificações, Windows-1251 e
  KOI8-R (cirílico), antes do UTF-16.
- A exibição de tipo (Alt+F) diz `EPUB ebook (ZIP)`, `FB2 ebook` e
  `FB2 ebook (ZIP)`.

**Correções.**

- Com o Git ligado, um repositório grande num sistema de arquivos lento
  (`/mnt/c` no WSL, um compartilhamento de rede) congelava a tela enquanto o
  `git status` rodava, por minutos. O status agora é lido em segundo plano, e
  um repositório lento é relido com menos frequência. [Git](git.md)
- A biblioteca que o Xylem usa para chamadas de sistema (rustix 1.1.6) foi
  atualizada por causa de um alerta sobre atributos estendidos
  ([RUSTSEC-2026-0486](https://rustsec.org/advisories/RUSTSEC-2026-0486)).

## 0.8.0

**Mudança: algumas teclas Alt.** O Alt não repete mais o Ctrl. Ele passa a ser
a outra versão de um comando.

| Tecla | Até a 0.7 | A partir da 0.8 |
|---|---|---|
| Alt+A | Attributes dos arquivos marcados | nada (Ctrl+A) |
| Alt+D | exclui os arquivos marcados | Duplicates (Ctrl+D os exclui) |
| Alt+R | renomeia os arquivos marcados | Release do ramo, como `-` (Ctrl+R os renomeia) |
| Alt+T | marca tudo | nada (Ctrl+T) |
| Alt+U | desmarca tudo | Usage (Ctrl+U desmarca) |
| Alt+W | Usage | nada (Alt+U) |
| Alt+Y | Duplicates | nada (Alt+D) |

Para manter as teclas antigas, inclua-as no `config.toml`, uma a uma ou todas:

```toml
[keys.any]
"alt+a" = "attributes_tagged"
"alt+d" = "delete_tagged"
"alt+r" = "rename_tagged"
"alt+t" = "tag_all"
"alt+u" = "untag_all"
"alt+w" = "usage"
"alt+y" = "duplicates"
```

**Mudança: o rodapé.** Todo comando agora pertence a um menu, e o rodapé tem
três linhas, ou duas em uma tela com largura suficiente para comportar cada menu
em linha única. Em telas estreitas, as palavras são abreviadas antes que
qualquer comando seja omitido. Dentro de arquivos compactados ou em máquinas
remotas, os menus listam apenas o que funciona naquele ambiente.
[Os menus](menus.md)

**Mudança: prompts.** ↑ abre o histórico de um prompt como uma lista; ↓ não
percorre mais as entradas individualmente. `B` em um Branch ou Showall volta à
lista própria do diretório. A janela de arquivos reduzida ocupa um quarto da
altura da tela.

**Novidades.**

- `W` mantém na lista os arquivos que atendem a uma condição SQL; Ctrl+W filtra
  dentre os arquivos marcados. `contains('texto')` lê o conteúdo dos arquivos e
  `words('texto')` consulta o índice construído com Alt+N.
  [Uma condição SQL](sql.md#uma-condição-sql-w)
- `P` abre uma caixa de busca que lista os arquivos pelas palavras digitadas.
  [A caixa de busca](sql.md#a-caixa-de-busca-p)
- Tabelas CSV, TSV, JSON Lines, DBF e SQLite não têm mais limite de linhas no
  visualizador. Planilhas mantêm o limite de 50.000 linhas.
  [Tabelas](tables.md)
- Numa tabela, Enter mostra o registro como ficha, e `c` o copia; `c` também
  copia a estrutura mostrada por `i`. [Tabelas](tables.md)
- Um texto acima de 16 MiB é lido por uma janela, sem limite de tamanho.
  [Arquivos grandes](viewer.md#arquivos-grandes)
- Visualizadores novos: JSON, YAML e TOML como uma árvore que se dobra; logs
  com as cores dos seus níveis, e acompanhamento; patches com um índice de
  arquivos e hunks e o que conferir; notebooks do Jupyter; documentos do
  Office, com suas macros, modelos remotos e texto oculto; executáveis (ELF,
  PE, Mach-O) e certificados e chaves como relatórios. [O visualizador](viewer.md)
- Compila no macOS (ainda não testado num Mac).
- Alt+L torna o diretório sob o cursor a raiz da árvore.
- No Treespec, Ctrl+← remove a última pasta do caminho e Ctrl+→ a restaura.
- `clock_seconds` exibe os segundos no relógio do cabeçalho.
- O mouse, desligado por padrão (F12, página 2, R): a roda, cliques e duplos
  cliques na árvore, nas listas e nas teclas do rodapé, marcar com o botão do
  meio, um menu no botão direito (também a tecla Menu e Shift+F10), e o mouse
  repassado aos programas do terminal integrado e do editor na moldura.
  [O mouse](mouse.md)
- Este manual: páginas sobre a tela, os menus, os comandos de cada menu, o
  vocabulário e as páginas de receitas.

## 0.7.0

**Mudança: leitura de diretórios (Log).** Um diretório é lido apenas quando
solicitado. Mover o cursor sobre um diretório não lido exibe `Not logged`;
`+` ou → realiza a leitura. Anteriormente, passar pelo diretório realizava a
leitura automática. [A janela da árvore](tree.md)

**Mudança: prompts.** Log, e Copy e Move sem tela dividida, abrem vazios.
↑ lista os locais utilizados anteriormente.

**Novidades.**

- Máquinas remotas deixam de ser somente leitura: Copy para elas, Move,
  Rename, Makedir, Delete, Prune, Attributes, New date e Duplicates passam a
  funcionar lá. [Máquinas remotas (SSH)](remote.md)
- `docker://container/caminho` abre um container via `docker exec`.
- Favoritos podem receber nomes, e um favorito em máquina remota retém o
  endereço de conexão. F11 nos prompts de Log, Copy e Move insere um favorito
  no prompt. [Favoritos](bookmarks.md)
- Pastas montadas via rclone e compartilhamentos de rede são lidos em segundo
  plano; a árvore é exibida de imediato.
  [Armazenamento na nuvem (rclone)](cloud.md)
- A tela de Uso permite marcar, excluir, remover ramos (Prune) e visualizar
  diretamente de seu interior.
- Tela inicial, nome e versão no cabeçalho, nomes de arquivo coloridos por tipo
  e bordas arredondadas. [Configuração](configuration.md)

## 0.6.1

Atualiza o leitor de planilhas (calamine 0.38) devido ao comunicado de segurança
RUSTSEC-2026-0436.

## 0.6.0

**Novidades.**

- O visualizador exibe planilhas, CSV, JSON Lines, DBF, SQLite e Parquet como
  tabelas com filtro SQL. Bancos SQLite também aceitam consultas completas de
  somente leitura. [O visualizador](viewer.md)
- Ctrl+B no visualizador grava um arquivo a partir dos registros de uma tabela
  ou linhas de texto: JSON, CSV, XML, HTML, um modelo (*template*) ou tabela
  SQLite. [Batch](execute.md)
- Duplicates localiza arquivos e pastas iguais, em discos locais e por meio dos
  hashes informados pelo rclone. A tecla era Alt+Y; passa a ser Alt+D a partir da
  0.8. [Duplicados](duplicates.md)
- Arquivos Markdown abrem renderizados no visualizador. `T` exibe o código-fonte.
- O visualizador filtra linhas com expressão regular (`&`).
  [Filtrar linhas](filter.md)
- `*` em pasta montada via rclone solicita o ramo inteiro em uma única
  requisição.
- Olhar um repositório Git não executa nada que o repositório indique, exceto se
  a pasta constar em `git_trusted`. [Git](git.md)

## 0.5.0

**Mudança.** `-` na árvore libera o ramo da memória; ← apenas o fecha. `E`
abre o editor dentro da moldura do Xylem.

**Novidades.**

- Medidas de segurança para arquivos hostis: limites de descompressão,
  verificação de caminhos extraídos, caracteres de controle barrados antes do
  terminal. [Segurança](security.md)
- O visualizador quebra linhas (`W`), numera linhas (`#`), pula para uma linha
  (`g`), exibe arquivos `.gz`, `.xz`, `.zst` e `.bz2` descomprimidos e
  reconhece CP437, CP850, Windows-1252 e UTF-16.
- Ajuda em seis idiomas; `L` na ajuda alterna entre eles.
- Ctrl+S na árvore lista apenas os arquivos marcados de tudo o que foi lido.
- Opção Graph no menu Git.
- Ícones com Nerd Fonts, desativados por padrão.
- Pacotes `.deb` e `.rpm`.

## 0.4.0

**Novidades.**

- Tela de Uso: pastas e arquivos de um ramo ordenados pelo espaço que ocupam.
  A tecla era Alt+W; passa a ser Alt+U a partir da 0.8.
  [Uso do disco](usage.md)
- Alt+K Percent exibe a fatia percentual de cada diretório em relação ao
  diretório pai.
- Máquinas remotas via SSH, somente leitura nesta versão.
  [Máquinas remotas (SSH)](remote.md)
- Google Drive, S3 e outros serviços de armazenamento via rclone.
  [Armazenamento na nuvem (rclone)](cloud.md)
- Temas slate, graphite, daylight e vivid.
- `[editors]` no `config.toml` escolhe o editor pelo nome do arquivo, e `E`
  recusa arquivos binários que não possuem editor configurado.
- `xy --man` exibe a página de manual.

## 0.3.0 e anteriores

- 0.3.0: ajuda em F1 e página de manual; arquivos compactados 7z, tar.xz,
  tar.bz2 e tar.zst; modos de exibição por proprietário e tipo (Alt+F); exibição
  de ambos os arquivos em caso de conflito de cópia.
- 0.2.0: mudança do nome do programa de xt para Xylem, e do comando para `xy`.
  Exclusão para a lixeira e Oops, renomeação em lote com máscara e suporte à
  área de transferência.
- 0.1.0: primeira versão, lançada sob o nome xt.
