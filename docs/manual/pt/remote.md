# Máquinas remotas (SSH)

O Xylem pode exibir os arquivos de outra máquina, acessada via SSH, como uma
árvore: sozinha, ou com esta máquina ao lado na [tela dividida](split.md).

## Como abrir

Digite o endereço em `L` ([Log](treespec.md)), ou passe-o para `xy` na linha de
comando:

- `user@host:` — o diretório pessoal (*home*); `user@host:/srv/www` — aquele
  diretório;
- `ssh://[user@]host[:port][/path]` — com porta, ou sem usuário (como no seu
  `~/.ssh/config`).

O Xylem executa o seu próprio `ssh`, portanto tudo o que você usa no terminal
funciona da mesma forma: `~/.ssh/config` (apelidos de host, portas, ProxyJump),
chaves, o agente e `known_hosts`. Não é preciso instalar nada na outra máquina:
o Xylem fala SFTP, que todo servidor SSH já possui (Linux, BSD, macOS,
Windows).

## Senhas e perguntas

Quando o ssh pergunta algo, a pergunta aparece no rodapé:

- uma **senha** ou a **frase secreta** (*passphrase*) de uma chave: digitada de
  forma oculta; Enter envia, Esc desiste da conexão. O Xylem a repassa ao ssh e
  não guarda nada.
- um **host desconhecido**: o host e a impressão digital (*fingerprint*) de sua
  chave, para conferir antes de responder `yes` (o ssh então o memoriza em
  `known_hosts`, como de costume).

`Esc` durante a conexão cancela. Com chave e agente, nada é perguntado.

![O ssh pedindo a senha de deploy@webserver no rodapé do Xylem](../../screenshots/remote-password.png)

## O que funciona

A árvore, a janela de arquivos, [Filespec](filespec.md), Branch, Showall,
marcações, [o visualizador](viewer.md), [Ctrl+S e Ctrl+V](search.md), Autoview,
Info (Alt+I), os modos de exibição (owner mostra os IDs numéricos de usuário e
grupo, que é o que o SFTP envia), [Comparar](compare.md) e Abrir (`O` baixa uma
cópia e a abre aqui).

**Copiar** traz os arquivos para cá, com suas permissões e datas (com precisão
de segundos, como o SFTP mantém); com a tela dividida, sugere a janela desta
máquina como destino. Alt+C (com caminhos) também funciona, com os caminhos
começando na raiz da janela remota.

![Tela dividida: três backups marcados na máquina remota, à esquerda, prestes a serem copiados para uma pasta desta máquina, à direita](../../screenshots/remote.png)

**Renomear** (um arquivo, um diretório ou os arquivos marcados com uma
máscara) e **Criar diretório** funcionam na máquina remota como aqui. Renomear
nunca substitui um arquivo sem perguntar; depois de um sim, o servidor o
substitui de uma vez, se puder (o OpenSSH pode), e senão o Xylem avisa e deixa
os dois.

**Copiar para a máquina remota** também funciona: da janela desta máquina;
com a tela dividida, a janela remota é o destino sugerido; ou digite um
caminho da janela remota, ou o endereço dela (`usuário@host:/srv/www`, dentro
da pasta em que a janela foi aberta). Cada arquivo é gravado num temporário ao
lado do destino, tem o tamanho conferido, recebe as permissões (sem setuid e
setgid) e a data, e só então vai para o lugar; Esc apaga o temporário. Copiar
dentro da máquina remota funciona do mesmo jeito. Links simbólicos não são
recriados lá, e isso é avisado.

**Mover** funciona em todas as direções: daqui para a máquina remota e de
volta (o original só é apagado depois que a cópia está no lugar e conferida), e
dentro da máquina remota (um renomear lá). **Excluir**, **Prune** e as mesmas
ações na Usage também funcionam. Numa máquina remota **não há lixeira**: a
pergunta diz isso, e o Oops não tem o que restaurar. Nada é apagado através de
um link simbólico: um link é apagado como ele mesmo. O Prune deixa de fora
outros discos montados dentro do ramo, distinguidos pela extensão `statvfs` do
OpenSSH; num servidor sem ela, o Prune é recusado.

O resto da escrita ainda não está disponível: Atributos, datas, Edit e eXecute. F3 relê um diretório; não há
atualização automática.

**F8** em uma janela remota abre uma janela nesta máquina (na pasta em que o
Xylem começou), permitindo ter ambas lado a lado.

## O terminal na máquina remota

**Alt+X** em uma janela remota abre um shell naquela máquina (`ssh -t`),
começando no diretório da janela; em uma janela local, abre o shell local, como
antes (cada um continua rodando enquanto o outro está na tela). No Linux e no
macOS, o shell remoto reutiliza a conexão já estabelecida, de modo que a senha
não é pedida novamente; no Windows, o ssh a pede no terminal.

![O shell da máquina remota sob a árvore dela, depois de cd logs: a árvore acompanhou até logs](../../screenshots/remote-terminal.png)

A árvore acompanha o `cd` do shell remoto quando o shell informa seu diretório
com a sequência OSC 7. Muitos sistemas já fazem isso; para o bash, adicione
isto ao `~/.bashrc` **na máquina remota**:

```bash
PROMPT_COMMAND='printf "\033]7;file://%s%s\007" "$HOSTNAME" "$PWD"'
```

O Xylem nunca digita comandos em um shell remoto (ele não consegue ver se há um
programa rodando lá), e Alt+O e `$XY_TAGGED` funcionam apenas em terminais
locais. Sair com um terminal remoto aberto pede confirmação antes.

## tmux ou screen iniciados no login

Algumas pessoas iniciam o tmux (ou o screen) automaticamente ao entrar via SSH,
a partir do `~/.bashrc` na máquina remota, para que uma queda de conexão (Wi-Fi
instável, VPN que reconecta) não encerre o que estava rodando: ao conectar
novamente, a mesma sessão é retomada. Por exemplo:

```bash
if [ -n "$SSH_CONNECTION" ] && [ -z "$TMUX" ]; then
    tmux attach-session -t remote 2>/dev/null || tmux new-session -s remote
fi
```

Nesse caso, o terminal remoto do Xylem também abre dentro do tmux. Tudo
funciona, exceto que **a árvore não acompanha o `cd` do shell**: o tmux fica
entre o shell e o Xylem, retém a sequência OSC 7 para si e não a repassa.

O shell remoto do Xylem inicia com `XY_TERMINAL=1` definido, assim como o
local, para que o login possa diferenciar os dois. Adicionar esse teste à
condição mantém o tmux para o seu `ssh` habitual e o deixa de fora quando o
Xylem se conecta:

```bash
if [ -n "$SSH_CONNECTION" ] && [ -z "$TMUX" ] && [ -z "$XY_TERMINAL" ]; then
    tmux attach-session -t remote 2>/dev/null || tmux new-session -s remote
fi
```

Dentro do Xylem, o shell não precisa do tmux para sobreviver: ele continua
rodando enquanto o Xylem está aberto e, se a conexão cair, a janela remota cai
junto de qualquer maneira.

## Containers (Docker, Podman)

Um container desta máquina abre do mesmo jeito, sem `sshd` dentro dele:

- `docker://container` — o diretório de trabalho dele; `docker://container/app` —
  esse diretório; `docker://usuário@container/app` — como esse usuário
  (`docker exec -u`).

O Xylem roda `docker exec` e, dentro do container, o `sftp-server` dele, o mesmo
programa que um servidor SSH usa. A maioria das imagens não o tem: instale-o no
container (`apk add openssh-sftp-server` no Alpine, `apt install
openssh-sftp-server` no Debian e no Ubuntu, `dnf install openssh-server` no
Fedora), ou o Xylem avisa que ele falta. A imagem precisa de `sh`, então imagens
distroless ficam de fora.

Tudo o que está acima funciona ali como numa máquina remota, e **Alt+X** abre um
shell no container, na pasta da janela. Com **Podman**, inicie o Xylem com
`XY_DOCKER=podman`. `DOCKER_HOST` e os contextos do docker valem como sempre.

O Xylem só roda `docker exec`: não inicia, não muda e não baixa container
nenhum. O que você escreve vai para o container, menos nas pastas montadas desta
máquina (`-v`): essas são arquivos desta máquina.

Para Google Drive, S3 e outros serviços de nuvem, veja
[Armazenamento na nuvem](cloud.md).
