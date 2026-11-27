# Estatísticas e selo SSH no macOS e no FreeBSD

Origem: o teste do zoomosis no issue #148 (macOS) e o uso dele no FreeBSD via
SSH. Entra no branch `macos-opener`, junto com o item 1 (`open` como programa
padrão), num PR só.

## Intenção

- **Estatísticas (`?`):** no macOS e no FreeBSD, a seção SYSTEM mostra hoje só
  CPU (threads), shell e terminal. Tudo o mais vem de `/proc` e de
  `/etc/os-release`, que o macOS não tem. A tela deve mostrar o sistema, o
  kernel, a máquina, a CPU, o uptime, a carga, a memória e a memória do Xylem,
  como no Linux.
- **Selo SSH:** sem as variáveis `SSH_*` (o `sudo` as descarta), a busca do
  `sshd` entre os processos pais só existe no Linux. No macOS e no FreeBSD, o
  `sudo xy` numa sessão SSH mostra `ROOT` sem `SSH`.

## Não objetivos

- Outros Unix (NetBSD, OpenBSD, illumos): ficam como hoje.
- A seção NETWORK no macOS e no FreeBSD (o gateway vem de `/proc/net/route`).
- Swap no FreeBSD (exigiria `swapinfo` ou uma struct binária do `sysctl`).
- O Linux e o Windows não mudam.

## Abordagens

- **A. Programas do sistema (escolhida pelo autor):** `sysctl` e `ps`, com
  caminho absoluto, e leitura da saída em texto. Sem `unsafe`. Os leitores são
  funções puras, testadas no Linux.
- B. `sysctlbyname` e `proc_pidinfo` via FFI: os primeiros `unsafe` do lado
  Unix, structs que mudam de sistema para sistema, e o código do FreeBSD não
  seria compilado em lugar nenhum.
- C. Crate `sysinfo`: dependência grande, com o próprio `unsafe`, e mudaria
  também o caminho do Linux.

## Desenho

**Escolha do sistema quando o programa roda.** O código escolhe pelo
`std::env::consts::OS` (`"macos"`, `"freebsd"`), como o `default_opener_for`
do item 1, e não por `#[cfg]`. Assim o código do macOS e do FreeBSD é
compilado, passa pelo clippy e é testado no Linux, aqui e no CI. Só a execução
real nesses sistemas fica sem teste automático.

**Como rodar um programa** (`run_capped`, novo, em `src/platform/unix/bsd.rs`):

- O caminho é absoluto: `/usr/sbin/sysctl` no macOS, `/sbin/sysctl` no
  FreeBSD e `/bin/ps` nos dois. Não há shell nem busca no `PATH`.
- Os argumentos são fixos e o ambiente é limpo, com só `LC_ALL=C`. A entrada e
  a saída de erro vão para o nulo.
- A saída é lida até um limite (64 KiB para o `sysctl`, 4 MiB para o
  `ps -A`). Depois de 1 s o processo é encerrado, e o resultado é "nada".
- A saída vira texto com `from_utf8_lossy`. Qualquer falha deixa o campo vazio,
  como hoje.

**Estatísticas.** Uma chamada a `sysctl` com nomes, sem `-n`, porque um nome
desconhecido sai só na saída de erro e não desalinha os outros. Cada linha
`nome: valor` é separada no primeiro `": "`.

| Campo | macOS | FreeBSD |
|---|---|---|
| OS | `macOS ` + `kern.osproductversion` | `/etc/os-release` (o leitor de hoje); sem ele, `kern.ostype` + `kern.osrelease` |
| Kernel | `kern.ostype` + `kern.osrelease` ("Darwin 24.1.0") | o mesmo ("FreeBSD 14.1-RELEASE-p5") |
| Host | `kern.hostname` | `kern.hostname` |
| CPU | `machdep.cpu.brand_string` | `hw.model` |
| Uptime | agora − `kern.boottime` (`{ sec = N, usec = M } ...`) | o mesmo |
| Carga | `vm.loadavg` (`{ 1.23 1.10 0.98 }`) | o mesmo |
| Memória total | `hw.memsize` | `hw.physmem` |
| Memória disponível | (`vm.page_free_count` + `vm.page_pageable_external_count` + `vm.page_purgeable_count`) × `hw.pagesize` | (`vm.stats.vm.v_free_count` + `vm.stats.vm.v_inactive_count`) × `hw.pagesize` |
| Swap | `vm.swapusage` (`total = 2048.00M  used = ...  free = ...`) | nenhum |
| Memória do Xylem | `ps -o rss= -p <pid>` (KiB) | o mesmo |

Abrir as estatísticas roda dois processos: o `sysctl` e o `ps`.

**Selo SSH.** Uma chamada a `/bin/ps -A -o pid= -o ppid= -o ucomm=` lista os
processos. Esse formato corrige o da primeira versão deste desenho
(`-o pid=,ppid=,comm=`), apontado na revisão final:

- **Uma coluna por `-o`:** pelo POSIX, o texto depois do `=` é o título da
  coluna e vai até o fim do argumento. O FreeBSD segue isso à risca, e
  `-o pid=,ppid=,comm=` dá uma coluna só. O procps do Linux aceita os dois
  formatos.
- **`ucomm` e não `comm`:** no macOS, `comm` é o argv[0], que o sshd reescreve
  (`sshd: usuario@ttys000`). `ucomm` é o nome do executável guardado pelo
  kernel, com até 16 caracteres, o mesmo no FreeBSD e no Linux.

- **Leitura de cada linha:** pid, ppid e o resto como nome. O nome pode ter
  espaços. Se algum `ps` der um caminho, vale o que vem depois da última `/`.
- **Linhas descartadas:** as que não começam com dois números. Um pid repetido
  fica com a primeira linha.
- **Busca:** o `ancestors_with` de hoje passa a receber uma função
  `pid → (nome, ppid)`. O Linux continua lendo `/proc/<pid>/stat` com o
  `parse_stat`; o macOS e o FreeBSD consultam a lista do `ps`. Os limites
  continuam os mesmos: para no pid 1, num laço ou em 64 passos.
- **Quando roda:** só sem as variáveis `SSH_*`, uma vez na abertura do Xylem e
  nunca nos testes. Custa alguns milissegundos.

**Nomes hostis.** Os nomes de processos são escolhidos por quem os roda. Eles
só são comparados com `sshd`, `sshd-session` e `mosh-server`, e só ao longo da
cadeia de pais do próprio Xylem. O tamanho da saída e o tempo têm limite.

## Decisões minhas (para o autor conferir)

1. A memória disponível do macOS é uma aproximação: livre + cache de arquivos +
   purgável. O mesmo espírito do `MemAvailable` do Linux. O zoomosis pode
   comparar com o Monitor de Atividade.
2. Não há swap no FreeBSD.
3. O FreeBSD usa o `/etc/os-release` que o sistema já tem (FreeBSD 13 ou mais
   novo) e o leitor que já existe.
4. Um arquivo novo, `src/platform/unix/bsd.rs` (submódulo de `unix`, para
   reaproveitar o `ancestors_with` e o `os_release_name` sem torná-los
   públicos), guarda o `run_capped`, os leitores do `sysctl` e do `ps` e a
   montagem do `System`. O `unix.rs` só despacha pelo nome do sistema.
5. O macOS e o FreeBSD entram no manual da tela de estatísticas e na nota de
   versão da 0.10.0.

## Testes

- **Leitores, com saídas reais de exemplo dos dois sistemas:**
  - linhas `nome: valor`, um nome ausente e um valor com `:` (a data do
    `kern.boottime`);
  - `kern.boottime`, `vm.loadavg` e `vm.swapusage`;
  - as linhas do `ps`, com nomes com espaço, caminho inteiro e lixo;
  - a montagem do `System` a partir do texto, para `"macos"` e `"freebsd"`.
- **`run_capped` no Linux:** com programas reais, a saída normal, o corte no
  limite (`/bin/sh -c 'yes'`) e o tempo esgotado (`/bin/sleep 5` com limite
  curto), que termina rápido e sem processo zumbi.
- **`ancestors_with`:** os testes de hoje passam a usar a nova forma da função.
- **macOS:** o job do macOS no CI compila o código.
- **Execução real:** pedir ao zoomosis que confira no Mac e no FreeBSD.
