# Aviso de root e de sessão SSH: plano de implementação

> **Para agentes:** SUB-SKILL OBRIGATÓRIA: use superpowers:subagent-driven-development (recomendado) ou superpowers:executing-plans para executar este plano tarefa por tarefa. Os passos usam caixas (`- [ ]`) para acompanhamento.

**Objetivo:** o xylem mostra, sem que o usuário procure, que roda como root/Administrador
(selo no cabeçalho, bordas vermelhas e selo nas perguntas de operações destrutivas) e
que roda dentro de uma sessão SSH (selo no cabeçalho com o nome da máquina).

**Arquitetura:** um módulo novo, `src/session.rs`, detecta a sessão uma vez, em
`main.rs`, com a ajuda de três funções novas de `platform` (`is_elevated`, `hostname`,
`ancestor_names`). O resultado fica em `App::session`. A UI (`src/ui.rs`) lê esse campo:
selos no cabeçalho, bordas por um `thread_local` posto em `draw()` (o mesmo padrão de
`HITS`) e selo no rodapé quando a pergunta altera arquivos desta máquina.

**Tecnologia:** Rust 2024, ratatui 0.30, `nix` (Unix), `windows-sys` (Windows).

**Desenho:** `docs/plans/root-ssh-indicator.md` (quem executa lê os dois).

## Restrições globais

- Antes de cada ponto de commit: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` e `cargo test`, todos sem erro (AGENTS.md).
- Commit e push **só com autorização explícita do autor**. Trabalho num branch a partir de `main` (`root-ssh-indicator`), com o e-mail noreply do GitHub (confira `git config user.email`).
- Arquivos temporários só em `~/.cache/xt-dev/`. Nos testes, use `crate::testing::tempdir()`.
- Código, comentários e manual em inglês. `docs/plans` em português.
- Textos dos selos: `ROOT` (Unix), `ADMIN` (Windows), `SSH <host>`, ou só `SSH` sem nome. Cada selo tem um espaço de cada lado do texto e é seguido de um espaço comum.
- Nome da máquina: até o primeiro `.`, só ASCII alfanumérico, `-` e `_`, no máximo 24 caracteres.
- Cores: selo de root na cor `error` do tema; selo SSH na cor `special`; texto dos selos na cor `background`, em negrito. Se a cor do selo for `Color::Reset` (tema `mono`): negrito + `REVERSED`.
- A detecção só roda em `main.rs`, nunca em `App::new`: testes rodando como root (contêiner, CI) não podem ver selos.
- Nenhuma crate nova. Só as features `hostname` do `nix` e `Win32_Security` do `windows-sys`.
- O código Windows precisa compilar: `cargo clippy --target x86_64-pc-windows-msvc --all-targets -- -D warnings` (o target está instalado). Se o `build.rs` impedir localmente, o CI Windows confere.
- O hostname e o conteúdo de `/proc` são entrada hostil.
- Se um teste de UI falhar só por caminho temporário longo, rode de novo com `XDG_CACHE_HOME=$HOME/.cache` (o Ghostty define essa variável).

## Foco da revisão

1. Terminal estreito, como root: `ROOT` continua inteiro no cabeçalho, e o relógio sai antes. Teste na Task 2.
2. Testes rodando como root: `App::new` não traz sessão nenhuma. Teste na Task 2.
3. Delete numa janela remota, como root local: sem selo na pergunta. O mesmo Delete local tem selo. Teste na Task 4.
4. Prompt de texto com selo: o cursor fica depois do que foi digitado, não 7 colunas antes. Teste na Task 4.
5. `sudo xy` dentro de SSH (variáveis apagadas): no Linux, a sessão SSH vem dos ancestrais. Teste na Task 1.

---

### Task 1: detectar root, SSH e o nome da máquina

**Arquivos:**
- Criar: `src/session.rs`
- Modificar: `src/main.rs` (`mod session;` na lista de módulos)
- Modificar: `src/platform/unix.rs`, `src/platform/windows.rs`
- Modificar: `Cargo.toml` (features `hostname` no `nix`, `Win32_Security` no `windows-sys`)

**Interfaces:**
- Produz, em `src/session.rs`:
  - `#[derive(Debug, Clone, Default, PartialEq, Eq)] pub struct Session { pub admin: bool, pub ssh: Option<String> }`. Em `ssh`, `None` quer dizer fora de SSH, e `Some("")` quer dizer SSH sem nome da máquina.
  - `pub const ADMIN_LABEL: &str`: `"ADMIN"` com `cfg(windows)`, `"ROOT"` nos outros.
  - `impl Session { pub fn detect() -> Session; pub fn from_parts(admin: bool, ssh: bool, host: Option<&str>) -> Session }`
  - `pub fn in_ssh(var: impl Fn(&str) -> Option<std::ffi::OsString>, ancestors: impl FnOnce() -> Vec<String>) -> bool`
  - `pub fn short_host(raw: &str) -> String`
- Produz, em `platform` (Unix e Windows, mesmas assinaturas):
  - `pub fn is_elevated() -> bool`
  - `pub fn hostname() -> Option<String>`
  - `pub fn ancestor_names() -> Vec<String>`
- Só no Linux (`#[cfg(target_os = "linux")]` em `unix.rs`): `fn parse_stat(text: &str) -> Option<(String, u32)>` e `fn ancestors_with(start: u32, read: impl Fn(u32) -> Option<String>) -> Vec<String>`.

- [ ] **Passo 1: escrever os testes que falham, em `src/session.rs`**

```rust
#[test]
fn ssh_from_any_of_the_variables() {
    for name in ["SSH_CONNECTION", "SSH_CLIENT", "SSH_TTY"] {
        let var = |k: &str| (k == name).then(|| OsString::from("10.0.0.1 22"));
        assert!(in_ssh(var, || panic!("ancestors read with {name} set")));
    }
}

#[test]
fn without_the_variables_the_ancestors_decide() {
    let empty = |_: &str| Some(OsString::new());
    let names = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    assert!(in_ssh(empty, || names(&["bash", "sudo", "bash", "sshd-session", "sshd"])));
    assert!(in_ssh(|_| None, || names(&["zsh", "mosh-server"])));
    assert!(in_ssh(|_| None, || names(&["bash", "sshd"])));
    assert!(!in_ssh(|_| None, || names(&["bash", "login", "systemd"])));
    assert!(!in_ssh(|_| None, || names(&["xsshd", "sshd2"])));
}

#[test]
fn short_host_keeps_the_first_label() {
    assert_eq!(short_host("srv01.example.com"), "srv01");
    assert_eq!(short_host("db-02_a"), "db-02_a");
}

#[test]
fn short_host_drops_what_is_not_a_name() {
    assert_eq!(short_host("\u{202e}evil\u{1b}[31m"), "evil31m");
    assert_eq!(short_host(&"a".repeat(100)), "a".repeat(24));
    assert_eq!(short_host(".hidden"), "");
}

#[test]
fn from_parts_names_the_machine_only_in_ssh() {
    assert_eq!(Session::from_parts(true, false, Some("srv01")), Session { admin: true, ssh: None });
    assert_eq!(Session::from_parts(false, true, Some("srv01.lan")).ssh.as_deref(), Some("srv01"));
    assert_eq!(Session::from_parts(false, true, None).ssh.as_deref(), Some(""));
}
```

E, em `src/platform/unix.rs`, num módulo de testes com `#[cfg(target_os = "linux")]`:

```rust
#[test]
fn parse_stat_reads_name_and_parent() {
    assert_eq!(parse_stat("1234 (sshd-session) S 1000 1234 1234 0 -1"), Some(("sshd-session".into(), 1000)));
    // The name may hold spaces and parentheses: it ends at the last `)`.
    assert_eq!(parse_stat("77 (a) b (c) S 5 77"), Some(("a) b (c".into(), 5)));
}

#[test]
fn parse_stat_rejects_garbage() {
    for bad in ["", "12 (x", "12 (x) S", "12 (x) S parent", "12 x) S 5"] {
        assert_eq!(parse_stat(bad), None, "{bad:?}");
    }
}

#[test]
fn ancestors_stop_at_init_at_a_loop_and_at_64() {
    let stat = |pid: u32, name: &str, ppid: u32| format!("{pid} ({name}) S {ppid} 0");
    let chain = |pid| match pid {
        10 => Some(stat(10, "bash", 9)),
        9 => Some(stat(9, "sshd-session", 1)),
        _ => None,
    };
    assert_eq!(ancestors_with(10, chain), ["bash", "sshd-session"]);
    // A loop (10 → 9 → 10) and an endless chain stop at 64.
    let looped = |pid| Some(stat(pid, "x", if pid == 10 { 9 } else { 10 }));
    assert_eq!(ancestors_with(10, looped).len(), 64);
    let endless = |pid| Some(stat(pid, "x", pid + 1));
    assert_eq!(ancestors_with(2, endless).len(), 64);
    assert!(ancestors_with(10, |_| None).is_empty());
    assert!(ancestors_with(1, chain).is_empty());
}

#[test]
fn ancestor_names_reads_this_process() {
    let names = ancestor_names();
    assert!(!names.is_empty() && names.len() <= 64, "{names:?}");
}
```

- [ ] **Passo 2: rodar e ver falhar**

Comando: `cargo test session:: platform::`
Esperado: erro de compilação (`in_ssh`, `short_host`, `Session`, `parse_stat`, `ancestors_with` e `ancestor_names` não existem).

- [ ] **Passo 3: implementar**

- `in_ssh`: verdadeiro se alguma das três variáveis existir e não for vazia. Só então, se nenhuma servir, chama `ancestors()` e procura um nome **exatamente** igual a `sshd`, `sshd-session` ou `mosh-server`.
- `short_host`: corta no primeiro `.`, filtra `is_ascii_alphanumeric() || '-' || '_'` e pega os 24 primeiros.
- `from_parts`: `ssh: ssh.then(|| host.map(short_host).unwrap_or_default())`.
- `detect`: `from_parts(platform::is_elevated(), in_ssh(|k| std::env::var_os(k), platform::ancestor_names), platform::hostname().as_deref())`.
- Unix:
  - `is_elevated`: `nix::unistd::geteuid().is_root()`.
  - `hostname`: `nix::unistd::gethostname()` convertido com `into_string().ok()`.
  - `ancestor_names`: no Linux, `ancestors_with(std::os::unix::process::parent_id(), |pid| std::fs::read_to_string(format!("/proc/{pid}/stat")).ok())`. Nos outros Unix, `Vec::new()`.
- `parse_stat`:
  - o nome vai do primeiro `(` ao último `)`;
  - depois do `)` vêm o estado e o pai (`u32`);
  - qualquer falta é `None`.
- `ancestors_with`:
  - para quando o pid é ≤ 1, quando a leitura ou o parse falha, quando o pai é igual ao próprio pid, ou com 64 nomes.
  - O `read` limita o que lê: o arquivo stat é pequeno, mas não confie nisso; corte o texto em 4096 bytes antes de fazer o parse.
- Windows:
  - `is_elevated`: `OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, …)` e `GetTokenInformation(…, TokenElevation, …)` para um `TOKEN_ELEVATION`. Feche o handle com `CloseHandle`. Qualquer falha dá `false`.
  - `hostname`: `GetComputerNameExW(ComputerNameDnsHostname, …)`, em duas chamadas (tamanho e depois o texto).
  - `ancestor_names`: `Vec::new()`. O OpenSSH do Windows define as variáveis, e lá não existe sudo.

- [ ] **Passo 4: rodar e ver passar**

Comando: `cargo test session:: platform::` e depois `cargo clippy --target x86_64-pc-windows-msvc --all-targets -- -D warnings`
Esperado: os testes passam e o clippy de Windows não tem warnings (ou, se o `build.rs` impedir localmente, anote para o CI).

- [ ] **Passo 5: ponto de commit** (só com autorização)

`feat: detect root, elevation and SSH sessions`

---

### Task 2: `App::session` e os selos no cabeçalho

**Arquivos:**
- Modificar: `src/app/mod.rs` (campo em `App` e valor padrão em `App::new`)
- Modificar: `src/main.rs:201` (depois de `App::new`: `app.session = session::Session::detect();`)
- Modificar: `src/ui.rs` (`draw_header`, mais duas funções novas)
- Testes: `src/ui.rs` (módulo de testes) e `src/app/tests.rs`

**Interfaces:**
- Consome: `Session`, `ADMIN_LABEL` (Task 1).
- Produz, em `App`: `pub session: crate::session::Session` (padrão `Session::default()`).
- Produz, em `src/ui.rs`:
  - `fn badge_style(t: &Theme, color: Color) -> Style`
  - `fn session_badge(text: &str, color: Color) -> Span<'static>`: o texto com um espaço de cada lado, no estilo `badge_style(&theme(), color)`.
  - `fn session_spans(session: &Session) -> Vec<Span<'static>>`: o selo de root e/ou o de SSH, cada um seguido de `Span::raw(" ")`.

- [ ] **Passo 1: escrever os testes que falham**

Em `src/app/tests.rs`:

```rust
#[test]
fn a_new_app_has_no_session() {
    // Even when the tests run as root: only main.rs detects.
    let (_tmp, app) = sample();
    assert_eq!(app.session, crate::session::Session::default());
}
```

Em `src/ui.rs`, ao lado de `header_shows_the_name_and_version_when_there_is_room`, use o mesmo fechamento `header(app, width)`:

```rust
#[test]
fn header_shows_root_and_ssh_before_the_path() {
    // `header(app, width)`: row 0 as text, as in the test above.
    let (t, label) = (theme(), crate::session::ADMIN_LABEL);
    app.session = Session { admin: true, ssh: Some("srv01".into()) };
    let text = header(&mut app, 100);
    assert!(text.starts_with(&format!(" {label}   SSH srv01  Path: ")), "{text:?}");
    // The cells of each badge: text in `background` on `error` / `special`.
    assert_eq!((cell(1).fg, cell(1).bg), (t.background, t.error));
    let s = text.find("SSH").unwrap() as u16;
    assert_eq!((cell(s).fg, cell(s).bg), (t.background, t.special));
    app.session = Session { admin: false, ssh: Some(String::new()) };
    assert!(header(&mut app, 100).starts_with(" SSH  Path: "));
    app.session = Session::default();
    assert!(header(&mut app, 100).starts_with("Path: "));
}

#[test]
fn header_keeps_root_whole_on_a_narrow_screen() {
    app.session = Session { admin: true, ssh: Some("a-long-machine-name".into()) };
    for width in [12, 22, 30, 40] {
        assert!(header(&mut app, width).starts_with(&format!(" {} ", crate::session::ADMIN_LABEL)), "{width}");
    }
}

#[test]
fn badges_in_mono_are_reversed() {
    let mono = Theme::mono();
    assert!(badge_style(&mono, mono.error).add_modifier.contains(Modifier::REVERSED));
    let classic = Theme::classic();
    let style = badge_style(&classic, classic.error);
    assert_eq!((style.fg, style.bg), (Some(classic.background), Some(classic.error)));
    assert!(!style.add_modifier.contains(Modifier::REVERSED));
}

#[cfg(unix)]
#[test]
fn session_badges_come_before_the_remote_one() {
    // A window on crate::remote::for_tests(tmp) (skip if None), via Pane::from_remote.
    app.session = Session { admin: true, ssh: Some("srv01".into()) };
    let text = header(&mut app, 120);
    let at = |s: &str| text.find(s).unwrap();
    assert!(at(crate::session::ADMIN_LABEL) < at("SSH srv01") && at("SSH srv01") < at("Path: "));
    assert!(at("Path: ") < at("test@testhost"));
}
```

(`cell(x)` é `buf[(x, 0)]` do mesmo desenho. Escreva o fechamento de modo que devolva o texto e o buffer.)

- [ ] **Passo 2: rodar e ver falhar**

Comando: `cargo test header_ badges_in_mono a_new_app_has_no_session session_badges`
Esperado: erro de compilação (`App::session` e `badge_style` não existem).

- [ ] **Passo 3: implementar**

- Os spans de `session_spans(&app.session)` entram antes de `"Path: "` em `draw_header`.
- O relógio e o nome/versão (`right_spans`) só aparecem se `area.width` ≥ largura dos selos + largura de `right_spans` + 1. Caso contrário, `right_spans` fica vazio.
- O resto do cabeçalho continua igual: o selo remoto depois de `Path:` e o caminho cortado por `fit_tail` no espaço que sobra.

- [ ] **Passo 4: rodar e ver passar**

Comando: `cargo test header_ badges_in_mono a_new_app_has_no_session session_badges`, depois `cargo test`
Esperado: passa, e nenhum teste antigo do cabeçalho quebra.

- [ ] **Passo 5: ponto de commit** (só com autorização)

`feat: header badges for root and SSH sessions`

---

### Task 3: bordas vermelhas quando root

**Arquivos:**
- Modificar: `src/ui.rs` (`draw`, `panel`)
- Testes: `src/ui.rs`

**Interfaces:**
- Consome: `App::session` (Task 2).
- Produz:
  - `thread_local! { static ADMIN: Cell<bool> = const { Cell::new(false) }; }`, definido no início de `draw()` com `ADMIN.set(app.session.admin)`.
  - `fn panel_with(t: &Theme, admin: bool, title: impl Into<Line<'static>>, active: bool) -> Block<'static>`. A função `panel(title, active)` passa a ser `panel_with(&theme(), ADMIN.get(), title, active)`.

- [ ] **Passo 1: escrever os testes que falham**

```rust
#[test]
fn root_borders_are_red_and_the_focus_thick() {
    use ratatui::{Terminal, backend::TestBackend};
    let corner = |t: &Theme, admin: bool, active: bool| {
        let mut term = Terminal::new(TestBackend::new(10, 3)).unwrap();
        term.draw(|f| f.render_widget(panel_with(t, admin, "", active), f.area())).unwrap();
        let c = term.backend().buffer()[(0, 0)].clone();
        (c.symbol().to_string(), c.fg)
    };
    for t in [Theme::classic(), Theme::light(), Theme::nord(), Theme::mono()] {
        assert_eq!(corner(&t, true, true), ("┏".into(), t.error));
        let normal = if border_type(&t) == BorderType::Rounded { "╭" } else { "┌" };
        assert_eq!(corner(&t, true, false), (normal.into(), t.error));
        // Without root, nothing changes.
        assert_eq!(corner(&t, false, true), (normal.into(), t.active_border));
        assert_eq!(corner(&t, false, false), (normal.into(), t.border));
    }
}

#[test]
fn the_drawn_screen_follows_the_session() {
    // App::new + draw at 100x20: (0, 1) is the corner of the tree.
    app.session.admin = true;
    let (symbol, fg) = corner_after_draw(&mut app);
    assert_eq!((symbol.as_str(), fg), ("┏", theme().error));
    app.session.admin = false;
    assert_ne!(corner_after_draw(&mut app).0, "┏");
}
```

- [ ] **Passo 2: rodar e ver falhar**

Comando: `cargo test root_borders the_drawn_screen_follows`
Esperado: erro de compilação (`panel_with` não existe).

- [ ] **Passo 3: implementar `panel_with`**

- Com `admin`: borda na cor `t.error`, `BorderType::Thick` se `active`, senão `border_type(t)`.
- Sem `admin`: exatamente o `panel` de hoje.
- Os outros usos de `border`/`active_border` (guias da árvore, divisórias de tabela, barras) não mudam: são linhas internas, não bordas de janela.

- [ ] **Passo 4: rodar e ver passar**

Comando: `cargo test root_borders the_drawn_screen_follows`, depois `cargo test`
Esperado: passa.

- [ ] **Passo 5: ponto de commit** (só com autorização)

`feat: red window borders when running as root`

---

### Task 4: selo de root nas perguntas que alteram arquivos desta máquina

**Arquivos:**
- Modificar: `src/ops/mod.rs` (`Operation::dst_is_local`)
- Modificar: `src/app/prompts.rs` (dois predicados)
- Modificar: `src/ui.rs` (`draw_menu` no ramo de `confirm`, `prompt_lines`, `op_lines`)
- Testes: `src/app/tests.rs`, `src/ui.rs`

**Interfaces:**
- Consome: `App::session`, `session_badge`, `ADMIN_LABEL` (Tasks 1 e 2).
- Produz:
  - `impl Operation { pub fn dst_is_local(&self) -> bool }`: `matches!(self.dst, Side::Local)`.
  - `impl App { pub(crate) fn prompt_changes_local_files(&self, kind: &PromptKind) -> bool; pub(crate) fn confirm_changes_local_files(&self, kind: &ConfirmKind) -> bool }`
  - Em `src/ui.rs`: `fn root_lead(app: &App, changes: bool) -> Vec<Span<'static>>`. Com `app.session.admin && changes`, devolve `[session_badge(ADMIN_LABEL, theme().error), Span::raw(" ")]`; senão, vazio.
  - Assinaturas novas: `prompt_lines(frame, prompt, area, lead: Vec<Span<'static>>)` e `op_lines(op, lead: Vec<Span<'static>>)`.

Regras dos predicados (o desenho, seção 3):

| Pergunta | Altera arquivos desta máquina quando |
|---|---|
| `PromptKind::Transfer` (Copy/Move) | sempre: enquanto se digita, o destino ainda não é conhecido |
| `RenameFile`, `RenameDir`, `RenameTagged`, `Chmod`, `NewDate`, `Prune` | `self.remote.is_none()` |
| `ConfirmKind::DeleteFiles`, `DeleteDir`, `DeleteFolders` | `self.remote.is_none()` |
| `ConfirmKind::CreateDest { dest, .. }` | `matches!(self.side_of(dest), Side::Local)` |
| conflito de sobrescrita (`op.conflict`) | `op.dst_is_local()` |
| todas as outras | nunca |

- [ ] **Passo 1: escrever os testes que falham**

Em `src/app/tests.rs`:

```rust
#[test]
fn delete_and_move_change_local_files() {
    let (_tmp, mut app) = sample();
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Char('d'));
    let kind = &app.confirm.as_ref().unwrap().kind;
    assert!(app.confirm_changes_local_files(kind));
    press(&mut app, KeyCode::Esc);
    press(&mut app, KeyCode::Char('m'));
    assert!(app.prompt_changes_local_files(&app.prompt.as_ref().unwrap().kind));
}

#[test]
fn filespec_and_quit_change_nothing() {
    let (_tmp, app) = sample();
    assert!(!app.prompt_changes_local_files(&PromptKind::Filespec));
    assert!(!app.confirm_changes_local_files(&ConfirmKind::Quit));
}

#[cfg(unix)]
#[test]
fn delete_on_a_remote_window_does_not_change_local_files() {
    let Some((_tmp, mut app)) = remote_sample() else { return };
    // Same keys as above: Enter, Down, d.
    let kind = &app.confirm.as_ref().unwrap().kind;
    assert!(!app.confirm_changes_local_files(kind));
}

#[test]
fn overwrite_here_changes_local_files() {
    // As in move_with_conflict_prompt: sub/b.txt exists, Move b.txt to sub.
    assert!(app.op.as_ref().unwrap().conflict.is_some());
    assert!(app.op.as_ref().unwrap().dst_is_local());
}
```

Em `src/ui.rs` (`footer_rows` já existe):

```rust
#[test]
fn root_badge_leads_the_delete_question() {
    let label = crate::session::ADMIN_LABEL;
    // sample-like app: Enter, Down, d (as in app/tests.rs).
    app.session.admin = true;
    assert!(footer_rows(&mut app, 120)[0].starts_with(&format!(" {label}  DELETE ")));
    app.session.admin = false;
    assert!(footer_rows(&mut app, 120)[0].starts_with("DELETE "));
}

#[test]
fn root_badge_is_not_on_the_filespec() {
    app.session.admin = true;
    // F (Filespec) open.
    assert!(!footer_rows(&mut app, 120)[0].contains(crate::session::ADMIN_LABEL));
}

#[test]
fn root_badge_moves_the_prompt_cursor() {
    use ratatui::{Terminal, backend::TestBackend};
    // r (Rename) on b.txt, with admin and without; same text typed.
    let cursor = |app: &mut App| {
        let mut term = Terminal::new(TestBackend::new(120, 20)).unwrap();
        term.draw(|f| draw(f, app)).unwrap();
        term.get_cursor_position().unwrap().x
    };
    app.session.admin = false;
    let plain = cursor(&mut app);
    app.session.admin = true;
    let lead = crate::session::ADMIN_LABEL.len() as u16 + 3; // " ROOT " + " "
    assert_eq!(cursor(&mut app), plain + lead);
}
```

- [ ] **Passo 2: rodar e ver falhar**

Comando: `cargo test change_local_files changes_nothing root_badge_`
Esperado: erro de compilação (os predicados e `dst_is_local` não existem).

- [ ] **Passo 3: implementar**

- Os predicados seguem a tabela.
- Em `draw_menu`:
  - `prompt_lines(frame, prompt, area, root_lead(app, app.prompt_changes_local_files(&prompt.kind)))`;
  - no ramo de `confirm`, a primeira linha passa a ser `Line::from(lead + [Span::styled(question, bold)])`;
  - no ramo de `op`, `op_lines(op, root_lead(app, op.conflict.is_some() && op.dst_is_local()))`.
- Em `prompt_lines`:
  - a largura do `lead` (soma de `content.chars().count()`) entra no cálculo de `col` do cursor e no `start` de `Spot::Prompt`, para o clique do mouse;
  - o `lead` vai na frente do rótulo.
- Em `op_lines`, o `lead` só vai na primeira linha do conflito.

- [ ] **Passo 4: rodar e ver passar**

Comando: `cargo test change_local_files changes_nothing root_badge_`, depois `cargo test`
Esperado: passa. Os testes de mouse no prompt continuam passando (sem root, o `lead` fica vazio).

- [ ] **Passo 5: ponto de commit** (só com autorização)

`feat: root badge on questions that change local files`

---

### Task 5: manual e notas de versão

**Arquivos:**
- Modificar: `docs/manual/screen.md` e `docs/manual/{pt,es,fr,de,zh}/screen.md`
- Modificar: `docs/manual/releases.md` e `docs/manual/{pt,es,fr,de,zh}/releases.md` (0.10.0, **New**)

- [ ] **Passo 1: `screen.md`**

Na linha 1 da tabela (Header), acrescente:
- os selos `ROOT` (Unix) e `ADMIN` (Windows), em vermelho;
- `SSH <machine>`, que aparece quando o xylem roda numa sessão SSH (variáveis `SSH_*` ou, no Linux, um `sshd`/`mosh-server` entre os processos ancestrais);
- que, como root, as bordas ficam vermelhas e a janela com o foco ganha borda grossa;
- que as perguntas de Delete, Prune, Move, Copy, Rename, Attributes e New date levam o selo quando alteram arquivos desta máquina.

Logo depois da tabela, registre as limitações em duas frases: dentro do tmux/screen, valem as variáveis da sessão que criou o servidor; no macOS sob `sudo`, a sessão SSH não é detectada.

As traduções seguem o mesmo conteúdo, nos termos já usados em cada arquivo.

- [ ] **Passo 2: `releases.md`**

Um item em **New** da 0.10.0, com link para [The screen](screen.md), em todas as línguas.

- [ ] **Passo 3: conferir**

Comando: `cargo test help` (os testes do manual embutido, inclusive `a_translation_opens_at_the_same_section`), depois `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` e `cargo test`
Esperado: tudo passa.

- [ ] **Passo 4: ponto de commit** (só com autorização)

`docs: root and SSH badges in the manual and release notes`
