# Instalar e os primeiros cinco minutos

## Instalação

O Xylem roda em Linux (x86_64 e ARM64). A compilação para Windows passa nos
testes mas não foi usada no dia a dia; veja [Windows](windows.md).

**Binário.** Baixe o pacote para o seu sistema em
`https://github.com/andrelnfernandes/xylem/releases`, extraia-o e coloque o `xy`
em um diretório do seu `PATH`:

```bash
tar xzf xy-v0.7.0-x86_64-unknown-linux-musl.tar.gz
install -m 755 xy-v0.7.0-x86_64-unknown-linux-musl/xy ~/.local/bin/
```

Os binários para Linux são estáticos e rodam em qualquer distribuição. Cada
arquivo possui um arquivo `.sha256` para conferência com `sha256sum -c`.

**Pacote.** A mesma página traz pacotes `.deb` e `.rpm`, que também instalam
a entrada no menu de aplicativos, os ícones e a página de manual (man):

```bash
sudo apt install ./xylem-fm_*_amd64.deb
sudo dnf install ./xylem-fm-*.x86_64.rpm
```

O pacote não vem de um repositório, portanto não é atualizado automaticamente.
Instale a versão seguinte da mesma forma.

**Cargo.** Requer Rust e um compilador C (para SQLite e zstd). O crate é
`xylem-fm`; o comando instalado é `xy`:

```bash
cargo install xylem-fm
```

## Início

```bash
xy
```

abre o diretório atual. `xy ~/projetos` abre outro diretório, e
`xy user@host:` abre uma [máquina remota](remote.md). [A linha de
comando](cli.md) traz as demais formas.

Na primeira vez, o Xylem pergunta se pode procurar uma versão nova uma vez
por dia: **Y** ou **N** (veja [Versões novas](configuration.md#new-versions)).

## Como mover na árvore

A árvore mostra o diretório inicial e seus subdiretórios. Apenas o diretório
inicial foi lido.

| Tecla | O que faz |
|---|---|
| ↑ ↓ | move o cursor |
| → | lê o diretório sob o cursor e mostra seu conteúdo |
| ← | fecha o diretório |
| `*` | lê o diretório e tudo o que está abaixo dele |

A janela inferior lista os arquivos do diretório sob o cursor. Em um
diretório que não foi lido, ela exibe `Not logged`.

## Indo para os arquivos

**Enter** vai para a janela de arquivos, que então ocupa a altura da tela.
Enter ou Esc volta para a árvore.

| Tecla | O que faz |
|---|---|
| `V` | visualiza o arquivo |
| `E` | abre o arquivo no seu editor |
| `C`, `M`, `D`, `R` | copia, move, exclui, renomeia |

## Marcando arquivos e copiando

1. Na janela de arquivos, pressione `T` em cada arquivo desejado. Um `■` surge
   antes do nome e o cursor desce uma linha.
2. Pressione Ctrl+C. O prompt pergunta onde copiar.
3. Digite um diretório e pressione Enter.

Os arquivos marcados são copiados. `U` desmarca um arquivo, e Ctrl+U desmarca
todos eles.

## Sair

`Q` sai, após confirmação. Alt+Q sai e deixa o shell no diretório em que você
estava, se a função do shell de [Sair](quit.md) estiver instalada.

## A seguir

- [A tela](screen.md) dá o nome de cada parte do que você vê.
- [Os menus](menus.md) explica o que uma letra faz sozinha, com Ctrl e com Alt.
- [Vocabulário](glossary.md) define log, branch, tag e os demais termos.
- **F1** abre a ajuda sobre o que está na tela.
