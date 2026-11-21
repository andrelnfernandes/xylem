# Plano: o status do Git lido em segundo plano

**Correção para a 0.9.0** (pedida em 2026-11-21; o bug está no `BACKLOG.md`,
"Git status freezes the screen on slow file systems").

## O problema

Com o Git ligado, `StatusCache::refresh` (`src/git.rs`) roda `git status` na
linha de execução que desenha a tela e lê as teclas: antes de cada desenho, de
novo a cada 10 s (`MAX_AGE`) e 500 ms depois de uma mudança
(`MIN_INTERVAL`). No `/mnt/c` do WSL (9p), um repositório de 7.621 arquivos
levou 130 s por leitura: o Xylem respondia por um instante e travava de novo.

## A correção

1. **A leitura vai para uma thread.** `refresh` não lê mais: começa a leitura
   (`read_status`, com o `has_own_filters` dela) numa thread e volta na hora.
   Cada `refresh` seguinte recolhe as leituras que terminaram (por um canal,
   sem esperar). Enquanto a primeira leitura não chega, a lista fica sem as
   letras; numa releitura, ficam as letras da leitura anterior.
2. **Nunca duas leituras do mesmo repositório ao mesmo tempo.** Uma releitura
   pedida durante uma leitura espera ela terminar (a marca de "desatualizado"
   fica, e a próxima começa depois).
3. **Um repositório que saiu da tela** tem a leitura em curso esquecida: o
   resultado, quando chegar, é jogado fora. O `git` em si termina sozinho
   (matar o processo pede o `Child` fora do `output()`; fica para depois, se
   fizer falta).
4. **Um repositório lento é relido menos.** A releitura periódica (a que pega
   commits feitos em outro terminal) espera `MAX_AGE` ou dez vezes o que a
   última leitura levou, o que for maior: uma leitura de 130 s só se repete
   depois de uns 20 minutos. F3, uma operação do Xylem, o Ctrl+G e o terminal
   embutido continuam pedindo uma releitura logo (`invalidate`).
5. **O título da janela de arquivos diz `git status...`** quando a leitura do
   repositório mostrado já passa de 1 s, como diz `reading...` numa pasta da
   nuvem.
6. **`repo_root`** (achar o `.git` subindo as pastas) continua na tela: são
   alguns `stat`, guardados em `roots`, e um diretório numa montagem lenta da
   nuvem já fica de fora (`reads_slowly`).

## Testes

- Um `git` falso e lento (um script no `PATH` do teste, que dorme antes de
  responder), só em Unix: `refresh` volta em menos de 100 ms; as letras
  aparecem quando a leitura termina; uma segunda `refresh` durante a leitura
  não começa outra (o script conta quantas vezes rodou).
- A releitura de um repositório lento espera; `invalidate` faz reler.
- Um repositório que saiu da tela não recebe o resultado atrasado.
- Os testes que já existem (as letras antes e depois do Add, a nuvem, os
  repositórios hostis) passam a esperar a leitura terminar.

## Manual

Em `git.md`, o parágrafo sobre desligar o Git deixa de dizer que a tela para:
diz que a leitura é em segundo plano, que um repositório lento é relido
menos, e que desligar o Git evita o custo de qualquer forma.

## Feito (2026-11-21)

Como acima, com estes detalhes:

- **O leitor é trocável** (`StatusCache.read`): os testes usam um que dorme
  300 ms e conta as leituras, em vez de um `git` falso no `PATH` (que
  mexeria no ambiente dos outros testes, em paralelo, e só valeria em Unix).
- **Uma leitura que morre** (a thread caiu) deixa o repositório sem letras
  e não recomeça na hora.
- **O `W` com `git`** espera até 2 s pelo status de que precisa; se ainda
  não chegou, diz `The Git status is still being read: try again in a
  moment` e não muda a lista.
