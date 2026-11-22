# Segurança

O Xylem trata como hostil todo arquivo que não criou: arquivos baixados,
arquivos compactados abertos, nomes enviados por um servidor remoto,
repositórios clonados. O conteúdo ou o nome de um arquivo nunca deve fazer o
Xylem quebrar, travar, executar código, gravar fora do destino ou esgotar a
memória. Se isso ocorrer, trata-se de uma falha a ser relatada.

## O que o Xylem faz

| Risco | Medida |
|---|---|
| Arquivo adulterado explora o leitor | Texto, hexadecimal, codificações, arquivos comprimidos, arquivos compactados, Markdown, planilhas, CSV, JSON Lines e DBF são lidos por analisadores escritos em Rust. SQLite e Parquet são lidos por bibliotecas escritas em C apenas quando o arquivo é aberto com `V`, nunca no Autoview. |
| Arquivo executa um programa | Nenhum programa externo é executado sobre um arquivo a menos que solicitado com `E`, `O` ou `X`. |
| Arquivo pequeno expande para esgotar memória ou disco | Cada leitor possui limites para o volume lido, tamanho descomprimido, tempo de execução e memória utilizada pelo descompressor. Apenas um nível de compressão é revertido. A visualização de um arquivo nada grava em disco. |
| Nome oculta sua real identidade | Caracteres de controle e caracteres de direção bidirecional (Trojan Source) não são enviados ao terminal. Nomes com caracteres de direção ocultos são exibidos em vermelho, e `O` solicita confirmação antes de abri-los. |
| Arquivo compactado grava fora do destino | Caminhos contendo `..` e caminhos absolutos são recusados, links simbólicos em arquivos compactados não são extraídos e bits setuid são removidos. As mesmas validações aplicam-se aos nomes enviados por um servidor SFTP. |
| Repositório Git executa comandos próprios | Navegar por um repositório não executa hooks, `core.fsmonitor`, diff externo nem conversão de texto, a menos que seu diretório conste em `git_trusted`. |
| Dependência com vulnerabilidade conhecida | As dependências são verificadas contra o banco de dados do RustSec na integração contínua (CI). |

## A rede

O Xylem se conecta à rede por conta própria para uma coisa só: se você disse
sim na primeira vez, ele pergunta ao site, uma vez por dia, o número da versão
mais recente ([Versões novas](configuration.md#new-versions)). Todo o resto
que usa a rede é algo que você pediu: uma máquina remota (`ssh`), a nuvem
(`rclone`), um container (`docker`), um compartilhamento de rede e os
programas que você mesmo executa (`X`, o terminal integrado).

## O que você pode fazer

- Desative a descompressão no visualizador (`viewer_decompress = false`,
  F12 página 2) ao navegar por pastas não confiáveis com o Autoview ativo, ou
  mantenha o Autoview fechado nessas pastas.
- Mantenha o suporte a Git desativado (`git = false`, o padrão) em repositórios
  que você não criou.
- Responda não à busca de versões novas, ou desligue-a na página 3 do F12, se
  o Xylem nunca deve se conectar sozinho.
- Não execute o Xylem como root.
- O comando `O` repassa o arquivo para outro programa. Os limites do Xylem não
  se aplicam a esse outro programa.

## O que não está coberto

- Os programas iniciados por `E`, `O` e `X`.
- `rclone`, `ssh`, `docker` e `git`, executados pelo Xylem conforme instalados
  no sistema operacional.
- O terminal em si tratar de forma inadequada o texto que recebe.

## Como relatar um problema

Relate em privado, por meio do GitHub Security Advisories:
`https://github.com/andrelnfernandes/xylem/security/advisories/new`. Não abra
uma issue pública para isso.

Veja também: [o visualizador](viewer.md), [arquivos compactados](archives.md), [Git](git.md).
