# Comparações

## Um agradecimento

Uso alguma variante do XTree desde os anos 1990 e tenho uma licença paga do
ZTreeWin desde 2013. Para mim, o ZTreeWin continua sendo o melhor gerenciador
de arquivos em modo texto que existe: rápido, previsível, todo no teclado, e
construído em torno de um jeito de trabalhar (ler a árvore, marcar arquivos em
ramos inteiros, agir sobre as marcações) que nenhum outro gerenciador de
arquivos igualou. Kim Henkel mantém vivo o jeito XTree no Windows há décadas,
e o Xylem não existiria sem esse trabalho.

O Xylem não pretende substituir o ZTreeWin. Ele existe porque eu queria o
mesmo jeito de trabalhar no Linux, onde o ZTreeWin não roda nativamente. O
Xylem também compila para Windows, porque o Rust torna isso fácil, mas essa
não é uma meta minha: no Windows, o ZTreeWin vale muito a licença:
[ztree.com](https://www.ztree.com/), e o [fórum](https://groups.io/g/ZTreeWin)
dos seus usuários.

## Outros gerenciadores de arquivos para o terminal

Há muitos bons gerenciadores de arquivos para o terminal, e a maioria de quem
experimenta o Xylem já usa um. É assim que eles se diferenciam, da forma mais
justa que consigo. Conferido na documentação de cada projeto em novembro de
2026; esses programas mudam rápido, então, se algo aqui estiver
desatualizado, abra uma issue.

| Programa | Disposição | Conhecido por |
|---|---|---|
| ranger | três colunas (pasta de cima, atual, pré-visualização) | teclas do vi, pré-visualizações por scripts, Python para estendê-lo |
| lf | três colunas, como o ranger | um único binário pequeno, rápido, comandos do shell como extensão |
| yazi | três colunas | velocidade (tudo assíncrono), pré-visualização de imagens, plugins em Lua com gerenciador de pacotes, SFTP embutido |
| superfile | vários painéis lado a lado | visual moderno, pré-visualização de imagens, temas |
| nnn | uma lista | minúsculo e rápido, seleção entre diretórios, um modo de uso do disco, muitos plugins |
| Midnight Commander (mc) | dois painéis | o clássico: visualizador e editor próprios, SFTP e arquivos compactados como pastas, os resultados do Find file num painel |
| vifm | dois painéis | teclas e comandos do vim, os resultados do `:find` como uma lista para trabalhar, uma visão em árvore |
| broot | uma árvore que cabe na tela | busca aproximada na árvore inteira, tamanhos, uma área de preparação (staging) para agir sobre arquivos de várias pastas |
| far2l | dois painéis | o Far Manager para Linux, com seus plugins: SFTP, SMB, NFS, WebDAV e S3 (NetRocks), um editor hexadecimal, scripts em Python |

**O que diferencia o Xylem.** A árvore é lida para a memória, e uma tecla
mostra numa lista só os arquivos de um ramo inteiro, ou de tudo o que foi lido
(Branch, Showall), filtrados por nome, data e tamanho (Filespec) ou por uma
condição SQL (`W`), com os bytes que combinam e os marcados contados enquanto
você trabalha. Marque ali o que quiser e copie, mova, renomeie ou exclua tudo
de uma vez. Vários dos programas acima também juntam numa lista arquivos de
muitas pastas (o panelize do mc, as visões personalizadas do vifm, os
resultados de busca do yazi, a área de preparação do broot), ou mantêm uma
seleção entre pastas (nnn, yazi); no Xylem essa lista é o jeito de trabalhar o
tempo todo, não o resultado de uma busca. Além disso, nenhum dos outros tem no
mesmo programa:

- tabelas com filtro SQL para planilhas, CSV, SQLite (com `JOIN`) e Parquet,
  com os registros transformados num script, numa página ou num banco de dados
  (Batch). Um plugin do yazi feito com o DuckDB mostra uma prévia e um resumo
  de arquivos CSV e Parquet, sem filtro, consulta ou exportação;
- relatórios sobre arquivos em que você não confia: as macros, os templates
  remotos e o texto oculto de documentos do Office, as proteções e os sinais de
  empacotamento de executáveis, os nomes a conferir num patch, sem rodar nada
  do que há neles;
- arquivos e pastas duplicados, até no Google Drive, sem baixá-los;
- um terminal integrado cuja árvore acompanha o `cd` do shell;
- arquivos de fora tratados como hostis em toda parte, com limite em tudo o que
  é lido ([Segurança](security.md)).

**Onde os outros estão à frente.** Pré-visualização de imagens (yazi,
superfile, ranger). Um sistema de plugins e os plugins já escritos para ele
(yazi, nnn, ranger, lf, far2l): o Xylem não tem, só o eXecute e o Batch.
Teclas do vi desde o início (ranger, lf, yazi, vifm), onde o Xylem tem seus
próprios comandos de uma letra. Anos de uso por muita gente, e versões para
macOS e Windows testadas, enquanto o Xylem é novo, não foi testado num Mac e é
experimental no Windows.

## ZTreeWin

Esta parte compara os dois comando por comando, com base no manual oficial do
ZTreeWin v2.2 ([ztwhlp.pdf](https://www.zedtek.com/download/ztwhlp.pdf), de
2011, em especial a "Quick Command Table" e a seção 3.7 sobre configuração).
Versões mais recentes do ZTreeWin podem ter mudado alguns detalhes; se algo
aqui estiver desatualizado, abra uma issue.

### O mesmo jeito de trabalhar

Árvore com leitura sob demanda (`+`, `*`), janela de arquivos, Filespec, Branch,
Showall (também só dos marcados, Ctrl+S), marcações (T, U, I, Ctrl+T/U),
Copiar/Mover/Excluir/Renomear com as versões Ctrl (marcados) e Alt (marcados com
caminhos), Criar diretório, Excluir diretório, Prune, Atributos, Visualizador,
Edit, eXecute, Comparar (diretório e ramo), Ordenar, Log, tela dividida (F8 e
Tab), reler (F3), Abrir (`O`), atualização automática, navegação e extração de
arquivos compactados, histórico nos prompts, favoritos, ajuda (F1) e tela de
configuração. As teclas são as do ZTreeWin sempre que possível; as exceções
estão em "Teclas diferentes", no fim.

### O que o Xylem ainda não tem

- **Árvore:** Global (G, e Ctrl+G para os marcados), Graft (Alt+G), Mirror (Alt+M), liberação parcial (Ctrl+Backspace), Hide (Alt+H), Wash (Alt+W), Volume
  (Ctrl+V), F5/F6 (ocultar níveis de subdiretórios), marcar por atributo
  (Alt+T/U), Treespec em modo de edição (F5, curingas) e pastas especiais (F6),
  busca ao digitar com prompt (`|`) e modo ativo, troca de unidade (`<` e `>`).
- **Arquivos:** Abrir em nova janela de terminal (Alt+O), Batch do arquivo
  atual (Alt+B), JFC (J, comparar o conteúdo de dois arquivos), sHortcut (H),
  Print (P), dono/grupo (chown), Megaview (Ctrl+F7 na janela de arquivos),
  mesclar arquivos marcados entre janelas (Ctrl+F6), Partial Untag (Ctrl+F8),
  Tags-Only fora do Showall (Ctrl+Enter e Ctrl+F4, Ctrl+B na árvore).
- **Geral:** macros (F12), menu de aplicações (F9), Sair e abrir shell
  (Ctrl+Q), salvar e carregar o log (Zlist, Ctrl/Alt+Z), cores por nome e
  atributo (ZCOLORS.INI), opções de linha de comando do ZTreeWin,
  comandos de vídeo (Alt+F6–F9).

Para alguns deles há um substituto em outro programa; veja "O que o Xylem
deixa para outros programas", mais abaixo.

### Onde o Xylem é mais simples

| Recurso | No ZTreeWin | No Xylem |
|---|---|---|
| Filespec | nome com regras próprias de nome e extensão, data, tamanho, atributo, filespec invertido (Ctrl+I), rótulos | glob para nomes, data e tamanho como no ZTreeWin (com K/M/G); sem atributos, inversão ou rótulos |
| Visualizador | modos Alpha, Dump, Hex, Junk e Wrap; conjuntos de caracteres; edição hexadecimal; favoritos; rolagem automática | texto, hexadecimal, tabelas com filtro SQL e Markdown renderizado, com a busca do ZTreeWin (F, B, /, \\, Space, + e -), quebra de linha (W), números de linha (#), ir para linha (g / :), filtro de linhas com regex (&); um arquivo no disco de qualquer tamanho, lido conforme a tela anda; sem edição hexadecimal, por opção (veja "O que o Xylem deixa para outros programas") |
| Ctrl+S | texto, hexadecimal ou Unicode; busca apenas no índice do ZIP | texto ou hexadecimal |
| Atributos | atributos do Windows (R, A, S, H) | permissões Unix em octal (`644`); sem dono/grupo. No Windows, R, H, S e A, como no ZTreeWin |
| Ordenar | Name, Ext, Date, Time, Length, Alpha, nuMber, Size, Value, Unsorted; Charset; igual/diferente na tela dividida | Name, Ext, Date, Size, crescente/decrescente, Path primeiro; cada janela tem sua própria ordem |
| Comparar | opções Identical, Unique, Newer, Older, Size, Binary, Case; filtro de exibição | marca únicos, mais novos, mais antigos, idênticos, diferentes ou únicos+mais novos; sem comparação binária |
| Prune | exclui apenas os subdiretórios ou também o atual; apenas diretórios vazios; lixeira | sempre exclui o diretório inteiro, permanentemente; confirmação configurável |
| Log | várias unidades lidas, Global, troca de unidade | uma raiz por janela; `L` abre outro diretório |
| eXecute | parâmetros de lote `%1`–`%8`, nova janela (NTW) | `sh` com `%f`, `%t` e `%d`; sempre no terminal do Xylem |
| Histórico | janela de histórico com favoritos | ↑ no prompt abre uma lista (digitar filtra, Del esquece), um histórico por tipo, salvo entre sessões |
| Configuração | 7 páginas, editor de cores por elemento (F4), Alt-Export | 4 páginas com as opções que fazem sentido no Linux; F4 escolhe entre 12 temas prontos; cores individuais em `config.toml` |
| Tela dividida | painéis opcionais de estatísticas por lado | sem painel de estatísticas |
| Menus | menu Ctrl/Alt ao segurar a tecla | igual em terminais com o protocolo do kitty; F9 em qualquer terminal |

Opções de configuração do ZTreeWin que não foram trazidas por serem específicas
do Windows: atributos RSH, junções (*junctions*), OEM/ANSI, EBCDIC, tamanho da
janela, atalhos (.lnk).

### Onde o Xylem vai além

Coisas para as quais o manual do ZTreeWin v2.2 não tem equivalente:

- **Um terminal integrado** (Alt+X): o shell continua rodando enquanto você usa
  a árvore; um `cd` no shell leva a árvore junto, e o shell acompanha a árvore
  quando você volta a ele. Alt+O digita os arquivos marcados no prompt. Veja
  [o terminal integrado](terminal.md).
- **Máquinas remotas via SSH** (`xy usuario@host:`): a árvore, o Branch, o
  visualizador e o Copiar funcionam em outra máquina, que também tem seu
  próprio terminal. Veja [máquinas remotas](remote.md).
- **Armazenamento na nuvem:** Google Drive, S3 e os outros serviços que o rclone suporta, montados e mostrados como uma árvore. Veja [armazenamento na nuvem](cloud.md).
- **SQL na lista de arquivos** (`W`): `ext = 'jpg' AND size > 10000000` mantém os arquivos que atendem à condição, e `contains()` e `words()` olham dentro deles; a caixa de busca (`P`) encontra arquivos pelas palavras que contêm. Veja [encontrar arquivos](sql.md).
- **Git** (opcional): letras de status na lista de arquivos, um menu Git
  (Ctrl+G) e diffs no visualizador. Veja [Git](git.md).
- **Uso do disco** (Alt+U): pastas e arquivos de um ramo juntos, os maiores
  primeiro, até chegar ao que enche o disco; Alt+K mostra a fatia de cada pasta
  na árvore. Veja [uso do disco](usage.md).
- **Duplicados** (Alt+D): arquivos e pastas inteiras com o mesmo conteúdo,
  comparados por SHA-256, ou pelos hashes do próprio serviço numa pasta montada
  pelo rclone, sem baixar nada. Uma cópia de cada grupo sempre fica. Veja
  [duplicados](duplicates.md).
- **Um visualizador que lê mais:** cores de sintaxe, a codificação do texto
  detectada sozinha (UTF-8, UTF-16, CP850, Windows-1252, CP437), arquivos
  `.gz`, `.xz`, `.zst` e `.bz2` mostrados descompactados, e caracteres ocultos
  de direção do texto (Trojan Source) mostrados em vez de obedecidos. Veja
  [o visualizador](viewer.md).
- **Tabelas e Markdown:** planilhas, CSV, JSON Lines, DBF, SQLite e Parquet
  abrem numa grade com filtro SQL, e arquivos Markdown abrem renderizados. Veja
  [tabelas](tables.md).
- **Arquivos compactados sem compactador externo:** ZIP, 7z e TAR (puro, gzip,
  xz, bzip2, zstd) abrem como pastas, e ZIP e TAR são criados, tudo pelo
  próprio Xylem. Veja [arquivos compactados](archives.md).
- **Renomear em lote** com busca e substituição (`s/foo/bar/`), contadores e
  mudança de maiúsculas/minúsculas. Veja [renomear](rename.md).
- **Batch sobre registros:** Ctrl+B no visualizador transforma os registros de
  uma tabela ou as linhas de um texto em script, JSON, CSV, XML, HTML, SQL ou um
  banco SQLite, com arquivos de template se você quiser. Veja [Batch](execute.md).
- **Oops:** O (ou Ctrl+Z) devolve ao lugar de origem o que foi enviado por
  último para a lixeira do sistema. Veja [excluir](delete.md).
- **A área de transferência em todo lugar:** Y copia o caminho mesmo via SSH
  ou dentro do tmux (OSC 52).
- **O que é cada arquivo:** um modo de exibição (Alt+F) que diz, no espírito
  do comando `file`, o que cada arquivo contém, e outro com donos e permissões.
- **Aparência:** 12 temas prontos e ícones Nerd Fonts opcionais. Toda tecla
  pode ser redefinida em `[keys]` no `config.toml`. Veja
  [configuração](configuration.md).
- **Arquivos hostis:** arquivos compactados, nomes e conteúdos são tratados
  como potencialmente hostis, com limites em tudo o que é lido e descompactado
  (veja a declaração de segurança no README).
- **Ajuda em seis línguas** (F1, depois L), o mesmo texto deste manual e da
  página man.

### O que o Xylem deixa para outros programas

Algumas coisas o Xylem não faz sozinho, de propósito: um programa dedicado as
faz melhor, e o Xylem o traz para dentro.

- **Edição hexadecimal:** o ZTreeWin edita bytes no próprio visualizador. O
  Xylem não tem um editor hexadecimal próprio: escrever e manter um seria muito
  trabalho para pouco uso, e editar bytes à mão é um jeito fácil de corromper
  um arquivo. Em vez disso, E num arquivo binário abre o `hexedit`, `heh`,
  `bvi` ou `dhex` (o que estiver instalado, ou o que você escolher) dentro da
  moldura e das cores do Xylem, ou um gráfico como o ImHex ou o GHex. Veja
  [Edit](execute.md).
- **Editar texto:** o seu editor (`$VISUAL`, `$EDITOR` ou o definido no F12),
  e um editor diferente por tipo de arquivo, se você quiser (`[editors]`).
- **SSH:** o `ssh` do sistema, então suas chaves, o agente, o `~/.ssh/config`
  e os *jump hosts* funcionam como em qualquer outro lugar.
- **Armazenamento em nuvem:** Google Drive, S3 e outros via `rclone mount`; o
  Xylem reconhece a montagem e se adapta. Veja
  [armazenamento em nuvem](cloud.md).
- **Git:** o próprio programa `git`.
- **Abrir arquivos** (O): o programa do ambiente gráfico para cada tipo
  (`xdg-open`).
- **Comparar o conteúdo de dois arquivos** (JFC no ZTreeWin): enquanto não
  estiver embutido, marque os dois arquivos e rode `diff %t` com o eXecute (X),
  ou use o terminal.
- **Imprimir** (P no ZTreeWin): `lp %t` com o eXecute.
- **Outros formatos de arquivo compactado** (o ZAAP do ZTreeWin): o seu
  compactador, com o eXecute ou no terminal.

### Teclas diferentes

As teclas seguem as do ZTreeWin. As exceções:

| Função | ZTreeWin | Xylem |
|---|---|---|
| Ajuda | F1 | F1 ou Alt+H (no GNOME Terminal, F1 abre a ajuda do terminal) |
| Configuração | Alt+F10 | F12 ou Alt+F10 (no GNOME, Alt+F10 maximiza a janela e nunca chega ao Xylem) |
| Favoritos | F11 | F11, Alt+B ou aspas simples `'` (no GNOME Terminal, F11 alterna a tela cheia) |
| Mover marcados | Ctrl+M | Ctrl+M em terminais com o protocolo do kitty; nos demais, `F9` `M` (ou Alt+M e F6) |
| F2 | (nada) | relê o diretório, igual a F3 |
| F9 | menu de aplicações (não implementado) | menus Ctrl/Alt |
| F12 | macros (não implementado) | configuração |
| Alt+B | Batch do arquivo atual (não implementado) | favoritos |
| Alt+H | Hide (não implementado) | ajuda |
| Alt+O | Abrir em nova janela de terminal (não implementado) | digita os arquivos marcados no terminal embutido |
| Alt+D | (nada) | Duplicados |
| Alt+T | marcar por atributo (não implementado) | (nada) |
| Alt+U | desmarcar por atributo (não implementado) | Uso, para onde vai o espaço |
| Ctrl+G | Global dos marcados (não implementado) | o menu Git (opcional) |
| Alt+Z | Zlist (não implementado) | o terminal embutido em tela cheia |
| `/` | igual a `?` (estatísticas) | Treespec, como `\` (`?` continua mostrando as estatísticas) |
| Ctrl+C, Ctrl+M na árvore | (nada) | Copiar/Mover os arquivos marcados do diretório atual |
| Copiar caminho para a área de transferência | Ctrl+Insert | Y ou Ctrl+Insert (Ctrl+Y para os marcados; funciona via OSC 52 e área de transferência do sistema) |

Todas as teclas podem ser redefinidas sob `[keys]` em `config.toml`.

## Nomes

O Xylem é um projeto independente, escrito do zero. Não é afiliado, endossado
nem ligado à Zedtek, Inc. (ZTreeWin), aos donos do XTree ou aos autores dos
outros programas citados nesta página, e não contém código de nenhum deles. Os
nomes aparecem só para descrever o que inspirou o Xylem e como ele se compara.
XTree, ZTree e ZTreeWin são marcas de seus respectivos donos.
