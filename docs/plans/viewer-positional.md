# Plano: visualizador multi-registro (posicional, código-fonte COBOL e SPED) e biblioteca comunitária de layouts

Origem: conversa de 2026-11-04, expandindo a nota de SDF/CNAB do Subprojeto 2 de
[docs/plans/viewers.md](viewers.md) e [docs/plans/viewer-tables.md](viewer-tables.md).

Estado: **retirado do Xylem em 2026-11-11, por decisão do autor.** O plano foi além do
escopo de um gerenciador de arquivos: catálogo de layouts, COBOL e mainframe interessam a
pouca gente e custariam código e manutenção ao núcleo. O visualizador posicional será um
produto separado, que talvez venha a ser integrado ao Xylem sem fazer parte dele. As
etapas 1a a 1c (seção 10) chegaram ao `main` pelo PR #103 e saíram antes de qualquer
release; o código está guardado na branch `positional-viewer` (commit `a3330ab`), como
ponto de partida do produto novo. Este arquivo fica como registro.

Estado anterior: aprovado pelo autor em 2026-11-04 (pronto para implementação por etapas).

---

## 1. Objetivo

1. Permitir que o visualizador do Xylem (`V` e, para layouts confiáveis, Autoview `F7`) exiba em
   grade tabular interativa **arquivos texto estruturados posicionais e multi-registro**, cobrindo:
   - **Posicionais de registro único** (ex.: SDF do dBase/Clipper, NOAA `.dly`);
   - **Posicionais multi-registro** (ex.: CNAB 240/400/750, FEBRABAN 150, Sintegra, AFD, SEFIP,
     EDI Proceda, Cielo/Rede, B3 COTAHIST, NACHA, Norma 43, CFONB, IATA SSIM);
   - **Delimitados multi-registro** (ex.: SPED Fiscal, EFD-Contribuições, ECD, ECF, MANAD, DIRF,
     DMED com pipe `|`; BAI2 com vírgula `,`).
2. Aceitar como fonte de estrutura do arquivo:
   - **Formato padronizado do Xylem (`.xlayout.toml`):** formalmente especificado nesta
     documentação, legível e validado estritamente;
   - **Código-fonte COBOL direto (`.cpy`, `.cob`, `.cbl`):** tanto *copybooks* isolados quanto
     **programas COBOL completos** (extraindo `FD` da `FILE SECTION` e/ou registros da
     `WORKING-STORAGE SECTION` da `DATA DIVISION`), usados diretamente ou combinados a um
     `.xlayout.toml`;
   - **Cabeçalho `.dbf`:** para arquivos SDF de registro único (`COPY TO ... SDF`).
3. Distribuir junto com o Xylem (GitHub Releases, `.deb`, `.rpm`, futuro instalador Windows e
   *fallback* embutido) uma **pasta `layouts/` com o maior catálogo possível de layouts públicos**,
   bem documentada para servir como porta de entrada de colaborações da comunidade (issues e PRs
   corrigindo posições ou adicionando novos padrões sem precisar alterar código Rust).

---

## 2. Modelo de Ameaça e Princípios de Segurança

Existem **duas entradas potencialmente hostis**: o **arquivo de dados** e, quando aberto de fora
das pastas confiáveis do Xylem, o **arquivo de layout / fonte COBOL**. Todos os princípios de
[viewers.md](viewers.md) são obrigatórios:

1. **Fronteira estrita de confiança para layouts:**
   - **Pastas confiáveis:** `~/.config/xylem/layouts/` (`%APPDATA%\xylem\layouts\` no Windows),
     `<dir_de_current_exe>/layouts/` (resolvida por `std::env::current_exe()`, **nunca** por
     `current_dir()` / `./layouts/` para evitar sequestro ao navegar em `Downloads/`) e
     `/usr/share/xylem-fm/layouts/`.
   - **Autoview (`F7`)** só usa layouts das pastas confiáveis (via auto-detecção segura, como
     SPED, ou regras do `config.toml`). Arquivos `.xlayout.toml`, `.cpy` ou `.cob` presentes na
     pasta visitada **jamais** são lidos sozinhos no Autoview (Princípios 1 e 5 de
     [viewers.md](viewers.md)).
   - Em **`V`**, um `.xlayout.toml`/`.cpy`/`.cob` fora das pastas confiáveis só é lido mediante
     **escolha explícita do usuário** na tecla `K`.
2. **100% Rust puro, zero `unsafe`, zero bibliotecas C:**
   - `toml` + `serde` para `.xlayout.toml`; parser iterativo próprio em Rust puro (com pilha
     explícita, zero recursão na *call stack*) para código-fonte COBOL.
3. **Defesa contra *Layout Bombs* e *COBOL Bombs*:**
   - Tamanho máximo do arquivo de layout/fonte COBOL: `MAX_LAYOUT_BYTES = 1 MiB` (`Read::take`).
   - Máximo de tipos de registro (*sheets*) por visualização: `MAX_RECORD_TYPES = 256`.
   - Máximo de colunas visíveis por registro: `MAX_COLS = 256` (`columns_cut()` acima disso).
   - Explosão combinatória de `OCCURS` aninhados no COBOL (`OCCURS 1000` dentro de `OCCURS 1000`):
     orçamento total de campos expandidos `MAX_EXPANDED_FIELDS = 1_024`, com `checked_add` e
     `checked_mul` em todo cálculo de posição e tamanho.
   - Largura máxima declarada por registro: `MAX_RECORD_BYTES = 64 KiB`.
   - **Proibição de inclusão arbitrária (`COPY` / `REPLACE` / `INCLUDE`):** fontes COBOL externos
     nunca abrem outros arquivos do sistema (`COPY "../../../../etc/passwd"` ou ciclos `A -> B`).
     Mesmo em `copybook = "..."` dentro de um `.xlayout.toml`, o caminho deve ser relativo sem `..`,
     sem separador de raiz/drive e resolvido dentro do mesmo diretório confiável.
4. **Defesa na leitura do arquivo de dados:**
   - Leitura em streaming via `Stoppable<BufReader<File>>` e `Budget` (3 s em `V`, 100 ms em `F7`,
     cancelável com `Esc`), parando em `MAX_TABLE_ROWS = 50.000` linhas por aba na tela e sem teto
     de linhas apenas no `Batch` (`Ctrl+B`).
   - Teto por linha física de `MAX_LINE_BYTES = 512 KiB` contra arquivos sem `\n`.
   - **Assinaturas binárias PKCS#7 pós-trailer (SPED ECD/ECF assinados):** parada limpa após o
     registro encerrador (`|9999|...|`), registrando a presença de bytes posteriores em `i`
     (`Structure`) sem corromper a grade nem forçar modo Hex por bytes `0x00` da assinatura.
   - Fatiamento seguro por caracteres ou bytes, sem jamais indexar fora de fronteira UTF-8 ou do
     comprimento real da linha (campos ausentes viram `Cell::Null`).
5. **Higienização de terminal e Trojan Source (CVE-2021-42574):**
   - Nomes de registros, campos, descrições, domínios `values`/`88` e células passam por
     `TableGrid::sanitize_cell` (C0, DEL, C1 e controles bidi viram `·`, corte em 2.048 chars) e
     pelo filtro final em `ui::draw`.
6. **Fuzzing com `proptest`:**
   - Cobertura obrigatória de `proptest` sobre o parser `.xlayout.toml`, o parser de fonte COBOL,
     o fatiador posicional e o leitor delimitado multi-registro.

---

## 3. Experiência de Uso no Viewer (TUI)

### 3.1. Grade Multi-Registro sobre `TableGrid`
- **Uma aba (`sheets`) por Tipo de Registro:** ex.: `0 - Header (1)`, `1 - Clientes (420)`,
  `3 - Faturas (1.890)`, `9 - Trailer (1)`, ou no SPED `0000 - Abertura (1)`, `C100 - NF-e (312)`,
  `C170 - Itens (1.420)`, `9999 - Encerramento (1)`.
  - `[` e `]` alternam entre os tipos de registro presentes no arquivo;
  - `Tab` abre a busca rápida por código ou nome do registro (`SheetList`);
  - Linhas órfãs/inválidas em arquivos posicionais aparecem na aba automática
    `[! Sem layout / Inválidas]` (criada apenas quando houver ocorrências);
  - Em arquivos delimitados com `allow_undeclared_records = true` (SPED), códigos de registro
    novos/raros ganham aba própria dinâmica (`G125`) com `CAMPO_02`, `CAMPO_03`...
- **Primeira coluna `#line`:** linha física (1-based) no arquivo original, essencial para cruzar
  com mensagens de erro de validadores bancários/PVA.
- **Sincronia bidirecional e Inspeção no Modo Texto (`T`):**
  - Apertar `T` num registro da grade leva o modo Texto exatamente àquela `#line` (e `T` no modo
    Texto reabre a aba correspondente posicionada naquele registro, com `Table` visível na barra de
    menu `VIEW`).
  - Enquanto um layout estiver ativo no modo Texto, o viewer exibe uma **régua / inspetor de campos**
    indicando o tipo de registro da linha do topo e o campo na coluna atual (nome, coordenadas
    `início..fim`, fatia bruta e valor decodificado/domínio), além de coloração alternada dos campos
    quando `C` (*Colors*) estiver ligado.
- **Descrições de domínio na célula ativa e na janela `i` (`Structure`):** quando o layout (ou um
  nível `88` do COBOL) declarar valores tabelados (ex.: `00 = Documento regular`,
  `02 = Documento cancelado`), a barra inferior de detalhe da célula exibe `00 — Documento regular`,
  mantendo o valor da célula como `"00"` para filtros SQL (`cod_sit = '00'`) e exportação `Ctrl+B`.
- **Janela `i` (`DISPLAY STRUCTURE`):** exibe posições `início..fim` (ou nº do campo no SPED),
  tamanho, tipo/máscara `PIC`, descrição, valores de domínio (`values` / `88`), campos `FILLER` e
  o resultado das conferências não bloqueantes de Header/Trailer (`9900`/`9999`, somatórios e
  contagens).
- **Visibilidade no Menu e Janela Seletora `K` (`K layout`):**
  - A barra inferior `VIEW` de `src/ui.rs` exibe explicitamente **`K layout`** (tanto no modo Texto
    quanto no modo Tabela), deixando claro para o usuário que pode aplicar ou trocar um layout; na
    grade multi-registro, a segunda linha mostra `Tab list · [ ] other record`.
  - Pressionar **`K`** abre a janela sobreposta `LAYOUT` (mesmo padrão visual de `SheetList`), com
    filtro instantâneo ao digitar e seções bem identificadas mostrando tudo o que pode ser escolhido:
    1. **Na pasta atual (mesmo nome-base em destaque):** `.xlayout.toml`, `.cpy`, `.cob`, `.cbl` e
       `.dbf` (SDF) encontrados ao lado do arquivo;
    2. **Meus layouts (`~/.config/xylem/layouts/`):** modelos personalizados do usuário;
    3. **Modelos do Xylem — Brasil** (Bancário, Cartões, Fiscal/SPED, Trabalhista, Logística, B3,
       Saúde) e **Internacional** (EUA, Espanha, França, Alemanha, Austrália, Canadá, IATA, NOAA),
       exibindo nome, categoria e descrição curta;
    4. **Ações explícitas no rodapé da janela (`LAYOUT`):** `↑↓ move · letters narrow · F2 file path... · Enter apply · Esc back` (e a opção `[No layout — plain text]` no topo quando já houver um layout ativo).

---

## 4. Especificação Formal do Formato `.xlayout.toml` (Schema v1)

Esta seção define o contrato completo do formato `.xlayout.toml`, servindo de referência tanto para
o validador em Rust quanto para a documentação pública de contribuidores.

### 4.1. Cabeçalho do Arquivo e Metadados de Rastreabilidade

Todo arquivo na biblioteca oficial `layouts/` deve declarar a fonte normativa (`spec_source` /
`spec_version`) para que qualquer usuário possa conferir posições contra o manual oficial e apontar
ajustes com precisão:

```toml
schema = 1                                      # Obrigatório. Versão da especificação Xylem (inteiro: 1).
id = "br/bancario/cnab240-febraban"             # Identificador único estável (opcional em arquivos avulsos).
name = "FEBRABAN CNAB 240 — Padrão Geral"       # Obrigatório. Nome legível exibido no seletor `K` e em `i`.
description = "Remessa e retorno bancário 240 posições (Cobrança, Pagamentos e Extrato)"
country = "BR"                                  # Código ISO-3166 ("BR", "US", "ES", "FR", "DE", "AU", "CA", "INTL").
category = "banking"                            # "banking" | "cards" | "tax" | "accounting" | "labor" | "logistics" | "market" | "healthcare" | "aviation" | "science" | "erp"
spec_source = "FEBRABAN Layout Padrão CNAB 240" # Órgão / manual de referência (recomendado na pasta layouts/).
spec_version = "10.11"                          # Versão/ano do manual oficial usado como base.
spec_url = "https://portal.febraban.org.br/"    # URL informativa (nunca acessada pelo Xylem!).

# Parâmetros físicos do arquivo de dados:
format = "positional"        # "positional" (padrão) ou "delimited".
encoding = "auto"            # "auto" (padrão), "utf-8", "cp1252", "cp850", "cp437", "utf-16le", "utf-16be".
offset_unit = "chars"        # Só posicional: "chars" (padrão) ou "bytes".
line_length = 240            # Só posicional (opcional): largura esperada da linha sem CR/LF.
strict_positions = true      # Só posicional (padrão true): exige que campos cubram 1..line_length sem sobreposição nem buracos.
hide_filler = true           # Padrão true: oculta campos `type = "filler"` na grade (mantendo-os em `i`).

# Parâmetros exclusivos de `format = "delimited"` (ex.: SPED, MANAD, DIRF, BAI2):
delimiter = "|"              # Caractere ASCII delimitador de 1 byte (ex.: "|", ",", ";", "\t").
outer_delimiters = true      # true para `|c1|c2|` (SPED: faz `col = 1` ser o 1º campo útil); false para CSV comum `c1,c2`.
quote = ""                   # Aspas opcionais de escape ("" = sem aspas, como no SPED; "\"" = aspas padrão).
decimal_sep = ","            # Separador decimal textual nos campos `decimal`: "," ou "." (ou "implied" se usar casas implícitas).
date_format = "DDMMYYYY"     # Formato padrão herdado pelos campos `type = "date"` que não informarem `format`.
allow_undeclared_records = true # Se true, códigos de registro não listados em [[records]] viram abas dinâmicas CAMPO_02...
stop_after_trailer = true    # Se true, encerra a leitura após uma linha com `role = "trailer"` (ignora assinatura PKCS#7).
```

### 4.2. Auto-detecção Segura (`[detect]`, opcional)

Permite que o Xylem reconheça automaticamente formatos inequívocos (como SPED, B3 COTAHIST,
Sintegra, AFD ou CNAB) sem falsos positivos em arquivos `.txt` comuns:

```toml
[detect]
extensions = ["txt", "rem", "ret"]              # Opcional: restringe extensões candidatas (vazio = qualquer texto).
line_length = 240                               # Opcional: primeira linha deve ter exatamente N caracteres.
first_line = [                                  # Todas as condições devem bater na 1ª linha:
  { pos = [8, 8], value = "0" },                #   posicional: fatia 1-based inclusiva
  { pos = [143, 143], in = ["1", "2"] },        #   conjunto de valores aceitos
]
# Em `format = "delimited"`, usa `col` e `outer_pipes = true`:
# first_line = [{ col = 1, value = "0000" }, { col = 2, regex = "^\\d{3}$" }]
```

### 4.3. Discriminador de Tipo de Registro (`[discriminator]` e `match`)

Três modos suportados:
1. **Sem `[discriminator]` (registro único):** o arquivo possui um único `[[records]]`, aplicado a
   todas as linhas (ex.: SDF, NOAA `.dly`).
2. **Discriminador simples global (`[discriminator]`):**
   - Posicional: `pos = [1, 1]` (1-based inclusivo; ex.: `[1, 2]` para Sintegra/B3, `[1, 3]` para
     Proceda). Cada `[[records]]` informa `id = "0"` (ou `ids = ["60M", "60A"]`).
   - Delimitado: `col = 1` (1-based). Cada `[[records]]` informa `id = "C100"`.
3. **Discriminador composto por registro (`match` em `[[records]]`):**
   - Necessário quando diferentes registros inspecionam colunas distintas (ex.: no CNAB 240, o
     Header olha só `pos = [8, 8] == "0"`, enquanto o Detalhe Segmento P olha `pos = [8, 8] == "3"`
     **e** `pos = [14, 14] == "P"`):
   ```toml
   [[records]]
   id = "3P"
   name = "Detalhe — Segmento P (Cobrança)"
   match = [
     { pos = [8, 8],   value = "3" },
     { pos = [14, 14], value = "P" },
   ]
   ```
   - A avaliação segue a ordem declarada (regras mais específicas com mais cláusulas `match` têm
     precedência sobre regras gerais).

### 4.4. Definição de Registros (`[[records]]`) e Campos (`fields`)

Cada bloco `[[records]]` aceita:
- `id` (string obrigatória em multi-registro): identificador curto (`"0"`, `"3P"`, `"C100"`).
- `name` (string obrigatória): título legível exibido na aba e em `Tab`.
- `role` (opcional): `"header"` | `"batch_header"` | `"detail"` (padrão) | `"batch_trailer"` |
  `"trailer"`.
- `line_length` / `min_length` (opcional, só posicional): sobrescreve o comprimento esperado para
  este tipo específico de registro (útil no AFD e no Sintegra onde alguns registros são mais curtos).
- `copybook` + `copybook_record` (opcional): importa os campos a partir de um `.cpy`/`.cob` na
  mesma pasta confiável (ver seção 5.4).
- `fields`: lista ordenada de especificações de campo.

#### Tabela completa de propriedades de um item em `fields`

| Propriedade | Onde vale | Obrigatório? | Descrição |
|---|---|---|---|
| `name` | ambos | Sim (exceto `filler`) | Nome do campo (ex.: `"cod_cliente"`, `"CHV_NFE"`). Em `filler`, assume `"filler"` se omitido. |
| `pos = [ini, fim]` | `positional` | Sim* | Posição inicial e final **1-based inclusivas** (`fim >= ini >= 1`). *Alternativa: `start` + `len`.* |
| `len` | `positional` | Opcional | Se informado junto com `pos`, o validador exige `fim - ini + 1 == len` (pega erros de digitação!). |
| `col` | `delimited` | Sim | Índice **1-based** do campo na linha delimitada (`1`, `2`, `3`...). |
| `type` | ambos | Sim | `"text"`, `"code"`, `"int"`, `"decimal"`, `"date"`, `"time"`, `"filler"`. |
| `decimals` | `decimal` | Opcional | Casas decimais implícitas em `positional` (ex.: `2` = COBOL `V99`); padrão `0` ou conforme `decimal_sep`. |
| `decimal_sep` | `decimal` | Opcional | Sobrescreve o global: `"implied"` (padrão posicional), `"."`, `","`. |
| `sign` | `int`/`decimal` | Opcional | `"none"` (padrão), `"leading"`, `"trailing"`, `"overpunch"` (sinal COBOL zonado `{`/`A..I`, `}`/`J..R`). |
| `format` | `date`/`time` | Opcional | Datas: `"YYYYMMDD"`, `"DDMMYYYY"`, `"YYMMDD"`, `"DDMMYY"`, `"YYYYDDD"` (juliana). Horas: `"HHMMSS"`, `"HHMM"`. |
| `trim` | `text`/`code` | Opcional | `"right"` (padrão em `text`), `"both"`, `"left"`, `"none"` (padrão em `code` preserva zeros, corta espaços finais). |
| `desc` | ambos | Opcional | Explicação oficial do campo mostrada em `i` (`Structure`). |
| `values` | ambos | Opcional | Mapa inline de domínio `{ "0" = "Entrada", "1" = "Saída" }`, exibido em `i` e no rodapé da célula ativa. |
| `check` | `int`/`decimal` | Opcional | Regra não bloqueante de conferência de Trailer (ver seção 4.5). |

#### Semântica de conversão dos tipos (`type`)
- **`"text"`:** texto alfanumérico (`PIC X`), espaços à direita removidos → `Cell::Text` (vazio → `Cell::Null`).
- **`"code"`:** identificadores numéricos ou alfanuméricos onde **zeros à esquerda são parte do dado**
  (CPF, CNPJ, CEP, agência, conta, NCM, CFOP, CST, chave NF-e de 44 dígitos, códigos de banco).
  Nunca é convertido para `i64`/`f64`, preservando todos os dígitos exatos como `Cell::Text`.
- **`"int"`:** quantidades e sequenciais numéricos inteiros → `Cell::Int(i64)` (em branco →
  `Cell::Null`; se contiver lixo não numérico, preserva `Cell::Text` bruto para evidenciar o erro).
- **`"decimal"`:** valores monetários, alíquotas e pesos → `Cell::Float(f64)` para comparações SQL,
  mantendo a representação formatada com casas exatas na célula.
- **`"date"` / `"time"`:** datas válidas são normalizadas para ISO `YYYY-MM-DD` (e horas para
  `HH:MM:SS`), permitindo filtros ordenáveis como `vencimento >= '2026-11-01'`. Preenchimentos
  nulos clássicos (`"00000000"`, `"99999999"` quando marcado ou espaços) viram `Cell::Null`; datas
  inválidas (ex.: `"20260231"`) são mantidas como texto bruto marcadas como inválidas em `i`.
- **`"filler"`:** áreas brancas/reservadas do layout. Ocultas na grade quando `hide_filler = true`,
  mas sempre listadas na janela `i` (`Structure`) para fechar 100% da matemática das posições.

### 4.5. Regras Declarativas de Conferência (`check`)
Executadas durante a leitura e resumidas em `i` (`Structure`) e num aviso discreto se divergirem:
- `"total_lines"`: valor deve bater com o número total de linhas do arquivo (ex.: `9999.QTD_LIN`).
- `"record_count:<ID>"`: deve bater com a quantidade de linhas do registro `<ID>` (ou da família).
- `"count_by_record:<COL_REG>"`: específico do bloco `9900` do SPED — confere se o valor bate com a
  contagem real do tipo de registro nomeado na coluna `<COL_REG>` da mesma linha `9900`!
- `"sum:<ID>.<CAMPO>"`: deve bater com o somatório (em centavos/escala fixa inteira `i128`, sem erro
  de ponto flutuante!) do `<CAMPO>` em todas as linhas do registro `<ID>`.

---

## 5. Especificação do Leitor de Código-Fonte COBOL (`.cpy`, `.cob`, `.cbl`)

Em muitos projetos corporativos, a documentação que o desenvolvedor recebe não é um PDF limpo: é um
*copybook* `.cpy` ou até o **código-fonte completo do programa COBOL (`.cob` / `.cbl`)** que lê ou
gera o arquivo. O Xylem lerá ambos nativamente.

### 5.1. Programas COBOL completos (`.cob` / `.cbl`) vs Copybooks (`.cpy`)
- Quando o arquivo contiver divisões de um programa completo (`IDENTIFICATION DIVISION`,
  `ENVIRONMENT DIVISION`, `DATA DIVISION`, `PROCEDURE DIVISION`):
  1. O lexer ignora `IDENTIFICATION` e `ENVIRONMENT DIVISION` (opcionalmente anotando declarações
     `SELECT ... ASSIGN TO` apenas como metadado informativo em `i`);
  2. Na `DATA DIVISION`, extrai prioritariamente os descritores `FD` (*File Description*) da
     `FILE SECTION` e seus registros `01` associados; se não houver `FILE SECTION` (ou se os layouts
     de registro estiverem montados em `WORKING-STORAGE SECTION` / `LINKAGE SECTION` com `READ ...
     INTO`), disponibiliza também os grupos `01` estruturados (ignorando variáveis escalares soltas
     de controle como contadores `77 WS-IND PIC 9(4)` ou flags de 1 campo);
  3. Ao atingir `PROCEDURE DIVISION`, encerra o parsing imediatamente!
- Quando o arquivo for um *copybook* puro (`.cpy`), inicia direto a leitura dos itens de nível
  `01`..`49`/`77`/`88` (mesmo que o `01` raiz tenha sido omitido e o `.cpy` comece em `03` ou `05`,
  caso muito comum em que o programa principal declarava o `01` antes do `COPY`: o Xylem envolve os
  itens num `01` sintético automaticamente!).

### 5.2. Formato de Colunas do Fonte COBOL
Auto-detecção linha a linha / por amostragem:
- **Formato Fixo (80 colunas ANS COBOL):** colunas `1..6` ignoradas; coluna `7` (`*` e `/` =
  comentário, `-` = continuação de literal, `D` = linha de debug ignorada); área útil nas colunas
  `8..72`; colunas `73..` descartadas (tags de modificação de mainframe).
- **Formato Livre (ISO 2002 / Micro Focus / GnuCOBOL):** comentários `*>` até o fim da linha ou `*`
  na coluna 1, sem truncamento na coluna 72.

### 5.3. Gramática de Dados Suportada
- **Níveis `01`..`49`, `66` (`RENAMES`), `77`, `88`:**
  - Pilha iterativa de grupos calculando deslocamentos exatos (`1-based`).
  - Desambiguação automática de nomes repetidos em subgrupos (`END-COBRANCA.CEP` vs
    `END-ENTREGA.CEP`).
  - **Níveis `88`:** `88 SIT-ATIVO VALUE 'A'.` ou `VALUES '01' THRU '09'`:
    - Quando ligados ao campo discriminador inicial de cada `01`, identificam automaticamente qual
      `01` vale para cada linha!
    - Nos demais campos, populam o dicionário `values` exibido em `i` (`Structure`) e no rodapé!
- **`PIC` / `PICTURE [IS]`:**
  - `X`, `A`, `9`, repetições `X(30)` e `XXX`, escala `P`, decimal implícito `V` (`0` bytes).
  - Sinal `S`: `SIGN [IS] LEADING/TRAILING [SEPARATE CHARACTER]` (+1 byte quando `SEPARATE`, ou
    decodificação *zoned overpunch* ASCII `{`/`A..I`, `}`/`J..R` quando embutido).
  - Máscaras editadas (`Z`, `*`, `+`, `-`, `.`, `,`, `/`, `B`, `0`, `CR`, `DB`).
  - `BLANK WHEN ZERO`.
  - Heurística inteligente `code` vs `int`: campos `PIC 9(n)` sem `V` nem `S` com nomes típicos de
    código/documento ou `n > 18` (que estourariam `i64`) preservam zeros à esquerda com segurança.
- **`USAGE [IS]`:**
  - `DISPLAY` (padrão texto: 1 posição por caractere).
  - `COMP-3` / `PACKED-DECIMAL` (`floor(n/2) + 1` bytes) e `COMP` / `COMP-4` / `COMP-5` / `BINARY`
    (`2`, `4` ou `8` bytes conforme dígitos): largura exata contabilizada para nunca deslocar campos
    seguintes, com decodificação BCD quando em modo de bytes.
- **`FILLER`** (explícito ou item elementar sem nome no COBOL 85+): oculto por padrão na grade e
  visível em `i`.
- **`OCCURS n [TIMES]`:** expandido em `campo[1]`..`campo[n]` (inclusive grupos aninhados), até o
  orçamento `MAX_EXPANDED_FIELDS = 1_024`. `OCCURS ... DEPENDING ON` gera nota explicativa em `i`.
- **`REDEFINES`:**
  - Em `01` (ou grupo raiz de registro): exposto como aba adicional de tipo de registro.
  - Em campos internos: retrocede o offset inicial ao do campo alvo e retoma após
    `max(fim_original, fim_redefines)` (regra exata do padrão COBOL), mantendo todos os campos
    posteriores perfeitamente alinhados; subcampos de `REDEFINES` internos ficam documentados em `i`
    (com opção futura de alternar visão redefinida).

### 5.4. Uso híbrido `.xlayout.toml` + `.cpy`/`.cob`
Quando um `.cpy` corporativo não contiver `VALUE`/`88` dizendo qual código na coluna 1 corresponde a
qual registro `01`, um `.xlayout.toml` mínimo resolve sem duplicar um único campo:

```toml
schema = 1
name = "Faturamento Legado (COBOL)"
[discriminator]
pos = [1, 1]

[[records]]
id = "0"
name = "Header"
copybook = "FATURA.COB"
copybook_record = "REG-HEADER"

[[records]]
id = "1"
name = "Detalhe"
copybook = "FATURA.COB"
copybook_record = "REG-DETALHE"
```

---

## 6. Catálogo Amplo `layouts/` e Governança Comunitária

A pasta `layouts/` na raiz do repositório será tratada como um **ativo de primeira classe do Xylem**,
projetada para crescer continuamente com contribuições de usuários.

### 6.1. Estrutura de diretórios e modelos levantados para o catálogo

```
layouts/
├── README.md                              # Guia rápido e convite à contribuição (issues/PRs)
├── br/
│   ├── bancario/
│   │   ├── cnab240-febraban.xlayout.toml  # Padrão FEBRABAN (Segs. A, B, E, G, J, J52, N, O, P, Q, R, S, T, U, W, Z)
│   │   ├── cnab400-itau.xlayout.toml      # Cobrança 400 Itaú
│   │   ├── cnab400-bradesco.xlayout.toml  # Cobrança 400 Bradesco
│   │   ├── cnab400-bb.xlayout.toml        # Cobrança 400 Banco do Brasil (CBR641/643)
│   │   ├── cnab400-santander.xlayout.toml # Cobrança 400 Santander
│   │   ├── cnab400-caixa.xlayout.toml     # Cobrança 400 Caixa Econômica Federal
│   │   ├── cnab400-sicoob.xlayout.toml    # Cobrança 400 Sicoob
│   │   ├── cnab400-sicredi.xlayout.toml   # Cobrança 400 Sicredi
│   │   ├── cnab750-pix.xlayout.toml       # FEBRABAN PIX Cobrança/Conciliação em Lote (750 pos.)
│   │   └── febraban-arrecadacao-150.xlayout.toml # Contas de consumo/tributos (registros A..Z, 150 pos.)
│   ├── cartoes/
│   │   ├── cielo-edi.xlayout.toml         # Extrato eletrônico Cielo
│   │   ├── rede-eevc.xlayout.toml         # Rede Crédito (EEVC)
│   │   ├── rede-eevd.xlayout.toml         # Rede Débito (EEVD)
│   │   ├── rede-eefi.xlayout.toml         # Rede Financeiro (EEFI)
│   │   ├── getnet-conciliacao.xlayout.toml
│   │   └── stone-conciliacao.xlayout.toml
│   ├── fiscal/
│   │   ├── sped-efd-icms-ipi.xlayout.toml # SPED Fiscal (ICMS/IPI)
│   │   ├── sped-efd-contribuicoes.xlayout.toml # EFD-Contribuições (PIS/COFINS)
│   │   ├── sped-ecd.xlayout.toml          # SPED Contábil (ECD)
│   │   ├── sped-ecf.xlayout.toml          # SPED Contábil Fiscal (ECF)
│   │   ├── sped-generico.xlayout.toml     # Qualquer arquivo pipe `|REG|...|`
│   │   ├── sintegra.xlayout.toml          # Convênio ICMS 57/95 (10, 11, 50..90 — 126 pos.)
│   │   ├── dimob.xlayout.toml             # Receita Federal — Atividades Imobiliárias (R01..R04, T9)
│   │   ├── dirf.xlayout.toml              # Receita Federal — DIRF (delimitado `|`)
│   │   ├── dmed.xlayout.toml              # Receita Federal — Serviços Médicos (delimitado `|`)
│   │   ├── manad.xlayout.toml             # Receita/Previdência — MANAD (delimitado `|`)
│   │   ├── paf-ecf.xlayout.toml           # Cupom Fiscal / Registros R01..R07
│   │   ├── nfse-rps-sp-paulistana.xlayout.toml # Prefeitura de SP — Lote RPS posicional
│   │   └── nfse-rps-barueri.xlayout.toml  # Prefeitura de Barueri — RPS posicional
│   ├── trabalhista/
│   │   ├── afd-ponto-1510.xlayout.toml    # Portaria MTE 1.510/2009
│   │   ├── afd-ponto-671.xlayout.toml     # Portaria MTP 671/2021 (REP-C / REP-P / REP-A)
│   │   ├── afdt-1510.xlayout.toml         # Arquivo Fonte de Dados Tratados
│   │   ├── acjef-1510.xlayout.toml        # Controle de Jornada para Efeitos Fiscais
│   │   ├── sefip-re.xlayout.toml          # SEFIP.RE FGTS/INSS (360 pos.)
│   │   ├── caged-legado.xlayout.toml      # Auditoria trabalhista CAGED posicional
│   │   └── rais-legado.xlayout.toml       # Auditoria trabalhista RAIS posicional
│   ├── logistica/
│   │   ├── proceda-notafis.xlayout.toml   # EDI Proceda Notas Fiscais
│   │   ├── proceda-conemb.xlayout.toml    # EDI Proceda Conhecimentos Embarcados
│   │   ├── proceda-ocoren.xlayout.toml    # EDI Proceda Ocorrências de Entrega
│   │   ├── proceda-doccob.xlayout.toml    # EDI Proceda Documentos de Cobrança
│   │   └── proceda-prefat.xlayout.toml    # EDI Proceda Pré-Fatura
│   ├── mercado/
│   │   └── b3-cotahist.xlayout.toml       # Série Histórica de Cotações da B3 (245 pos.)
│   └── saude/
│       ├── ans-sib.xlayout.toml           # ANS Sistema de Informações de Beneficiários
│       └── ptu-a500.xlayout.toml          # Cobrança/Intercâmbio PTU Batch posicional
├── intl/
│   ├── us-nacha-ach.xlayout.toml          # EUA — ACH / NACHA (94 pos.: 1, 5, 6, 7, 8, 9)
│   ├── us-bai2-cash.xlayout.toml          # EUA/Global — BAI2 Cash Management (delimitado `,`)
│   ├── us-irs-1099-fire.xlayout.toml      # EUA — IRS Pub. 1220 FIRE (750 pos.: T, A, B, C, K, F)
│   ├── es-aeb-norma43.xlayout.toml        # Espanha — Cuaderno 43 Extrato Bancário (80 pos.)
│   ├── es-aeb-norma19.xlayout.toml        # Espanha — Adeudos Directos Norma 19
│   ├── es-aeb-norma34.xlayout.toml        # Espanha — Transferências Norma 34
│   ├── fr-cfonb-120.xlayout.toml          # França — Extrato Bancário CFONB (120 pos.)
│   ├── fr-cfonb-240.xlayout.toml          # França — Operações CFONB (240 pos.)
│   ├── de-dtaus.xlayout.toml              # Alemanha — DTAUS Zahlungsverkehr (auditoria)
│   ├── au-aba-cemtex.xlayout.toml         # Austrália — ABA Direct Entry `.aba` (120 pos.)
│   ├── ca-cpa005-eft.xlayout.toml         # Canadá — Payments Canada Standard 005
│   ├── iata-ssim.xlayout.toml             # Aviação Global — IATA Schedules (200 pos.)
│   └── noaa-ghcn-daily.xlayout.toml       # Clima Global — NOAA GHCN-Daily `.dly` (269 pos.)
└── examples/
    ├── erp-remessa.xlayout.toml           # Exemplo didático comentado linha a linha
    ├── erp-copybook.cpy                   # Exemplo de Copybook COBOL puro com níveis 88
    └── importa-clientes.cob               # Exemplo de programa COBOL completo com FD na FILE SECTION
```

### 6.2. Garantia de Qualidade Automatizada no CI (para PRs da comunidade)
Para que qualquer usuário possa abrir um PR corrigindo uma posição num layout ou adicionando um novo
modelo com total segurança:
1. **Teste automático de integridade do catálogo (`tests/layouts_catalog.rs`):** percorre
   recursivamente todos os arquivos de `layouts/` durante `cargo test` e verifica:
   - sintaxe TOML/COBOL válida e `schema = 1`;
   - presença de `name`, `country`, `category`, `spec_source` e `spec_version`;
   - em todo registro posicional com `strict_positions = true`: **nenhum byte faltando**, **nenhuma
     sobreposição** de `pos = [ini, fim]`, soma exata igual a `line_length` e `len` consistente;
   - ausência de nomes de colunas duplicados no mesmo registro e ausência de caracteres de controle;
   - `id`s e regras `match` sem colisão ambígua.
2. **Empacotamento em todos os destinos:**
   - GitHub Releases (`.tar.gz` e `.zip` em `.github/workflows/release.yml`): pasta `layouts/` ao
     lado do binário;
   - Linux `.deb` e `.rpm` (`Cargo.toml` + `packaging/linux/packages.sh`): `/usr/share/xylem-fm/layouts/`;
   - Futuro instalador Windows: `<pasta_do_programa>\layouts\`;
   - `Cargo.toml` (`include`) + *fallback* somente-leitura para `cargo install xylem-fm`.

---

## 7. Plano de Documentação Completa

Por ser um recurso voltado também à colaboração aberta de usuários:

1. **Guia da Biblioteca e Contribuição (`layouts/README.md` + `layouts/README.pt-BR.md`):**
   - Lista todos os layouts incluídos com suas fontes oficiais e versões;
   - Explica passo a passo como testar uma alteração local copiando para `~/.config/xylem/layouts/`;
   - Ensina como abrir uma *issue* apontando divergência de manual (informando registro, campo,
     posição atual vs posição do manual oficial) ou enviar um PR adicionando um `.xlayout.toml` /
     `.cpy`.
2. **Referência Técnica da Especificação (`docs/manual/layouts.md` e traduções nos 6 idiomas —
   `en`, `pt`, `es`, `de`, `fr`, `zh`):**
   - Uso no viewer (`K`, abas `[` `]` `Tab`, `i`, filtro SQL `\`, `T` sincronizado e `Ctrl+B`);
   - Especificação completa de todas as chaves do `.xlayout.toml` com exemplos posicionais e SPED;
   - Guia completo da gramática COBOL suportada (`.cpy`, `.cob`, `.cbl`, formato fixo/livre, `PIC`,
     `88`, `REDEFINES`, `OCCURS` e uso híbrido TOML + COBOL);
   - Configuração de `[[layouts]]` no `config.toml`.
3. **Atualização de `docs/manual/viewer.md` (nos 6 idiomas) e do site (`site/` / mdBook):**
   - Nova página dedicada no menu lateral do manual online, facilitando que alguém pesquisando na
     web encontre a documentação e contribua no GitHub.

---

## 8. Roteiro de Implementação (Etapas)

Como o catálogo de `layouts/` é extenso e exige conferência cuidadosa contra os manuais públicos,
dividimos o trabalho em etapas incrementais revisáveis:

1. **Etapa 1 — Motor Multi-Registro (`positional` + `delimited`/SPED) e Validador `.xlayout.toml`:**
   - `src/viewer/multirecord/{mod.rs, xlayout.rs, decode.rs}`, integração em `records.rs` e
     `tabular_loader.rs`, coluna `#line`, abas por registro, `values` no rodapé e em `i`
     (`Structure`), checagens de Trailer/`9900`/`9999`, + suíte `proptest`.
   - Primeiro layout real do catálogo: `layouts/br/bancario/cnab240-febraban.xlayout.toml`,
     conferido contra o manual FEBRABAN, como caso de validação do motor (header de arquivo e de
     lote, segmentos com `match` composto e trailers com contagens), testado com arquivos
     sintéticos de remessa e retorno, de dados fictícios. O `tests/layouts_catalog.rs` passa a
     valer desde esta etapa.
2. **Etapa 2 — Leitor de Código-Fonte COBOL (`.cpy`, `.cob`, `.cbl`):**
   - `src/viewer/multirecord/cobol.rs` cobrindo copybooks e programas completos (`FILE SECTION` /
     `WORKING-STORAGE`), formato fixo e livre, `PIC`, `88`, `REDEFINES`, `OCCURS`, `FILLER`,
     ponte `copybook = "..."`, + suíte `proptest`.
3. **Etapa 3 — UI no Viewer (`K layout` no menu, inspetor/régua em `T`, auto-detecção SPED, `config.toml`) e SDF (`.dbf`):**
   - `K layout` bem visível na barra `VIEW` (`src/ui.rs`), janela seletora `LAYOUT` categorizada com
     filtro digitado e `F2` para caminho livre, sincronia de linha física e inspetor/cores de campos
     no modo Texto (`T`), suporte a `.dbf` para SDF e empacotamento em `release.yml` / `Cargo.toml` /
     `packages.sh`.
4. **Etapa 4 — Extensão Mainframe (`COMP-3` / `COMP` em registros fixos sem `\n` + codepages EBCDIC `CP037`/`CP500`):**
   - Tabela de conversão EBCDIC em `src/charset.rs` (`CP037` EUA/Brasil e `CP500` Internacional),
     leitura de registros de largura fixa em bytes (`line_ending = "none"`) e decodificação BCD de
     `COMP-3` / binário big-endian `COMP`, acompanhados de arquivos sintéticos de exemplo e testes.
5. **Etapa 5 — Construção do Catálogo `layouts/` (em lotes verificados por `tests/layouts_catalog.rs`):**
   - **Lote 5a (Fiscal e Trabalhista BR):** SPED ICMS/IPI, Contribuições, ECD, ECF, Genérico,
     Sintegra, DIMOB, DIRF, DMED, MANAD, PAF-ECF, RPS SP/Barueri, AFD 1510/671, AFDT, ACJEF,
     SEFIP, CAGED, RAIS.
   - **Lote 5b (Bancário, Cartões, Logística, Mercado e Saúde BR):** CNAB 400 (Itaú, Bradesco,
     BB, Santander, Caixa, Sicoob, Sicredi), CNAB 750 PIX, FEBRABAN 150, adquirentes
     com especificação pública verificável, Proceda (NOTAFIS, CONEMB, OCOREN, DOCCOB, PREFAT),
     B3 COTAHIST, ANS SIB, PTU.
   - **Lote 5c (Internacionais + Exemplos COBOL/Mainframe):** NACHA, BAI2, IRS 1099 FIRE,
     Norma 43/19/34, CFONB 120/240, DTAUS, ABA, CPA-005, IATA SSIM, NOAA GHCN-Daily e exemplos
     `.cpy`/`.cob` (incluindo um exemplo `COMP-3`/EBCDIC).
6. **Etapa 6 — Documentação Completa (`layouts/README*`, nova página `layouts.md` e `viewer.md` nos 6 idiomas, exemplos em `demo/`).**

---

## 9. Decisões Registradas (2026-11-04)

1. **Tecla e visibilidade no menu:** aprovada a tecla **`K`**, com **`K layout`** explícito na barra
   inferior `VIEW` do viewer e janela seletora `LAYOUT` mostrando claramente todas as categorias e
   fontes disponíveis (arquivos locais, modelos do usuário, catálogo Brasil/Internacional e `F2`
   para abrir caminho livre).
2. **Coluna `#line`:** aprovada como primeira coluna de cada registro na grade, sincronizada com o
   modo Texto pela tecla `T`.
3. **Inspeção no Modo Texto (`T`):** aprovada a régua/inspetor de campos no modo Texto quando houver
   layout ativo (indicando registro, campo, posição e valor decodificado, mais cores alternadas com
   `C`).
4. **`COMP-3` e EBCDIC:** foco principal do MVP (Etapas 1–3) em arquivos texto (`USAGE DISPLAY` e
   delimitados), seguido pela Etapa 4 adicionando `EBCDIC` (`CP037`/`CP500`), registros de bytes
   fixos sem `\n` e `COMP-3`/`COMP` com exemplos sintéticos dedicados de teste/demonstração.
5. **Catálogo `layouts/`:** incluir todos os modelos com especificação pública verificável (validados
   por `tests/layouts_catalog.rs` no CI) e documentar em `layouts/README.md` como a comunidade pode
   apontar correções ou contribuir novos layouts.
6. **CNAB 240 na Etapa 1:** o `cnab240-febraban.xlayout.toml` sai do Lote 5b e entra na Etapa 1,
   como primeiro layout real e caso de validação do motor multi-registro, junto com o
   `tests/layouts_catalog.rs`.

---

## 10. Etapas 1a, 1b e 1c (2026-11-09)

Aprovado pelo autor em 2026-11-09: implementar as fatias 1a, 1b e 1c (abaixo), **só com
layouts de manual público**; os layouts de banco cujo manual não é público ficam
documentados em `layouts/README*.md` para a comunidade. Feito na branch `positional-viewer`.

### 10.1. Fatias

As etapas 1 e 3 da seção 8 foram refatiadas para que cada fatia funcione sozinha no `main`:

| Fatia | O que entrou |
|---|---|
| 1a | Motor posicional e validador do `.xlayout.toml` (`src/viewer/multirecord/{xlayout,decode,mod}.rs`), catálogo embutido com detecção pela primeira linha, abas por registro, `#line`, `i`, significado dos códigos no rodapé, conferências. Primeiro layout: CNAB 240 FEBRABAN |
| 1b | Formato delimitado (SPED): registros não declarados com abas dinâmicas (`CAMPO_nn`), `9900`/`9990`/`x990`/`9999`, parada no trailer com aviso dos bytes seguintes (assinatura PKCS#7). Layout: SPED genérico |
| 1c | Tecla `K` com a janela `LAYOUT` (seções Here, Mine, Brazil, World; F2 para caminho livre), pasta do usuário, SDF a partir do cabeçalho de um `.dbf`, `T` levando à linha do registro |

### 10.2. Decisões

1. **Catálogo embutido, e só duas fontes confiáveis.** O `build.rs` embute todo
   `layouts/**/*.xlayout.toml` (`CATALOG`), e confiáveis são só ele e
   `~/.config/xylem/layouts/` (até 3 níveis, 256 arquivos). Saíram `<dir_do_exe>/layouts/` e
   `/usr/share/xylem-fm/layouts/`: um `xy` rodado de dentro de `Downloads/` tornaria a pasta
   confiável, e o catálogo nunca difere da versão em uso. Nada muda no empacotamento; o
   `Cargo.toml` inclui `layouts/` para o crates.io. Contribuir com um layout não exige Rust.
2. **`decimal` é texto com as casas exatas** (`1234.50`), nunca `f64`; o filtro SQL o lê como
   número. As somas das conferências são inteiros `i128` na escala de 9 casas.
3. **Uma passada pelo arquivo inteiro** guarda só as linhas da aba pedida (até 50.000) e conta
   todas, para os nomes das abas e as conferências. Trocar de aba relê o arquivo. As abas
   vêm na ordem da primeira aparição. Se a leitura parar (tempo ou Esc), as contagens levam
   `+` e as conferências não são feitas. O Batch para quando tem as suas linhas.
4. **Novidades no esquema:** `batch_match` (condições sobre a linha do header do lote, para
   separar o segmento A de pagamento do de débito no CNAB 240); conferências `batch_lines`,
   `batch_sum`, `block_lines` (o x990 do SPED conta o 0000 e o 9999, como manda o Guia
   Prático); `*` como prefixo e `+` para somar ids ou termos; `status = "verified" | "draft"`.
5. **Ainda recusado com mensagem clara:** `copybook` (etapa 2), `quote` diferente de `""`,
   UTF-16. O `[[layouts]]` no `config.toml` ficou de fora: a pasta do usuário com `[detect]`
   cobre o mesmo uso com menos superfície.
6. **`T`** leva da linha da grade à linha do texto, dentro do que o visualizador leu (16 MiB
   hoje); além disso, uma nota explica. A régua/inspetor de campos no modo texto fica para
   depois, junto com o viewer em janelas (subprojeto 3 de [viewers.md](viewers.md)).
7. **Teste do catálogo** é um teste unitário (`catalog::tests`), não `tests/layouts_catalog.rs`:
   o crate só tem binário. Ele confere que todo arquivo de `layouts/` é válido, estrito, tem
   `id` igual ao caminho, metadados e `[detect]`.
8. **Textos da interface em inglês**, como o resto do Xylem (`! No record type`, `K layout`);
   os layouts brasileiros têm nomes e descrições em português. `CAMPO_nn` segue o spec.

### 10.3. Fontes conferidas

- **CNAB 240:** "Layout Padrão FEBRABAN 240 posições", versão 11.0, de 11/09/2026, da página
  "Layouts" do portal da FEBRABAN. As tabelas foram extraídas do PDF e convertidas em TOML; o
  validador confirmou que cada registro cobre 1..240. Ficou de fora o J-52 para o Pix: o PDF
  não deixou extrair o código de forma de lançamento que o distingue do J-52 comum.
- **SPED:** Guia Prático da EFD ICMS/IPI, versão 3.2.2 (11/02/2026), Capítulo III, registros
  0001, 0990, 9001, 9900, 9990 e 9999.

### 10.4. Próximo

O resto do plano (régua no modo texto, COBOL, mainframe, o catálogo em lotes, a documentação
completa) segue as etapas da seção 8, sem data; a ordem em relação ao resto do
[roadmap](roadmap.md) ainda será decidida. O catálogo e o que fica para a comunidade estão
em `layouts/README.pt-BR.md`.
