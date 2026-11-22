# Visões (Alt+V)

**Alt+V** abre as visões da lista: outras formas de olhar os arquivos que a
janela de arquivos tem, para uma pergunta que a árvore e a lista não
respondem bem. Um pequeno menu pergunta qual, cada uma com a sua letra:

| Tecla | Visão | O que ela responde |
|---|---|---|
| `S` | [Varredura](#varredura) | O que nestes arquivos merece um olhar? |
| `T` | [Linha do tempo](#linha-do-tempo) | O que mudou, e quando? |
| `L` | [Tabela](#tabela) | Quais arquivos atendem a esta condição, vistos como tabela? |

Uma visão trabalha sobre **a lista atual**: o diretório, o Branch ou o
Showall, com o filespec e a condição `W` que estão em vigor. Ela só lê: nada
numa visão exclui, move ou altera um arquivo. O que você escolhe nela volta
para a lista clássica como um filtro, e dali Ctrl+T, Ctrl+C ou Batch agem
como sempre. **W sem texto** tira esse filtro, como faz com uma condição.

Cada visão também pode ter uma tecla própria em `[keys]`
([configuração](configuration.md)), sem tecla por padrão: `view_scan`,
`view_timeline`, `view_table`.

## Varredura

Os relatórios que o [visualizador](viewer.md) faz de arquivos que podem ser
hostis (executáveis, documentos do Office, patches), um arquivo por vez com
`V`, aplicados a uma lista inteira: um ZIP que você baixou, um repositório que
clonou, um pendrive. **Alt+V, S** (antes o Branch, com `B`, para incluir as
pastas abaixo).

```
      1  Names with control or text-direction characters
      1  Names that deceive (fatura.pdf.exe, spaces, a trailing dot)
      3  setuid, setgid, writable by anyone
      2  Links that leave the folder, or point to nothing
      1  Executables: packed, without PIE or NX, writable and executable
      1  Office: macros, remote template, embedded objects, hidden text
      0  Patches: dangerous paths, hidden characters, hidden changes
      0  Not analysed: over a limit, or the reader could not take it
```

| Categoria | O que ela olha |
|---|---|
| Nomes com caracteres de controle ou de direção do texto | O nome de cada arquivo e pasta: caracteres de controle, os que invertem o texto (um nome desenhado como `exe.pdf` que é `fdp.exe`), bytes que não são UTF-8 |
| Nomes que enganam | A extensão de um documento antes de uma que executa (`fatura.pdf.exe`, `photo.jpg.scr`), espaços antes da extensão, um nome que termina num espaço ou num ponto |
| setuid, setgid, gravável por qualquer um | O modo de cada arquivo, e de cada pasta: uma pasta em que qualquer um pode gravar só conta sem o sticky bit, que o `/tmp` tem |
| Links que saem da pasta, ou apontam para nada | Para onde cada link aponta, pelo seu texto: o link nunca é seguido para ler o que há lá |
| Executáveis | ELF, PE e Mach-O, reconhecidos pelos primeiros bytes e não pelo nome: sem PIE, sem NX ou DEP, sem ASLR, pilha executável, seções empacotadas (UPX e outros) ou ao mesmo tempo graváveis e executáveis, dados depois da última seção |
| Office | Word, PowerPoint e OpenDocument: macros, o que o documento busca fora quando abre (um modelo remoto), objetos embutidos, texto oculto e slides ocultos |
| Patches | `.patch` e `.diff`: arquivos que sobem para fora com `..`, são absolutos, entram em `.git` ou passam a ser executáveis; nomes com caracteres ocultos; hunks cujas contagens não batem; linhas alteradas só no que não se vê |
| Não analisados | Os arquivos que os leitores não conseguiram ler, cada um com o motivo: acima de um limite, um pacote quebrado, um leitor que falhou |

Os nomes e os modos dos arquivos já estão na memória, e aparecem na hora. Os
modos das pastas, os links e o conteúdo pedem o disco, em segundo plano: a
tela vai se enchendo conforme eles chegam, diz quantos faltam, e **Esc** para.

O conteúdo passa pelos mesmos leitores dos relatórios do
[visualizador](viewer.md) (`V`), com os mesmos limites, um arquivo por vez. Um
arquivo acima de um limite, ou que um leitor não conseguiu ler, vai para
**Não analisados** com o motivo: ele nunca some em silêncio. No máximo
`scan_max_files` arquivos (10000) têm o conteúdo lido; passado isso, eles são
listados como não analisados ([configuração](configuration.md)).

| Tecla | O que faz |
|---|---|
| ↑ ↓ | move |
| Enter | os arquivos da categoria, cada um com o motivo de estar ali |
| Enter, num arquivo | de volta à janela de arquivos, que fica só com os arquivos daquela categoria, com o cursor neste |
| Enter, numa pasta | a pasta na árvore |
| `V`, num arquivo | o mostra, com o seu relatório; Esc volta à Varredura |
| `R` | varre de novo |
| Esc | de volta às categorias, depois à lista |

Esc na lista guarda a Varredura: Alt+V, S na mesma lista a abre como estava,
sem ler de novo. `R` lê de novo.

### Onde ela não olha

- **Unidades do Windows e FAT, exFAT, NTFS, SMB, montagens do rclone.** Ali o
  driver dá a todo arquivo o mesmo modo, então os modos não dizem nada, e a
  Varredura não os conta: a categoria diz isso no lugar. No WSL, são o
  `/mnt/c` e as outras unidades do Windows.
- **Dentro de um arquivo compactado** (Alt+F5) os modos dos arquivos são os
  que o arquivo compactado registra, e eles contam: um arquivo setuid num TAR
  é algo que vale saber. Cada membro é extraído com um limite (64 MiB) e lido
  como no disco. Pastas e links não são olhados. Num TAR, um membro muito
  adentro não é extraído, e é listado como não analisado.
- **Numa montagem de nuvem** (rclone) o conteúdo não é lido: cada arquivo
  seria baixado. Os nomes e o resto são.
- **Numa máquina remota** só os nomes e os modos dos arquivos são olhados,
  por enquanto: ler o conteúdo traria cada arquivo pelo SSH.

A Varredura não é um antivírus. Ela aponta o que merece um olhar, e não diz
que o resto é seguro.

## Linha do tempo

Os arquivos da lista num histograma pela data em que foram alterados: quando
uma pasta teve movimento, o que chegou na semana passada, quais fotos são
daquela viagem. Ela não lê nada do disco: só as datas que já estão na memória.

```
 2026-11-18: 312 files, 1.4 GiB

                                    █
                                    █▆
 ▁▁▂▁▁▁▁▁▃▁▁▂▁▁▁▁▁                  ██▁
                                    ▲
 11-01     11-05     11-09     11-13     11-17
```

A escala é escolhida pelas datas: horas, dias, semanas, meses ou anos. Datas
anteriores a 1980 e no futuro vão para uma barra própria em cada ponta,
porque datas erradas ou hostis existem e, se não fosse assim, achatariam o
gráfico. Arquivos sem data são contados abaixo dele.

| Tecla | O que faz |
|---|---|
| ← → PgUp PgDn Home End | move de barra em barra; a linha do topo diz os arquivos e os bytes da barra |
| Shift+← → | escolhe um intervalo de barras |
| `+` `-` | uma escala mais fina ou mais grossa (`+` de dias para horas) |
| `B` | a altura das barras por bytes em vez de arquivos |
| Enter | de volta à janela de arquivos só com os arquivos da barra ou do intervalo |
| Esc | volta, sem mudar nada |

Enter transforma a escolha numa [condição W](sql.md) sobre `modified`, como
`modified >= '2026-11-18 00:00:00' AND modified < '2026-11-19 00:00:00'`,
unida com AND à condição que já estava em vigor. O título da janela de
arquivos a mostra, e **W sem texto** a tira.

## Tabela

A lista abre no [visualizador de tabelas](tables.md), um arquivo por linha,
com as colunas da [condição W](sql.md): `name`, `ext`, `dir`, `size`, `disk`,
`modified`, `age_days`, `mode`, `kind` e `tagged`, e `git` e `dup` quando
algum arquivo da lista tem um status do Git ou está num grupo de
[duplicados](duplicates.md). Os valores são os que o `W` vê, então uma
condição que funciona num funciona no outro.

Ali a lista tem o que as tabelas têm: o filtro SQL (`\` ou `&`), o registro
como ficha (Enter), copiar um registro, marcas (`m`) e o
[Batch](execute.md) (Ctrl+B), uma linha de saída por linha da tabela. Uma
lista de milhões de arquivos abre na hora: a tabela é lida por janelas, como
um arquivo grande.

| Tecla | O que faz |
|---|---|
| `\` ou `&` | o filtro SQL da tabela |
| `s` | ordena pela coluna do cursor; `s` de novo, ao contrário; o filtro continua valendo |
| `W` | de volta à janela de arquivos, com o filtro como a sua condição W, unido com AND à que estava em vigor |
| Esc | volta, sem mudar nada |

A tabela é a lista como estava quando abriu: um arquivo marcado ou alterado
depois disso aparece nela na próxima vez que ela abrir. `type`, que lê cada
arquivo, não é uma coluna aqui; o `W` continua a tê-lo.
