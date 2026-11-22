<img src="docs/icons/xylem.svg" width="96" align="right" alt="Ícone do Xylem">

# Xylem

[Site](https://andrelnfernandes.github.io/xylem/pt/) · [Read in English](README.md)

Um gerenciador de arquivos para o terminal que lê os seus dados com SQL e abre os
arquivos em que você não confia sem executar nada deles. Feito do jeito do XTree:
o disco como uma só árvore, arquivos de muitas pastas reunidos numa lista, e uma
tecla para agir sobre todos.

> **Antes da 1.0.** O Xylem ainda não chegou à versão 1.0. Até lá, comandos
> podem mudar de menu, teclas podem mudar e opções podem ganhar outro nome.
> A ideia é manter a familiaridade dos comandos antigos enquanto chegam
> recursos novos, e às vezes isso pede mudar um atalho. As
> [notas da versão](docs/manual/releases.md) dizem o que mudou em cada
> versão, com as teclas que mudaram de lugar.

![O Xylem em uso: o terminal embutido com a árvore acompanhando um cd, arquivos marcados num Branch e digitados num comando com Alt+O](docs/screenshots/demo-intro.gif)

## Por que o Xylem?

| Você quer... | O jeito de sempre | No Xylem |
|---|---|---|
| olhar dentro de um CSV, uma planilha, um SQLite ou um Parquet | outro programa, ou um plugin de prévia | uma grade com filtro SQL, `JOIN` no SQLite, e os registros transformados em script, página ou banco de dados |
| conferir um arquivo em que você não confia | abrir e torcer, ou várias ferramentas | um relatório do que ele esconde: as macros e os templates remotos de um arquivo do Office, as proteções que faltam e as seções empacotadas de um executável, os nomes enganosos de um patch; nada nele é executado |
| agir sobre arquivos espalhados por muitas pastas | entrar em cada pasta, ou fazer uma busca e trabalhar no resultado | `B` lista todos os arquivos de um ramo numa lista só: filtre, marque, e copie, mova ou apague todos de uma vez |
| liberar espaço no Google Drive | baixar os arquivos para compará-los | duplicados achados pelos hashes que o Drive já guarda, sem baixar nada |

Como ele se compara com ranger, yazi, nnn, mc, broot e outros, inclusive onde eles
estão à frente: [Comparações](docs/manual/pt/comparison.md) (por enquanto, a
versão em inglês está mais completa: [Comparisons](docs/manual/comparison.md)).

## Início rápido (30 segundos)

```bash
cargo install xylem-fm      # exige Rust e um compilador C
xy
```

Sem Rust? A
[release mais recente](https://github.com/andrelnfernandes/xylem/releases/latest)
tem binários estáticos para Linux (x86_64 e ARM64), pacotes `.deb` e `.rpm`, e
uma versão experimental para Windows. Os detalhes estão em [Instalação](#instalação).

Cinco teclas para começar:

| Tecla | O que faz |
|---|---|
| `B` | Branch: todos os arquivos do diretório sob o cursor e de tudo abaixo dele, numa lista só |
| `F` | Filespec: deixa na lista só os arquivos que combinam, como `*.pdf` |
| `T` | marca o arquivo sob o cursor (Ctrl+T marca todos os da lista) |
| Ctrl+C | copia os arquivos marcados para uma pasta (Ctrl+D os apaga, depois de perguntar) |
| F1 | ajuda sobre onde você está (Alt+H onde o terminal fica com o F1) |

`Q` sai. O rodapé sempre mostra as teclas que funcionam onde você está.

## O que ele faz

- **Abra quase tudo ali mesmo:** SQLite com queries completas com `JOIN`, planilhas,
  CSV, Parquet, JSON, Markdown, logs e arquivos compactados.
- **Transforme dados em arquivos:** uma tabela vira página, script SQL ou banco de
  dados, com prévia antes de escrever qualquer coisa.
- **Libere espaço:** veja o que ocupa mais e ache arquivos e pastas duplicados, até no
  Google Drive sem baixar nada.
- **Vá além do seu disco:** trabalhe em servidores via SSH e em containers como no
  seu próprio disco, e com Google Drive, S3 e outras nuvens via rclone.
- **Fique num lugar só:** um terminal embutido com a árvore acompanhando o seu `cd`, e
  o status do Git na própria lista de arquivos.
- **Abra arquivos em que você não confia:** nada neles é executado, e tudo tem limite.
- **Deixe do seu jeito:** doze temas, teclas configuráveis e ajuda em seis línguas.

Em mais detalhe:

- **Uma árvore, muitos arquivos:** uma árvore de diretórios logada na memória, a janela de
  arquivos embaixo, Branch e Showall para listar os arquivos de um ramo
  inteiro ou de tudo o que foi logado, e tags sobre as quais agem as operações
  em grupo.
- **Operações com arquivos:** copiar e mover (com ou sem os caminhos, com
  progresso e detalhes dos conflitos), apagar para a lixeira com Oops para
  restaurar, Prune, renomear com máscaras, permissões e datas, Compare entre
  duas janelas.
- **Achar coisas:** Filespec por nome, data e tamanho, Treespec para digitar um
  caminho, a busca aproximada (`|`) por pedaços do nome de uma pasta ou
  arquivo, uma condição SQL (`W`) sobre a lista, por nome, data, tamanho ou
  conteúdo, bookmarks, e Ctrl+S para buscar texto nos arquivos marcados.
- **Visões da lista (Alt+V):** a Varredura aponta o que numa pasta merece um
  olhar (caracteres escondidos nos nomes, arquivos setuid, executáveis sem
  proteções, documentos do Office com macros), a Linha do tempo mostra os
  arquivos por data, e a Tabela abre a lista no visualizador de tabelas.
- **Olhar dentro:** um visualizador de texto e hex com busca, filtro de linhas
  com expressões regulares e cores de sintaxe, Markdown renderizado, ebooks EPUB e
  FB2 para ler como livro, Norton Guides, um modo tabela com filtro SQL
  para planilhas, CSV, JSON Lines, DBF, SQLite e Parquet, Ctrl+B para transformar
  esses registros ou linhas em um script, JSON, SQL ou o que for, Autoview ao lado da lista,
  arquivos ZIP, 7z e TAR abertos como diretórios, e modos de exibição que
  mostram o dono ou o que cada arquivo é.
- **Duplicados:** Alt+D acha arquivos e pastas inteiras com o mesmo conteúdo,
  sem baixar nada de uma pasta montada pelo rclone, e sempre deixa uma cópia.
- **Para onde vai o espaço:** a visão Usage (Alt+U) lista as pastas e os
  arquivos de um ramo pelo espaço que ocupam, do maior para o menor, até o
  que enche o disco; o Alt+K Percent mostra a parte de cada pasta na própria
  árvore.
- **E mais:** um terminal embutido cuja árvore acompanha o shell,
  máquinas remotas via SSH e containers via `docker exec` (copiar, mover,
  renomear, apagar, permissões e datas lá, com o terminal de lá), Google Drive,
  S3 e outros armazenamentos na nuvem via rclone, compartilhamentos de rede lidos
  em segundo plano, favoritos com nome, um menu Git
  opcional, a área de transferência, uma tela de configuração e temas.

## Telas


A árvore de diretórios, a janela de arquivos e as estatísticas:

![Xylem: árvore de diretórios, janela de arquivos e estatísticas](docs/screenshots/tree.png)

**Branch** (`B`) lista os arquivos de um ramo inteiro; os arquivos marcados
(`T`) ficam destacados e são contados à direita:

![Branch com três arquivos marcados](docs/screenshots/branch-tagged.png)

**Janela dividida** (F8), cada lado com seu próprio diretório:

![Janela dividida: fotos à esquerda, código-fonte à direita](docs/screenshots/split.png)

**Menus no Ctrl e no Alt**: segurar Ctrl ou Alt mostra o que cada letra faz
com ela (em terminais com o protocolo de teclado do kitty; nos outros, F9 troca
os menus):

![Segurando Ctrl e depois Alt, o menu de baixo muda; depois Ctrl+G mostra o menu do Git](docs/screenshots/demo-menus.gif)

**Um banco SQLite** no visualizador: a lista de tabelas, um filtro, uma query com
`JOIN` e um `DELETE` recusado, porque o Xylem só lê:

![Um banco SQLite: a lista de tabelas, um filtro em pedidos, uma query juntando duas tabelas e um DELETE recusado](docs/screenshots/demo-sqlite.gif)

**Batch** (Ctrl+B): uma tabela vira página, script SQL e banco de dados, cada um com
prévia, e cada valor escapado para aquele tipo de arquivo:

![Uma tabela de clientes gravada pelo Batch como página HTML, script SQL e banco SQLite](docs/screenshots/demo-batch.gif)

**Visualizador** (`V`), com cores de sintaxe:

![Visualizador mostrando código Rust com cores de sintaxe](docs/screenshots/viewer.png)

**Terminal embutido** (Alt+X): um `cd` no shell leva a árvore junto, e o
Alt+O digita os arquivos marcados no prompt:

![Três fotos marcadas digitadas num wc -c com Alt+O](docs/screenshots/terminal-tagged.png)

**Git** (opcional): as letras de status na lista de arquivos e o menu Git
(Ctrl+G):

![Lista de arquivos com as letras de status do Git e o menu GIT no rodapé](docs/screenshots/git-menu.png)

**Usage** (Alt+U): para onde vai o espaço, pastas e arquivos juntos pelo
espaço que ocupam:

![A visão Usage de uma pasta pessoal: um arquivo de backup antigo no topo, depois as pastas de projetos, vídeos e downloads](docs/screenshots/usage.png)

**Temas**: além do azul clássico, `dark`, `light`, `mono`, `norton`,
`solarized`, `gruvbox` (abaixo), `nord` e três que combinam com os desktops de
hoje: `slate` (no estilo do Windows Terminal), `graphite` e `daylight` (macOS
nos modos escuro e claro), além do `vivid`, com cores fortes para
apresentações:

![O tema gruvbox](docs/screenshots/theme-gruvbox.png)

## Uso

```bash
xy [diretório]
```

Sem argumento, abre o diretório atual. `xy --help` mostra as opções. Para o
shell ficar no diretório em que você estava ao sair com Alt+Q, acrescente a
pequena função de shell de
[Sair e mudar o diretório da shell](docs/manual/pt/quit.md).

**Vindo da 0.7?** Algumas teclas Alt mudaram na 0.8 (Alt+U agora é o Uso, Alt+D os
Duplicados); [O que Ctrl e Alt querem dizer](docs/manual/pt/tagging.md#o-que-ctrl-e-alt-querem-dizer)
tem a tabela e as linhas do `config.toml` que trazem as antigas de volta.

## Instalação

**Binário pronto:** baixe o arquivo do seu sistema na página de
[Releases](https://github.com/andrelnfernandes/xylem/releases): Linux x86_64 ou
ARM64 (estático, roda em qualquer distribuição) ou Windows x86_64. Extraia e
coloque o `xy` (`xy.exe`) numa pasta do `PATH`, por exemplo:

```bash
tar xzf xy-v0.9.0-x86_64-unknown-linux-musl.tar.gz
install -m 755 xy-v0.9.0-x86_64-unknown-linux-musl/xy ~/.local/bin/
```

Cada arquivo tem um `.sha256` ao lado, para conferir com `sha256sum -c`.

**Pacote (.deb ou .rpm):** as versões a partir da 0.5.0 também têm pacotes
para Debian, Ubuntu e derivados (`.deb`) e para Fedora, RHEL, openSUSE e
derivados (`.rpm`), para x86_64 e ARM64 (`arm64`/`aarch64` no nome). Além
do `xy`, eles instalam a entrada no menu, os ícones e a página de manual:

```bash
sudo apt install ./xylem-fm_*_amd64.deb     # Debian, Ubuntu
sudo dnf install ./xylem-fm-*.x86_64.rpm    # Fedora, RHEL (zypper no openSUSE)
```

O pacote se chama `xylem-fm`, como no crates.io. Ele vem da página de
Releases, não de um repositório, então não se atualiza sozinho: instale o
próximo do mesmo jeito. Para remover, `sudo apt remove xylem-fm` ou
`sudo dnf remove xylem-fm`.

**Com o Cargo** (requer Rust, via [rustup](https://rustup.rs), e um compilador
C, para o SQLite e o zstd), a partir do
[crates.io](https://crates.io/crates/xylem-fm), onde o Xylem se chama
`xylem-fm` (o nome `xylem` já era de outro projeto); o comando é `xy`:

```bash
cargo install xylem-fm
```

Ou o código mais recente, direto do GitHub, ou a partir de um clone do
repositório (na pasta do projeto):

```bash
cargo install --git https://github.com/andrelnfernandes/xylem
cargo install --path .
```

O binário vai para `~/.cargo/bin/xy` (que o rustup já coloca no `PATH`).
Para atualizar, rode o mesmo comando de novo; para remover, `cargo uninstall xylem-fm`.

**Atalho no menu (Linux, opcional; os pacotes já o trazem):** para ter o
Xylem no menu de aplicativos do seu desktop, abrindo num terminal, rode isto
num clone do repositório. O `xy` precisa estar no `PATH` que o desktop
enxerga (`~/.local/bin` costuma estar; `~/.cargo/bin` nem sempre).

```bash
for s in 16 32 48 64 128 256 512; do
    install -Dm644 docs/icons/xylem-$s.png ~/.local/share/icons/hicolor/${s}x$s/apps/xylem.png
done
install -Dm644 docs/icons/xylem.svg ~/.local/share/icons/hicolor/scalable/apps/xylem.svg
install -Dm644 packaging/linux/xylem.desktop ~/.local/share/applications/xylem.desktop
```

**Página de manual (opcional; os pacotes já a trazem):** num terminal,
`xy --man` a mostra; redirecionado, imprime o fonte para instalar (os
arquivos das releases para Linux também trazem o `xy.1`):

```bash
mkdir -p ~/.local/share/man/man1 && xy --man > ~/.local/share/man/man1/xy.1
```

## Documentação

**F1** (ou **Alt+H**, nos terminais que guardam o F1 para si, como o GNOME
Terminal) abre a ajuda sobre o que você está fazendo: dentro de um prompt, a
ajuda daquele prompt; numa janela, a ajuda daquela janela, com links para o
resto.

A mesma ajuda é [o manual](docs/manual/pt/index.md), que dá para ler aqui no
GitHub, e a página de manual (`xy --man`). Por onde começar:

- [o que é o Xylem](docs/manual/pt/overview.md) e
  [os primeiros cinco minutos](docs/manual/pt/start.md);
- [a tela](docs/manual/pt/screen.md), [os menus](docs/manual/pt/menus.md) e o
  [vocabulário](docs/manual/pt/glossary.md);
- os comandos de cada menu: [DIR e FILE](docs/manual/pt/commands.md),
  [Ctrl](docs/manual/pt/commands-ctrl.md) e [Alt](docs/manual/pt/commands-alt.md);
- receitas com uma animação de cada passo, como
  [copiar arquivos com os caminhos](docs/manual/pt/howto-copy-with-paths.md) e
  [achar o que enche o disco](docs/manual/pt/howto-disk-space.md);
- [notas de versão](docs/manual/pt/releases.md) e as
  [comparações com outros gerenciadores](docs/manual/pt/comparison.md).

A ajuda também existe em português, espanhol, francês, alemão e chinês. As
traduções em espanhol, francês, alemão e chinês não foram revisadas por falantes
nativos: se você lê um desses idiomas, [sugestões são bem-vindas](docs/manual/TRANSLATING.md#reviews-wanted).

## Segurança: arquivos hostis

O Xylem nunca deve travar, congelar, executar código, escrever fora do destino
ou esgotar a memória devido ao conteúdo ou ao nome de um arquivo, mesmo que ele
tenha sido criado propositalmente para isso. Arquivos baixados, pacotes abertos,
nomes enviados por um servidor remoto e repositórios clonados são todos tratados
como hostis. Qualquer falha desse tipo é um erro grave e será corrigida com
rapidez assim que relatada.

Ainda assim, erros acontecem. Veja o que o Xylem faz para manter o risco baixo:

- Tudo o que o Xylem lê por conta própria (texto, hex, codificações, arquivos
  comprimidos, pacotes, Markdown, planilhas, CSV, JSON Lines, DBF) passa por analisadores em Rust com
  segurança de memória. A exceção são bancos SQLite e arquivos Parquet, lidos
  por bibliotecas em C: elas só rodam quando você abre esse arquivo com V, nunca
  no Autoview. Nenhum programa externo é executado em um arquivo a menos que
  você solicite com E, O ou X.
- Tudo tem limite: a quantidade lida, a quantidade descomprimida, o tempo gasto
  e o volume de memória que um descompressor pode usar. Apenas um nível de
  compressão é desfeito, e nada é gravado em disco para exibir um arquivo.
- Caracteres de controle e controles bidirecionais (Trojan Source) nunca chegam
  ao terminal. Nomes com caracteres bidi ocultos são sinalizados, e o O pede
  confirmação antes de abri-los.
- Ao extrair pacotes ou copiar de uma máquina remota, caminhos com `..` ou
  caminhos absolutos são recusados, links simbólicos dentro de pacotes não são
  extraídos, bits setuid são descartados e nomes enviados por servidores SFTP são
  verificados.
- Olhar um repositório Git não executa nada que ele nomeie: nem ganchos,
  `core.fsmonitor`, diffs externos ou conversões de texto, a não ser que a pasta
  dele esteja em `git_trusted`. Um repositório com filtros próprios fica sem
  marcadores, Status, Diff e Blame.
- As dependências são verificadas contra o banco de dados do RustSec no CI.

Você pode reduzir ainda mais a superfície de ataque:

- Desative a descompressão no visualizador (`viewer_decompress = false`, F12)
  ao navegar em pastas não confiáveis com o Autoview, ou mantenha o Autoview (F7)
  fechado nelas.
- Mantenha a integração com o Git desativada (`git = false`, o padrão) para
  repositórios que você não criou.
- Não execute o Xylem como root.
- Só abra arquivos não confiáveis com O se você confiar no programa que irá
  abri-los: o Xylem apenas repassa o arquivo para ele.

Encontrou um problema? Relate-o de forma privada [pelos avisos de segurança do GitHub](https://github.com/andrelnfernandes/xylem/security/advisories/new), não em uma issue pública.

## Roteiro

### Feito

- O núcleo (fases 0 a 4): a tela clássica, a árvore e a janela de
  arquivos, Filespec, Showall e Branch, marcações e operações em grupo, a
  janela dividida, Compare, arquivos compactados, Edit e eXecute,
  configuração, temas, teclas configuráveis, a ajuda e o histórico
  persistente.
- Além dele: o terminal integrado, máquinas remotas via SSH e containers
  (navegar, escrever lá, o terminal deles), armazenamento em nuvem via rclone,
  com um Drive grande abrindo na hora, Git, a visão Usage e suas ações,
  favoritos com nome, Oops, arquivos binários num editor hexadecimal à sua
  escolha, ícones Nerd Fonts, a ajuda em seis línguas e uma versão
  experimental para Windows.
- O visualizador: codificações detectadas sozinhas, arquivos comprimidos,
  limite de tamanho configurável, quebra de linha, números de linha, ir para
  a linha, filtro de linhas, proteção contra caracteres ocultos de direção do
  texto e um [modo tabela](docs/manual/pt/viewer.md) com filtro SQL, e queries
  inteiras no SQLite, para planilhas (XLSX, ODS), CSV, JSON Lines, DBF com seus
  arquivos memo, SQLite e Parquet, e Markdown renderizado.
- Batch (Ctrl+B) sobre arquivos marcados, os registros de uma tabela ou as linhas
  de um texto, com templates, condições, totais e saída em SQLite.
- Duplicados (Alt+D): arquivos e pastas inteiras, em discos locais e pelos hashes
  que o rclone fornece.
- Busca: a condição SQL (`W`) com um índice do conteúdo, a caixa de busca (P) e
  a busca aproximada (`|`); as visões da lista (Alt+V).

### Próximos passos

- **Máquinas remotas (SSH):** Edit e Abrir (por uma cópia temporária que volta
  ao salvar), eXecute e Git via ssh, e atualização periódica.
- **Arquivos:** comparar o conteúdo de dois arquivos (JFC), e salvar o log para
  abrir a árvore de um disco na hora da próxima vez.
- **Endurecimento:** as sobras da revisão de segurança, listadas no
  [BACKLOG.md](BACKLOG.md) (em inglês).

### Para depois

- Windows: compila e passa nos testes (veja
  [Windows](docs/manual/pt/windows.md)), mas não é prioridade. macOS: compila, e
  o CI confere; um usuário o rodou no macOS 15 (Intel) e no 27 (Apple silicon)
  e conta que funciona. Ainda não há pacote para macOS.
- Adiados por opção: macros, chown e Graft.

A lista completa do que falta, com as notas de desenvolvimento, está em
[BACKLOG.md](BACKLOG.md) (em inglês), e a ordem do trabalho está
[no roteiro](docs/plans/roadmap.md).

## Estrutura

- `src/model/` — árvore de diretórios em memória (arena), sem dependência da UI
- `src/app/` — estado global (prompts, operações) e execução de comandos
- `src/pane.rs` — uma janela: árvore, lista de arquivos, cursores e filespec
- `src/input.rs` — comandos e o campo de texto dos prompts
- `src/keymap.rs` — teclas padrão e trocas da configuração
- `src/help.rs` — a ajuda (F1): tópicos, links e a página de manual
- `docs/manual/` — o manual: os tópicos da ajuda, em Markdown (em inglês; as traduções ficam em subpastas)
- [`demo/`](demo/README.pt-BR.md) — arquivos em todos os formatos e templates de Batch para testar
- `src/filetype.rs` — o que um arquivo é, para o modo de exibição Tipo
- `src/ui.rs` — desenho da tela (ratatui)
- `src/viewer.rs` — visualizador texto/hex
- `src/viewer/` — modo tabela, Markdown renderizado, filtro de linhas, filtro SQL e Batch sobre registros
- `src/usage.rs` — a visão Usage
- `src/duplicates.rs` — a busca de duplicados
- `src/search.rs` — busca de texto, nos arquivos marcados e no visualizador
- `src/decompress.rs` — arquivos comprimidos, com limites
- `src/charset.rs` — codificações de texto
- `src/bidi.rs` — a proteção contra caracteres ocultos de direção do texto
- `src/archive.rs` — leitura de ZIP, 7z e TAR; criação de ZIP e TAR
- `src/watch.rs` — avisos de mudanças no disco (inotify)
- `src/shell.rs` — editor e comandos do shell (com aspas seguras)
- `src/editors.rs` — qual editor o E usa para cada arquivo (`[editors]`)
- `src/term.rs` — terminal embutido (pseudoterminal, códigos de teclas, cores)
- `src/git.rs` — menu Git (Ctrl+G) e marcadores de estado, pelo programa `git`
- `src/rclone.rs` — pastas montadas pelo rclone: qual serviço, listagens e hashes
- `src/remote/` — máquinas remotas: SFTP pelo `ssh` do sistema
- `src/config.rs` — arquivo de configuração, temas e formatos
- `src/settings.rs` — tela de configuração (F12)
- `src/state.rs` — histórico dos prompts entre execuções
- `src/model/compare.rs` — regras do Compare
- `src/model/filespec.rs` — filtro de arquivos
- `src/ops/` — copiar, mover e apagar, em passos (progresso, conflitos, cancelamento)
- `docs/screenshots/tools/` — refaz as capturas de tela do README (`run.sh`, Linux)
- `tests/e2e.rs` — testes de ponta a ponta: o `xy` de verdade num pseudo-terminal, dirigido por teclas (Linux); o `cargo test` também roda testes de propriedades (proptest) do Filespec e das máscaras de renomeação
- `docs/icons/` — o ícone, em SVG, PNG (16 a 512) e ICO
- `build.rs` — embute o ícone no `xy.exe` (Windows)
- `packaging/linux/` — atalho para o menu (`xylem.desktop`) e os pacotes `.deb` e `.rpm` (`packages.sh`)
- `site/` — o site

## Sobre

Ele se chamava **xt** até a versão 0.1; na primeira execução, move a configuração, o
histórico e os bookmarks do xt para as pastas novas. O nome: o xilema (*xylem*, em
inglês) é o tecido que leva a água pela árvore.

## Aviso

O Xylem é um projeto independente, escrito do zero, sem código do XTree, do
ZTreeWin ou dos outros programas com que é comparado, e sem ligação com eles. Os
nomes pertencem aos seus donos; veja [Comparisons](docs/manual/comparison.md#names).

## Licença

O Xylem é distribuído sob a [licença MIT](LICENSE).
