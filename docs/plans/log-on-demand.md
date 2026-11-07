# Log sob demanda na árvore

Aprovado em 2026-11-07.

## Problema

Hoje uma pasta "fechada" sempre mostra os próprios arquivos, porque o Xylem lê
pastas sem que o usuário peça:

- `+` (e →) lê a pasta e também cada subpasta dela, só para saber quais têm o
  `+` no conector (`├─+`). As subpastas ficam lidas, mas recolhidas: os
  arquivos aparecem e as subpastas delas não.
- Mover o cursor até uma pasta não lida faz o Xylem lê-la na hora
  (`Tree::touch`, chamado em `Pane::select_row`).
- `-` descarta o que foi lido abaixo da pasta e lê a própria pasta de novo:
  os arquivos dela continuam aparecendo.

## Comportamento novo

Não logada quer dizer não lida do disco.

- **Pasta não logada:** a janela de arquivos mostra `Not logged`, sem nenhum
  arquivo. Na árvore, o conector ganha uma marca própria (`├─?`, `└─?`), já que
  não se sabe se ela tem subpastas.
- **Mover o cursor** não lê nada.
- **`+` (e →):** loga a pasta, se ainda não foi, e a expande. Os arquivos
  aparecem, e as subpastas aparecem como não logadas.
- **`*`:** loga e expande o ramo todo (sem mudança).
- **`-`:** descarta o ramo inteiro, incluindo a própria pasta, que volta a não
  logada e recolhida. As marcas perdidas são contadas como hoje.
- **←** não muda: uma pasta logada e recolhida continua mostrando os arquivos.
- **Abertura e `L`:** a raiz é logada e expandida (como um `+` nela); as
  subpastas dela ficam não logadas.
- **Ir a uma pasta pelo caminho** (Treespec, voltar de Showall/Branch, ir a
  partir de Usage ou Duplicados): a pasta de destino é logada, como já é hoje
  (`Pane::show_node`).
- **Nuvem (rclone):** `+` pede a leitura em segundo plano; as subpastas não são
  mais pedidas.
- **Árvores virtuais** (arquivo compactado aberto como pasta): já vêm lidas
  inteiras; `-` continua só recolhendo.

## Roteiro

1. `src/model/tree.rs`:
   - `expand` sem ler as subpastas;
   - `apply_reads` sem pedir as subpastas;
   - `release` sem reler a pasta;
   - sem `touch`;
   - conector `?` para pasta não logada.
2. `src/pane.rs`: `select_row` sem ler a pasta.
3. `src/ui.rs`: `Not logged` na janela de arquivos, com a visão de diretório
   numa pasta não logada, fora de uma leitura em andamento e sem erro.
4. Testes: novos para cada item acima; os que contavam com a leitura
   automática passam a logar a pasta antes.
5. Manual: `docs/manual/tree.md` e as traduções (pt, de, es, fr, zh).

## Fora do escopo

- F3 numa pasta não logada continua sem efeito.
