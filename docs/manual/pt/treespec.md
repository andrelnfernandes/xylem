# Treespec e Log

## Treespec (`\` ou `/`)

Abre um prompt com o caminho atual. Conforme você digita, o cursor da árvore
vai para o primeiro diretório que começa com o que foi digitado (sem
diferenciar maiúsculas e minúsculas):

- `/` ou `\` completam o nome e descem um nível;
- Tab e Shift+Tab completam e movem para o próximo diretório ou para o anterior
  no mesmo nível;
- Backspace sobe níveis;
- Enter confirma e Esc retorna para onde você estava;
- ↑ traz caminhos anteriores (salvos entre sessões).

Um caminho fora da árvore se torna um Log ao ser confirmado. Em Branch e
Showall, `\` vai direto para o diretório do arquivo selecionado, com ele
destacado.

## Log (L)

Abre outro diretório na janela, substituindo a árvore (um diretório dentro da
árvore atual apenas move o cursor até lá). Um endereço como `user@host:` abre
uma [máquina remota](remote.md). O caminho pode ser absoluto, relativo ao
diretório atual ou começar com `~`; **F2** [escolhe-o em uma árvore](picker.md)
e ↑ traz os anteriores. Novas janelas mantêm a [ordem de classificação](sort.md)
e o modo de exibição atuais.

**Alt+L** faz do diretório sob o cursor a raiz da árvore, na hora: o `L` para um
diretório de dentro da árvore só move o cursor, e o Alt+L então reduz a árvore a
ele. Na janela de arquivos, vale o diretório mostrado (em Branch e Showall, o do
arquivo selecionado). A janela é aberta de novo ali, então o que foi lido e as
marcas se perdem; `L ..` volta a abrir o diretório de cima. Não vale dentro de
arquivos compactados nem em máquinas remotas.

O prompt abre vazio. ↑ percorre os lugares usados antes, do último para o mais
antigo: os que foram logados e as pastas para onde arquivos foram copiados ou
movidos.

**F11** traz um [favorito](bookmarks.md) para o prompt, pelo nome, em vez de
digitar o endereço de uma máquina remota ou container.

Veja também: [a janela da árvore](tree.md), [favoritos](bookmarks.md).
