# Renomear muitos arquivos com um contador

Este procedimento renomeia as nove fotos em `photos/2025/trip` de `DSC_01.jpg`
… `DSC_09.jpg` para `trip_001.jpg` … `trip_009.jpg`.

## Passo a passo

1. Na árvore, posicione o cursor sobre `trip` e pressione Enter para ir aos
   seus arquivos.
2. Pressione Ctrl+T. Todos os arquivos da lista são marcados.
3. Pressione Ctrl+R. O prompt solicita a máscara e abre sugerindo `*.*`.
4. Pressione Ctrl+U para limpar o prompt, digite `trip_<001>.*` e pressione
   Enter.

![Os quatro passos: os arquivos de trip, Ctrl+T, Ctrl+R e a máscara trip_<001>.* resultando em trip_001.jpg a trip_009.jpg](../../screenshots/howto-rename-mask.gif)

O rodapé exibe `Renamed 9 of 9 files`. Os arquivos são numerados na ordem em
que aparecem na lista; ordene a lista antes (Alt+S) se a ordem for relevante.

## A máscara

| Parte | Significado |
|---|---|
| `trip_` | texto copiado para cada nome |
| `<001>` | um contador com três dígitos: 001, 002, … |
| `.*` | a extensão de cada arquivo, inalterada |

O passo 4 limpa o prompt porque a digitação se soma ao `*.*` existente:
`*.*trip_<001>.*` nomearia o primeiro arquivo como `DSC_01.DSC_01trip_001.jpg`.

## Outras máscaras

| Máscara | `DSC_01.jpg` torna-se |
|---|---|
| `*.jpeg` | `DSC_01.jpeg` |
| `2025_*.*` | `2025_DSC_01.jpg` |
| `s/DSC/trip/` | `trip_01.jpg` |
| `*.<upper>` | `DSC_01.JPG` |

[Renomear](rename.md) descreve todas as opções suportadas.

## Quando não se comporta como esperado

- **Nenhum arquivo é renomeado.** A máscara geraria o mesmo nome para dois ou
  mais arquivos. Sem contador e sem `*`, todos receberiam o mesmo nome.
- **Apenas alguns arquivos foram renomeados.** Apenas arquivos marcados são
  renomeados. Um Filespec pode ocultar arquivos da lista, e Ctrl+T marca apenas
  o que a lista exibe.
- **Ctrl+R renomeia apenas um arquivo.** Foi pressionado apenas `R`, que renomeia
  o arquivo sob o cursor.

Veja também: [Marcações](tagging.md), [O menu Ctrl](commands-ctrl.md).
