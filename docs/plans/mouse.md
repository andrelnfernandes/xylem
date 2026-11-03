# Plano: mouse como complemento

Origem: conversa de 2026-11-02. O mouse estava "fora, por decisão" no
[roadmap](roadmap.md). O autor lembrou que alguma versão do XTreeGold o aceitava como
um extra, e decidiu que ele entra **como complemento, sem data**: coisas simples,
como escolher uma pasta com um clique. O teclado continua sendo o caminho completo.
Ganha spec próprio antes do código; o tamanho estimado é de uma fatia (uma semana).

## Escopo

- **Clique numa pasta da árvore:** cursor nela. Duplo clique: o mesmo que Enter.
- **Clique num arquivo da lista:** cursor nele. Duplo clique: o mesmo que Enter.
- **Clique na outra janela,** com a tela dividida: ela passa a ser a ativa.
- **Roda:** move na árvore, na lista, no viewer e no help.

Fora: arrastar, botão direito, cliques em menus, prompts e diálogos.

## Opção, desligada por padrão

Capturar o mouse tira do terminal a seleção de texto: enquanto o Xylem captura,
selecionar e copiar com o mouse passa a pedir Shift na maioria dos terminais. Para
quem copia caminhos e trechos da tela, é uma perda real, então o mouse é uma opção no
F12 (`mouse`), desligada por padrão.

## O que existe e o que muda

- O laço principal (`src/main.rs`, `event::read`) só trata `Event::Key`. O `crossterm`
  já entrega `Event::Mouse` (Windows incluído): sem dependência nova.
  `EnableMouseCapture` na entrada, quando a opção estiver ligada, e
  `DisableMouseCapture` no `restore`.
- `src/ui.rs` monta as áreas a cada desenho e não as guarda. É o trabalho principal:
  guardar, a cada desenho, onde ficou cada janela, a árvore e a lista, e o primeiro
  item visível de cada uma, para saber o que está sob o clique.
- A lista em colunas (`draw_file_columns`) calcula a largura de cada coluna ao
  desenhar; a conta precisa ficar disponível para achar o arquivo da posição clicada.
- **Terminal embutido (Alt+X) e editor no quadro:** o clique não deve ir para o Xylem.
  A captura sai enquanto eles têm o foco, e também ao chamar um programa externo
  (`run_external`), voltando depois.

## Segurança

Nada muda no modelo de ameaça: os eventos de mouse vêm do terminal, não do conteúdo
dos arquivos, e todo texto mostrado já passa por `bidi::sanitize`. Um arquivo hostil
não fabrica um clique. Coordenadas fora de qualquer área conhecida são ignoradas.

## Testes

Eventos `MouseEvent` sintéticos nos testes do app: clique e duplo clique na árvore e
na lista (inclusive em colunas), troca de janela na tela dividida, roda no viewer,
nada acontecendo com a opção desligada ou com o terminal em foco.
