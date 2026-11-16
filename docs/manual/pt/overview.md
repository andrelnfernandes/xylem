# O que é o Xylem

O Xylem é um gerenciador de arquivos que roda no terminal. Sua tela e teclas
seguem o XTreeGold e o ZTreeWin: uma árvore de diretórios no topo, os arquivos
do diretório sob o cursor abaixo dela, estatísticas à direita e um menu de
comandos de uma letra no rodapé. O comando é `xy`.

## Leitura de diretórios (Log)

O Xylem lê um diretório para a memória quando você solicita, e o mantém lá.
Isso se chama leitura (*logging*). `→` lê um diretório, `*` lê todo o ramo
sob o cursor e `-` libera o ramo da memória.

Dois comandos listam arquivos lidos de mais de um diretório de uma só vez: `B`
(Branch) lista os arquivos do diretório e de tudo abaixo dele, e `S`
(Showall) lista todos os arquivos lidos. Nessas listas você marca arquivos com
`T`, e os comandos com Ctrl atuam sobre os arquivos marcados: Ctrl+C copia,
Ctrl+D exclui, Ctrl+R renomeia com uma máscara.

Arquivos em um diretório que não foi lido não aparecem em nenhuma lista e não
são afetados por nenhum comando.

## O que os comandos cobrem

| Para fazer isto | Use | Tópico |
|---|---|---|
| Copiar ou mover, com ou sem os caminhos de origem | `C`, `M`, Ctrl+C, Alt+C | [Copiar e Mover](copy-move.md) |
| Excluir, e restaurar o que foi para a lixeira | `D`, Ctrl+Z | [Excluir](delete.md) |
| Renomear um arquivo, ou vários com uma máscara | `R`, Ctrl+R | [Renomear](rename.md) |
| Exibir apenas alguns arquivos, por nome, data ou tamanho | `F` | [Filespec](filespec.md) |
| Comparar dois diretórios | `C` na árvore | [Comparar](compare.md) |
| Ver um arquivo como texto, hexadecimal ou tabela | `V` | [O visualizador](viewer.md) |
| Abrir ZIP, 7z e TAR como diretórios | Alt+F5 | [Arquivos compactados](archives.md) |

[Os menus](menus.md) explica como as teclas estão organizadas, e
[os comandos](commands.md) lista todos eles.

## O que o XTreeGold e o ZTreeWin não possuem

- Um terminal dentro da tela (Alt+X). A árvore acompanha o `cd` do shell,
  e Alt+O digita os arquivos marcados na linha de comando.
  Veja [o terminal integrado](terminal.md).
- Outras máquinas via SSH, e containers via `docker exec`, exibidos como uma
  árvore. Copiar, Mover, Renomear, Excluir, Permissões e Data funcionam lá.
  Edit e Batch não. Veja [máquinas remotas](remote.md).
- Google Drive, S3 e outros armazenamentos suportados pelo rclone.
  Veja [armazenamento na nuvem](cloud.md).
- Uma condição SQL na lista de arquivos (`W`): `ext = 'jpg' AND size > 10000000`
  mantém os arquivos que a satisfazem. Veja [Filespec](filespec.md#uma-condicao-sql-w).
- SQL no conteúdo de um arquivo: o visualizador exibe planilhas, CSV, JSON
  Lines, DBF, SQLite e Parquet como tabelas e os filtra com uma condição
  `WHERE`. Veja [o visualizador](viewer.md).
- Duplicados (Alt+D): arquivos e pastas com o mesmo conteúdo em um ramo.
  Em uma pasta montada pelo rclone ele compara os hashes que o serviço já
  mantém e não baixa nada. Veja [duplicados](duplicates.md).
- Uso (Alt+U): as pastas e arquivos de um ramo ordenados pelo espaço que
  ocupam. Veja [uso do disco](usage.md).
- Um menu Git (Ctrl+G), desativado por padrão. Veja [Git](git.md).

[Comparação com o ZTreeWin](ztreewin.md) lista os comandos do ZTreeWin que o
Xylem não possui e as teclas que diferem.

## Arquivos em que você não confia

O Xylem lê cada arquivo como se tivesse sido preparado para atacá-lo. Ver um
arquivo não executa nada dele, extrair um arquivo compactado não grava fora do
destino e cada leitor para em um limite de tamanho.
[Segurança](security.md) lista os limites e o que não está coberto.
