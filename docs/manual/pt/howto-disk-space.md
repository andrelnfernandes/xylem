# Achar o que enche o disco

Este procedimento parte do diretório pessoal até localizar os maiores arquivos
nele contidos e exclui um deles.

## Passo a passo

1. Na árvore, posicione o cursor sobre o diretório que deseja analisar e
   pressione Alt+U. O Xylem lê o ramo e lista suas pastas e arquivos pelo
   espaço ocupado em disco, do maior para o menor.
2. Navegue até uma pasta (cujo nome começa com `/`) e pressione Enter para
   listar seu conteúdo.
3. Pressione Enter novamente sobre a maior pasta exibida e repita o processo até
   que a lista apresente arquivos.
4. Navegue até o arquivo a excluir e pressione Del. O Xylem solicita confirmação
   antes da exclusão.
5. Pressione `T` para enviá-lo à lixeira e, em seguida, `Y`.

![Os cinco passos: Alt+U no diretório pessoal, Enter em projects, Enter em backup, Del no arquivo compactado antigo e T, Y enviando-o para a lixeira](../../screenshots/howto-disk-space.gif)

O rodapé exibe `Trashed 1 of 1 files` e os percentuais são recalculados sem o
arquivo removido.

## Como ler uma linha

```
     19.0 MiB ████████████████    51%   projects-2026-10.tar.gz
```

| Parte | Significado |
|---|---|
| `19.0 MiB` | o espaço ocupado pelo item no disco |
| a barra | proporção visual em relação ao maior item da lista |
| `51%` | fatia percentual do item no diretório listado |
| o nome | uma pasta quando começa por `/` |

## Teclas na lista

| Tecla | O que faz |
|---|---|
| Enter | entra na pasta; sobre um arquivo, abre o visualizador |
| Backspace ou ← | retorna à pasta superior |
| `T` | marca o arquivo, ou todos os arquivos da pasta |
| Ctrl+D | exclui os arquivos marcados |
| Alt+P | exclui a pasta com todo o seu conteúdo, de forma definitiva |
| `D` | procura duplicados na pasta listada |
| `\` | fecha a lista e posiciona o cursor da árvore na pasta listada |
| Esc | fecha a lista |

## Quando não se comporta como esperado

- **Uma linha começa com `>`.** Trata-se de outro sistema de arquivos montado ali.
  Ele não é somado no total, e pressionar Enter sobre ele abre uma lista própria
  para aquele disco.
- **Uma linha começa com `!` ou `.`.** A pasta, ou algo dentro dela, não pôde
  ser lida, geralmente por falta de permissão de acesso. O espaço real ocupado
  pode ser maior que o indicado.
- **Os tamanhos diferem dos exibidos na janela de arquivos.** Esta lista mostra o
  espaço ocupado em disco, em blocos inteiros. A tecla `A` alterna para o tamanho
  próprio dos arquivos.

## Armazenamento na nuvem

Os mesmos passos funcionam em pastas do Google Drive ou Amazon S3 montadas via
rclone. [Armazenamento na nuvem (rclone)](cloud.md) descreve a configuração e o
que muda nesses serviços.

Veja também: [Uso do disco](usage.md),
[achar duplicados](howto-duplicates.md).
