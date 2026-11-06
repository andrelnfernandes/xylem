# Armazenamento na nuvem (rclone)

O Xylem não tem um cliente próprio para Google Drive ou S3, mas não precisa de
um: o [rclone](https://rclone.org) monta esses serviços (e cerca de 70 outros:
OneDrive, Dropbox, Backblaze B2, Cloudflare R2, MinIO...) como uma pasta, e o
Xylem trabalha nessa pasta como em qualquer outra. Tudo funciona ali: leitura
de diretórios, Branch, marcações, visualizador, Copiar e Mover nas duas
direções, Renomear, Criar diretório e Excluir.

## Por que rclone

Optamos por trabalhar com o rclone em vez de conversar diretamente com o Google
Drive e o Amazon S3. Código próprio para cada serviço seria mais um lugar para
erros, e o Google Drive também traria junto as políticas do Google: um
aplicativo que lê todos os arquivos de alguém precisa de verificação do Google
e de uma avaliação de segurança anual, ou então exibe um aviso de "aplicativo
não verificado" e fica limitado a poucos usuários. Para o S3, a biblioteca
oficial é muito grande e traria muita complexidade, o que o Xylem, por projeto,
evita sempre que pode. O rclone já faz tudo isso muito bem, para vários
serviços de uma só vez.

## Configuração inicial (uma única vez)

Instale o rclone (`apt install rclone`, `dnf install rclone`, `brew install
rclone` ou em [rclone.org](https://rclone.org/downloads/)) e crie um *remote*
com `rclone config`. Você dá um nome a ele, que é o que vai antes dos dois
pontos abaixo.

- **Google Drive:** tipo de armazenamento `drive`; os padrões servem. O rclone
  abre o navegador para você entrar com sua conta do Google e autorizar. Em uma
  máquina sem navegador (via SSH), responda *no* à pergunta sobre o navegador:
  o rclone mostra um comando para rodar com `rclone authorize` em uma máquina
  que tenha navegador.
- **Amazon S3:** tipo de armazenamento `s3`, provedor `AWS`, e depois a chave
  de acesso e o segredo, ou *env_auth* para usar o que as ferramentas da AWS já
  usam (`~/.aws/credentials`, variáveis de ambiente, a *role* da instância).
  Outros serviços S3 (R2, MinIO, Wasabi...) aparecem como outros provedores na
  mesma lista.

`rclone lsd gdrive:` (ou `rclone lsd s3:`) lista as pastas da raiz (os
*buckets*, no S3): se funcionar, o remote está pronto.

## Montar e abrir

No Linux, com um remote chamado `gdrive`:

```bash
mkdir -p ~/gdrive
rclone mount gdrive: ~/gdrive --vfs-cache-mode writes --daemon
xy ~/gdrive
```

Para o S3, faça o mesmo com `s3:` (todos os buckets) ou `s3:meu-bucket` (apenas
um). `fusermount -u ~/gdrive` desmonta.

- `--vfs-cache-mode writes` mantém uma cópia local dos arquivos que estão sendo
  gravados, o que editores (`E`) e alguns programas exigem.
- `--dir-cache-time 1h` mantém as listagens de diretórios por mais tempo que os
  5 minutos padrão: reler (F3) fica mais rápido, mas alterações feitas fora dali
  demoram mais para aparecer.

No **macOS**, `rclone mount` precisa do [macFUSE](https://osxfuse.github.io);
`rclone nfsmount gdrive: ~/gdrive` não precisa de nada extra. No **Windows**,
precisa do [WinFsp](https://winfsp.dev) e monta em uma letra de unidade:
`rclone mount gdrive: X:`, e depois `xy X:\`.

Para ter a pasta sempre à mão, salve-a como um [favorito](bookmarks.md)
(Alt+B).

No Linux, o Xylem reconhece as pastas montadas pelo rclone: o painel DISK
mostra o serviço (Google Drive, Amazon S3, OneDrive...) e, abaixo dele, o
remote (`rclone gdrive:`). Info (Alt+I) e Avail (`A`) também os mostram. O
serviço vem do arquivo de configuração do rclone
(`~/.config/rclone/rclone.conf`, ou `$RCLONE_CONFIG`); quando ele está
criptografado, o painel diz apenas rclone.

## O que muda lá

**Espere lentidão em alguns momentos.** Tudo passa pelo rclone e pela rede do serviço, então
pode ser bem mais lento que um disco local, pela própria natureza desta integração. Ver um
arquivo, abrir um archive ou logar uma pasta grande pode demorar, e a tela pode não responder
até o serviço responder: em geral é espera, não travamento. No Google Drive, vale criar o seu próprio `client_id`: veja a seção sobre o
erro 403, mais abaixo.

- **Tudo passa pela rede.** Cada leitura de diretório é uma requisição, e um
  serviço como o Google Drive as recusa quando são muitas. Por isso, numa pasta
  montada, `*`, Branch (`B`), a visão Usage e o Compare em ramo pedem ao rclone o ramo
  inteiro numa só listagem, bem mais rápida, a não ser que ele já esteja todo
  lido; Esc deixa a árvore como estava. Se o rclone falhar ou mostrar pastas
  diferentes das da montagem, eles leem pasta por pasta. Só `*` expande a árvore.
- **A tela não espera.** A árvore aparece na hora e se preenche conforme as pastas
  chegam, também ao abrir uma (→ ou `+`); enquanto isso, a janela de arquivos
  mostra `reading...`. F3 e ir a um caminho digitado ainda
  esperam a sua leitura.
- **Alterações feitas em outro lugar** (no navegador, no celular) só aparecem
  após F3 e depois que o cache de diretórios expira (`--dir-cache-time`).
- **Excluir:** responda `P` (permanentemente) em vez de `T`: no Linux, a
  lixeira de uma pasta montada é uma pasta `.Trash-` (com o número do seu
  usuário) na raiz dela, de modo que o arquivo continuaria no remote ocupando
  espaço. O Google Drive já guarda o que o rclone exclui na própria lixeira do
  Drive; no S3, um arquivo excluído some de vez, a menos que o bucket tenha
  versionamento.
- **Permissões e donos** não são mantidos: a montagem exibe os mesmos para
  todos os arquivos, e o comando de Atributos não pode alterá-los. As datas são
  mantidas.
- **Git:** a lista de arquivos não mostra as letras de status numa pasta montada, porque o
  `git status` lê o repositório inteiro pela rede. Ctrl+G continua funcionando ali.
- **Google Docs, Planilhas e Apresentações** não são arquivos comuns: o rclone
  os mostra exportados (`.docx`, `.xlsx`, `.pptx`), com tamanho 0. A opção
  `--drive-skip-gdocs` os oculta.
- **O S3 não tem diretórios reais**, apenas nomes com `/` neles: um diretório
  vazio pode desaparecer quando desmontado, e renomear ou mover um diretório
  copia e exclui cada arquivo dentro dele.

## Compartilhamentos de rede (SMB, WSL)

A árvore também lê em segundo plano numa pasta compartilhada pela rede: SMB
(`cifs`), SSHFS, WebDAV (`davfs`) e, no WSL, um compartilhamento do Windows
montado pelo caminho UNC:

```
sudo mount -t drvfs '\\servidor\compartilhamento$' /mnt/compartilhamento
```

Lá, como numa montagem do rclone, a lista de arquivos não mostra as letras do Git
(Ctrl+G continua funcionando), o que muda em outro lugar aparece depois do F3, e o
Treespec não espera uma pasta que está sendo lida: o cursor segue o que você
digita conforme as pastas chegam. `/`, Tab e Enter continuam esperando a sua
leitura. `*`, Branch e Usage leem pasta por pasta, com progresso e Esc.

Não reconhecido: um compartilhamento mapeado numa letra de unidade no Windows
(`mount -t drvfs Z: /mnt/z`) parece um disco local visto do Linux; monte-o pelo
caminho UNC. NFS é lido como disco local.

## Quando o serviço recusa: erro 403

O Google Drive limita quantos pedidos aceita por minuto. Em um Drive
congestionado de pequenos arquivos e pastas (backups de projetos, `node_modules`,
extensões de editor), **erros 403 são comuns**: `Quota exceeded`,
`rateLimitExceeded`. O rclone espera e tenta de novo, então tudo fica lento, e no
fim ele pode desistir. Qualquer gerenciador de arquivos que leia pasta por pasta
pela montagem esbarra no mesmo limite.

O que o Xylem faz: o `*` mostra quantos pedidos foram recusados e, depois de 100,
pergunta se você quer continuar tentando ou parar; parar deixa a árvore como
estava. Se o rclone desistir, o Xylem avisa e não passa a ler pasta por pasta
sozinho, porque isso só seria mais recusado.

O que ajuda, começando pelo primeiro:

- **Logar por pastas.** Em vez de `*` na raiz, use `*` em pastas menores, ou abra-as
  com → ou `+`. Em um Drive congestionado, costuma ser o melhor caminho.
- **Seu próprio `client_id`.** Se você não criou um, o rclone usa um cliente
  dividido com os outros usuários dele, e a cota também. Criar o seu é grátis: o
  [guia do rclone](https://rclone.org/drive/#making-your-own-client-id) explica
  como. Depois, autorize de novo com `rclone config`. Os limites da sua conta
  continuam, mas o uso dos outros deixa de contar contra você.
- **Guardar as listagens por mais tempo:** `--dir-cache-time 1h` no `rclone mount`.
- **Remover o que pode ser recriado.** Menos pastas significam menos pedidos, em
  qualquer programa.

## Descobrir o que está enchendo o seu Google Drive

Quando o Drive vive pedindo um plano de armazenamento maior, o Xylem pode
mostrar para onde está indo o espaço antes de você pagar por mais. Monte o
Drive, vá até a raiz dele, leia tudo com `*` (em um Drive grande isso leva um
tempo) e escolha Alt+K → **Percent** ([Tamanhos de diretórios](info.md)): cada
pasta mostra quanto ocupa da pasta acima, com uma barra proporcional à maior
entre as pastas irmãs. Siga as maiores barras até chegar aos backups antigos,
vídeos e arquivos duplicados.

A [visualização de Uso](usage.md) (Alt+W, na raiz do Drive) vai além: junta
arquivos e pastas, do maior para o menor, para que um único arquivo enorme se
destaque tanto quanto uma pasta cheia. E o Branch (`B`) lista os arquivos de
uma pasta inteira: [ordenados](sort.md) por tamanho, do maior para o menor
(Alt+S, depois `O` para decrescente e `S` para Size), os maiores ficam no topo.

Algumas coisas contam na cota de armazenamento sem aparecer na montagem:

- **A lixeira do Drive** ocupa espaço até ser esvaziada (ou até passarem 30
  dias). `rclone about gdrive:` mostra o total, o usado e o que está na
  lixeira; montar com `--drive-trashed-only` mostra apenas o que está na
  lixeira.
- **Gmail e Google Fotos** compartilham o mesmo armazenamento, e somente os
  arquivos do Drive aparecem na montagem. Se `rclone about` mostrar muito mais
  espaço usado do que a soma da montagem, o restante provavelmente está lá.
- **Google Docs, Planilhas e Apresentações** aparecem com tamanho 0, mas os
  mais recentes contam no armazenamento.
- Arquivos **compartilhados com você** não contam: eles consomem o
  armazenamento do dono.
