# 安装与最初五分钟

## 安装

Xylem 运行在 Linux（x86_64 与 ARM64）上。Windows 构建版本已通过测试，但尚未经过日常实际使用；参见 [Windows](windows.md)。

**二进制文件。** 从 `https://github.com/andrelnfernandes/xylem/releases` 下载适合您系统的压缩包，解压后将 `xy` 放置在 `PATH` 包含的目录中：

```bash
tar xzf xy-v0.7.0-x86_64-unknown-linux-musl.tar.gz
install -m 755 xy-v0.7.0-x86_64-unknown-linux-musl/xy ~/.local/bin/
```

Linux 二进制文件为静态链接，可在任何发行版上运行。每个压缩包都附带一个 `.sha256` 文件，可用 `sha256sum -c` 进行校验。

**安装包。** 同一页面还提供了 `.deb` 和 `.rpm` 安装包，它们还会一并安装菜单项、图标和 man 手册页：

```bash
sudo apt install ./xylem-fm_*_amd64.deb
sudo dnf install ./xylem-fm-*.x86_64.rpm
```

由于安装包并非来自软件源，因此不会自动更新。安装新版本请按同样方式操作。

**Cargo。** 需要 Rust 和 C 编译器（用于 SQLite 和 zstd）。crate 名称为 `xylem-fm`；安装的命令为 `xy`：

```bash
cargo install xylem-fm
```

## 启动

```bash
xy
```

打开当前目录。`xy ~/projects` 打开其他目录，`xy user@host:` 打开[远程主机](remote.md)。[命令行](cli.md)列出了其他启动形式。

首次启动时，Xylem 会询问是否可以每天检查一次新版本：按 **Y** 或 **N**（参见[新版本](configuration.md#new-versions)）。

## 在目录树中移动

目录树显示起始目录及其子目录。此时仅起始目录已被读取。

| 按键 | 功能 |
|---|---|
| ↑ ↓ | 移动光标 |
| → | 读取光标下的目录并展示其中的内容 |
| ← | 折叠该目录 |
| `*` | 读取该目录及其下方的全部内容 |

下方窗口列出光标所在目录中的文件。在尚未读取的目录上，显示 `Not logged`。

## 进入文件窗口

**Enter** 切换到文件窗口，此时文件窗口将占据整个屏幕的高度。按 Enter 或 Esc 返回目录树。

| 按键 | 功能 |
|---|---|
| `V` | 查看文件 |
| `E` | 在编辑器中打开文件 |
| `C`、`M`、`D`、`R` | 复制、移动、删除、重命名文件 |

## 标记文件并复制

1. 在文件窗口中，对需要的每个文件按 `T`。文件名左侧会出现 `■`，光标自动下移。
2. 按 Ctrl+C。提示行询问复制到何处。
3. 输入目标目录并按 Enter。

已标记的文件即被复制。按 `U` 取消单个文件的标记，按 Ctrl+U 取消所有标记。

## 退出

`Q` 询问确认后退出。如果已安装[退出](quit.md)中介绍的 shell 函数，按 Alt+Q 退出时会将 shell 保留在退出前所在的目录中。

## 下一步

- [屏幕](screen.md)介绍了屏幕上各个区域的名称。
- [菜单](menus.md)解释了字母单独按下、配合 Ctrl 以及配合 Alt 的不同含义。
- [术语](glossary.md)定义了 Log、Branch、Tag 等核心概念。
- **F1** 打开当前屏幕所处状态的相应帮助。
