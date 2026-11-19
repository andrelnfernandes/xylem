# Plano: patch e diff no viewer

Origem: conversa de 2026-11-16 (veja [viewer-logs.md](viewer-logs.md)).
**Implementado em 2026-11-16 no branch `viewers-next`, ainda sem revisão do autor** (veja "Implementação", no fim).

## Objetivo

Um patch abre como hoje, em texto, e ganha o que se faz ao revisar um patch:
o resumo (quantos arquivos e linhas), a lista dos arquivos, e pular de um
arquivo e de um trecho (hunk) para o próximo. Também destaca o que um patch
pode fazer além de mudar linhas, e que passa despercebido numa leitura rápida:
tornar um arquivo executável, criar um link simbólico, mexer fora da pasta.

Hoje um `.patch` e um `.diff` já têm a cor do `syntect` (a mesma que a saída do
`D`, o Git Diff, usa por `git.diff`). Isso continua.

## O que conta como patch

- Pelo nome: `.patch` e `.diff`, também comprimidos (`.patch.gz`).
- Pelo conteúdo: um `diff --git`, ou um `--- ` seguido de `+++ ` e `@@`, nas
  primeiras 200 linhas.
- A saída do `D` (Git Diff) dentro do Xylem ganha o mesmo, porque é o mesmo
  formato.
- Também o `git format-patch` (com o cabeçalho de e-mail `From <sha>`,
  `Subject:`) e uma série de patches num arquivo só: cada mensagem vira uma
  parte da lista, com o assunto.

## Modo e teclas

Continua o modo texto, com o que já existe (busca, `&`, `g`, Ctrl+B, janelas).
Novas, só quando o arquivo é patch:

- **`]` e `[`:** próximo e anterior arquivo. Hoje só trocam de folha na tabela, e
  no texto não fazem nada.
- **`}` e `{`:** próximo e anterior hunk.
- **Tab:** a lista dos arquivos, como a das folhas da tabela, com `+n −m` de
  cada um e a marca do que é novo, apagado, renomeado ou binário. Enter vai ao
  arquivo.
- O rodapé diz em que arquivo e hunk a tela está (`3/12 src/viewer.rs @@ 2/5`).
- O cabeçalho mostra o resumo: `12 files, +340 −98`.

## O que fica em destaque

Na lista e na própria linha do patch:

- **Modo do arquivo:** `new mode 100755` (passa a ser executável) e
  `old mode 100755` / `new mode 100644`.
- **Link simbólico** (`120000`): aparece com o alvo do link. Um link num patch
  pode apontar para fora do repositório, e foi assim no CVE-2023-23946 do
  `git apply`.
- **Caminho perigoso:** absoluto, com `..`, com `.git/` em qualquer ponto, ou
  com caracteres de controle. O `.git/hooks/` em especial, porque um hook é
  código que o Git executa.
- **Submódulo** (`Subproject commit …`): o commit antigo e o novo.
- **Patch binário** (`GIT binary patch`, `Binary files … differ`): o bloco em
  base85 aparece numa linha só, `[binary patch, 12 KiB]`, que se abre com Enter
  sobre ela.
- **Espaço em branco:** espaço no fim de linha adicionada e mistura de tab com
  espaço no recuo ficam marcados, como o `git diff --check`.
- **Mudança invisível:** um par de linhas, uma removida e uma adicionada, que só
  difere em espaço, em fim de linha (`\r`) ou em caracteres que se parecem
  (homóglifos, espaço sem quebra, ZWSP) fica marcado `looks unchanged`. É o
  jeito de esconder uma mudança na revisão. O bidi já é marcado em todo o
  viewer (Trojan Source).

## Como o índice é montado

- Um passe pelas linhas guarda onde começa cada arquivo e cada hunk e conta os
  `+` e `−`. É linear e não usa regex na linha inteira: só o começo de cada
  linha é olhado.
- Num arquivo em janela, o passe roda junto com a contagem de linhas, em
  segundo plano. Enquanto não termina, `]`, `}` e Tab andam até onde o passe
  chegou, e o rodapé diz que o resto ainda está sendo lido.
- Os números do cabeçalho do hunk (`@@ -a,b +c,d @@`) são conferidos contra as
  linhas do hunk. Um hunk que mente (diz 3 linhas e tem 300) fica marcado, e o
  índice segue as linhas reais, nunca os números.

## Segurança

- Nada é aplicado. O viewer nunca chama `git apply` nem `patch`.
- Os números do `@@` não reservam memória nem orientam a leitura: só são
  comparados com o que se contou.
- O base85 de um patch binário não é decodificado. Só o tamanho declarado é
  mostrado, conferido com o tamanho do bloco.
- O nome de arquivo com aspas do Git (`"a/\303\251.txt"`) é decodificado e passa
  pela limpeza do texto, como qualquer nome.
- Tetos: 1 milhão de arquivos e 10 milhões de hunks no índice. Acima disso,
  corta e diz.
- proptest: linhas aleatórias de patch (cabeçalhos quebrados, `@@` com números
  extremos, nomes com aspas inválidas) nunca dão pânico, e o índice nunca aponta
  fora do arquivo.

## Etapas (um PR cada)

1. **Índice, resumo, `]` `[` `}` `{`, rodapé e Tab.**
2. **Destaques:** modo, link, caminho, submódulo, binário e espaço.
3. **Mudança invisível** e a série de `format-patch`.

## Questões abertas

| # | Questão | Proposta |
|---|---|---|
| ?1 | Teclas para arquivo e hunk | `]` `[` para arquivo, `}` `{` para hunk |
| ?2 | A saída do `D` ganha o mesmo | sim, é o mesmo formato |
| ?3 | Lado a lado (antes e depois em duas colunas)? | para depois |

## Para depois

- Lado a lado, numa tela larga.
- Abrir o arquivo do patch, na linha do hunk, com o editor (`E`).
- Patch de outros sistemas (`svn diff`, `hg diff`), que são quase o mesmo formato.

## Implementação (2026-11-16)

Feito (commit `a7625fc`), em `src/viewer/patch.rs`, `src/viewer.rs` e `src/ui.rs`:
as etapas 1, 2 e 3, com as teclas propostas (?1) e a saída do `D` igual (?2). O
lado a lado (?3) ficou para depois, como proposto.

Diferente do spec:

- **num arquivo em janela (acima de 16 MiB), não há índice:** as cores ficam
  (são feitas linha a linha), mas `]`, `}` e Tab dizem `No file list in a patch
  this big`. O passe junto com a contagem de linhas fica para depois;
- **espaço em branco** só é contado no cabeçalho (`n whitespace`), sem marca na
  linha: a cor da linha é só a do texto, e um espaço não tem cor que se veja;
- o bloco de um patch binário não é recolhido numa linha: aparece como está,
  com a cor de aviso.
