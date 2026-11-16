# Escolher arquivos com uma condição SQL

Este procedimento lista os arquivos JPG maiores que 1 MB localizados em
qualquer ponto abaixo de `~/projects` e os marca, prontos para Copiar, Mover ou
Excluir.

## Passo a passo

1. Na árvore, posicione o cursor sobre `projects` e pressione `B`. A janela de
   arquivos lista os arquivos de todo o ramo.
2. Pressione `W`. O prompt solicita a condição.
3. Digite `ext = 'jpg' AND size > 1000000` e pressione Enter. A lista mantém os
   arquivos que atendem à condição e o rodapé informa a quantidade restante.
4. Pressione Ctrl+T. Os arquivos exibidos na lista são marcados.

![Os quatro passos: Branch em projects, W, a condição digitada e os 12 arquivos resultantes marcados com Ctrl+T](../../screenshots/howto-sql-filter.gif)

O título da janela de arquivos exibe a condição: `Branch: * where ext =
'jpg' AND size > 1000000 (12)`. Pressionar `W` com o prompt vazio restaura a
lista completa; as marcas são preservadas.

## Como escrever a condição

A condição corresponde ao que se segue a `WHERE` em uma consulta SQL. Textos
ficam entre aspas simples e números são grafados por extenso.

| Para manter | Condição |
|---|---|
| arquivos modificados na última semana | `age_days < 7` |
| arquivos Markdown e de texto | `ext IN ('md', 'txt')` |
| nomes iniciados por `report` | `name LIKE 'report%'` |
| arquivos ainda não marcados | `NOT tagged` |
| arquivos que contenham um texto | `contains('invoice')` |

[Uma condição SQL](filespec.md#uma-condicao-sql-w) lista todas as colunas
disponíveis.

## Quando não se comporta como esperado

- **`No column "jpg"`.** O texto foi digitado sem aspas. `ext = jpg` compara
  duas colunas entre si; `ext = 'jpg'` compara o valor com a sequência de texto.
- **Arquivos esperados não aparecem.** O comando `W` consulta apenas arquivos
  lidos na memória. Abra o Branch antes ou leia outros diretórios com `*` e abra
  o Showall (`S`).
- **Nada muda na tela.** Nenhum arquivo atende à condição especificada e a lista
  permaneceu como estava.
- **`contains` solicita confirmação antes de executar.** O operador lê cada
  arquivo que precisa verificar; a pergunta informa quantos arquivos e bytes
  serão processados.

Veja também: [Filespec](filespec.md), [Marcações](tagging.md),
[copiar arquivos mantendo os caminhos](howto-copy-with-paths.md).
