# Plano: o Xylem como cockpit de programação

**Rascunho.** Origem: conversa de 2026-11-01. O autor quer, com o tempo, que o Xylem seja
sua principal ferramenta de programação, como o XTreeGold foi na época do Clipper. Ainda
vamos trabalhar muito em cima deste plano: nada aqui tem data, e fica para um release
depois da 0.6.0, a definir no [roteiro](roadmap.md).

**Prioridade do autor:** busca no projeto, lista de locais e editor na linha certa. Antes
de implementar, cada fase ganha spec próprio.

## A ideia

Um cockpit em volta do editor e do compilador de quem programa, sem substituir nenhum dos
dois. O Xylem não vira uma IDE: não edita texto, não entende a linguagem, não tem servidor
de linguagem. Ele cuida de achar, navegar e chamar a ferramenta certa no lugar certo.

O ciclo a fechar é **procurar, editar, compilar, ver o erro, voltar ao ponto**, sem sair do
Xylem.

## Fase 1: o que vem primeiro

Com estas três peças já dá para programar de dentro do Xylem. O editor na linha vem antes
porque é pequeno, tem valor sozinho e a lista depende dele.

### 1. Editor na linha certa

Hoje `shell::editor` e `shell::editor_command` só recebem o arquivo.

- **E no visualizador** abre o editor na linha que está na tela, e na lista de locais, na
  linha do local.
- **`%l` em `[editors]`** é a linha: `'*.rs' = 'code -g %f:%l'`.
- **Editores conhecidos** recebem a linha sem configurar nada: `vi`, `vim`, `nvim`, `nano`,
  `emacs`, `micro`, `hx`, `code`. A forma de cada um será conferida na hora de fazer, no
  programa real. O nome do programa já é conhecido (`External::program_name`).
- **Editor desconhecido** abre o arquivo no começo, e o Xylem diz que não soube passar a
  linha. Nunca tenta adivinhar.
- **A linha é só um número** gerado pelo Xylem, nunca texto vindo de um arquivo.

### 2. Lista de locais

Uma janela única para listas de `arquivo:linha: texto`. É a peça central: a busca é a
primeira fonte, e as outras (fase 2) reaproveitam tudo sem tela nova.

- **Enter** abre no visualizador naquela linha; **E**, no editor naquela linha.
- **T** marca o arquivo, e as operações em grupo continuam valendo.
- **Voltar à lista** com o cursor onde estava, depois do visualizador ou do editor.
- **Próximo e anterior** sem precisar voltar à lista, como o Espaço em Ctrl+V.
- **Local que mudou:** se o arquivo mudou depois da lista, a linha pode ter andado. O Xylem
  abre mesmo assim e avisa; nunca finge que está certo.

### 3. Busca no projeto

Hoje Ctrl+S só desmarca os arquivos sem o termo, e `search.rs` responde sim ou não por
arquivo. Falta devolver as linhas.

- **Onde:** no ramo atual, ou nos arquivos marcados, se houver.
- **O quê:** o mesmo `SearchSpec` de Ctrl+S (texto, maiúsculas, `*`), mais expressão
  regular com o crate `regex`, que não tem o risco de tempo exponencial.
- **Sem dependência externa:** não exige `rg` instalado. Candidato para respeitar o
  `.gitignore`: o crate `ignore`, em Rust; conferir com `cargo tree | grep -- -sys`.
- **Em segundo plano:** thread própria, resultados chegando na lista enquanto ela roda,
  progresso no rodapé e Esc para parar, como ao abrir archives.
- **Por padrão, fora:** `.git`, `target`, `node_modules` e binários, com um jeito claro de
  incluir, e o rodapé dizendo o que ficou de fora.

## Modelo de ameaça

Tudo que está num repositório baixado é hostil, inclusive nomes e conteúdo.

- **Nomes e linhas de texto** passam pelo mesmo tratamento do resto: sem controle nem bidi
  na tela, e tamanho limitado por linha.
- **Memória:** limite de resultados e de bytes guardados. Passou disso, para e diz que
  cortou, nunca mostra uma lista cortada como se fosse completa.
- **Arquivos gigantes ou pastas enormes:** lidos em streaming, com a tela livre e Esc.
- **Links simbólicos** não são seguidos, para a busca não sair do ramo nem entrar em ciclo.
  FIFOs e dispositivos nunca são abertos.
- **`.gitignore` hostil** só consegue esconder arquivos da busca: por isso o aviso do que
  ficou de fora, e um jeito de ignorá-lo.
- **Nada é executado:** procurar e listar não rodam nada do repositório.
- **Caminho na lista:** ao abrir, confere de novo que é um arquivo comum.

**Rodar comandos do projeto (fase 2) muda a promessa.** Olhar um repositório não executa
nada dele; compilar executa (`build.rs`, `Makefile`). Só pode acontecer por pedido
explícito, nunca sozinho. Vale conferir como isso se liga a `git_trusted`.

## Para decidir

- **Teclas:** da busca, da lista e de voltar a ela. `F` é Find no visualizador, e as teclas
  sugeridas (`Ctrl+F`, `Ctrl+P`, `Alt+G`) precisam ser conferidas contra o mapa atual e
  contra o ZTreeWin.
- **Fora do disco local:** nuvem (cada arquivo lido custa uma requisição), SSH e archives.
  Começar só no disco local, e dizer isso na tela.
- **Regex ou texto como padrão**, e como trocar no prompt.
- **`ignore` ou um percurso próprio**, que já existe para o Log e os duplicados.
- **Ordem no roteiro:** antes ou depois da escrita por SSH. Decisão do autor; por enquanto
  a ordem das frentes não muda.

## Fase 2: depois, conforme fizer falta

A régua é o uso: programar no Xylem a partir de agora e anotar cada atrito. Ideias, sem
ordem nem compromisso:

- **Erros do terminal:** uma tecla lê a tela do terminal integrado e leva os `arquivo:linha`
  para a lista de locais. Funciona com qualquer compilador, sem manter lista de
  ecossistemas. Provável substituto de um "task runner" com detecção de projeto.
- **Repetir o último comando** daquele projeto.
- **Outras fontes da lista:** trechos alterados do Git, `TODO` e `FIXME`.
- **Editor no painel:** o editor no lugar do terminal, com a árvore visível em cima.
- **Conjuntos de trabalho:** marcações salvas com nome, por projeto, fora do repositório.
- **"O que eu mexi":** Showall só do que o Git aponta como alterado.
- **Achar arquivo digitando trechos do caminho.**
- **Trocar de branch** pelo menu Git.
- **Diff lado a lado:** ver o JFC no [BACKLOG](../../BACKLOG.md).
- **IA sobre os marcados:** já dá com `$XY_TAGGED`, `Alt+O` e o eXecute. Basta documentar,
  sem integrar nenhum provedor.
- **Dobrar JSON e YAML:** depois do visualizador em janelas, que mexe na mesma parte.
- **Outline de símbolos:** tree-sitter traz C, e o Xylem está saindo do C. Começar, se fizer
  falta, com `ctags` opcional.

## Fora, por enquanto

Editar texto dentro do Xylem, servidor de linguagem, depurador e um rascunho próprio (um
arquivo fixo aberto no editor já resolve).

## O site

É uma boa história para contar, se chegar lá: o fluxo do XTreeGold para quem programa hoje.
