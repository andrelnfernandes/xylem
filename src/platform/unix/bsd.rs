//! macOS and FreeBSD, which have no `/proc`: what Linux reads there comes
//! from `ps` and `sysctl`. The system is told by `std::env::consts::OS` when
//! Xylem runs, not by `#[cfg]`, so this code builds and is tested on Linux
//! too.

use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::platform::{Memory, System};

/// `ps` on macOS and FreeBSD.
const PS: &str = "/bin/ps";
/// Of `ps -A`, at most.
const PS_MAX: u64 = 4 << 20;
/// Of `sysctl`, at most.
const SYSCTL_MAX: u64 = 64 << 10;
/// Of `ps -o rss=`, at most.
const RSS_MAX: u64 = 1 << 10;
/// How long a program may take.
const LIMIT: Duration = Duration::from_secs(1);

/// What the statistics read on macOS.
const MAC_NAMES: [&str; 13] = [
    "kern.ostype",
    "kern.osrelease",
    "kern.osproductversion",
    "kern.hostname",
    "machdep.cpu.brand_string",
    "kern.boottime",
    "vm.loadavg",
    "hw.memsize",
    "hw.pagesize",
    "vm.page_free_count",
    "vm.page_pageable_external_count",
    "vm.page_purgeable_count",
    "vm.swapusage",
];
/// What the statistics read on FreeBSD.
const FREEBSD_NAMES: [&str; 10] = [
    "kern.ostype",
    "kern.osrelease",
    "kern.hostname",
    "hw.model",
    "kern.boottime",
    "vm.loadavg",
    "hw.physmem",
    "hw.pagesize",
    "vm.stats.vm.v_free_count",
    "vm.stats.vm.v_inactive_count",
];

/// Runs `program` and returns what it prints, at most `cap` bytes. There is
/// no shell and no `PATH`: the environment is empty but for `LC_ALL=C`, and
/// stdin and stderr go nowhere. A program that does not start, or that has
/// not finished printing within `limit`, gives `None`; one that prints more
/// than `cap` is ended.
pub(super) fn run_capped(
    program: &Path,
    args: &[&str],
    cap: u64,
    limit: Duration,
) -> Option<String> {
    let mut child = Command::new(program)
        .args(args)
        .env_clear()
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let (tx, rx) = mpsc::channel();
    if let Some(stdout) = child.stdout.take() {
        std::thread::spawn(move || {
            let mut bytes = Vec::new();
            let read = stdout.take(cap).read_to_end(&mut bytes);
            let _ = tx.send(read.map(|_| bytes));
        });
    }
    let out = rx.recv_timeout(limit);
    // Ends one still running (past the cap or the limit), then reaps it.
    let _ = child.kill();
    let _ = child.wait();
    let bytes = out.ok()?.ok()?;
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

/// The lines of `ps -o pid=,ppid=,comm=`, as pid → (name, parent's pid). The
/// name is what follows the last `/`, should a `ps` give a path. A line
/// without both numbers and a name is skipped, and a repeated pid keeps its
/// first line.
fn parse_ps(text: &str) -> HashMap<u32, (String, u32)> {
    let mut table = HashMap::new();
    for (pid, parent, name) in text.lines().filter_map(ps_line) {
        table
            .entry(pid)
            .or_insert_with(|| (name.to_string(), parent));
    }
    table
}

fn ps_line(line: &str) -> Option<(u32, u32, &str)> {
    let (pid, rest) = line.trim_start().split_once(char::is_whitespace)?;
    let (parent, comm) = rest.trim_start().split_once(char::is_whitespace)?;
    let name = comm.trim().rsplit('/').next()?;
    if name.is_empty() {
        return None;
    }
    Some((pid.parse().ok()?, parent.parse().ok()?, name))
}

/// Every process, as `pid ppid name`: one column per `-o`, since after `=`
/// the rest of an argument is the column's header (POSIX; FreeBSD keeps to
/// it), and `ucomm`, the executable's name as the kernel keeps it, not
/// `comm`, which on macOS is argv[0] as sshd rewrites it.
const PS_ARGS: &[&str] = &["-A", "-o", "pid=", "-o", "ppid=", "-o", "ucomm="];

/// The names of the ancestors from `start` up, from one `ps -A`.
pub(super) fn ancestor_names(start: u32) -> Vec<String> {
    let Some(text) = run_capped(Path::new(PS), PS_ARGS, PS_MAX, LIMIT) else {
        return Vec::new();
    };
    let table = parse_ps(&text);
    super::ancestors_with(start, |pid| table.get(&pid).cloned())
}

/// The machine, for the statistics screen, from one `sysctl` and one `ps`.
pub(super) fn system(os: &str) -> System {
    let names = sysctl_names(os);
    let sysctl = run_capped(Path::new(sysctl_path(os)), names, SYSCTL_MAX, LIMIT);
    let os_release = (os != "macos")
        .then(|| fs::read_to_string("/etc/os-release").ok())
        .flatten();
    let pid = std::process::id().to_string();
    let rss = run_capped(Path::new(PS), &["-o", "rss=", "-p", &pid], RSS_MAX, LIMIT);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    system_from(
        os,
        sysctl.as_deref().unwrap_or_default(),
        os_release.as_deref(),
        rss.as_deref(),
        now,
    )
}

fn sysctl_names(os: &str) -> &'static [&'static str] {
    if os == "macos" {
        &MAC_NAMES
    } else {
        &FREEBSD_NAMES
    }
}

fn sysctl_path(os: &str) -> &'static str {
    if os == "macos" {
        "/usr/sbin/sysctl"
    } else {
        "/sbin/sysctl"
    }
}

/// The machine from the output of `sysctl` (`name: value` lines), of
/// `/etc/os-release` and of `ps -o rss=`, at `now_secs` since the epoch.
fn system_from(
    os: &str,
    sysctl: &str,
    os_release: Option<&str>,
    rss: Option<&str>,
    now_secs: u64,
) -> System {
    let v = parse_sysctl(sysctl);
    let text = |name: &str| v.get(name).filter(|s| !s.is_empty()).map(|s| s.to_string());
    let number = |name: &str| v.get(name)?.parse::<u64>().ok();
    let mac = os == "macos";
    let kernel = text("kern.ostype")
        .zip(text("kern.osrelease"))
        .map(|(os, release)| format!("{os} {release}"));
    let os_name = if mac {
        text("kern.osproductversion").map(|v| format!("macOS {v}"))
    } else {
        os_release
            .and_then(super::os_release_name)
            .or_else(|| kernel.clone())
    };
    // Pages that can still be used, in bytes.
    let pages = |names: &[&str]| {
        let size = number("hw.pagesize")?;
        names
            .iter()
            .try_fold(0u64, |sum, n| sum.checked_add(number(n)?))?
            .checked_mul(size)
    };
    let memory = if mac {
        number("hw.memsize").zip(pages(&[
            "vm.page_free_count",
            "vm.page_pageable_external_count",
            "vm.page_purgeable_count",
        ]))
    } else {
        number("hw.physmem").zip(pages(&[
            "vm.stats.vm.v_free_count",
            "vm.stats.vm.v_inactive_count",
        ]))
    };
    System {
        os: os_name,
        kernel,
        host: text("kern.hostname"),
        cpu: text(if mac {
            "machdep.cpu.brand_string"
        } else {
            "hw.model"
        }),
        uptime: v
            .get("kern.boottime")
            .and_then(|b| boottime_secs(b))
            .and_then(|boot| now_secs.checked_sub(boot))
            .map(Duration::from_secs),
        load: v.get("vm.loadavg").and_then(|l| loadavg(l)),
        memory: memory.map(|(total, available)| Memory { total, available }),
        swap: mac
            .then(|| v.get("vm.swapusage").and_then(|s| swapusage(s)))
            .flatten(),
        xt_memory: rss
            .and_then(|r| r.trim().parse::<u64>().ok())
            .and_then(|kib| kib.checked_mul(1024)),
    }
}

/// `name: value` lines, split at the first `": "`.
fn parse_sysctl(text: &str) -> HashMap<&str, &str> {
    text.lines()
        .filter_map(|l| l.split_once(": "))
        .map(|(name, value)| (name, value.trim()))
        .collect()
}

/// The seconds of `kern.boottime`: `{ sec = 1764000000, usec = 123456 } Mon ...`.
fn boottime_secs(v: &str) -> Option<u64> {
    let rest = v.trim_start().strip_prefix("{ sec = ")?;
    rest.split(|c: char| !c.is_ascii_digit())
        .next()?
        .parse()
        .ok()
}

/// `vm.loadavg`: `{ 1.23 1.10 0.98 }`.
fn loadavg(v: &str) -> Option<[f64; 3]> {
    let inner = v.trim().strip_prefix('{')?.strip_suffix('}')?;
    let mut it = inner.split_whitespace().map(|n| n.parse::<f64>().ok());
    Some([it.next()??, it.next()??, it.next()??])
}

/// macOS `vm.swapusage`: `total = 2048.00M  used = 1024.50M  free = 1023.50M  (encrypted)`.
fn swapusage(v: &str) -> Option<Memory> {
    let field = |key: &str| size_bytes(v.split_once(key)?.1.split_whitespace().next()?);
    Some(Memory {
        total: field("total = ")?,
        available: field("free = ")?,
    })
}

/// `2048.00M`, `2.00G`, `512.00K` in bytes.
fn size_bytes(word: &str) -> Option<u64> {
    let unit = match word.chars().last()? {
        'K' => 1u64 << 10,
        'M' => 1 << 20,
        'G' => 1 << 30,
        _ => return None,
    };
    // The unit is one ASCII byte.
    let n: f64 = word[..word.len() - 1].parse().ok()?;
    (n.is_finite() && n >= 0.0).then(|| (n * unit as f64).round() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::time::{Duration, Instant};

    #[test]
    fn run_capped_returns_what_the_program_prints() {
        let out = run_capped(
            Path::new("/bin/sh"),
            &["-c", "echo a; echo b"],
            1024,
            Duration::from_secs(5),
        );
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
        assert_eq!(
            run_capped(
                Path::new("/bin/sleep"),
                &["5"],
                1024,
                Duration::from_millis(200)
            ),
            None
        );
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
        assert_eq!(
            run_capped(
                Path::new("/nonexistent/ps"),
                &[],
                1024,
                Duration::from_secs(1)
            ),
            None
        );
    }

    #[test]
    fn parse_ps_reads_pid_parent_and_name() {
        let t = parse_ps(
            "    1     0 /sbin/launchd\n  612     1 /usr/sbin/sshd\n  640   612 sshd-session\n  700   640 -zsh\n  800   700 /Applications/Visual Studio Code.app/Contents/MacOS/Electron\n  810   700 Google Chrome Helper\n",
        );
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
        let t = parse_ps(
            "    1     0 /sbin/launchd\n  612     1 /usr/sbin/sshd\n  640   612 sshd-session\n  700   640 -zsh\n",
        );
        assert_eq!(
            super::super::ancestors_with(700, |pid| t.get(&pid).cloned()),
            ["-zsh", "sshd-session", "sshd"]
        );
    }

    #[test]
    fn ps_asks_for_one_column_per_option_and_the_kernel_name() {
        // POSIX: after `=` the rest of the argument is the column's header,
        // so FreeBSD reads `-o pid=,ppid=` as one column. And `comm` is
        // argv[0] on macOS, which sshd rewrites ("sshd: user@ttys000");
        // `ucomm` is the executable's name, as the kernel keeps it.
        let columns: Vec<&str> = PS_ARGS
            .windows(2)
            .filter(|w| w[0] == "-o")
            .map(|w| w[1])
            .collect();
        assert_eq!(columns, ["pid=", "ppid=", "ucomm="]);
        assert_eq!(PS_ARGS[0], "-A");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn the_walk_over_the_real_ps_matches_proc() {
        // Linux has /bin/ps too (procps), whose ucomm is /proc's name.
        let parent = std::os::unix::process::parent_id();
        if !Path::new("/bin/ps").exists() || parent <= 1 {
            return;
        }
        let names = ancestor_names(parent);
        assert!(!names.is_empty());
        assert_eq!(names, super::super::ancestor_names());
    }

    const MAC: &str = "kern.ostype: Darwin\nkern.osrelease: 24.1.0\nkern.osproductversion: 15.1\nkern.hostname: mac-mini.local\nmachdep.cpu.brand_string: Apple M1\nkern.boottime: { sec = 1764000000, usec = 123456 } Mon Nov 24 16:00:00 2025\nvm.loadavg: { 1.23 1.10 0.98 }\nhw.memsize: 17179869184\nhw.pagesize: 16384\nvm.page_free_count: 1000\nvm.page_pageable_external_count: 2000\nvm.page_purgeable_count: 500\nvm.swapusage: total = 2048.00M  used = 1024.50M  free = 1023.50M  (encrypted)\n";
    const BSD: &str = "kern.ostype: FreeBSD\nkern.osrelease: 14.1-RELEASE-p5\nkern.hostname: fbsd\nhw.model: Intel(R) Xeon(R) CPU E5-2680 v4 @ 2.40GHz\nkern.boottime: { sec = 1764000000, usec = 0 } Mon Nov 24 16:00:00 2025\nvm.loadavg: { 0.10 0.20 0.30 }\nhw.physmem: 8589934592\nhw.pagesize: 4096\nvm.stats.vm.v_free_count: 100000\nvm.stats.vm.v_inactive_count: 50000\n";
    const BOOT: u64 = 1764000000;
    const MIB: u64 = 1 << 20;

    #[test]
    fn parse_sysctl_splits_at_the_first_colon_space() {
        let names = parse_sysctl(MAC);
        assert_eq!(
            names["kern.boottime"],
            "{ sec = 1764000000, usec = 123456 } Mon Nov 24 16:00:00 2025"
        );
        assert_eq!(names["hw.memsize"], "17179869184");
        assert!(!parse_sysctl("garbage\nsysctl: unknown oid 'x'\n").contains_key("x"));
    }

    #[test]
    fn boottime_reads_sec() {
        assert_eq!(
            boottime_secs("{ sec = 1764000000, usec = 123456 } Mon Nov 24 16:00:00 2025"),
            Some(1764000000)
        );
        assert_eq!(boottime_secs("{ sec = 42 }"), Some(42));
        assert_eq!(boottime_secs("garbage"), None);
    }

    #[test]
    fn loadavg_reads_three_numbers() {
        assert_eq!(loadavg("{ 1.23 1.10 0.98 }"), Some([1.23, 1.10, 0.98]));
        assert_eq!(loadavg("{ 1.0 }"), None);
        assert_eq!(loadavg("{ a b c }"), None);
    }

    #[test]
    fn swapusage_reads_total_and_free() {
        assert_eq!(
            swapusage("total = 2048.00M  used = 1024.50M  free = 1023.50M  (encrypted)"),
            Some(Memory {
                total: 2048 * MIB,
                available: 1023 * MIB + MIB / 2,
            })
        );
        assert_eq!(
            swapusage("total = 2.00G  used = 0.00M  free = 2.00G"),
            Some(Memory {
                total: 2048 * MIB,
                available: 2048 * MIB,
            })
        );
        assert_eq!(swapusage("total = lots"), None);
    }

    #[test]
    fn macos_system_from_sysctl() {
        let s = system_from("macos", MAC, None, Some("  12345\n"), BOOT + 3600);
        assert_eq!(s.os.as_deref(), Some("macOS 15.1"));
        assert_eq!(s.kernel.as_deref(), Some("Darwin 24.1.0"));
        assert_eq!(s.host.as_deref(), Some("mac-mini.local"));
        assert_eq!(s.cpu.as_deref(), Some("Apple M1"));
        assert_eq!(s.uptime, Some(Duration::from_secs(3600)));
        assert_eq!(s.load, Some([1.23, 1.10, 0.98]));
        assert_eq!(
            s.memory,
            Some(Memory {
                total: 17179869184,
                available: 3500 * 16384,
            })
        );
        assert_eq!(
            s.swap,
            Some(Memory {
                total: 2048 * MIB,
                available: 1023 * MIB + MIB / 2,
            })
        );
        assert_eq!(s.xt_memory, Some(12345 * 1024));
    }

    #[test]
    fn freebsd_system_from_sysctl() {
        let release = "NAME=FreeBSD\nPRETTY_NAME=\"FreeBSD 14.1-RELEASE-p5\"\n";
        let s = system_from("freebsd", BSD, Some(release), None, BOOT + 60);
        assert_eq!(s.os.as_deref(), Some("FreeBSD 14.1-RELEASE-p5"));
        assert_eq!(s.kernel.as_deref(), Some("FreeBSD 14.1-RELEASE-p5"));
        assert_eq!(s.host.as_deref(), Some("fbsd"));
        assert_eq!(
            s.cpu.as_deref(),
            Some("Intel(R) Xeon(R) CPU E5-2680 v4 @ 2.40GHz")
        );
        assert_eq!(s.uptime, Some(Duration::from_secs(60)));
        assert_eq!(
            s.memory,
            Some(Memory {
                total: 8589934592,
                available: 150000 * 4096,
            })
        );
        assert_eq!(s.swap, None);
        assert_eq!(s.xt_memory, None);
        // Without /etc/os-release, the system is the kernel's.
        let s = system_from("freebsd", BSD, None, None, BOOT);
        assert_eq!(s.os.as_deref(), Some("FreeBSD 14.1-RELEASE-p5"));
    }

    #[test]
    fn missing_names_leave_only_their_fields_empty() {
        let text: String = MAC
            .lines()
            .filter(|l| !l.starts_with("kern.osproductversion") && !l.starts_with("hw.pagesize"))
            .map(|l| format!("{l}\nsysctl: unknown oid 'x'\n"))
            .collect();
        let s = system_from("macos", &text, None, None, BOOT);
        assert_eq!(s.os, None);
        assert_eq!(s.memory, None);
        assert_eq!(s.kernel.as_deref(), Some("Darwin 24.1.0"));
        assert_eq!(s.cpu.as_deref(), Some("Apple M1"));
        assert_eq!(s.load, Some([1.23, 1.10, 0.98]));
        assert!(system_from("macos", "", None, None, BOOT).kernel.is_none());
    }

    #[test]
    fn a_boottime_in_the_future_has_no_uptime() {
        assert_eq!(system_from("macos", MAC, None, None, BOOT - 1).uptime, None);
    }

    #[test]
    fn system_reads_this_process_with_the_real_ps() {
        // Linux's /sbin/sysctl knows none of these names, but /bin/ps answers.
        if !Path::new("/bin/ps").exists() {
            return;
        }
        assert!(system("freebsd").xt_memory.is_some_and(|b| b > 0));
    }

    #[test]
    fn sysctl_names_and_paths() {
        assert_eq!(sysctl_path("macos"), "/usr/sbin/sysctl");
        assert_eq!(sysctl_path("freebsd"), "/sbin/sysctl");
        // Each name the samples hold, the ones `system_from` reads, is asked for.
        for (os, sample) in [("macos", MAC), ("freebsd", BSD)] {
            for line in sample.lines() {
                let name = line.split_once(": ").unwrap().0;
                assert!(sysctl_names(os).contains(&name), "{os}: {name}");
            }
        }
        assert!(!sysctl_names("freebsd").contains(&"kern.osproductversion"));
        assert!(!sysctl_names("macos").contains(&"hw.model"));
    }
}
