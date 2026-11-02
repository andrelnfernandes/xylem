# Spec: escrever na máquina remota (SSH, fase 3)

Semanas 4 a 7 do [roadmap](roadmap.md). Origem: BACKLOG, Features, "Remote machines,
phase 3". Pedido do autor em 2026-11-02: começar a frente do SSH, importante para quem
mantém máquinas remotas. **Aprovado pelo autor em 2026-11-02, com `docker://` como
fatia própria depois da 3a.** A fase 4 (Edit/Open com
volta, refresh periódico, eXecute e Git por ssh) ganha spec próprio quando a 3 fechar.

## O que existe hoje

- `src/remote/mod.rs`: `Remote` fala SFTP v3 pelo `ssh -s <destino> sftp` do sistema
  (valem `~/.ssh/config`, chaves, agent, `known_hosts`). Só leitura: `realpath`, `stat`,
  `lstat`, `readlink`, `opendir/readdir`, `open/read/close`.
- `src/remote/sftp.rs`: o cliente. Pacote de resposta limitado a 1 MiB; nomes do
  servidor com `/`, `\0`, `.` ou `..` são recusados (`is_plain_name`).
- `src/ops/mod.rs`: `Operation` em passos, com conflito, cancelamento e arquivo
  temporário que só substitui o destino no fim. Fonte local, de archive ou remota
  (`reading_remote`, só Copy); **o destino é sempre local** (`std::fs` direto).
- `src/app/commands.rs`: `remote_writes` bloqueia Move, Delete, Oops, Rename,
  Attributes, MakeDir, Edit, Execute, TerminalTagged, Prune, NewDate, CreateArchive e
  Batch numa janela remota. `start_transfer` recusa destino remoto.
- Testes: `remote::for_tests` fala direto com o `sftp-server` do OpenSSH, sem rede;
  o e2e usa `XY_SSH` apontando para um wrapper.

## Modelo de ameaça

O servidor é hostil, como na leitura: pode mentir em `stat`, mandar status falso,
fechar a conexão no meio, trocar um arquivo por symlink entre dois pedidos, demorar
sem fim. O que ele **não** pode conseguir:

1. **Escrever ou apagar fora do que o usuário pediu na máquina local.** Toda escrita
   local continua passando pelo caminho atual (temporário + rename, `links`).
2. **Fazer o Xylem perder um arquivo local.** Move para o remoto só apaga o original
   depois de: WRITE de todos os blocos com status OK, CLOSE OK, `fstat` do
   temporário com o tamanho enviado, e o rename final OK. Mentir em tudo isso ainda é
   possível (o servidor guarda os dados, afinal); o manual diz isso com franqueza e
   sugere Copy + Delete para o que não pode se perder.
3. **Prender a tela ou esgotar memória.** Toda chamada de escrita roda dentro da
   `Operation` em passos (Esc cancela); um servidor que para de responder é cortado
   pelo `ServerAliveInterval`. Nada de resposta além de 1 MiB.
4. **Desviar um nome digitado.** Nome novo (Rename, Makedir) passa por
   `ops::check_name` e é um componente só; o caminho remoto é sempre montado por
   `remote_path`, componente por componente.

E o que vale para a própria máquina remota, que é do usuário:

- **Nunca seguir symlink ao apagar ou descer:** Prune e Delete usam `lstat` antes de
  cada `remove`/`rmdir`; um symlink é apagado como link, nunca o que ele aponta. O
  `Lister` remoto já põe symlinks entre os arquivos, e o `LogJob` não desce neles.
- **Escrita nunca através de um link:** o upload cria o temporário com
  `CREAT|EXCL` (falha se o nome já existir, link incluído) e só então renomeia sobre
  o destino. O mesmo `links` de hoje impede gravar dentro de um link criado pela
  própria cópia.
- **Sem setuid/setgid/sticky no upload:** o modo vai `& 0o777`, como `foreign_mode`.
- **Delete remoto é permanente:** não há lixeira; a pergunta diz "permanently" e
  Oops não se aplica.

## Desenho

### Cliente SFTP (`sftp.rs`)

Pedidos novos: `OPEN` com `WRITE|CREAT|EXCL` (e `TRUNC` só onde não há temporário),
`WRITE`, `FSTAT`, `SETSTAT`/`FSETSTAT` (modo e datas), `REMOVE`, `MKDIR`, `RMDIR`,
`RENAME`, `SYMLINK`. Ler as extensões do `VERSION`: com `posix-rename@openssh.com`, o
rename final é atômico e substitui o destino; sem ela (servidores não OpenSSH), o
destino aprovado no conflito é removido antes do `RENAME`, e isso fica dito no código.

Desempenho: SFTP é pedido-resposta. Com 32 KiB por ida e volta, 50 ms de latência dão
~640 KiB/s. O upload manda até 16 `WRITE` sem esperar resposta (a resposta de cada um
é conferida pelo id, fora de ordem permitido) e o download passa a fazer o mesmo com
`READ`. Limite de pedidos em voo fixo, sem crescer com o arquivo.

### Camada de sistema de arquivos sob `ops`

Um `enum Side { Local, Remote(Arc<Remote>) }` (não trait: são dois casos, e o `match`
deixa claro o que cada lado faz), com o que `Operation` usa hoje de `std::fs`:
`lstat`, `create_dir_all`, `rename`, `remove_file`, `remove_dir`, `create_tmp`
(devolve um `Box<dyn Write>` e o caminho), `finish` (modo, data, rename),
`symlink`, `read_link`. `Operation` passa a ter `src: Side` e `dst: Side`;
`reading_remote` vira um caso disso. Archive continua como fonte à parte.

Combinações:

| De → para | Copy | Move |
|---|---|---|
| local → remoto | upload | upload + apagar local (regra 2 acima) |
| remoto → local | download (já existe) | download + `REMOVE` remoto |
| remoto → mesmo remoto | download+upload em fluxo, sem disco | `RENAME` no servidor |
| remoto → outro remoto | fica de fora nesta fase | idem |

### Interface

- Destino remoto no prompt de Copy/Move: endereço digitado (`user@host:/x`) só se já
  houver uma janela conectada a ele; senão, a mensagem sugere abrir com `L` antes (a
  conexão, a senha e o `known_hosts` acontecem num lugar só). Com a tela dividida, a
  janela remota é sugerida como destino, como já ocorre com a local.
- Depois de cada operação, `rescan_everywhere` relê os diretórios tocados pelo
  `Lister` remoto (não há inotify).
- Os comandos saem de `remote_writes` conforme cada fatia fica pronta. Ficam
  bloqueados até a fase 4: Edit, Execute, TerminalTagged, CreateArchive, Batch.
- Remoto Windows (`/C:/...`): Attributes e modo octal não se aplicam e avisam.

## Fatias (uma por semana, cada uma sozinha no `main`)

1. **3a — base, Makedir e Rename.** *Feito na branch `ssh-write-3a`.* Pedidos `MKDIR`,
   `RENAME` e `posix-rename@openssh.com` (lido das extensões do `VERSION`); `Side` em
   `ops` (`exists`, `create_dir`, `rename`), com os testes antigos passando sem
   mudar; Makedir, Rename de arquivo e de pasta e Rename dos marcados (máscara) no
   remoto. Rename sem "sim" usa o `RENAME` do SFTP v3, que recusa destino existente;
   com "sim", o `posix-rename`, e sem ele o Xylem recusa e deixa os dois (o fallback
   de remover antes vem com o `REMOVE` da 3c). `SETSTAT` ficou para a 3b, onde é
   usado. Testes: contra o `sftp-server` e contra um servidor roteirizado (status
   EOF ou id trocado não viram sucesso; extensões malformadas são ignoradas).
2. **docker:// — containers sem sshd.** *Feito na branch `docker-remote` (sobre a
   `ssh-write-3a`).* Ver a seção no fim. Testado também com um Alpine real pelo
   Podman 5.7: erro sem `sftp-server`, Makedir, Rename e o shell do Alt+X. De quebra,
   o título da janela remota deixou de dizer "(read-only)".
3. **3b — enviar.** `OPEN` de escrita, `WRITE` em voo, `FSTAT`, `SETSTAT`; Copy local → remoto e
   remoto → mesmo remoto; Alt+C com caminhos; criar pasta de destino que falta;
   conflito com tamanho e data dos dois lados; Esc apaga o temporário remoto.
4. **3c — apagar e mover.** `REMOVE`, `RMDIR`; Delete (permanente), Prune, Move nas
   quatro combinações da tabela; Usage passa a apagar e podar no remoto.
5. **3d — permissões e datas.** Attributes (octal) e NewDate por `SETSTAT`;
   Duplicados por SSH, comparando por tamanho e depois pelo conteúdo lido pelo SFTP
   (não há hash no servidor sem rodar comando, que é fase 4).

Ao fim de cada fatia: manual (6 línguas) e "What works" de `docs/manual/remote.md`,
BACKLOG, e este spec com o status.

## Testes

- Contra o `sftp-server` real (`for_tests`): cada pedido novo; cada combinação de
  Copy/Move; Prune com symlink para fora da árvore (o alvo fica intacto); conflito;
  cancelamento no meio do upload sem sobrar temporário.
- Contra um **servidor falso hostil** em memória (o `Sftp` já é genérico em
  `Read`/`Write`): status OK com `fstat` de tamanho errado (Move não apaga o local),
  resposta fora de ordem e com id repetido, conexão fechada no meio do `WRITE`, nome
  com `/` vindo do `readdir` durante um Prune.
- e2e com `XY_SSH`: um Copy para o remoto e um Delete pela interface.

## Containers Docker (`docker://`, aprovado)

Funciona hoje, sem nada novo, quando o container tem `sshd`: `ssh://root@localhost:2222`
com a porta mapeada, o IP do container, ou um `Host` no `~/.ssh/config`. Sem `sshd`, a
fatia `docker://` resolve: o Xylem só precisa de um `sftp-server` falando por
stdin/stdout, que é o que os testes já fazem.

### Uso

- `docker://[usuário@]container[/caminho]` em `L` ou na linha de comando. Sem caminho,
  começa no diretório de trabalho do container. O usuário vai para `docker exec -u`.
- A raiz da janela aparece como `docker:container:/caminho` (ou
  `docker:usuário@container:/caminho`); o rodapé diz `Remote: docker:container`.
- Alt+X abre um shell no container, começando na pasta da janela.
- O programa é `docker`, ou `$XY_DOCKER` (Podman: `XY_DOCKER=podman`; os testes usam um
  wrapper). Vale o que o próprio CLI usa: `DOCKER_HOST`, contextos.

### Modelo de ameaça

O container é tão hostil quanto um servidor SFTP: a imagem pode ter vindo de qualquer
lugar, e o `sftp-server` dentro dela pode ter sido trocado. Tudo o que vale para a
leitura e a escrita por SFTP vale aqui, sem exceção, porque o código depois da conexão
é o mesmo.

O que é novo é **como** a conexão começa:

1. **Nada digitado chega a um shell.** O nome do container e o usuário passam pela
   mesma validação dos nomes de host (letras, dígitos, `-`, `.`, `_`, sem começar com
   `-`) e vão para o `docker` como argumentos próprios, depois de `--`. O script que
   procura o `sftp-server` é fixo, escrito no código; o caminho inicial vai pelo
   `realpath` do SFTP, e a pasta do terminal vai como `$1` do `sh -c`, nunca no texto
   do script.
2. **Só `docker exec`.** O Xylem nunca roda outro subcomando (`run`, `cp`, `pull`): não
   cria, não muda e não baixa container nenhum. Quem tem acesso ao socket do Docker é
   quase root na máquina; o Xylem usa esse acesso só para o `exec` pedido.
3. **Escrita fica dentro do container,** como o usuário do `exec`, com uma exceção que o
   manual diz: pastas montadas do host (`-v`) são arquivos do host, e escrever nelas
   pelo container é escrever no host.
4. **Erros do `docker` são texto de fora:** a última linha do stderr aparece no rodapé
   como já acontece com o ssh, passando pelo mesmo filtro de caracteres de controle.

### Desenho

- `Address` ganha `via: Via { Ssh, Docker }`; `docker://` não aceita porta.
- `Connecting::start` monta `docker exec -i [-u usuário] -- <container> sh -c <script>`,
  sem askpass (o `docker exec` não pergunta nada) e sem socket de controle.
- O script tenta, nesta ordem, `/usr/lib/openssh/sftp-server` (Debian, Ubuntu),
  `/usr/libexec/openssh/sftp-server` (Fedora, RHEL), `/usr/lib/ssh/sftp-server`
  (Alpine, Arch) e `/usr/libexec/sftp-server`; sem nenhum, termina dizendo "no
  sftp-server in the container (install openssh-sftp-server)". Exige `sh`: imagens
  distroless ficam de fora, e de qualquer forma não têm `sftp-server`.
- Terminal: `docker exec -it [-u usuário] -- <container> sh -c '<fixo>' sh <pasta>`, que
  entra na pasta e abre `$SHELL`, ou `bash` se existir, ou `sh`, com `XY_TERMINAL=1`.

### Testes

- Unitários: endereços (`docker://web`, `docker://root@web/app`, recusa de porta e de
  nome com `-` ou espaço) e os argumentos montados para conexão e terminal.
- Com um `docker` falso (wrapper que descarta tudo até o nome do container e roda o
  resto aqui): a conexão completa, a listagem e o erro sem `sftp-server`.
- Com um container de verdade (`#[ignore]`, como o teste do sshd): Alpine com
  `openssh-sftp-server`, por `XY_DOCKER` (Docker ou Podman).
