# Plano: um visual mais atraente, dentro do XTree

Origem: conversa de 2026-11-02. O autor pediu sugestões para deixar o Xylem mais
atraente sem sair da identidade do XTree, e aprovou implementar as sete numa branch
própria (`visual`): se não gostar, a branch é esquecida. **Os sete feitos na branch
`visual`, com testes; capturas regeneradas em `docs/screenshots/` (tree, tree-nord,
split-gruvbox, theme-vivid, branch-tagged-dark, remote-nord).** As funções de estilo
recebem o tema como parâmetro, porque o tema global é trocado em paralelo por outros
testes (o seletor do F12). Também no F12: **N** (cores por tipo) e **O** (bordas).

Nada muda o layout do XTree: são cores, intensidades e alguns caracteres de desenho
de caixa.

## Regras para todas

- Tudo passa pelo tema. O `mono` continua sem cor; os temas de 16 cores usam só as 16.
- Nada depende de Nerd Fonts nem de emoji.
- O que mostra texto de fora continua passando por `bidi::sanitize` no fim do desenho.

## Os sete itens

1. **Cor por tipo de arquivo.** Seis categorias pela extensão: compactados, imagens,
   áudio e vídeo, documentos, código, dados. Cada tema ganha uma cor para cada uma
   (`archive`, `image`, `media`, `document`, `code`, `data`, também em `[colors]`).
   Precedência: marcado, symlink, especial, executável, categoria, texto. Opção
   `type_colors` (padrão: ligada), para quem quer o XTree puro.
2. **Tamanho e data com ênfase.** Na coluna de detalhes: tamanho abaixo de 1 MiB no
   tom apagado de hoje, até 1 GiB no tom do texto, acima em negrito; data de menos de
   24 horas no tom do texto, o resto apagado. As permissões seguem apagadas.
3. **Caminho até o cursor na árvore.** Na janela ativa, as linhas guia da raiz até a
   pasta do cursor na cor da borda ativa: o conector de cada pasta do caminho inteiro,
   e só o traço vertical nas linhas entre uma pasta do caminho e a próxima.
4. **Barras no painel de estatísticas.** Abaixo de Available/Total, a ocupação do
   disco (verde até 75%, amarelo até 90%, vermelho acima, com as cores `bar_*`). Em
   Tagged, a parte marcada dos bytes que atendem ao Filespec, quando houver marcados.
5. **Cabeçalho.** `~` no lugar da pasta pessoal; o último nome do caminho em negrito e
   o resto apagado. Numa janela remota ou de container, um rótulo com a máquina em
   destaque (`deploy@web`, `docker:app`) antes do caminho remoto.
6. **Barra de rolagem.** Na borda direita da árvore e da lista, só quando o conteúdo
   não cabe, sem setas, por cima da borda.
7. **Bordas arredondadas.** `╭╮╰╯` nos temas modernos (nord, gruvbox, slate,
   graphite, daylight, vivid); os clássicos (classic, norton, dark, light, mono,
   solarized) seguem retos. Opção `borders` = `"theme"` (padrão), `"plain"` ou
   `"rounded"`.

## Testes

Testes de desenho com `TestBackend`: cores por tipo e a opção desligada, ênfase de
tamanho e data, os traços do caminho na árvore, a barra do disco, o cabeçalho com `~`
e com máquina remota, a barra de rolagem só quando precisa, as bordas por tema e pela
opção. Depois, capturas com `docs/screenshots/tools/run.sh` para o autor ver.
