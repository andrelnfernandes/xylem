# Favoritos

Os favoritos (*bookmarks*) levam você de volta aos diretórios que usa com
frequência, com uma tecla para cada um (`0`–`9`, `a`–`z`). Eles são salvos
entre sessões, junto com o histórico de prompts.

- **Ctrl+F11** (ou Ctrl+B na árvore) salva o diretório atual: pressione a tecla
  que deseja atribuir a ele.
- **`'`** (aspas simples) salta para um favorito: pressione a tecla dele. No
  mesmo prompt, `'` ou Enter abre a lista e `+` salva o diretório atual.
- **F11** (ou Alt+B) abre o gerenciador de favoritos: ↑ ↓ escolhem, Enter (ou a
  tecla do favorito) salta para ele, **F2** dá um nome a ele, Del remove,
  Insert ou `+` salva o diretório atual e Esc fecha.
- **F11 nos prompts do [Log](treespec.md), do [Copy e do Move](copy-move.md)**
  abre a mesma lista para escolher um lugar: Enter (ou a tecla do favorito) o
  põe no prompt, onde você ainda confirma; Esc volta ao que você digitou.

Um diretório fora da árvore abre como uma nova árvore, como em
[Log](treespec.md). Na janela de arquivos, Ctrl+B é [Batch](execute.md); ali,
use Ctrl+F11.

## Nomes

Um nome diz para que serve o favorito: "Servidor do Oracle", "Docker do
Postgres Amb 1". No gerenciador, **F2** sobre um favorito pede o nome, com o
atual para editar; um nome vazio o remove. O nome aparece antes do lugar, e o
lugar aparece sempre ao lado, para que um nome não esconda aonde o favorito
leva. Um nome tem no máximo 60 caracteres, sem caracteres de controle nem de
direção do texto.

## Máquinas remotas e containers

Numa janela de uma [máquina remota](remote.md) ou de um container, Ctrl+F11
salva a pasta pelo endereço que a abre: `ssh://oracle@db01/u01/app`,
`docker://postgres-amb1/var/lib/postgresql`. Ir até ele conecta como digitar o
endereço no Log, pedindo a senha se o ssh precisar; nenhuma senha é salva. Se a
janela já está nessa máquina e a pasta está na árvore dela, só o cursor se
move. Uma pasta cujo caminho tem espaço não pode ser salva assim, porque um
endereço não comporta espaços.

O gerenciador marca `[not found]` numa pasta local que não existe mais. Ele não
confere favoritos remotos, nem pastas numa montagem de nuvem ou num
compartilhamento de rede: isso esperaria a rede.
