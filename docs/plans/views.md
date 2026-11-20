# Plano: visões fora do visual clássico (Linha do tempo, Varredura, Tabela)

**Proposta para a 0.9.0** (pedida em 2026-11-20). Nada aprovado para código.

Origem: o autor propôs romper com o visual clássico em situações específicas,
por um comando. Critério combinado: só vale uma tela nova que responda a uma
pergunta que a árvore e a lista não respondem bem; o visual clássico continua o
padrão, e cada visão é ida e volta. As três visões saem do que o Xylem já tem na
memória (tamanho, data, tipo, permissões, status do Git) ou dos relatórios que
já existem por arquivo.

Descartados na conversa: treemap (ilegível em texto; o Usage responde melhor),
imagens (um protocolo por terminal, e um decodificador alimentado com arquivos
hostis), grafo do Git (fora do papel do Xylem), modo foco (trivial e sem valor
para quem chega).

## A tecla: um seletor de visões

Uma tecla abre um seletor no estilo dos favoritos: **Timeline**, **Scan**,
**Table**, cada um com a sua letra. A visão vale para **a lista atual**: o
diretório, o Branch, o Showall, com o Filespec e o `W` que estiverem valendo.
Esc volta ao visual clássico, na mesma posição.

Tecla a decidir antes da etapa 1. Letras livres hoje: G, H, J, K, Z; G, H e J
estão reservadas para comandos que ainda faltam (Global, sHortcut, JFC), e Z
está proposta para o zoxide. Sobra **K**, ou uma
combinação com Alt ou Ctrl livre (a conferir no `keymap.rs`). Cada visão também
pode ter tecla própria em `[keys]`, sem padrão.

As três visões são só leitura: nada nelas apaga, move ou muda arquivos. Quando
uma visão leva a uma escolha (uma barra, uma categoria, uma linha), a escolha
volta para a lista clássica como filtro, e de lá seguem Ctrl+T, Ctrl+C, Batch
como sempre.

## 1. Varredura (Scan): o que há de perigoso nesta pasta

Os relatórios de arquivos não confiáveis, hoje um arquivo por vez com `V`,
aplicados a uma lista inteira: um ZIP baixado, um repositório clonado, um
pendrive.

### O que mostra

Uma tela por categoria, com a contagem, e Enter abre a lista dos arquivos da
categoria (e `V` em cada um mostra o relatório de sempre):

| Categoria | De onde vem | Lê o conteúdo? |
|---|---|---|
| Nomes com caracteres de direção do texto ou de controle | `bidi.rs`, o nome | não |
| Nomes que enganam: extensão dupla (`fatura.pdf.exe`), espaços antes da extensão | o nome | não |
| setuid e setgid; arquivos e pastas que qualquer um pode escrever | o modo | não |
| Links simbólicos que saem da pasta varrida, ou apontam para o que não existe | o destino do link, sem segui-lo | não |
| Executáveis: empacotados (UPX e outros), sem PIE, sem NX, com seções graváveis e executáveis | `viewer/binaries.rs` | sim, o cabeçalho, com limite |
| Documentos do Office com macro, template remoto, objetos embutidos ou texto oculto | `viewer/office.rs` | sim, com os limites de hoje |
| Patches que tocam caminhos perigosos ou nomes com caracteres escondidos | `viewer/patch.rs` | sim, com limite |
| Scripts de instalação em pacotes, quando os formatos novos existirem | plano de formatos | sim |

### Como roda

- **Primeiro o que não lê nada** (nome, modo, link): aparece na hora, porque já
  está na memória.
- **Depois o que lê conteúdo**, em segundo plano, com progresso e Esc para
  parar. A tela se completa enquanto lê, e diz quantos arquivos faltam.
- **Limites**: o mesmo limite de leitura de cada relatório, um teto de arquivos
  lidos (configurável, padrão a decidir medindo), e um tempo por arquivo. Um
  arquivo que estoura o limite entra em "não analisado", com o motivo: nunca
  some em silêncio.
- **Numa montagem do rclone**, a parte que lê conteúdo fica desligada, como o
  `contains` do `W`: cada arquivo seria baixado. A tela diz isso e mostra só o
  que vem do nome e do modo. O mesmo numa máquina remota, até medir o custo pelo
  SSH.
- **Dentro de um arquivo compactado**: a varredura roda sobre a lista do
  arquivo, lendo cada membro com os limites de descompactação de hoje (bomba de
  ZIP já tratada).

### Segurança

A varredura é a função que mais lê arquivos hostis de uma vez. Ela não traz
parser novo: chama os mesmos que o `V` já usa, com os mesmos limites, um
arquivo por vez. Um parser que falha num arquivo marca aquele arquivo como "não
analisado" e segue (o `panic_guard` já cobre o viewer: conferir que cobre aqui).

A varredura não é antivírus e a tela diz isso: aponta o que merece um olhar,
não garante que o resto é seguro.

## 2. Linha do tempo (Timeline): o que mudou, e quando

### O que mostra

Os arquivos da lista num histograma pela data de modificação. A escala se
escolhe sozinha pelo intervalo das datas (horas, dias, semanas, meses, anos), e
`+`/`-` mudam a escala. Cada barra mostra, ao lado do cursor, quantos arquivos e
quantos bytes.

```
 nov 2026  ▁▁▂▁▁▁▁▁▃▁▁▂▁▁▁▁▁█▆▁   18/11: 312 files, 1.4 GiB
           1        10        20
```

- `B` alterna entre contar arquivos e somar bytes.
- Enter numa barra volta à lista com o período como condição. Um intervalo
  (Shift+← →) também.
- As datas no futuro e as anteriores a 1980 ficam numa barra à parte em cada
  ponta, porque datas hostis ou erradas existem e não podem achatar o gráfico.

### Como a escolha volta para a lista

Como uma condição do `W` sobre `modified` (`modified >= '2026-11-18' AND
modified < '2026-11-19'`), somada com AND à condição que já estiver valendo. O
título da janela mostra a condição, e `W` vazio a desfaz, como hoje. Não lê
nada do disco: só as datas já na memória.

## 3. Tabela (Table): a lista de arquivos no viewer de tabelas

### O que mostra

A lista atual aberta no viewer de tabelas, com as colunas que o `W` já conhece:
`name`, `ext`, `dir`, `size`, `disk`, `modified`, `age_days`, `mode`, `kind`,
`tagged`, `git`, `dup` (e `type` só se pedido, porque lê o começo de cada
arquivo).

Ganha o que as tabelas já têm: ordenar por qualquer coluna, o filtro SQL, o
registro como formulário, copiar um registro (`c`) e o Batch de uma linha por
registro (com uma máscara, um CSV, um script ou uma página).

### Como a escolha volta para a lista

- `T` numa linha marca o arquivo na lista clássica; as marcas aparecem como a
  coluna `tagged`.
- Esc volta para a lista; o filtro SQL usado na tabela pode virar o `W` da
  lista (uma pergunta, ou uma tecla própria), para os comandos que agem nos
  arquivos.

### A medir antes

Com 2.000.000 de nós (o teto da árvore), a tabela não pode copiar a lista
inteira para strings: as linhas têm de ser lidas da memória da árvore sob
demanda, como o viewer já faz com tabelas grandes por janela. Medir memória e
tempo de abertura com a árvore de teste maior antes de prometer a visão para
Showall de um disco inteiro.

## Etapas

Cada uma com a página do manual em inglês, as notas da versão e os testes; um
GIF quando valer.

1. **O seletor e a Varredura**, a parte sem leitura (nomes, modos, links).
2. **A Varredura com leitura**: executáveis, Office, patches, em segundo plano
   e com limites. Testes com os arquivos hostis do `fixture.sh` (os mesmos que
   o GIF dos arquivos não confiáveis vai usar).
3. **A Linha do tempo.**
4. **A Tabela**, depois da medição.

## Fora deste plano

- Agir em arquivos de dentro das visões (apagar, mover): a escolha sempre volta
  para a lista clássica.
- Salvar uma varredura como relatório (HTML, JSON): talvez depois, pelo Batch da
  tabela de resultados.
- Outras visões (composição por tipo): pode entrar como uma linha no topo do
  Usage, num plano próprio.
