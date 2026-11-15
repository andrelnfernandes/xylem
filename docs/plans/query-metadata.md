# Estudo: metadados dentro dos arquivos, na consulta do W

Origem: conversa de 2026-11-15. O autor perguntou se a consulta ao disco
([query.md](query.md)) acharia metadados de dentro dos arquivos, como a cidade onde uma
foto foi tirada, e pediu que isso ficasse registrado só como estudo.

**Estado: estudo. Nada aprovado, nada implementado.**

## A ideia

A tabela `files` do W ganharia colunas que vêm de dentro dos arquivos:

```sql
ext IN ('jpg', 'heic') AND city = 'Lisboa' AND taken >= '2024-06-01'
ext = 'mp3' AND artist = 'Cartola'
ext = 'docx' AND author = 'Maria' AND modified_by <> author
```

Hoje a consulta vê o que a árvore sabe (nome, tamanho, data, permissões), o texto
(`contains`) e as palavras indexadas (`words`). Uma foto é binária: fica fora dos dois
últimos, e nada lê os metadados dela.

## Os formatos mais conhecidos

| Família | Formatos | Onde ficam os metadados | O que se procura neles | Leitor no Xylem hoje |
|---|---|---|---|---|
| Fotos | JPEG, TIFF | EXIF, IPTC e XMP | data em que foi tirada, câmera, lente, GPS, orientação, autor, legenda, palavras-chave | não |
| Fotos de celular | HEIC/HEIF (iPhone), AVIF | EXIF e XMP dentro de caixas ISO BMFF | os mesmos, quase sempre com GPS | não |
| Fotos RAW | DNG, CR2/CR3, NEF, ARW, ORF, RAF | EXIF (TIFF por baixo, quase todos), mais notas do fabricante | câmera, lente, exposição, número de série | não |
| Imagens da web | PNG, WebP, GIF | blocos de texto (`tEXt`, `iTXt`), `eXIf`, XMP | software, autor, às vezes EXIF inteiro | não |
| Música | MP3 | ID3v1 e ID3v2 | artista, álbum, faixa, ano, gênero, capa | não |
| Música | FLAC, Ogg Vorbis, Opus | Vorbis comments | os mesmos | não |
| Música e vídeo | M4A, MP4, MOV | átomos `moov/udta`, `meta` | artista, álbum; nos vídeos de celular, data e GPS | não |
| Vídeo | MKV, WebM | tags do Matroska | título, idioma, data | não |
| Documentos | PDF | dicionário Info e XMP | título, autor, criador, programa, datas | não |
| Office | DOCX, XLSX, PPTX | `docProps/core.xml` e `app.xml`, dentro do ZIP | autor, último a modificar, empresa, datas, revisão, páginas | **parcial**: o `zip` já está, e o `calamine` lê XLSX |
| LibreOffice | ODT, ODS, ODP | `meta.xml`, dentro do ZIP | os mesmos | **parcial**: o `zip` já está, e o `calamine` lê ODS |
| Livros | EPUB | o `.opf`, dentro do ZIP | título, autor, editora, idioma, ISBN | **parcial**: o `zip` já está |
| E-mail | EML | o cabeçalho | de, para, assunto, data | não, mas é texto (`contains` já acha) |
| Programas | EXE e DLL (PE) | recurso de versão | fabricante, versão, assinatura | não |
| Fontes | TTF, OTF | tabela `name` | família, estilo, fabricante | não |

Observações:

- **Os do Office, LibreOffice e EPUB são os mais baratos:** são ZIP com XML dentro, e
  o Xylem já abre ZIP e lê XML de planilha. O resto pede leitor novo.
- **Os que mais vazam informação pessoal** são as fotos e os vídeos de celular (GPS,
  data, aparelho) e os documentos do Office (nome do autor, de quem revisou, empresa).
  Uma coluna que mostra isso também ajuda a achar o que limpar antes de mandar um
  arquivo para alguém.
- **Áudio e vídeo** têm um formato de metadados por contêiner, e cada um tem suas
  versões (ID3v2.2, 2.3, 2.4).

## Formatos menos conhecidos

Fora do uso doméstico, muitos formatos guardam um cabeçalho rico, e às vezes é só isso
que interessa: quem não abre a imagem de um exame quer saber de que aparelho ela veio.

### Saúde

| Formato | Uso | Metadados | Leitor |
|---|---|---|---|
| **DICOM** (`.dcm`, ou sem extensão; `DICM` no byte 128) | exames de imagem: tomografia, ressonância, raio-X, ultrassom, mamografia, medicina nuclear | centenas de campos (*tags*): modalidade, fabricante, modelo, número de série e software do aparelho, estação, instituição, parte do corpo, espessura do corte, kVp, tempo de exposição, dose, tamanho e espaçamento dos pixels, datas do estudo e da série, e os dados do paciente | não; há o projeto `dicom-rs`, em Rust (não conferido) |
| **DICOMDIR** | o índice de um CD ou pendrive de exames | a lista de pacientes, estudos, séries e imagens do disco | o mesmo |
| **NIfTI** (`.nii`, `.nii.gz`), Analyze (`.hdr`/`.img`) | neuroimagem, pesquisa | dimensões, tamanho do voxel, orientação, unidade de tempo, descrição | não; o cabeçalho tem tamanho fixo (348 ou 540 bytes) |
| **NRRD**, MINC | pesquisa em imagem médica | campos de texto no cabeçalho (NRRD); HDF5 por baixo (MINC 2) | não |
| **EDF/EDF+**, BDF | EEG, polissonografia, sinais fisiológicos | identificação do paciente e do registro, data, duração, canais, unidades, filtros | não; o cabeçalho é texto ASCII de largura fixa |
| **SCP-ECG**, **HL7 aECG** (XML), DICOM ECG | eletrocardiograma | aparelho, derivações, taxa de amostragem, paciente | aECG é XML |
| **Lâminas digitais**: SVS (Aperio), NDPI (Hamamatsu), MRXS (3DHistech), DICOM WSI | patologia digital | scanner, aumento, resolução, data; quase todos são TIFF por baixo | não |
| **HL7 v2** (`.hl7`) | mensagens entre sistemas do hospital | segmentos de texto separados por `\|` (paciente, pedido, resultado) | é texto: `contains` já acha |
| **FHIR** (JSON ou XML) | o padrão novo de troca de dados de saúde | recursos com tipo, datas, códigos | é texto: `contains` já acha |

Observações sobre o DICOM, o mais importante deles:

- **Os metadados vêm antes da imagem:** dá para ler só o cabeçalho e parar na tag dos
  pixels (`7FE0,0010`), sem descomprimir a imagem (que pode ser JPEG 2000, JPEG-LS,
  RLE). Uma busca sobre milhares de exames fica rápida.
- **Os campos teriam nomes de coluna próprios,** pelo dicionário do padrão:
  `dicom('Manufacturer')`, ou colunas fixas para os mais usados (`modality`,
  `manufacturer`, `model`, `station`, `body_part`, `study_date`).
- **Exemplos de consulta:**
  `modality = 'CT' AND manufacturer = 'SIEMENS' AND study_date >= '2025-01-01'`;
  `modality = 'MG' AND software NOT LIKE '%VB10%'` (mamografias de um software
  antigo); `kvp > 120`.
- **Variações que o leitor precisa aguentar:** VR implícita e explícita, little e big
  endian, sintaxe *deflate* (o cabeçalho comprimido), tags privadas de cada
  fabricante, sequências aninhadas (com profundidade limitada) e arquivos sem o
  preâmbulo de 128 bytes.

### Ciência, engenharia e mapas

| Formato | Uso | Metadados |
|---|---|---|
| **GeoTIFF** | mapas, imagens de satélite | sistema de coordenadas, extensão, resolução |
| **GPX**, KML/KMZ | trilhas de GPS, mapas | pontos, datas, nomes (é XML) |
| **LAS/LAZ** | nuvens de pontos de LiDAR | sistema de coordenadas, extensão, equipamento, data |
| **Shapefile** (`.shp` com `.dbf` e `.prj`) | dados geográficos | projeção (`.prj`), atributos (o `.dbf`, que o Xylem já lê) |
| **FITS** | astronomia | cabeçalho de texto: telescópio, instrumento, objeto, exposição, coordenadas |
| **HDF5**, NetCDF | dados científicos, clima | atributos livres em cada grupo e conjunto de dados |
| **SEG-Y** | sísmica | cabeçalho de texto (EBCDIC) e binário |
| **BWF** (WAV com o bloco `bext`) | áudio de rádio e TV | origem, data, código de tempo |
| **MXF** | vídeo profissional | câmera, timecode, produção |

## Dados de saúde: o cuidado a mais

Um arquivo DICOM, EDF ou HL7 traz **dado pessoal sensível**: nome, identificação, data
de nascimento e sexo do paciente, nome do médico, instituição. No Brasil isso cai na
LGPD como dado de saúde. Por isso, se um dia for feito:

- **Os campos do paciente ficam fora do índice por padrão.** O índice (Alt+N) é uma
  cópia no cache do usuário; copiar nomes de pacientes para lá multiplica o dado sem
  ninguém perceber. Indexar os campos do equipamento e do exame (modalidade,
  fabricante, kVp) não traz esse risco.
- **Na consulta sem índice,** os campos do paciente podem ser lidos, porque nada sai da
  memória; mas o manual avisa que o resultado na tela é dado sensível.
- **O mesmo leitor serve para achar o que não deveria estar lá:** exames que deviam ter
  sido anonimizados e ainda têm `PatientName`, por exemplo. Isso é útil numa
  instituição.
- **Nunca enviar nada para fora** (já é regra do `query.md`).

## A cidade de uma foto

A foto não guarda a cidade: guarda latitude e longitude. Passar de uma à outra é
geocodificação reversa.

- **Sem internet:** uma lista de cidades embutida, como a do GeoNames com as cidades
  acima de 15 mil habitantes (cerca de 25 mil entradas, alguns MB). A cidade é a mais
  próxima da coordenada.
- **Mandar as coordenadas para um serviço online fica fora,** como no `query.md`: nada
  do conteúdo sai da máquina.
- **Colunas possíveis:** `lat`, `lon`, `city`, `country`, e uma função de distância
  como `near('Lisboa', 20)` (até 20 km).
- **Limites:** perto de uma divisa, a cidade mais próxima erra; foto sem GPS (câmera
  comum, imagem editada, foto mandada por aplicativo que tira o GPS) não tem cidade.

## Como entraria

- **Colunas por família,** vazias (`NULL`) nos arquivos que não as têm: `taken`,
  `camera`, `lat`, `lon`, `city`, `artist`, `album`, `title`, `author`,
  `modified_by`, `pages`, `duration`.
- **A mesma regra da etapa 2:** só se lê o metadado quando a condição precisa dele, e
  o resto da condição decide primeiro (`ext = 'jpg' AND city = 'Lisboa'` só abre os
  `.jpg`).
- **Ler só o começo:** quase todos os formatos guardam os metadados no início (EXIF,
  ID3v2, `docProps` no índice do ZIP). Ler poucos KiB por arquivo mantém a busca
  rápida mesmo sem índice. Exceções: MP4 com o `moov` no fim, ID3v1 no fim do MP3.
- **No índice (Alt+N):** os metadados entram junto, e a consulta sobre milhares de
  fotos responde na hora.

## Segurança

- **Cada família é um leitor novo de arquivo hostil.** Pela regra do projeto: Rust
  puro, limites de tamanho, tempo e profundidade, `panic_guard`, e fuzz com arquivos
  corrompidos antes de entrar.
- **Os valores são texto hostil:** na tela, sem controles nem bidi, com tamanho
  limitado, como os nomes de arquivo.
- **O índice passa a guardar dados pessoais** (GPS, autores): o manual diz isso, junto
  com o comando de apagar.

## Proposta: leitores à parte, instalados por quem quer (2026-11-15)

Proposta do autor, registrada para estudo: os leitores especializados **não fazem
parte do núcleo do Xylem.** O Xylem leva só os mais usados e sem dado pessoal sensível;
os outros (DICOM, EDF, lâminas digitais...) são instalados pelo usuário que precisa
deles. O autor pretende escrever leitores para o uso dele, e o formato fica aberto para
qualquer pessoa escrever um leitor novo.

### Por que um programa à parte

Foram pesados quatro caminhos:

| Caminho | Como se adiciona | Complexidade | Problema |
|---|---|---|---|
| Features do Cargo | compilando o Xylem com `--features dicom` | baixa | quem baixa o binário pronto não consegue |
| **Programa à parte** | instalando um executável, que o Xylem descobre | média | processo e protocolo a desenhar |
| WebAssembly | copiando um `.wasm` | média a alta | motor grande (`wasmtime`, ~10 a 20 MB) ou lento (`wasmi`) |
| Biblioteca dinâmica | copiando um `.so` ou `.dll` | alta | Rust sem ABI estável: C e `unsafe`, e um erro no leitor derruba o Xylem |

Recomendado: **o programa à parte.** O arquivo hostil é lido fora do processo do
Xylem, que só recebe texto; um leitor que trava ou quebra vira "o leitor falhou", e o
Xylem segue.

Isso revê em parte a decisão de deixar de fora programas externos
([roadmap.md](roadmap.md), linha dos "conversores externos"): ali eram programas
quaisquer do sistema (`exiftool`, `pdftotext`); aqui é um formato de leitor do próprio
Xylem, com as regras abaixo. A decisão continua sendo do autor.

### Como funciona

- **Um leitor é um executável** numa pasta de leitores
  (`~/.local/share/xylem/readers/`, `%LOCALAPPDATA%\xylem\readers\` no Windows).
  Nada é baixado pelo Xylem: o usuário põe o leitor lá, ou instala um pacote que põe.
- **Ele se apresenta** quando chamado com `--describe`: nome, versão, versão do
  protocolo, extensões e assinaturas de bytes que lê, as colunas que dá (nome, tipo,
  descrição) e **quais delas são dado pessoal**.
- **Ele lê um arquivo de cada vez pela entrada padrão.** O Xylem abre o arquivo e
  passa o descritor como stdin; o leitor não precisa (e não pode, ver abaixo) abrir
  nada no disco. A resposta é uma linha JSON com os valores, na saída padrão.
- **No W,** as colunas dos leitores instalados aparecem com o prefixo do leitor
  (`dicom.modality`, `dicom.manufacturer`), e valem `NULL` nos arquivos que ele não
  lê. Só se chama o leitor quando a condição precisa da coluna, como o `contains`.
- **No índice (Alt+N),** os valores entram junto, menos os declarados como dado
  pessoal.
- **Um kit para quem escreve leitores:** a especificação do protocolo no manual e uma
  crate de apoio no repositório (ler o stdin com limites, escrever a resposta, declarar
  as colunas), com um leitor de exemplo.
- **Os leitores do projeto** (os mais usados, se um dia não couberem no núcleo) seriam
  compilados pelo CI e publicados como pacotes à parte, ao lado do Xylem.

### Segurança: memória, arquivos hostis e cavalos de Troia

Um leitor é código de outra pessoa rodando na máquina do usuário. O Xylem não consegue
tornar seguro um programa malicioso; o que ele faz é **não rodar nada sem o usuário
saber, e prender o que roda.**

**Confiança, contra o cavalo de Troia:**

- **Nada roda sem ser aprovado.** Na primeira vez que um leitor seria usado, o Xylem
  mostra o caminho, o nome, a versão, as colunas e o SHA-256 do executável, e pergunta.
  A aprovação fica guardada com o hash (como o `git_trusted` da configuração).
- **Executável mudado é leitor novo:** se o hash mudar (uma atualização, ou alguém
  trocou o arquivo), pergunta de novo.
- **A pasta e o executável** precisam ser do usuário e não graváveis por outros; se
  forem, o leitor é recusado (como o Git recusa repositórios de outro dono).
- **Leitores do projeto assinados:** uma chave pública dentro do Xylem confere a
  assinatura (Ed25519, em Rust puro); esses dispensam a pergunta. Os de terceiros
  sempre perguntam.
- **Nunca no Autoview nem ao passar o cursor:** um leitor só roda numa ação pedida (W,
  Alt+N).

**Isolamento, contra o leitor que se comporta mal:**

- **Linux:** Landlock sem nenhum acesso ao sistema de arquivos (o arquivo já chega
  aberto pelo stdin), sem rede (Landlock de rede, kernel 6.7 ou mais novo; seccomp
  antes disso), `no_new_privs`, ambiente vazio, diretório de trabalho vazio, novo grupo
  de processos.
- **Windows:** Job Object com limite de memória e de processos, morto quando o Xylem
  fecha, e token restrito. O isolamento é mais fraco que no Linux, e o manual diz isso.
- **macOS:** sem máquina para testar; fica para depois, como o resto do macOS.

**Limites, contra o excesso de memória e o arquivo hostil:**

- **Memória:** `RLIMIT_AS` (por exemplo 256 MiB por leitor) no Linux; o limite do Job
  Object no Windows. Estourou, o processo morre e o arquivo conta como "o leitor
  falhou".
- **Tempo:** um prazo por arquivo (por exemplo 2 s), e o grupo de processos é morto.
- **Um leitor de cada vez** (ou poucos), para a memória total ser conhecida.
- **A resposta:** tamanho máximo (por exemplo 64 KiB), número máximo de colunas,
  profundidade do JSON limitada, valores longos cortados; texto sem controles nem bidi
  na tela, como os nomes de arquivo.
- **O leitor não decide o que o Xylem faz:** a resposta são só valores; nada nela é
  comando, caminho a abrir ou texto de consulta.

**Dado pessoal:**

- O leitor declara as colunas sensíveis, e o índice as deixa fora por padrão. Isso só
  protege contra o leitor honesto que esqueceu algo no índice; contra o desonesto,
  vale a aprovação acima.
- Um leitor de saúde não vem com o Xylem: só existe na máquina de quem o instalou.

### O que fica em aberto

- A pasta de leitores única, ou também uma por projeto.
- Um processo por arquivo (simples) ou um processo vivo atendendo vários (mais rápido
  numa busca sobre milhares de exames, mas com estado entre arquivos).
- O formato da assinatura dos leitores do projeto e quem guarda a chave.
- Como o usuário vê e revoga as aprovações (uma página na configuração, F12).

### Estimativa

No ritmo das sessões de 2026-11:

| Parte | Tempo |
|---|---|
| Protocolo, descoberta, aprovação por hash, limites e isolamento no Linux | 2 a 3 dias |
| Colunas dos leitores no W e no índice | meio dia |
| Kit para quem escreve leitores e um leitor de exemplo | meio dia |
| Isolamento no Windows (Job Object) | meio a um dia |
| Assinatura dos leitores do projeto e empacotamento no CI | meio a um dia |
| Um leitor de DICOM (depende da maturidade do `dicom-rs`, não conferida) e o fuzz | 1 a 2 dias |

Cerca de uma semana até o núcleo mais um leitor especializado.

## Ordem sugerida, se um dia for feito

1. Office, LibreOffice e EPUB: o leitor de ZIP e XML já existe.
2. EXIF de JPEG e TIFF, sem cidade (data, câmera, `lat` e `lon`).
3. A cidade, pela lista embutida.
4. HEIC e os vídeos de celular.
5. Áudio (ID3 e Vorbis comments).
6. PDF, que pede o leitor mais delicado.

## Em aberto

- Quais famílias valem para o público do Xylem: o DICOM é muito usado em saúde, mas
  só lá.
- Se os campos do paciente (DICOM, EDF, HL7) podem ser lidos na consulta, ou só os do
  equipamento e do exame.

- Se entra na consulta do W ou fica como uma visão à parte (como o Alt+F mostra o tipo).
- Qual biblioteca em Rust para cada família: não conferido.
- Quanto a lista de cidades pesa no binário, e se fica atrás de uma feature, como o
  índice.
