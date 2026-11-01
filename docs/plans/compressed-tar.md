# Plano: abrir archives em segundo plano

Origem: item 1 de Security no [BACKLOG](../../BACKLOG.md), primeira frente de
[roadmap.md](roadmap.md), aprovada pelo autor em 2026-11-01 para a 0.6.0.

## O problema

Entrar num `.tar.gz`, `.tar.xz`, `.tar.bz2` ou `.tar.zst` (Alt+F5) descomprime tudo,
na linha de execução da tela (`Pane::from_archive`). O formato não tem índice. Uma
bomba de descompressão congela o Xylem, e nem Esc funciona. Copiar de dentro lista
tudo de novo em `Operation::reading_archive`: segundo congelamento, igual ao primeiro.

## Modelo de ameaça

Quem criou o archive controla os bytes, os nomes e os tamanhos:

- **Bomba de dados:** poucos KiB viram TiB. Não pode prender a tela.
- **Bomba de nada:** blocos, membros ou frames vazios fazem o descompressor girar sem
  produzir saída, então conferir só a saída não basta.
- **Bomba de entradas:** cabeçalhos pequenos que descomprimem em milhões de entradas,
  ou nomes longos, esgotam a memória mesmo com a tela livre.
- **Pastas com milhões de filhos:** a montagem da árvore, com busca linear, ficaria
  quadrática.
- **`panic` num parser:** não pode derrubar o programa nem bagunçar o terminal.

## O que muda

1. **`archive::Listing`:** abre e lista em outra thread, para todos os formatos, e
   monta ali também a árvore virtual. O ZIP e o 7z leem índice ao abrir, que também
   vem de fora, então vão pelo mesmo caminho.
2. **Cancelar:** soltar o `Listing` cancela, e Esc volta na hora, sem esperar a thread.
   O leitor confere o pedido dos dois lados do descompressor: na entrada (bomba de
   nada) e na saída (bomba de dados).
3. **Progresso** no rodapé, com entradas e bytes lidos. Não há limite de tempo: como a
   tela fica livre, quem decide é o usuário, e um `.tar.xz` legítimo e grande abre.
4. **Limites de memória:** no máximo 1 milhão de entradas e 256 MiB de caminhos somados.
   Passou disso, o archive é recusado com uma mensagem clara, em vez de mostrar uma
   lista cortada que pareceria completa.
5. **Árvore sem custo quadrático:** `Tree::from_listing` acha os filhos num mapa.
6. **Reaproveitar a lista:** a janela guarda as entradas, e a cópia usa essas, sem
   descomprimir de novo. O `Extractor` abre o archive na própria thread.
7. **`panic`:** a thread roda em `panic_guard`, e vira mensagem de erro.

Durante a leitura, o resto do Xylem continua respondendo, como ao conectar por SSH, e a
janela muda quando a leitura termina.

## Fora desta etapa

- **Copiar N arquivos de um TAR comprimido** ainda descomprime desde o começo para cada
  um. Agora dá para cancelar, mas é lento; vai para o BACKLOG.
- **Busca (Ctrl+S) e viewer dentro do archive:** têm limites próprios; o item 4 de
  Security trata a busca.

## Testes

- Bomba real (`.tar.gz` de zeros, com poucos MiB comprimidos): a abertura volta na hora,
  e cancelar encerra a thread.
- Archive normal abre pelo novo caminho; cópia usa a lista guardada, mesmo que o arquivo
  mude depois de aberto.
- Limite de entradas e de caminhos recusa com mensagem.
- Pasta com muitos filhos monta a árvore.
- O `Extractor` informa o erro de um archive que sumiu.

## Medido

Binário de release, numa máquina de desenvolvimento, em 2026-11-01:

| Archive | Resultado |
|---|---|
| Bomba `.tar.bz2`: 1,3 MiB que viram 200 GiB | 11 MiB de memória; F1 respondeu em 14 ms e Esc em 13 ms, com a leitura em andamento; depois do Esc, a thread terminou e a CPU foi a zero |
| `.tar.gz` com 999.000 entradas | abriu em 1,8 s; 484 MiB no pico e 383 MiB depois: é o pior caso que o limite permite, com caminhos curtos |
| `.tar.gz` com 1.000.100 entradas | recusado, com a mensagem; 97 MiB no pico e 11 MiB depois |
