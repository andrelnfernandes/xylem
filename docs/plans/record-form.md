# Plano: o registro em forma de ficha, e copiá-lo

Origem: pedido de 2026-11-14. Estado: implementado na branch `record-form`.

## 1. O que falta

Na tabela, um registro com muitos campos (o `pequeno.dbf` tem 106) não cabe na
largura: lê-se coluna a coluna. Falta ver o registro inteiro de uma vez, um campo por
linha, como o `EDIT`/`DISPLAY` do dBase, e copiá-lo para outra ferramenta (editor,
terminal).

## 2. Teclas

Letras livres no modo tabela: `a`, `j`, `k`, `x`, `z` (o `r` ficou com "ler de novo"
quando o arquivo muda), nenhuma com mnemônico.

- **Enter** na tabela abre a ficha do registro sob o cursor, como nos visualizadores
  de DBF (o DBU do Clipper). Esc, `q` e `v` continuam fechando o viewer; Enter fecha
  nos outros modos.
- Na ficha:
  - ↑ ↓ PgUp PgDn Home End: o campo em destaque.
  - ← →: registro anterior e seguinte (o cursor da grade vai junto; sob filtro, só os
    que ele mantém).
  - `c`: copia o registro inteiro, uma linha por campo, `NOME : valor`, com os nomes
    alinhados.
  - `y`: copia só o valor do campo em destaque.
  - Esc, Enter, `q`: de volta à grade.

## 3. Segurança

- O que se copia é o que a tela mostra: nomes e valores já passaram por
  `TableGrid::sanitize_cell` na leitura (controle, bidi e separadores de linha viram
  `·`, até 2.048 caracteres por célula). Nada do arquivo chega à área de transferência
  com escape ou quebra de linha: colar num terminal não executa nada que o arquivo
  tenha escondido. Uma memo com várias linhas sai numa linha só, com `·`.
- A cópia usa `clipboard::set_clipboard` (OSC 52 em base64, mais as ferramentas do
  sistema), a mesma do `y` que copia o caminho.
- Numa tabela em janela (sem limite de linhas), o registro pode não ter sido lido
  ainda: a ficha diz `Reading the record…` e não copia nada.

## 4. Tela

- Uma janela sobre a grade, título `Record 7 of 9,518`. Os nomes à esquerda, alinhados;
  um valor longo quebra em várias linhas, recuadas sob o valor.
- Rodapé com a ficha aberta: `RECORD ←→ record  ↑↓ field  c copy record  y copy field
  Esc back`, e o aviso da cópia na segunda linha.
- Rodapé da tabela: `Enter record`.

## 5. Testes

- Enter abre e fecha; ← → trocam de registro e movem a grade; ↑↓ movem o campo.
- `c` copia o texto alinhado; `y` copia o valor; um valor hostil (escape, bidi, quebra
  de linha) sai limpo.
- Enter ainda fecha o viewer fora da tabela.
- Desenho: a ficha mostra nomes e valores, quebra um valor longo e mostra o título.
- Manual nas 6 línguas por último.
