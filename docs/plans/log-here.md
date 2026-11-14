# Plano: Alt+L, a pasta do cursor como raiz

Origem: conversa de 2026-11-14. O autor abriu o Xylem em `~`, deu `L` em `~/projeto`
e esperava que `~/projeto` virasse a raiz; o `L` só levou o cursor até lá, porque a
pasta já estava dentro da árvore. É o comportamento documentado, mas faltava um jeito
de estreitar a raiz sem sair do programa.

**Estado: aprovado pelo autor em 2026-11-14.** Branch `log-here`, a partir do `main`.

## O que muda

**Alt+L** faz da pasta do cursor a raiz da janela, na hora, sem prompt. Segue a regra
do Alt (`alt-ctrl-semantics.md`): na árvore, a versão do comando sobre o ramo, como
Alt+P (Prune) e Alt+R (Release).

- Vale nas duas janelas. Na lista de arquivos, é a pasta que a lista mostra; em
  Branch e Showall, a pasta do arquivo sob o cursor.
- Para uma pasta distante: `L caminho` leva o cursor até lá, Alt+L a torna raiz.
- Com o cursor já na raiz: nada muda, com uma mensagem que ensina a subir (`L ..`).
- O `L` fica como está.
- A volta é `L ..` (ou qualquer caminho de fora). Alt+L tem um sentido só.

## O que se perde

A janela é aberta de novo na pasta, como o `L` já faz com um caminho de fora: o que
estava lido e as marcas se perdem, **inclusive as de dentro do ramo**. A mensagem diz
quantas marcas foram perdidas, como a do Release. A ordenação e o modo de exibição
ficam, como no `L`.

Manter o ramo como estava (lido, expandido, marcado) pediria renumerar os nós da
árvore (`src/model/tree.rs`), com as leituras em segundo plano, o Usage e as vistas
que guardam `NodeId`. Fica para depois, se fizer falta.

## Onde não vale

- **Dentro de um ZIP/TAR:** recusado, como o `L`.
- **Numa máquina remota:** recusado por enquanto, com mensagem própria. A árvore
  remota nasce da raiz do endereço (`Pane::from_remote`); abrir numa subpasta pede
  conferir o que supõe essa raiz.

## No código

- `Command::LogHere` (`src/input.rs`), nome `log_here` na tabela e `alt+l` nas duas
  janelas (`src/keymap.rs`).
- `log_here` em `src/app/actions.rs`; as recusas em `refusal` (`src/app/commands.rs`).
- A palavra `Log` nos dois menus ALT e a dica da segunda linha (`src/ui.rs`).
- Testes em `src/app/tests.rs` e os do rodapé.
- Manual, nas seis línguas, por último: `treespec.md` (seção Log), `tree.md`,
  `tagging.md` (a lista do Alt) e `keys.md` se listar as teclas.

## ZTreeWin

Lá o Alt+L é do Log de disco (anotado em `alt-ctrl-semantics.md`, sem o detalhe).
Como o Xylem não tem discos, o sentido aqui é próprio; a divergência entra na tabela
de `ztreewin.md` se a página listar o Alt+L.
