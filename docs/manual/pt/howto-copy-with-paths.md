# Copiar arquivos de várias pastas mantendo os caminhos

Este procedimento copia todos os arquivos JPG abaixo de `~/projects` para
`/mnt/backup`, cada um no mesmo subdiretório de origem. Os mesmos passos
aplicam-se ao comando Move (Alt+M).

## Passo a passo

1. Na árvore, posicione o cursor sobre `projects`.
2. Pressione `B`. O Xylem lê o ramo e a janela de arquivos lista os arquivos de
   `projects` e de todas as pastas abaixo dele. Em ramos grandes, o rodapé exibe
   o progresso e Esc cancela a operação.
3. Pressione `F`, digite `*.jpg` e pressione Enter. A lista passa a exibir
   apenas os arquivos JPG.
4. Pressione Ctrl+T. Todos os arquivos da lista são marcados (`■`).
5. Pressione Alt+C. O prompt solicita o diretório de destino.
6. Digite `/mnt/backup`. Antes de pressionar Enter, observe o modo indicado no
   prompt e pressione **F7** até selecionar `Relative`.
7. Pressione Enter. Se `/mnt/backup` não existir, o Xylem solicita confirmação
   antes de criá-lo.

![Os sete passos: Branch em projects, filespec *.jpg, Ctrl+T, Alt+C, o destino no modo Relative e 21 arquivos copiados](../../screenshots/howto-copy-paths.gif)

A barra abaixo da tela exibe o passo em execução e suas teclas.

## Qual modo de caminho escolher

O passo 6 define onde cada caminho tem início no destino. Para o arquivo
`~/projects/photos/2025/trip/DSC_01.jpg`, com o Branch aberto em `projects`:

| Modo | O arquivo é gravado em |
|---|---|
| Full | `/mnt/backup/home/user/projects/photos/2025/trip/DSC_01.jpg` |
| Tree | `/mnt/backup/` e o caminho a partir da raiz da árvore do Xylem |
| Current | `/mnt/backup/projects/photos/2025/trip/DSC_01.jpg` |
| Relative | `/mnt/backup/photos/2025/trip/DSC_01.jpg` |

O prompt inicia no modo `Full`, a menos que configurado de outra forma por
`copy_paths` no arquivo de configuração.

## Variações

- Sem recriar os caminhos: Ctrl+C em vez de Alt+C grava todos os arquivos JPG
  diretamente em `/mnt/backup`. Dois arquivos com o mesmo nome colidem, e o
  Xylem pergunta o que fazer com cada um.
- Por data ou tamanho: no passo 3, `*.jpg,>TODAY-30` mantém apenas os JPGs
  modificados nos últimos 30 dias. [Filespec](filespec.md) descreve a sintaxe.
- De todo o disco: leia mais diretórios com `*` e pressione `S` (Showall) no
  passo 2.
- Seleção manual: pule o passo 4 e marque cada arquivo individualmente com `T`.

## Quando não se comporta como esperado

- **Alt+C inicia uma comparação (Compare).** O foco está na árvore. Alt+C copia
  recriando caminhos apenas na janela de arquivos; pressione Enter para ir a ela.
- **Arquivos esperados não aparecem na lista.** Um Filespec definido
  anteriormente continua ativo. Ele é indicado no título da janela de arquivos;
  pressione `F`, `*` e Enter para limpá-lo.
- **As pastas no destino começam em `/home`.** O modo selecionado foi `Full`.
  Veja o passo 6.

Ctrl+U desmarca toda a lista e `B` retorna à exibição apenas dos arquivos do
diretório sob o cursor.

Veja também: [Copiar e Mover](copy-move.md), [Marcações](tagging.md),
[A janela da árvore](tree.md).
