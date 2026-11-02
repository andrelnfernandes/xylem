# Plano: nuvem sem tela em branco

Semana 3 do [roadmap](roadmap.md). Origem: o BACKLOG mediu mais de 29 s de tela em
branco ao abrir a raiz de um Drive grande, com o cache frio.

## A causa

`Tree::new` chama `expand`, que lê a raiz e depois **cada subpasta dela**, para saber
quais têm `+`. Tudo isso acontece antes do primeiro desenho, uma leitura por vez, cada
uma um pedido ao serviço. O mesmo vale para → numa pasta com muitas subpastas, e para
mover o cursor até uma pasta ainda não lida.

## Parte 1: leitura em segundo plano

Só em pasta montada pelo rclone (`fuse.rclone`). Disco local, archive e SSH continuam
como estão.

- **`Tree` ganha uma fila de leitura.** Em pasta lenta, `expand` e `select_row` pedem a
  leitura em vez de fazê-la, e voltam na hora. A tela é desenhada antes de qualquer
  pedido ao serviço.
- **Quatro threads** leem pela montagem; o rclone controla o ritmo dos pedidos. Elas só
  leem pastas que a pessoa abriu ou que estão a um nível de uma aberta, então a fila não
  cresce sozinha por causa de uma árvore hostil.
- **A pasta sob o cursor passa à frente** da fila.
- **O laço principal aplica o que chegou,** como faz com archive e conexão. A árvore e a
  lista de arquivos se atualizam mantendo o cursor. Enquanto a pasta sob o cursor não
  chega, a janela de arquivos diz que está lendo, para não parecer vazia.
- **Uma montagem que trava** segura só a thread, nunca a tela. Ao fechar, a thread presa
  é abandonada.
- **Sobrepõe sem perder nada:** se a mesma pasta foi lida de outro jeito no meio do
  caminho, o resultado que chega usa o mesmo caminho do re-log, que mantém as marcas e o
  que já foi lido abaixo.

## Parte 2: o ramo de uma vez

`*` já pede o ramo inteiro ao rclone. Branch (`B`), Usage (Alt+W) e Compare em ramo passam
a fazer o mesmo, com o mesmo progresso, Esc e pergunta sobre pedidos recusados.

- **Só se falta algo:** com o ramo já lido, abre na hora, sem pedido nenhum.
- **Sem expandir a árvore:** só `*` expande.
- **Se o rclone falhar,** lê pasta por pasta, como hoje, e avisa. Se o serviço recusar,
  para e avisa, sem insistir.
- **Usage não consulta o dispositivo de cada pasta** numa montagem do rclone: cada `stat`
  seria mais um pedido, feito no meio do desenho. Consequência: outro disco montado
  *dentro* de uma montagem do rclone não seria separado, caso que não existe na prática.

## O que continua síncrono

Uma leitura de cada vez, que ainda segura a tela se o serviço demorar:

- F3 e ir a um caminho digitado (Treespec, marcadores), que precisam do resultado na hora;
- abrir o Prune, que confere cada pasta antes de perguntar.

Ficam no BACKLOG.

## Testes

Sem rede e sem Drive: as threads leem um diretório temporário, com a árvore marcada como
lenta, e o rclone é o script falso que os testes de `*` já usam.
