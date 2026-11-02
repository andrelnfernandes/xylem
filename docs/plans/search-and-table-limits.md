# Plano: busca em entrada enorme e o que os leitores de tabela não limitavam

Origem: itens 4 e 5 de Security no [BACKLOG](../../BACKLOG.md), semana 3 de
[roadmap.md](roadmap.md).

## Modelo de ameaça

Quem criou o arquivo controla os bytes, os tamanhos declarados e a forma como as
células se referem umas às outras. Nada disso pode prender a tela sem saída, esgotar a
memória ou derrubar o programa. Lembrete: falha de alocação em Rust **aborta**, e
`panic_guard` não pega; o terminal fica bagunçado e o trabalho aberto se perde.

## O que foi conferido antes de desenhar

Medido com arquivos montados à mão, em 2026-11-01:

| Suspeita | Resultado | Consequência |
|---|---|---|
| ZIP que mente sobre os tamanhos | A `zip` 9 já limita a saída ao tamanho declarado: `File is larger than its declared uncompressed size`. Com isso, a soma de `check_unpacked` vale de verdade | Só falta um teste de regressão, para uma troca de versão não desfazer isso |
| O que o backlog não citava: `Range::from_sparse` do calamine | Um `.xlsx` de **1,4 KiB** com células em `A1` e `XFD1048576` pede **550 GiB** e **aborta o programa**. Roda também no Autoview, então basta o cursor passar pelo arquivo | É o problema mais grave desta frente |
| Texto compartilhado | Uma string longa referida por muitas células é copiada em cada célula pelo `worksheet_range` | Some com a leitura em fluxo: a célula pega a string emprestada |
| Página de Parquet | Snappy e zstd param no tamanho declarado, que pode chegar a 2 GiB por página e coluna. **GZIP lê até o fim**, sem limite algum. Os codecs LZ4 nem estão compilados: a feature ligada é `lz4_flex`, não `lz4`, e um arquivo com eles dá erro | Conferir os cabeçalhos das páginas antes de ler, e descomprimir com limite as páginas GZIP |
| Dicionário de Parquet | Uma string de 100 MiB no dicionário, referida 1024 vezes, vira 100 GiB num lote de 1024 linhas | O tamanho do lote passa a depender do tamanho dos dicionários |
| Hook para o codec do Parquet | Não há: `PageHeader` e o leitor de Thrift são privados na crate | Um leitor próprio e pequeno de cabeçalho (Thrift compact) |
| Esc numa tabela | A leitura roda na linha da tela, com até 3 s de limite, e a planilha nem olhava o relógio | Thread própria, como na listagem de archives e no Batch |

## Item 4: busca com o leitor aberto entre os passos

`SearchJob::step` passa a guardar o leitor da entrada atual e o que já leu da linha.

- Volta quando o tempo do passo acaba, e continua de onde parou no passo seguinte.
- Dentro de um archive, a thread do `Extractor` descomprime, e a busca espera por um
  bloco só por um instante: uma entrada que demora (ou que gira sem produzir nada) não
  prende a tela. `WouldBlock` significa "volte depois", não erro.
- Um único `Extractor` serve a busca toda, e abre o archive na thread dele. Antes, cada
  arquivo procurado lia o índice do ZIP ou 7z na linha da tela.
- Esc solta o leitor, e isso para a thread de extração.
- Vale para qualquer origem: arquivo no disco, archive e SSH. O SSH ainda bloqueia
  durante a leitura de um bloco.

## Item 5: leitores de tabela

### Planilhas

1. **`.xlsx` e `.xlsb` em fluxo,** pelo `worksheet_cells_reader`: uma linha por vez na
   memória, sem `from_sparse`. O texto compartilhado vem emprestado.
2. **A leitura do arquivo olha o orçamento** (`Stoppable`): tempo e Esc param também a
   abertura do workbook e o XML do meio de um Deflate. A checagem fica antes do
   descompressor, então pega também a bomba que gira sem produzir nada.
3. **Colunas:** a letra é a da própria planilha, a partir de A. Quem começa em C mostra
   A e B vazias. O `<dimension>` serve só de largura inicial: ele pode estar errado, e
   confiar no início dele esconderia dados à esquerda. Coluna além disso alarga a
   tabela, como no JSON Lines.
4. **Linhas:** vazios no meio viram linhas vazias, como antes, e as vazias do começo não
   aparecem. Linha além de 1.048.576 (limite do formato) é ignorada, e também célula de
   linha anterior à atual: cada linha sai uma vez só, então o que um arquivo consegue
   pedir fica no limite do formato, mesmo no Batch, que não tem limite próprio. Um
   arquivo com linhas fora de ordem perde as que voltam.

`.ods` e `.xls` não têm leitura por célula no calamine: ver "O que continua".

### Parquet

O leitor da crate recebe um arquivo vigiado (`GuardedFile`), que olha cada página no
caminho, e só as que forem lidas. Conferir tudo antes da primeira linha gastava os 3 s
num arquivo grande e saudável, e cada página era mais uma leitura na nuvem.

1. **Ao abrir um row group:** só o cabeçalho da primeira página de cada coluna, que é
   onde fica o dicionário. O lote é de `min(1024, 1 GiB / soma dos dicionários)` linhas.
   Dicionário em outro lugar é recusado, porque o lote conta com ele.
2. **Antes de cada página:** o cabeçalho. A soma, por coluna, do dicionário e da maior
   página vista é o que fica na memória ao mesmo tempo. Acima de 1 GiB, recusa com
   mensagem. O limite é fixo, e não o do viewer: arquivo legítimo com 256 colunas de
   1 MiB precisa passar.
3. **GZIP:** antes de entregar a página, descomprime com limite no tamanho declarado,
   descartando a saída, sobre os bytes já lidos. Página que produz mais que isso é
   recusada. Custa uma descompressão a mais só nas páginas lidas desse formato.
4. **O orçamento é olhado a cada row group:** um rodapé com milhões de grupos vazios não
   tem outra coisa que olhe o relógio.

### Esc e a tela livre

- **V** lê a tabela em outra thread. Espera 150 ms pela resposta, para que uma tabela
  comum apareça como sempre, sem piscar o texto. Depois disso a tela fica livre,
  mostrando o arquivo como texto, com a nota `Reading the table... Esc stops`.
- **Trocar de planilha ou de tabela e rodar SQL** vão pelo mesmo caminho.
- **Esc** para a leitura e não fecha o viewer; o Esc seguinte fecha. O que já foi lido
  aparece, com `stopped by Esc: showing N rows`. Sem nenhuma linha, o viewer fica como
  estava: em texto, ou na tabela anterior.
- **Se a thread não responde em 200 ms** (por exemplo, no meio de um `.ods`), a tela não
  espera por ela: fica para trás e termina sozinha. Enquanto ela vive, nenhuma outra
  leitura começa, em nenhum viewer, e a nota diz isso. Repetir V e Esc num `.ods` hostil
  não acumula memória.
- **Recarregar** uma tabela ainda em leitura mostra o texto até ela chegar. Se o modo em
  uso for outro (Hex, por exemplo), a tabela chega sem mudá-lo, e T vai até ela.
- **O limite de tempo continua** (3 s no V). Sair do viewer cancela a leitura.
- **Autoview** continua síncrono, com os 100 ms dele: uma thread por movimento do cursor
  não compensa, e agora a planilha também respeita esse tempo. `.ods` e `.xls` não abrem
  como tabela nele, como SQLite e Parquet: a montagem deles não para, e rodaria na linha
  da tela. V abre.
- **Leitura parada sem nenhuma coluna** (planilha ainda abrindo): continua em texto, com
  a nota dizendo o motivo, em vez de uma tabela vazia.

## O que continua (fica no BACKLOG)

- **`.ods` monta a planilha inteira no V e no Batch:** o calamine limita a 100 milhões de
  células, ou cerca de 3 GiB, e isso cabe num arquivo pequeno com
  `number-rows-repeated`. Tempo e Esc param a leitura do arquivo, mas não a montagem no
  fim; a tela fica livre, e há no máximo um leitor assim no viewer. Um `.xls` é limitado
  pelo formato (65.536 × 256 células, cerca de 512 MiB).
- **Letras de `.ods` e `.xls`:** continuam contando da primeira coluna usada, diferente
  de `.xlsx` e `.xlsb`.
- **Parquet com listas:** uma linha pode ter muitos valores apontando para o mesmo item
  do dicionário, então o lote menor não limita esse caso por completo.
- **Página de Parquet em si:** é descomprimida inteira, dentro do limite acima; Esc a
  espera terminar, mas a tela não.
- **Parquet com LZ4:** continua sem abrir, como hoje. Ligar a feature `lz4` é outra
  decisão, porque aumenta o que lê arquivo hostil.
- **Procurar o próximo arquivo marcado dentro do viewer (F em Tagged View):** continua
  lendo cada candidato de uma vez, e a tela espera por eles.

O Batch não precisou de nada próprio: já roda em thread, com cancelamento, e ganha os
mesmos limites.

## Testes

- Busca: uma entrada grande leva vários passos, com orçamento zero, e o resultado é o
  mesmo; `WouldBlock` no meio de uma linha não perde a linha; entrada de archive que
  demora não prende o passo; Esc para a extração.
- Planilha: o `.xlsx` de 1,4 KiB que abortava lê em instantes; ZIP que mente é recusado;
  texto compartilhado não é copiado; orçamento vencido para a abertura; `<dimension>`
  errado não esconde dados; linhas que voltam não geram linhas vazias a mais; Autoview
  não abre `.ods`.
- Parquet: cabeçalhos de um arquivo real (com estatísticas e dicionário) são lidos por
  todos os codecs habilitados; página acima do limite é recusada; bomba de GZIP com
  tamanho declarado falso é recusada sem descomprimir tudo; página nunca alcançada não
  custa nada; grupos vazios olham o relógio; lote diminui com dicionário grande.
- Esc: a tela não espera, a leitura para, o parcial aparece; o Esc seguinte fecha; uma
  thread que não responde fica para trás, e é a única; recarregar não deixa a tela em
  branco nem muda o modo em uso.
