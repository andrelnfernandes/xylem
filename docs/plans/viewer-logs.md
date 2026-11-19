# Plano: logs no viewer (subprojeto 4)

Origem: conversa de 2026-11-16, que escolheu sete viewers novos; a lista e a
ordem estão em [viewers.md](viewers.md), na seção "Viewers novos". Este é o
subprojeto 4 de lá. O filtro de linhas (`&`) já está feito. **Implementado em 2026-11-16 no branch `viewers-next`, ainda sem revisão do autor** (veja "Implementação", no fim).

## Objetivo

Um log abre como hoje, em texto e em janelas quando é grande, e ganha o que um
log pede:

1. **Ir ao fim na hora**, sem esperar a contagem de linhas (?3 de
   [text-windowed.md](text-windowed.md)).
2. **Seguir o arquivo** enquanto ele cresce, como `tail -f`.
3. **Cor por nível** (erro, aviso, info, debug).
4. **Cor ANSI** que o próprio programa gravou, só SGR de cor, e nada mais.
5. **Log comprimido** (`.log.gz`, `.log.1.gz`): ir ao fim em segundo plano, com
   Esc.

## O que conta como log

- Pelo nome: `.log`, `.out`, `.err`, um nome rodado (`app.log.1`,
  `app.log.2026-11-16`, `app.log.1.gz`), `syslog`, `messages`, `kern.log`,
  `auth.log` e `*.journal.txt`.
- Pelo conteúdo, no `V` e só nos primeiros 64 KiB: se a maioria das primeiras
  200 linhas começa com uma data e hora reconhecida (ISO 8601, syslog
  `Nov 16 10:00:00` ou `[2026-11-16 ...]`), o arquivo é tratado como log mesmo
  sem o nome.
- O cabeçalho diz `LOG`. A tecla `c` desliga as cores, como faz com a sintaxe.

## Decisões

### Ir ao fim sem esperar a contagem

- Com o arquivo em janela, End lê de trás para a frente: busca o último `\n`
  nos últimos 64 KiB e mostra as últimas linhas da tela.
- As linhas do fim ainda não têm número até a contagem chegar lá. A margem
  mostra `~` no lugar do número, como o `less`. Quando a contagem termina, os
  números aparecem sem mudar o lugar da tela.
- Subir a partir do fim continua lendo para trás, em blocos de 64 KiB, com o
  mesmo teto de `AROUND` linhas guardadas em volta da tela.
- `g` para uma linha ainda não contada continua dizendo que a contagem não
  chegou lá. Só End usa o fim lido para trás.
- Isso vale para todo texto em janela, não só para log. É a parte que falta de
  [text-windowed.md](text-windowed.md).

### Seguir o arquivo

- **Sem tecla nova:** quando a tela está na última linha e o arquivo cresce, as
  linhas novas entram e a tela continua no fim. Sair do fim (subir ou buscar)
  para de seguir. Voltar com End retoma. O rodapé mostra `FOLLOW` enquanto
  segue.
- A conferência é por tempo, a cada 500 ms, com o tamanho e a data
  (`records::file_stamp`), sem `notify`, porque o `notify` não funciona em
  rede, SSHFS e rclone.
- Se o arquivo **cresceu**, os últimos 4 KiB do tamanho antigo são comparados
  com os de antes. Iguais: lê só o acréscimo. Diferentes: o arquivo foi
  reescrito, e vale o aviso de hoje, `the file changed: R reloads`.
- Se o arquivo **encolheu** ou trocou de inode (rotação), aparece o aviso
  `log rotated or truncated: R reloads`. Nada é relido sozinho, porque a nota
  da tela precisa continuar valendo para o que se vê.
- Com o filtro `&` ligado, as linhas novas passam pelo filtro e só as que casam
  aparecem (ideia já anotada no BACKLOG).
- Não vale para arquivo comprimido nem remoto por SFTP. O remoto fica para
  depois.

### Cor por nível

- O nível de cada linha é procurado só nas primeiras 120 colunas, como palavra
  inteira, sem distinguir maiúscula de minúscula:
  - erro: `FATAL`, `CRIT`, `CRITICAL`, `PANIC`, `ERROR`, `ERR`, `EMERG`,
    `ALERT`;
  - aviso: `WARN`, `WARNING`;
  - info: `INFO`, `NOTICE`;
  - debug: `DEBUG`, `TRACE`.
- Também reconhecidos: o `<n>` de prioridade do syslog no começo da linha e o
  campo de nível de um log em JSON (`"level":"error"`, `"lvl"`, `"severity"`),
  sem montar o JSON, só pelo texto.
- Uma linha sem nível (continuação, stack trace) herda o nível da anterior, no
  máximo por 200 linhas.
- A cor é calculada só para as linhas da tela, então não custa nada num arquivo
  de qualquer tamanho.
- As cores vêm do tema: as mesmas de erro, aviso e destaque que a interface já
  usa.

### ANSI dentro do log

- Hoje o ESC vira `·` e o resto da sequência aparece como texto (`·[31m`).
- No log, as sequências são lidas por um parser de verdade (`vte`, que já está
  na árvore de dependências pelo `vt100`) e:
  - SGR de cor (`ESC [ ... m`) vira cor da linha, pela mesma conversão que
    `highlight::from_ansi` usa hoje para o Git;
  - **todo o resto some**: CSI que não é SGR, OSC (incluindo o OSC 52, que
    escreve no clipboard, e o OSC 8, de links), DCS, APC, PM e SOS, e também
    as formas de 8 bits (C1, como `0x9B`);
  - o cabeçalho conta o que foi removido (`ANSI: 12 removed`), como o contador
    bidi conta o que foi marcado.
- A cor ANSI ganha da cor por nível na mesma linha.
- O `H` (hex) mostra os bytes como estão, e a busca procura no texto já sem as
  sequências.

### Log comprimido

- Ir ao fim num `.gz`, `.xz`, `.zst` ou `.bz2` exige descomprimir tudo. Isso
  roda numa thread, joga fora o que não está nas últimas linhas, e tem Esc,
  prazo e teto de bytes descartados, como o `archive::Extractor`.
- Teto padrão: 4 GiB descomprimidos ou 60 s, o que vier primeiro. Ao parar,
  mostra o que tiver e diz por quê.

## Segurança

- Nada muda no que chega ao terminal: os caracteres de controle continuam
  virando `·` e o bidi continua filtrado no fim de `ui::draw`. O parser de ANSI
  só **tira** coisas. A única coisa que ele produz é cor.
- Um arquivo que cresce sem parar (um processo hostil escrevendo) não ocupa mais
  memória: a janela guarda só `AROUND` linhas, e a contagem segue em segundo
  plano como hoje.
- A leitura para trás confere cada posição contra o tamanho atual do arquivo e
  trata o arquivo que encolheu entre duas leituras como rotação.
- Linha gigante (sem `\n`): continua valendo o `MAX_LINE` de 1 MiB, também na
  leitura para trás.
- O parser de nível não usa regex sobre a linha inteira: é uma busca de palavras
  nas primeiras 120 colunas, com custo linear.
- proptest: bytes aleatórios com ESC, C1 e `\r` misturados nunca fazem um byte
  de controle chegar ao que é desenhado, e nunca dão pânico. A leitura para trás,
  com o arquivo mudando entre os passos, nunca dá pânico.

## Etapas (um PR cada)

1. **End lido do fim** em todo texto em janela, com `~` na margem. Fecha o ?3.
2. **Detecção de log, cor por nível e `c`.**
3. **ANSI com `vte`**, só no log.
4. **Seguir o arquivo**, com rotação e o filtro `&`.
5. **Fim de log comprimido** em segundo plano.

Cada etapa entra no manual (`viewer.md`) e no BACKLOG ao terminar. As traduções
do manual ficam para o fim do trabalho, como de costume.

## Questões abertas

| # | Questão | Proposta |
|---|---|---|
| ?1 | Seguir sem tecla, só por estar no fim, ou com uma tecla (o `F` do `less` já é Find aqui)? | Sem tecla: estar no fim é seguir |
| ?2 | Detectar log pelo conteúdo, além do nome? | Sim, só no `V` e só nos primeiros 64 KiB |
| ?3 | O Autoview também segue o arquivo? | Não: ele lê 1 MiB e se fecha ao trocar de arquivo |
| ?4 | Teto do fim de log comprimido | 4 GiB descomprimidos ou 60 s |

## Para depois

- Seguir um arquivo remoto (SFTP), com o mesmo intervalo.
- Ir para uma hora (`g 10:30`), com busca binária pelas datas das linhas.
- Juntar os arquivos rodados (`app.log.3.gz` … `app.log`) numa leitura só.

## Implementação (2026-11-16)

Feito (commit `cd69f5f`), em `src/viewer/log.rs` e `src/viewer.rs`:

- detecção pelo nome e pelo conteúdo, com as listas de nomes e de formatos de
  data desta página;
- cor por nível, feita linha a linha (`Highlighter::per_line`, em
  `highlight.rs`), que por isso vale também em arquivo em janela; a linha sem
  nível herda o anterior até 200 linhas, quando o arquivo é lido inteiro;
- ANSI com o `vte`: a cor SGR fica e todo o resto sai e é contado no cabeçalho
  (`[ANSI: n removed]`); as formas de 8 bits viram as de 7 antes do parser;
- seguir o arquivo sem tecla nova (?1 resolvido como proposto): a cada 500 ms,
  com os últimos 4 KiB conferidos; em janela, a contagem continua de onde
  parou (`TextWindow::grow`); rotação ou reescrita deixa `(the file changed: R
  reloads)` no cabeçalho;
- ?2 resolvido de outro jeito: a detecção pelo conteúdo vale também no
  Autoview, porque custa só 200 linhas; ?3 como proposto (o Autoview não segue).

Diferente do spec:

- **ANSI só no arquivo lido inteiro** (até o limite, 16 MiB por padrão). Num
  arquivo em janela, as sequências continuam aparecendo como `·`, como em
  qualquer texto: a janela lê as linhas do disco, e tirar sequências mudaria as
  colunas que a busca e o filtro usam. A cor por nível continua valendo ali.

Falta:

- **etapa 1:** End lendo o texto em janela de trás para a frente (o ?3 de
  `text-windowed.md`); hoje End espera a contagem, que é rápida (`memchr`);
- **etapa 5:** o fim de um log comprimido em segundo plano;
- seguir um arquivo remoto.
