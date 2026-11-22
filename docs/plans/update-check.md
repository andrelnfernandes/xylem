# Plano: avisar que saiu uma versão nova

**Feito na 0.9.0** (pedido e aprovado em 2026-11-21; PR #132). O padrão foi decidido pelo autor: o Xylem
**pergunta na primeira execução** se pode procurar versões novas.

## O motivo e o cuidado

Hoje o Xylem só usa a rede quando o usuário pede: ssh, rclone, git. O aviso de versão
nova seria a **primeira conexão feita sem um pedido**, então:

- ela só acontece com o "sim" do usuário, dado uma vez e guardado na configuração;
- o manual diz para onde vai, o que leva e como desligar;
- a resposta é tratada como hostil, como qualquer arquivo de fora;
- nada é baixado nem instalado: o Xylem só avisa.

## A fonte: um arquivo mínimo no site

`https://andrelnfernandes.github.io/xylem/latest.txt`, com uma linha só: a versão
(`0.10.0`).

- **Quem gera:** o `pages.yml`, a partir do `version` do `Cargo.toml`, em
  `site/build.sh`. O workflow passa a rodar também quando o `Cargo.toml` muda.
- **Por que não a API do GitHub** (`/releases/latest`): o JSON é grande e pede um
  parser de JSON vindo da rede, e o limite é de 60 consultas por hora por IP, o que é
  pouco atrás de NAT corporativo.
- **Cuidado de ordem:** o `version` do `Cargo.toml` muda na main antes de a release
  ser publicada (a 0.9.0 já está lá, sem tag). Então o arquivo deve vir da **última
  tag `v*`** (`git describe --tags --abbrev=0`, com `fetch-depth: 0` no checkout),
  não do `Cargo.toml`. Assim ninguém é avisado de uma versão que ainda não dá para
  baixar.
- **O que a requisição leva:** só um GET com User-Agent `xylem/<versão>`. Não há id
  de instalação, sistema nem configuração. O GitHub Pages vê o IP, como em qualquer
  visita ao site; o manual diz isso.

## A versão protegida contra PRs de fora

Pedido do autor (2026-11-21): **sempre avisar quando um PR que não seja dele tentar
mudar a versão.** Com o aviso de versão nova, o número no site manda todos os
usuários atualizarem; mudá-lo indevidamente seria um caminho de ataque.

O que conta como mudar a versão:

- o `version` do pacote `xylem-fm` no `Cargo.toml` ou no `Cargo.lock`;
- as tags `v*`;
- como o `latest.txt` é feito: `site/build.sh` e `.github/workflows/pages.yml`;
- `.github/workflows/release.yml` e `.github/release-notes/`.

Duas camadas:

1. **No CI, um job `version-guard`** em `ci.yml`, só em `pull_request`. Se o autor
   do PR não for o dono do repositório (`github.event.pull_request.user.login !=
   github.repository_owner`) e o diff tocar um item da lista, o job falha com uma
   mensagem que diz quais arquivos mudaram. O job só lê o diff (`git diff --name-only`
   e, no `Cargo.toml`, a linha `version` do pacote): não roda nada do PR, e roda com
   `permissions: contents: read`.
2. **Um `CODEOWNERS`** com o autor como dono desses arquivos. Com a proteção de
   branch pedindo a revisão dos donos, o GitHub não deixa o merge acontecer sem ele.
   Ligar essa proteção é feito nas configurações do repositório, pelo autor.

Além disso, ao revisar ou fazer o merge de um PR de outra pessoa, o Claude confere a
lista e avisa primeiro, mesmo que a pergunta seja sobre outra coisa.

As tags só são criadas pelo autor, e o `pages.yml` só roda na `main`. Um PR de fora
não consegue publicar o `latest.txt` sozinho, mas pode mudar como ele é feito. As
duas camadas pegam isso antes do merge.

## Como buscar: o `curl` do sistema

`curl --silent --fail --max-time 5 --max-filesize 64 --proto =https --tlsv1.2 <url>`

- É o mesmo caminho de git, ssh e rclone: um programa externo, sem cliente HTTP nem
  TLS dentro do Xylem (um `ureq` com `rustls` traria dezenas de crates).
- O `curl` existe no Linux, no macOS e no Windows 10 (1803) ou mais novo. Se não
  existir, a consulta falha em silêncio.
- **Sem shell:** `Command::new("curl")` com os argumentos separados. A URL é fixa no
  código, e nada do usuário entra nela.
- **No Windows**, o executável é procurado só em `System32` (`%SystemRoot%\System32\curl.exe`),
  para que um `curl.exe` na pasta atual não seja executado. No Unix, é procurado no
  `PATH`, como o git.
- A saída é lida com limite (`take(64)`), e o stderr é descartado.

## A resposta, como hostil

- Aceita no máximo 64 bytes. Tira espaços e `\n` das pontas.
- Precisa casar `^\d{1,4}\.\d{1,4}\.\d{1,4}$`. Qualquer outra coisa (HTML de um
  portal cativo, erro de proxy) é ignorada, sem mensagem.
- Só a versão já parseada (três números) é mostrada; o texto recebido nunca vai para
  a tela.
- O aviso só aparece se a versão recebida for **maior** que a do binário
  (comparação dos três números). Uma menor ou igual não diz nada.

## Quando consultar

- **Numa thread**, como o git status em segundo plano. A tela nunca espera.
- **No máximo uma vez por dia.** A data da última consulta e a versão achada ficam
  em `state_dir()/update-check` (duas linhas). A cada início, se a última consulta
  tem menos de 24 h, usa o guardado e não consulta.
- **Nunca** em execuções não interativas (`--man`, `--version`, `--help`), nem
  quando a saída não é um terminal.
- **Sem rede, com proxy recusando ou com o `curl` faltando,** nada aparece, e a
  próxima tentativa fica para o dia seguinte. Não há mensagem de erro.

## A pergunta na primeira execução

- A opção nova é `check_updates` em `[options]`. Ausente do arquivo significa "ainda
  não perguntado" (`Option<bool>` no `Config`; o exemplo comentado mostra a chave
  comentada).
- **Quando:** depois da tela de abertura (`splash`), numa caixa do tipo das
  confirmações (`ask_options`):

  > Check once a day for a new version of Xylem?
  > It asks andrelnfernandes.github.io for one line with the latest version number;
  > nothing about you or your files is sent. You can change this in F12.
  >
  > [Yes] [No]

- **Sim** grava `check_updates = true` (`Config::save`, que mantém o resto do
  arquivo) e faz a primeira consulta. **Não** grava `false`. **Esc** não grava nada,
  e a pergunta volta no próximo início.
- **Se o arquivo de configuração não puder ser gravado** (só leitura, erro de
  sintaxe), a resposta vale só para esta execução e a pergunta volta, sem ficar
  repetindo durante a sessão.
- **Uma variável de ambiente `XYLEM_CHECK_UPDATES=0`** (ou `1`) responde sem
  perguntar e tem prioridade sobre o arquivo. Serve para instalação em massa e para
  os testes.

## O aviso

- Na linha de cima, à direita, ao lado de `Xylem`: `0.10.0 available`, numa cor
  discreta. Ele fica enquanto o Xylem estiver aberto e some quando a versão rodando
  for a nova.
- O `?` (Info) mostra uma linha com a versão nova e o endereço das releases
  (`https://github.com/andrelnfernandes/xylem/releases`). O endereço é fixo no
  código, não vem da resposta.
- Não há janela, nem som, nem repetição: é só a linha.

## Para quem empacota

Uma **feature de compilação** `update-check`, ligada por padrão. Sem ela, não há
pergunta, nem consulta, nem opção no F12. Os pacotes de distribuição costumam exigir
isso; o `.deb` e o `.rpm` do próprio projeto continuam com ela ligada.

## F12

`check_updates` aparece na página de opções como as outras booleanas. Ao ligar, a
consulta é feita na hora (respeitando o "uma vez por dia").

## Testes

- O parser da resposta: versões válidas; com `\r\n`; vazio; HTML; `0.10.0-rc1`;
  números longos demais; 65 bytes; sequências de escape no meio. Só os válidos
  passam.
- A comparação: `0.9.0 < 0.10.0`, igual e menor não avisam.
- O estado de um dia: consultado há 23 h não consulta, há 25 h consulta, e um
  arquivo de estado corrompido conta como "nunca consultou".
- A pergunta: sem a chave aparece; Sim, Não e Esc gravam `true`, `false` e nada; com
  a variável de ambiente não aparece.
- A consulta usa um buscador trocável (`fn() -> Option<String>`), como o leitor do
  git status. Os testes nunca chamam o `curl` nem usam a rede.
- Um teste da linha de comando do `curl` (os argumentos exatos), sem executá-lo.

## Manual (só inglês)

- `security.md`: uma seção "Network": o que o Xylem acessa por si (só isto) e o
  que acessa por pedido (ssh, rclone, git).
- `configuration.md`: `check_updates` e `XYLEM_CHECK_UPDATES`.
- `start.md`: a pergunta da primeira execução.
- `releases.md`: o item na versão em que entrar.

## Tamanho

Umas 200 a 300 linhas com testes (`src/update.rs`, a pergunta, a linha de cima, a
opção), mais o `latest.txt` no `pages.yml`. Uma etapa só, um PR.

## Em aberto

- **?1 resolvido (2026-11-21): entra na 0.9.0.** A 0.9.0 ainda não tem tag; quem
  estiver nela será avisado da 0.10.0 em diante.
- **?2** Avisar também de uma versão de segurança (por exemplo, uma segunda linha
  `security` no `latest.txt`) com mais destaque. Proposta: não agora, para manter o
  formato de uma linha.

## Feito (2026-11-21)

Como acima, com estas diferenças:

- **O `latest.txt` vem da última release publicada**, não da última tag:
  `gh release view --json tagName` no `pages.yml`, conferida como `N.N.N` (por
  regex do bash, que não deixa passar uma segunda linha). Uma tag com a release
  ainda em rascunho não avisa ninguém. Não precisa de `fetch-depth: 0`.
- **Publicar uma release refaz o site:** `release-site.yml` (em `release:
  published`) roda o `pages.yml` na `main` (`gh workflow run`, com `actions:
  write`). O ambiente `github-pages` só aceita deploy da `main`, e o evento de
  release roda na tag.
- **O aviso:** em vez de uma linha no `?` (Info), uma mensagem na abertura,
  quando a versão nova é conhecida (`Xylem 0.10.0 is out: <releases>`), e
  `0.10.0 is out` na linha de cima, a partir de 90 colunas.
- **A pergunta** usa a confirmação de sempre (Yes/No no rodapé). Qualquer
  outra tecla fecha sem gravar, como Esc.
- **F12, página 3, `G`**: mostra `(not asked yet)` antes da resposta. Ligar
  pelo F12 consulta na hora (dentro do "uma vez por dia"), e desligar esquece a
  versão achada.
- **A versão é guardada como `checked=<segundos>` e `latest=<versão>`** em
  `state_dir()/update-check`. Um arquivo com mais de 256 bytes, ou que diga
  outra coisa, conta como "nunca consultou".
- **O `version-guard` tem um limite:** um PR roda a cópia dele do `ci.yml` e
  pode tirar o job. Por isso o `CODEOWNERS` cobre `.github/` inteiro, além do
  `Cargo.toml`, do `Cargo.lock` e do `site/build.sh`. Quem barra o merge é o
  `CODEOWNERS`, com "Require review from Code Owners" ligado na proteção da
  `main`, nas configurações do repositório.
- **Os testes** usam um buscador falso e um arquivo de estado numa pasta
  temporária. Os e2e rodam com `XYLEM_CHECK_UPDATES=0`.
