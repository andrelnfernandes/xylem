# Instalar

<p class="xylem-lead">Funcionando em um minuto</p>
<p class="section-lead">O comando é <code>xy</code>. Rode <code>xy</code> para abrir o diretório
atual, ou <code>xy algum/diretorio</code>.</p>
<div class="install">
<div class="card">
<h3>Binário pronto</h3>
<p>Baixe o arquivo para o seu sistema na página de
<a href="https://github.com/andrelnfernandes/xylem/releases/latest">Releases</a>:
Linux x86_64 ou ARM64 (estático, roda em qualquer distribuição) ou Windows x86_64.
Depois coloque o <code>xy</code> no seu <code>PATH</code>:</p>

```sh
tar xzf xy-*-x86_64-unknown-linux-musl.tar.gz
install -m 755 xy-*-x86_64-unknown-linux-musl/xy ~/.local/bin/
```

<p>Cada arquivo tem um <code>.sha256</code> ao lado, para conferir com <code>sha256sum -c</code>.</p>
</div>
<div class="card">
<h3>Pacote (.deb ou .rpm)</h3>
<p>As versões a partir da 0.5.0 também têm pacotes para Debian, Ubuntu, Fedora, RHEL,
openSUSE e derivados, x86_64 ou ARM64. Eles também instalam a entrada no menu, os
ícones e a página de manual:</p>

```sh
sudo apt install ./xylem-fm_*_amd64.deb
sudo dnf install ./xylem-fm-*.x86_64.rpm
```

<p>Eles vêm da página de Releases, não de um repositório: para atualizar, instale o
próximo do mesmo jeito.</p>
</div>
<div class="card">
<h3>Com o Cargo</h3>
<p>Precisa do Rust, via <a href="https://rustup.rs">rustup</a>, e de um compilador C. No
<a href="https://crates.io/crates/xylem-fm">crates.io</a> o Xylem se chama
<code>xylem-fm</code>:</p>

```sh
cargo install xylem-fm
```

<p>Ou o código mais recente, direto do GitHub:</p>

```sh
cargo install --git https://github.com/andrelnfernandes/xylem
```

</div>
</div>
<p class="section-lead">Há também uma
<a href="https://github.com/andrelnfernandes/xylem/blob/main/README.pt-BR.md#instalação">entrada no menu e uma página de manual</a>
(<code>xy --man</code>), e uma função de shell para terminar no diretório em que você estava
ao sair com <kbd>Alt</kbd>+<kbd>Q</kbd>.</p>

