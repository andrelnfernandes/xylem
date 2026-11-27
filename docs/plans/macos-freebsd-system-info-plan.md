# Estatísticas e selo SSH no macOS e no FreeBSD: plano

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** no macOS e no FreeBSD, a tela de estatísticas mostra o sistema e o
selo SSH sobrevive ao `sudo`, lendo `sysctl` e `ps` sem `unsafe`.

**Architecture:** um submódulo novo, `src/platform/unix/bsd.rs`, tem um
executor de programas com limites (`run_capped`) e leitores puros das saídas
do `sysctl` e do `ps`. O `unix.rs` escolhe o caminho pelo
`std::env::consts::OS`, quando o programa roda, e não por `#[cfg]`. Assim todo
o código é compilado e testado no Linux.

**Tech Stack:** Rust 2024, `std::process::Command`, `std::sync::mpsc`. Nenhuma
dependência nova.

**Spec:** `docs/plans/macos-freebsd-system-info.md`

## Global Constraints

- Nenhum `unsafe` no lado Unix.
- Caminhos absolutos: `/usr/sbin/sysctl` (macOS), `/sbin/sysctl` (FreeBSD),
  `/bin/ps` (os dois). Sem shell, sem `PATH`.
- Ambiente limpo, só `LC_ALL=C`. Entrada e saída de erro para o nulo.
- Limites: 64 KiB para o `sysctl`, 4 MiB para o `ps -A`, 1 KiB para o
  `ps -o rss=`. Tempo máximo de 1 s por processo, e depois o processo é
  encerrado e esperado.
- A escolha do sistema é feita pelo `std::env::consts::OS` (`"macos"`,
  `"freebsd"`); qualquer outro sistema segue o caminho de hoje. O Linux e o
  Windows não mudam.
- Qualquer falha deixa o campo `None`, sem mensagem.
- Antes de cada commit: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`
  e `cargo test`. Commits com o e-mail noreply e a linha
  `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Sem push.
- Arquivos temporários só em `~/.cache/xt-dev/`. Texto do repositório em
  inglês.

## Review Focus

1. **Um nome do `sysctl` desconhecido nesta versão** (macOS antigo sem
   `kern.osproductversion`): os outros campos continuam preenchidos e nenhum
   valor vai para o campo errado. É o teste `missing_names_leave_only_their_fields_empty`
   (Task 2).
2. **Um `ps -A` cortado no limite no meio de uma linha:** a linha parcial não
   cria um processo com nome vazio, e a busca para sem travar. É o teste
   `parse_ps_ignores_lines_without_a_name` (Task 1).
3. **Um programa que não termina:** a abertura do Xylem e a tela de
   estatísticas esperam no máximo 1 s por processo e não deixam zumbis. É o
   teste `run_capped_gives_up_after_the_limit` (Task 1).
4. **Formatos fora do esperado:** um `kern.boottime` sem `usec`, um
   `vm.swapusage` em `G` e um `boottime` no futuro (uptime `None`, sem
   underflow). São os testes de leitor da Task 2.
5. **O Linux não muda:** o `system()` e o `ancestor_names()` do Linux
   continuam lendo `/proc`. São o teste existente `ancestor_names_reads_this_process`
   e o novo `linux_system_still_reads_proc` (Task 2).

---

### Task 1: Selo SSH no macOS e no FreeBSD (`ps`)

**Files:**
- Create: `src/platform/unix/bsd.rs`
- Modify: `src/platform/unix.rs` (`mod bsd;`, `ancestor_names`, `ancestors_with`, as constantes e o `parse_stat` deixam de ser só do Linux)

**Interfaces:**
- Produces:
  - `pub(super) fn run_capped(program: &Path, args: &[&str], cap: u64, limit: Duration) -> Option<String>`
  - `fn parse_ps(text: &str) -> HashMap<u32, (String, u32)>` (pid → (nome, ppid))
  - `pub(super) fn ancestor_names(start: u32) -> Vec<String>`
  - Em `unix.rs`: `fn ancestors_with(start: u32, lookup: impl Fn(u32) -> Option<(String, u32)>) -> Vec<String>`

- [ ] **Step 1: Escrever os testes que falham** (em `bsd.rs`, `#[cfg(test)]`)

```rust
#[test]
fn run_capped_returns_what_the_program_prints() {
    let out = run_capped(Path::new("/bin/sh"), &["-c", "echo a; echo b"], 1024, Duration::from_secs(5));
    assert_eq!(out.as_deref(), Some("a\nb\n"));
}
#[test]
fn run_capped_stops_at_the_cap() {
    let start = Instant::now();
    let out = run_capped(Path::new("/usr/bin/yes"), &[], 10, Duration::from_secs(5)).unwrap();
    assert_eq!(out.len(), 10);
    assert!(start.elapsed() < Duration::from_secs(3));
}
#[test]
fn run_capped_gives_up_after_the_limit() {
    let start = Instant::now();
    assert_eq!(run_capped(Path::new("/bin/sleep"), &["5"], 1024, Duration::from_millis(200)), None);
    assert!(start.elapsed() < Duration::from_secs(3));
}
#[test]
fn run_capped_passes_only_lc_all() {
    // env_clear also removes PATH, so the programs are called by full path.
    let env = run_capped(Path::new("/usr/bin/env"), &[], 4096, Duration::from_secs(5)).unwrap();
    assert!(env.lines().any(|l| l == "LC_ALL=C"), "{env}");
    assert!(!env.lines().any(|l| l.starts_with("HOME=")), "{env}");
}
#[test]
fn run_capped_without_the_program_is_none() {
    assert_eq!(run_capped(Path::new("/nonexistent/ps"), &[], 1024, Duration::from_secs(1)), None);
}
#[test]
fn parse_ps_reads_pid_parent_and_name() {
    let t = parse_ps("    1     0 /sbin/launchd\n  612     1 /usr/sbin/sshd\n  640   612 sshd-session\n  700   640 -zsh\n  800   700 /Applications/Visual Studio Code.app/Contents/MacOS/Electron\n  810   700 Google Chrome Helper\n");
    assert_eq!(t[&612], ("sshd".into(), 1));
    assert_eq!(t[&640], ("sshd-session".into(), 612));
    assert_eq!(t[&700], ("-zsh".into(), 640));
    assert_eq!(t[&800], ("Electron".into(), 700));
    assert_eq!(t[&810], ("Google Chrome Helper".into(), 700));
}
#[test]
fn parse_ps_ignores_lines_without_a_name() {
    let t = parse_ps("garbage\n  900\n  901   1\n  902   1 /\nx 1 sshd\n  903   1 ok\n  90");
    assert_eq!(t.len(), 1);
    assert_eq!(t[&903], ("ok".into(), 1));
}
#[test]
fn parse_ps_keeps_the_first_of_a_repeated_pid() {
    let t = parse_ps("  5  1 first\n  5  1 second\n");
    assert_eq!(t[&5].0, "first");
}
#[test]
fn the_walk_over_ps_finds_sshd() {
    let t = parse_ps("    1     0 /sbin/launchd\n  612     1 /usr/sbin/sshd\n  640   612 sshd-session\n  700   640 -zsh\n");
    assert_eq!(super::ancestors_with(700, |pid| t.get(&pid).cloned()), ["-zsh", "sshd-session", "sshd"]);
}
#[test]
fn ancestor_names_runs_the_real_ps() {
    // Linux also has /bin/ps (procps), with the same options.
    if !Path::new("/bin/ps").exists() || std::os::unix::process::parent_id() <= 1 {
        return;
    }
    assert!(!ancestor_names(std::os::unix::process::parent_id()).is_empty());
}
```

Em `unix.rs`, o teste existente `ancestors_stop_at_init_at_a_loop_and_at_64`
passa a dar ao `ancestors_with` um `lookup` que devolve `(nome, ppid)`. Os
testes do `parse_stat` não mudam.

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test --bin xy -- bsd:: ancestors_`
Expected: erro de compilação (`run_capped`, `parse_ps` e `ancestor_names` não existem; `ancestors_with` com outra forma).

- [ ] **Step 3: Implementar**

- **`run_capped`:**
  - `Command::new(program)` com `.args(args)`, `.env_clear()`, `.env("LC_ALL", "C")`, `stdin` e `stderr` nulos e `stdout` em pipe.
  - Uma thread lê `stdout.take(cap)` e manda o `Vec` por um `mpsc`; a espera usa `recv_timeout(limit)`.
  - No tempo esgotado: `kill`, `wait` e `None`. No sucesso, ou quando a saída chega ao limite: `kill` (sem efeito se já terminou), `wait` e `Some(from_utf8_lossy)`.
  - Uma falha no `spawn` dá `None`.
- **`parse_ps`:** cada linha passa por `trim_start`; o pid e o ppid são os dois primeiros campos separados por espaço, e o resto, com `trim`, é o nome. Vale o que vem depois da última `/`. Uma linha sem os dois números ou com nome vazio é descartada; um pid repetido fica com a primeira.
- **`ancestor_names(start)`:** `run_capped("/bin/ps", ["-A", "-o", "pid=,ppid=,comm="], 4 MiB, 1 s)` e depois `parse_ps` e `super::ancestors_with(start, |pid| tabela.get(&pid).cloned())`.
- **`unix.rs`:**
  - `ancestors_with` recebe o `lookup`.
  - O `ancestor_names()` público despacha por `std::env::consts::OS`: `"linux"` e `"android"` vão para `/proc`, com o `parse_stat` dentro do `lookup`; `"macos"` e `"freebsd"` vão para `bsd::ancestor_names(parent_id())`; os demais dão `Vec::new()`.
  - `STAT_MAX`, `ANCESTORS_MAX`, `parse_stat` e `ancestors_with` perdem o `#[cfg(target_os = "linux")]`. Os testes que leem o `/proc` de verdade continuam só no Linux.

- [ ] **Step 4: Rodar e ver passar**

Run: `cargo test --bin xy -- bsd:: ancestors_ parse_stat ancestor_names`
Expected: PASS em todos.

- [ ] **Step 5: Commit**

`fmt`, `clippy` e `cargo test` completos; depois:

```bash
git add src/platform/unix.rs src/platform/unix/bsd.rs docs/plans/macos-freebsd-system-info.md docs/plans/macos-freebsd-system-info-plan.md
git commit -m "SSH badge under sudo on macOS and FreeBSD, from ps"
```

### Task 2: Estatísticas no macOS e no FreeBSD (`sysctl`)

**Files:**
- Modify: `src/platform/unix/bsd.rs`, `src/platform/unix.rs` (`system()` despacha)

**Interfaces:**
- Consumes: `run_capped` (Task 1).
- Produces:
  - `fn parse_sysctl(text: &str) -> HashMap<&str, &str>`
  - `fn boottime_secs(v: &str) -> Option<u64>`
  - `fn loadavg(v: &str) -> Option<[f64; 3]>`
  - `fn swapusage(v: &str) -> Option<Memory>`
  - `fn system_from(os: &str, sysctl: &str, os_release: Option<&str>, rss: Option<&str>, now_secs: u64) -> System`
  - `pub(super) fn system(os: &str) -> System`
  - `fn sysctl_names(os: &str) -> &'static [&'static str]`
  - `fn sysctl_path(os: &str) -> &'static str`

- [ ] **Step 1: Escrever os testes que falham**

Amostras, como constantes de teste:

```rust
const MAC: &str = "kern.ostype: Darwin\nkern.osrelease: 24.1.0\nkern.osproductversion: 15.1\nkern.hostname: mac-mini.local\nmachdep.cpu.brand_string: Apple M1\nkern.boottime: { sec = 1764000000, usec = 123456 } Mon Nov 24 16:00:00 2025\nvm.loadavg: { 1.23 1.10 0.98 }\nhw.memsize: 17179869184\nhw.pagesize: 16384\nvm.page_free_count: 1000\nvm.page_pageable_external_count: 2000\nvm.page_purgeable_count: 500\nvm.swapusage: total = 2048.00M  used = 1024.50M  free = 1023.50M  (encrypted)\n";
const BSD: &str = "kern.ostype: FreeBSD\nkern.osrelease: 14.1-RELEASE-p5\nkern.hostname: fbsd\nhw.model: Intel(R) Xeon(R) CPU E5-2680 v4 @ 2.40GHz\nkern.boottime: { sec = 1764000000, usec = 0 } Mon Nov 24 16:00:00 2025\nvm.loadavg: { 0.10 0.20 0.30 }\nhw.physmem: 8589934592\nhw.pagesize: 4096\nvm.stats.vm.v_free_count: 100000\nvm.stats.vm.v_inactive_count: 50000\n";
```

Testes:
- `parse_sysctl_splits_at_the_first_colon_space`: em `MAC`, `kern.boottime` vale `"{ sec = 1764000000, usec = 123456 } Mon Nov 24 16:00:00 2025"`.
- `boottime_reads_sec`: `boottime_secs("{ sec = 1764000000, usec = 123456 } Mon ...") == Some(1764000000)`, `boottime_secs("{ sec = 42 }") == Some(42)`, `boottime_secs("garbage") == None`.
- `loadavg_reads_three_numbers`: `loadavg("{ 1.23 1.10 0.98 }") == Some([1.23, 1.10, 0.98])`, `loadavg("{ 1.0 }") == None`.
- `swapusage_reads_total_and_free`: `"total = 2048.00M  used = 1024.50M  free = 1023.50M  (encrypted)"` dá `Memory { total: 2048 MiB, available: 1023.5 MiB }`; `"total = 2.00G  used = 0.00M  free = 2.00G"` dá `total` e `available` de 2 GiB.
- `macos_system_from_sysctl`: `system_from("macos", MAC, None, Some("  12345\n"), 1764000000 + 3600)` dá `os == "macOS 15.1"`, `kernel == "Darwin 24.1.0"`, `host == "mac-mini.local"`, `cpu == "Apple M1"`, `uptime == 3600 s`, `load == [1.23, 1.10, 0.98]`, `memory == { total: 17179869184, available: 3500 * 16384 }`, `swap` como acima, `xt_memory == 12345 * 1024`.
- `freebsd_system_from_sysctl`: com `os_release = Some("NAME=FreeBSD\nPRETTY_NAME=\"FreeBSD 14.1-RELEASE-p5\"\n")` dá `os == "FreeBSD 14.1-RELEASE-p5"`; com `None` dá o mesmo valor, vindo de `kern.ostype` + `kern.osrelease`. Ainda: `kernel == "FreeBSD 14.1-RELEASE-p5"`, `cpu == "Intel(R) Xeon(R) CPU E5-2680 v4 @ 2.40GHz"`, `memory == { total: 8589934592, available: 150000 * 4096 }`, `swap == None`.
- `missing_names_leave_only_their_fields_empty`: `MAC` sem a linha `kern.osproductversion` e sem `hw.pagesize` dá `os == None` e `memory == None`, mas `kernel`, `cpu` e `load` preenchidos.
- `a_boottime_in_the_future_has_no_uptime`: `now_secs` menor que o boot dá `uptime == None`.
- `sysctl_names_and_paths`: `sysctl_path("macos") == "/usr/sbin/sysctl"`, `sysctl_path("freebsd") == "/sbin/sysctl"`, e cada nome lido por `system_from` para o sistema está em `sysctl_names(os)`. Verificar com uma lista no teste: `kern.osproductversion` e `machdep.cpu.brand_string` só no macOS, `hw.model` e `vm.stats.vm.v_free_count` só no FreeBSD.
- `linux_system_still_reads_proc` (em `unix.rs`, só no Linux): `system().kernel` começa com `"Linux "`.

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test --bin xy -- bsd:: linux_system`
Expected: erro de compilação (as funções não existem).

- [ ] **Step 3: Implementar**

- **`parse_sysctl`:** cada linha é separada no primeiro `": "`.
- **`system_from`:**
  - `memory` é `None` se faltar o total, o tamanho da página ou algum dos contadores.
  - A conta usa `checked_mul` e `checked_add`, e um estouro dá `None`.
  - O `os` do macOS é `format!("macOS {v}")`. No FreeBSD ele vem do `os_release_name` (do `unix.rs`) ou, sem ele, de `kern.ostype` + `kern.osrelease`.
- **`system(os)`:**
  - `run_capped(sysctl_path(os), sysctl_names(os), 64 KiB, 1 s)`.
  - No FreeBSD, `fs::read_to_string("/etc/os-release")`.
  - `run_capped("/bin/ps", ["-o", "rss=", "-p", &pid], 1 KiB, 1 s)`, depois `system_from(os, ..., now)`.
- **`unix.rs`:** o `system()` passa a `match std::env::consts::OS { "macos" | "freebsd" => bsd::system(OS), _ => <o corpo de hoje> }`.

- [ ] **Step 4: Rodar e ver passar**

Run: `cargo test --bin xy -- bsd:: linux_system`
Expected: PASS.

- [ ] **Step 5: Commit**

`fmt`, `clippy` e `cargo test` completos; depois:

```bash
git add src/platform/unix.rs src/platform/unix/bsd.rs
git commit -m "Statistics on macOS and FreeBSD, from sysctl"
```

### Task 3: Manual e nota de versão

**Files:**
- Modify: `docs/manual/info.md` e as cinco traduções (`pt`, `es`, `fr`, `de`, `zh`); `docs/manual/releases.md`

- [ ] **Step 1: Manual.** No item "Statistics (`?`)" do `info.md`, depois da
  frase sobre a máquina, acrescentar: "On macOS and FreeBSD the machine comes
  from `sysctl` (no swap on FreeBSD)." Traduzir nos outros cinco idiomas.
- [ ] **Step 2: Nota de versão.** Em `releases.md`, seção 0.10.0, **Fixed.**,
  junto da entrada do `open`, só em inglês:
  - "On macOS and FreeBSD, the statistics (`?`) show the system, kernel, CPU, uptime, load and memory, read from `sysctl`."
  - "On macOS and FreeBSD, the SSH badge stays under `sudo`: the parent processes are read with `ps`."
- [ ] **Step 3: Verificar.** Run: `cargo test`. Expected: PASS, porque os testes
  do manual e da ajuda leem esses arquivos.
- [ ] **Step 4: Commit**

```bash
git add docs/manual
git commit -m "macOS and FreeBSD statistics and SSH badge in the manual"
```
