# A linha de comando

```
xy [--cwd-file FILE] [ONDE]
```

## O que abrir

| `ONDE` | Abre |
|---|---|
| nada | o diretório atual |
| `DIRETÓRIO` | esse diretório, como a raiz da árvore |
| `user@host:` ou `user@host:CAMINHO` | uma máquina por SSH, no diretório pessoal ou em `CAMINHO` |
| `ssh://[user@]host[:porta][/CAMINHO]` | o mesmo, especificando a porta |
| `docker://[user@]container[/CAMINHO]` | um container em execução, via `docker exec` |

Os mesmos endereços funcionam no comando `L` (Log) dentro do Xylem. Veja
[máquinas remotas](remote.md).

## Opções

| Opção | O que faz |
|---|---|
| `--cwd-file FILE` | Ao pressionar Alt+Q, grava o diretório atual em `FILE`. Uma função do shell o lê e muda de diretório. [Sair e mudar o diretório do shell](quit.md) traz a função. |
| `--default-config` | Exibe um arquivo de configuração com todas as opções, seus valores padrão e comentários. |
| `--man` | Exibe a página de manual. Se a saída for redirecionada, grava seu código-fonte troff. |
| `-h`, `--help` | Exibe as opções da linha de comando e o caminho do arquivo de configuração. |
| `-V`, `--version` | Exibe a versão do programa. |

## Instalar a página de manual

Os pacotes `.deb` e `.rpm` a instalam automaticamente. A partir de um binário avulso ou do Cargo:

```bash
mkdir -p ~/.local/share/man/man1 && xy --man > ~/.local/share/man/man1/xy.1
```

## Começar com uma configuração comentada

```bash
mkdir -p ~/.config/xylem && xy --default-config > ~/.config/xylem/config.toml
```

Isto sobrescreve um arquivo `config.toml` existente.

## Variáveis de ambiente

| Variável | Efeito |
|---|---|
| `XDG_CONFIG_HOME` | A configuração é lida de `$XDG_CONFIG_HOME/xylem/` em vez de `~/.config/xylem/`. |
| `SHELL` | O shell do terminal integrado. |
| `VISUAL`, `EDITOR` | O editor para `E`, quando nenhum estiver definido na configuração. |
| `XY_DOCKER` | O programa usado para endereços `docker://`. `XY_DOCKER=podman` usa o Podman. |

Dentro do terminal integrado, o Xylem define `XY_TERMINAL=1` e, no bash, zsh
e fish, `XY_TAGGED` com os caminhos dos arquivos marcados. Veja
[o terminal integrado](terminal.md).

Veja também: [configuração](configuration.md).
