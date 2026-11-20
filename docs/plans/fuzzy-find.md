# Plano: busca aproximada ao digitar (`|`), e o zoxide como apêndice

**Proposta para a 0.9.0** (pedida em 2026-11-20). Nada aprovado para código.

## Objetivo

Achar uma pasta ou um arquivo digitando pedaços do nome, em qualquer nível da
árvore: `cfgxy` acha `config/xylem`, `rdme` acha `README.md`. O cursor vai até
o escolhido, e o que se marcar ali fica marcado na lista.

Hoje o Xylem tem três buscas, e nenhuma faz isto:

- o **Treespec** (`/`, `\`) anda pelo caminho, nível por nível, pelo começo do
  nome;
- o **Filespec** escolhe os arquivos da lista por curinga, data e tamanho;
- a **caixa de busca** (`P`) acha arquivos pelas palavras do conteúdo.

A página de comparações lista, entre o que falta do ZTreeWin, a busca ao
digitar com prompt (`|`). Esta é a versão do Xylem dela: a mesma tecla, com
busca aproximada em vez de pelo começo do nome.

Feita dentro do Xylem, sem programa externo: os nomes já estão na memória, a
tela é a do Xylem (tema, mouse, rodapé), os nomes passam pelo mesmo tratamento
de sempre, e a marcação é o `T` de sempre.

## Na tela

- **`|`** abre um prompt sobre a janela atual, com a lista dos resultados
  acima dele, do melhor para o pior, como a caixa de busca (`P`).
- **Na árvore**: as pastas logadas, pelo caminho a partir da raiz.
- **Na janela de arquivos**: os arquivos da lista atual (o diretório, o Branch
  ou o Showall, com o Filespec e o `W` que estiverem valendo), pelo nome; no
  Branch e no Showall, também pelo caminho.
- **↑ ↓** escolhem o resultado; o cursor da árvore ou da lista acompanha,
  como no Treespec.
- **Enter** confirma: o cursor fica no escolhido e o prompt fecha.
- **Esc** volta para onde estava.
- **Ctrl+T no prompt** marca ou desmarca o resultado sob o cursor sem fechar;
  **Alt+T** marca todos os resultados. Depois, Ctrl+C, Ctrl+M e o Batch agem
  nos marcados, como sempre. (Teclas a conferir no `keymap.rs`: no prompt, `T`
  é texto.)
- As letras que casaram aparecem destacadas em cada resultado.
- O rodapé diz quantos resultados há e em quantos nomes a busca procurou.

## Como casa

- **As letras na ordem**, não necessariamente juntas, sem diferença entre
  maiúsculas e minúsculas; com uma maiúscula na busca, passa a diferenciar
  (`Rd` não acha `readme`).
- **Espaços separam termos**: todos têm de casar, em qualquer ordem (`xy docs`
  acha `xylem/docs` e `docs/xylem`).
- **Pontuação**, do que mais conta para o que menos conta:
  - letras seguidas;
  - letra no começo do nome, depois de `/`, `_`, `-`, `.` ou de espaço, ou numa
    troca de minúscula para maiúscula (`fileSpec`);
  - o casamento no nome vale mais que nas pastas acima dele;
  - empatados, ganha o caminho mais curto e, depois, a ordem da lista.
- **Depois, se fizer falta**: `'termo` para exato, `^` para começo, `$` para
  fim, `!` para excluir. Fora da etapa 1.

Nomes que não são UTF-8 casam pela sua forma com substituição (`U+FFFD`), como
são mostrados.

## Desempenho

A árvore pode ter até 2.000.000 de nós.

- **Em segundo plano**: a busca roda numa thread, em blocos, e a lista se
  completa enquanto ela anda; digitar de novo cancela a busca anterior.
- **Só os melhores**: guarda os 500 melhores resultados (um heap), não todos.
- **Busca que só cresce**: quando o texto novo é o anterior com mais letras,
  procura só entre os que já casavam.
- **Metas a medir na etapa 1**: resposta visível em até 50 ms com 100.000
  nomes; com 2.000.000, os primeiros resultados no mesmo tempo e o resto
  chegando.

**Escrever o casamento ou usar uma crate**: decidido na etapa 1, medindo.
Critérios para uma crate: licença compatível com a MIT do Xylem, sem `unsafe`
ou com pouco, e o quanto acrescenta ao binário. O casamento próprio é pequeno
(algumas centenas de linhas); a crate só vale se for claramente mais rápida
com milhões de nomes.

## Onde funciona

Em qualquer árvore logada: local, numa máquina remota (SSH), dentro de um
arquivo compactado e numa montagem do rclone. Nada é lido do disco nem da rede:
só os nomes já na memória.

## Segurança

- **Nada é lido** fora da memória; nenhum programa é executado.
- **Os nomes** aparecem pelo mesmo tratamento do resto do Xylem: controles e
  caracteres de direção do texto marcados, nunca obedecidos.
- **A busca** tem até 256 caracteres; os destaques de letras casadas são
  calculados sobre o nome tratado, para que um nome com caracteres
  invisíveis não desalinhe o destaque.

## Etapas

1. **O casamento**: as regras acima, a pontuação, os testes (casos de
   pontuação, maiúsculas, termos, nomes não UTF-8, nomes com controles), e a
   medição com 100.000 e 2.000.000 nomes; a decisão entre código próprio e
   crate.
2. **O prompt**: `|` na árvore e na janela de arquivos, a lista, o cursor
   acompanhando, Enter e Esc, em segundo plano.
3. **Marcar**: Ctrl+T e Alt+T no prompt.
4. A página do manual (em inglês), ligada no índice, em `keys` e no Treespec;
   as notas da versão; e a linha da busca ao digitar na página de comparações,
   revista (o `|` passa a existir, com busca aproximada).

## Fora deste plano

- O "modo ativo" (letras que buscam sem prompt): no Xylem as letras são
  comandos, e Shift+letra já leva ao próximo nome que começa com ela.
- A sintaxe estendida (`'`, `^`, `$`, `!`) na etapa 1.
- Buscar no conteúdo: é o `P`.

## Apêndice: zoxide (opcional)

Para quem usa o zoxide no shell: o valor dele é o banco que aprendeu com o uso,
e esse banco não se recria dentro do Xylem. Por isso, aqui sim, o Xylem chama o
programa. Custa pouco e é só para quem já o tem instalado.

- **Tecla** (proposta): `Z`, livre hoje. Abre uma lista no estilo dos
  favoritos com o resultado de `zoxide query --list --score`: a pontuação e o
  caminho, filtrando enquanto se digita. Enter loga a pasta e vai até ela.
  Sem o zoxide instalado, `Z` diz que ele falta.
- **Alimentar o zoxide** (desligado por padrão): com `zoxide_add = true`, cada
  pasta em que o usuário entra pelo Xylem é registrada com `zoxide add`. Só
  pastas locais: nunca remotas, dentro de arquivos compactados ou de montagens
  do rclone, porque o banco é do shell desta máquina.
- **Segurança**: os argumentos vão para o `zoxide` como argumentos, nunca por um
  shell. Os caminhos que ele devolve passam pelas mesmas checagens do Log
  (existe, é diretório), são mostrados tratados como qualquer nome, e a saída é
  lida com limite de tamanho.
- **Onde**: só na máquina local. Numa sessão remota, `Z` diz que a lista é da
  máquina local.
- **Quando**: depois da etapa 2 da busca, se ainda fizer sentido.
