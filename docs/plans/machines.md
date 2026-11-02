# Plano: máquinas e containers no F2, contextos do Docker e Zlist remoto

Origem: conversa de 2026-11-02, depois da fase 3 do SSH e do `docker://`
([ssh-write.md](ssh-write.md)). O autor perguntou se o F2 do `L` poderia mostrar os
containers locais, e pediu que o Zlist, quando vier, cubra também containers e imagens
conhecidos, mesmo remotos. **Registrado, sem data:** a ordem em relação ao resto entra
no [roadmap](roadmap.md). Cada parte ganha spec próprio antes do código.

## Hoje

- O F2 nos prompts que pedem pasta (Copy, Move, `L`, Ctrl+B, F5) abre o seletor em
  árvore (`src/picker.rs`), uma árvore por janela, só desta máquina.
- O `L` guarda o que foi digitado no histórico (seta para cima), endereços remotos
  incluídos, mas não descobre nada sozinho.
- `docker://[usuário@]container[/caminho]` conecta por `docker exec` ao `sftp-server`
  do container. O modelo de ameaça dessa fatia diz que o Xylem só roda `docker exec`.

## Parte 1: página Máquinas no F2

O seletor do F2 ganha uma página **Máquinas**, ao lado das árvores das janelas (Tab
já alterna entre janelas). Ela lista:

- **Containers**, de `docker ps -a` (ou o programa de `$XY_DOCKER`): os em execução
  abrem com Enter (`docker://nome`); os parados aparecem apagados e dizem por que não
  abrem (o `docker exec` precisa do container rodando; o Xylem não o inicia).
- **Hosts do `~/.ssh/config`:** as entradas `Host` sem curinga (`*`, `?`, `!`). Enter
  abre `ssh://host`. `Include` é seguido só dentro de `~/.ssh`, com limite de
  profundidade.
- **Endereços usados antes**, do histórico do `L`.

Com Enter, o endereço volta ao prompt do `L` e conecta como se fosse digitado.

### Modelo de ameaça

- **Um segundo comando do Docker:** `docker ps`, que só lê. A regra da fatia
  `docker://` passa a ser "só `ps` e `exec`". O formato é fixo
  (`--format '{{json .}}'`, uma linha por container) e nada digitado vai para ele.
- **A saída do `docker ps` é texto de fora:** qualquer um com acesso ao daemon escolhe
  nomes e rótulos. Linhas malformadas ou longas demais são descartadas; a saída tem
  limite de tamanho e de linhas; um nome só entra na lista se passar na validação de
  `Address::parse`. Nada da lista vira conexão sem Enter.
- **O `~/.ssh/config` é do usuário,** mas é lido como texto qualquer (limite de
  tamanho, de `Include` e de entradas). Só o nome do `Host` é usado; o `ssh` continua
  lendo o arquivo ele mesmo, como sempre.
- **Nada prende a tela:** o `docker ps` roda em segundo plano, com tempo limite e Esc.
  Um daemon remoto lento (`DOCKER_HOST=ssh://...`) só atrasa a lista.

## Parte 2: contextos do Docker

O CLI já fala com daemons em outras máquinas, por `DOCKER_HOST` ou por contextos
(`docker context`). Proposta:

- Endereço com o contexto antes do nome: `docker://contexto:container[/caminho]`.
  Nome de container não pode ter `:`, então não há ambiguidade com o formato atual. O
  contexto passa pela mesma validação de nome e vai como `docker --context <ctx>`,
  antes do `exec` e do `--`.
- A página Máquinas lista os containers de cada contexto (`docker context ls`, depois
  um `docker --context X ps` por contexto, em paralelo e com tempo limite).
- Podman: `podman --connection` é o equivalente; fica para quando houver uso.

## Parte 3: Zlist com máquinas remotas, containers e imagens

Vai junto com o Zlist (Parte 2 do roadmap), não antes. O arquivo salvo é lido como
hostil, como o roadmap já diz para o Zlist local.

- **Máquinas remotas e containers:** o Zlist guarda a árvore e o endereço. Reabrir
  mostra a árvore salva na hora, marcada como "salva em <data>", e conecta em segundo
  plano para atualizar. Até conectar, nada se escreve nela. Vale para `ssh://` e
  `docker://`, também com o daemon em outra máquina.
- **Imagens:** uma imagem não é um container em execução, então não há `docker exec`.
  Dois caminhos, a decidir no spec:
  1. **`docker export`** de um container criado e não iniciado (`docker create`): um
     TAR do sistema de arquivos inteiro, que o Xylem já abre em segundo plano, com Esc
     e como hostil. Custo: o `create` deixa um container que o Xylem precisa remover,
     e isso muda a regra de "só ler" do Docker. **Recomendado,** se esse efeito
     colateral for aceito.
  2. **`docker image save`:** não cria nada, mas é um TAR de TARs (uma camada por
     arquivo); mostrar o resultado exige aplicar as camadas, inclusive os arquivos
     apagados (`.wh.`), o que é mais código lendo dados de fora.

  Em qualquer caso a imagem é somente leitura, como um archive.

## Decisões pendentes do autor

- Parte 3, imagens: aceitar o `docker create` + `docker export` (com remoção do
  container criado), ou ficar no `docker image save`, ou deixar imagens de fora.
- Ordem das partes 1 e 2 em relação à fase 4 do SSH e ao JFC.
