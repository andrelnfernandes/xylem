# Configuração

## A tela de configuração (F12)

**F12** (ou Alt+F10; no GNOME, Alt+F10 geralmente maximiza a
janela e nunca chega ao Xylem) abre cinco páginas:

| Página | Opções |
|---|---|
| 1. Programas e formatos | editor, abrir editor dentro da moldura do Xylem, tema do Xylem no editor, editar arquivos binários com E, editor binário, idioma da ajuda, programa de abertura (`opener`), formato de data, formato numérico, comandos Git, pastas Git confiáveis |
| 2. Exibição e ordenação | arquivos ocultos, ordem e direção iniciais de classificação, o que Enter faz na janela de arquivos, modo de exibição inicial, cores de sintaxe, quebra de linha, números de linha, arquivos comprimidos, a codificação de textos antigos, o limite de tamanho do que o visualizador lê, ícones e glifos (Nerd Fonts), nomes de arquivo coloridos por tipo, bordas das janelas, cores do terminal integrado, segundos no relógio, [o mouse](mouse.md) |
| 3. Confirmações e histórico | confirmar ao sair, excluir para a lixeira, confirmação do Prune, entradas com `@` não salvas, histórico próprio do terminal integrado, [Alt+T na busca aproximada marca todas as correspondências](#marcar-todas-as-correspondências-da-busca-aproximada), [procurar uma versão nova](#versões-novas) |
| 4. Copiar, comparar e atualizar | ignorar 2 s, ignorar 1 h, atualização automática, onde começam os caminhos de Alt+C/Alt+M, arquivos e pastas deixados fora do índice |
| 5. Leitura (ebooks) | a largura da coluna de texto, parágrafos como num livro ou em blocos, texto justificado, a velocidade de leitura para o tempo que falta, [o tempo gasto lendo](#tempo-de-leitura), os minutos sem tecla até o relógio parar, os títulos guardados para uma lista dos livros |

O limite de tamanho do visualizador (**L** na página 2) custa memória e tempo: com `1024` MiB, abrir um arquivo de texto comum desse tamanho ocupa cerca de 2,5 GiB de RAM e leva uns 5 s, e arquivos de linhas muito curtas custam mais por byte (veja [o visualizador](viewer.md)).

Ícones e glifos (**M** na página 2) exigem uma fonte com o patch do Nerd Fonts. Não é necessária uma fonte específica: qualquer fonte monoespaçada funciona (como Fira Code, JetBrains Mono, Hack, Meslo ou Cascadia Code), desde que seja a sua versão "Nerd Font". O Nerd Fonts injeta milhares de glifos padronizados na área privada do Unicode. Fontes comuns sem o patch (mesmo a Fira Code original) não contêm esses desenhos e exibirão retângulos vazios (`□`). Para utilizá-los:
- Baixe qualquer Nerd Font de sua preferência em nerdfonts.com ou instale um pacote Nerd Fonts pelo gerenciador de pacotes da sua distribuição.
- Selecione a fonte instalada nas preferências do seu emulador de terminal.

Se as colunas se desalinharem com os ícones ligados, o terminal provavelmente desenha os caracteres de "largura ambígua" como largos (ajuste comum com fontes chinesas, japonesas e coreanas), enquanto o Xylem conta cada ícone como uma célula. Ajuste essa opção para estreito ou desligue os ícones.

Nomes coloridos por tipo (**N** na página 2) dão a compactados, imagens, áudio e vídeo, documentos, código e dados uma cor do tema cada, só pelo nome (nada é lido). Desligado, a lista fica com o visual clássico. As bordas das janelas (**O**) são arredondadas nos temas nord, gruvbox, slate, graphite, daylight e vivid, e retas nos outros; `plain` ou `rounded` as fixa em todos os temas. A tela de abertura (**P**) mostra o nome em letras de bloco por dois segundos quando o Xylem abre; qualquer tecla a fecha (essa tecla não faz mais nada), e a pasta inicial é lida enquanto isso. Não aparece ao conectar numa máquina remota ou container pela linha de comando, nem num terminal menor que 58 por 13.

A letra da opção (ou Enter) altera o valor, PgUp/PgDn ou 1–5 trocam de página,
**F4** escolhe o tema em uma lista (com pré-visualização instantânea), **F3**
salva e aplica, **Esc** cancela (pergunta se houver alterações).

### Marcar todas as correspondências da busca aproximada

Na busca aproximada (`|`), **Alt+T** marca os resultados da lista. A lista
guarda as 500 melhores correspondências, e uma busca pode corresponder a
muito mais: `f` numa árvore grande pode corresponder a cem mil arquivos. Por
padrão, Alt+T marca só as 500 da lista e, quando mais corresponderam, a linha
abaixo da lista diz isso e cita esta opção.

Com **Alt+T na busca aproximada marca todas as correspondências** ligada
(**F** na página 3, `fuzzy_tag_all = true` no arquivo), Alt+T marca todas as
correspondências, inclusive as que você nunca viu. A linha abaixo da lista
então sempre diz quantas correspondências não estavam nela.

Ela vem desligada por padrão, porque as marcas valem para o que você fizer em
seguida. Ctrl+C, Ctrl+M, Ctrl+D e o Batch agem sobre todos os arquivos
marcados, não só sobre os que você viu. Milhares de arquivos marcados por uma
busca que você só olhou de relance, e depois excluídos, são milhares de
arquivos perdidos. Ligue a opção só se você confere o que está marcado antes
de agir:

- o painel de estatísticas mostra quantos arquivos estão marcados e o tamanho
  deles;
- **Ctrl+S** na árvore lista os arquivos marcados da árvore inteira (um
  Showall dos marcados);
- **Ctrl+V** na janela de arquivos os mostra um depois do outro.

### Versões novas

Na primeira vez que o Xylem abre, depois que a tela de abertura some, ele
pergunta: *Look for a new version of Xylem once a day?* (procurar uma versão
nova do Xylem uma vez por dia?). **Y** diz sim, **N** não, e a resposta fica
guardada no arquivo de configuração (`check_updates`). **Esc** não guarda
nada, e a pergunta volta na próxima vez. **G** na página 3 muda a resposta
depois.

Com ela ligada, uma vez por dia o Xylem pede ao site
(`https://andrelnfernandes.github.io/xylem/latest.txt`) uma linha: o número
da versão mais recente. Ele faz isso pelo `curl` do sistema, em segundo
plano, só com HTTPS e por cinco segundos no máximo; sem rede, ou sem `curl`,
nada é dito e ele tenta de novo no dia seguinte. O pedido não leva mais que
`xylem/` e a versão que você usa; o site vê o seu endereço IP, como em
qualquer visita. Nada é baixado ou instalado: quando há uma versão mais nova,
a linha do topo diz `0.10.0 is out` ao lado do relógio, e uma mensagem dá o
endereço das versões. Atualize como você instalou: pelo gerenciador de
pacotes para um `.deb` ou `.rpm`, por um arquivo compactado novo nos outros
casos.

A resposta do site é lida como hostil: qualquer coisa que não sejam três
números, como a página do Wi-Fi de um hotel, é ignorada. A data da última
verificação e a versão encontrada ficam guardadas na pasta de estado
(`~/.local/state/xylem/update-check`).

`XYLEM_CHECK_UPDATES=0` (ou `1`) no ambiente responde sem perguntar e vale
mais que o arquivo, para instalações em muitas máquinas. Um build sem a
feature `update-check` (`cargo build --no-default-features --features
index`), como uma distribuição pode fazer, não tem a pergunta, nem a
verificação, nem a opção.

### Tempo de leitura

O visualizador lê [ebooks](viewer.md#ebooks) EPUB e FB2. A página 5 define
como o texto é disposto, e se o Xylem guarda o tempo que você passa lendo.

**Guardar o tempo gasto lendo** (**E**, `reading_stats = true` no arquivo)
vem desligado por padrão. Ligado, o Xylem guarda, para cada livro lembrado
com `R` no visualizador:

- o tempo lido, e as sessões: uma pausa de mais de 30 minutos começa uma
  nova;
- o ponto mais distante alcançado, o primeiro e o último dia de leitura, e se
  o livro foi terminado;
- a velocidade de leitura, em palavras por minuto. Só contam as páginas lidas
  para a frente, e só quando ficaram pelo menos 5 segundos na tela, então
  saltos, voltas e leitura por cima não contam. Depois de 10 minutos medidos,
  o rodapé conta o tempo que falta com a velocidade daquele livro em vez da
  desta página (**D**).

O relógio só corre enquanto você lê: de uma tecla à seguinte, se não passaram
mais de **F** minutos (3 por padrão) entre elas. Uma pausa mais longa o para,
e essa pausa não conta. O Autoview (F7) e o relatório (`I`) nunca contam.

Tudo fica no seu computador, em `~/.local/state/xylem/reading.toml` (ou
`$XDG_STATE_HOME/xylem/`), legível só por você. Um livro é guardado como um
hash do que ele diz ser e do seu tamanho, e números: não o nome, não onde
está, não o título. Com **G** ligado (`reading_titles = true`), o título e o
autor também são guardados, e o relatório de qualquer livro então lista
todos os livros lidos, em "Your books".

Desligar o **G** e salvar (F3) apaga na hora os títulos guardados. Desligar o
**E** e salvar pergunta se o tempo lido de todos os livros deve ser apagado;
sem isso, ele fica no arquivo e não é mostrado. Nos dois casos, os lugares
lembrados com `R` ficam; `R` num livro o esquece.

## O arquivo de configuração

Tudo fica armazenado em `~/.config/xylem/config.toml` (ou
`$XDG_CONFIG_HOME/xylem/`; `%APPDATA%\xylem\` no Windows), que também pode ser
editado manualmente: a tela preserva seus comentários. Para começar a partir de
um arquivo comentado com todos os padrões:

```bash
mkdir -p ~/.config/xylem && xy --default-config > ~/.config/xylem/config.toml
```

- `theme`: `classic` (branco e amarelo sobre azul), `dark`, `light`, `mono` (as cores do próprio
  terminal), `norton`, `solarized`, `gruvbox`, `nord`, `slate` (no estilo do
  Windows Terminal), `graphite` (macOS no modo escuro), `daylight` (macOS no
  modo claro) ou `vivid` (cores fortes, para apresentações e capturas de tela).
  F4 mostra todos eles, com pré-visualização.
- `[options]`: as opções da tela de configuração, e algumas que só existem no
  arquivo: `scan_max_files` (10000), o máximo de arquivos cujo conteúdo a
  [Varredura](views.md#scan) lê.
- `[colors]`: cores individuais sobrepostas ao tema, como nomes (`"yellow"`),
  `"#rrggbb"` ou índices de 0 a 255.
- `[keys.any]`, `[keys.tree]`, `[keys.files]`: redefine teclas em ambas as
  janelas ou em apenas uma (que tem precedência). Por exemplo, `"f3" = "view"`,
  `"ctrl+r" = "relog"`; `"none"` desativa uma tecla. A lista de teclas da ajuda
  (**K**) mostra as teclas em vigor, com as suas alterações.
- `[editors]`: o editor para cada tipo de arquivo (`"*.png *.jpg" = "gimp %f"`,
  `binary = "hexedit"`); veja [Edit](execute.md).

Um erro no arquivo não impede o Xylem de iniciar: ele usa os padrões, mostra o
erro no rodapé e a tela de configuração não salvará até que o arquivo seja
corrigido, para não apagar o que você escreveu.

O Xylem chamava-se xt até a versão 0.1. Na primeira execução, ele move a
configuração, o histórico e os favoritos do xt para as suas próprias pastas.
