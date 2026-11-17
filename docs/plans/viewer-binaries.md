# Plano: executáveis no viewer (ELF, PE e Mach-O)

Origem: conversa de 2026-11-16 (veja [viewer-logs.md](viewer-logs.md)).
**Spec proposto, ainda não aprovado.**

## Objetivo

Um executável, uma biblioteca ou um objeto abre como um relatório: o que é, para
que máquina, o que importa e exporta, as proteções com que foi compilado, e
os sinais de que esconde algo (empacotado, dados no fim, assinatura). As listas
longas (seções, símbolos, importações) abrem como tabela, com busca, SQL e
Batch.

Hoje esses arquivos abrem em hex. O hex continua a um `H` de distância.

## Formatos

- Reconhecidos **pelo conteúdo**, nunca pela extensão (um `.txt` pode ser ELF, e
  um `.exe` pode não ser):
  - **ELF** (`\x7fELF`): executável, `.so`, `.o` e core dump (só o resumo);
  - **PE/COFF** (`MZ` com `PE\0\0` no `e_lfanew`): `.exe`, `.dll`, `.sys` e
    `.efi`, e também .NET (com o cabeçalho CLR);
  - **Mach-O** (`feedface`, `feedfacf` e as formas invertidas) e **fat/universal**
    (`cafebabe`). O `cafebabe` também é o começo de um `.class` do Java: só é
    fat quando o número de arquiteturas é pequeno (menos de 20), como o
    `file` faz.
- Fora, por agora: `.a` (ar), WebAssembly, `.class` e `.dex`.

## Leitor

- O crate **`object`** (do grupo gimli-rs), só com as features de leitura
  (`read_core`, `elf`, `pe`, `coff`, `macho`), sem as de escrita. É o
  leitor usado pelo `backtrace` e testado com fuzzing contínuo.
- Dependência nova, que precisa de revisão (princípio 7 de
  [viewers.md](viewers.md)): o `object` tem `unsafe` para ler estruturas
  alinhadas (`Pod`). A revisão confere que todo acesso passa por `ReadRef`, que
  checa o tamanho.
- **O arquivo não é lido inteiro:** um `ReadRef` próprio sobre o `File` lê cada
  faixa pedida, confere `offset + size` contra o tamanho real do arquivo
  **antes** de reservar memória, e soma o que já leu contra um teto (256 MiB no
  `V`, 4 MiB no Autoview). É um `ReadRef` próprio, e não o `ReadCache` do
  `object`, para que a conferência e o teto fiquem no código do Xylem e sejam
  testados aqui.

## O relatório

Montado pelo `markdown::Builder` com eventos, como o de
[viewer-office.md](viewer-office.md), com título, listas e tabelas curtas.

1. **Resumo:** formato, tipo (executável, biblioteca, objeto, core), arquitetura,
   32 ou 64 bits, ordem dos bytes, ponto de entrada, sistema ou subsistema (PE),
   interpretador (ELF), versão mínima do SO (Mach-O) e, num fat, a lista das
   arquiteturas.
2. **Proteções**, uma linha cada, `sim`/`não`/`parcial`:
   - ELF: PIE, NX (`PT_GNU_STACK`), RELRO (`PT_GNU_RELRO` e `BIND_NOW`, para
     total ou parcial), canário (`__stack_chk_fail` importado), FORTIFY
     (símbolos `*_chk`), `RPATH`/`RUNPATH` (em destaque se relativo ou
     gravável, como `$ORIGIN/..` ou `.`);
   - PE: ASLR (`DYNAMIC_BASE`), ASLR de 64 bits (`HIGH_ENTROPY_VA`), DEP
     (`NX_COMPAT`), CFG (`GUARD_CF`), SafeSEH (32 bits), `FORCE_INTEGRITY`;
   - Mach-O: PIE (`MH_PIE`), pilha não executável, `__RESTRICT`.
3. **Assinatura:** só se existe e de que tamanho (diretório de segurança do PE,
   `LC_CODE_SIGNATURE` do Mach-O). **Não é verificada**, e o relatório diz isso,
   para não passar a ideia de que uma assinatura vista é válida.
4. **Sinais:**
   - **Entropia** de cada seção, com destaque acima de 7,2 bits por byte;
     nomes como `UPX0` e `.aspack` dizem qual empacotador;
   - **dados no fim (overlay):** bytes além da última seção ou segmento, com a
     posição e o tamanho (é onde instaladores e alguns malwares guardam carga);
   - seção ao mesmo tempo gravável e executável;
   - ponto de entrada fora de qualquer seção executável.
5. **Identidade:** build-id (ELF), `.gnu_debuglink`, data de compilação do PE
   (avisando que pode ser falsa), GUID e caminho do PDB (CodeView), UUID do
   Mach-O, compilador (`.comment` do ELF), versão do Go (`.go.buildinfo`) e do
   Rust (`.comment` ou o `rustc` em `.rodata`) quando der para achar sem
   procurar no arquivo inteiro.
6. **Bibliotecas** de que depende (`DT_NEEDED`, a tabela de importação do PE e
   `LC_LOAD_DYLIB`), com a contagem de símbolos importados de cada uma.

## As tabelas

As listas longas são planilhas do modo tabela, uma por folha (`[` e `]` trocam,
Tab lista), com o filtro SQL e o Ctrl+B que a tabela já tem:

| Folha | Colunas |
|---|---|
| Sections | nome, endereço, tamanho em memória, tamanho no arquivo, posição, permissões (`rwx`), entropia |
| Segments | tipo, endereço, tamanhos, permissões (ELF e Mach-O) |
| Imports | biblioteca, símbolo, ordinal (PE) |
| Exports | símbolo, endereço, ordinal (PE), encaminhado para (PE) |
| Symbols | nome, endereço, tamanho, tipo, escopo, seção |

- O nome do símbolo aparece como está no arquivo. Demangling (Rust e C++) fica
  para depois.
- `H` faz Relatório → Tabela → Hex. `T` alterna relatório e tabela.

## Segurança

- **Tamanhos:** toda contagem lida do arquivo (seções, símbolos, importações,
  arquiteturas de um fat) é conferida contra o que cabe no arquivo antes de
  ser usada. Limite de 1 milhão de linhas por folha (o resto é cortado, e a
  nota diz isso) e de 64 arquiteturas num fat.
- **Nomes:** um nome de símbolo ou de seção é cortado em 4 KiB e passa pela
  limpeza do texto (controle vira `·`, bidi marcado e contado), porque um nome
  é bytes arbitrários.
- **Laços:** cadeias que apontam umas para as outras (tabelas de importação do
  PE, `LC_*` do Mach-O) têm teto de passos e não seguem a mesma posição duas
  vezes.
- **Nada é executado nem carregado.** O relatório nunca chama `ldd`,
  `otool`, `dumpbin` nem o carregador do sistema. O `ldd` executa o programa
  em alguns casos.
- **Tempo:** no `V`, a leitura das tabelas roda em segundo plano, com Esc,
  como a das tabelas grandes (`in_background`). O relatório sai primeiro. No
  Autoview, só o resumo e as proteções, com o prazo de 100 ms e sem as tabelas.
- **Pânico** vira hex, por `panic_guard::catch`.
- proptest: bytes aleatórios depois de cada magic (ELF, PE, Mach-O e fat), e
  executáveis reais com campos trocados por valores extremos, nunca dão pânico,
  nunca leem fora do arquivo e nunca passam do teto de memória.

## Etapas (um PR cada)

1. **ELF:** detecção, `ReadRef` limitado, relatório e as folhas.
2. **PE,** inclusive .NET (só o cabeçalho CLR e a versão do runtime) e o overlay.
3. **Mach-O e fat.**
4. **Identidade:** Go, Rust, PDB, build-id.

## Questões abertas

| # | Questão | Proposta |
|---|---|---|
| ?1 | Abrir no relatório ou no hex, como hoje? | no relatório; `H` leva ao hex |
| ?2 | Core dump: relatório completo ou só o resumo? | só o resumo (o programa, o sinal e os segmentos) |
| ?3 | Demangling de Rust e C++ | para depois; o `cpp_demangle` já teve estouro de pilha com entrada hostil e precisa de limite próprio |

## Para depois

- Demangling (`rustc-demangle`, `cpp_demangle` com limite de recursão).
- `.a`, WebAssembly, `.class` e `.dex`.
- Strings legíveis do binário como uma folha (como o `strings`), com tamanho
  mínimo configurável.
- Verificar a assinatura, se houver um leitor em Rust que não dependa do
  sistema.
