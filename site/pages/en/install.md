# Install

<p class="xylem-lead">Get it running in a minute</p>
<p class="section-lead">The command is <code>xy</code>. Run <code>xy</code> to open the current
directory, or <code>xy some/directory</code>.</p>
<div class="install">
<div class="card">
<h3>Ready-made binary</h3>
<p>Download the archive for your system from the
<a href="https://github.com/andrelnfernandes/xylem/releases/latest">Releases</a> page:
Linux x86_64 or ARM64 (static, runs on any distribution) or Windows x86_64. Then put
<code>xy</code> on your <code>PATH</code>:</p>

```sh
tar xzf xy-*-x86_64-unknown-linux-musl.tar.gz
install -m 755 xy-*-x86_64-unknown-linux-musl/xy ~/.local/bin/
```

<p>Each archive has a <code>.sha256</code> next to it, to check with <code>sha256sum -c</code>.</p>
</div>
<div class="card">
<h3>Package (.deb or .rpm)</h3>
<p>The releases from 0.5.0 on also have packages for Debian, Ubuntu, Fedora, RHEL, openSUSE
and their derivatives, x86_64 or ARM64. They also install the menu entry, the icons
and the man page:</p>

```sh
sudo apt install ./xylem-fm_*_amd64.deb
sudo dnf install ./xylem-fm-*.x86_64.rpm
```

<p>They come from the Releases page, not from a repository: to update, install the next
one the same way.</p>
</div>
<div class="card">
<h3>With Cargo</h3>
<p>Needs Rust, via <a href="https://rustup.rs">rustup</a>, and a C compiler. On
<a href="https://crates.io/crates/xylem-fm">crates.io</a> Xylem is called
<code>xylem-fm</code>:</p>

```sh
cargo install xylem-fm
```

<p>Or the latest code, straight from GitHub:</p>

```sh
cargo install --git https://github.com/andrelnfernandes/xylem
```

</div>
</div>
<p class="section-lead">There are also a
<a href="https://github.com/andrelnfernandes/xylem#installation">menu entry and a man page</a>
(<code>xy --man</code>), and a shell function to end up in the directory you were in when you
quit with <kbd>Alt</kbd>+<kbd>Q</kbd>.</p>

