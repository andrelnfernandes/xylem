# Plano: o sentido do Ctrl e do Alt

Origem: conversa de 2026-11-12. **Aprovado** pelo autor em 2026-11-12, com as
recomendações marcadas **(?)** e sem manter Alt+W e Alt+Y: as teclas antigas
saem, com aviso nas notas de versão e no manual. O menu ALT mostrava `usage (W)`, `duplicates (Y)` e
`Compare branch Delete`, e o autor perguntou por quê. A causa: o ALT do Xylem repete
o CTRL (Alt+A, D, R, T, U fazem o mesmo que Ctrl+A, D, R, T, U), e as letras
ocupadas empurram as ferramentas novas para letras sem relação com o nome.

## O que o ZTreeWin faz (validação)

Fonte: a ajuda do ZTreeWin 1.20 (ZTW.HLP, 1997), seções 2.2 a 2.7, e o
`docs/manual/ztreewin.md` (versões novas: Mirror, Hide, Wash, Zlist).

- **Ctrl = os marcados.** Ctrl+A, B, C, D, M, N, R, S, T, U, V: tudo sobre os
  marcados; o alcance é o da janela (a pasta, o disco no Showall, todos no Global).
- **Alt = a versão alternativa do comando**, não uma cópia do Ctrl:
  - na árvore, **o ramo ou o disco**: Alt+P Prune, Alt+G Graft, Alt+M Mirror,
    Alt+R Release, Alt+L Log, Alt+T/U marcar e desmarcar por atributo no disco;
  - na lista, **os marcados com caminhos**: Alt+C e Alt+M recriam as pastas;
  - **versões estendidas**: Alt+E, Alt+O, Alt+X;
  - **ferramentas**: Alt+F File display, Alt+S Sort, Alt+Q sair mudando de pasta,
    Alt+F10 configuração.
- Nenhum Alt do ZTreeWin repete um Ctrl. Alt+D não existe; Alt+A não existe.

## De onde vieram as repetições

Antes da tabela de teclas (commit 2658865), `src/input.rs` tratava "Ctrl ou Alt +
letra" como a versão sobre os marcados, com o Alt só mudando "com caminhos" no
Copy/Move. A tabela herdou isso para A, D, R, T, U. Não há motivo de terminal: as
teclas Ctrl em questão chegam em qualquer terminal (as que não chegam, como Ctrl+M,
já têm saída própria: `F9` `M`).

## A regra

- **Letra sozinha:** o item sob o cursor.
- **Ctrl:** os marcados. Sem mudança.
- **Alt:** a versão alternativa, como no ZTreeWin. Na árvore, o ramo; na lista, os
  marcados com caminhos; e as ferramentas. No menu, tudo em ordem alfabética,
  como nos outros menus (decisão do autor: sem separar grupos).

## Antes e depois

Decisões em aberto marcadas com **(?)**, com a recomendação.

| Tecla | Hoje | ZTreeWin | Proposta |
|---|---|---|---|
| Alt+A | Attributes dos marcados (= Ctrl+A) | — | **sai** |
| Alt+D | Delete dos marcados (= Ctrl+D) | — | **Duplicates** |
| Alt+R | Rename dos marcados (= Ctrl+R) | Release | **Release do ramo** (o mesmo que `-`) |
| Alt+T | Tag tudo (= Ctrl+T) | Tag por atributo | **sai**; T fica reservado para marcar por atributo |
| Alt+U | Untag tudo (= Ctrl+U) | Untag por atributo | **Usage** (marcar por atributo no Linux é raro; Usage com U lê sozinho) |
| Alt+W | Usage | Wash | **sai** (aviso nas notas de versão e no manual) |
| Alt+Y | Duplicates | — | **sai** (idem) |
| Alt+G | — | Graft | **Graft** (passo seguinte: mover o ramo) |
| Alt+M (árvore) | — | Mirror | **reservado** para o Mirror (plano próprio) |
| Ctrl+G | menu Git | Global dos marcados | igual; divergência anotada em `ztreewin.md` |
| Alt+C (árvore) | Compare branch | — | igual; no menu, só **Compare** (o Alt já diz "o ramo") |
| Alt+C, Alt+M (lista) | Copy/Move com caminhos | igual | **igual** (é do ZTreeWin) |
| Alt+P | Prune | igual | igual |
| Alt+K | Kbyte / percentual | — | igual |
| Alt+F, Alt+S, Alt+Q, Alt+F10, Alt+F5 | ferramentas | iguais | iguais, no grupo de ferramentas |
| Alt+B, Alt+H, Alt+I | Bookmarks, Help, Info | outros sentidos | iguais, no grupo de ferramentas (divergências já registradas) |
| Alt+X, Alt+Z, Alt+O | terminal, tela cheia, marcados no terminal | eXecute/Open estendidos | iguais (são o "eXecute estendido" do Xylem) |
| Ctrl+… | os marcados | os marcados | **sem mudança** |

## Os menus resultantes (120 colunas)

```
ALT      Bookmarks Compare Duplicates File display Help Info Kbyte Prune Release Sort Usage
         X terminal   O type tagged files in terminal   F9 next menu

ALT      Bookmarks Copy Duplicates File display Help Info Move Sort Usage
         X terminal   O type tagged files in terminal   F5 open archive   Copy/Move: with paths   F9 next menu
```

Sem parênteses nas letras: cada palavra tem a sua. O terminal fica no X (Alt+T é o
menu "Terminal" de alguns emuladores, que não deixam a tecla chegar ao Xylem); o
Alt+Z (tela cheia) aparece só no menu do próprio terminal.

## O que muda além do código

- `src/keymap.rs`: as linhas `alt+a`, `alt+d`, `alt+r`, `alt+t`, `alt+u`; o
  comentário que diz que o Alt também age sobre os marcados.
- `src/ui.rs`: as listas do ALT,
  as dicas da segunda linha; `ARCHIVE_WRITES`.
- Quem tem `[keys]` próprio no `config.toml` não é afetado; quem usava Alt+D para
  apagar os marcados passa a abrir Duplicates (que não apaga nada sem confirmação).
  Uma nota nas notas de versão.
- Manual, nos seis idiomas: `keys.md`, `ztreewin.md` (a tabela "Different keys"),
  `usage.md`, `duplicates.md`, `rename.md`, `attributes.md`, `tagging.md`,
  `delete.md`, `index.md`; a ajuda (F1) sai da tabela de teclas; README e site.
- Testes: as teclas novas e as que saíram; o menu ALT sem parênteses; o separador.
- As capturas com o menu ALT (`demo-menus-nord.gif`).

## Fora deste passo

- **Graft (Alt+G):** mover a pasta atual com tudo abaixo dela para outro lugar da
  árvore. Passo seguinte.
- **Mirror (Alt+M na árvore):** espelhar o ramo num destino, de mão única (apaga no
  destino o que sumiu da origem) ou de duas mãos. Apaga arquivos: plano próprio, com
  prévia do que será copiado e apagado antes de confirmar.
- Completar o menu CTRL com o que já funciona e não aparece: Batch (B), New date (N),
  Search (S), View (V) na lista; Oops (Z).
- O menu vertical para o que não cabe (`docs/plans/menus-narrow.md`).
