# Treespec e Log

## Treespec (`\` ou `/`)

Abre um prompt com o caminho atual. Conforme você digita, o cursor da árvore
vai para o primeiro diretório que começa com o que foi digitado (sem
diferenciar maiúsculas e minúsculas):

- `/` ou `\` completam o nome e descem um nível;
- Tab e Shift+Tab completam e movem para o próximo diretório ou para o anterior
  no mesmo nível;
- Backspace sobe níveis;
- Ctrl+← tira a última pasta do caminho (`~/projects/xylem/docs/` vira
  `~/projects/xylem/`, com o cursor em `xylem`), e Ctrl+→ traz de volta, uma por
  vez, as pastas tiradas; digitar outra coisa começa um caminho novo;
- Enter confirma e Esc retorna para onde você estava;
- ↑ abre os caminhos anteriores em uma [lista](execute.md#histórico) (salvos entre sessões).

Um caminho fora da árvore se torna um Log ao ser confirmado. Em Branch e
Showall, `\` vai direto para o diretório do arquivo selecionado, com ele
destacado.

Para encontrar uma pasta por pedaços do nome, em qualquer nível, sem digitar
o caminho: [Busca aproximada](fuzzy.md) (`|`).

## Log (L)

Abre outro diretório na janela, substituindo a árvore (um diretório dentro da
árvore atual apenas move o cursor até lá). Um endereço como `user@host:` abre
uma [máquina remota](remote.md). O caminho pode ser absoluto, relativo ao
diretório atual ou começar com `~`; **F2** [escolhe-o em uma árvore](picker.md)
e ↑ lista os anteriores. Novas janelas mantêm a [ordem de classificação](sort.md)
e o modo de exibição atuais.

**Alt+L** faz do diretório sob o cursor a raiz da árvore, na hora: o `L` para um
diretório de dentro da árvore só move o cursor, e o Alt+L então reduz a árvore a
ele. Na janela de arquivos, vale o diretório mostrado (em Branch e Showall, o do
arquivo selecionado). A janela é aberta de novo ali, então o que foi lido e as
marcas se perdem; `L ..` volta a abrir o diretório de cima. Não vale dentro de
arquivos compactados nem em máquinas remotas.

O prompt abre vazio. ↑ abre uma [lista](execute.md#histórico) dos lugares
usados antes, com o último embaixo: os que foram logados e as pastas para onde
arquivos foram copiados ou movidos. Digitar filtra a lista, Enter põe um lugar
na linha e Del o tira da lista.

**F11** traz um [favorito](bookmarks.md) para o prompt, pelo nome, em vez de
digitar o endereço de uma máquina remota ou container.

Veja também: [a janela da árvore](tree.md), [favoritos](bookmarks.md).
