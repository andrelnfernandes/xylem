# Aviso de root e de sessão SSH

Desenho curto, 2026-11-27.

## Por quê

Duas situações pedem atenção e hoje não aparecem na tela:

- **root** (no Windows, processo elevado como Administrador): o xylem tem
  poder para apagar ou alterar o que quebra a máquina;
- **sessão SSH**: o xylem roda em outra máquina, não na do usuário.

O autor usa as duas de vez em quando. Por isso o aviso deve chamar a
atenção, sem medo de ser notado demais: o risco de virar ruído e ser
ignorado só existiria no uso diário.

## Objetivos

- Ver, sem procurar, que o xylem roda como root e/ou numa sessão SSH.
- Ver de novo no momento em que o estrago acontece: na confirmação de uma
  operação que altera ou apaga arquivos.
- Funcionar por padrão, sem configuração: na máquina remota normalmente não
  existe o `config.toml` do usuário.

## Fora do escopo

- Mudar o comportamento (bloquear, pedir confirmação a mais): só o visual.
- A janela conectada como `root@host` pela função SSH do próprio xylem
  (root na outra máquina). O selo dela continua como está. Pode vir depois,
  mas exige saber o usuário real da conexão, que pode vir do `~/.ssh/config`.
- Título da aba do terminal (OSC). Ficou como ideia para depois.

## Antiobjetivos

- Um aviso que some ou fica ilegível num tema (o `mono` não tem cores, o
  `light` já usa vermelho na borda ativa).
- Um aviso que o usuário consiga esconder por engano.
- Detecção que leia algo hostil e leve controle ou bidi para a tela.

## Abordagens consideradas

1. **Selo no cabeçalho**: barato, no padrão do selo remoto que já existe
   (`draw_header`, `src/ui.rs`). Sozinho, é uma linha só, e o olho se acostuma.
2. **Bordas vermelhas quando root**: a visão periférica pega o aviso, olhe o
   usuário para onde olhar. Só para root, que é o caso perigoso.
3. **Selo na confirmação de operações destrutivas**: aparece onde o estrago
   acontece.
4. Título da aba do terminal: útil com muitas abas, mas alguns terminais
   ignoram, e é preciso restaurar o título ao sair.
5. Tema próprio para root/SSH: impossível de não notar, mas depende de
   configuração, que não existe na máquina remota.

**Escolhidas pelo autor: 1, 2 e 3.**

## O desenho

### Detecção (uma vez, na partida)

- **root**: `geteuid() == 0` no Unix. No Windows, o token do processo está
  elevado (`TokenElevation`). O texto do selo é `ROOT` no Unix e `ADMIN` no
  Windows.
- **SSH**: `SSH_CONNECTION`, `SSH_CLIENT` ou `SSH_TTY` não vazios. Se nenhuma
  existir, no Linux, sobe a cadeia de processos ancestrais por `/proc/<pid>/stat`
  até o pid 1 procurando `sshd`, `sshd-session` (OpenSSH 9.8+) ou
  `mosh-server`. Isso cobre `sudo xy`, `sudo -i` e `su -`, que apagam as
  variáveis de SSH.
- **Nome da máquina** no selo SSH: o hostname até o primeiro `.`, sem
  caracteres de controle e com no máximo 24 caracteres.
- O resultado fica guardado no `App` (por exemplo `Session { admin, ssh:
  Option<String> }`), e os testes podem injetá-lo. A UI não lê estado global
  do processo.

Limitações conhecidas, para o manual: dentro do tmux/screen, as variáveis de
SSH são as da sessão que criou o servidor do tmux. No macOS, sem as
variáveis (sob `sudo`), a sessão SSH não é detectada.

### 1. Selo no cabeçalho

Os selos da sessão ficam na ponta esquerda, antes de `Path:`, porque são da
sessão e não do caminho. O selo da janela remota continua depois de `Path:`.

```
 ROOT  Path: /etc/nginx                              Xylem v0.10.0  2026-11-27 14:32
 SSH srv01  Path: ~/projetos                         Xylem v0.10.0  2026-11-27 14:32
 ROOT  SSH srv01  Path:  oracle@db01  /u01/app       Xylem v0.10.0  2026-11-27 14:32
```

- `ROOT`/`ADMIN`: texto na cor `background` sobre a cor `error` do tema, em
  negrito.
- `SSH <host>`: texto na cor `background` sobre a cor `special` do tema
  (magenta em todos os temas coloridos), em negrito.
- No tema `mono`, os dois selos usam vídeo invertido.
- Os selos têm prioridade sobre o caminho, que já é cortado pela esquerda
  (`fit_tail`). Em terminal estreito, o nome da máquina pode ser cortado.
  `ROOT` nunca é cortado.

### 2. Bordas vermelhas quando root

- Todas as janelas com borda (tela principal, visualizador, F12,
  favoritos, etc.) ficam com a borda na cor `error` do tema.
- A janela com o foco continua distinguível pela borda grossa
  (`BorderType::Thick`). As outras ficam com o traço normal do tema (simples
  ou arredondado), também em vermelho.
- O aviso continua valendo depois de trocar de tema no F12.
- Só para root. A sessão SSH fica só com o selo.

### 3. Selo na confirmação

O mesmo selo `ROOT`/`ADMIN` no início da linha da pergunta ou do prompt das
operações que alteram ou apagam o que já existe: Delete (arquivos e
diretório), Prune, Move, Copy (pode sobrescrever, inclusive na pergunta de
conflito), Rename (de um arquivo, de um diretório ou em lote), Attributes
(chmod) e New date.

```
 ROOT  DELETE 3 files? This cannot be undone.
       Yes  No  Trash
```

O selo não aparece quando a operação só escreve na máquina de uma janela
remota (SSH/docker), porque lá quem age é o usuário da conexão, não o root
local.

Nos prompts de texto (Move, Rename), o cursor é posicionado pelo
comprimento do rótulo. O selo precisa entrar nessa conta.

## Fica a cargo de quem implementar

- Onde aplicar a cor e o tipo das bordas (em `panel()` e nos outros pontos
  que leem `border`/`active_border`, ou num tema derivado na hora de
  desenhar), desde que não dependa de estado global nos testes e sobreviva à
  troca de tema.
- A API do Windows para elevação e para o nome da máquina (`windows-sys`,
  feature `Win32_Security` a acrescentar).
- `nix`: feature `hostname` para `gethostname`.

## Testes e documentação

- Unitários: a detecção como funções puras (ambiente e ancestrais como
  entrada), incluindo hostname hostil (controle, bidi, comprido).
- UI com `TestBackend`: os selos no cabeçalho (com e sem janela remota,
  estreito), as bordas com root em `light` e `mono`, e o selo nas
  confirmações (presente local, ausente só remoto).
- Manual: `screen.md` (o cabeçalho) e as traduções, `releases.md` (0.10.0,
  New).
